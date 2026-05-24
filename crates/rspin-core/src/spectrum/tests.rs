use super::*;

#[test]
fn creates_linear_axis() -> Result<()> {
    let axis = Axis::linear("shift", Unit::Ppm, 10.0, 8.0, 3)?;
    assert_eq!(axis.values, vec![10.0, 9.0, 8.0]);
    Ok(())
}

#[test]
fn creates_ppm_axes_with_helpers() -> Result<()> {
    let axis = Axis::ppm(vec![1.0, 2.0, 3.0])?;
    assert_eq!(axis.label, "chemical shift");
    assert_eq!(axis.unit, Unit::Ppm);
    assert_eq!(axis.values, vec![1.0, 2.0, 3.0]);

    let linear = Axis::linear_ppm(10.0, 8.0, 3)?;
    assert_eq!(linear.label, "chemical shift");
    assert_eq!(linear.unit, Unit::Ppm);
    assert_eq!(linear.values, vec![10.0, 9.0, 8.0]);
    Ok(())
}

#[test]
fn rejects_empty_axis() {
    assert!(Axis::new("x", Unit::Points, Vec::new()).is_err());
}

#[test]
fn creates_1d_spectrum() -> Result<()> {
    let x = Axis::linear("shift", Unit::Ppm, 0.0, 2.0, 3)?;
    let spectrum = Spectrum1D::new(x, vec![1.0, 2.0, 3.0], Metadata::default())?;
    assert_eq!(
        spectrum.points().collect::<Vec<_>>(),
        vec![(0.0, 1.0), (1.0, 2.0), (2.0, 3.0)]
    );
    Ok(())
}

#[test]
fn rejects_mismatched_1d_data() -> Result<()> {
    let x = Axis::linear("shift", Unit::Ppm, 0.0, 2.0, 3)?;
    assert!(Spectrum1D::new(x, vec![1.0, 2.0], Metadata::default()).is_err());
    Ok(())
}

#[test]
fn reads_2d_row_major_values() -> Result<()> {
    let x = Axis::linear("x", Unit::Ppm, 0.0, 1.0, 2)?;
    let y = Axis::linear("y", Unit::Ppm, 10.0, 12.0, 3)?;
    let spectrum = Spectrum2D::new(
        x,
        y,
        vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
        Metadata::default(),
    )?;
    assert_eq!(spectrum.shape(), (2, 3));
    assert_eq!(spectrum.value_at(1, 2), Some(6.0));
    assert_eq!(spectrum.value_at(2, 2), None);
    Ok(())
}

#[test]
fn creates_complex_2d_spectrum() -> Result<()> {
    let spectrum = Spectrum2D::new_complex(
        Axis::linear("x", Unit::Ppm, 0.0, 1.0, 2)?,
        Axis::linear("y", Unit::Ppm, 10.0, 11.0, 2)?,
        vec![1.0, 2.0, 3.0, 4.0],
        Some(vec![0.1, 0.2, 0.3, 0.4]),
        Metadata::default(),
    )?;
    assert_eq!(spectrum.imaginary_at(1, 1), Some(0.4));
    assert_eq!(spectrum.imaginary_at(2, 1), None);
    Ok(())
}

#[test]
fn rejects_mismatched_complex_2d_data() -> Result<()> {
    let x = Axis::linear("x", Unit::Ppm, 0.0, 1.0, 2)?;
    let y = Axis::linear("y", Unit::Ppm, 10.0, 11.0, 2)?;
    assert!(
        Spectrum2D::new_complex(
            x,
            y,
            vec![1.0, 2.0, 3.0, 4.0],
            Some(vec![0.1, 0.2]),
            Metadata::default(),
        )
        .is_err()
    );
    Ok(())
}
