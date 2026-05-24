use rspin_core::{Axis, Metadata, RSpinError, Spectrum1D, Unit};

use super::*;

#[test]
fn corrects_zero_order_phase() -> anyhow::Result<()> {
    let phased = phase_correct(&real_spectrum()?, 45.0, 0.0, 0.5)?;
    let result = auto_phase_correct(
        &phased,
        AutoPhaseOptions::default()
            .zero_order_range(-90.0, 90.0, 5.0)
            .first_order_range(0.0, 0.0, 1.0),
    )?;

    assert_close(result.zero_order_deg, -45.0);
    assert_close(result.first_order_deg, 0.0);
    assert_vec_close(&result.spectrum.intensities, &[1.0, 2.0, 1.0]);
    assert_eq!(
        result
            .spectrum
            .processing
            .last()
            .map(|record| record.operation.as_str()),
        Some("auto_phase_correct")
    );
    Ok(())
}

#[test]
fn corrects_first_order_phase() -> anyhow::Result<()> {
    let spectrum = Spectrum1D::new_complex(
        Axis::linear("shift", Unit::Ppm, 0.0, 2.0, 3)?,
        vec![1.0, 1.0, 1.0],
        Some(vec![0.0, 0.0, 0.0]),
        Metadata::default(),
    )?;
    let phased = phase_correct(&spectrum, 0.0, 60.0, 0.5)?;
    let result = AutoPhaseCorrection::new()
        .zero_order_range(0.0, 0.0, 1.0)
        .first_order_range(-90.0, 90.0, 5.0)
        .pivot_fraction(0.5)
        .apply(&phased)?;

    assert_vec_close(&result.intensities, &[1.0, 1.0, 1.0]);
    Ok(())
}

#[test]
fn rejects_invalid_options() -> anyhow::Result<()> {
    let spectrum = real_spectrum()?;
    let error = auto_phase_correct(
        &spectrum,
        AutoPhaseOptions::default().zero_order_range(10.0, -10.0, 5.0),
    )
    .expect_err("inverted zero-order range should fail");
    assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));

    let error = auto_phase_correct(
        &spectrum,
        AutoPhaseOptions::default().scoring_weights(0.0, 0.0),
    )
    .expect_err("zero scoring weights should fail");
    assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));
    Ok(())
}

fn real_spectrum() -> anyhow::Result<Spectrum1D> {
    Ok(Spectrum1D::new_complex(
        Axis::linear("shift", Unit::Ppm, 0.0, 2.0, 3)?,
        vec![1.0, 2.0, 1.0],
        Some(vec![0.0, 0.0, 0.0]),
        Metadata::default(),
    )?)
}

fn assert_vec_close(actual: &[f64], expected: &[f64]) {
    assert_eq!(actual.len(), expected.len());
    for (left, right) in actual.iter().zip(expected) {
        assert_close(*left, *right);
    }
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1.0e-10,
        "{actual} != {expected}"
    );
}
