use std::f64::consts::{LN_2, PI};

use rspin_core::{Axis, Metadata, RSpinError, Spectrum2D, Unit};

use super::*;

#[test]
fn applies_separable_exponential_window() -> anyhow::Result<()> {
    let spectrum = demo_spectrum()?;
    let processed = exponential_apodization_2d(&spectrum, 1.0, 2.0, 0.1, 0.2)?;

    let x_decay = (-PI * 0.1).exp();
    let y_decay = (-PI * 0.4).exp();
    assert_close(processed.z[0], 1.0);
    assert_close(processed.z[1], 2.0 * x_decay);
    assert_close(processed.z[2], 3.0 * x_decay * x_decay);
    assert_close(processed.z[3], 4.0 * y_decay);
    assert_close(processed.z[4], 5.0 * x_decay * y_decay);
    assert_close(processed.z[5], 6.0 * x_decay * x_decay * y_decay);
    assert_eq!(
        processed.processing[0].operation,
        "exponential_apodization_2d"
    );
    Ok(())
}

#[test]
fn applies_separable_exponential_window_to_imaginary_channel() -> anyhow::Result<()> {
    let spectrum = Spectrum2D::new_complex(
        Axis::linear("x", Unit::Seconds, 0.0, 0.2, 3)?,
        Axis::linear("y", Unit::Seconds, 0.0, 0.1, 2)?,
        vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
        Some(vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0]),
        Metadata::default(),
    )?;
    let processed = exponential_apodization_2d(&spectrum, 1.0, 2.0, 0.1, 0.2)?;
    let imaginary = require_imaginary(&processed)?;

    let x_decay = (-PI * 0.1).exp();
    let y_decay = (-PI * 0.4).exp();
    assert_close(imaginary[0], 10.0);
    assert_close(imaginary[1], 20.0 * x_decay);
    assert_close(imaginary[5], 60.0 * x_decay * x_decay * y_decay);
    Ok(())
}

#[test]
fn applies_separable_gaussian_window() -> anyhow::Result<()> {
    let spectrum = demo_spectrum()?;
    let processed = gaussian_apodization_2d(&spectrum, 1.0, 2.0, 0.1, 0.2)?;

    let x_one = (-(PI * 0.1_f64).powi(2) / (4.0 * LN_2)).exp();
    let x_two = (-(PI * 0.2_f64).powi(2) / (4.0 * LN_2)).exp();
    let y_one = (-(PI * 0.4_f64).powi(2) / (4.0 * LN_2)).exp();
    assert_close(processed.z[0], 1.0);
    assert_close(processed.z[1], 2.0 * x_one);
    assert_close(processed.z[2], 3.0 * x_two);
    assert_close(processed.z[3], 4.0 * y_one);
    assert_close(processed.z[4], 5.0 * x_one * y_one);
    assert_close(processed.z[5], 6.0 * x_two * y_one);
    assert_eq!(processed.processing[0].operation, "gaussian_apodization_2d");
    Ok(())
}

#[test]
fn applies_separable_gaussian_window_to_imaginary_channel() -> anyhow::Result<()> {
    let spectrum = Spectrum2D::new_complex(
        Axis::linear("x", Unit::Seconds, 0.0, 0.2, 3)?,
        Axis::linear("y", Unit::Seconds, 0.0, 0.1, 2)?,
        vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
        Some(vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0]),
        Metadata::default(),
    )?;
    let processed = GaussianApodization2D {
        x_gaussian_broadening_hz: 1.0,
        y_gaussian_broadening_hz: 2.0,
        x_dwell_time_s: 0.1,
        y_dwell_time_s: 0.2,
    }
    .apply(&spectrum)?;
    let imaginary = require_imaginary(&processed)?;

    let x_one = (-(PI * 0.1_f64).powi(2) / (4.0 * LN_2)).exp();
    let x_two = (-(PI * 0.2_f64).powi(2) / (4.0 * LN_2)).exp();
    let y_one = (-(PI * 0.4_f64).powi(2) / (4.0 * LN_2)).exp();
    assert_close(imaginary[0], 10.0);
    assert_close(imaginary[1], 20.0 * x_one);
    assert_close(imaginary[5], 60.0 * x_two * y_one);
    Ok(())
}

#[test]
fn applies_separable_sine_bell_window() -> anyhow::Result<()> {
    let spectrum = demo_spectrum()?;
    let processed = sine_bell_apodization_2d(&spectrum, 30.0, 90.0, 1.0, 90.0, 30.0, 2.0)?;

    let x0 = 0.5;
    let x1 = 60.0_f64.to_radians().sin();
    let x2 = 1.0;
    let y0 = 1.0;
    let y1 = 0.25;
    assert_close(processed.z[0], x0 * y0);
    assert_close(processed.z[1], 2.0 * x1 * y0);
    assert_close(processed.z[2], 3.0 * x2 * y0);
    assert_close(processed.z[3], 4.0 * x0 * y1);
    assert_close(processed.z[4], 5.0 * x1 * y1);
    assert_close(processed.z[5], 6.0 * x2 * y1);
    assert_eq!(
        processed.processing[0].operation,
        "sine_bell_apodization_2d"
    );
    Ok(())
}

#[test]
fn applies_separable_sine_bell_window_to_imaginary_channel() -> anyhow::Result<()> {
    let spectrum = Spectrum2D::new_complex(
        Axis::linear("x", Unit::Seconds, 0.0, 0.2, 3)?,
        Axis::linear("y", Unit::Seconds, 0.0, 0.1, 2)?,
        vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
        Some(vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0]),
        Metadata::default(),
    )?;
    let processed = SineBellApodization2D {
        x_start_angle_deg: 30.0,
        x_end_angle_deg: 90.0,
        x_exponent: 1.0,
        y_start_angle_deg: 90.0,
        y_end_angle_deg: 30.0,
        y_exponent: 2.0,
    }
    .apply(&spectrum)?;
    let imaginary = require_imaginary(&processed)?;

    assert_close(imaginary[0], 5.0);
    assert_close(imaginary[1], 20.0 * 60.0_f64.to_radians().sin());
    assert_close(imaginary[5], 15.0);
    Ok(())
}

#[test]
fn supports_processing_step_api() -> anyhow::Result<()> {
    let spectrum = demo_spectrum()?;
    let processed = ExponentialApodization2D {
        x_line_broadening_hz: 0.0,
        y_line_broadening_hz: 0.0,
        x_dwell_time_s: 0.1,
        y_dwell_time_s: 0.1,
    }
    .apply(&spectrum)?;

    assert_eq!(processed.z, spectrum.z);
    assert_eq!(
        processed.processing[0].operation,
        "exponential_apodization_2d"
    );
    Ok(())
}

#[test]
fn rejects_invalid_parameters() -> anyhow::Result<()> {
    let spectrum = demo_spectrum()?;
    let broadening_error = exponential_apodization_2d(&spectrum, -1.0, 0.0, 0.1, 0.1)
        .expect_err("negative broadening should fail");
    assert!(matches!(
        broadening_error,
        RSpinError::InvalidSpectrum { .. }
    ));

    let dwell_error = exponential_apodization_2d(&spectrum, 0.0, 0.0, 0.0, 0.1)
        .expect_err("zero dwell should fail");
    assert!(matches!(dwell_error, RSpinError::InvalidSpectrum { .. }));

    let gaussian_error = gaussian_apodization_2d(&spectrum, 0.0, -1.0, 0.1, 0.1)
        .expect_err("negative Gaussian broadening should fail");
    assert!(matches!(gaussian_error, RSpinError::InvalidSpectrum { .. }));

    let sine_error = sine_bell_apodization_2d(&spectrum, 0.0, 181.0, 1.0, 0.0, 90.0, 1.0)
        .expect_err("out-of-range sine-bell angle should fail");
    assert!(matches!(sine_error, RSpinError::InvalidSpectrum { .. }));

    let exponent_error = sine_bell_apodization_2d(&spectrum, 0.0, 90.0, 0.0, 0.0, 90.0, 1.0)
        .expect_err("zero sine-bell exponent should fail");
    assert!(matches!(exponent_error, RSpinError::InvalidSpectrum { .. }));
    Ok(())
}

#[test]
fn lorentz_to_gauss_2d_matches_separable_product() -> anyhow::Result<()> {
    let spectrum = demo_spectrum()?;
    let processed =
        lorentz_to_gauss_apodization_2d(&spectrum, 1.0, 2.0, 0.0, 0.1, 2.0, 2.0, 0.0, 0.2)?;

    // x_dim: lorentz=1, gauss=2, dwell=0.1 → step factor = PI * 1.0 * 0.1 = 0.1*PI.
    // y_dim: lorentz=2, gauss=2, dwell=0.2 → step factor = PI * 2.0 * 0.2 = 0.4*PI.
    let x_rise_one = (PI * 0.1_f64).exp();
    let x_gauss_one = (-(PI * 0.2_f64).powi(2) / (4.0 * LN_2)).exp();
    let x_gauss_two = (-(PI * 0.4_f64).powi(2) / (4.0 * LN_2)).exp();
    let y_rise_one = (PI * 0.4_f64).exp();
    let y_gauss_one = (-(PI * 0.4_f64).powi(2) / (4.0 * LN_2)).exp();

    let x_one = x_rise_one * x_gauss_one;
    let x_two = x_rise_one * x_rise_one * x_gauss_two;
    let y_one = y_rise_one * y_gauss_one;

    assert_close(processed.z[0], 1.0);
    assert_close(processed.z[1], 2.0 * x_one);
    assert_close(processed.z[2], 3.0 * x_two);
    assert_close(processed.z[3], 4.0 * y_one);
    assert_close(processed.z[4], 5.0 * x_one * y_one);
    assert_close(processed.z[5], 6.0 * x_two * y_one);
    assert_eq!(
        processed.processing[0].operation,
        "lorentz_to_gauss_apodization_2d"
    );
    Ok(())
}

#[test]
fn lorentz_to_gauss_2d_rejects_invalid_shift() -> anyhow::Result<()> {
    let spectrum = demo_spectrum()?;
    let error = lorentz_to_gauss_apodization_2d(&spectrum, 1.0, 2.0, 1.5, 0.1, 1.0, 2.0, 0.0, 0.2)
        .expect_err("out-of-range gauss_shift should fail");
    assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));
    Ok(())
}

#[test]
fn trapezoidal_2d_matches_separable_product() -> anyhow::Result<()> {
    let spectrum = demo_spectrum()?;
    // x dim: 3 points → fractions 0, 0.5, 1; rise=0.5, fall=0.5 → 0, 1, 0
    // y dim: 2 points → fractions 0, 1; rise=0.0, fall=1.0 → 1, 1
    let processed = trapezoidal_apodization_2d(&spectrum, 0.5, 0.5, 0.0, 1.0)?;
    assert_close(processed.z[0], 0.0);
    assert_close(processed.z[1], 2.0);
    assert_close(processed.z[2], 0.0);
    assert_close(processed.z[3], 0.0);
    assert_close(processed.z[4], 5.0);
    assert_close(processed.z[5], 0.0);
    assert_eq!(
        processed.processing[0].operation,
        "trapezoidal_apodization_2d"
    );
    Ok(())
}

#[test]
fn trapezoidal_2d_rejects_inverted_window() -> anyhow::Result<()> {
    let spectrum = demo_spectrum()?;
    let error = trapezoidal_apodization_2d(&spectrum, 0.7, 0.3, 0.0, 1.0)
        .expect_err("inverted rise/fall should fail");
    assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));
    Ok(())
}

fn demo_spectrum() -> anyhow::Result<Spectrum2D> {
    Ok(Spectrum2D::new(
        Axis::linear("x", Unit::Seconds, 0.0, 0.2, 3)?,
        Axis::linear("y", Unit::Seconds, 0.0, 0.1, 2)?,
        vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
        Metadata::default(),
    )?)
}

fn assert_close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-12, "{actual} != {expected}");
}

fn require_imaginary(spectrum: &Spectrum2D) -> anyhow::Result<&[f64]> {
    match &spectrum.imaginary {
        Some(imaginary) => Ok(imaginary),
        None => anyhow::bail!("missing imaginary channel"),
    }
}
