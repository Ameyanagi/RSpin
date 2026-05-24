use rspin_core::{Axis, Metadata, RSpinError, Spectrum1D, Unit};

use super::*;
use crate::{PeakPickOptions, PeakPolarity, RangeDetectionOptions};

#[test]
fn detects_consensus_peaks_across_spectra() -> anyhow::Result<()> {
    let spectra = vec![
        spectrum("a", 0.0, &[0.0, 5.0, 0.0, 3.0, 0.0])?,
        spectrum("b", 0.02, &[0.0, 4.0, 0.0, 6.0, 0.0])?,
    ];

    let peaks = detect_consensus_peaks_1d(
        &spectra,
        ConsensusPeakOptions::new()
            .with_max_shift(0.05)
            .with_min_spectrum_count(2)
            .with_peak_options(PeakPickOptions::new().with_min_abs_intensity(1.0)),
    )?;

    assert_eq!(peaks.len(), 2);
    assert_eq!(peaks[0].id, "consensus1d:0");
    assert_eq!(peaks[0].peak_count, 2);
    assert_eq!(peaks[0].spectrum_count, 2);
    assert_eq!(peaks[0].members[0].row_id, "0:a");
    assert_eq!(peaks[0].members[1].row_id, "1:b");
    assert_close(peaks[0].from_x, 1.0);
    assert_close(peaks[0].to_x, 1.02);
    assert_close(peaks[0].center_x, (1.0 * 5.0 + 1.02 * 4.0) / 9.0);
    assert_close(peaks[1].center_x, (3.0 * 3.0 + 3.02 * 6.0) / 9.0);
    Ok(())
}

#[test]
fn filters_by_spectrum_count_and_separates_polarity() -> anyhow::Result<()> {
    let spectra = vec![
        spectrum("a", 0.0, &[0.0, 5.0, 0.0, -4.0, 0.0])?,
        spectrum("b", 0.02, &[0.0, 4.0, 0.0, 0.0, 0.0])?,
        spectrum("c", 0.04, &[0.0, 3.0, 0.0, 0.0, 0.0])?,
    ];

    let peaks = detect_consensus_peaks_1d(
        &spectra,
        ConsensusPeakOptions::new()
            .with_max_shift(0.08)
            .with_min_spectrum_count(2)
            .with_peak_options(
                PeakPickOptions::new()
                    .with_min_abs_intensity(1.0)
                    .with_polarity(PeakPolarity::Both),
            ),
    )?;

    assert_eq!(peaks.len(), 1);
    assert_eq!(peaks[0].spectrum_count, 3);
    assert_eq!(peaks[0].members.len(), 3);
    assert!(
        peaks[0]
            .members
            .iter()
            .all(|member| member.peak.polarity == PeakPolarity::Positive)
    );
    Ok(())
}

#[test]
fn returns_empty_when_no_peaks_pass_threshold() -> anyhow::Result<()> {
    let spectra = vec![
        spectrum("a", 0.0, &[0.0, 0.5, 0.0])?,
        spectrum("b", 0.0, &[0.0, 0.6, 0.0])?,
    ];

    let peaks = detect_consensus_peaks_1d(
        &spectra,
        ConsensusPeakOptions::new()
            .with_max_shift(0.1)
            .with_peak_options(PeakPickOptions::new().with_min_abs_intensity(1.0)),
    )?;

    assert!(peaks.is_empty());
    Ok(())
}

#[test]
fn detects_consensus_ranges_across_spectra() -> anyhow::Result<()> {
    let spectra = vec![
        spectrum("a", 0.0, &[0.0, 2.0, 3.0, 0.0, 0.0])?,
        spectrum("b", 0.02, &[0.0, 4.0, 5.0, 0.0, 0.0])?,
    ];

    let ranges = detect_consensus_ranges_1d(
        &spectra,
        ConsensusRangeOptions::new()
            .with_max_gap(0.05)
            .with_min_spectrum_count(2)
            .with_range_options(
                RangeDetectionOptions::new()
                    .with_threshold_abs(1.0)
                    .with_min_active_points(1),
            ),
    )?;

    assert_eq!(ranges.len(), 1);
    assert_eq!(ranges[0].id, "consensus-range1d:0");
    assert_eq!(ranges[0].range_count, 2);
    assert_eq!(ranges[0].spectrum_count, 2);
    assert_eq!(ranges[0].members[0].row_id, "0:a");
    assert_eq!(ranges[0].members[1].row_id, "1:b");
    assert_close(ranges[0].from, 1.0);
    assert_close(ranges[0].to, 2.02);
    assert_close(ranges[0].max_abs_intensity, 5.0);
    assert!(ranges[0].total_abs_area > 0.0);
    assert!(ranges[0].center_x > 1.0 && ranges[0].center_x < 2.02);
    Ok(())
}

#[test]
fn filters_consensus_ranges_by_spectrum_count_and_gap() -> anyhow::Result<()> {
    let spectra = vec![
        spectrum("a", 0.0, &[0.0, 2.0, 3.0, 0.0, 5.0, 0.0])?,
        spectrum("b", 0.02, &[0.0, 4.0, 5.0, 0.0, 0.0, 0.0])?,
        spectrum("c", 0.5, &[0.0, 0.0, 0.0, 6.0, 0.0, 0.0])?,
    ];

    let ranges = detect_consensus_ranges_1d(
        &spectra,
        ConsensusRangeOptions::new()
            .with_max_gap(0.05)
            .with_min_spectrum_count(2)
            .with_range_options(RangeDetectionOptions::new().with_threshold_abs(1.0)),
    )?;

    assert_eq!(ranges.len(), 1);
    assert_eq!(ranges[0].spectrum_count, 2);
    assert_eq!(ranges[0].members.len(), 2);
    Ok(())
}

#[test]
fn returns_empty_when_no_ranges_pass_threshold() -> anyhow::Result<()> {
    let spectra = vec![
        spectrum("a", 0.0, &[0.0, 0.5, 0.0])?,
        spectrum("b", 0.0, &[0.0, 0.6, 0.0])?,
    ];

    let ranges = detect_consensus_ranges_1d(
        &spectra,
        ConsensusRangeOptions::new()
            .with_max_gap(0.1)
            .with_range_options(RangeDetectionOptions::new().with_threshold_abs(1.0)),
    )?;

    assert!(ranges.is_empty());
    Ok(())
}

#[test]
fn rejects_invalid_consensus_range_inputs() -> anyhow::Result<()> {
    let spectra = vec![spectrum("a", 0.0, &[0.0, 1.0, 0.0])?];
    let negative_gap_error =
        detect_consensus_ranges_1d(&spectra, ConsensusRangeOptions::new().with_max_gap(-1.0))
            .expect_err("negative gap should fail");
    assert!(matches!(
        negative_gap_error,
        RSpinError::InvalidSpectrum { .. }
    ));

    let empty_error = detect_consensus_ranges_1d(&[], ConsensusRangeOptions::new())
        .expect_err("empty input should fail");
    assert!(matches!(empty_error, RSpinError::InvalidSpectrum { .. }));

    let mixed_units = vec![
        Spectrum1D::new(
            Axis::linear("x", Unit::Ppm, 0.0, 2.0, 3)?,
            vec![0.0, 1.0, 0.0],
            Metadata::named("ppm"),
        )?,
        Spectrum1D::new(
            Axis::linear("x", Unit::Hertz, 0.0, 2.0, 3)?,
            vec![0.0, 1.0, 0.0],
            Metadata::named("hz"),
        )?,
    ];
    let unit_error = detect_consensus_ranges_1d(&mixed_units, ConsensusRangeOptions::new())
        .expect_err("mixed units should fail");
    assert!(matches!(unit_error, RSpinError::InvalidSpectrum { .. }));

    Ok(())
}

#[test]
fn rejects_invalid_consensus_inputs() -> anyhow::Result<()> {
    let spectra = vec![spectrum("a", 0.0, &[0.0, 1.0, 0.0])?];
    let negative_shift_error =
        detect_consensus_peaks_1d(&spectra, ConsensusPeakOptions::new().with_max_shift(-1.0))
            .expect_err("negative shift should fail");
    assert!(matches!(
        negative_shift_error,
        RSpinError::InvalidSpectrum { .. }
    ));

    let empty_error = detect_consensus_peaks_1d(&[], ConsensusPeakOptions::new())
        .expect_err("empty input should fail");
    assert!(matches!(empty_error, RSpinError::InvalidSpectrum { .. }));

    let mixed_units = vec![
        Spectrum1D::new(
            Axis::linear("x", Unit::Ppm, 0.0, 2.0, 3)?,
            vec![0.0, 1.0, 0.0],
            Metadata::named("ppm"),
        )?,
        Spectrum1D::new(
            Axis::linear("x", Unit::Hertz, 0.0, 2.0, 3)?,
            vec![0.0, 1.0, 0.0],
            Metadata::named("hz"),
        )?,
    ];
    let unit_error = detect_consensus_peaks_1d(&mixed_units, ConsensusPeakOptions::new())
        .expect_err("mixed units should fail");
    assert!(matches!(unit_error, RSpinError::InvalidSpectrum { .. }));

    Ok(())
}

fn spectrum(name: &str, offset: f64, intensities: &[f64]) -> anyhow::Result<Spectrum1D> {
    let end = offset + f64::from(u32::try_from(intensities.len() - 1)?);
    Ok(Spectrum1D::new(
        Axis::linear("x", Unit::Ppm, offset, end, intensities.len())?,
        intensities.to_vec(),
        Metadata::named(name),
    )?)
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1.0e-12,
        "{actual} != {expected}"
    );
}
