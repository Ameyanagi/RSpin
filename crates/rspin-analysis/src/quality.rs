//! One-dimensional quality metrics.

use rspin_core::{RSpinError, Result, Spectrum1D};
use serde::{Deserialize, Serialize};

/// Region used for estimating noise from a one-dimensional spectrum.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoiseRegion {
    /// Select points whose x coordinates are inside the inclusive range.
    AxisRange {
        /// First requested x coordinate.
        from: f64,
        /// Second requested x coordinate.
        to: f64,
    },
    /// Select the same fraction of points from both spectrum edges.
    EdgeFraction {
        /// Fraction of the spectrum length used at each edge, in `(0, 0.5]`.
        fraction: f64,
    },
    /// Select a half-open index range `[start, end)`.
    Indices {
        /// First selected index.
        start: usize,
        /// Exclusive end index.
        end: usize,
    },
}

impl Default for NoiseRegion {
    fn default() -> Self {
        Self::EdgeFraction { fraction: 0.05 }
    }
}

/// Region used for measuring signal amplitude.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalRegion {
    /// Select points whose x coordinates are inside the inclusive range.
    AxisRange {
        /// First requested x coordinate.
        from: f64,
        /// Second requested x coordinate.
        to: f64,
    },
    /// Select a half-open index range `[start, end)`.
    Indices {
        /// First selected index.
        start: usize,
        /// Exclusive end index.
        end: usize,
    },
}

/// Noise statistics for a selected region.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NoiseEstimate {
    /// Mean intensity.
    pub mean: f64,
    /// Root-mean-square intensity.
    pub rms: f64,
    /// Population standard deviation around [`Self::mean`].
    pub std_dev: f64,
    /// Difference between the maximum and minimum selected intensities.
    pub peak_to_peak: f64,
    /// Number of points used in the estimate.
    pub n_points: usize,
}

/// Signal-to-noise estimate for a spectrum.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SnrEstimate {
    /// Signed signal intensity at the maximum absolute point in the signal region.
    pub signal_peak: f64,
    /// Absolute signal intensity used for SNR.
    pub signal_peak_abs: f64,
    /// X coordinate of the selected signal point.
    pub signal_x: f64,
    /// Noise estimate used as the denominator.
    pub noise: NoiseEstimate,
    /// `signal_peak_abs / noise.std_dev`.
    pub snr: f64,
}

/// Estimates noise statistics from a selected region.
///
/// # Errors
///
/// Returns an error when the region is invalid, selects no points, or computed
/// statistics are non-finite.
pub fn estimate_noise_1d(spectrum: &Spectrum1D, region: NoiseRegion) -> Result<NoiseEstimate> {
    let indices = noise_indices(spectrum, region)?;
    noise_from_indices(spectrum, &indices)
}

/// Estimates signal-to-noise ratio from explicit signal and noise regions.
///
/// # Errors
///
/// Returns an error when either region is invalid or the noise standard
/// deviation is zero.
pub fn estimate_snr_1d(
    spectrum: &Spectrum1D,
    signal_region: SignalRegion,
    noise_region: NoiseRegion,
) -> Result<SnrEstimate> {
    let signal_indices = signal_indices(spectrum, signal_region)?;
    let (signal_x, signal_peak) = max_abs_signal(spectrum, &signal_indices)?;
    let noise = estimate_noise_1d(spectrum, noise_region)?;
    if noise.std_dev <= 0.0 {
        return Err(RSpinError::InvalidSpectrum {
            message: "SNR requires non-zero noise standard deviation".to_owned(),
        });
    }
    let snr = signal_peak.abs() / noise.std_dev;
    ensure_finite("snr", snr)?;
    Ok(SnrEstimate {
        signal_peak,
        signal_peak_abs: signal_peak.abs(),
        signal_x,
        noise,
        snr,
    })
}

fn noise_indices(spectrum: &Spectrum1D, region: NoiseRegion) -> Result<Vec<usize>> {
    match region {
        NoiseRegion::AxisRange { from, to } => axis_range_indices(spectrum, from, to),
        NoiseRegion::EdgeFraction { fraction } => edge_fraction_indices(spectrum.len(), fraction),
        NoiseRegion::Indices { start, end } => index_range_indices(spectrum.len(), start, end),
    }
}

fn signal_indices(spectrum: &Spectrum1D, region: SignalRegion) -> Result<Vec<usize>> {
    match region {
        SignalRegion::AxisRange { from, to } => axis_range_indices(spectrum, from, to),
        SignalRegion::Indices { start, end } => index_range_indices(spectrum.len(), start, end),
    }
}

fn axis_range_indices(spectrum: &Spectrum1D, from: f64, to: f64) -> Result<Vec<usize>> {
    ensure_finite("region start", from)?;
    ensure_finite("region end", to)?;
    let lower = from.min(to);
    let upper = from.max(to);
    let indices = spectrum
        .x
        .values
        .iter()
        .enumerate()
        .filter_map(|(index, value)| {
            if *value >= lower && *value <= upper {
                Some(index)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    require_non_empty(indices, "axis range selected no points")
}

fn edge_fraction_indices(len: usize, fraction: f64) -> Result<Vec<usize>> {
    ensure_finite("noise edge fraction", fraction)?;
    if !(0.0..=0.5).contains(&fraction) || fraction == 0.0 {
        return Err(RSpinError::InvalidSpectrum {
            message: "noise edge fraction must be in (0, 0.5]".to_owned(),
        });
    }
    if len < 2 {
        return Err(RSpinError::InvalidSpectrum {
            message: "edge-fraction noise estimate requires at least two points".to_owned(),
        });
    }
    let len_f = f64_count(len, "spectrum length")?;
    let mut count = 1_usize;
    while f64_count(count, "noise edge count")? < len_f * fraction {
        count = count
            .checked_add(1)
            .ok_or_else(|| RSpinError::InvalidSpectrum {
                message: "noise edge count is too large".to_owned(),
            })?;
    }
    let total = count
        .checked_mul(2)
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "noise edge count is too large".to_owned(),
        })?;
    if total > len {
        return Err(RSpinError::InvalidSpectrum {
            message: "noise edge fraction selects overlapping edges".to_owned(),
        });
    }
    let mut indices = Vec::with_capacity(total);
    indices.extend(0..count);
    indices.extend((len - count)..len);
    Ok(indices)
}

fn index_range_indices(len: usize, start: usize, end: usize) -> Result<Vec<usize>> {
    if start >= end {
        return Err(RSpinError::InvalidSpectrum {
            message: "index region requires start < end".to_owned(),
        });
    }
    if end > len {
        return Err(RSpinError::InvalidSpectrum {
            message: "index region end exceeds spectrum length".to_owned(),
        });
    }
    Ok((start..end).collect())
}

fn noise_from_indices(spectrum: &Spectrum1D, indices: &[usize]) -> Result<NoiseEstimate> {
    if indices.is_empty() {
        return Err(RSpinError::InvalidSpectrum {
            message: "noise estimate requires at least one point".to_owned(),
        });
    }
    let n = f64_count(indices.len(), "noise sample count")?;
    let mut sum = 0.0;
    let mut sum_sq = 0.0;
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    for index in indices {
        let value = spectrum.intensities[*index];
        sum += value;
        sum_sq += value * value;
        min = min.min(value);
        max = max.max(value);
    }
    let mean = sum / n;
    let rms = (sum_sq / n).sqrt();
    let variance = (sum_sq / n) - mean * mean;
    let std_dev = variance.max(0.0).sqrt();
    let peak_to_peak = max - min;
    ensure_finite("noise mean", mean)?;
    ensure_finite("noise rms", rms)?;
    ensure_finite("noise std_dev", std_dev)?;
    ensure_finite("noise peak_to_peak", peak_to_peak)?;
    Ok(NoiseEstimate {
        mean,
        rms,
        std_dev,
        peak_to_peak,
        n_points: indices.len(),
    })
}

fn max_abs_signal(spectrum: &Spectrum1D, indices: &[usize]) -> Result<(f64, f64)> {
    let mut best: Option<(f64, f64)> = None;
    for index in indices {
        let value = spectrum.intensities[*index];
        let x = spectrum.x.values[*index];
        match best {
            Some((_, best_value)) if value.abs() <= best_value.abs() => {}
            _ => best = Some((x, value)),
        }
    }
    best.ok_or_else(|| RSpinError::InvalidSpectrum {
        message: "signal region selected no points".to_owned(),
    })
}

fn require_non_empty(indices: Vec<usize>, message: &'static str) -> Result<Vec<usize>> {
    if indices.is_empty() {
        return Err(RSpinError::InvalidSpectrum {
            message: message.to_owned(),
        });
    }
    Ok(indices)
}

fn ensure_finite(field: &'static str, value: f64) -> Result<()> {
    if !value.is_finite() {
        return Err(RSpinError::NonFinite { field });
    }
    Ok(())
}

fn f64_count(count: usize, field: &'static str) -> Result<f64> {
    let count = u32::try_from(count).map_err(|_| RSpinError::InvalidSpectrum {
        message: format!("{field} is too large"),
    })?;
    Ok(f64::from(count))
}

#[cfg(test)]
mod tests {
    use rspin_core::{Axis, Metadata, Unit};

    use super::*;

    #[test]
    fn estimates_noise_from_axis_range() -> anyhow::Result<()> {
        let spectrum = demo_spectrum(&[1.0, 2.0, 3.0, 4.0])?;
        let noise = estimate_noise_1d(&spectrum, NoiseRegion::AxisRange { from: 0.0, to: 1.0 })?;

        assert_eq!(noise.n_points, 2);
        assert_close(noise.mean, 1.5, 1.0e-12);
        assert_close(noise.std_dev, 0.5, 1.0e-12);
        assert_close(noise.peak_to_peak, 1.0, 1.0e-12);
        Ok(())
    }

    #[test]
    fn estimates_noise_from_edges() -> anyhow::Result<()> {
        let spectrum = demo_spectrum(&[1.0, 10.0, 20.0, 30.0, 3.0])?;
        let noise = estimate_noise_1d(&spectrum, NoiseRegion::EdgeFraction { fraction: 0.2 })?;

        assert_eq!(noise.n_points, 2);
        assert_close(noise.mean, 2.0, 1.0e-12);
        Ok(())
    }

    #[test]
    fn estimates_snr_from_regions() -> anyhow::Result<()> {
        let spectrum = demo_spectrum(&[-1.0, 1.0, 2.0, 20.0, 3.0, 1.0, -1.0])?;
        let snr = estimate_snr_1d(
            &spectrum,
            SignalRegion::AxisRange { from: 3.0, to: 3.0 },
            NoiseRegion::Indices { start: 0, end: 2 },
        )?;

        assert_close(snr.signal_x, 3.0, 1.0e-12);
        assert_close(snr.signal_peak_abs, 20.0, 1.0e-12);
        assert_close(snr.snr, 20.0, 1.0e-12);
        Ok(())
    }

    #[test]
    fn rejects_zero_noise_for_snr() -> anyhow::Result<()> {
        let spectrum = demo_spectrum(&[1.0, 1.0, 10.0, 1.0])?;
        let error = estimate_snr_1d(
            &spectrum,
            SignalRegion::Indices { start: 2, end: 3 },
            NoiseRegion::Indices { start: 0, end: 2 },
        )
        .expect_err("zero noise should fail");

        assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));
        Ok(())
    }

    fn demo_spectrum(values: &[f64]) -> anyhow::Result<Spectrum1D> {
        let last = u32::try_from(values.len() - 1)?;
        Ok(Spectrum1D::new(
            Axis::linear("shift", Unit::Ppm, 0.0, f64::from(last), values.len())?,
            values.to_vec(),
            Metadata::default(),
        )?)
    }

    fn assert_close(left: f64, right: f64, tolerance: f64) {
        assert!(
            (left - right).abs() <= tolerance,
            "left={left}, right={right}, tolerance={tolerance}"
        );
    }
}
