use rspin_core::{AnnotationTarget, Axis, Metadata, RSpinError, Spectrum1D, Spectrum2D, Unit};

use super::super::{
    annotate_spectrum_1d_with_assignments_json, annotate_spectrum_2d_with_assignments_json,
    spectrum1d_from_json, spectrum2d_from_json, to_json,
};

#[test]
fn annotates_1d_spectrum_with_assignment_json() -> anyhow::Result<()> {
    let spectrum_json = to_json(&Spectrum1D::new(
        Axis::linear("shift", Unit::Ppm, 0.0, 8.0, 9)?,
        vec![0.0; 9],
        Metadata::default(),
    )?)?;
    let annotated_json = annotate_spectrum_1d_with_assignments_json(
        &spectrum_json,
        r#"{"assignments":[{"id":"assign:peak1d:2:H2","target":{"Peak1D":{"index":2,"x":7.12}},"atoms":[{"id":"H2","label":"H-2","nucleus":"Hydrogen1"}],"confidence":0.9,"note":null}]}"#,
    )?;
    let annotated = spectrum1d_from_json(&annotated_json)?;

    assert_eq!(annotated.annotations.len(), 1);
    assert_eq!(annotated.annotations[0].id, "assign:peak1d:2:H2");
    assert_eq!(annotated.annotations[0].label.as_deref(), Some("H-2"));
    assert!(matches!(
        annotated.annotations[0].target,
        AnnotationTarget::Point1D { index: 2, x } if (x - 7.12).abs() < 1.0e-12
    ));
    annotated.validate_annotations()?;
    Ok(())
}

#[test]
fn annotates_2d_spectrum_with_assignment_json() -> anyhow::Result<()> {
    let spectrum_json = to_json(&Spectrum2D::new(
        Axis::linear("x", Unit::Ppm, 0.0, 2.0, 3)?,
        Axis::linear("y", Unit::Ppm, 0.0, 2.0, 3)?,
        vec![0.0; 9],
        Metadata::default(),
    )?)?;
    let annotated_json = annotate_spectrum_2d_with_assignments_json(
        &spectrum_json,
        r#"{"assignments":[{"id":"assign:zone2d:center:H1","target":{"Zone2D":{"id":"zone:x1-1:y1-1"}},"atoms":[{"id":"H1","label":null,"nucleus":"Hydrogen1"}],"confidence":null,"note":null}]}"#,
    )?;
    let annotated = spectrum2d_from_json(&annotated_json)?;

    assert_eq!(annotated.annotations.len(), 1);
    assert_eq!(annotated.annotations[0].label.as_deref(), Some("H1"));
    assert!(matches!(
        &annotated.annotations[0].target,
        AnnotationTarget::Zone2DId { id } if id == "zone:x1-1:y1-1"
    ));
    annotated.validate_annotations()?;
    Ok(())
}

#[test]
fn rejects_assignment_annotation_dimension_mismatch_json() -> anyhow::Result<()> {
    let spectrum_json = to_json(&Spectrum1D::new(
        Axis::linear("shift", Unit::Ppm, 0.0, 2.0, 3)?,
        vec![0.0; 3],
        Metadata::default(),
    )?)?;
    let error = annotate_spectrum_1d_with_assignments_json(
        &spectrum_json,
        r#"{"assignments":[{"id":"assign:zone2d:center:H1","target":{"Zone2D":{"id":"zone:x1-1:y1-1"}},"atoms":[{"id":"H1","label":null,"nucleus":"Hydrogen1"}],"confidence":null,"note":null}]}"#,
    )
    .expect_err("2D assignment target should fail on 1D spectrum");

    assert!(matches!(error, RSpinError::InvalidMetadata { .. }));
    Ok(())
}
