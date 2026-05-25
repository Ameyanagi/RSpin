// The implementation faithfully mirrors the multi-stage pipeline described
// in the Zorin et al. paper; flattening it to satisfy clippy's stylistic
// nags hurts readability without changing semantics.
#![allow(
    clippy::similar_names,
    clippy::if_same_then_else,
    clippy::redundant_closure_for_method_calls,
    clippy::manual_midpoint,
    clippy::too_many_lines,
    clippy::needless_range_loop,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::needless_late_init,
    clippy::manual_unwrap_or
)]

//! Region-based automatic phase correction (Zorin, Bernstein, Cobas 2017).
//!
//! Clean-room implementation of the "Regions" algorithm described in
//!
//! > V. Zorin, M. A. Bernstein, C. Cobas.
//! > *A robust, general automatic phase correction algorithm for
//! > high-resolution NMR data.*
//! > Magn. Reson. Chem. 55 (2017) 738–746. DOI: 10.1002/mrc.4586.
//!
//! Outline of the algorithm (paper's stage numbering preserved):
//!
//! 1. Build a peak/baseline binary map from a smooth-noise derivative of
//!    the magnitude spectrum (Holoborodko 5-tap filter, eq. 5 in the paper).
//! 2. Estimate the noise autocorrelation distance and use it to suppress
//!    spurious peaks caused by zero-fill or apodization correlations.
//! 3. Cluster peak indices into contiguous regions separated by at least
//!    ~0.1 ppm of baseline; widen each region edge by ~0.05 ppm.
//! 4. For every region, find the zero-order phase that minimises the area
//!    below the linear baseline through the region endpoints (3 cycles of
//!    20-point coarse-to-fine search).
//! 5. Compute the global `(ph0, ph1)` by intensity-weighted linear
//!    regression of region phases vs. their positions, with iterative
//!    outlier rejection at 0.6 rad. When the regression `R²` is below
//!    0.2 the linear term is dropped and the weighted mean of region
//!    phases is used as `ph0`.
//!
//! The implementation here only depends on `rspin-core` types and the
//! magnitude / complex buffer helpers in this crate.

use rustfft::num_complex::Complex;

use rspin_core::{ProcessingRecord, RSpinError, Result, Spectrum1D};

use super::{PhaseCandidate, complex_buffer};
use crate::phase_correct;

/// Result returned by [`auto_phase_correct_regions`].
#[derive(Clone, Debug, PartialEq)]
pub struct RegionsResult {
    /// Phased spectrum.
    pub spectrum: Spectrum1D,
    /// Fitted global zero-order phase in degrees.
    pub zero_order_deg: f64,
    /// Fitted global first-order phase in degrees across the full spectrum.
    pub first_order_deg: f64,
    /// Pivot fraction used by [`phase_correct`].
    pub pivot_fraction: f64,
    /// Number of peak regions retained in the final fit.
    pub region_count: usize,
    /// `R²` of the final weighted linear regression (0 when only one region
    /// or when the regression is degenerate).
    pub regression_r_squared: f64,
}

/// Configuration knobs for the Regions algorithm.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RegionsOptions {
    /// Minimum width (in points) of a baseline gap that splits two peak
    /// blocks. Approximates the paper's "0.1 ppm at 500 MHz" rule.
    pub min_baseline_gap: usize,
    /// Extra points added on each side of every region when converting the
    /// binary map into peak windows (the paper widens edges by ~0.05 ppm).
    pub region_edge_widening: usize,
    /// `R²` threshold below which the first-order term is forced to zero.
    pub r_squared_drop_threshold: f64,
    /// Maximum residual (degrees) at which a region is kept in the
    /// linear-regression fit. Translates the paper's 0.6 rad threshold.
    pub outlier_threshold_deg: f64,
    /// Pivot fraction passed to [`phase_correct`] when reconstructing
    /// the final spectrum.
    pub pivot_fraction: f64,
}

impl Default for RegionsOptions {
    fn default() -> Self {
        Self {
            min_baseline_gap: 8,
            region_edge_widening: 4,
            r_squared_drop_threshold: 0.2,
            outlier_threshold_deg: 0.6_f64.to_degrees(),
            pivot_fraction: 0.5,
        }
    }
}

impl RegionsOptions {
    /// Returns options with a custom minimum baseline gap (in points).
    #[must_use]
    pub fn with_min_baseline_gap(mut self, gap: usize) -> Self {
        self.min_baseline_gap = gap;
        self
    }

    /// Returns options with a custom region-edge widening (in points).
    #[must_use]
    pub fn with_edge_widening(mut self, edge: usize) -> Self {
        self.region_edge_widening = edge;
        self
    }

    /// Returns options with a custom pivot fraction.
    #[must_use]
    pub fn with_pivot_fraction(mut self, pivot: f64) -> Self {
        self.pivot_fraction = pivot;
        self
    }
}

/// Auto-phases a complex spectrum using the Zorin et al. 2017 Regions
/// algorithm.
///
/// # Errors
///
/// Returns an error when the spectrum has fewer than 16 points or when no
/// reliable peak region can be detected.
pub fn auto_phase_correct_regions(
    spectrum: &Spectrum1D,
    options: RegionsOptions,
) -> Result<RegionsResult> {
    if spectrum.len() < 16 {
        return Err(RSpinError::InvalidSpectrum {
            message: "regions auto-phase requires at least 16 points".to_owned(),
        });
    }
    let buffer = complex_buffer(spectrum);
    let magnitude: Vec<f64> = buffer.iter().map(|c| c.norm()).collect();

    // ── Stage 1: smooth-noise derivative and 3-sigma thresholding ──
    let derivative = holoborodko_derivative(&magnitude);
    let derivative_threshold = iterative_three_sigma(&derivative);
    let mut peak_map: Vec<bool> = derivative
        .iter()
        .map(|value| value.abs() >= derivative_threshold)
        .collect();
    // The derivative misses "plateau" peaks: re-threshold the magnitude
    // itself to recover them.
    let magnitude_threshold = iterative_three_sigma(&magnitude);
    for (flag, value) in peak_map.iter_mut().zip(&magnitude) {
        if *value >= magnitude_threshold {
            *flag = true;
        }
    }

    // ── Stage 2: autocorrelation-distance suppression of spurious peaks ──
    let autocorrelation = noise_autocorrelation_distance(&magnitude, &peak_map);
    drop_narrow_blocks(&mut peak_map, autocorrelation);

    // ── Stage 3: cluster the binary map into peak regions ──
    let regions = build_regions(
        &peak_map,
        options.min_baseline_gap,
        options.region_edge_widening,
        magnitude.len(),
    );
    if regions.is_empty() {
        return Err(RSpinError::InvalidSpectrum {
            message: "regions auto-phase found no peak regions".to_owned(),
        });
    }

    // ── Stage 4: per-region zero-order phase via area-below-baseline ──
    let region_phases: Vec<RegionPhase> = regions
        .iter()
        .filter_map(|region| phase_region(&buffer, region))
        .collect();
    if region_phases.is_empty() {
        return Err(RSpinError::InvalidSpectrum {
            message: "regions auto-phase could not phase any region".to_owned(),
        });
    }

    // ── Stage 5: weighted linear regression with outlier rejection ──
    let global = global_phase_from_regions(&region_phases, options, spectrum.len())?;

    let mut spectrum = phase_correct(
        spectrum,
        global.zero_order_deg,
        global.first_order_deg,
        options.pivot_fraction,
    )?;
    spectrum.processing.pop();
    spectrum = spectrum.with_processing_record(
        ProcessingRecord::new("auto_phase_correct_regions").with_details(format!(
            "zero_order_deg={},first_order_deg={},regions={},r_squared={}",
            global.zero_order_deg,
            global.first_order_deg,
            global.region_count,
            global.regression_r_squared
        )),
    );

    Ok(RegionsResult {
        spectrum,
        zero_order_deg: global.zero_order_deg,
        first_order_deg: global.first_order_deg,
        pivot_fraction: options.pivot_fraction,
        region_count: global.region_count,
        regression_r_squared: global.regression_r_squared,
    })
}

/// Five-tap Holoborodko smooth-noise derivative (paper eq. 5).
///
/// `d_k = (42·(S_1 − S_−1) + 48·(S_2 − S_−2) + 27·(S_3 − S_−3)
///         + 8·(S_4 − S_−4) + (S_5 − S_−5)) / 512`
fn holoborodko_derivative(values: &[f64]) -> Vec<f64> {
    let mut result = vec![0.0_f64; values.len()];
    if values.len() < 11 {
        return result;
    }
    for k in 5..values.len() - 5 {
        let s1 = values[k + 1] - values[k - 1];
        let s2 = values[k + 2] - values[k - 2];
        let s3 = values[k + 3] - values[k - 3];
        let s4 = values[k + 4] - values[k - 4];
        let s5 = values[k + 5] - values[k - 5];
        result[k] = (42.0 * s1 + 48.0 * s2 + 27.0 * s3 + 8.0 * s4 + s5) / 512.0;
    }
    result
}

/// Iterative three-sigma threshold (paper, eq. 5 commentary).
fn iterative_three_sigma(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut keep: Vec<bool> = vec![true; values.len()];
    let mut prev_threshold = f64::INFINITY;
    for _ in 0..20 {
        let mut sum = 0.0_f64;
        let mut count = 0_u64;
        for (index, value) in values.iter().enumerate() {
            if keep[index] {
                sum += value;
                count += 1;
            }
        }
        if count == 0 {
            break;
        }
        let count_f = u32_from_u64(count);
        let mean = sum / f64::from(count_f);
        let mut variance = 0.0_f64;
        for (index, value) in values.iter().enumerate() {
            if keep[index] {
                let delta = value - mean;
                variance += delta * delta;
            }
        }
        let std_dev = (variance / f64::from(count_f)).sqrt();
        let threshold = mean.abs() + 3.0 * std_dev;
        let mut changed = false;
        for (index, value) in values.iter().enumerate() {
            if keep[index] && value.abs() > threshold {
                keep[index] = false;
                changed = true;
            }
        }
        if !changed || (prev_threshold - threshold).abs() < f64::EPSILON {
            return threshold;
        }
        prev_threshold = threshold;
    }
    prev_threshold
}

/// Estimates the noise autocorrelation distance (paper eqs. 6–7).
///
/// Returns the smallest shift `Δx` for which `g(Δx) < 0.6 · g(0)`. The
/// paper uses this to set a minimum region width that prevents spurious
/// peaks from zero-fill or apodization correlations being mistaken for
/// real peaks.
fn noise_autocorrelation_distance(values: &[f64], peak_map: &[bool]) -> usize {
    let mut noise_indices: Vec<usize> = Vec::new();
    for (index, flag) in peak_map.iter().enumerate() {
        if !*flag {
            noise_indices.push(index);
        }
    }
    if noise_indices.len() < 8 {
        return 0;
    }
    let mean: f64 =
        noise_indices.iter().map(|i| values[*i]).sum::<f64>() / safe_count_f64(noise_indices.len());
    let g_zero: f64 = noise_indices
        .iter()
        .map(|i| {
            let centered = values[*i] - mean;
            centered * centered
        })
        .sum::<f64>()
        / safe_count_f64(noise_indices.len());
    if g_zero <= 0.0 {
        return 0;
    }
    let max_shift = noise_indices.len().min(64);
    for shift in 1..max_shift {
        let mut acc = 0.0_f64;
        let mut count = 0_u64;
        for &i in &noise_indices {
            let j = i + shift;
            if j >= values.len() || peak_map[j] {
                continue;
            }
            let a = values[i] - mean;
            let b = values[j] - mean;
            acc += a * b;
            count += 1;
        }
        if count < 4 {
            return shift;
        }
        let g_shift = acc / f64::from(u32_from_u64(count));
        if g_shift < 0.6 * g_zero {
            return shift * 2;
        }
    }
    max_shift
}

/// Removes peak blocks shorter than `min_width` points.
fn drop_narrow_blocks(map: &mut [bool], min_width: usize) {
    if min_width <= 1 {
        return;
    }
    let mut index = 0;
    while index < map.len() {
        if !map[index] {
            index += 1;
            continue;
        }
        let start = index;
        while index < map.len() && map[index] {
            index += 1;
        }
        let end = index;
        if end - start < min_width {
            for cell in map.iter_mut().take(end).skip(start) {
                *cell = false;
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct PeakRegion {
    start: usize,
    end: usize,
}

fn build_regions(
    map: &[bool],
    min_baseline_gap: usize,
    edge_widening: usize,
    length: usize,
) -> Vec<PeakRegion> {
    if map.is_empty() {
        return Vec::new();
    }
    // Collect raw peak blocks.
    let mut blocks: Vec<PeakRegion> = Vec::new();
    let mut index = 0;
    while index < map.len() {
        if !map[index] {
            index += 1;
            continue;
        }
        let start = index;
        while index < map.len() && map[index] {
            index += 1;
        }
        let end = index;
        blocks.push(PeakRegion { start, end });
    }

    // Merge blocks separated by less than the minimum baseline gap.
    let mut merged: Vec<PeakRegion> = Vec::new();
    for block in blocks {
        if let Some(last) = merged.last_mut() {
            if block.start.saturating_sub(last.end) < min_baseline_gap {
                last.end = block.end;
                continue;
            }
        }
        merged.push(block);
    }

    // Widen edges, clamp to [0, length).
    merged
        .into_iter()
        .map(|region| PeakRegion {
            start: region.start.saturating_sub(edge_widening),
            end: (region.end + edge_widening).min(length),
        })
        .filter(|region| region.end.saturating_sub(region.start) >= 4)
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct RegionPhase {
    position: f64,
    phase_deg: f64,
    weight: f64,
}

fn phase_region(buffer: &[Complex<f64>], region: &PeakRegion) -> Option<RegionPhase> {
    let span = region.end.saturating_sub(region.start);
    if span < 4 {
        return None;
    }
    let segment = &buffer[region.start..region.end];

    // Find zero-order phase by 3-cycle coarse-to-fine search of the area
    // below the linear baseline anchored at the region endpoints. Initial
    // sweep covers ±180° in 18° steps (paper uses 20 steps over ±180°).
    let mut center_deg = 0.0_f64;
    let mut half_width_deg = 180.0_f64;
    let mut best_phase = 0.0_f64;
    for cycle in 0..3 {
        let step_count: usize = if cycle == 0 { 20 } else { 20 };
        let lo = center_deg - half_width_deg;
        let hi = center_deg + half_width_deg;
        let step = (hi - lo) / safe_count_f64(step_count);
        let mut best_score = f64::INFINITY;
        let mut best_index = 0_usize;
        for index in 0..=step_count {
            let phase_deg = lo + step * safe_count_f64(index);
            let score = area_below_baseline(segment, phase_deg);
            if score < best_score {
                best_score = score;
                best_index = index;
                best_phase = phase_deg;
            }
        }
        // Refine around best_index in the next cycle.
        let index_f = safe_count_f64(best_index);
        let count_f = safe_count_f64(step_count);
        center_deg = lo + step * index_f;
        half_width_deg = (hi - lo) / count_f;
    }

    // Region weight = peak magnitude inside the region.
    let weight: f64 = segment
        .iter()
        .map(|value| value.norm())
        .fold(0.0_f64, |acc, m| acc.max(m));
    if weight <= 0.0 {
        return None;
    }

    let position = (safe_count_f64(region.start) + safe_count_f64(region.end)) / 2.0;
    Some(RegionPhase {
        position,
        phase_deg: best_phase,
        weight,
    })
}

/// Area below a linear baseline through the segment endpoints, after the
/// segment has been rotated by `phase_deg`. Following Fig. 1 of the paper:
/// we sum positive deviations below the baseline (i.e. samples whose real
/// part falls under the line) and squared deviations to penalise large
/// dips.
fn area_below_baseline(segment: &[Complex<f64>], phase_deg: f64) -> f64 {
    let phase_rad = phase_deg.to_radians();
    let rotation = Complex::new(phase_rad.cos(), phase_rad.sin());
    let rotated: Vec<f64> = segment.iter().map(|v| (*v * rotation).re).collect();
    if rotated.len() < 2 {
        return f64::INFINITY;
    }
    let n = rotated.len();
    let first = rotated[0];
    let last = rotated[n - 1];
    let denom = safe_count_f64(n - 1);
    let mut total = 0.0_f64;
    for (index, value) in rotated.iter().enumerate() {
        let t = safe_count_f64(index) / denom;
        let baseline = first * (1.0 - t) + last * t;
        let diff = baseline - value;
        if diff > 0.0 {
            total += diff * diff;
        }
    }
    total
}

#[derive(Clone, Copy, Debug)]
struct GlobalPhase {
    zero_order_deg: f64,
    first_order_deg: f64,
    region_count: usize,
    regression_r_squared: f64,
}

fn global_phase_from_regions(
    regions: &[RegionPhase],
    options: RegionsOptions,
    length: usize,
) -> Result<GlobalPhase> {
    if regions.is_empty() {
        return Err(RSpinError::InvalidSpectrum {
            message: "no phased regions for global fit".to_owned(),
        });
    }

    if regions.len() == 1 {
        return Ok(GlobalPhase {
            zero_order_deg: regions[0].phase_deg,
            first_order_deg: 0.0,
            region_count: 1,
            regression_r_squared: 0.0,
        });
    }

    let length_f = safe_count_f64(length.max(1));
    let mut points: Vec<(f64, f64, f64)> = regions
        .iter()
        .map(|region| {
            let fraction = region.position / length_f;
            // Unwrap phases to keep adjacent regions within ±180°.
            (fraction, region.phase_deg, region.weight)
        })
        .collect();
    points.sort_by(|a, b| a.0.total_cmp(&b.0));
    unwrap_phases_in_place(&mut points);

    let mut active: Vec<bool> = vec![true; points.len()];
    let outlier_threshold = options.outlier_threshold_deg;

    loop {
        let (a, b, r_sq) = weighted_linear_fit(&points, &active);
        let mut worst_index: Option<usize> = None;
        let mut worst_diff = outlier_threshold;
        for (index, point) in points.iter().enumerate() {
            if !active[index] {
                continue;
            }
            let predicted = a + b * point.0;
            let diff = (point.1 - predicted).abs();
            if diff > worst_diff {
                worst_diff = diff;
                worst_index = Some(index);
            }
        }
        if let Some(index) = worst_index {
            active[index] = false;
            if active.iter().filter(|flag| **flag).count() < 2 {
                let kept: Vec<(f64, f64, f64)> = points
                    .iter()
                    .zip(&active)
                    .filter_map(|(point, flag)| if *flag { Some(*point) } else { None })
                    .collect();
                let weighted_mean = if kept.is_empty() {
                    0.0
                } else {
                    let total: f64 = kept.iter().map(|p| p.2).sum();
                    kept.iter().map(|p| p.1 * p.2).sum::<f64>() / total.max(f64::EPSILON)
                };
                return Ok(GlobalPhase {
                    zero_order_deg: weighted_mean,
                    first_order_deg: 0.0,
                    region_count: kept.len(),
                    regression_r_squared: 0.0,
                });
            }
            continue;
        }
        if r_sq < options.r_squared_drop_threshold {
            let kept: Vec<(f64, f64, f64)> = points
                .iter()
                .zip(&active)
                .filter_map(|(point, flag)| if *flag { Some(*point) } else { None })
                .collect();
            let total: f64 = kept.iter().map(|p| p.2).sum();
            let weighted_mean = if total <= 0.0 {
                match kept.first() {
                    Some(p) => p.1,
                    None => 0.0,
                }
            } else {
                kept.iter().map(|p| p.1 * p.2).sum::<f64>() / total
            };
            return Ok(GlobalPhase {
                zero_order_deg: weighted_mean,
                first_order_deg: 0.0,
                region_count: kept.len(),
                regression_r_squared: r_sq,
            });
        }
        // The fit converts the slope (deg per fraction-of-axis) into the
        // ph1 convention used by `phase_correct`. The intercept becomes
        // the global ph0 once mapped to the configured pivot.
        let pivot = options.pivot_fraction;
        let ph0 = a + b * pivot;
        let ph1 = b;
        let kept_count = active.iter().filter(|flag| **flag).count();
        return Ok(GlobalPhase {
            zero_order_deg: ph0,
            first_order_deg: ph1,
            region_count: kept_count,
            regression_r_squared: r_sq,
        });
    }
}

fn unwrap_phases_in_place(points: &mut [(f64, f64, f64)]) {
    for index in 1..points.len() {
        let prev = points[index - 1].1;
        while points[index].1 - prev > 180.0 {
            points[index].1 -= 360.0;
        }
        while points[index].1 - prev < -180.0 {
            points[index].1 += 360.0;
        }
    }
}

fn weighted_linear_fit(points: &[(f64, f64, f64)], active: &[bool]) -> (f64, f64, f64) {
    let mut sum_w = 0.0_f64;
    let mut sum_wx = 0.0_f64;
    let mut sum_wy = 0.0_f64;
    let mut sum_wxx = 0.0_f64;
    let mut sum_wxy = 0.0_f64;
    let mut sum_wyy = 0.0_f64;
    for (index, point) in points.iter().enumerate() {
        if !active[index] {
            continue;
        }
        let (x, y, w) = (point.0, point.1, point.2);
        sum_w += w;
        sum_wx += w * x;
        sum_wy += w * y;
        sum_wxx += w * x * x;
        sum_wxy += w * x * y;
        sum_wyy += w * y * y;
    }
    if sum_w <= 0.0 {
        return (0.0, 0.0, 0.0);
    }
    let denom = sum_w * sum_wxx - sum_wx * sum_wx;
    if denom.abs() <= f64::EPSILON {
        let mean = sum_wy / sum_w;
        return (mean, 0.0, 0.0);
    }
    let slope = (sum_w * sum_wxy - sum_wx * sum_wy) / denom;
    let intercept = (sum_wy - slope * sum_wx) / sum_w;
    // R²: total variance vs residual variance.
    let mean_y = sum_wy / sum_w;
    let mut ss_tot = sum_wyy - sum_w * mean_y * mean_y;
    let mut ss_res = 0.0_f64;
    for (index, point) in points.iter().enumerate() {
        if !active[index] {
            continue;
        }
        let predicted = intercept + slope * point.0;
        let residual = point.1 - predicted;
        ss_res += point.2 * residual * residual;
    }
    if ss_tot <= 0.0 {
        ss_tot = f64::EPSILON;
    }
    let r_sq = (1.0 - ss_res / ss_tot).clamp(0.0, 1.0);
    (intercept, slope, r_sq)
}

fn safe_count_f64(value: usize) -> f64 {
    let bounded = match u32::try_from(value) {
        Ok(v) => v,
        Err(_) => u32::MAX,
    };
    f64::from(bounded)
}

fn u32_from_u64(value: u64) -> u32 {
    if value > u64::from(u32::MAX) {
        u32::MAX
    } else {
        match u32::try_from(value) {
            Ok(v) => v,
            Err(_) => u32::MAX,
        }
    }
}

// Silence unused-import warnings while keeping the type in scope for
// future code paths.
#[allow(dead_code)]
fn _phase_candidate_marker(_: PhaseCandidate) {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phase_correct;
    use rspin_core::{Axis, Metadata, Unit};

    fn lorentzian_spectrum(points: usize, centers: &[(f64, f64)]) -> anyhow::Result<Spectrum1D> {
        let mut real = Vec::with_capacity(points);
        let mut imag = Vec::with_capacity(points);
        let denom = u32::try_from(points - 1)?;
        for index in 0..u32::try_from(points)? {
            let position = -5.0 + 10.0 * f64::from(index) / f64::from(denom);
            let mut re = 0.0;
            let mut im = 0.0;
            for (center, half_width) in centers {
                let x = (position - center) / half_width;
                let n = 1.0 + x * x;
                re += 1.0 / n;
                im += x / n;
            }
            real.push(re);
            imag.push(im);
        }
        let axis = Axis::linear("shift", Unit::Ppm, -5.0, 5.0, points)?;
        Ok(Spectrum1D::new_complex(
            axis,
            real,
            Some(imag),
            Metadata::default(),
        )?)
    }

    #[test]
    fn regions_recovers_zero_order_phase() -> anyhow::Result<()> {
        let spectrum = lorentzian_spectrum(1024, &[(-3.0, 0.04), (0.0, 0.04), (3.0, 0.04)])?;
        let phased = phase_correct(&spectrum, 30.0, 0.0, 0.5)?;
        let result = auto_phase_correct_regions(&phased, RegionsOptions::default())?;
        assert!(
            (result.zero_order_deg + 30.0).abs() < 10.0,
            "expected ph0 near -30, got {}",
            result.zero_order_deg
        );
        Ok(())
    }

    #[test]
    fn regions_recovers_first_order_phase() -> anyhow::Result<()> {
        let spectrum = lorentzian_spectrum(2048, &[(-4.0, 0.04), (-1.5, 0.04), (2.0, 0.04)])?;
        let phased = phase_correct(&spectrum, 20.0, 60.0, 0.5)?;
        let result = auto_phase_correct_regions(&phased, RegionsOptions::default())?;
        assert!(
            (result.zero_order_deg + 20.0).abs() < 10.0,
            "expected ph0 near -20, got {}",
            result.zero_order_deg
        );
        assert!(
            (result.first_order_deg + 60.0).abs() < 15.0,
            "expected ph1 near -60, got {}",
            result.first_order_deg
        );
        Ok(())
    }

    #[test]
    fn regions_detects_multiple_regions() -> anyhow::Result<()> {
        let spectrum = lorentzian_spectrum(1024, &[(-3.0, 0.04), (0.0, 0.04), (3.0, 0.04)])?;
        let result = auto_phase_correct_regions(&spectrum, RegionsOptions::default())?;
        assert!(result.region_count >= 2);
        Ok(())
    }

    #[test]
    fn regions_falls_back_when_r_squared_low() -> anyhow::Result<()> {
        // A spectrum with a single isolated peak should fall back to a
        // weighted-mean estimate and report ph1 = 0.
        let spectrum = lorentzian_spectrum(512, &[(0.0, 0.05)])?;
        let phased = phase_correct(&spectrum, 45.0, 0.0, 0.5)?;
        let result = auto_phase_correct_regions(&phased, RegionsOptions::default())?;
        assert!(result.first_order_deg.abs() < 1.0e-9);
        assert!((result.zero_order_deg + 45.0).abs() < 10.0);
        Ok(())
    }
}
