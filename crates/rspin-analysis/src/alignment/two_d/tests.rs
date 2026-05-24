use rspin_core::{Axis, Metadata, RSpinError, Spectrum2D, Unit};

use crate::{
    AlignmentWindow, MatrixGeneration2DOptions, ZoneAlignmentOptions, ZoneDetectionOptions,
    align_spectra_by_zone, align_spectra_by_zone_to_matrix,
};

#[test]
fn aligns_to_first_spectrum_zone_by_default() -> anyhow::Result<()> {
    let first = spectrum(
        "ref",
        &[0.0, 1.0, 2.0],
        &[0.0, 1.0, 2.0],
        &[0.0, 0.0, 0.0, 0.0, 5.0, 0.0, 0.0, 0.0, 0.0],
    )?;
    let second = spectrum(
        "shifted",
        &[0.5, 1.5, 2.5],
        &[-0.25, 0.75, 1.75],
        &[0.0, 0.0, 0.0, 0.0, 7.0, 0.0, 0.0, 0.0, 0.0],
    )?;

    let result = align_spectra_by_zone(&[first, second], ZoneAlignmentOptions::default())?;

    assert_eq!(result.shifts.len(), 2);
    assert_close(result.shifts[0].delta_x, 0.0);
    assert_close(result.shifts[0].delta_y, 0.0);
    assert_close(result.shifts[1].delta_x, -0.5);
    assert_close(result.shifts[1].delta_y, 0.25);
    assert_eq!(result.shifts[1].row_id, "1:shifted");
    assert_eq!(result.spectra[1].x.values, vec![0.0, 1.0, 2.0]);
    assert_eq!(result.spectra[1].y.values, vec![0.0, 1.0, 2.0]);
    assert_eq!(
        result.spectra[1].processing[0].operation,
        "align_spectrum_by_zone"
    );
    Ok(())
}

#[test]
fn aligns_to_explicit_target_with_windows() -> anyhow::Result<()> {
    let input = spectrum(
        "sample a",
        &[0.0, 1.0, 2.0],
        &[0.0, 1.0, 2.0],
        &[10.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 20.0],
    )?;
    let options = ZoneAlignmentOptions::new()
        .with_target(2.0, 3.0)
        .with_windows(
            AlignmentWindow::new(-0.1, 0.1),
            AlignmentWindow::new(-0.1, 0.1),
        )
        .with_zone_options(ZoneDetectionOptions::new().with_threshold_abs(1.0));

    let result = align_spectra_by_zone(&[input], options)?;

    assert_close(result.shifts[0].observed_x, 0.0);
    assert_close(result.shifts[0].observed_y, 0.0);
    assert_close(result.shifts[0].target_x, 2.0);
    assert_close(result.shifts[0].target_y, 3.0);
    assert_eq!(result.spectra[0].x.values, vec![2.0, 3.0, 4.0]);
    assert_eq!(result.spectra[0].y.values, vec![3.0, 4.0, 5.0]);
    Ok(())
}

#[test]
fn aligns_spectra_and_generates_2d_matrix() -> anyhow::Result<()> {
    let first = spectrum(
        "ref",
        &[0.0, 1.0, 2.0],
        &[0.0, 1.0, 2.0],
        &[0.0, 0.0, 0.0, 0.0, 5.0, 0.0, 0.0, 0.0, 0.0],
    )?;
    let second = spectrum(
        "shifted",
        &[0.5, 1.5, 2.5],
        &[-0.25, 0.75, 1.75],
        &[0.0, 0.0, 0.0, 0.0, 7.0, 0.0, 0.0, 0.0, 0.0],
    )?;

    let result = align_spectra_by_zone_to_matrix(
        &[first, second],
        ZoneAlignmentOptions::new(),
        MatrixGeneration2DOptions::new(),
    )?;

    assert_eq!(result.matrix.shape(), (2, 3, 3));
    assert_eq!(result.matrix.spectrum_ids, vec!["0:ref", "1:shifted"]);
    assert_eq!(result.matrix.x.values, vec![0.0, 1.0, 2.0]);
    assert_eq!(result.matrix.y.values, vec![0.0, 1.0, 2.0]);
    assert_eq!(
        result.matrix.values,
        vec![
            0.0, 0.0, 0.0, 0.0, 5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 7.0, 0.0, 0.0, 0.0,
            0.0,
        ]
    );
    assert_close(result.shifts[1].delta_x, -0.5);
    assert_close(result.shifts[1].delta_y, 0.25);
    Ok(())
}

#[test]
fn builder_options_can_clear_optional_values() {
    let options = ZoneAlignmentOptions::new()
        .with_target_x(1.0)
        .with_target_y(2.0)
        .without_target()
        .with_x_window(AlignmentWindow::new(0.0, 2.0))
        .with_y_window(AlignmentWindow::new(0.0, 2.0))
        .without_windows();

    assert_eq!(options.target_x, None);
    assert_eq!(options.target_y, None);
    assert_eq!(options.x_window, None);
    assert_eq!(options.y_window, None);
}

#[test]
fn rejects_empty_input_missing_zones_and_invalid_options() -> anyhow::Result<()> {
    let empty_error = align_spectra_by_zone(&[], ZoneAlignmentOptions::default())
        .expect_err("empty input should fail");
    assert!(matches!(empty_error, RSpinError::InvalidSpectrum { .. }));

    let flat = spectrum("flat", &[0.0, 1.0], &[0.0, 1.0], &[0.0, 0.0, 0.0, 0.0])?;
    let zone_error = align_spectra_by_zone(&[flat], ZoneAlignmentOptions::default())
        .expect_err("missing zone should fail");
    assert!(matches!(zone_error, RSpinError::InvalidSpectrum { .. }));

    let target_error =
        align_spectra_by_zone(&[], ZoneAlignmentOptions::new().with_target_x(f64::NAN))
            .expect_err("non-finite target should fail");
    assert!(matches!(target_error, RSpinError::NonFinite { .. }));

    Ok(())
}

fn spectrum(name: &str, x: &[f64], y: &[f64], z: &[f64]) -> anyhow::Result<Spectrum2D> {
    Ok(Spectrum2D::new(
        Axis::new("x", Unit::Ppm, x.to_vec())?,
        Axis::new("y", Unit::Ppm, y.to_vec())?,
        z.to_vec(),
        Metadata::named(name),
    )?)
}

fn assert_close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-12, "{actual} != {expected}");
}
