use std::f64::consts::PI;

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
