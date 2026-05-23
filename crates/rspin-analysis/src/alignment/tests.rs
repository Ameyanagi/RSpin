use rspin_core::{Axis, Metadata, RSpinError, Spectrum1D, Unit};

use super::*;
use crate::{PeakPickOptions, PeakPolarity};

#[test]
fn aligns_to_first_spectrum_peak_by_default() -> anyhow::Result<()> {
    let first = spectrum("ref", &[0.0, 1.0, 2.0], &[0.0, 5.0, 0.0])?;
    let second = spectrum("shifted", &[0.5, 1.5, 2.5], &[0.0, 7.0, 0.0])?;

    let result = align_spectra_by_peak(&[first, second], PeakAlignmentOptions::default())?;

    assert_eq!(result.shifts.len(), 2);
    assert_close(result.shifts[0].delta, 0.0);
    assert_close(result.shifts[1].delta, -0.5);
    assert_eq!(result.shifts[1].row_id, "1:shifted");
    assert_eq!(result.spectra[1].x.values, vec![0.0, 1.0, 2.0]);
    assert_eq!(
        result.spectra[1].processing[0].operation,
        "align_spectrum_by_peak"
    );
    Ok(())
}

#[test]
fn aligns_to_explicit_target_with_search_window() -> anyhow::Result<()> {
    let spectrum = spectrum(
        "sample a",
        &[0.0, 1.0, 2.0, 3.0, 4.0],
        &[0.0, 10.0, 0.0, 20.0, 0.0],
    )?;
    let options = PeakAlignmentOptions {
        target_x: Some(2.0),
        search_window: Some(AlignmentWindow { from: 0.5, to: 1.5 }),
        peak_options: PeakPickOptions {
            min_abs_intensity: 1.0,
            min_prominence: 1.0,
            polarity: PeakPolarity::Positive,
        },
    };

    let result = align_spectra_by_peak(&[spectrum], options)?;

    assert_close(result.shifts[0].observed_x, 1.0);
    assert_close(result.shifts[0].target_x, 2.0);
    assert_close(result.shifts[0].delta, 1.0);
    assert_eq!(result.spectra[0].x.values, vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    Ok(())
}

#[test]
fn supports_negative_alignment_peaks() -> anyhow::Result<()> {
    let spectrum = spectrum("negative", &[0.0, 1.0, 2.0], &[0.0, -5.0, 0.0])?;
    let result = align_spectra_by_peak(
        &[spectrum],
        PeakAlignmentOptions {
            target_x: Some(0.0),
            peak_options: PeakPickOptions {
                polarity: PeakPolarity::Negative,
                ..PeakPickOptions::default()
            },
            ..PeakAlignmentOptions::default()
        },
    )?;

    assert_close(result.shifts[0].delta, -1.0);
    assert_eq!(result.spectra[0].x.values, vec![-1.0, 0.0, 1.0]);
    Ok(())
}

#[test]
fn rejects_empty_input_and_missing_peaks() -> anyhow::Result<()> {
    let empty_error =
        align_spectra_by_peak(&[], PeakAlignmentOptions::default()).expect_err("empty should fail");
    assert!(matches!(empty_error, RSpinError::InvalidSpectrum { .. }));

    let flat = spectrum("flat", &[0.0, 1.0, 2.0], &[1.0, 1.0, 1.0])?;
    let peak_error = align_spectra_by_peak(&[flat], PeakAlignmentOptions::default())
        .expect_err("missing peak should fail");
    assert!(matches!(peak_error, RSpinError::InvalidSpectrum { .. }));
    Ok(())
}

fn spectrum(name: &str, x: &[f64], intensities: &[f64]) -> anyhow::Result<Spectrum1D> {
    Ok(Spectrum1D::new(
        Axis::new("x", Unit::Ppm, x.to_vec())?,
        intensities.to_vec(),
        Metadata::named(name),
    )?)
}

fn assert_close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-12, "{actual} != {expected}");
}
