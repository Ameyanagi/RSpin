use rspin_core::{Axis, Metadata, Spectrum2D, Unit};

use super::*;

#[test]
fn integrates_full_constant_cell() -> anyhow::Result<()> {
    let spectrum = Spectrum2D::new(
        Axis::linear("x", Unit::Ppm, 0.0, 1.0, 2)?,
        Axis::linear("y", Unit::Ppm, 0.0, 1.0, 2)?,
        vec![1.0, 1.0, 1.0, 1.0],
        Metadata::default(),
    )?;
    let integral = integrate_region_2d(
        &spectrum,
        IntegralRegion2D {
            x_from: 0.0,
            x_to: 1.0,
            y_from: 0.0,
            y_to: 1.0,
        },
    )?;

    assert_close(integral.volume, 1.0);
    assert_eq!(integral.cells, 1);
    Ok(())
}

#[test]
fn integrates_partial_region_with_bilinear_interpolation() -> anyhow::Result<()> {
    let spectrum = plane_spectrum(0.0, 2.0, 0.0, 2.0)?;
    let integral = BilinearIntegrator2D.integrate(
        &spectrum,
        IntegralRegion2D {
            x_from: 0.5,
            x_to: 1.5,
            y_from: 0.5,
            y_to: 1.5,
        },
    )?;

    assert_close(integral.volume, 2.0);
    assert_eq!(integral.cells, 4);
    Ok(())
}

#[test]
fn handles_descending_axes_and_reversed_region() -> anyhow::Result<()> {
    let spectrum = plane_spectrum(2.0, 0.0, 2.0, 0.0)?;
    let integral = integrate_region_2d(
        &spectrum,
        IntegralRegion2D {
            x_from: 1.5,
            x_to: 0.5,
            y_from: 1.5,
            y_to: 0.5,
        },
    )?;

    assert_close(integral.volume, 2.0);
    assert_eq!(integral.cells, 4);
    Ok(())
}

#[test]
fn returns_zero_outside_spectrum_domain() -> anyhow::Result<()> {
    let spectrum = plane_spectrum(0.0, 2.0, 0.0, 2.0)?;
    let integral = integrate_region_2d(
        &spectrum,
        IntegralRegion2D {
            x_from: 3.0,
            x_to: 4.0,
            y_from: 3.0,
            y_to: 4.0,
        },
    )?;

    assert_close(integral.volume, 0.0);
    assert_eq!(integral.cells, 0);
    Ok(())
}

#[test]
fn integrates_multiple_2d_regions_in_order() -> anyhow::Result<()> {
    let spectrum = plane_spectrum(0.0, 2.0, 0.0, 2.0)?;
    let regions = [
        IntegralRegion2D {
            x_from: 0.0,
            x_to: 1.0,
            y_from: 0.0,
            y_to: 1.0,
        },
        IntegralRegion2D {
            x_from: 1.0,
            x_to: 2.0,
            y_from: 1.0,
            y_to: 2.0,
        },
    ];
    let integrals = integrate_regions_2d(&spectrum, &regions)?;

    assert_eq!(integrals.len(), 2);
    assert_close(integrals[0].volume, 1.0);
    assert_close(integrals[1].volume, 3.0);
    assert_eq!(integrals[0].region, regions[0]);
    assert_eq!(integrals[1].region, regions[1]);
    Ok(())
}

#[test]
fn rejects_invalid_region_and_shape() -> anyhow::Result<()> {
    let spectrum = plane_spectrum(0.0, 2.0, 0.0, 2.0)?;
    let error = integrate_region_2d(
        &spectrum,
        IntegralRegion2D {
            x_from: f64::NAN,
            x_to: 1.0,
            y_from: 0.0,
            y_to: 1.0,
        },
    )
    .expect_err("non-finite region should fail");
    assert!(matches!(error, RSpinError::NonFinite { .. }));

    let line = Spectrum2D::new(
        Axis::linear("x", Unit::Ppm, 0.0, 1.0, 2)?,
        Axis::linear("y", Unit::Ppm, 0.0, 0.0, 1)?,
        vec![1.0, 2.0],
        Metadata::default(),
    )?;
    let error = integrate_region_2d(
        &line,
        IntegralRegion2D {
            x_from: 0.0,
            x_to: 1.0,
            y_from: 0.0,
            y_to: 1.0,
        },
    )
    .expect_err("single-row spectrum should fail");
    assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));
    Ok(())
}

fn plane_spectrum(
    x_start: f64,
    x_end: f64,
    y_start: f64,
    y_end: f64,
) -> anyhow::Result<Spectrum2D> {
    let x = Axis::linear("x", Unit::Ppm, x_start, x_end, 3)?;
    let y = Axis::linear("y", Unit::Ppm, y_start, y_end, 3)?;
    let mut z = Vec::with_capacity(x.len() * y.len());
    for y_value in &y.values {
        for x_value in &x.values {
            z.push(x_value + y_value);
        }
    }
    Ok(Spectrum2D::new(x, y, z, Metadata::default())?)
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1.0e-12,
        "{actual} != {expected}"
    );
}
