//! Two-dimensional apodization.

use std::f64::consts::PI;

use rspin_core::{ProcessingRecord, RSpinError, Result, Spectrum2D};

use crate::ProcessingStep;

/// Applies separable exponential apodization to a two-dimensional spectrum.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExponentialApodization2D {
    /// X-dimension line broadening in hertz.
    pub x_line_broadening_hz: f64,
    /// Y-dimension line broadening in hertz.
    pub y_line_broadening_hz: f64,
    /// X-dimension dwell time in seconds.
    pub x_dwell_time_s: f64,
    /// Y-dimension dwell time in seconds.
    pub y_dwell_time_s: f64,
}

impl ProcessingStep<Spectrum2D> for ExponentialApodization2D {
    fn apply(&self, spectrum: &Spectrum2D) -> Result<Spectrum2D> {
        exponential_apodization_2d(
            spectrum,
            self.x_line_broadening_hz,
            self.y_line_broadening_hz,
            self.x_dwell_time_s,
            self.y_dwell_time_s,
        )
    }
}

/// Applies a separable exponential window in x and y.
///
/// The multiplier at `(x, y)` is
/// `exp(-pi * x_line_broadening_hz * x_dwell_time_s * x_index) *
/// exp(-pi * y_line_broadening_hz * y_dwell_time_s * y_index)`.
///
/// # Errors
///
/// Returns an error when line broadening is negative or any parameter is
/// non-finite. Dwell times must be positive.
pub fn exponential_apodization_2d(
    spectrum: &Spectrum2D,
    x_line_broadening_hz: f64,
    y_line_broadening_hz: f64,
    x_dwell_time_s: f64,
    y_dwell_time_s: f64,
) -> Result<Spectrum2D> {
    ensure_non_negative("x_line_broadening_hz", x_line_broadening_hz)?;
    ensure_non_negative("y_line_broadening_hz", y_line_broadening_hz)?;
    ensure_positive("x_dwell_time_s", x_dwell_time_s)?;
    ensure_positive("y_dwell_time_s", y_dwell_time_s)?;

    let (width, height) = spectrum.shape();
    let x_decay = (-PI * x_line_broadening_hz * x_dwell_time_s).exp();
    let y_decay = (-PI * y_line_broadening_hz * y_dwell_time_s).exp();
    let x_weights = exponential_weights(width, x_decay);
    let y_weights = exponential_weights(height, y_decay);

    let mut processed = spectrum.clone();
    for (y_index, y_weight) in y_weights.iter().copied().enumerate() {
        let row_start = y_index * width;
        for (x_index, x_weight) in x_weights.iter().copied().enumerate() {
            processed.z[row_start + x_index] *= x_weight * y_weight;
            if let Some(imaginary) = &mut processed.imaginary {
                imaginary[row_start + x_index] *= x_weight * y_weight;
            }
        }
    }

    Ok(processed.with_processing_record(
        ProcessingRecord::new("exponential_apodization_2d").with_details(format!(
            "x_line_broadening_hz={x_line_broadening_hz},y_line_broadening_hz={y_line_broadening_hz},x_dwell_time_s={x_dwell_time_s},y_dwell_time_s={y_dwell_time_s}"
        )),
    ))
}

fn exponential_weights(len: usize, decay: f64) -> Vec<f64> {
    let mut weights = Vec::with_capacity(len);
    let mut weight = 1.0;
    for _ in 0..len {
        weights.push(weight);
        weight *= decay;
    }
    weights
}

fn ensure_non_negative(field: &'static str, value: f64) -> Result<()> {
    ensure_finite(field, value)?;
    if value < 0.0 {
        return Err(RSpinError::InvalidSpectrum {
            message: format!("{field} must be non-negative"),
        });
    }
    Ok(())
}

fn ensure_positive(field: &'static str, value: f64) -> Result<()> {
    ensure_finite(field, value)?;
    if value <= 0.0 {
        return Err(RSpinError::InvalidSpectrum {
            message: format!("{field} must be positive"),
        });
    }
    Ok(())
}

fn ensure_finite(field: &'static str, value: f64) -> Result<()> {
    if !value.is_finite() {
        return Err(RSpinError::NonFinite { field });
    }
    Ok(())
}

#[cfg(test)]
mod tests;
