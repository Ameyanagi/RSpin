//! Spectral bucketing.

mod two_d;

use serde::{Deserialize, Serialize};

use rspin_core::{RSpinError, Result, Spectrum1D};

use crate::{IntegralRegion, integrate_region};

pub use two_d::{
    BucketMatrix2D, BucketOptions2D, SpectralBucket2D, bucket_spectra_2d, bucket_spectrum_2d,
};

/// Options for equal-width one-dimensional spectral buckets.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct BucketOptions1D {
    /// First edge of the bucket range.
    pub from: f64,
    /// Final edge of the bucket range.
    pub to: f64,
    /// Number of equal-width buckets.
    pub bucket_count: usize,
}

impl BucketOptions1D {
    /// Creates equal-width bucket options.
    #[must_use]
    pub fn new(from: f64, to: f64, bucket_count: usize) -> Self {
        Self {
            from,
            to,
            bucket_count,
        }
    }

    /// Sets the bucket range.
    #[must_use]
    pub fn with_range(mut self, from: f64, to: f64) -> Self {
        self.from = from;
        self.to = to;
        self
    }

    /// Sets the number of buckets.
    #[must_use]
    pub fn with_bucket_count(mut self, bucket_count: usize) -> Self {
        self.bucket_count = bucket_count;
        self
    }

    /// Returns bucket regions in requested coordinate order.
    ///
    /// # Errors
    ///
    /// Returns an error when the range or bucket count is invalid.
    pub fn regions(self) -> Result<Vec<IntegralRegion>> {
        self.validate()?;
        let bucket_count =
            u32::try_from(self.bucket_count).map_err(|_| RSpinError::InvalidSpectrum {
                message: "bucket_count is too large".to_owned(),
            })?;
        let step = (self.to - self.from) / f64::from(bucket_count);
        let mut next_from = self.from;
        Ok((0..self.bucket_count)
            .map(|index| {
                let from = next_from;
                let to = if index + 1 == self.bucket_count {
                    self.to
                } else {
                    from + step
                };
                next_from = to;
                IntegralRegion { from, to }
            })
            .collect())
    }

    fn validate(self) -> Result<()> {
        ensure_finite("bucket from", self.from)?;
        ensure_finite("bucket to", self.to)?;
        if self.bucket_count == 0 {
            return Err(RSpinError::InvalidSpectrum {
                message: "bucket_count must be positive".to_owned(),
            });
        }
        if (self.to - self.from).abs() <= f64::EPSILON {
            return Err(RSpinError::InvalidSpectrum {
                message: "bucket range must have non-zero width".to_owned(),
            });
        }
        Ok(())
    }
}

/// Integrated value for one spectral bucket.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpectralBucket1D {
    /// Bucket index in output order.
    pub index: usize,
    /// Bucket integration region.
    pub region: IntegralRegion,
    /// Trapezoidal bucket area.
    pub area: f64,
    /// Number of spectrum segments contributing to the bucket.
    pub segments: usize,
}

/// Row-major bucket matrix generated from one-dimensional spectra.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BucketMatrix1D {
    /// Bucket regions in column order.
    pub regions: Vec<IntegralRegion>,
    /// Deterministic row identifiers.
    pub row_ids: Vec<String>,
    /// Row-major bucket areas: `row_ids.len() * regions.len()`.
    pub values: Vec<f64>,
}

impl BucketMatrix1D {
    /// Returns the matrix shape as `(rows, columns)`.
    #[must_use]
    pub fn shape(&self) -> (usize, usize) {
        (self.row_ids.len(), self.regions.len())
    }

    /// Returns one matrix value, or `None` when out of bounds.
    #[must_use]
    pub fn value_at(&self, row_index: usize, column_index: usize) -> Option<f64> {
        let (rows, columns) = self.shape();
        if row_index >= rows || column_index >= columns {
            return None;
        }
        self.values.get(row_index * columns + column_index).copied()
    }
}

/// Integrates one spectrum into equal-width buckets.
///
/// # Errors
///
/// Returns an error when options are invalid or integration fails.
pub fn bucket_spectrum_1d(
    spectrum: &Spectrum1D,
    options: BucketOptions1D,
) -> Result<Vec<SpectralBucket1D>> {
    options
        .regions()?
        .into_iter()
        .enumerate()
        .map(|(index, region)| {
            let integral = integrate_region(spectrum, region)?;
            Ok(SpectralBucket1D {
                index,
                region,
                area: integral.area,
                segments: integral.segments,
            })
        })
        .collect()
}

/// Integrates spectra into a common equal-width bucket matrix.
///
/// # Errors
///
/// Returns an error when no spectra are provided, options are invalid, or
/// integration fails.
pub fn bucket_spectra_1d(
    spectra: &[Spectrum1D],
    options: BucketOptions1D,
) -> Result<BucketMatrix1D> {
    if spectra.is_empty() {
        return Err(RSpinError::InvalidSpectrum {
            message: "bucket matrix requires at least one spectrum".to_owned(),
        });
    }
    let regions = options.regions()?;
    let total_len =
        spectra
            .len()
            .checked_mul(regions.len())
            .ok_or_else(|| RSpinError::InvalidSpectrum {
                message: "bucket matrix size overflow".to_owned(),
            })?;
    let mut row_ids = Vec::with_capacity(spectra.len());
    let mut values = Vec::with_capacity(total_len);

    for (row_index, spectrum) in spectra.iter().enumerate() {
        row_ids.push(row_id(row_index, spectrum));
        for region in &regions {
            values.push(integrate_region(spectrum, *region)?.area);
        }
    }

    Ok(BucketMatrix1D {
        regions,
        row_ids,
        values,
    })
}

fn row_id(index: usize, spectrum: &Spectrum1D) -> String {
    match spectrum.metadata.name.as_deref() {
        Some(name) if !name.trim().is_empty() => format!("{index}:{}", sanitize_id_token(name)),
        _ => format!("spectrum-{index}"),
    }
}

fn sanitize_id_token(value: &str) -> String {
    value
        .trim()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.') {
                character
            } else {
                '_'
            }
        })
        .collect()
}

fn ensure_finite(field: &'static str, value: f64) -> Result<()> {
    if !value.is_finite() {
        return Err(RSpinError::NonFinite { field });
    }
    Ok(())
}

#[cfg(test)]
mod tests;
