//! Complex-domain one-dimensional processing.

use std::f64::consts::{LN_2, PI};

use rspin_core::{ProcessingRecord, RSpinError, Result, Spectrum1D};
use rustfft::{FftPlanner, num_complex::Complex};
use serde::{Deserialize, Serialize};

use crate::ProcessingStep;

/// Applies exponential apodization to real and imaginary channels.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExponentialApodization {
    /// Line broadening in hertz.
    pub line_broadening_hz: f64,
    /// Dwell time in seconds.
    pub dwell_time_s: f64,
}

impl ExponentialApodization {
    /// Creates an exponential apodization step.
    #[must_use]
    pub fn new(line_broadening_hz: f64, dwell_time_s: f64) -> Self {
        Self {
            line_broadening_hz,
            dwell_time_s,
        }
    }
}

impl ProcessingStep<Spectrum1D> for ExponentialApodization {
    fn apply(&self, spectrum: &Spectrum1D) -> Result<Spectrum1D> {
        exponential_apodization(spectrum, self.line_broadening_hz, self.dwell_time_s)
    }
}

/// Applies Gaussian apodization to real and imaginary channels.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GaussianApodization {
    /// Gaussian broadening full width at half maximum in hertz.
    pub gaussian_broadening_hz: f64,
    /// Dwell time in seconds.
    pub dwell_time_s: f64,
}

impl GaussianApodization {
    /// Creates a Gaussian apodization step.
    #[must_use]
    pub fn new(gaussian_broadening_hz: f64, dwell_time_s: f64) -> Self {
        Self {
            gaussian_broadening_hz,
            dwell_time_s,
        }
    }
}

impl ProcessingStep<Spectrum1D> for GaussianApodization {
    fn apply(&self, spectrum: &Spectrum1D) -> Result<Spectrum1D> {
        gaussian_apodization(spectrum, self.gaussian_broadening_hz, self.dwell_time_s)
    }
}

/// Applies sine-bell apodization to real and imaginary channels.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SineBellApodization {
    /// Start angle in degrees.
    pub start_angle_deg: f64,
    /// End angle in degrees.
    pub end_angle_deg: f64,
    /// Positive exponent applied to the sine-bell weights.
    pub exponent: f64,
}

impl SineBellApodization {
    /// Creates a sine-bell apodization step.
    #[must_use]
    pub fn new(start_angle_deg: f64, end_angle_deg: f64, exponent: f64) -> Self {
        Self {
            start_angle_deg,
            end_angle_deg,
            exponent,
        }
    }
}

impl ProcessingStep<Spectrum1D> for SineBellApodization {
    fn apply(&self, spectrum: &Spectrum1D) -> Result<Spectrum1D> {
        sine_bell_apodization(
            spectrum,
            self.start_angle_deg,
            self.end_angle_deg,
            self.exponent,
        )
    }
}

/// Converts a complex spectrum to magnitude mode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Magnitude;

impl Magnitude {
    /// Creates a magnitude-mode processing step.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl ProcessingStep<Spectrum1D> for Magnitude {
    fn apply(&self, spectrum: &Spectrum1D) -> Result<Spectrum1D> {
        magnitude_spectrum(spectrum)
    }
}

/// FFT direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FftDirection {
    /// Forward transform.
    Forward,
    /// Inverse transform normalized by `1 / len`.
    Inverse,
}

/// FFT processing step.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fft1D {
    /// Transform direction.
    pub direction: FftDirection,
}

impl Fft1D {
    /// Creates a one-dimensional FFT step.
    #[must_use]
    pub fn new(direction: FftDirection) -> Self {
        Self { direction }
    }

    /// Creates a forward FFT step.
    #[must_use]
    pub fn forward() -> Self {
        Self::new(FftDirection::Forward)
    }

    /// Creates an inverse FFT step.
    #[must_use]
    pub fn inverse() -> Self {
        Self::new(FftDirection::Inverse)
    }
}

impl ProcessingStep<Spectrum1D> for Fft1D {
    fn apply(&self, spectrum: &Spectrum1D) -> Result<Spectrum1D> {
        fft_1d(spectrum, self.direction)
    }
}

/// Manual zero- and first-order phase correction.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhaseCorrection {
    /// Zero-order phase in degrees.
    pub zero_order_deg: f64,
    /// First-order phase in degrees across the full spectrum.
    pub first_order_deg: f64,
    /// Pivot position as a fraction of the index range, typically in `[0, 1]`.
    pub pivot_fraction: f64,
}

impl Default for PhaseCorrection {
    fn default() -> Self {
        Self {
            zero_order_deg: 0.0,
            first_order_deg: 0.0,
            pivot_fraction: 0.5,
        }
    }
}

impl PhaseCorrection {
    /// Creates a no-op phase correction step.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a phase correction step from explicit zero- and first-order phases.
    #[must_use]
    pub fn from_degrees(zero_order_deg: f64, first_order_deg: f64, pivot_fraction: f64) -> Self {
        Self {
            zero_order_deg,
            first_order_deg,
            pivot_fraction,
        }
    }

    /// Returns this step with a zero-order phase.
    #[must_use]
    pub fn zero_order(mut self, zero_order_deg: f64) -> Self {
        self.zero_order_deg = zero_order_deg;
        self
    }

    /// Returns this step with a first-order phase.
    #[must_use]
    pub fn first_order(mut self, first_order_deg: f64) -> Self {
        self.first_order_deg = first_order_deg;
        self
    }

    /// Returns this step with a pivot fraction.
    #[must_use]
    pub fn pivot_fraction(mut self, pivot_fraction: f64) -> Self {
        self.pivot_fraction = pivot_fraction;
        self
    }
}

impl ProcessingStep<Spectrum1D> for PhaseCorrection {
    fn apply(&self, spectrum: &Spectrum1D) -> Result<Spectrum1D> {
        phase_correct(
            spectrum,
            self.zero_order_deg,
            self.first_order_deg,
            self.pivot_fraction,
        )
    }
}

/// Applies exponential apodization.
///
/// The multiplier at point `i` is `exp(-pi * line_broadening_hz * dwell_time_s * i)`.
///
/// # Errors
///
/// Returns an error when line broadening is negative or either parameter is
/// non-finite.
pub fn exponential_apodization(
    spectrum: &Spectrum1D,
    line_broadening_hz: f64,
    dwell_time_s: f64,
) -> Result<Spectrum1D> {
    ensure_non_negative("line_broadening_hz", line_broadening_hz)?;
    ensure_positive("dwell_time_s", dwell_time_s)?;

    let decay = (-PI * line_broadening_hz * dwell_time_s).exp();
    let mut weight = 1.0;
    let mut processed = spectrum.clone();
    for value in &mut processed.intensities {
        *value *= weight;
        weight *= decay;
    }
    if let Some(imaginary) = &mut processed.imaginary {
        weight = 1.0;
        for value in imaginary {
            *value *= weight;
            weight *= decay;
        }
    }

    Ok(processed.with_processing_record(
        ProcessingRecord::new("exponential_apodization").with_details(format!(
            "line_broadening_hz={line_broadening_hz},dwell_time_s={dwell_time_s}"
        )),
    ))
}

/// Applies Gaussian apodization.
///
/// The multiplier at point `i` is
/// `exp(-(pi * gaussian_broadening_hz * dwell_time_s * i)^2 / (4 * ln(2)))`.
/// `gaussian_broadening_hz` is interpreted as the frequency-domain full width
/// at half maximum contributed by the Gaussian window.
///
/// # Errors
///
/// Returns an error when Gaussian broadening is negative, dwell time is not
/// positive, any parameter is non-finite, or the point count is too large for
/// checked numeric conversion.
pub fn gaussian_apodization(
    spectrum: &Spectrum1D,
    gaussian_broadening_hz: f64,
    dwell_time_s: f64,
) -> Result<Spectrum1D> {
    ensure_non_negative("gaussian_broadening_hz", gaussian_broadening_hz)?;
    ensure_positive("dwell_time_s", dwell_time_s)?;

    let weights = gaussian_weights(
        spectrum.len(),
        gaussian_broadening_hz,
        dwell_time_s,
        "Gaussian apodization",
    )?;
    let mut processed = spectrum.clone();
    for (value, weight) in processed.intensities.iter_mut().zip(&weights) {
        *value *= *weight;
    }
    if let Some(imaginary) = &mut processed.imaginary {
        for (value, weight) in imaginary.iter_mut().zip(&weights) {
            *value *= *weight;
        }
    }

    Ok(processed.with_processing_record(
        ProcessingRecord::new("gaussian_apodization").with_details(format!(
            "gaussian_broadening_hz={gaussian_broadening_hz},dwell_time_s={dwell_time_s}"
        )),
    ))
}

/// Applies sine-bell apodization.
///
/// The multiplier at point `i` is `sin(theta_i)^exponent`, where `theta_i`
/// moves linearly from `start_angle_deg` to `end_angle_deg` across the spectrum.
/// Angles are constrained to `0..=180` degrees so weights remain non-negative.
///
/// # Errors
///
/// Returns an error when either angle is outside `0..=180` degrees, the
/// exponent is not positive, any parameter is non-finite, or the point count is
/// too large for checked numeric conversion.
pub fn sine_bell_apodization(
    spectrum: &Spectrum1D,
    start_angle_deg: f64,
    end_angle_deg: f64,
    exponent: f64,
) -> Result<Spectrum1D> {
    let weights = sine_bell_weights(
        spectrum.len(),
        start_angle_deg,
        end_angle_deg,
        exponent,
        "sine-bell apodization",
    )?;
    let mut processed = spectrum.clone();
    for (value, weight) in processed.intensities.iter_mut().zip(&weights) {
        *value *= *weight;
    }
    if let Some(imaginary) = &mut processed.imaginary {
        for (value, weight) in imaginary.iter_mut().zip(&weights) {
            *value *= *weight;
        }
    }

    Ok(processed.with_processing_record(
        ProcessingRecord::new("sine_bell_apodization").with_details(format!(
            "start_angle_deg={start_angle_deg},end_angle_deg={end_angle_deg},exponent={exponent}"
        )),
    ))
}

/// Converts a spectrum to magnitude mode.
///
/// # Errors
///
/// Returns an error when computed magnitude data is invalid.
pub fn magnitude_spectrum(spectrum: &Spectrum1D) -> Result<Spectrum1D> {
    let intensities = match &spectrum.imaginary {
        Some(imaginary) => spectrum
            .intensities
            .iter()
            .zip(imaginary)
            .map(|(real, imag)| real.hypot(*imag))
            .collect(),
        None => spectrum
            .intensities
            .iter()
            .map(|value| value.abs())
            .collect(),
    };

    let mut processed =
        Spectrum1D::new(spectrum.x.clone(), intensities, spectrum.metadata.clone())?;
    processed.processing.clone_from(&spectrum.processing);
    Ok(processed.with_processing_record(ProcessingRecord::new("magnitude_spectrum")))
}

/// Applies a forward or inverse FFT to a one-dimensional spectrum.
///
/// The inverse direction is normalized by `1 / len`, making
/// `inverse(forward(spectrum))` recover the original values within floating
/// point tolerance.
///
/// # Errors
///
/// Returns an error when the point count cannot be represented safely for
/// normalization.
pub fn fft_1d(spectrum: &Spectrum1D, direction: FftDirection) -> Result<Spectrum1D> {
    let mut buffer = complex_buffer(spectrum);
    let mut planner = FftPlanner::<f64>::new();
    let fft = match direction {
        FftDirection::Forward => planner.plan_fft_forward(buffer.len()),
        FftDirection::Inverse => planner.plan_fft_inverse(buffer.len()),
    };
    fft.process(&mut buffer);

    if direction == FftDirection::Inverse {
        let len = u32::try_from(buffer.len()).map_err(|_| RSpinError::InvalidSpectrum {
            message: "spectrum is too large to normalize inverse FFT".to_owned(),
        })?;
        let scale = 1.0 / f64::from(len);
        for value in &mut buffer {
            *value *= scale;
        }
    }

    let intensities = buffer.iter().map(|value| value.re).collect();
    let imaginary = Some(buffer.iter().map(|value| value.im).collect());
    let mut processed = Spectrum1D::new_complex(
        spectrum.x.clone(),
        intensities,
        imaginary,
        spectrum.metadata.clone(),
    )?;
    processed.processing.clone_from(&spectrum.processing);
    Ok(processed.with_processing_record(
        ProcessingRecord::new("fft_1d").with_details(format!("direction={direction:?}")),
    ))
}

/// Applies manual phase correction to a complex one-dimensional spectrum.
///
/// The phase at point `i` is `zero_order_deg + first_order_deg *
/// (fraction(i) - pivot_fraction)`, where `fraction(i)` spans `0..=1` across
/// the spectrum. Real-only input is treated as complex data with zero imaginary
/// values, and the output always contains an imaginary channel.
///
/// # Errors
///
/// Returns an error when phase parameters are non-finite, the pivot is outside
/// `[0, 1]`, or the point count is too large for safe conversion.
pub fn phase_correct(
    spectrum: &Spectrum1D,
    zero_order_deg: f64,
    first_order_deg: f64,
    pivot_fraction: f64,
) -> Result<Spectrum1D> {
    ensure_finite("zero_order_deg", zero_order_deg)?;
    ensure_finite("first_order_deg", first_order_deg)?;
    if !pivot_fraction.is_finite() || !(0.0..=1.0).contains(&pivot_fraction) {
        return Err(RSpinError::InvalidSpectrum {
            message: "phase pivot fraction must be finite and between 0 and 1".to_owned(),
        });
    }

    let denominator = index_denominator(spectrum.len())?;
    let mut real = Vec::with_capacity(spectrum.len());
    let mut imaginary = Vec::with_capacity(spectrum.len());
    for (index, value) in complex_buffer(spectrum).into_iter().enumerate() {
        let fraction = if denominator == 0.0 {
            0.0
        } else {
            f64::from(
                u32::try_from(index).map_err(|_| RSpinError::InvalidSpectrum {
                    message: "spectrum is too large for phase correction".to_owned(),
                })?,
            ) / denominator
        };
        let phase_rad =
            (zero_order_deg + first_order_deg * (fraction - pivot_fraction)).to_radians();
        let rotation = Complex::new(phase_rad.cos(), phase_rad.sin());
        let corrected = value * rotation;
        real.push(corrected.re);
        imaginary.push(corrected.im);
    }

    let mut processed = Spectrum1D::new_complex(
        spectrum.x.clone(),
        real,
        Some(imaginary),
        spectrum.metadata.clone(),
    )?;
    processed.processing.clone_from(&spectrum.processing);
    Ok(processed.with_processing_record(
        ProcessingRecord::new("phase_correct").with_details(format!(
            "zero_order_deg={zero_order_deg},first_order_deg={first_order_deg},pivot_fraction={pivot_fraction}"
        )),
    ))
}

fn complex_buffer(spectrum: &Spectrum1D) -> Vec<Complex<f64>> {
    match &spectrum.imaginary {
        Some(imaginary) => spectrum
            .intensities
            .iter()
            .zip(imaginary)
            .map(|(real, imag)| Complex::new(*real, *imag))
            .collect(),
        None => spectrum
            .intensities
            .iter()
            .map(|real| Complex::new(*real, 0.0))
            .collect(),
    }
}

fn index_denominator(len: usize) -> Result<f64> {
    if len <= 1 {
        return Ok(0.0);
    }
    let denominator = u32::try_from(len - 1).map_err(|_| RSpinError::InvalidSpectrum {
        message: "spectrum is too large for phase correction".to_owned(),
    })?;
    Ok(f64::from(denominator))
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
