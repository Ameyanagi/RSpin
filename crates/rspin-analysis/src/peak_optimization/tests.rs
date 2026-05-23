use rspin_core::{Axis, Metadata, Unit};

use super::*;
use crate::{PeakPickOptions, pick_peaks};

#[test]
fn optimizes_positive_peak_position_and_intensity() -> anyhow::Result<()> {
    let spectrum = spectrum(&[0.0, 0.75, 1.0, 0.75, 0.0], 0.2)?;
    let peaks = pick_peaks(&spectrum, PeakPickOptions::default())?;
    let optimized =
        optimize_peaks_quadratic(&spectrum, &peaks, PeakOptimizationOptions::default())?;

    assert_eq!(optimized.len(), 1);
    assert!(optimized[0].optimized);
    assert_close(optimized[0].x, 0.4);
    assert_close(optimized[0].intensity, 1.0);
    assert!(optimized[0].curvature.is_some());
    Ok(())
}

#[test]
fn optimizes_non_uniform_axis() -> anyhow::Result<()> {
    let axis = Axis::new("x", Unit::Ppm, vec![0.0, 0.4, 1.0])?;
    let spectrum = Spectrum1D::new(axis, vec![0.0, 0.96, 0.0], Metadata::default())?;
    let peaks = pick_peaks(&spectrum, PeakPickOptions::default())?;
    let optimized =
        optimize_peaks_quadratic(&spectrum, &peaks, PeakOptimizationOptions::default())?;

    assert_eq!(optimized.len(), 1);
    assert!(optimized[0].optimized);
    assert_close(optimized[0].x, 0.5);
    assert!(optimized[0].intensity > peaks[0].intensity);
    Ok(())
}

#[test]
fn supports_descending_axis() -> anyhow::Result<()> {
    let axis = Axis::new("x", Unit::Ppm, vec![2.0, 1.0, 0.0])?;
    let spectrum = Spectrum1D::new(axis, vec![0.0, 1.0, 0.0], Metadata::default())?;
    let peaks = pick_peaks(&spectrum, PeakPickOptions::default())?;
    let optimized =
        optimize_peaks_quadratic(&spectrum, &peaks, PeakOptimizationOptions::default())?;

    assert_eq!(optimized.len(), 1);
    assert_close(optimized[0].x, 1.0);
    assert!(optimized[0].optimized);
    Ok(())
}

#[test]
fn returns_unoptimized_peak_when_curvature_does_not_match() -> anyhow::Result<()> {
    let spectrum = spectrum(&[2.0, 1.0, 2.0], 1.0)?;
    let peaks = vec![Peak {
        index: 1,
        x: 1.0,
        intensity: 1.0,
        prominence: 0.1,
        polarity: PeakPolarity::Positive,
    }];
    let optimized =
        optimize_peaks_quadratic(&spectrum, &peaks, PeakOptimizationOptions::default())?;

    assert_eq!(optimized.len(), 1);
    assert!(!optimized[0].optimized);
    assert_close(optimized[0].x, peaks[0].x);
    Ok(())
}

#[test]
fn rejects_peak_without_neighbors() -> anyhow::Result<()> {
    let spectrum = spectrum(&[1.0, 0.0, 0.0], 1.0)?;
    let peaks = vec![Peak {
        index: 0,
        x: 0.0,
        intensity: 1.0,
        prominence: 1.0,
        polarity: PeakPolarity::Positive,
    }];
    let error = optimize_peaks_quadratic(&spectrum, &peaks, PeakOptimizationOptions::default())
        .expect_err("endpoint peak should fail");

    assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));
    Ok(())
}

#[test]
fn optimizer_trait_delegates_to_function() -> anyhow::Result<()> {
    let spectrum = spectrum(&[0.0, 1.0, 0.0], 1.0)?;
    let peaks = pick_peaks(&spectrum, PeakPickOptions::default())?;
    let quadratic = QuadraticPeakOptimizer::default();
    let optimized = quadratic.optimize(&spectrum, &peaks)?;

    assert_eq!(optimized.len(), 1);
    assert!(optimized[0].optimized);
    Ok(())
}

fn spectrum(intensities: &[f64], step: f64) -> anyhow::Result<Spectrum1D> {
    let point_count = u32::try_from(intensities.len() - 1)?;
    let axis = Axis::linear(
        "x",
        Unit::Ppm,
        0.0,
        f64::from(point_count) * step,
        intensities.len(),
    )?;
    Ok(Spectrum1D::new(
        axis,
        intensities.to_vec(),
        Metadata::default(),
    )?)
}

fn assert_close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-12, "{actual} != {expected}");
}
