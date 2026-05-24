//! Automatic two-dimensional phase correction.

use rustfft::num_complex::Complex;

use rspin_core::{ProcessingRecord, RSpinError, Result, Spectrum2D};

use crate::{PhaseCorrection2D, ProcessingStep, phase_correct_2d};

const MAX_GRID_COMBINATIONS: usize = 50_000;

mod model;

pub use model::{AutoPhase2DOptions, AutoPhase2DResult, AutoPhaseCorrection2D};

impl AutoPhase2DOptions {
    fn validate(self) -> Result<()> {
        ensure_ordered_range(
            "x_zero_order",
            self.x_zero_order_min_deg,
            self.x_zero_order_max_deg,
        )?;
        ensure_ordered_range(
            "x_first_order",
            self.x_first_order_min_deg,
            self.x_first_order_max_deg,
        )?;
        ensure_ordered_range(
            "y_zero_order",
            self.y_zero_order_min_deg,
            self.y_zero_order_max_deg,
        )?;
        ensure_ordered_range(
            "y_first_order",
            self.y_first_order_min_deg,
            self.y_first_order_max_deg,
        )?;
        ensure_positive("x_zero_order_step_deg", self.x_zero_order_step_deg)?;
        ensure_positive("x_first_order_step_deg", self.x_first_order_step_deg)?;
        ensure_positive("y_zero_order_step_deg", self.y_zero_order_step_deg)?;
        ensure_positive("y_first_order_step_deg", self.y_first_order_step_deg)?;
        ensure_pivot("x_pivot_fraction", self.x_pivot_fraction)?;
        ensure_pivot("y_pivot_fraction", self.y_pivot_fraction)?;
        ensure_non_negative("imaginary_weight", self.imaginary_weight)?;
        ensure_non_negative("negative_weight", self.negative_weight)?;
        if self.imaginary_weight <= f64::EPSILON && self.negative_weight <= f64::EPSILON {
            return Err(RSpinError::InvalidSpectrum {
                message: "at least one 2D auto-phase scoring weight must be positive".to_owned(),
            });
        }
        Ok(())
    }
}

impl ProcessingStep<Spectrum2D> for AutoPhaseCorrection2D {
    fn apply(&self, spectrum: &Spectrum2D) -> Result<Spectrum2D> {
        auto_phase_correct_2d(spectrum, self.options).map(|result| result.spectrum)
    }
}

/// Searches separable x/y phase parameters and applies the best correction.
///
/// The scoring function penalizes residual imaginary signal and negative real
/// signal after applying each candidate phase. The search is deterministic and
/// favors smaller absolute phase corrections when scores are numerically tied.
///
/// # Errors
///
/// Returns an error when options are invalid, the configured search grid is too
/// large, or a spectrum dimension is too large for safe phase calculations.
pub fn auto_phase_correct_2d(
    spectrum: &Spectrum2D,
    options: AutoPhase2DOptions,
) -> Result<AutoPhase2DResult> {
    options.validate()?;
    let (width, height) = spectrum.shape();
    let buffer = complex_buffer(spectrum);
    let x_fractions = index_fractions(width, "x auto phase correction")?;
    let y_fractions = index_fractions(height, "y auto phase correction")?;
    let x_zero_order_values = grid_values(
        options.x_zero_order_min_deg,
        options.x_zero_order_max_deg,
        options.x_zero_order_step_deg,
        "x zero-order phase",
    )?;
    let x_first_order_values = grid_values(
        options.x_first_order_min_deg,
        options.x_first_order_max_deg,
        options.x_first_order_step_deg,
        "x first-order phase",
    )?;
    let y_zero_order_values = grid_values(
        options.y_zero_order_min_deg,
        options.y_zero_order_max_deg,
        options.y_zero_order_step_deg,
        "y zero-order phase",
    )?;
    let y_first_order_values = grid_values(
        options.y_first_order_min_deg,
        options.y_first_order_max_deg,
        options.y_first_order_step_deg,
        "y first-order phase",
    )?;
    ensure_grid_size(&[
        x_zero_order_values.len(),
        x_first_order_values.len(),
        y_zero_order_values.len(),
        y_first_order_values.len(),
    ])?;

    let mut best: Option<PhaseCandidate2D> = None;
    for x_zero_order_deg in x_zero_order_values {
        for x_first_order_deg in &x_first_order_values {
            for y_zero_order_deg in &y_zero_order_values {
                for y_first_order_deg in &y_first_order_values {
                    let correction = PhaseCorrection2D::new()
                        .x_phase(
                            x_zero_order_deg,
                            *x_first_order_deg,
                            options.x_pivot_fraction,
                        )
                        .y_phase(
                            *y_zero_order_deg,
                            *y_first_order_deg,
                            options.y_pivot_fraction,
                        );
                    let candidate = PhaseCandidate2D {
                        correction,
                        score: score_candidate(
                            &buffer,
                            width,
                            &x_fractions,
                            &y_fractions,
                            correction,
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
        }
    }

    let Some(best) = best else {
        return Err(RSpinError::InvalidSpectrum {
            message: "2D auto-phase search grid is empty".to_owned(),
        });
    };

    let mut spectrum = phase_correct_2d(spectrum, best.correction)?;
    spectrum.processing.pop();
    spectrum = spectrum.with_processing_record(
        ProcessingRecord::new("auto_phase_correct_2d").with_details(format!(
            "x_zero_order_deg={},x_first_order_deg={},x_pivot_fraction={},y_zero_order_deg={},y_first_order_deg={},y_pivot_fraction={},score={}",
            best.correction.x_zero_order_deg,
            best.correction.x_first_order_deg,
            best.correction.x_pivot_fraction,
            best.correction.y_zero_order_deg,
            best.correction.y_first_order_deg,
            best.correction.y_pivot_fraction,
            best.score
        )),
    );

    Ok(AutoPhase2DResult {
        spectrum,
        correction: best.correction,
        score: best.score,
    })
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct PhaseCandidate2D {
    correction: PhaseCorrection2D,
    score: f64,
}

impl PhaseCandidate2D {
    fn is_better_than(self, current: &Self) -> bool {
        let score_delta = self.score - current.score;
        if score_delta.abs() > f64::EPSILON {
            return score_delta < 0.0;
        }

        let self_norm = correction_norm(self.correction);
        let current_norm = correction_norm(current.correction);
        if (self_norm - current_norm).abs() > f64::EPSILON {
            return self_norm < current_norm;
        }

        self.correction.x_zero_order_deg.abs() < current.correction.x_zero_order_deg.abs()
    }
}

fn score_candidate(
    buffer: &[Complex<f64>],
    width: usize,
    x_fractions: &[f64],
    y_fractions: &[f64],
    correction: PhaseCorrection2D,
    options: AutoPhase2DOptions,
) -> f64 {
    buffer
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let x_index = index % width;
            let y_index = index / width;
            let phase_deg = correction.x_zero_order_deg
                + correction.x_first_order_deg
                    * (x_fractions[x_index] - correction.x_pivot_fraction)
                + correction.y_zero_order_deg
                + correction.y_first_order_deg
                    * (y_fractions[y_index] - correction.y_pivot_fraction);
            let phase_rad = phase_deg.to_radians();
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

fn correction_norm(correction: PhaseCorrection2D) -> f64 {
    correction.x_zero_order_deg.abs()
        + correction.x_first_order_deg.abs()
        + correction.y_zero_order_deg.abs()
        + correction.y_first_order_deg.abs()
}

fn complex_buffer(spectrum: &Spectrum2D) -> Vec<Complex<f64>> {
    match &spectrum.imaginary {
        Some(imaginary) => spectrum
            .z
            .iter()
            .zip(imaginary)
            .map(|(real, imag)| Complex::new(*real, *imag))
            .collect(),
        None => spectrum
            .z
            .iter()
            .map(|real| Complex::new(*real, 0.0))
            .collect(),
    }
}

fn index_fractions(len: usize, context: &'static str) -> Result<Vec<f64>> {
    if len <= 1 {
        return Ok(vec![0.0; len]);
    }
    let denominator = u32::try_from(len - 1).map_err(|_| RSpinError::InvalidSpectrum {
        message: format!("2D spectrum is too large for {context}"),
    })?;
    let denominator = f64::from(denominator);
    (0..len)
        .map(|index| {
            let index = u32::try_from(index).map_err(|_| RSpinError::InvalidSpectrum {
                message: format!("2D spectrum is too large for {context}"),
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
        values.push(value.min(max));
        value += step;
    }
    if values.is_empty() {
        return Err(RSpinError::InvalidSpectrum {
            message: format!("{field} search grid is empty"),
        });
    }
    Ok(values)
}

fn ensure_grid_size(lengths: &[usize; 4]) -> Result<()> {
    let mut total = 1usize;
    for length in lengths {
        total = total
            .checked_mul(*length)
            .ok_or_else(|| RSpinError::InvalidSpectrum {
                message: "2D auto-phase search grid size overflow".to_owned(),
            })?;
        if total > MAX_GRID_COMBINATIONS {
            return Err(RSpinError::InvalidSpectrum {
                message: format!(
                    "2D auto-phase search grid has {total} combinations; maximum is {MAX_GRID_COMBINATIONS}"
                ),
            });
        }
    }
    Ok(())
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

fn ensure_pivot(field: &'static str, value: f64) -> Result<()> {
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        return Err(RSpinError::InvalidSpectrum {
            message: format!("{field} must be finite and between 0 and 1"),
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
