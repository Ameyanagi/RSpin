use rspin_core::{Metadata, Unit};

use super::*;

#[test]
fn generates_matrix_on_first_spectrum_axis() -> anyhow::Result<()> {
    let first = spectrum("a", &[0.0, 1.0, 2.0], &[1.0, 2.0, 3.0])?;
    let second = spectrum("b b", &[0.0, 2.0], &[10.0, 14.0])?;

    let matrix = generate_spectrum_matrix_1d(&[first, second], MatrixGenerationOptions::default())?;

    assert_eq!(matrix.shape(), (2, 3));
    assert_eq!(matrix.row_ids, vec!["0:a", "1:b_b"]);
    assert_eq!(matrix.axis.values, vec![0.0, 1.0, 2.0]);
    assert_eq!(matrix.values, vec![1.0, 2.0, 3.0, 10.0, 12.0, 14.0]);
    assert_eq!(matrix.value_at(1, 1), Some(12.0));
    assert_eq!(matrix.value_at(2, 0), None);
    Ok(())
}

#[test]
fn supports_explicit_axis_and_outside_value() -> anyhow::Result<()> {
    let spectrum = spectrum("sample", &[0.0, 2.0], &[1.0, 5.0])?;
    let matrix = generate_spectrum_matrix_1d(
        &[spectrum],
        MatrixGenerationOptions::new()
            .with_target_axis(Axis::linear("x", Unit::Ppm, -1.0, 3.0, 5)?)
            .with_outside_value(-1.0),
    )?;

    assert_eq!(matrix.values, vec![-1.0, 1.0, 3.0, 5.0, -1.0]);
    Ok(())
}

#[test]
fn builder_can_clear_explicit_axis() -> anyhow::Result<()> {
    let spectrum = spectrum("sample", &[0.0, 2.0], &[1.0, 5.0])?;
    let options = MatrixGenerationOptions::new()
        .with_target_axis(Axis::linear("x", Unit::Ppm, -1.0, 3.0, 5)?)
        .without_target_axis();
    let matrix = generate_spectrum_matrix_1d(&[spectrum], options)?;

    assert_eq!(matrix.axis.values, vec![0.0, 2.0]);
    assert_eq!(matrix.values, vec![1.0, 5.0]);
    Ok(())
}

#[test]
fn supports_descending_axes() -> anyhow::Result<()> {
    let first = spectrum("descending", &[2.0, 1.0, 0.0], &[2.0, 4.0, 6.0])?;
    let second = spectrum("other", &[2.0, 0.0], &[10.0, 20.0])?;

    let matrix = generate_spectrum_matrix_1d(&[first, second], MatrixGenerationOptions::default())?;

    assert_eq!(matrix.axis.values, vec![2.0, 1.0, 0.0]);
    assert_eq!(matrix.values, vec![2.0, 4.0, 6.0, 10.0, 15.0, 20.0]);
    Ok(())
}

#[test]
fn rejects_empty_input_and_non_monotonic_axes() -> anyhow::Result<()> {
    let empty_error = generate_spectrum_matrix_1d(&[], MatrixGenerationOptions::default())
        .expect_err("empty input should fail");
    assert!(matches!(empty_error, RSpinError::InvalidSpectrum { .. }));

    let bad = spectrum("bad", &[0.0, 2.0, 1.0], &[1.0, 2.0, 3.0])?;
    let axis_error = generate_spectrum_matrix_1d(&[bad], MatrixGenerationOptions::default())
        .expect_err("non-monotonic axis should fail");
    assert!(matches!(axis_error, RSpinError::InvalidAxis { .. }));
    Ok(())
}

fn spectrum(name: &str, x: &[f64], intensities: &[f64]) -> anyhow::Result<Spectrum1D> {
    Ok(Spectrum1D::new(
        Axis::new("x", Unit::Ppm, x.to_vec())?,
        intensities.to_vec(),
        Metadata::named(name),
    )?)
}
