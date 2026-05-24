//! Automatic one-dimensional phase correction.

use rustfft::num_complex::Complex;

use rspin_core::{ProcessingRecord, RSpinError, Result, Spectrum1D};

use crate::{ProcessingStep, phase_correct};

const MAX_GRID_POINTS: usize = 10_000;

mod model;

pub use model::{AutoPhaseCorrection, AutoPhaseOptions, AutoPhaseResult};

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

    let mut best: Option<PhaseCandidate> = None;
    for zero_order_deg in zero_order_values {
        for first_order_deg in &first_order_values {
            let candidate = PhaseCandidate {
                zero_order_deg,
                first_order_deg: *first_order_deg,
                score: score_candidate(
                    &buffer,
                    &fractions,
                    zero_order_deg,
                    *first_order_deg,
                    options,
                ),
            };
            if best
                .as_ref()
                .is_none_or(|current| candidate.is_better_than(current))
            {
                best = Some(candidate);
            }
        }
    }

    let Some(best) = best else {
        return Err(RSpinError::InvalidSpectrum {
            message: "auto-phase search grid is empty".to_owned(),
        });
    };

    let mut spectrum = phase_correct(
        spectrum,
        best.zero_order_deg,
        best.first_order_deg,
        options.pivot_fraction,
    )?;
    spectrum.processing.pop();
    spectrum = spectrum.with_processing_record(
        ProcessingRecord::new("auto_phase_correct").with_details(format!(
            "zero_order_deg={},first_order_deg={},pivot_fraction={},score={}",
            best.zero_order_deg, best.first_order_deg, options.pivot_fraction, best.score
        )),
    );

    Ok(AutoPhaseResult {
        spectrum,
        zero_order_deg: best.zero_order_deg,
        first_order_deg: best.first_order_deg,
        score: best.score,
    })
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
