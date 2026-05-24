//! Matrix generation for multi-spectrum analysis.

use serde::{Deserialize, Serialize};

use rspin_core::{Axis, RSpinError, Result, Spectrum1D};

/// Options for generating a 1D spectrum matrix.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MatrixGenerationOptions {
    /// Target axis. When omitted, the first spectrum axis is used.
    pub target_axis: Option<Axis>,
    /// Value used when a target coordinate lies outside a spectrum axis domain.
    pub outside_value: f64,
}

impl Default for MatrixGenerationOptions {
    fn default() -> Self {
        Self {
            target_axis: None,
            outside_value: 0.0,
        }
    }
}

impl MatrixGenerationOptions {
    /// Creates default matrix generation options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the target axis used for every generated row.
    #[must_use]
    pub fn with_target_axis(mut self, axis: Axis) -> Self {
        self.target_axis = Some(axis);
        self
    }

    /// Uses the first input spectrum axis as the target axis.
    #[must_use]
    pub fn without_target_axis(mut self) -> Self {
        self.target_axis = None;
        self
    }

    /// Sets the value used outside each source spectrum axis domain.
    #[must_use]
    pub fn with_outside_value(mut self, outside_value: f64) -> Self {
        self.outside_value = outside_value;
        self
    }

    fn validate(&self) -> Result<()> {
        if !self.outside_value.is_finite() {
            return Err(RSpinError::NonFinite {
                field: "outside_value",
            });
        }
        if let Some(axis) = &self.target_axis {
            validate_axis(axis)?;
        }
        Ok(())
    }
}

/// Row-major matrix generated from one-dimensional spectra.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpectrumMatrix1D {
    /// Common x axis for all rows.
    pub axis: Axis,
    /// Deterministic row identifiers.
    pub row_ids: Vec<String>,
    /// Row-major matrix values: `row_ids.len() * axis.len()`.
    pub values: Vec<f64>,
}

impl SpectrumMatrix1D {
    /// Returns the matrix shape as `(rows, columns)`.
    #[must_use]
    pub fn shape(&self) -> (usize, usize) {
        (self.row_ids.len(), self.axis.len())
    }

    /// Returns the matrix value at row/column, or `None` when out of bounds.
    #[must_use]
    pub fn value_at(&self, row_index: usize, column_index: usize) -> Option<f64> {
        let (rows, columns) = self.shape();
        if row_index >= rows || column_index >= columns {
            return None;
        }
        self.values.get(row_index * columns + column_index).copied()
    }
}

/// Generates a row-major matrix from one-dimensional spectra.
///
/// Each spectrum is linearly interpolated onto the target axis. Spectra may use
/// ascending or descending monotonic axes.
///
/// # Errors
///
/// Returns an error when no spectra are provided, axes are non-monotonic,
/// target options are invalid, or matrix dimensions overflow.
pub fn generate_spectrum_matrix_1d(
    spectra: &[Spectrum1D],
    options: MatrixGenerationOptions,
) -> Result<SpectrumMatrix1D> {
    options.validate()?;
    let outside_value = options.outside_value;
    let axis = target_axis(spectra, options.target_axis)?;
    validate_axis(&axis)?;

    let target_len = axis.len();
    let total_len =
        spectra
            .len()
            .checked_mul(target_len)
            .ok_or_else(|| RSpinError::InvalidSpectrum {
                message: "spectrum matrix size overflow".to_owned(),
            })?;
    let mut row_ids = Vec::with_capacity(spectra.len());
    let mut values = Vec::with_capacity(total_len);

    for (index, spectrum) in spectra.iter().enumerate() {
        validate_axis(&spectrum.x)?;
        row_ids.push(row_id(index, spectrum));
        values.extend(
            axis.values
                .iter()
                .copied()
                .map(|x| interpolate(spectrum, x, outside_value)),
        );
    }

    Ok(SpectrumMatrix1D {
        axis,
        row_ids,
        values,
    })
}

fn target_axis(spectra: &[Spectrum1D], configured_axis: Option<Axis>) -> Result<Axis> {
    if let Some(axis) = configured_axis {
        return Ok(axis);
    }
    spectra
        .first()
        .map(|spectrum| spectrum.x.clone())
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "spectrum matrix requires at least one spectrum".to_owned(),
        })
}

fn row_id(index: usize, spectrum: &Spectrum1D) -> String {
    match spectrum.metadata.name.as_deref() {
        Some(name) if !name.trim().is_empty() => format!("{index}:{}", sanitize_id_token(name)),
        _ => format!("spectrum-{index}"),
    }
}

fn interpolate(spectrum: &Spectrum1D, x: f64, outside_value: f64) -> f64 {
    let values = &spectrum.x.values;
    let intensities = &spectrum.intensities;
    if values.len() == 1 {
        return if (values[0] - x).abs() <= f64::EPSILON {
            intensities[0]
        } else {
            outside_value
        };
    }

    for (index, pair) in values.windows(2).enumerate() {
        let x0 = pair[0];
        let x1 = pair[1];
        if point_in_segment(x, x0, x1) {
            if (x1 - x0).abs() <= f64::EPSILON {
                return intensities[index];
            }
            let fraction = (x - x0) / (x1 - x0);
            return intensities[index] + fraction * (intensities[index + 1] - intensities[index]);
        }
    }

    outside_value
}

fn point_in_segment(x: f64, x0: f64, x1: f64) -> bool {
    if x0 <= x1 {
        x >= x0 && x <= x1
    } else {
        x <= x0 && x >= x1
    }
}

fn validate_axis(axis: &Axis) -> Result<()> {
    if axis.is_empty() {
        return Err(RSpinError::InvalidAxis {
            message: "matrix axis must contain at least one point".to_owned(),
        });
    }
    if !axis.values.iter().all(|value| value.is_finite()) {
        return Err(RSpinError::NonFinite { field: "axis" });
    }
    if axis.len() > 1 && !is_strictly_monotonic(&axis.values) {
        return Err(RSpinError::InvalidAxis {
            message: "matrix axes must be strictly monotonic".to_owned(),
        });
    }
    Ok(())
}

fn is_strictly_monotonic(values: &[f64]) -> bool {
    let ascending = values.windows(2).all(|pair| pair[0] < pair[1]);
    let descending = values.windows(2).all(|pair| pair[0] > pair[1]);
    ascending || descending
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

#[cfg(test)]
mod tests;
