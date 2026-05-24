use rspin_core::{Metadata, Unit};

use super::*;

#[test]
fn generates_2d_matrix_on_first_spectrum_axes() -> anyhow::Result<()> {
    let first = spectrum(
        "a",
        &[0.0, 1.0, 2.0],
        &[0.0, 1.0],
        &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
    )?;
    let second = spectrum("b b", &[0.0, 2.0], &[0.0, 1.0], &[10.0, 14.0, 20.0, 24.0])?;

    let matrix = generate_spectrum_matrix_2d(&[first, second], MatrixGeneration2DOptions::new())?;

    assert_eq!(matrix.shape(), (2, 2, 3));
    assert_eq!(matrix.spectrum_ids, vec!["0:a", "1:b_b"]);
    assert_eq!(matrix.x.values, vec![0.0, 1.0, 2.0]);
    assert_eq!(matrix.y.values, vec![0.0, 1.0]);
    assert_eq!(
        matrix.values,
        vec![
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 10.0, 12.0, 14.0, 20.0, 22.0, 24.0
        ]
    );
    assert_eq!(matrix.value_at(1, 1, 1), Some(22.0));
    assert_eq!(matrix.value_at(2, 0, 0), None);
    Ok(())
}

#[test]
fn supports_explicit_2d_axes_and_outside_value() -> anyhow::Result<()> {
    let input = spectrum("sample", &[0.0, 2.0], &[0.0, 1.0], &[0.0, 2.0, 10.0, 12.0])?;
    let options = MatrixGeneration2DOptions::new()
        .with_target_axes(
            Axis::new("x", Unit::Ppm, vec![-1.0, 1.0, 3.0])?,
            Axis::new("y", Unit::Ppm, vec![0.0, 0.5, 2.0])?,
        )
        .with_outside_value(-1.0);

    let matrix = generate_spectrum_matrix_2d(&[input], options)?;

    assert_eq!(matrix.shape(), (1, 3, 3));
    assert_eq!(
        matrix.values,
        vec![-1.0, 1.0, -1.0, -1.0, 6.0, -1.0, -1.0, -1.0, -1.0]
    );
    Ok(())
}

#[test]
fn builder_can_clear_explicit_2d_axes() -> anyhow::Result<()> {
    let input = spectrum("sample", &[0.0, 2.0], &[0.0, 1.0], &[0.0, 2.0, 10.0, 12.0])?;
    let options = MatrixGeneration2DOptions::new()
        .with_target_x_axis(Axis::linear("x", Unit::Ppm, -1.0, 3.0, 3)?)
        .with_target_y_axis(Axis::linear("y", Unit::Ppm, -1.0, 2.0, 4)?)
        .without_target_axes();

    let matrix = generate_spectrum_matrix_2d(&[input], options)?;

    assert_eq!(matrix.x.values, vec![0.0, 2.0]);
    assert_eq!(matrix.y.values, vec![0.0, 1.0]);
    assert_eq!(matrix.values, vec![0.0, 2.0, 10.0, 12.0]);
    Ok(())
}

#[test]
fn supports_descending_2d_axes() -> anyhow::Result<()> {
    let first = spectrum(
        "descending",
        &[2.0, 1.0, 0.0],
        &[1.0, 0.0],
        &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
    )?;
    let second = spectrum("other", &[2.0, 0.0], &[1.0, 0.0], &[10.0, 14.0, 20.0, 24.0])?;

    let matrix = generate_spectrum_matrix_2d(&[first, second], MatrixGeneration2DOptions::new())?;

    assert_eq!(matrix.x.values, vec![2.0, 1.0, 0.0]);
    assert_eq!(matrix.y.values, vec![1.0, 0.0]);
    assert_eq!(
        matrix.values,
        vec![
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 10.0, 12.0, 14.0, 20.0, 22.0, 24.0
        ]
    );
    Ok(())
}

#[test]
fn rejects_empty_input_and_non_monotonic_2d_axes() -> anyhow::Result<()> {
    let empty_error = generate_spectrum_matrix_2d(&[], MatrixGeneration2DOptions::new())
        .expect_err("empty input should fail");
    assert!(matches!(empty_error, RSpinError::InvalidSpectrum { .. }));

    let bad = spectrum(
        "bad",
        &[0.0, 2.0, 1.0],
        &[0.0, 1.0],
        &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
    )?;
    let axis_error = generate_spectrum_matrix_2d(&[bad], MatrixGeneration2DOptions::new())
        .expect_err("non-monotonic axis should fail");
    assert!(matches!(axis_error, RSpinError::InvalidAxis { .. }));
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
