use rspin_core::{Axis, Metadata, RSpinError, Spectrum2D, Unit};

use super::*;

#[test]
fn detects_four_connected_zones() -> anyhow::Result<()> {
    let spectrum = spectrum(
        4,
        3,
        &[
            0.0, 2.0, 2.5, 0.0, //
            0.0, 2.2, 0.0, 0.0, //
            0.0, 0.0, 0.0, -4.0,
        ],
    )?;
    let zones = detect_zones(
        &spectrum,
        ZoneDetectionOptions {
            threshold_abs: 1.0,
            min_active_points: 1,
            connectivity: ZoneConnectivity::Four,
        },
    )?;

    assert_eq!(zones.len(), 2);
    assert_eq!(zones[0].id, "zone:x1-2:y0-1");
    assert_eq!(zones[0].active_points, 3);
    assert_eq!(zones[0].x_start_index, 1);
    assert_eq!(zones[0].x_end_index, 2);
    assert_eq!(zones[0].y_start_index, 0);
    assert_eq!(zones[0].y_end_index, 1);
    assert_close(zones[0].max_abs_intensity, 2.5);
    assert_close(zones[0].sum_intensity, 6.7);
    assert_close(zones[1].sum_abs_intensity, 4.0);
    Ok(())
}

#[test]
fn eight_connectivity_merges_diagonal_points() -> anyhow::Result<()> {
    let spectrum = spectrum(
        3,
        3,
        &[
            3.0, 0.0, 0.0, //
            0.0, 4.0, 0.0, //
            0.0, 0.0, 5.0,
        ],
    )?;
    let zones = ThresholdZoneDetector::new()
        .with_options(
            ZoneDetectionOptions::new()
                .with_threshold_abs(1.0)
                .with_min_active_points(1)
                .with_connectivity(ZoneConnectivity::Eight),
        )
        .detect(&spectrum)?;

    assert_eq!(zones.len(), 1);
    assert_eq!(zones[0].active_points, 3);
    assert_eq!(zones[0].id, "zone:x0-2:y0-2");
    assert_close(zones[0].centroid_x, 7.0 / 6.0);
    assert_close(zones[0].centroid_y, 7.0 / 6.0);
    Ok(())
}

#[test]
fn filters_small_zones() -> anyhow::Result<()> {
    let spectrum = spectrum(
        3,
        2,
        &[
            2.0, 0.0, 0.0, //
            0.0, 3.0, 4.0,
        ],
    )?;
    let zones = detect_zones(
        &spectrum,
        ZoneDetectionOptions {
            threshold_abs: 1.0,
            min_active_points: 2,
            connectivity: ZoneConnectivity::Four,
        },
    )?;

    assert_eq!(zones.len(), 1);
    assert_eq!(zones[0].id, "zone:x1-2:y1-1");
    Ok(())
}

#[test]
fn rejects_invalid_options() -> anyhow::Result<()> {
    let spectrum = spectrum(2, 2, &[0.0, 1.0, 0.0, 0.0])?;
    let error = detect_zones(
        &spectrum,
        ZoneDetectionOptions {
            threshold_abs: -1.0,
            ..ZoneDetectionOptions::default()
        },
    )
    .expect_err("negative threshold should fail");
    assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));
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
