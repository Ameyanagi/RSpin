use rspin_core::{Axis, Metadata, RSpinError, Spectrum1D, Unit};

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
fn applies_zero_order_phase_correction() -> anyhow::Result<()> {
    let spectrum = Spectrum1D::new_complex(
        Axis::linear("shift", Unit::Ppm, 0.0, 1.0, 2)?,
        vec![1.0, 0.0],
        Some(vec![0.0, 1.0]),
        Metadata::default(),
    )?;

    let processed = PhaseCorrection {
        zero_order_deg: 90.0,
        first_order_deg: 0.0,
        pivot_fraction: 0.0,
    }
    .apply(&spectrum)?;

    assert_vec_close(&processed.intensities, &[0.0, -1.0]);
    assert_vec_close(require_imaginary(&processed)?, &[1.0, 0.0]);
    assert_eq!(processed.processing[0].operation, "phase_correct");
    Ok(())
}

#[test]
fn applies_first_order_phase_around_pivot() -> anyhow::Result<()> {
    let spectrum = Spectrum1D::new_complex(
        Axis::linear("shift", Unit::Ppm, 0.0, 2.0, 3)?,
        vec![1.0, 1.0, 1.0],
        Some(vec![0.0, 0.0, 0.0]),
        Metadata::default(),
    )?;

    let processed = phase_correct(&spectrum, 0.0, 180.0, 0.5)?;

    assert_vec_close(&processed.intensities, &[0.0, 1.0, 0.0]);
    assert_vec_close(require_imaginary(&processed)?, &[-1.0, 0.0, 1.0]);
    Ok(())
}

#[test]
fn rejects_invalid_phase_pivot() -> anyhow::Result<()> {
    let spectrum = complex_spectrum()?;
    let error = phase_correct(&spectrum, 0.0, 0.0, 1.5).expect_err("invalid pivot should fail");
    assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));
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
