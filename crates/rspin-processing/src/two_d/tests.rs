use rspin_core::{Axis, Metadata, RSpinError, Spectrum1D, Spectrum2D, Unit};

use super::*;

#[test]
fn scales_2d_values() -> anyhow::Result<()> {
    let spectrum = demo_spectrum()?;
    let processed = Scale2D { factor: 2.0 }.apply(&spectrum)?;
    assert_eq!(processed.z, vec![2.0, -4.0, 6.0, 8.0, -10.0, 12.0]);
    assert_eq!(processed.processing[0].operation, "scale_2d");
    Ok(())
}

#[test]
fn scales_complex_2d_values() -> anyhow::Result<()> {
    let spectrum = demo_complex_spectrum()?;
    let processed = Scale2D { factor: 2.0 }.apply(&spectrum)?;
    assert_eq!(processed.z, vec![2.0, -4.0, 6.0, 8.0, -10.0, 12.0]);
    assert_eq!(
        require_imaginary_2d(&processed)?,
        &[20.0, 40.0, 60.0, 80.0, 100.0, 120.0]
    );
    Ok(())
}

#[test]
fn normalizes_2d_values() -> anyhow::Result<()> {
    let spectrum = demo_spectrum()?;
    let processed = Normalize2DMaxAbs.apply(&spectrum)?;
    assert_vec_close(
        &processed.z,
        &[1.0 / 6.0, -2.0 / 6.0, 3.0 / 6.0, 4.0 / 6.0, -5.0 / 6.0, 1.0],
    );
    Ok(())
}

#[test]
fn projects_x_and_y() -> anyhow::Result<()> {
    let spectrum = demo_spectrum()?;
    let x_projection = project_x(&spectrum, ProjectionMode::Sum)?;
    let y_projection = project_y(&spectrum, ProjectionMode::Mean)?;
    assert_eq!(x_projection.intensities, vec![5.0, -7.0, 9.0]);
    assert_eq!(y_projection.intensities, vec![2.0 / 3.0, 5.0 / 3.0]);
    Ok(())
}

#[test]
fn projects_complex_2d_values() -> anyhow::Result<()> {
    let spectrum = demo_complex_spectrum()?;
    let x_projection = project_x(&spectrum, ProjectionMode::Sum)?;
    let y_projection = project_y(&spectrum, ProjectionMode::Mean)?;

    assert_eq!(x_projection.intensities, vec![5.0, -7.0, 9.0]);
    assert_eq!(require_imaginary(&x_projection)?, &[50.0, 70.0, 90.0]);
    assert_eq!(y_projection.intensities, vec![2.0 / 3.0, 5.0 / 3.0]);
    assert_eq!(require_imaginary(&y_projection)?, &[20.0, 50.0]);
    Ok(())
}

#[test]
fn projects_max_abs_with_sign() -> anyhow::Result<()> {
    let spectrum = demo_spectrum()?;
    let projection = project_x(&spectrum, ProjectionMode::MaxAbs)?;
    assert_eq!(projection.intensities, vec![4.0, -5.0, 6.0]);
    Ok(())
}

#[test]
fn projects_complex_max_abs_with_selected_imaginary_value() -> anyhow::Result<()> {
    let spectrum = demo_complex_spectrum()?;
    let projection = project_x(&spectrum, ProjectionMode::MaxAbs)?;
    assert_eq!(projection.intensities, vec![4.0, -5.0, 6.0]);
    assert_eq!(require_imaginary(&projection)?, &[40.0, 50.0, 60.0]);
    Ok(())
}

#[test]
fn extracts_row_and_column_slices() -> anyhow::Result<()> {
    let spectrum = demo_spectrum()?;
    let row = slice_x_at_y_index(&spectrum, 1)?;
    let column = slice_y_at_x_index(&spectrum, 1)?;
    assert_eq!(row.intensities, vec![4.0, -5.0, 6.0]);
    assert_eq!(row.x.values, spectrum.x.values);
    assert_eq!(column.intensities, vec![-2.0, -5.0]);
    assert_eq!(column.x.values, spectrum.y.values);
    Ok(())
}

#[test]
fn extracts_complex_row_and_column_slices() -> anyhow::Result<()> {
    let spectrum = demo_complex_spectrum()?;
    let row = slice_x_at_y_index(&spectrum, 1)?;
    let column = slice_y_at_x_index(&spectrum, 1)?;
    assert_eq!(row.intensities, vec![4.0, -5.0, 6.0]);
    assert_eq!(require_imaginary(&row)?, &[40.0, 50.0, 60.0]);
    assert_eq!(column.intensities, vec![-2.0, -5.0]);
    assert_eq!(require_imaginary(&column)?, &[20.0, 50.0]);
    Ok(())
}

#[test]
fn extracts_nearest_coordinate_slices() -> anyhow::Result<()> {
    let spectrum = demo_complex_spectrum()?;
    let row = slice_x_at_y(&spectrum, 10.6)?;
    let column = slice_y_at_x(&spectrum, 1.6)?;

    assert_eq!(row.intensities, vec![4.0, -5.0, 6.0]);
    assert_eq!(row.x.values, spectrum.x.values);
    assert_eq!(
        row.processing
            .last()
            .map(|record| record.operation.as_str()),
        Some("slice_x_at_y")
    );
    assert_eq!(column.intensities, vec![3.0, 6.0]);
    assert_eq!(column.x.values, spectrum.y.values);
    assert_eq!(
        column
            .processing
            .last()
            .map(|record| record.operation.as_str()),
        Some("slice_y_at_x")
    );
    Ok(())
}

#[test]
fn rejects_out_of_bounds_slice() -> anyhow::Result<()> {
    let spectrum = demo_spectrum()?;
    let error = slice_y_at_x_index(&spectrum, 3).expect_err("x index should be out of bounds");
    assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));
    Ok(())
}

#[test]
fn rejects_non_finite_coordinate_slice() -> anyhow::Result<()> {
    let spectrum = demo_spectrum()?;
    let error = slice_x_at_y(&spectrum, f64::NAN).expect_err("non-finite y should fail");
    assert!(matches!(
        error,
        RSpinError::NonFinite {
            field: "y coordinate"
        }
    ));
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

fn demo_complex_spectrum() -> anyhow::Result<Spectrum2D> {
    Ok(Spectrum2D::new_complex(
        Axis::linear("x", Unit::Ppm, 0.0, 2.0, 3)?,
        Axis::linear("y", Unit::Ppm, 10.0, 11.0, 2)?,
        vec![1.0, -2.0, 3.0, 4.0, -5.0, 6.0],
        Some(vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0]),
        Metadata::named("2d"),
    )?)
}

fn require_imaginary(spectrum: &Spectrum1D) -> anyhow::Result<&[f64]> {
    match &spectrum.imaginary {
        Some(imaginary) => Ok(imaginary),
        None => anyhow::bail!("missing imaginary channel"),
    }
}

fn require_imaginary_2d(spectrum: &Spectrum2D) -> anyhow::Result<&[f64]> {
    match &spectrum.imaginary {
        Some(imaginary) => Ok(imaginary),
        None => anyhow::bail!("missing imaginary channel"),
    }
}

fn assert_vec_close(actual: &[f64], expected: &[f64]) {
    assert_eq!(actual.len(), expected.len());
    for (left, right) in actual.iter().zip(expected) {
        assert!((left - right).abs() < 1e-12, "{left} != {right}");
    }
}
