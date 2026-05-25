use rspin_core::{Axis, Metadata, RSpinError, Spectrum1D, Unit};

use super::*;

#[test]
fn corrects_zero_order_phase() -> anyhow::Result<()> {
    let phased = phase_correct(&real_spectrum()?, 45.0, 0.0, 0.5)?;
    let result = auto_phase_correct(
        &phased,
        AutoPhaseOptions::default()
            .with_cost(AutoPhaseCost::LegacyImagNegArea)
            .with_refine(false)
            .zero_order_range(-90.0, 90.0, 5.0)
            .first_order_range(0.0, 0.0, 1.0),
    )?;

    assert_close(result.zero_order_deg, -45.0);
    assert_close(result.first_order_deg, 0.0);
    assert_vec_close(&result.spectrum.intensities, &[1.0, 2.0, 1.0]);
    assert_eq!(
        result
            .spectrum
            .processing
            .last()
            .map(|record| record.operation.as_str()),
        Some("auto_phase_correct")
    );
    Ok(())
}

#[test]
fn serializes_auto_phase_result_and_step() -> anyhow::Result<()> {
    let phased = phase_correct(&real_spectrum()?, 45.0, 0.0, 0.5)?;
    let step = AutoPhaseCorrection::new()
        .zero_order_range(-90.0, 90.0, 5.0)
        .first_order_range(0.0, 0.0, 1.0);
    let result = auto_phase_correct(&phased, step.options)?;
    let result_json = serde_json::to_string(&result)?;
    let parsed_result: AutoPhaseResult = serde_json::from_str(&result_json)?;
    let step_json = serde_json::to_string(&step)?;
    let parsed_step: AutoPhaseCorrection = serde_json::from_str(&step_json)?;

    assert_eq!(parsed_result, result);
    assert_eq!(parsed_step, step);
    assert!(result_json.contains("\"zero_order_deg\""));
    assert!(step_json.contains("\"zero_order_min_deg\""));
    Ok(())
}

#[test]
fn corrects_first_order_phase() -> anyhow::Result<()> {
    let spectrum = Spectrum1D::new_complex(
        Axis::linear("shift", Unit::Ppm, 0.0, 2.0, 3)?,
        vec![1.0, 1.0, 1.0],
        Some(vec![0.0, 0.0, 0.0]),
        Metadata::default(),
    )?;
    let phased = phase_correct(&spectrum, 0.0, 60.0, 0.5)?;
    let result = AutoPhaseCorrection::new()
        .zero_order_range(0.0, 0.0, 1.0)
        .first_order_range(-90.0, 90.0, 5.0)
        .pivot_fraction(0.5)
        .apply(&phased)?;

    assert_vec_close(&result.intensities, &[1.0, 1.0, 1.0]);
    Ok(())
}

#[test]
fn rejects_invalid_options() -> anyhow::Result<()> {
    let spectrum = real_spectrum()?;
    let error = auto_phase_correct(
        &spectrum,
        AutoPhaseOptions::default().zero_order_range(10.0, -10.0, 5.0),
    )
    .expect_err("inverted zero-order range should fail");
    assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));

    let error = auto_phase_correct(
        &spectrum,
        AutoPhaseOptions::default().scoring_weights(0.0, 0.0),
    )
    .expect_err("zero scoring weights should fail");
    assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));
    Ok(())
}

#[test]
fn default_options_search_first_order_phase() -> anyhow::Result<()> {
    let spectrum = Spectrum1D::new_complex(
        Axis::linear("shift", Unit::Ppm, 0.0, 4.0, 5)?,
        vec![1.0, 2.0, 4.0, 2.0, 1.0],
        Some(vec![0.0; 5]),
        Metadata::default(),
    )?;
    let phased = phase_correct(&spectrum, 30.0, 60.0, 0.5)?;
    let result = auto_phase_correct(&phased, AutoPhaseOptions::default())?;
    assert!(
        result.first_order_deg.abs() > 1.0e-12,
        "default options must explore first-order phase"
    );
    Ok(())
}

#[test]
fn acme_recovers_zero_order_on_lorentzian() -> anyhow::Result<()> {
    let spectrum = lorentzian_spectrum(64, 0.05)?;
    let phased = phase_correct(&spectrum, 60.0, 0.0, 0.5)?;
    let result = auto_phase_correct(&phased, AutoPhaseOptions::default())?;
    assert!(
        (result.zero_order_deg + 60.0).abs() < 3.0,
        "expected ph0 near -60, got {}",
        result.zero_order_deg
    );
    let max_re = result
        .spectrum
        .intensities
        .iter()
        .copied()
        .fold(0.0_f64, f64::max);
    let min_re = result
        .spectrum
        .intensities
        .iter()
        .copied()
        .fold(0.0_f64, f64::min);
    assert!(max_re > 0.0);
    assert!(
        min_re > -0.05 * max_re,
        "phased spectrum should be mostly positive"
    );
    Ok(())
}

#[test]
fn acme_recovers_combined_phase_on_lorentzian() -> anyhow::Result<()> {
    let spectrum = lorentzian_spectrum(96, 0.04)?;
    let phased = phase_correct(&spectrum, 45.0, 30.0, 0.5)?;
    let result = auto_phase_correct(&phased, AutoPhaseOptions::default())?;
    assert!(
        (result.zero_order_deg + 45.0).abs() < 5.0,
        "expected ph0 near -45, got {}",
        result.zero_order_deg
    );
    assert!(
        (result.first_order_deg + 30.0).abs() < 10.0,
        "expected ph1 near -30, got {}",
        result.first_order_deg
    );
    Ok(())
}

#[test]
fn refinement_improves_grid_score() -> anyhow::Result<()> {
    let spectrum = lorentzian_spectrum(64, 0.05)?;
    let phased = phase_correct(&spectrum, 37.5, 22.5, 0.5)?;
    let without = auto_phase_correct(&phased, AutoPhaseOptions::default().with_refine(false))?;
    let with = auto_phase_correct(&phased, AutoPhaseOptions::default().with_refine(true))?;
    assert!(
        with.score <= without.score + 1.0e-9,
        "refinement should never worsen the score: {} vs {}",
        with.score,
        without.score
    );
    Ok(())
}

#[test]
fn pivot_value_resolves_to_expected_fraction() -> anyhow::Result<()> {
    let spectrum = two_peak_lorentzian_spectrum(192, 0.06)?;
    let pivot_fraction_at_zero = peak_pivot_fraction(&spectrum, 0.0);
    let phased = phase_correct(&spectrum, 0.0, 25.0, pivot_fraction_at_zero)?;
    let result = auto_phase_correct(&phased, AutoPhaseOptions::default().with_pivot_value(0.0))?;
    let max_re = result
        .spectrum
        .intensities
        .iter()
        .copied()
        .fold(0.0_f64, f64::max);
    let min_re = result
        .spectrum
        .intensities
        .iter()
        .copied()
        .fold(0.0_f64, f64::min);
    assert!(max_re > 0.0);
    assert!(
        min_re > -0.1 * max_re,
        "pivot-aware phasing should keep spectrum mostly positive, min={min_re}, max={max_re}"
    );
    Ok(())
}

#[test]
fn active_region_improves_off_center_spectrum() -> anyhow::Result<()> {
    let spectrum = two_peak_lorentzian_spectrum(192, 0.06)?;
    let pivot_fraction_at_zero = peak_pivot_fraction(&spectrum, 0.0);
    let phased = phase_correct(&spectrum, 30.0, 0.0, pivot_fraction_at_zero)?;
    let without = auto_phase_correct(
        &phased,
        AutoPhaseOptions::default()
            .with_pivot_value(0.0)
            .first_order_range(0.0, 0.0, 1.0),
    )?;
    let with = auto_phase_correct(
        &phased,
        AutoPhaseOptions::default()
            .with_pivot_value(0.0)
            .with_active_region(-1.5, 4.0)
            .first_order_range(0.0, 0.0, 1.0),
    )?;
    let neg_fraction = |s: &Spectrum1D| -> f64 {
        let max = s.intensities.iter().copied().fold(0.0_f64, f64::max);
        if max <= 0.0 {
            return 1.0;
        }
        let neg: f64 = s
            .intensities
            .iter()
            .map(|v| if *v < 0.0 { v.abs() } else { 0.0 })
            .sum();
        let total: f64 = s.intensities.iter().map(|v| v.abs()).sum();
        if total <= 0.0 { 0.0 } else { neg / total }
    };
    let neg_without = neg_fraction(&without.spectrum);
    let neg_with = neg_fraction(&with.spectrum);
    assert!(
        neg_with <= neg_without + 1.0e-9,
        "active region should not increase negative-area fraction (with={neg_with}, without={neg_without})"
    );
    Ok(())
}

fn two_peak_lorentzian_spectrum(
    point_count: usize,
    half_width_ppm: f64,
) -> anyhow::Result<Spectrum1D> {
    let segments = u32::try_from(point_count.saturating_sub(1))?;
    let mut real = Vec::with_capacity(point_count);
    let mut imag = Vec::with_capacity(point_count);
    for index in 0..u32::try_from(point_count)? {
        let position = f64::from(index) * 10.0 / f64::from(segments) - 5.0;
        let mut re = 0.0;
        let mut im = 0.0;
        for center in [0.0_f64, 3.0_f64] {
            let x = (position - center) / half_width_ppm;
            let denom = 1.0 + x * x;
            re += 1.0 / denom;
            im += x / denom;
        }
        real.push(re);
        imag.push(im);
    }
    Ok(Spectrum1D::new_complex(
        Axis::linear("shift", Unit::Ppm, -5.0, 5.0, point_count)?,
        real,
        Some(imag),
        Metadata::default(),
    )?)
}

fn peak_pivot_fraction(spectrum: &Spectrum1D, value: f64) -> f64 {
    let first = spectrum.x.values[0];
    let last = spectrum.x.values[spectrum.x.values.len() - 1];
    let range = last - first;
    ((value - first) / range).clamp(0.0, 1.0)
}

fn lorentzian_spectrum(point_count: usize, half_width_ppm: f64) -> anyhow::Result<Spectrum1D> {
    let segments = u32::try_from(point_count.saturating_sub(1))?;
    let mut real = Vec::with_capacity(point_count);
    let mut imag = Vec::with_capacity(point_count);
    for index in 0..u32::try_from(point_count)? {
        let position = f64::from(index) * 10.0 / f64::from(segments) - 5.0;
        let x = position / half_width_ppm;
        let denom = 1.0 + x * x;
        real.push(1.0 / denom);
        imag.push(x / denom);
    }
    Ok(Spectrum1D::new_complex(
        Axis::linear("shift", Unit::Ppm, -5.0, 5.0, point_count)?,
        real,
        Some(imag),
        Metadata::default(),
    )?)
}

fn real_spectrum() -> anyhow::Result<Spectrum1D> {
    Ok(Spectrum1D::new_complex(
        Axis::linear("shift", Unit::Ppm, 0.0, 2.0, 3)?,
        vec![1.0, 2.0, 1.0],
        Some(vec![0.0, 0.0, 0.0]),
        Metadata::default(),
    )?)
}

fn assert_vec_close(actual: &[f64], expected: &[f64]) {
    assert_eq!(actual.len(), expected.len());
    for (left, right) in actual.iter().zip(expected) {
        assert_close(*left, *right);
    }
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1.0e-10,
        "{actual} != {expected}"
    );
}
