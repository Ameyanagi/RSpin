use rspin_core::{Axis, Metadata, Spectrum2D, Unit};

use super::*;

#[test]
fn extracts_single_cell_contour_segment() -> anyhow::Result<()> {
    let spectrum = spectrum(2, 2, &[0.0, 2.0, 0.0, 2.0])?;
    let segments = contour_segments(&spectrum, 1.0)?;

    assert_eq!(segments.len(), 1);
    assert_close(segments[0].start.x, 0.5);
    assert_close(segments[0].start.y, 0.0);
    assert_close(segments[0].end.x, 0.5);
    assert_close(segments[0].end.y, 1.0);
    assert_close(segments[0].level, 1.0);
    Ok(())
}

#[test]
fn extracts_multiple_levels_in_input_order() -> anyhow::Result<()> {
    let spectrum = spectrum(2, 2, &[0.0, 4.0, 0.0, 4.0])?;
    let contours = extract_contours(&spectrum, &[1.0, 3.0])?;

    assert_eq!(contours.len(), 2);
    assert_close(contours[0].level, 1.0);
    assert_close(contours[1].level, 3.0);
    assert_close(contours[0].segments[0].start.x, 0.25);
    assert_close(contours[1].segments[0].start.x, 0.75);
    Ok(())
}

#[test]
fn splits_ambiguous_cell_deterministically() -> anyhow::Result<()> {
    let spectrum = spectrum(2, 2, &[0.0, 2.0, 2.0, 0.0])?;
    let segments = contour_segments(&spectrum, 1.0)?;

    assert_eq!(segments.len(), 2);
    assert_close(segments[0].start.x, 0.5);
    assert_close(segments[0].start.y, 0.0);
    assert_close(segments[0].end.x, 1.0);
    assert_close(segments[0].end.y, 0.5);
    assert_close(segments[1].start.x, 0.5);
    assert_close(segments[1].start.y, 1.0);
    assert_close(segments[1].end.x, 0.0);
    assert_close(segments[1].end.y, 0.5);
    Ok(())
}

#[test]
fn rejects_invalid_inputs() -> anyhow::Result<()> {
    let one_point = spectrum(1, 1, &[0.0])?;
    let shape_error = contour_segments(&one_point, 1.0).expect_err("shape should fail");
    assert!(matches!(shape_error, RSpinError::InvalidSpectrum { .. }));

    let spectrum = spectrum(2, 2, &[0.0, 1.0, 0.0, 1.0])?;
    let level_error = contour_segments(&spectrum, f64::NAN).expect_err("NaN level should fail");
    assert!(matches!(level_error, RSpinError::NonFinite { .. }));
    Ok(())
}

fn spectrum(width: usize, height: usize, z: &[f64]) -> anyhow::Result<Spectrum2D> {
    let x_end = f64::from(u32::try_from(width - 1)?);
    let y_end = f64::from(u32::try_from(height - 1)?);
    Ok(Spectrum2D::new(
        Axis::linear("x", Unit::Ppm, 0.0, x_end, width)?,
        Axis::linear("y", Unit::Ppm, 0.0, y_end, height)?,
        z.to_vec(),
        Metadata::default(),
    )?)
}

fn assert_close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-12, "{actual} != {expected}");
}
