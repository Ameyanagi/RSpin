use rspin_core::{Axis, Metadata, RSpinError, Spectrum2D, Unit};

use super::*;

#[test]
fn zero_fills_x_and_y_dimensions() -> anyhow::Result<()> {
    let spectrum = demo_spectrum()?;
    let processed = zero_fill_2d(&spectrum, 4, 3)?;

    assert_eq!(processed.shape(), (4, 3));
    assert_eq!(processed.x.values, vec![0.0, 1.0, 2.0, 3.0]);
    assert_eq!(processed.y.values, vec![10.0, 11.0, 12.0]);
    assert_eq!(
        processed.z,
        vec![
            1.0, 2.0, 3.0, 0.0, //
            4.0, 5.0, 6.0, 0.0, //
            0.0, 0.0, 0.0, 0.0,
        ]
    );
    assert_eq!(processed.processing[0].operation, "zero_fill_2d");
    Ok(())
}

#[test]
fn supports_processing_step_api() -> anyhow::Result<()> {
    let spectrum = demo_spectrum()?;
    let processed = ZeroFill2D {
        target_x_len: 3,
        target_y_len: 4,
    }
    .apply(&spectrum)?;

    assert_eq!(processed.shape(), (3, 4));
    assert_eq!(processed.z[0..6], spectrum.z);
    assert_eq!(processed.z[6..], vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    Ok(())
}

#[test]
fn records_noop_zero_fill() -> anyhow::Result<()> {
    let spectrum = demo_spectrum()?;
    let processed = zero_fill_2d(&spectrum, 3, 2)?;

    assert_eq!(processed.z, spectrum.z);
    assert_eq!(processed.x.values, spectrum.x.values);
    assert_eq!(processed.y.values, spectrum.y.values);
    assert_eq!(processed.processing[0].operation, "zero_fill_2d");
    Ok(())
}

#[test]
fn rejects_smaller_target_dimensions() -> anyhow::Result<()> {
    let spectrum = demo_spectrum()?;
    let x_error = zero_fill_2d(&spectrum, 2, 2).expect_err("smaller x target should fail");
    assert!(matches!(x_error, RSpinError::InvalidSpectrum { .. }));

    let y_error = zero_fill_2d(&spectrum, 3, 1).expect_err("smaller y target should fail");
    assert!(matches!(y_error, RSpinError::InvalidSpectrum { .. }));
    Ok(())
}

#[test]
fn extends_single_point_axes_with_unit_spacing() -> anyhow::Result<()> {
    let spectrum = Spectrum2D::new(
        Axis::linear("x", Unit::Ppm, 5.0, 5.0, 1)?,
        Axis::linear("y", Unit::Ppm, 7.0, 7.0, 1)?,
        vec![2.0],
        Metadata::default(),
    )?;

    let processed = zero_fill_2d(&spectrum, 2, 2)?;

    assert_eq!(processed.x.values, vec![5.0, 6.0]);
    assert_eq!(processed.y.values, vec![7.0, 8.0]);
    assert_eq!(processed.z, vec![2.0, 0.0, 0.0, 0.0]);
    Ok(())
}

fn demo_spectrum() -> anyhow::Result<Spectrum2D> {
    Ok(Spectrum2D::new(
        Axis::linear("x", Unit::Ppm, 0.0, 2.0, 3)?,
        Axis::linear("y", Unit::Ppm, 10.0, 11.0, 2)?,
        vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
        Metadata::named("2d"),
    )?)
}
