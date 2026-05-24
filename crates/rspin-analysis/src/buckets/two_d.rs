//! Two-dimensional spectral bucketing.

use serde::{Deserialize, Serialize};

use rspin_core::{RSpinError, Result, Spectrum2D};

use crate::{IntegralRegion2D, integrate_region_2d};

/// Options for equal-size rectangular two-dimensional spectral buckets.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct BucketOptions2D {
    /// First x edge of the bucket range.
    pub x_from: f64,
    /// Final x edge of the bucket range.
    pub x_to: f64,
    /// First y edge of the bucket range.
    pub y_from: f64,
    /// Final y edge of the bucket range.
    pub y_to: f64,
    /// Number of equal-width buckets along x.
    pub x_bucket_count: usize,
    /// Number of equal-width buckets along y.
    pub y_bucket_count: usize,
}

impl BucketOptions2D {
    /// Creates equal-size rectangular bucket options.
    #[must_use]
    pub fn new(
        x_from: f64,
        x_to: f64,
        y_from: f64,
        y_to: f64,
        x_bucket_count: usize,
        y_bucket_count: usize,
    ) -> Self {
        Self {
            x_from,
            x_to,
            y_from,
            y_to,
            x_bucket_count,
            y_bucket_count,
        }
    }

    /// Sets the x bucket range.
    #[must_use]
    pub fn with_x_range(mut self, from: f64, to: f64) -> Self {
        self.x_from = from;
        self.x_to = to;
        self
    }

    /// Sets the y bucket range.
    #[must_use]
    pub fn with_y_range(mut self, from: f64, to: f64) -> Self {
        self.y_from = from;
        self.y_to = to;
        self
    }

    /// Sets both bucket counts.
    #[must_use]
    pub fn with_bucket_counts(mut self, x_bucket_count: usize, y_bucket_count: usize) -> Self {
        self.x_bucket_count = x_bucket_count;
        self.y_bucket_count = y_bucket_count;
        self
    }

    /// Sets the number of buckets along x.
    #[must_use]
    pub fn with_x_bucket_count(mut self, bucket_count: usize) -> Self {
        self.x_bucket_count = bucket_count;
        self
    }

    /// Sets the number of buckets along y.
    #[must_use]
    pub fn with_y_bucket_count(mut self, bucket_count: usize) -> Self {
        self.y_bucket_count = bucket_count;
        self
    }

    /// Returns rectangular bucket regions in row-major y/x order.
    ///
    /// # Errors
    ///
    /// Returns an error when either range or bucket count is invalid.
    pub fn regions(self) -> Result<Vec<IntegralRegion2D>> {
        self.validate()?;
        let x_regions = axis_regions(
            "x_bucket_count",
            self.x_from,
            self.x_to,
            self.x_bucket_count,
        )?;
        let y_regions = axis_regions(
            "y_bucket_count",
            self.y_from,
            self.y_to,
            self.y_bucket_count,
        )?;
        let total_len = x_regions
            .len()
            .checked_mul(y_regions.len())
            .ok_or_else(|| RSpinError::InvalidSpectrum {
                message: "2D bucket region size overflow".to_owned(),
            })?;
        let mut regions = Vec::with_capacity(total_len);
        for (y_from, y_to) in y_regions {
            regions.extend(x_regions.iter().map(|(x_from, x_to)| IntegralRegion2D {
                x_from: *x_from,
                x_to: *x_to,
                y_from,
                y_to,
            }));
        }
        Ok(regions)
    }

    fn validate(self) -> Result<()> {
        ensure_finite("bucket x_from", self.x_from)?;
        ensure_finite("bucket x_to", self.x_to)?;
        ensure_finite("bucket y_from", self.y_from)?;
        ensure_finite("bucket y_to", self.y_to)?;
        ensure_positive_count("x_bucket_count", self.x_bucket_count)?;
        ensure_positive_count("y_bucket_count", self.y_bucket_count)?;
        ensure_non_zero_width("x bucket range", self.x_to - self.x_from)?;
        ensure_non_zero_width("y bucket range", self.y_to - self.y_from)?;
        Ok(())
    }
}

/// Integrated value for one rectangular two-dimensional spectral bucket.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpectralBucket2D {
    /// Linear bucket index in row-major y/x order.
    pub index: usize,
    /// Bucket x index.
    pub x_index: usize,
    /// Bucket y index.
    pub y_index: usize,
    /// Bucket integration region.
    pub region: IntegralRegion2D,
    /// Bilinear bucket volume.
    pub volume: f64,
    /// Number of spectrum cells contributing to the bucket.
    pub cells: usize,
}

/// Layer-major bucket matrix generated from two-dimensional spectra.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BucketMatrix2D {
    /// Bucket regions in row-major y/x order.
    pub regions: Vec<IntegralRegion2D>,
    /// Number of x buckets.
    pub x_bucket_count: usize,
    /// Number of y buckets.
    pub y_bucket_count: usize,
    /// Deterministic layer identifiers.
    pub layer_ids: Vec<String>,
    /// Layer-major bucket volumes: `layers * y_bucket_count * x_bucket_count`.
    pub values: Vec<f64>,
}

impl BucketMatrix2D {
    /// Returns the matrix shape as `(layers, y_buckets, x_buckets)`.
    #[must_use]
    pub fn shape(&self) -> (usize, usize, usize) {
        (
            self.layer_ids.len(),
            self.y_bucket_count,
            self.x_bucket_count,
        )
    }

    /// Returns one matrix value, or `None` when out of bounds.
    #[must_use]
    pub fn value_at(&self, layer_index: usize, y_index: usize, x_index: usize) -> Option<f64> {
        let (layers, y_buckets, x_buckets) = self.shape();
        if layer_index >= layers || y_index >= y_buckets || x_index >= x_buckets {
            return None;
        }
        let layer_stride = y_buckets.checked_mul(x_buckets)?;
        let layer_offset = layer_index.checked_mul(layer_stride)?;
        let row_offset = y_index.checked_mul(x_buckets)?;
        let value_index = layer_offset.checked_add(row_offset)?.checked_add(x_index)?;
        self.values.get(value_index).copied()
    }
}

/// Integrates one two-dimensional spectrum into equal-size rectangular buckets.
///
/// # Errors
///
/// Returns an error when options are invalid or integration fails.
pub fn bucket_spectrum_2d(
    spectrum: &Spectrum2D,
    options: BucketOptions2D,
) -> Result<Vec<SpectralBucket2D>> {
    let x_bucket_count = options.x_bucket_count;
    options
        .regions()?
        .into_iter()
        .enumerate()
        .map(|(index, region)| {
            let integral = integrate_region_2d(spectrum, region)?;
            Ok(SpectralBucket2D {
                index,
                x_index: index % x_bucket_count,
                y_index: index / x_bucket_count,
                region,
                volume: integral.volume,
                cells: integral.cells,
            })
        })
        .collect()
}

/// Integrates spectra into a common equal-size rectangular bucket matrix.
///
/// # Errors
///
/// Returns an error when no spectra are provided, options are invalid, or
/// integration fails.
pub fn bucket_spectra_2d(
    spectra: &[Spectrum2D],
    options: BucketOptions2D,
) -> Result<BucketMatrix2D> {
    if spectra.is_empty() {
        return Err(RSpinError::InvalidSpectrum {
            message: "2D bucket matrix requires at least one spectrum".to_owned(),
        });
    }
    let regions = options.regions()?;
    let bucket_count = options
        .x_bucket_count
        .checked_mul(options.y_bucket_count)
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "2D bucket matrix size overflow".to_owned(),
        })?;
    let total_len =
        spectra
            .len()
            .checked_mul(bucket_count)
            .ok_or_else(|| RSpinError::InvalidSpectrum {
                message: "2D bucket matrix size overflow".to_owned(),
            })?;
    let mut layer_ids = Vec::with_capacity(spectra.len());
    let mut values = Vec::with_capacity(total_len);

    for (layer_index, spectrum) in spectra.iter().enumerate() {
        layer_ids.push(layer_id(layer_index, spectrum));
        for region in &regions {
            values.push(integrate_region_2d(spectrum, *region)?.volume);
        }
    }

    Ok(BucketMatrix2D {
        regions,
        x_bucket_count: options.x_bucket_count,
        y_bucket_count: options.y_bucket_count,
        layer_ids,
        values,
    })
}

fn axis_regions(
    count_field: &'static str,
    from: f64,
    to: f64,
    bucket_count: usize,
) -> Result<Vec<(f64, f64)>> {
    let bucket_count_u32 =
        u32::try_from(bucket_count).map_err(|_| RSpinError::InvalidSpectrum {
            message: format!("{count_field} is too large"),
        })?;
    let step = (to - from) / f64::from(bucket_count_u32);
    let mut next_from = from;
    Ok((0..bucket_count)
        .map(|index| {
            let region_from = next_from;
            let region_to = if index + 1 == bucket_count {
                to
            } else {
                region_from + step
            };
            next_from = region_to;
            (region_from, region_to)
        })
        .collect())
}

fn layer_id(index: usize, spectrum: &Spectrum2D) -> String {
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

fn ensure_positive_count(field: &'static str, value: usize) -> Result<()> {
    if value == 0 {
        return Err(RSpinError::InvalidSpectrum {
            message: format!("{field} must be positive"),
        });
    }
    Ok(())
}

fn ensure_non_zero_width(field: &'static str, width: f64) -> Result<()> {
    if width.abs() <= f64::EPSILON {
        return Err(RSpinError::InvalidSpectrum {
            message: format!("{field} must have non-zero width"),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests;
