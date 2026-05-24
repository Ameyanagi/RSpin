use rspin_core::{Axis, Metadata, Unit};

use super::*;
use crate::{PeakPickOptions, pick_peaks};

#[test]
fn groups_nearby_peaks_into_multiplets() -> anyhow::Result<()> {
    let mut spectrum = spectrum(&[
        (0.00, 0.0),
        (1.00, 0.0),
        (1.01, 1.0),
        (1.02, 0.0),
        (1.03, 0.8),
        (1.04, 0.0),
        (1.30, 0.0),
        (1.31, 1.2),
        (1.32, 0.0),
    ])?;
    spectrum.metadata.frequency_mhz = Some(400.0);
    let peaks = pick_peaks(&spectrum, PeakPickOptions::default())?;

    let multiplets = detect_multiplets(
        &spectrum,
        &peaks,
        MultipletDetectionOptions {
            max_peak_gap_ppm: 0.05,
            include_singlets: true,
            ..MultipletDetectionOptions::default()
        },
    )?;

    assert_eq!(multiplets.len(), 2);
    assert_eq!(multiplets[0].id, "multiplet1d:2-4");
    assert_eq!(multiplets[0].kind, MultipletKind::Doublet);
    assert_eq!(multiplets[0].spacings_ppm.len(), 1);
    assert!((multiplets[0].spacings_ppm[0] - 0.02).abs() < 1e-12);
    assert_close(multiplets[0].estimated_j_hz, Some(8.0));
    assert_eq!(multiplets[1].kind, MultipletKind::Singlet);
    Ok(())
}

#[test]
fn can_omit_singlets() -> anyhow::Result<()> {
    let spectrum = spectrum(&[(0.0, 0.0), (1.0, 1.0), (2.0, 0.0)])?;
    let peaks = pick_peaks(&spectrum, PeakPickOptions::default())?;
    let multiplets = GapMultipletDetector::new()
        .with_options(MultipletDetectionOptions::new().with_singlets(false))
        .detect(&spectrum, &peaks)?;

    assert!(multiplets.is_empty());
    Ok(())
}

#[test]
fn builder_options_group_multiplets() -> anyhow::Result<()> {
    let mut spectrum = spectrum(&[
        (0.00, 0.0),
        (1.00, 0.0),
        (1.01, 1.0),
        (1.02, 0.0),
        (1.03, 0.8),
        (1.04, 0.0),
    ])?;
    spectrum.metadata.frequency_mhz = Some(400.0);
    let peaks = pick_peaks(&spectrum, PeakPickOptions::default())?;
    let multiplets = detect_multiplets(
        &spectrum,
        &peaks,
        MultipletDetectionOptions::new()
            .with_max_peak_gap_ppm(0.05)
            .with_min_peak_count(2)
            .with_singlets(true)
            .without_spectrometer_mhz(),
    )?;

    assert_eq!(multiplets.len(), 1);
    assert_eq!(multiplets[0].kind, MultipletKind::Doublet);
    assert_close(multiplets[0].estimated_j_hz, Some(8.0));
    Ok(())
}

#[test]
fn splits_different_peak_polarities() -> anyhow::Result<()> {
    let spectrum = spectrum(&[(0.0, 0.0), (1.0, 1.0), (2.0, 0.0), (3.0, -1.0), (4.0, 0.0)])?;
    let peaks = pick_peaks(
        &spectrum,
        PeakPickOptions {
            polarity: PeakPolarity::Both,
            ..PeakPickOptions::default()
        },
    )?;
    let multiplets = detect_multiplets(
        &spectrum,
        &peaks,
        MultipletDetectionOptions {
            max_peak_gap_ppm: 3.0,
            ..MultipletDetectionOptions::default()
        },
    )?;

    assert_eq!(multiplets.len(), 2);
    assert_eq!(multiplets[0].kind, MultipletKind::Singlet);
    assert_eq!(multiplets[1].kind, MultipletKind::Singlet);
    Ok(())
}

#[test]
fn validates_options_and_duplicate_peaks() -> anyhow::Result<()> {
    let spectrum = spectrum(&[(0.0, 0.0), (1.0, 1.0), (2.0, 0.0)])?;
    let peaks = pick_peaks(&spectrum, PeakPickOptions::default())?;
    let error = detect_multiplets(
        &spectrum,
        &peaks,
        MultipletDetectionOptions {
            max_peak_gap_ppm: -1.0,
            ..MultipletDetectionOptions::default()
        },
    )
    .expect_err("negative gap should fail");
    assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));

    let error = detect_multiplets(
        &spectrum,
        &[peaks[0].clone(), peaks[0].clone()],
        MultipletDetectionOptions::default(),
    )
    .expect_err("duplicate peaks should fail");
    assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));
    Ok(())
}

fn spectrum(points: &[(f64, f64)]) -> anyhow::Result<Spectrum1D> {
    let axis = Axis::new("x", Unit::Ppm, points.iter().map(|point| point.0).collect())?;
    Ok(Spectrum1D::new(
        axis,
        points.iter().map(|point| point.1).collect(),
        Metadata::default(),
    )?)
}

fn assert_close(actual: Option<f64>, expected: Option<f64>) {
    match (actual, expected) {
        (Some(actual), Some(expected)) => {
            assert!((actual - expected).abs() < 1e-12, "{actual} != {expected}");
        }
        (None, None) => {}
        _ => panic!("{actual:?} != {expected:?}"),
    }
}
