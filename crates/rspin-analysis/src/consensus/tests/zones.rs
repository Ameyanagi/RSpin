use rspin_core::{Axis, Metadata, RSpinError, Spectrum2D, Unit};

use crate::{ConsensusZoneOptions, ZoneDetectionOptions, detect_consensus_zones_2d};

use super::assert_close;

#[test]
fn detects_consensus_zones_across_spectra() -> anyhow::Result<()> {
    let spectra = vec![
        spectrum_2d(
            "a",
            0.0,
            0.0,
            3,
            3,
            &[0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0],
        )?,
        spectrum_2d(
            "b",
            0.02,
            0.01,
            3,
            3,
            &[0.0, 0.0, 0.0, 0.0, 4.0, 0.0, 0.0, 0.0, 0.0],
        )?,
    ];

    let zones = detect_consensus_zones_2d(
        &spectra,
        ConsensusZoneOptions::new()
            .with_max_gap(0.05)
            .with_min_spectrum_count(2)
            .with_zone_options(
                ZoneDetectionOptions::new()
                    .with_threshold_abs(1.0)
                    .with_min_active_points(1),
            ),
    )?;

    assert_eq!(zones.len(), 1);
    assert_eq!(zones[0].id, "consensus-zone2d:0");
    assert_eq!(zones[0].zone_count, 2);
    assert_eq!(zones[0].spectrum_count, 2);
    assert_eq!(zones[0].members[0].row_id, "0:a");
    assert_eq!(zones[0].members[1].row_id, "1:b");
    assert_close(zones[0].x_from, 1.0);
    assert_close(zones[0].x_to, 1.02);
    assert_close(zones[0].y_from, 1.0);
    assert_close(zones[0].y_to, 1.01);
    assert_close(zones[0].centroid_x, (1.0 * 2.0 + 1.02 * 4.0) / 6.0);
    assert_close(zones[0].centroid_y, (1.0 * 2.0 + 1.01 * 4.0) / 6.0);
    assert_close(zones[0].total_abs_intensity, 6.0);
    assert_close(zones[0].max_abs_intensity, 4.0);
    Ok(())
}

#[test]
fn filters_consensus_zones_by_spectrum_count_and_gap() -> anyhow::Result<()> {
    let spectra = vec![
        spectrum_2d(
            "a",
            0.0,
            0.0,
            3,
            3,
            &[0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 3.0],
        )?,
        spectrum_2d(
            "b",
            0.02,
            0.01,
            3,
            3,
            &[0.0, 0.0, 0.0, 0.0, 4.0, 0.0, 0.0, 0.0, 0.0],
        )?,
        spectrum_2d(
            "c",
            0.5,
            0.5,
            3,
            3,
            &[0.0, 0.0, 0.0, 0.0, 5.0, 0.0, 0.0, 0.0, 0.0],
        )?,
    ];

    let zones = detect_consensus_zones_2d(
        &spectra,
        ConsensusZoneOptions::new()
            .with_max_x_gap(0.05)
            .with_max_y_gap(0.05)
            .with_min_spectrum_count(2)
            .with_zone_options(ZoneDetectionOptions::new().with_threshold_abs(1.0)),
    )?;

    assert_eq!(zones.len(), 1);
    assert_eq!(zones[0].spectrum_count, 2);
    assert_eq!(zones[0].members.len(), 2);
    Ok(())
}

#[test]
fn returns_empty_when_no_zones_pass_threshold() -> anyhow::Result<()> {
    let spectra = vec![
        spectrum_2d("a", 0.0, 0.0, 2, 2, &[0.0, 0.5, 0.0, 0.0])?,
        spectrum_2d("b", 0.0, 0.0, 2, 2, &[0.0, 0.6, 0.0, 0.0])?,
    ];

    let zones = detect_consensus_zones_2d(
        &spectra,
        ConsensusZoneOptions::new()
            .with_max_gap(0.1)
            .with_zone_options(ZoneDetectionOptions::new().with_threshold_abs(1.0)),
    )?;

    assert!(zones.is_empty());
    Ok(())
}

#[test]
fn rejects_invalid_consensus_zone_inputs() -> anyhow::Result<()> {
    let spectra = vec![spectrum_2d("a", 0.0, 0.0, 2, 2, &[0.0, 1.0, 0.0, 0.0])?];
    let negative_gap_error =
        detect_consensus_zones_2d(&spectra, ConsensusZoneOptions::new().with_max_gap(-1.0))
            .expect_err("negative gap should fail");
    assert!(matches!(
        negative_gap_error,
        RSpinError::InvalidSpectrum { .. }
    ));

    let min_count_error = detect_consensus_zones_2d(
        &spectra,
        ConsensusZoneOptions::new().with_min_spectrum_count(0),
    )
    .expect_err("zero spectrum count should fail");
    assert!(matches!(
        min_count_error,
        RSpinError::InvalidSpectrum { .. }
    ));

    let empty_error = detect_consensus_zones_2d(&[], ConsensusZoneOptions::new())
        .expect_err("empty input should fail");
    assert!(matches!(empty_error, RSpinError::InvalidSpectrum { .. }));

    let mixed_units = vec![
        Spectrum2D::new(
            Axis::linear("x", Unit::Ppm, 0.0, 1.0, 2)?,
            Axis::linear("y", Unit::Ppm, 0.0, 1.0, 2)?,
            vec![0.0, 1.0, 0.0, 0.0],
            Metadata::named("ppm"),
        )?,
        Spectrum2D::new(
            Axis::linear("x", Unit::Hertz, 0.0, 1.0, 2)?,
            Axis::linear("y", Unit::Ppm, 0.0, 1.0, 2)?,
            vec![0.0, 1.0, 0.0, 0.0],
            Metadata::named("hz"),
        )?,
    ];
    let unit_error = detect_consensus_zones_2d(&mixed_units, ConsensusZoneOptions::new())
        .expect_err("mixed units should fail");
    assert!(matches!(unit_error, RSpinError::InvalidSpectrum { .. }));

    Ok(())
}

fn spectrum_2d(
    name: &str,
    x_offset: f64,
    y_offset: f64,
    width: usize,
    height: usize,
    z: &[f64],
) -> anyhow::Result<Spectrum2D> {
    let x_end = x_offset + f64::from(u32::try_from(width - 1)?);
    let y_end = y_offset + f64::from(u32::try_from(height - 1)?);
    Ok(Spectrum2D::new(
        Axis::linear("x", Unit::Ppm, x_offset, x_end, width)?,
        Axis::linear("y", Unit::Ppm, y_offset, y_end, height)?,
        z.to_vec(),
        Metadata::named(name),
    )?)
}
