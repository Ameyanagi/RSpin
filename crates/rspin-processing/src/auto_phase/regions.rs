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
    clippy::needless_late_init
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

use super::{unwrap_phases_deg, weighted_linear_fit};
use crate::phase_correct;
use crate::transform::complex_buffer;

/// Result returned by [`auto_phase_correct_regions`].
#[derive(Clone, Debug, PartialEq)]
pub struct RegionsResult {
    /// Phased spectrum.
    pub spectrum: Spectrum1D,
    /// Fitted global zero-order phase in degrees.
    pub zero_order_deg: f64,
    /// Fitted global first-order phase in degrees across the full spectrum.
    pub first_order_deg: f64,
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
    /// Optional `(start, end)` window in the spectrum's x-axis units.
    /// Indices outside the window are forced off in the peak map so that
    /// detected regions never span the inactive area.
    pub active_region: Option<(f64, f64)>,
    /// Allow genuinely negative absorption peaks to remain negative.
    ///
    /// When `true`, per-region phasing accepts a clean negative absorption
    /// lineshape as readily as a positive one, and the wrap-resolution loss
    /// drops its negative-content penalty. Used for DEPT/APT-style sign-edited
    /// spectra. Defaults to `false`.
    pub allow_negative: bool,
}

impl Default for RegionsOptions {
    fn default() -> Self {
        Self {
            min_baseline_gap: 8,
            region_edge_widening: 4,
            r_squared_drop_threshold: 0.2,
            outlier_threshold_deg: 0.6_f64.to_degrees(),
            pivot_fraction: 0.5,
            active_region: None,
            allow_negative: false,
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

    /// Returns options that restrict peak detection to the supplied x-axis
    /// window.
    #[must_use]
    pub fn with_active_region(mut self, start: f64, end: f64) -> Self {
        self.active_region = Some((start, end));
        self
    }

    /// Returns options that preserve genuinely negative peaks (DEPT/APT).
    #[must_use]
    pub fn with_allow_negative(mut self, allow_negative: bool) -> Self {
        self.allow_negative = allow_negative;
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

    // Honor the caller's active-region window by masking out everything
    // outside it before regions are clustered. The Regions algorithm has
    // no notion of a "weighted" mask, so we just force those indices off.
    if let Some((lo_raw, hi_raw)) = options.active_region
        && lo_raw.is_finite()
        && hi_raw.is_finite()
    {
        let lo = lo_raw.min(hi_raw);
        let hi = lo_raw.max(hi_raw);
        for (index, value) in spectrum.x.values.iter().enumerate() {
            if !(*value >= lo && *value <= hi) {
                peak_map[index] = false;
            }
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
        .filter_map(|region| phase_region(&buffer, region, options.allow_negative))
        .collect();
    if region_phases.is_empty() {
        return Err(RSpinError::InvalidSpectrum {
            message: "regions auto-phase could not phase any region".to_owned(),
        });
    }

    // ── Stage 5: weighted linear regression with outlier rejection ──
    let global = global_phase_from_regions(&region_phases, options, spectrum.len())?;

    // ── Stage 5b: canonicalize ph0 and pick the ph1 wrap whose phased real
    //              part best matches the (phase-independent) magnitude. ──
    let (zero_order_deg, first_order_deg) = resolve_wrap_ambiguity(
        &buffer,
        &magnitude,
        canonicalize_phase(global.zero_order_deg),
        global.first_order_deg,
        options.pivot_fraction,
        options.allow_negative,
    );

    let mut spectrum = phase_correct(
        spectrum,
        zero_order_deg,
        first_order_deg,
        options.pivot_fraction,
    )?;
    spectrum.processing.pop();
    spectrum = spectrum.with_processing_record(
        ProcessingRecord::new("auto_phase_correct_regions").with_details(format!(
            "zero_order_deg={},first_order_deg={},regions={},r_squared={}",
            zero_order_deg, first_order_deg, global.region_count, global.regression_r_squared,
        )),
    );

    Ok(RegionsResult {
        spectrum,
        zero_order_deg,
        first_order_deg,
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
///
/// Maintains a running sum / sum-of-squares so each rejection cycle is one
/// pass instead of three.
fn iterative_three_sigma(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut keep: Vec<bool> = vec![true; values.len()];
    let mut sum: f64 = values.iter().sum();
    let mut sum_sq: f64 = values.iter().map(|v| v * v).sum();
    let mut count = values.len() as u64;
    let mut prev_threshold = f64::INFINITY;
    for _ in 0..20 {
        if count == 0 {
            break;
        }
        let count_f = f64::from(u32_from_u64(count));
        let mean = sum / count_f;
        let variance = (sum_sq / count_f - mean * mean).max(0.0);
        let std_dev = variance.sqrt();
        let threshold = mean.abs() + 3.0 * std_dev;
        let mut changed = false;
        for (index, value) in values.iter().enumerate() {
            if keep[index] && value.abs() > threshold {
                keep[index] = false;
                sum -= *value;
                sum_sq -= value * value;
                count -= 1;
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
        if let Some(last) = merged.last_mut()
            && block.start.saturating_sub(last.end) < min_baseline_gap
        {
            last.end = block.end;
            continue;
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

fn phase_region(
    buffer: &[Complex<f64>],
    region: &PeakRegion,
    allow_negative: bool,
) -> Option<RegionPhase> {
    let span = region.end.saturating_sub(region.start);
    if span < 4 {
        return None;
    }
    let segment = &buffer[region.start..region.end];

    // Find zero-order phase by 3-cycle coarse-to-fine search of the area
    // below the linear baseline anchored at the region endpoints. Initial
    // sweep covers ±180° in 18° steps (paper uses 20 steps over ±180°).
    //
    // When `allow_negative` is set, the search uses a sign-agnostic
    // one-sidedness score over a narrower ±90° window: within that window
    // there is exactly one absorption solution (the smaller-magnitude of the
    // two 180°-apart phases), so the acquired DEPT/APT peak sign is kept.
    let mut center_deg = 0.0_f64;
    let mut half_width_deg = if allow_negative { 90.0 } else { 180.0 };
    let mut best_phase = 0.0_f64;
    for _cycle in 0..3 {
        let step_count: usize = 20;
        let lo = center_deg - half_width_deg;
        let hi = center_deg + half_width_deg;
        let step = (hi - lo) / safe_count_f64(step_count);
        let mut best_score = f64::INFINITY;
        let mut best_index = 0_usize;
        for index in 0..=step_count {
            let phase_deg = lo + step * safe_count_f64(index);
            let score = if allow_negative {
                negated_absolute_area(segment, phase_deg)
            } else {
                area_below_baseline(segment, phase_deg)
            };
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
/// dips. Minimising it drives the region to a positive absorption lineshape.
fn area_below_baseline(segment: &[Complex<f64>], phase_deg: f64) -> f64 {
    let n = segment.len();
    if n < 2 {
        return f64::INFINITY;
    }
    let phase_rad = phase_deg.to_radians();
    let rotation = Complex::new(phase_rad.cos(), phase_rad.sin());
    let first = (segment[0] * rotation).re;
    let last = (segment[n - 1] * rotation).re;
    let denom = safe_count_f64(n - 1);
    let mut total = 0.0_f64;
    for (index, value) in segment.iter().enumerate() {
        let real = (*value * rotation).re;
        let t = safe_count_f64(index) / denom;
        let baseline = first * (1.0 - t) + last * t;
        let diff = baseline - real;
        if diff > 0.0 {
            total += diff * diff;
        }
    }
    total
}

/// Sign-agnostic absorption score for [`phase_region`] when `allow_negative`
/// is set. Returns the negated absolute net area of the baseline-subtracted
/// real part: a clean absorption peak of *either* sign integrates to a large
/// one-signed area (low score), while dispersion (antisymmetric, net ≈ 0) and
/// a flattened real channel (net ≈ 0) both score near zero. Minimising it
/// therefore keeps a tall one-sided lineshape regardless of sign. Restricting
/// the caller's search window to ±90° selects the smaller-magnitude of the two
/// absorption solutions, preserving the acquired DEPT/APT sign.
fn negated_absolute_area(segment: &[Complex<f64>], phase_deg: f64) -> f64 {
    let n = segment.len();
    if n < 2 {
        return f64::INFINITY;
    }
    let phase_rad = phase_deg.to_radians();
    let rotation = Complex::new(phase_rad.cos(), phase_rad.sin());
    let first = (segment[0] * rotation).re;
    let last = (segment[n - 1] * rotation).re;
    let denom = safe_count_f64(n - 1);
    let mut net = 0.0_f64;
    for (index, value) in segment.iter().enumerate() {
        let real = (*value * rotation).re;
        let t = safe_count_f64(index) / denom;
        let baseline = first * (1.0 - t) + last * t;
        net += real - baseline;
    }
    -net.abs()
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
    unwrap_phases_deg(&mut points, |p| &mut p.1);

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
                if let Some(p) = kept.first() { p.1 } else { 0.0 }
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

/// Maps an angle in degrees to the canonical `(-180, 180]` window.
fn canonicalize_phase(angle_deg: f64) -> f64 {
    let mut value = angle_deg % 360.0;
    if value > 180.0 {
        value -= 360.0;
    } else if value <= -180.0 {
        value += 360.0;
    }
    value
}

/// Searches `ph1 + k * 360` over a small set of `k` and returns the
/// `(ph0, ph1)` whose phased real spectrum best matches the magnitude
/// envelope on high-magnitude points.
///
/// The magnitude spectrum is independent of phase, so it serves as a
/// canonical absorption-mode target. Among all wrap-equivalent linear
/// phase ramps the one that best reproduces the magnitude shape on the
/// peaks is the most physically meaningful answer.
/// Picks the `ph1 + k * 360` wrap that minimises a magnitude-target loss
/// while always keeping ph0 inside the canonical `(-180, 180]` window.
///
/// At every peak position, candidates that differ by `k · 360°` in ph1
/// produce *different* rotations away from the pivot, so they are
/// genuinely distinct solutions. The candidate whose phased real part
/// fits the (phase-independent) magnitude envelope best on its peaks is
/// the most physically meaningful answer.
fn resolve_wrap_ambiguity(
    buffer: &[Complex<f64>],
    magnitude: &[f64],
    ph0_canonical: f64,
    ph1_fit: f64,
    pivot_fraction: f64,
    allow_negative: bool,
) -> (f64, f64) {
    // Trust the fit when it already lands inside the canonical
    // [-180, 180] window — the wrap search adds candidates that aren't
    // spectrum-equivalent and can pull a good fit off the answer.
    if ph1_fit.abs() <= 180.0 {
        return (ph0_canonical, ph1_fit);
    }
    let max_mag = magnitude.iter().copied().fold(0.0_f64, f64::max);
    if max_mag <= 0.0 {
        return (ph0_canonical, ph1_fit);
    }
    let threshold = 0.30 * max_mag;
    let fractions = index_fractions(buffer.len());

    // Build wrap candidates around the original fit, plus the canonical
    // wrap and one full turn either side of it. Out-of-window fits
    // (typical JEOL after group-delay correction) are the only case
    // where this search overrides the fit value.
    let canonical_ph1 = canonicalize_phase(ph1_fit);
    let mut candidates = vec![ph1_fit, canonical_ph1];
    for k in [-1_i32, 1_i32] {
        candidates.push(canonical_ph1 + f64::from(k) * 360.0);
    }

    let mut best = (ph0_canonical, ph1_fit, f64::INFINITY);
    for candidate in candidates {
        let loss = magnitude_target_loss(
            buffer,
            &fractions,
            magnitude,
            threshold,
            ph0_canonical,
            candidate,
            pivot_fraction,
            allow_negative,
        );
        if loss < best.2 {
            best = (ph0_canonical, candidate, loss);
        }
    }
    (best.0, best.1)
}

fn index_fractions(len: usize) -> Vec<f64> {
    if len <= 1 {
        return vec![0.0; len];
    }
    let denom = safe_count_f64(len - 1);
    (0..len).map(|i| safe_count_f64(i) / denom).collect()
}

/// Magnitude-target loss used to choose between wrap-equivalent ph1
/// candidates. Maximises the (peak-weighted) correlation between the
/// rotated real part and the magnitude envelope while penalising any
/// residual negative-going content on a peak.
///
/// When `allow_negative` is set the negative-content penalty is dropped and
/// the absolute correlation is used, so a uniformly negative (but absorptive)
/// solution scores as well as a positive one.
#[allow(clippy::too_many_arguments)]
fn magnitude_target_loss(
    buffer: &[Complex<f64>],
    fractions: &[f64],
    magnitude: &[f64],
    threshold: f64,
    ph0_deg: f64,
    ph1_deg: f64,
    pivot_fraction: f64,
    allow_negative: bool,
) -> f64 {
    let mut numerator = 0.0_f64;
    let mut neg_area = 0.0_f64;
    let mut mag_norm = 0.0_f64;
    let mut real_sq = 0.0_f64;
    for ((value, fraction), m) in buffer.iter().zip(fractions).zip(magnitude) {
        if *m < threshold {
            continue;
        }
        let phase_rad = (ph0_deg + ph1_deg * (*fraction - pivot_fraction)).to_radians();
        let rotation = Complex::new(phase_rad.cos(), phase_rad.sin());
        let real = (*value * rotation).re;
        numerator += real * m;
        if real < 0.0 {
            neg_area += (-real) * m;
        }
        mag_norm += m * m;
        real_sq += real * real;
    }
    let denom = (mag_norm * real_sq).sqrt();
    if denom <= 0.0 {
        return f64::INFINITY;
    }
    let correlation = numerator / denom;
    if allow_negative {
        // Sign-agnostic: reward strong alignment with the magnitude envelope
        // regardless of sign, and do not penalise negative-going content.
        return -correlation.abs();
    }
    let normalised_negativity = if mag_norm > 0.0 {
        neg_area / mag_norm.sqrt()
    } else {
        0.0
    };
    -correlation + 4.0 * normalised_negativity
}

fn safe_count_f64(value: usize) -> f64 {
    let bounded: u32 = u32::try_from(value).ok().map_or(u32::MAX, |v| v);
    f64::from(bounded)
}

fn u32_from_u64(value: u64) -> u32 {
    if value > u64::from(u32::MAX) {
        u32::MAX
    } else {
        u32::try_from(value).ok().map_or(u32::MAX, |v| v)
    }
}

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
