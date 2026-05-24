//! Two-dimensional apodization.

use std::f64::consts::{LN_2, PI};

use rspin_core::{ProcessingRecord, RSpinError, Result, Spectrum2D};
use serde::{Deserialize, Serialize};

use crate::ProcessingStep;

/// Applies separable exponential apodization to a two-dimensional spectrum.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
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

impl ExponentialApodization2D {
    /// Creates a separable exponential apodization step.
    #[must_use]
    pub fn new(
        x_line_broadening_hz: f64,
        y_line_broadening_hz: f64,
        x_dwell_time_s: f64,
        y_dwell_time_s: f64,
    ) -> Self {
        Self {
            x_line_broadening_hz,
            y_line_broadening_hz,
            x_dwell_time_s,
            y_dwell_time_s,
        }
    }
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

/// Applies separable Gaussian apodization to a two-dimensional spectrum.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GaussianApodization2D {
    /// X-dimension Gaussian broadening full width at half maximum in hertz.
    pub x_gaussian_broadening_hz: f64,
    /// Y-dimension Gaussian broadening full width at half maximum in hertz.
    pub y_gaussian_broadening_hz: f64,
    /// X-dimension dwell time in seconds.
    pub x_dwell_time_s: f64,
    /// Y-dimension dwell time in seconds.
    pub y_dwell_time_s: f64,
}

impl GaussianApodization2D {
    /// Creates a separable Gaussian apodization step.
    #[must_use]
    pub fn new(
        x_gaussian_broadening_hz: f64,
        y_gaussian_broadening_hz: f64,
        x_dwell_time_s: f64,
        y_dwell_time_s: f64,
    ) -> Self {
        Self {
            x_gaussian_broadening_hz,
            y_gaussian_broadening_hz,
            x_dwell_time_s,
            y_dwell_time_s,
        }
    }
}

impl ProcessingStep<Spectrum2D> for GaussianApodization2D {
    fn apply(&self, spectrum: &Spectrum2D) -> Result<Spectrum2D> {
        gaussian_apodization_2d(
            spectrum,
            self.x_gaussian_broadening_hz,
            self.y_gaussian_broadening_hz,
            self.x_dwell_time_s,
            self.y_dwell_time_s,
        )
    }
}

/// Applies separable sine-bell apodization to a two-dimensional spectrum.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SineBellApodization2D {
    /// X-dimension start angle in degrees.
    pub x_start_angle_deg: f64,
    /// X-dimension end angle in degrees.
    pub x_end_angle_deg: f64,
    /// X-dimension positive exponent.
    pub x_exponent: f64,
    /// Y-dimension start angle in degrees.
    pub y_start_angle_deg: f64,
    /// Y-dimension end angle in degrees.
    pub y_end_angle_deg: f64,
    /// Y-dimension positive exponent.
    pub y_exponent: f64,
}

impl SineBellApodization2D {
    /// Creates a separable sine-bell apodization step.
    #[must_use]
    pub fn new(
        x_start_angle_deg: f64,
        x_end_angle_deg: f64,
        x_exponent: f64,
        y_start_angle_deg: f64,
        y_end_angle_deg: f64,
        y_exponent: f64,
    ) -> Self {
        Self {
            x_start_angle_deg,
            x_end_angle_deg,
            x_exponent,
            y_start_angle_deg,
            y_end_angle_deg,
            y_exponent,
        }
    }
}

impl ProcessingStep<Spectrum2D> for SineBellApodization2D {
    fn apply(&self, spectrum: &Spectrum2D) -> Result<Spectrum2D> {
        sine_bell_apodization_2d(
            spectrum,
            self.x_start_angle_deg,
            self.x_end_angle_deg,
            self.x_exponent,
            self.y_start_angle_deg,
            self.y_end_angle_deg,
            self.y_exponent,
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

/// Applies a separable Gaussian window in x and y.
///
/// The multiplier at `(x, y)` is the product of the x and y Gaussian
/// multipliers, where each dimension uses
/// `exp(-(pi * gaussian_broadening_hz * dwell_time_s * index)^2 / (4 * ln(2)))`.
///
/// # Errors
///
/// Returns an error when broadening is negative, dwell times are not positive,
/// any parameter is non-finite, or the shape is too large for checked numeric
/// conversion.
pub fn gaussian_apodization_2d(
    spectrum: &Spectrum2D,
    x_gaussian_broadening_hz: f64,
    y_gaussian_broadening_hz: f64,
    x_dwell_time_s: f64,
    y_dwell_time_s: f64,
) -> Result<Spectrum2D> {
    ensure_non_negative("x_gaussian_broadening_hz", x_gaussian_broadening_hz)?;
    ensure_non_negative("y_gaussian_broadening_hz", y_gaussian_broadening_hz)?;
    ensure_positive("x_dwell_time_s", x_dwell_time_s)?;
    ensure_positive("y_dwell_time_s", y_dwell_time_s)?;

    let (width, height) = spectrum.shape();
    let x_weights = gaussian_weights(
        width,
        x_gaussian_broadening_hz,
        x_dwell_time_s,
        "2D x Gaussian apodization",
    )?;
    let y_weights = gaussian_weights(
        height,
        y_gaussian_broadening_hz,
        y_dwell_time_s,
        "2D y Gaussian apodization",
    )?;

    let mut processed = spectrum.clone();
    for (y_index, y_weight) in y_weights.iter().copied().enumerate() {
        let row_start = y_index * width;
        for (x_index, x_weight) in x_weights.iter().copied().enumerate() {
            let weight = x_weight * y_weight;
            processed.z[row_start + x_index] *= weight;
            if let Some(imaginary) = &mut processed.imaginary {
                imaginary[row_start + x_index] *= weight;
            }
        }
    }

    Ok(processed.with_processing_record(
        ProcessingRecord::new("gaussian_apodization_2d").with_details(format!(
            "x_gaussian_broadening_hz={x_gaussian_broadening_hz},y_gaussian_broadening_hz={y_gaussian_broadening_hz},x_dwell_time_s={x_dwell_time_s},y_dwell_time_s={y_dwell_time_s}"
        )),
    ))
}

/// Applies a separable sine-bell window in x and y.
///
/// Each dimension uses `sin(theta_i)^exponent`, where `theta_i` moves linearly
/// from the configured start angle to end angle. Angles are constrained to
/// `0..=180` degrees so weights remain non-negative.
///
/// # Errors
///
/// Returns an error when an angle is outside `0..=180` degrees, an exponent is
/// not positive, any parameter is non-finite, or the shape is too large for
/// checked numeric conversion.
pub fn sine_bell_apodization_2d(
    spectrum: &Spectrum2D,
    x_start_angle_deg: f64,
    x_end_angle_deg: f64,
    x_exponent: f64,
    y_start_angle_deg: f64,
    y_end_angle_deg: f64,
    y_exponent: f64,
) -> Result<Spectrum2D> {
    let (width, height) = spectrum.shape();
    let x_weights = sine_bell_weights(
        width,
        x_start_angle_deg,
        x_end_angle_deg,
        x_exponent,
        "2D x sine-bell apodization",
    )?;
    let y_weights = sine_bell_weights(
        height,
        y_start_angle_deg,
        y_end_angle_deg,
        y_exponent,
        "2D y sine-bell apodization",
    )?;

    let mut processed = spectrum.clone();
    for (y_index, y_weight) in y_weights.iter().copied().enumerate() {
        let row_start = y_index * width;
        for (x_index, x_weight) in x_weights.iter().copied().enumerate() {
            let weight = x_weight * y_weight;
            processed.z[row_start + x_index] *= weight;
            if let Some(imaginary) = &mut processed.imaginary {
                imaginary[row_start + x_index] *= weight;
            }
        }
    }

    Ok(processed.with_processing_record(
        ProcessingRecord::new("sine_bell_apodization_2d").with_details(format!(
            "x_start_angle_deg={x_start_angle_deg},x_end_angle_deg={x_end_angle_deg},x_exponent={x_exponent},y_start_angle_deg={y_start_angle_deg},y_end_angle_deg={y_end_angle_deg},y_exponent={y_exponent}"
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

fn gaussian_weights(
    len: usize,
    gaussian_broadening_hz: f64,
    dwell_time_s: f64,
    context: &'static str,
) -> Result<Vec<f64>> {
    let scale = PI * gaussian_broadening_hz * dwell_time_s;
    let denominator = 4.0 * LN_2;
    (0..len)
        .map(|index| {
            let index =
                f64::from(
                    u32::try_from(index).map_err(|_| RSpinError::InvalidSpectrum {
                        message: format!("{context} input is too large"),
                    })?,
                );
            let scaled = scale * index;
            Ok((-(scaled * scaled) / denominator).exp())
        })
        .collect()
}

fn sine_bell_weights(
    len: usize,
    start_angle_deg: f64,
    end_angle_deg: f64,
    exponent: f64,
    context: &'static str,
) -> Result<Vec<f64>> {
    ensure_angle_degrees("start_angle_deg", start_angle_deg)?;
    ensure_angle_degrees("end_angle_deg", end_angle_deg)?;
    ensure_positive("exponent", exponent)?;
    let denominator = if len <= 1 {
        0.0
    } else {
        f64::from(
            u32::try_from(len - 1).map_err(|_| RSpinError::InvalidSpectrum {
                message: format!("{context} input is too large"),
            })?,
        )
    };

    (0..len)
        .map(|index| {
            let index =
                f64::from(
                    u32::try_from(index).map_err(|_| RSpinError::InvalidSpectrum {
                        message: format!("{context} input is too large"),
                    })?,
                );
            let fraction = if denominator == 0.0 {
                0.0
            } else {
                index / denominator
            };
            let angle = start_angle_deg + (end_angle_deg - start_angle_deg) * fraction;
            Ok(angle.to_radians().sin().max(0.0).powf(exponent))
        })
        .collect()
}

fn ensure_angle_degrees(field: &'static str, value: f64) -> Result<()> {
    ensure_finite(field, value)?;
    if !(0.0..=180.0).contains(&value) {
        return Err(RSpinError::InvalidSpectrum {
            message: format!("{field} must be between 0 and 180 degrees"),
        });
    }
    Ok(())
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
