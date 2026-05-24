use rspin_core::{Axis, Metadata, RSpinError, Spectrum2D, Unit};

use super::*;

#[test]
fn corrects_2d_zero_order_phase() -> anyhow::Result<()> {
    let phased = phase_correct_2d(
        &real_spectrum()?,
        PhaseCorrection2D::new().x_phase(45.0, 0.0, 0.5),
    )?;
    let result = auto_phase_correct_2d(
        &phased,
        AutoPhase2DOptions::default()
            .x_zero_order_range(-90.0, 90.0, 5.0)
            .x_first_order_range(0.0, 0.0, 1.0)
            .y_zero_order_range(0.0, 0.0, 1.0)
            .y_first_order_range(0.0, 0.0, 1.0),
    )?;

    assert_close(result.correction.x_zero_order_deg, -45.0);
    assert_close(result.correction.x_first_order_deg, 0.0);
    assert_close(result.correction.y_zero_order_deg, 0.0);
    assert_vec_close(&result.spectrum.z, &[1.0, 2.0, 3.0, 4.0]);
    assert_eq!(
        result
            .spectrum
            .processing
            .last()
            .map(|record| record.operation.as_str()),
        Some("auto_phase_correct_2d")
    );
    Ok(())
}

#[test]
fn serializes_2d_auto_phase_result_and_step() -> anyhow::Result<()> {
    let phased = phase_correct_2d(
        &real_spectrum()?,
        PhaseCorrection2D::new().x_phase(45.0, 0.0, 0.5),
    )?;
    let step = AutoPhaseCorrection2D::new()
        .x_zero_order_range(-90.0, 90.0, 5.0)
        .x_first_order_range(0.0, 0.0, 1.0)
        .y_zero_order_range(0.0, 0.0, 1.0)
        .y_first_order_range(0.0, 0.0, 1.0);
    let result = auto_phase_correct_2d(&phased, step.options)?;
    let result_json = serde_json::to_string(&result)?;
    let parsed_result: AutoPhase2DResult = serde_json::from_str(&result_json)?;
    let step_json = serde_json::to_string(&step)?;
    let parsed_step: AutoPhaseCorrection2D = serde_json::from_str(&step_json)?;

    assert_eq!(parsed_result, result);
    assert_eq!(parsed_step, step);
    assert!(result_json.contains("\"correction\""));
    assert!(step_json.contains("\"x_zero_order_min_deg\""));
    Ok(())
}

#[test]
fn corrects_2d_y_first_order_phase() -> anyhow::Result<()> {
    let spectrum = Spectrum2D::new_complex(
        Axis::linear("x", Unit::Points, 0.0, 1.0, 2)?,
        Axis::linear("y", Unit::Points, 0.0, 1.0, 2)?,
        vec![1.0, 1.0, 1.0, 1.0],
        Some(vec![0.0, 0.0, 0.0, 0.0]),
        Metadata::default(),
    )?;
    let phased = phase_correct_2d(&spectrum, PhaseCorrection2D::new().y_phase(0.0, 60.0, 0.5))?;
    let result = AutoPhaseCorrection2D::new()
        .x_zero_order_range(0.0, 0.0, 1.0)
        .x_first_order_range(0.0, 0.0, 1.0)
        .y_zero_order_range(0.0, 0.0, 1.0)
        .y_first_order_range(-90.0, 90.0, 5.0)
        .y_pivot_fraction(0.5)
        .apply(&phased)?;

    assert_vec_close(&result.z, &[1.0, 1.0, 1.0, 1.0]);
    Ok(())
}

#[test]
fn rejects_invalid_2d_auto_phase_options() -> anyhow::Result<()> {
    let spectrum = real_spectrum()?;
    let error = auto_phase_correct_2d(
        &spectrum,
        AutoPhase2DOptions::default().x_zero_order_range(10.0, -10.0, 5.0),
    )
    .expect_err("inverted x zero-order range should fail");
    assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));

    let error = auto_phase_correct_2d(
        &spectrum,
        AutoPhase2DOptions::default().scoring_weights(0.0, 0.0),
    )
    .expect_err("zero scoring weights should fail");
    assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));
    Ok(())
}

#[test]
fn rejects_excessive_2d_auto_phase_grid() -> anyhow::Result<()> {
    let spectrum = real_spectrum()?;
    let error = auto_phase_correct_2d(
        &spectrum,
        AutoPhase2DOptions::default()
            .x_zero_order_range(-180.0, 180.0, 1.0)
            .y_zero_order_range(-180.0, 180.0, 1.0),
    )
    .expect_err("large 2D auto-phase grid should fail");

    assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));
    Ok(())
}

fn real_spectrum() -> anyhow::Result<Spectrum2D> {
    Ok(Spectrum2D::new_complex(
        Axis::linear("x", Unit::Points, 0.0, 1.0, 2)?,
        Axis::linear("y", Unit::Points, 0.0, 1.0, 2)?,
        vec![1.0, 2.0, 3.0, 4.0],
        Some(vec![0.0, 0.0, 0.0, 0.0]),
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
