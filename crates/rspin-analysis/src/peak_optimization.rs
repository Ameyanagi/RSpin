//! Peak optimization for one-dimensional spectra.

use serde::{Deserialize, Serialize};

use rspin_core::{RSpinError, Result, Spectrum1D};

use crate::{Peak, PeakOptimizer, PeakPolarity};

/// Options for quadratic peak optimization.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeakOptimizationOptions {
    /// Keep the original peak when the fitted vertex falls outside adjacent samples.
    pub require_vertex_inside: bool,
    /// Require fitted curvature to match the source peak polarity.
    pub require_matching_curvature: bool,
}

impl Default for PeakOptimizationOptions {
    fn default() -> Self {
        Self {
            require_vertex_inside: true,
            require_matching_curvature: true,
        }
    }
}

/// A peak refined by local quadratic interpolation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OptimizedPeak {
    /// Original peak.
    pub peak: Peak,
    /// Refined x-axis coordinate.
    pub x: f64,
    /// Refined intensity.
    pub intensity: f64,
    /// Difference between refined and original x coordinates.
    pub delta_x: f64,
    /// Fitted quadratic curvature coefficient when optimization succeeded.
    pub curvature: Option<f64>,
    /// Whether the quadratic fit was accepted.
    pub optimized: bool,
}

/// Quadratic peak optimizer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuadraticPeakOptimizer {
    /// Optimization options.
    pub options: PeakOptimizationOptions,
}

impl PeakOptimizer for QuadraticPeakOptimizer {
    fn optimize(&self, spectrum: &Spectrum1D, peaks: &[Peak]) -> Result<Vec<OptimizedPeak>> {
        optimize_peaks_quadratic(spectrum, peaks, self.options)
    }
}

/// Refines picked peaks by fitting a quadratic through each peak and its two neighbors.
///
/// The fit uses the physical x-axis values and therefore supports non-uniform
/// and descending axes.
///
/// # Errors
///
/// Returns an error when a peak index cannot be fitted because it is outside the
/// spectrum or lacks left/right neighbors.
pub fn optimize_peaks_quadratic(
    spectrum: &Spectrum1D,
    peaks: &[Peak],
    options: PeakOptimizationOptions,
) -> Result<Vec<OptimizedPeak>> {
    peaks
        .iter()
        .map(|peak| optimize_peak(spectrum, peak, options))
        .collect()
}

fn optimize_peak(
    spectrum: &Spectrum1D,
    peak: &Peak,
    options: PeakOptimizationOptions,
) -> Result<OptimizedPeak> {
    validate_peak_index(spectrum, peak)?;
    let index = peak.index;
    let fit = fit_local_quadratic(
        spectrum.x.values[index - 1],
        spectrum.intensities[index - 1],
        spectrum.x.values[index],
        spectrum.intensities[index],
        spectrum.x.values[index + 1],
        spectrum.intensities[index + 1],
    );

    let Some(fit) = fit else {
        return Ok(unoptimized_peak(peak));
    };

    if options.require_matching_curvature
        && !curvature_matches_polarity(fit.curvature, peak.polarity)
    {
        return Ok(unoptimized_peak(peak));
    }

    if options.require_vertex_inside
        && !inside_interval(
            fit.x,
            spectrum.x.values[index - 1],
            spectrum.x.values[index + 1],
        )
    {
        return Ok(unoptimized_peak(peak));
    }

    Ok(OptimizedPeak {
        peak: peak.clone(),
        x: fit.x,
        intensity: fit.intensity,
        delta_x: fit.x - peak.x,
        curvature: Some(fit.curvature),
        optimized: true,
    })
}

fn validate_peak_index(spectrum: &Spectrum1D, peak: &Peak) -> Result<()> {
    if peak.index >= spectrum.len() {
        return Err(RSpinError::InvalidSpectrum {
            message: "peak index is outside the spectrum".to_owned(),
        });
    }
    if peak.index == 0 || peak.index + 1 >= spectrum.len() {
        return Err(RSpinError::InvalidSpectrum {
            message: "peak optimization requires left and right neighbors".to_owned(),
        });
    }
    Ok(())
}

fn unoptimized_peak(peak: &Peak) -> OptimizedPeak {
    OptimizedPeak {
        peak: peak.clone(),
        x: peak.x,
        intensity: peak.intensity,
        delta_x: 0.0,
        curvature: None,
        optimized: false,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct QuadraticFit {
    x: f64,
    intensity: f64,
    curvature: f64,
}

fn fit_local_quadratic(
    left_x: f64,
    left_y: f64,
    center_x: f64,
    center_y: f64,
    right_x: f64,
    right_y: f64,
) -> Option<QuadraticFit> {
    let left_delta = left_x - center_x;
    let right_delta = right_x - center_x;
    let left_intensity = left_y - center_y;
    let right_intensity = right_y - center_y;
    let determinant = left_delta * right_delta * (left_delta - right_delta);
    if determinant.abs() <= f64::EPSILON || !determinant.is_finite() {
        return None;
    }

    let curvature = (left_intensity * right_delta - right_intensity * left_delta) / determinant;
    let slope =
        (left_delta.powi(2) * right_intensity - right_delta.powi(2) * left_intensity) / determinant;
    if !curvature.is_finite() || !slope.is_finite() || curvature.abs() <= f64::EPSILON {
        return None;
    }

    let vertex_delta = -slope / (2.0 * curvature);
    let x = center_x + vertex_delta;
    let intensity = curvature * vertex_delta.powi(2) + slope * vertex_delta + center_y;
    if !x.is_finite() || !intensity.is_finite() {
        return None;
    }

    Some(QuadraticFit {
        x,
        intensity,
        curvature,
    })
}

fn curvature_matches_polarity(curvature: f64, polarity: PeakPolarity) -> bool {
    match polarity {
        PeakPolarity::Positive => curvature < 0.0,
        PeakPolarity::Negative => curvature > 0.0,
        PeakPolarity::Both => true,
    }
}

fn inside_interval(value: f64, left: f64, right: f64) -> bool {
    let min = left.min(right);
    let max = left.max(right);
    value >= min && value <= max
}

#[cfg(test)]
mod tests;
