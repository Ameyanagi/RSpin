//! Automatic one-dimensional phase correction.

use rustfft::num_complex::Complex;

use rspin_core::{ProcessingRecord, RSpinError, Result, Spectrum1D};

use crate::{ProcessingStep, phase_correct};

const MAX_GRID_POINTS: usize = 10_000;
const NELDER_MEAD_MAX_ITERS: usize = 200;
const NELDER_MEAD_TOLERANCE_DEG: f64 = 1.0e-4;

mod model;
mod regions;

pub use model::{
    AutoPhaseCorrection, AutoPhaseCost, AutoPhaseOptions, AutoPhaseResult, AutoPhaseStrategy,
};
pub use regions::{RegionsOptions, RegionsResult, auto_phase_correct_regions};

impl AutoPhaseOptions {
    fn validate(self) -> Result<()> {
        ensure_ordered_range(
            "zero_order",
            self.zero_order_min_deg,
            self.zero_order_max_deg,
        )?;
        ensure_ordered_range(
            "first_order",
            self.first_order_min_deg,
            self.first_order_max_deg,
        )?;
        ensure_positive("zero_order_step_deg", self.zero_order_step_deg)?;
        ensure_positive("first_order_step_deg", self.first_order_step_deg)?;
        ensure_non_negative("imaginary_weight", self.imaginary_weight)?;
        ensure_non_negative("negative_weight", self.negative_weight)?;
        if self.imaginary_weight <= f64::EPSILON && self.negative_weight <= f64::EPSILON {
            return Err(RSpinError::InvalidSpectrum {
                message: "at least one auto-phase scoring weight must be positive".to_owned(),
            });
        }
        if !self.pivot_fraction.is_finite() || !(0.0..=1.0).contains(&self.pivot_fraction) {
            return Err(RSpinError::InvalidSpectrum {
                message: "phase pivot fraction must be finite and between 0 and 1".to_owned(),
            });
        }
        Ok(())
    }
}

impl ProcessingStep<Spectrum1D> for AutoPhaseCorrection {
    fn apply(&self, spectrum: &Spectrum1D) -> Result<Spectrum1D> {
        auto_phase_correct(spectrum, self.options).map(|result| result.spectrum)
    }
}

/// Searches phase parameters and applies the best correction.
///
/// The scoring function penalizes residual imaginary signal and negative real
/// signal after applying each candidate phase. The search is deterministic and
/// favors smaller absolute phase corrections when scores are numerically tied.
///
/// # Errors
///
/// Returns an error when options are invalid or the spectrum is too large for
/// safe phase-grid calculations.
#[allow(clippy::too_many_lines)]
pub fn auto_phase_correct(
    spectrum: &Spectrum1D,
    options: AutoPhaseOptions,
) -> Result<AutoPhaseResult> {
    if options.strategy == AutoPhaseStrategy::Regions {
        let pivot_fraction = resolve_pivot_fraction(spectrum, options)?;
        let regions_options = RegionsOptions::default().with_pivot_fraction(pivot_fraction);
        let result = regions::auto_phase_correct_regions(spectrum, regions_options)?;
        return Ok(AutoPhaseResult {
            spectrum: result.spectrum,
            zero_order_deg: result.zero_order_deg,
            first_order_deg: result.first_order_deg,
            score: 1.0 - result.regression_r_squared,
        });
    }

    options.validate()?;
    let pivot_fraction = resolve_pivot_fraction(spectrum, options)?;
    let active_mask = resolve_active_mask(spectrum, options);
    let buffer = complex_buffer(spectrum);
    let fractions = index_fractions(spectrum.len())?;
    let options = AutoPhaseOptions {
        pivot_fraction,
        pivot_value: None,
        ..options
    };
    let zero_order_values = grid_values(
        options.zero_order_min_deg,
        options.zero_order_max_deg,
        options.zero_order_step_deg,
        "zero-order phase",
    )?;
    let first_order_values = grid_values(
        options.first_order_min_deg,
        options.first_order_max_deg,
        options.first_order_step_deg,
        "first-order phase",
    )?;

    let evaluate = |ph0: f64, ph1: f64| -> f64 {
        score_candidate(
            &buffer,
            &fractions,
            active_mask.as_deref(),
            ph0,
            ph1,
            options,
        )
    };

    let mut best: Option<PhaseCandidate> = None;
    for zero_order_deg in zero_order_values {
        for first_order_deg in &first_order_values {
            let candidate = PhaseCandidate {
                zero_order_deg,
                first_order_deg: *first_order_deg,
                score: evaluate(zero_order_deg, *first_order_deg),
            };
            if best
                .as_ref()
                .is_none_or(|current| candidate.is_better_than(current))
            {
                best = Some(candidate);
            }
        }
    }

    let Some(mut best) = best else {
        return Err(RSpinError::InvalidSpectrum {
            message: "auto-phase search grid is empty".to_owned(),
        });
    };

    if options.refine {
        let initial_step_zero = options.zero_order_step_deg.max(0.5);
        let initial_step_first = options.first_order_step_deg.max(0.5);
        let bounds = SearchBounds {
            zero_min: options.zero_order_min_deg,
            zero_max: options.zero_order_max_deg,
            first_min: options.first_order_min_deg,
            first_max: options.first_order_max_deg,
        };
        let refined = nelder_mead(
            best.zero_order_deg,
            best.first_order_deg,
            initial_step_zero,
            initial_step_first,
            &bounds,
            &evaluate,
        );
        if refined.score < best.score {
            best = refined;
        }
    }

    let mut spectrum = phase_correct(
        spectrum,
        best.zero_order_deg,
        best.first_order_deg,
        options.pivot_fraction,
    )?;
    spectrum.processing.pop();
    spectrum = spectrum.with_processing_record(
        ProcessingRecord::new("auto_phase_correct").with_details(format!(
            "zero_order_deg={},first_order_deg={},pivot_fraction={},cost={:?},refine={},score={}",
            best.zero_order_deg,
            best.first_order_deg,
            options.pivot_fraction,
            options.cost,
            options.refine,
            best.score
        )),
    );

    Ok(AutoPhaseResult {
        spectrum,
        zero_order_deg: best.zero_order_deg,
        first_order_deg: best.first_order_deg,
        score: best.score,
    })
}

#[derive(Clone, Copy, Debug)]
struct SearchBounds {
    zero_min: f64,
    zero_max: f64,
    first_min: f64,
    first_max: f64,
}

impl SearchBounds {
    fn clamp(self, ph0: f64, ph1: f64) -> (f64, f64) {
        (
            ph0.max(self.zero_min).min(self.zero_max),
            ph1.max(self.first_min).min(self.first_max),
        )
    }
}

fn nelder_mead<F>(
    ph0_init: f64,
    ph1_init: f64,
    step_zero: f64,
    step_first: f64,
    bounds: &SearchBounds,
    cost: &F,
) -> PhaseCandidate
where
    F: Fn(f64, f64) -> f64,
{
    let mut simplex: [(f64, f64, f64); 3] = [
        (ph0_init, ph1_init, cost(ph0_init, ph1_init)),
        (ph0_init + step_zero, ph1_init, 0.0),
        (ph0_init, ph1_init + step_first, 0.0),
    ];
    for vertex in &mut simplex[1..] {
        let (ph0, ph1) = bounds.clamp(vertex.0, vertex.1);
        vertex.0 = ph0;
        vertex.1 = ph1;
        vertex.2 = cost(ph0, ph1);
    }

    let evaluate = |ph0: f64, ph1: f64| -> (f64, f64, f64) {
        let (ph0, ph1) = bounds.clamp(ph0, ph1);
        (ph0, ph1, cost(ph0, ph1))
    };

    for _ in 0..NELDER_MEAD_MAX_ITERS {
        simplex.sort_by(|a, b| a.2.total_cmp(&b.2));
        let best = simplex[0];
        let mid = simplex[1];
        let worst = simplex[2];

        let span_zero = (best.0 - worst.0)
            .abs()
            .max((best.0 - mid.0).abs())
            .max((mid.0 - worst.0).abs());
        let span_first = (best.1 - worst.1)
            .abs()
            .max((best.1 - mid.1).abs())
            .max((mid.1 - worst.1).abs());
        if span_zero < NELDER_MEAD_TOLERANCE_DEG && span_first < NELDER_MEAD_TOLERANCE_DEG {
            break;
        }

        let centroid_ph0 = f64::midpoint(best.0, mid.0);
        let centroid_ph1 = f64::midpoint(best.1, mid.1);

        let (r_ph0, r_ph1, r_score) = evaluate(
            centroid_ph0 + (centroid_ph0 - worst.0),
            centroid_ph1 + (centroid_ph1 - worst.1),
        );

        if r_score < best.2 {
            let (e_ph0, e_ph1, e_score) = evaluate(
                centroid_ph0 + 2.0 * (centroid_ph0 - worst.0),
                centroid_ph1 + 2.0 * (centroid_ph1 - worst.1),
            );
            simplex[2] = if e_score < r_score {
                (e_ph0, e_ph1, e_score)
            } else {
                (r_ph0, r_ph1, r_score)
            };
            continue;
        }

        if r_score < mid.2 {
            simplex[2] = (r_ph0, r_ph1, r_score);
            continue;
        }

        let (c_ph0, c_ph1, c_score) = evaluate(
            centroid_ph0 + 0.5 * (worst.0 - centroid_ph0),
            centroid_ph1 + 0.5 * (worst.1 - centroid_ph1),
        );

        if c_score < worst.2 {
            simplex[2] = (c_ph0, c_ph1, c_score);
            continue;
        }

        for vertex in &mut simplex[1..] {
            let new_ph0 = best.0 + 0.5 * (vertex.0 - best.0);
            let new_ph1 = best.1 + 0.5 * (vertex.1 - best.1);
            let (ph0, ph1, score) = evaluate(new_ph0, new_ph1);
            vertex.0 = ph0;
            vertex.1 = ph1;
            vertex.2 = score;
        }
    }

    simplex.sort_by(|a, b| a.2.total_cmp(&b.2));
    let best = simplex[0];
    PhaseCandidate {
        zero_order_deg: best.0,
        first_order_deg: best.1,
        score: best.2,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PhaseCandidate {
    pub(crate) zero_order_deg: f64,
    pub(crate) first_order_deg: f64,
    pub(crate) score: f64,
}

impl PhaseCandidate {
    fn is_better_than(self, current: &Self) -> bool {
        let score_delta = self.score - current.score;
        if score_delta.abs() > f64::EPSILON {
            return score_delta < 0.0;
        }

        let self_norm = self.zero_order_deg.abs() + self.first_order_deg.abs();
        let current_norm = current.zero_order_deg.abs() + current.first_order_deg.abs();
        if (self_norm - current_norm).abs() > f64::EPSILON {
            return self_norm < current_norm;
        }

        self.zero_order_deg.abs() < current.zero_order_deg.abs()
    }
}

fn score_candidate(
    buffer: &[Complex<f64>],
    fractions: &[f64],
    mask: Option<&[bool]>,
    zero_order_deg: f64,
    first_order_deg: f64,
    options: AutoPhaseOptions,
) -> f64 {
    let base = match options.cost {
        AutoPhaseCost::LegacyImagNegArea => legacy_cost(
            buffer,
            fractions,
            mask,
            zero_order_deg,
            first_order_deg,
            options,
        ),
        AutoPhaseCost::AcmeEntropy => acme_cost(
            buffer,
            fractions,
            mask,
            zero_order_deg,
            first_order_deg,
            options,
        ),
    };
    let weight = options.regularization_weight.max(0.0);
    if weight <= 0.0 {
        return base;
    }
    let ph0_norm = zero_order_deg / 180.0;
    let ph1_norm = first_order_deg / 180.0;
    base + weight * (ph0_norm * ph0_norm + ph1_norm * ph1_norm)
}

fn is_active(mask: Option<&[bool]>, index: usize) -> bool {
    match mask {
        None => true,
        Some(m) => match m.get(index) {
            Some(active) => *active,
            None => false,
        },
    }
}

fn legacy_cost(
    buffer: &[Complex<f64>],
    fractions: &[f64],
    mask: Option<&[bool]>,
    zero_order_deg: f64,
    first_order_deg: f64,
    options: AutoPhaseOptions,
) -> f64 {
    buffer
        .iter()
        .zip(fractions)
        .enumerate()
        .filter_map(|(index, (value, fraction))| {
            if !is_active(mask, index) {
                return None;
            }
            let phase_rad = (zero_order_deg
                + first_order_deg * (*fraction - options.pivot_fraction))
                .to_radians();
            let rotation = Complex::new(phase_rad.cos(), phase_rad.sin());
            let corrected = *value * rotation;
            let imaginary_penalty = options.imaginary_weight * corrected.im.powi(2);
            let negative_penalty = if corrected.re < 0.0 {
                options.negative_weight * corrected.re.powi(2)
            } else {
                0.0
            };
            Some(imaginary_penalty + negative_penalty)
        })
        .sum()
}

fn acme_cost(
    buffer: &[Complex<f64>],
    fractions: &[f64],
    mask: Option<&[bool]>,
    zero_order_deg: f64,
    first_order_deg: f64,
    options: AutoPhaseOptions,
) -> f64 {
    if buffer.len() < 3 {
        return legacy_cost(
            buffer,
            fractions,
            mask,
            zero_order_deg,
            first_order_deg,
            options,
        );
    }

    let mut real_parts = Vec::with_capacity(buffer.len());
    for (value, fraction) in buffer.iter().zip(fractions) {
        let phase_rad =
            (zero_order_deg + first_order_deg * (*fraction - options.pivot_fraction)).to_radians();
        let rotation = Complex::new(phase_rad.cos(), phase_rad.sin());
        real_parts.push((*value * rotation).re);
    }

    let mut deriv_sum = 0.0_f64;
    let mut abs_deriv = Vec::with_capacity(real_parts.len().saturating_sub(1));
    for (offset, window) in real_parts.windows(2).enumerate() {
        let both_active = is_active(mask, offset) && is_active(mask, offset + 1);
        let d = if both_active {
            (window[1] - window[0]).abs()
        } else {
            0.0
        };
        deriv_sum += d;
        abs_deriv.push(d);
    }
    if deriv_sum <= f64::EPSILON {
        return legacy_cost(
            buffer,
            fractions,
            mask,
            zero_order_deg,
            first_order_deg,
            options,
        );
    }

    let mut entropy = 0.0_f64;
    for d in &abs_deriv {
        if *d <= 0.0 {
            continue;
        }
        let p = d / deriv_sum;
        if p > 0.0 {
            entropy -= p * p.ln();
        }
    }

    let mut sum_sq = 0.0_f64;
    let mut neg_sq = 0.0_f64;
    for (index, value) in real_parts.iter().enumerate() {
        if !is_active(mask, index) {
            continue;
        }
        sum_sq += value * value;
        if *value < 0.0 {
            neg_sq += value * value;
        }
    }
    let normalized_negativity = if sum_sq > f64::EPSILON {
        neg_sq / sum_sq
    } else {
        0.0
    };

    entropy + options.negative_weight * normalized_negativity
}

/// Estimates `(ph0, ph1)` from caller-supplied peak positions.
///
/// For each peak position the algorithm reads the nearest complex sample,
/// records the rotation needed to make it purely positive real, unwraps the
/// phases by 360 degrees, and least-squares fits
/// `phi(x) = ph0 + ph1 * (x - pivot)`.
///
/// `pivot_value` defaults to the midpoint of the spectrum x-axis when `None`.
///
/// # Errors
///
/// Returns an error when the peak list is empty, peak positions are not finite,
/// or the spectrum is too short to derive an axis.
pub fn peak_based_phase_estimate(
    spectrum: &Spectrum1D,
    peak_centers: &[f64],
    pivot_value: Option<f64>,
) -> Result<(f64, f64)> {
    if peak_centers.is_empty() {
        return Err(RSpinError::InvalidSpectrum {
            message: "peak-based phase estimate requires at least one peak".to_owned(),
        });
    }
    if spectrum.x.values.len() < 2 {
        return Err(RSpinError::InvalidSpectrum {
            message: "spectrum is too short for peak-based phase estimate".to_owned(),
        });
    }
    let buffer = complex_buffer(spectrum);
    let axis_span = (spectrum.x.values[spectrum.x.values.len() - 1] - spectrum.x.values[0]).abs();
    let search_half_ppm = (axis_span * 0.05).max(0.0);
    let mut samples: Vec<(f64, f64)> = peak_centers
        .iter()
        .map(|center| {
            if !center.is_finite() {
                return Err(RSpinError::NonFinite {
                    field: "peak_center",
                });
            }
            let seed_index = nearest_index(&spectrum.x.values, *center);
            let peak_index =
                refine_peak_index(&spectrum.x.values, &buffer, seed_index, search_half_ppm);
            let (sum_value, peak_x) =
                window_complex_sum(&spectrum.x.values, &buffer, peak_index, 0.3);
            let phi_rad = -sum_value.im.atan2(sum_value.re);
            Ok((peak_x, phi_rad.to_degrees()))
        })
        .collect::<Result<Vec<_>>>()?;
    samples.sort_by(|a, b| a.0.total_cmp(&b.0));

    let mut unwrapped = samples;
    for i in 1..unwrapped.len() {
        let prev = unwrapped[i - 1].1;
        let mut current = unwrapped[i].1;
        while current - prev > 180.0 {
            current -= 360.0;
        }
        while current - prev < -180.0 {
            current += 360.0;
        }
        unwrapped[i].1 = current;
    }

    let first = spectrum.x.values[0];
    let last = spectrum.x.values[spectrum.x.values.len() - 1];
    let span = last - first;
    if span.abs() <= f64::EPSILON {
        return Ok((unwrapped[0].1, 0.0));
    }
    let pivot = match pivot_value {
        Some(value) => value,
        None => f64::midpoint(first, last),
    };

    let count = unwrapped.len();
    let count_u32 = u32::try_from(count).map_err(|_| RSpinError::InvalidSpectrum {
        message: "too many peaks for phase estimate".to_owned(),
    })?;
    let n = f64::from(count_u32);

    if count == 1 {
        return Ok((unwrapped[0].1, 0.0));
    }

    let mut sum_dx = 0.0_f64;
    let mut sum_dx2 = 0.0_f64;
    let mut sum_phi = 0.0_f64;
    let mut sum_dx_phi = 0.0_f64;
    for (x, phi) in &unwrapped {
        let dx = (*x - pivot) / span;
        sum_dx += dx;
        sum_dx2 += dx * dx;
        sum_phi += *phi;
        sum_dx_phi += dx * *phi;
    }
    let denom = n * sum_dx2 - sum_dx * sum_dx;
    if denom.abs() <= f64::EPSILON {
        return Ok((sum_phi / n, 0.0));
    }
    let ph1 = (n * sum_dx_phi - sum_dx * sum_phi) / denom;
    let ph0 = (sum_phi - ph1 * sum_dx) / n;
    Ok((ph0, ph1))
}

/// Auto-phases using caller-supplied peak centers as a warm-start.
///
/// Runs [`peak_based_phase_estimate`] to seed an initial `(ph0, ph1)`, then
/// polishes with a Nelder-Mead simplex against the configured cost function
/// (skipping the coarse grid).
///
/// # Errors
///
/// Returns an error when options are invalid or the seed cannot be derived.
pub fn auto_phase_correct_with_peaks(
    spectrum: &Spectrum1D,
    options: AutoPhaseOptions,
    peak_centers: &[f64],
) -> Result<AutoPhaseResult> {
    options.validate()?;
    let pivot_fraction = resolve_pivot_fraction(spectrum, options)?;
    let active_mask = resolve_active_mask(spectrum, options);
    let (ph0_seed, ph1_seed) =
        peak_based_phase_estimate(spectrum, peak_centers, options.pivot_value)?;
    let buffer = complex_buffer(spectrum);
    let fractions = index_fractions(spectrum.len())?;
    let options = AutoPhaseOptions {
        pivot_fraction,
        pivot_value: None,
        ..options
    };
    let evaluate = |ph0: f64, ph1: f64| -> f64 {
        score_candidate(
            &buffer,
            &fractions,
            active_mask.as_deref(),
            ph0,
            ph1,
            options,
        )
    };

    let bounds = SearchBounds {
        zero_min: options.zero_order_min_deg,
        zero_max: options.zero_order_max_deg,
        first_min: options.first_order_min_deg,
        first_max: options.first_order_max_deg,
    };
    let (ph0_clamped, ph1_clamped) = bounds.clamp(ph0_seed, ph1_seed);

    let seed_candidate = PhaseCandidate {
        zero_order_deg: ph0_clamped,
        first_order_deg: ph1_clamped,
        score: evaluate(ph0_clamped, ph1_clamped),
    };

    let mut best = seed_candidate;
    if options.refine {
        let refined = nelder_mead(
            ph0_clamped,
            ph1_clamped,
            options.zero_order_step_deg.max(2.0),
            options.first_order_step_deg.max(2.0),
            &bounds,
            &evaluate,
        );
        if refined.score < best.score {
            best = refined;
        }
    }

    let mut spectrum = phase_correct(
        spectrum,
        best.zero_order_deg,
        best.first_order_deg,
        options.pivot_fraction,
    )?;
    spectrum.processing.pop();
    spectrum = spectrum.with_processing_record(
        ProcessingRecord::new("auto_phase_correct_with_peaks").with_details(format!(
            "zero_order_deg={},first_order_deg={},pivot_fraction={},peak_count={},seed_ph0={},seed_ph1={},refine={},score={}",
            best.zero_order_deg,
            best.first_order_deg,
            options.pivot_fraction,
            peak_centers.len(),
            ph0_seed,
            ph1_seed,
            options.refine,
            best.score
        )),
    );

    Ok(AutoPhaseResult {
        spectrum,
        zero_order_deg: best.zero_order_deg,
        first_order_deg: best.first_order_deg,
        score: best.score,
    })
}

fn nearest_index(values: &[f64], target: f64) -> usize {
    let mut best = 0_usize;
    let mut best_distance = f64::INFINITY;
    for (index, value) in values.iter().enumerate() {
        let distance = (*value - target).abs();
        if distance < best_distance {
            best_distance = distance;
            best = index;
        }
    }
    best
}

fn window_complex_sum(
    axis: &[f64],
    buffer: &[Complex<f64>],
    peak_index: usize,
    magnitude_threshold: f64,
) -> (Complex<f64>, f64) {
    let peak_magnitude = buffer[peak_index].norm();
    if peak_magnitude <= 0.0 {
        return (buffer[peak_index], axis[peak_index]);
    }
    let threshold = magnitude_threshold * peak_magnitude;
    let mut sum = Complex::new(0.0_f64, 0.0_f64);
    let mut weight = 0.0_f64;
    let mut centroid_num = 0.0_f64;
    let mut left = peak_index;
    while left > 0 && buffer[left - 1].norm() >= threshold {
        left -= 1;
    }
    let mut right = peak_index;
    while right + 1 < buffer.len() && buffer[right + 1].norm() >= threshold {
        right += 1;
    }
    for index in left..=right {
        let magnitude = buffer[index].norm();
        sum += buffer[index];
        weight += magnitude;
        centroid_num += magnitude * axis[index];
    }
    let centroid_x = if weight > 0.0 {
        centroid_num / weight
    } else {
        axis[peak_index]
    };
    (sum, centroid_x)
}

fn refine_peak_index(
    axis: &[f64],
    buffer: &[Complex<f64>],
    seed: usize,
    half_window_ppm: f64,
) -> usize {
    if axis.is_empty() || buffer.is_empty() {
        return seed;
    }
    let center = axis[seed];
    let mut best = seed;
    let mut best_magnitude = buffer[seed].norm();
    for (index, value) in axis.iter().enumerate() {
        if (*value - center).abs() > half_window_ppm {
            continue;
        }
        let magnitude = buffer[index].norm();
        if magnitude > best_magnitude {
            best_magnitude = magnitude;
            best = index;
        }
    }
    best
}

fn resolve_pivot_fraction(spectrum: &Spectrum1D, options: AutoPhaseOptions) -> Result<f64> {
    let Some(pivot_value) = options.pivot_value else {
        return Ok(options.pivot_fraction);
    };
    if !pivot_value.is_finite() {
        return Err(RSpinError::NonFinite {
            field: "pivot_value",
        });
    }
    let values = &spectrum.x.values;
    if values.len() < 2 {
        return Ok(options.pivot_fraction);
    }
    let first = values[0];
    let last = values[values.len() - 1];
    let range = last - first;
    if range.abs() <= f64::EPSILON {
        return Ok(options.pivot_fraction);
    }
    let fraction = (pivot_value - first) / range;
    Ok(fraction.clamp(0.0, 1.0))
}

fn resolve_active_mask(spectrum: &Spectrum1D, options: AutoPhaseOptions) -> Option<Vec<bool>> {
    let (raw_start, raw_end) = options.active_region?;
    if !raw_start.is_finite() || !raw_end.is_finite() {
        return None;
    }
    let lo = raw_start.min(raw_end);
    let hi = raw_start.max(raw_end);
    Some(
        spectrum
            .x
            .values
            .iter()
            .map(|value| *value >= lo && *value <= hi)
            .collect(),
    )
}

pub(crate) fn complex_buffer(spectrum: &Spectrum1D) -> Vec<Complex<f64>> {
    match &spectrum.imaginary {
        Some(imaginary) => spectrum
            .intensities
            .iter()
            .zip(imaginary)
            .map(|(real, imag)| Complex::new(*real, *imag))
            .collect(),
        None => spectrum
            .intensities
            .iter()
            .map(|real| Complex::new(*real, 0.0))
            .collect(),
    }
}

fn index_fractions(len: usize) -> Result<Vec<f64>> {
    if len <= 1 {
        return Ok(vec![0.0; len]);
    }
    let denominator = u32::try_from(len - 1).map_err(|_| RSpinError::InvalidSpectrum {
        message: "spectrum is too large for auto phase correction".to_owned(),
    })?;
    let denominator = f64::from(denominator);
    (0..len)
        .map(|index| {
            let index = u32::try_from(index).map_err(|_| RSpinError::InvalidSpectrum {
                message: "spectrum is too large for auto phase correction".to_owned(),
            })?;
            Ok(f64::from(index) / denominator)
        })
        .collect()
}

fn grid_values(min: f64, max: f64, step: f64, field: &'static str) -> Result<Vec<f64>> {
    let mut values = Vec::new();
    let mut value = min;
    let tolerance = step.abs() * 1.0e-12;
    while value <= max + tolerance {
        if values.len() >= MAX_GRID_POINTS {
            return Err(RSpinError::InvalidSpectrum {
                message: format!("{field} search grid is too large"),
            });
        }
        values.push(value.min(max));
        value += step;
    }
    Ok(values)
}

fn ensure_ordered_range(field: &'static str, min: f64, max: f64) -> Result<()> {
    ensure_finite(field, min)?;
    ensure_finite(field, max)?;
    if min > max {
        return Err(RSpinError::InvalidSpectrum {
            message: format!("{field} minimum must not exceed maximum"),
        });
    }
    Ok(())
}

fn ensure_non_negative(field: &'static str, value: f64) -> Result<()> {
    ensure_finite(field, value)?;
    if value < 0.0 {
        return Err(RSpinError::InvalidSpectrum {
            message: format!("{field} must be non-negative"),
        });
    }
    Ok(())
}

fn ensure_positive(field: &'static str, value: f64) -> Result<()> {
    ensure_finite(field, value)?;
    if value <= 0.0 {
        return Err(RSpinError::InvalidSpectrum {
            message: format!("{field} must be positive"),
        });
    }
    Ok(())
}

fn ensure_finite(field: &'static str, value: f64) -> Result<()> {
    if !value.is_finite() {
        return Err(RSpinError::NonFinite { field });
    }
    Ok(())
}

#[cfg(test)]
mod tests;
