use rspin_core::{Axis, Metadata, Spectrum2D, Unit};

use super::super::{from_json, to_json};
use super::*;

#[test]
fn extracts_contours_json() -> anyhow::Result<()> {
    let spectrum = Spectrum2D::new(
        Axis::linear("x", Unit::Ppm, 0.0, 1.0, 2)?,
        Axis::linear("y", Unit::Ppm, 0.0, 1.0, 2)?,
        vec![0.0, 4.0, 0.0, 4.0],
        Metadata::default(),
    )?;
    let contours_json = extract_contours_2d_json(&to_json(&spectrum)?, "[1.0,3.0]")?;
    let contours: Vec<rspin_processing::ContourSet> = from_json(&contours_json)?;

    assert_eq!(contours.len(), 2);
    assert_close(contours[0].level, 1.0);
    assert_close(contours[1].level, 3.0);
    assert_eq!(contours[0].segments.len(), 1);
    assert_eq!(contours[1].segments.len(), 1);
    assert_close(contours[0].segments[0].start.x, 0.25);
    assert_close(contours[1].segments[0].start.x, 0.75);
    Ok(())
}

#[test]
fn rejects_invalid_contour_inputs_json() -> anyhow::Result<()> {
    let one_point = Spectrum2D::new(
        Axis::linear("x", Unit::Ppm, 0.0, 0.0, 1)?,
        Axis::linear("y", Unit::Ppm, 0.0, 0.0, 1)?,
        vec![0.0],
        Metadata::default(),
    )?;
    let error = extract_contours_2d_json(&to_json(&one_point)?, "[1.0]")
        .expect_err("one-point spectrum should fail");
    assert!(error.to_string().contains("contour extraction"));

    let spectrum = Spectrum2D::new(
        Axis::linear("x", Unit::Ppm, 0.0, 1.0, 2)?,
        Axis::linear("y", Unit::Ppm, 0.0, 1.0, 2)?,
        vec![0.0, 1.0, 0.0, 1.0],
        Metadata::default(),
    )?;
    let error = extract_contours_2d_json(&to_json(&spectrum)?, r#""not-levels""#)
        .expect_err("non-array levels should fail");
    assert!(error.to_string().contains("JSON"));
    Ok(())
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1.0e-12,
        "{actual} != {expected}"
    );
}
