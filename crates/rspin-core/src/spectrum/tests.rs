use super::*;
use crate::AnnotationTarget;

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
fn attaches_and_validates_1d_annotations() -> Result<()> {
    let spectrum = Spectrum1D::new(
        Axis::linear("shift", Unit::Ppm, 0.0, 2.0, 3)?,
        vec![1.0, 2.0, 3.0],
        Metadata::default(),
    )?
    .with_annotation(
        SpectrumAnnotation::new("peak-1", AnnotationTarget::point_1d(1, 1.0)).with_label("peak"),
    );

    spectrum.validate_annotations()?;
    assert_eq!(
        spectrum
            .annotation("peak-1")
            .and_then(|annotation| annotation.label.as_deref()),
        Some("peak")
    );
    assert_eq!(spectrum.without_annotations().annotations.len(), 0);
    Ok(())
}

#[test]
fn rejects_invalid_1d_annotations() -> Result<()> {
    let duplicate = Spectrum1D::new(
        Axis::linear("shift", Unit::Ppm, 0.0, 2.0, 3)?,
        vec![1.0, 2.0, 3.0],
        Metadata::default(),
    )?
    .with_annotations(vec![
        SpectrumAnnotation::new("a", AnnotationTarget::point_1d(0, 0.0)),
        SpectrumAnnotation::new("a", AnnotationTarget::point_1d(1, 1.0)),
    ]);
    let wrong_dimension = Spectrum1D::new(
        Axis::linear("shift", Unit::Ppm, 0.0, 2.0, 3)?,
        vec![1.0, 2.0, 3.0],
        Metadata::default(),
    )?
    .with_annotation(SpectrumAnnotation::new(
        "zone",
        AnnotationTarget::zone_2d(0.0, 1.0, 0.0, 1.0),
    ));

    assert!(matches!(
        duplicate.validate_annotations(),
        Err(RSpinError::InvalidMetadata { .. })
    ));
    assert!(matches!(
        wrong_dimension.validate_annotations(),
        Err(RSpinError::InvalidMetadata { .. })
    ));
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
fn attaches_and_validates_2d_annotations() -> Result<()> {
    let spectrum = Spectrum2D::new(
        Axis::linear("x", Unit::Ppm, 0.0, 1.0, 2)?,
        Axis::linear("y", Unit::Ppm, 10.0, 11.0, 2)?,
        vec![1.0, 2.0, 3.0, 4.0],
        Metadata::default(),
    )?
    .with_annotation(SpectrumAnnotation::new(
        "cross-peak",
        AnnotationTarget::point_2d(1, 1, 1.0, 11.0),
    ));

    spectrum.validate_annotations()?;
    assert!(spectrum.annotation("cross-peak").is_some());
    Ok(())
}

#[test]
fn rejects_invalid_2d_annotations() -> Result<()> {
    let spectrum = Spectrum2D::new(
        Axis::linear("x", Unit::Ppm, 0.0, 1.0, 2)?,
        Axis::linear("y", Unit::Ppm, 10.0, 11.0, 2)?,
        vec![1.0, 2.0, 3.0, 4.0],
        Metadata::default(),
    )?
    .with_annotation(SpectrumAnnotation::new(
        "range",
        AnnotationTarget::range_1d(0.0, 1.0),
    ));

    assert!(matches!(
        spectrum.validate_annotations(),
        Err(RSpinError::InvalidMetadata { .. })
    ));
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
