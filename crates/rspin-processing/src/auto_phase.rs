//! Automatic one-dimensional phase correction.

use rustfft::num_complex::Complex;

use rspin_core::{ProcessingRecord, RSpinError, Result, Spectrum1D};

use crate::{ProcessingStep, phase_correct};

const MAX_GRID_POINTS: usize = 10_000;
const NELDER_MEAD_MAX_ITERS: usize = 200;
const NELDER_MEAD_TOLERANCE_DEG: f64 = 1.0e-4;

mod model;

pub use model::{AutoPhaseCorrection, AutoPhaseCost, AutoPhaseOptions, AutoPhaseResult};

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
pub fn auto_phase_correct(
    spectrum: &Spectrum1D,
    options: AutoPhaseOptions,
) -> Result<AutoPhaseResult> {
    options.validate()?;
    let buffer = complex_buffer(spectrum);
    let fractions = index_fractions(spectrum.len())?;
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

    let evaluate =
        |ph0: f64, ph1: f64| -> f64 { score_candidate(&buffer, &fractions, ph0, ph1, options) };

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
struct PhaseCandidate {
    zero_order_deg: f64,
    first_order_deg: f64,
    score: f64,
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
    zero_order_deg: f64,
    first_order_deg: f64,
    options: AutoPhaseOptions,
) -> f64 {
    match options.cost {
        AutoPhaseCost::LegacyImagNegArea => {
            legacy_cost(buffer, fractions, zero_order_deg, first_order_deg, options)
        }
        AutoPhaseCost::AcmeEntropy => {
            acme_cost(buffer, fractions, zero_order_deg, first_order_deg, options)
        }
    }
}

fn legacy_cost(
    buffer: &[Complex<f64>],
    fractions: &[f64],
    zero_order_deg: f64,
    first_order_deg: f64,
    options: AutoPhaseOptions,
) -> f64 {
    buffer
        .iter()
        .zip(fractions)
        .map(|(value, fraction)| {
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
            imaginary_penalty + negative_penalty
        })
        .sum()
}

fn acme_cost(
    buffer: &[Complex<f64>],
    fractions: &[f64],
    zero_order_deg: f64,
    first_order_deg: f64,
    options: AutoPhaseOptions,
) -> f64 {
    if buffer.len() < 3 {
        return legacy_cost(buffer, fractions, zero_order_deg, first_order_deg, options);
    }

    let mut real_parts = Vec::with_capacity(buffer.len());
    for (value, fraction) in buffer.iter().zip(fractions) {
        let phase_rad =
            (zero_order_deg + first_order_deg * (*fraction - options.pivot_fraction)).to_radians();
        let rotation = Complex::new(phase_rad.cos(), phase_rad.sin());
        real_parts.push((*value * rotation).re);
    }

    let mut abs_deriv = Vec::with_capacity(real_parts.len().saturating_sub(1));
    let mut deriv_sum = 0.0_f64;
    for window in real_parts.windows(2) {
        let d = (window[1] - window[0]).abs();
        deriv_sum += d;
        abs_deriv.push(d);
    }
    if deriv_sum <= f64::EPSILON {
        return legacy_cost(buffer, fractions, zero_order_deg, first_order_deg, options);
    }

    let mut entropy = 0.0_f64;
    for d in &abs_deriv {
        let p = d / deriv_sum;
        if p > 0.0 {
            entropy -= p * p.ln();
        }
    }

    let mut sum_sq = 0.0_f64;
    let mut neg_sq = 0.0_f64;
    for value in &real_parts {
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

fn complex_buffer(spectrum: &Spectrum1D) -> Vec<Complex<f64>> {
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
