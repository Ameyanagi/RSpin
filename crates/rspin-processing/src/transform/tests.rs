use std::f64::consts::{LN_2, PI};

use rspin_core::{Axis, Metadata, RSpinError, Spectrum1D, Unit};

use super::*;

#[test]
fn remove_group_delay_rotates_leading_samples() -> anyhow::Result<()> {
    let axis = Axis::linear("time", Unit::Seconds, 0.0, 4.0, 5)?;
    let spectrum = Spectrum1D::new_complex(
        axis,
        vec![10.0, 20.0, 30.0, 40.0, 50.0],
        Some(vec![1.0, 2.0, 3.0, 4.0, 5.0]),
        Metadata::default(),
    )?;
    let shifted = remove_group_delay(&spectrum, 2.0)?;
    assert_eq!(shifted.intensities, vec![30.0, 40.0, 50.0, 10.0, 20.0]);
    assert_eq!(shifted.imaginary, Some(vec![3.0, 4.0, 5.0, 1.0, 2.0]));
    assert_eq!(
        shifted.processing.last().map(|r| r.operation.as_str()),
        Some("remove_group_delay")
    );
    Ok(())
}

#[test]
fn subsample_shift_is_inverse_of_remove_group_delay() -> anyhow::Result<()> {
    // Apply a known fractional shift to a complex spectrum and verify
    // that the inverse fractional shift undoes it (round-trip identity).
    let axis = Axis::linear("shift", Unit::Ppm, -1.0, 1.0, 16)?;
    let real = (0..16_i32)
        .map(|i| (f64::from(i) * 0.5).sin())
        .collect::<Vec<_>>();
    let imag = (0..16_i32)
        .map(|i| (f64::from(i) * 0.5).cos())
        .collect::<Vec<_>>();
    let spectrum =
        Spectrum1D::new_complex(axis, real.clone(), Some(imag.clone()), Metadata::default())?;
    let shifted = apply_subsample_shift(&spectrum, 0.4)?;
    let recovered = apply_subsample_shift(&shifted, -0.4)?;
    assert_vec_close(&recovered.intensities, &real);
    assert_vec_close(require_imaginary(&recovered)?, &imag);
    Ok(())
}

#[test]
fn subsample_shift_rejects_real_only_spectrum() -> anyhow::Result<()> {
    let axis = Axis::linear("shift", Unit::Ppm, 0.0, 1.0, 4)?;
    let spectrum = Spectrum1D::new(axis, vec![1.0, 2.0, 3.0, 4.0], Metadata::default())?;
    assert!(apply_subsample_shift(&spectrum, 0.1).is_err());
    Ok(())
}

#[test]
fn remove_group_delay_rejects_invalid_input() -> anyhow::Result<()> {
    let axis = Axis::linear("time", Unit::Seconds, 0.0, 1.0, 2)?;
    let spectrum = Spectrum1D::new_complex(
        axis,
        vec![1.0, 2.0],
        Some(vec![0.0, 0.0]),
        Metadata::default(),
    )?;
    assert!(remove_group_delay(&spectrum, -1.0).is_err());
    assert!(remove_group_delay(&spectrum, f64::NAN).is_err());
    Ok(())
}

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
fn gaussian_apodization_damps_real_and_imaginary_channels() -> anyhow::Result<()> {
    let spectrum = complex_spectrum()?;
    let processed = gaussian_apodization(&spectrum, 1.0, 0.1)?;
    let weight_one = (-(PI * 0.1_f64).powi(2) / (4.0 * LN_2)).exp();
    let weight_two = (-(PI * 0.2_f64).powi(2) / (4.0 * LN_2)).exp();

    assert_close(processed.intensities[0], 1.0);
    assert_close(processed.intensities[1], 2.0 * weight_one);
    assert_close(processed.intensities[2], 4.0 * weight_two);
    let imaginary = require_imaginary(&processed)?;
    assert_close(imaginary[0], 0.5);
    assert_close(imaginary[1], weight_one);
    assert_close(imaginary[2], 0.0);
    assert_eq!(processed.processing[0].operation, "gaussian_apodization");
    Ok(())
}

#[test]
fn sine_bell_apodization_weights_real_and_imaginary_channels() -> anyhow::Result<()> {
    let spectrum = complex_spectrum()?;
    let processed = sine_bell_apodization(&spectrum, 0.0, 180.0, 1.0)?;

    assert_close(processed.intensities[0], 0.0);
    assert_close(processed.intensities[1], 2.0);
    assert_close(processed.intensities[2], 0.0);
    let imaginary = require_imaginary(&processed)?;
    assert_close(imaginary[0], 0.0);
    assert_close(imaginary[1], 1.0);
    assert_close(imaginary[2], 0.0);
    assert_eq!(processed.processing[0].operation, "sine_bell_apodization");
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

#[test]
fn rejects_negative_gaussian_broadening() -> anyhow::Result<()> {
    let spectrum = complex_spectrum()?;
    let error = GaussianApodization {
        gaussian_broadening_hz: -1.0,
        dwell_time_s: 0.1,
    }
    .apply(&spectrum)
    .expect_err("negative Gaussian broadening should fail");
    assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));
    Ok(())
}

#[test]
fn rejects_invalid_sine_bell_parameters() -> anyhow::Result<()> {
    let spectrum = complex_spectrum()?;
    let angle_error = sine_bell_apodization(&spectrum, -1.0, 180.0, 1.0)
        .expect_err("negative sine-bell angle should fail");
    assert!(matches!(angle_error, RSpinError::InvalidSpectrum { .. }));

    let exponent_error = SineBellApodization {
        start_angle_deg: 0.0,
        end_angle_deg: 180.0,
        exponent: 0.0,
    }
    .apply(&spectrum)
    .expect_err("zero sine-bell exponent should fail");
    assert!(matches!(exponent_error, RSpinError::InvalidSpectrum { .. }));
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

#[test]
fn fft_forward_relabels_time_axis_to_hertz() -> anyhow::Result<()> {
    let dwell = 0.001_f64;
    let len = 8;
    let axis_values: Vec<f64> = (0..u32::try_from(len)?)
        .map(|i| f64::from(i) * dwell)
        .collect();
    let axis = Axis::new("time", Unit::Seconds, axis_values)?;
    let spectrum = Spectrum1D::new_complex(
        axis,
        vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Some(vec![0.0; 8]),
        Metadata::default(),
    )?;
    let transformed = fft_1d(&spectrum, FftDirection::Forward)?;
    assert_eq!(transformed.x.unit, Unit::Hertz);
    let sw = 1.0 / dwell;
    let expected_first = -sw / 2.0;
    assert_close(transformed.x.values[0], expected_first);
    let dc_index = len / 2;
    assert_close(transformed.x.values[dc_index], 0.0);
    Ok(())
}

#[test]
fn fft_forward_relabels_to_ppm_when_metadata_has_frequency() -> anyhow::Result<()> {
    let dwell = 0.001_f64;
    let len = 8;
    let axis_values: Vec<f64> = (0..u32::try_from(len)?)
        .map(|i| f64::from(i) * dwell)
        .collect();
    let axis = Axis::new("time", Unit::Seconds, axis_values)?;
    let metadata = Metadata::default().with_frequency_mhz(500.0);
    let spectrum = Spectrum1D::new_complex(
        axis,
        vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Some(vec![0.0; 8]),
        metadata,
    )?;
    let transformed = fft_1d(&spectrum, FftDirection::Forward)?;
    assert_eq!(transformed.x.unit, Unit::Ppm);
    let sw_hz = 1.0 / dwell;
    let expected_first_ppm = -sw_hz / 2.0 / 500.0;
    assert_close(transformed.x.values[0], expected_first_ppm);
    Ok(())
}
