use rspin_core::{Axis, Metadata, RSpinError, Spectrum2D, Unit};

use super::*;

#[test]
fn chains_common_2d_processing_steps() -> anyhow::Result<()> {
    let spectrum = demo_spectrum()?;
    let processed = spectrum
        .process()
        .scale(2.0)
        .absolute_value()
        .crop(1.0, 2.0, 10.0, 11.0)
        .resample(
            Axis::linear("x", Unit::Ppm, 1.0, 2.0, 3)?,
            Axis::linear("y", Unit::Ppm, 10.0, 11.0, 2)?,
        )
        .zero_fill(4, 3)
        .gaussian_apodization(0.0, 0.0, 0.1, 0.1)
        .normalize_max_abs()
        .finish()?;

    assert_eq!(processed.shape(), (4, 3));
    assert_vec_close(
        &processed.z,
        &[
            4.0 / 12.0,
            5.0 / 12.0,
            6.0 / 12.0,
            0.0,
            10.0 / 12.0,
            11.0 / 12.0,
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
        ],
    );
    assert_eq!(processed.processing.len(), 7);
    assert_eq!(processed.processing[0].operation, "scale_2d");
    assert_eq!(processed.processing[1].operation, "abs_2d");
    assert_eq!(processed.processing[2].operation, "crop_2d");
    assert_eq!(processed.processing[3].operation, "resample_2d");
    assert_eq!(processed.processing[5].operation, "gaussian_apodization_2d");
    assert_eq!(processed.processing[6].operation, "normalize_2d_max_abs");
    Ok(())
}

#[test]
fn borrowed_pipeline_leaves_original_2d_spectrum_unchanged() -> anyhow::Result<()> {
    let spectrum = demo_spectrum()?;
    let processed = (&spectrum).process().scale(3.0).finish()?;

    assert_eq!(spectrum.z, vec![1.0, -2.0, 3.0, 4.0, -5.0, 6.0]);
    assert_eq!(processed.z, vec![3.0, -6.0, 9.0, 12.0, -15.0, 18.0]);
    Ok(())
}

#[test]
fn chains_from_fallible_2d_spectrum_result() -> anyhow::Result<()> {
    let spectrum_result: rspin_core::Result<Spectrum2D> = Ok(demo_spectrum()?);
    let processed = spectrum_result.process().scale(2.0).finish()?;

    assert_eq!(processed.z, vec![2.0, -4.0, 6.0, 8.0, -10.0, 12.0]);
    assert_eq!(processed.processing.len(), 1);
    assert_eq!(processed.processing[0].operation, "scale_2d");
    Ok(())
}

#[test]
fn result_2d_pipeline_preserves_initial_error() {
    let spectrum_result: rspin_core::Result<Spectrum2D> = Err(RSpinError::InvalidSpectrum {
        message: "initial 2d failure".to_owned(),
    });
    let error = spectrum_result
        .process()
        .scale(2.0)
        .finish()
        .expect_err("initial error should be preserved");

    assert_eq!(
        error,
        RSpinError::InvalidSpectrum {
            message: "initial 2d failure".to_owned()
        }
    );
}

#[test]
fn terminal_projection_includes_prior_processing() -> anyhow::Result<()> {
    let spectrum = demo_spectrum()?;
    let projection = spectrum
        .process()
        .scale(2.0)
        .project_x(ProjectionMode::Sum)?;

    assert_eq!(projection.intensities, vec![10.0, -14.0, 18.0]);
    assert_eq!(projection.processing.len(), 2);
    assert_eq!(projection.processing[0].operation, "scale_2d");
    assert_eq!(projection.processing[1].operation, "project_x");
    Ok(())
}

#[test]
fn terminal_slices_use_processed_data() -> anyhow::Result<()> {
    let spectrum = demo_spectrum()?;
    let row = (&spectrum).process().scale(0.5).slice_x_at_y_index(1)?;
    let row_by_coordinate = (&spectrum).process().scale(0.5).slice_x_at_y(10.8)?;
    let column = spectrum
        .clone()
        .process()
        .scale(0.5)
        .slice_y_at_x_index(1)?;
    let column_by_coordinate = spectrum.process().scale(0.5).slice_y_at_x(0.8)?;

    assert_eq!(row.intensities, vec![2.0, -2.5, 3.0]);
    assert_eq!(row_by_coordinate.intensities, row.intensities);
    assert_eq!(column.intensities, vec![-1.0, -2.5]);
    assert_eq!(column_by_coordinate.intensities, column.intensities);
    Ok(())
}

#[test]
fn chains_2d_fft_steps() -> anyhow::Result<()> {
    let spectrum = demo_spectrum()?;
    let processed = spectrum
        .process()
        .fft(FftDirection::Forward)
        .fft(FftDirection::Inverse)
        .finish()?;

    assert_vec_close(&processed.z, &[1.0, -2.0, 3.0, 4.0, -5.0, 6.0]);
    assert!(processed.imaginary.is_some());
    assert_eq!(processed.processing.len(), 2);
    assert_eq!(processed.processing[0].operation, "fft_2d");
    assert_eq!(processed.processing[1].operation, "fft_2d");
    Ok(())
}

#[test]
fn chains_2d_phase_steps() -> anyhow::Result<()> {
    let spectrum = demo_spectrum()?;
    let processed = spectrum
        .process()
        .phase_x(90.0, 0.0, 0.5)
        .phase_y(-90.0, 0.0, 0.5)
        .phase(PhaseCorrection2D::new().x_phase(180.0, 0.0, 0.5))
        .finish()?;

    assert_vec_close(&processed.z, &[-1.0, 2.0, -3.0, -4.0, 5.0, -6.0]);
    assert!(processed.imaginary.is_some());
    assert_eq!(processed.processing.len(), 3);
    assert_eq!(processed.processing[0].operation, "phase_correct_2d");
    assert_eq!(processed.processing[2].operation, "phase_correct_2d");
    Ok(())
}

#[test]
fn chains_2d_auto_phase_step() -> anyhow::Result<()> {
    let spectrum = positive_spectrum()?;
    let phased = spectrum.process().phase_x(45.0, 0.0, 0.5).finish()?;
    let processed = phased
        .process()
        .auto_phase_with(
            AutoPhase2DOptions::default()
                .x_zero_order_range(-90.0, 90.0, 5.0)
                .x_first_order_range(0.0, 0.0, 1.0)
                .y_zero_order_range(0.0, 0.0, 1.0)
                .y_first_order_range(0.0, 0.0, 1.0),
        )
        .finish()?;

    assert_vec_close(&processed.z, &[1.0, 2.0, 3.0, 4.0]);
    assert_eq!(
        processed
            .processing
            .last()
            .map(|record| record.operation.as_str()),
        Some("auto_phase_correct_2d")
    );
    Ok(())
}

#[test]
fn preserves_first_2d_pipeline_error() -> anyhow::Result<()> {
    let spectrum = demo_spectrum()?;
    let error = spectrum
        .process()
        .scale(f64::NAN)
        .zero_fill(4, 3)
        .project_y(ProjectionMode::Sum)
        .expect_err("non-finite scale should fail");

    assert!(matches!(error, RSpinError::NonFinite { .. }));
    Ok(())
}

fn demo_spectrum() -> anyhow::Result<Spectrum2D> {
    Ok(Spectrum2D::new(
        Axis::linear("x", Unit::Ppm, 0.0, 2.0, 3)?,
        Axis::linear("y", Unit::Ppm, 10.0, 11.0, 2)?,
        vec![1.0, -2.0, 3.0, 4.0, -5.0, 6.0],
        Metadata::named("2d"),
    )?)
}

fn positive_spectrum() -> anyhow::Result<Spectrum2D> {
    Ok(Spectrum2D::new(
        Axis::linear("x", Unit::Ppm, 0.0, 1.0, 2)?,
        Axis::linear("y", Unit::Ppm, 10.0, 11.0, 2)?,
        vec![1.0, 2.0, 3.0, 4.0],
        Metadata::named("positive 2d"),
    )?)
}

fn assert_vec_close(actual: &[f64], expected: &[f64]) {
    assert_eq!(actual.len(), expected.len());
    for (left, right) in actual.iter().zip(expected) {
        assert!((left - right).abs() < 1.0e-12, "{left} != {right}");
    }
}
