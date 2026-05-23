//! Complex-domain one-dimensional processing.

use std::f64::consts::PI;

use rspin_core::{ProcessingRecord, RSpinError, Result, Spectrum1D};
use rustfft::{FftPlanner, num_complex::Complex};

use crate::ProcessingStep;

/// Applies exponential apodization to real and imaginary channels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExponentialApodization {
    /// Line broadening in hertz.
    pub line_broadening_hz: f64,
    /// Dwell time in seconds.
    pub dwell_time_s: f64,
}

impl ProcessingStep<Spectrum1D> for ExponentialApodization {
    fn apply(&self, spectrum: &Spectrum1D) -> Result<Spectrum1D> {
        exponential_apodization(spectrum, self.line_broadening_hz, self.dwell_time_s)
    }
}

/// Converts a complex spectrum to magnitude mode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Magnitude;

impl ProcessingStep<Spectrum1D> for Magnitude {
    fn apply(&self, spectrum: &Spectrum1D) -> Result<Spectrum1D> {
        magnitude_spectrum(spectrum)
    }
}

/// FFT direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FftDirection {
    /// Forward transform.
    Forward,
    /// Inverse transform normalized by `1 / len`.
    Inverse,
}

/// FFT processing step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fft1D {
    /// Transform direction.
    pub direction: FftDirection,
}

impl ProcessingStep<Spectrum1D> for Fft1D {
    fn apply(&self, spectrum: &Spectrum1D) -> Result<Spectrum1D> {
        fft_1d(spectrum, self.direction)
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
mod tests {
    use rspin_core::{Axis, Metadata, Unit};

    use super::*;

    #[test]
    fn apodization_decays_real_and_imaginary_channels() -> anyhow::Result<()> {
        let spectrum = complex_spectrum()?;
        let processed = exponential_apodization(&spectrum, 1.0, 0.1)?;
        assert_close(processed.intensities[0], 1.0);
        assert!(processed.intensities[1] < 2.0);
        let imaginary = require_imaginary(&processed)?;
        assert_close(imaginary[0], 0.5);
        assert!(imaginary[1] < 1.0);
        Ok(())
    }

    #[test]
    fn magnitude_combines_real_and_imaginary_channels() -> anyhow::Result<()> {
        let spectrum = complex_spectrum()?;
        let processed = Magnitude.apply(&spectrum)?;
        assert_vec_close(
            &processed.intensities,
            &[1.118_033_988_749_895, 2.236_067_977_499_79, 4.0],
        );
        assert!(processed.imaginary.is_none());
        Ok(())
    }

    #[test]
    fn fft_inverse_roundtrip_recovers_complex_data() -> anyhow::Result<()> {
        let spectrum = complex_spectrum()?;
        let transformed = Fft1D {
            direction: FftDirection::Forward,
        }
        .apply(&spectrum)?;
        let recovered = fft_1d(&transformed, FftDirection::Inverse)?;
        assert_vec_close(&recovered.intensities, &spectrum.intensities);
        assert_vec_close(
            require_imaginary(&recovered)?,
            require_imaginary(&spectrum)?,
        );
        Ok(())
    }

    #[test]
    fn rejects_negative_line_broadening() -> anyhow::Result<()> {
        let spectrum = complex_spectrum()?;
        let error = exponential_apodization(&spectrum, -1.0, 0.1)
            .expect_err("negative line broadening should fail");
        assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));
        Ok(())
    }

    fn complex_spectrum() -> anyhow::Result<Spectrum1D> {
        let axis = Axis::linear("time", Unit::Seconds, 0.0, 0.2, 3)?;
        Ok(Spectrum1D::new_complex(
            axis,
            vec![1.0, 2.0, 4.0],
            Some(vec![0.5, 1.0, 0.0]),
            Metadata::default(),
        )?)
    }

    fn require_imaginary(spectrum: &Spectrum1D) -> anyhow::Result<&[f64]> {
        match &spectrum.imaginary {
            Some(imaginary) => Ok(imaginary),
            None => anyhow::bail!("missing imaginary channel"),
        }
    }

    fn assert_vec_close(actual: &[f64], expected: &[f64]) {
        assert_eq!(actual.len(), expected.len());
        for (left, right) in actual.iter().zip(expected) {
            assert_close(*left, *right);
        }
    }

    fn assert_close(actual: f64, expected: f64) {
        assert!((actual - expected).abs() < 1e-10, "{actual} != {expected}");
    }
}
