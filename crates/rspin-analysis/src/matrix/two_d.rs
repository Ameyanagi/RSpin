//! Matrix generation for multiple two-dimensional spectra.

use serde::{Deserialize, Serialize};

use rspin_core::{Axis, RSpinError, Result, Spectrum2D};

use super::{point_in_segment, spectrum_id, validate_axis};

/// Options for generating a 2D spectrum matrix.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MatrixGeneration2DOptions {
    /// Target x axis. When omitted, the first spectrum x axis is used.
    pub target_x_axis: Option<Axis>,
    /// Target y axis. When omitted, the first spectrum y axis is used.
    pub target_y_axis: Option<Axis>,
    /// Value used when a target coordinate lies outside a spectrum domain.
    pub outside_value: f64,
}

impl Default for MatrixGeneration2DOptions {
    fn default() -> Self {
        Self {
            target_x_axis: None,
            target_y_axis: None,
            outside_value: 0.0,
        }
    }
}

impl MatrixGeneration2DOptions {
    /// Creates default matrix generation options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets both target axes used for every generated spectrum layer.
    #[must_use]
    pub fn with_target_axes(mut self, x_axis: Axis, y_axis: Axis) -> Self {
        self.target_x_axis = Some(x_axis);
        self.target_y_axis = Some(y_axis);
        self
    }

    /// Sets the target x axis used for every generated spectrum layer.
    #[must_use]
    pub fn with_target_x_axis(mut self, axis: Axis) -> Self {
        self.target_x_axis = Some(axis);
        self
    }

    /// Sets the target y axis used for every generated spectrum layer.
    #[must_use]
    pub fn with_target_y_axis(mut self, axis: Axis) -> Self {
        self.target_y_axis = Some(axis);
        self
    }

    /// Uses the first input spectrum axes as target axes.
    #[must_use]
    pub fn without_target_axes(mut self) -> Self {
        self.target_x_axis = None;
        self.target_y_axis = None;
        self
    }

    /// Sets the value used outside each source spectrum domain.
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
        if let Some(axis) = &self.target_x_axis {
            validate_axis(axis)?;
        }
        if let Some(axis) = &self.target_y_axis {
            validate_axis(axis)?;
        }
        Ok(())
    }
}

/// Layer-major matrix generated from two-dimensional spectra.
///
/// Values are ordered as `spectrum_ids.len() * y.len() * x.len()`. Each spectrum
/// layer is row-major in y/x order, matching `Spectrum2D::z`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpectrumMatrix2D {
    /// Common x axis for all layers.
    pub x: Axis,
    /// Common y axis for all layers.
    pub y: Axis,
    /// Deterministic spectrum layer identifiers.
    pub spectrum_ids: Vec<String>,
    /// Layer-major matrix values.
    pub values: Vec<f64>,
}

impl SpectrumMatrix2D {
    /// Returns the matrix shape as `(spectra, y, x)`.
    #[must_use]
    pub fn shape(&self) -> (usize, usize, usize) {
        (self.spectrum_ids.len(), self.y.len(), self.x.len())
    }

    /// Returns the matrix value at spectrum/x/y index, or `None` when out of bounds.
    #[must_use]
    pub fn value_at(&self, spectrum_index: usize, x_index: usize, y_index: usize) -> Option<f64> {
        let (spectra, height, width) = self.shape();
        if spectrum_index >= spectra || x_index >= width || y_index >= height {
            return None;
        }
        let layer_len = width.checked_mul(height)?;
        let layer_offset = spectrum_index.checked_mul(layer_len)?;
        let row_offset = y_index.checked_mul(width)?;
        self.values
            .get(layer_offset.checked_add(row_offset)?.checked_add(x_index)?)
            .copied()
    }
}

/// Generates a layer-major matrix from two-dimensional spectra.
///
/// Each spectrum is bilinearly interpolated onto the target x/y axes. Spectra
/// may use ascending or descending monotonic axes.
///
/// # Errors
///
/// Returns an error when no spectra are provided, axes are non-monotonic,
/// target options are invalid, or matrix dimensions overflow.
pub fn generate_spectrum_matrix_2d(
    spectra: &[Spectrum2D],
    options: MatrixGeneration2DOptions,
) -> Result<SpectrumMatrix2D> {
    options.validate()?;
    let outside_value = options.outside_value;
    let Some(first) = spectra.first() else {
        return Err(RSpinError::InvalidSpectrum {
            message: "2D spectrum matrix requires at least one spectrum".to_owned(),
        });
    };
    let x = match options.target_x_axis {
        Some(axis) => axis,
        None => first.x.clone(),
    };
    let y = match options.target_y_axis {
        Some(axis) => axis,
        None => first.y.clone(),
    };
    validate_axis(&x)?;
    validate_axis(&y)?;

    let layer_len = x
        .len()
        .checked_mul(y.len())
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "2D spectrum matrix layer size overflow".to_owned(),
        })?;
    let total_len =
        spectra
            .len()
            .checked_mul(layer_len)
            .ok_or_else(|| RSpinError::InvalidSpectrum {
                message: "2D spectrum matrix size overflow".to_owned(),
            })?;
    let mut spectrum_ids = Vec::with_capacity(spectra.len());
    let mut values = Vec::with_capacity(total_len);

    for (index, spectrum) in spectra.iter().enumerate() {
        validate_axis(&spectrum.x)?;
        validate_axis(&spectrum.y)?;
        spectrum_ids.push(spectrum_id(index, spectrum.metadata.name.as_deref()));
        for y_value in y.values.iter().copied() {
            values.extend(
                x.values
                    .iter()
                    .copied()
                    .map(|x_value| interpolate_2d(spectrum, x_value, y_value, outside_value)),
            );
        }
    }

    Ok(SpectrumMatrix2D {
        x,
        y,
        spectrum_ids,
        values,
    })
}

fn interpolate_2d(spectrum: &Spectrum2D, x: f64, y: f64, outside_value: f64) -> f64 {
    let Some(x_interval) = interval(&spectrum.x.values, x) else {
        return outside_value;
    };
    let Some(y_interval) = interval(&spectrum.y.values, y) else {
        return outside_value;
    };

    let Some(z00) = grid_value(spectrum, x_interval.lower_index, y_interval.lower_index) else {
        return outside_value;
    };
    let Some(z10) = grid_value(spectrum, x_interval.upper_index, y_interval.lower_index) else {
        return outside_value;
    };
    let Some(z01) = grid_value(spectrum, x_interval.lower_index, y_interval.upper_index) else {
        return outside_value;
    };
    let Some(z11) = grid_value(spectrum, x_interval.upper_index, y_interval.upper_index) else {
        return outside_value;
    };

    let lower_y = z00 + x_interval.fraction * (z10 - z00);
    let upper_y = z01 + x_interval.fraction * (z11 - z01);
    lower_y + y_interval.fraction * (upper_y - lower_y)
}

fn grid_value(spectrum: &Spectrum2D, x_index: usize, y_index: usize) -> Option<f64> {
    let width = spectrum.x.len();
    spectrum
        .z
        .get(y_index.checked_mul(width)?.checked_add(x_index)?)
        .copied()
}

fn interval(values: &[f64], value: f64) -> Option<InterpolationInterval> {
    if values.len() == 1 {
        return if (values[0] - value).abs() <= f64::EPSILON {
            Some(InterpolationInterval {
                lower_index: 0,
                upper_index: 0,
                fraction: 0.0,
            })
        } else {
            None
        };
    }

    for (index, pair) in values.windows(2).enumerate() {
        let lower = pair[0];
        let upper = pair[1];
        if point_in_segment(value, lower, upper) {
            let fraction = if (upper - lower).abs() <= f64::EPSILON {
                0.0
            } else {
                (value - lower) / (upper - lower)
            };
            return Some(InterpolationInterval {
                lower_index: index,
                upper_index: index + 1,
                fraction,
            });
        }
    }

    None
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct InterpolationInterval {
    lower_index: usize,
    upper_index: usize,
    fraction: f64,
}

#[cfg(test)]
mod tests;
