//! One-dimensional baseline correction.

use nalgebra::{DMatrix, DVector};
use rspin_core::{ProcessingRecord, RSpinError, Result, Spectrum1D};
use serde::{Deserialize, Serialize};

use crate::ProcessingStep;

/// Baseline-correction algorithm.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "method", rename_all = "snake_case")]
pub enum BaselineMethod {
    /// A fixed baseline value subtracted from all real intensities.
    Constant {
        /// Constant baseline value.
        value: f64,
    },
    /// Rolling-window minimum baseline.
    MovingMinimum {
        /// Number of points on each side of the center point.
        half_window: usize,
    },
    /// Asymmetric least-squares Whittaker smoothing.
    WhittakerAsls {
        /// Smoothness penalty. Larger values produce smoother baselines.
        lambda: f64,
        /// Asymmetry parameter in `(0, 1)`.
        p: f64,
        /// Maximum number of reweighting iterations.
        max_iter: usize,
        /// Relative weight-change tolerance.
        tolerance: f64,
    },
}

impl BaselineMethod {
    fn validate(self, len: usize) -> Result<()> {
        match self {
            Self::Constant { value } => ensure_finite("constant baseline", value),
            Self::MovingMinimum { .. } => {
                if len == 0 {
                    return Err(RSpinError::InvalidSpectrum {
                        message: "baseline correction requires at least one point".to_owned(),
                    });
                }
                Ok(())
            }
            Self::WhittakerAsls {
                lambda,
                p,
                max_iter,
                tolerance,
            } => {
                if len < 3 {
                    return Err(RSpinError::InvalidSpectrum {
                        message: "Whittaker baseline correction requires at least three points"
                            .to_owned(),
                    });
                }
                ensure_positive("lambda", lambda)?;
                if !p.is_finite() || p <= 0.0 || p >= 1.0 {
                    return Err(RSpinError::InvalidSpectrum {
                        message: "asymmetry parameter p must be finite and between 0 and 1"
                            .to_owned(),
                    });
                }
                if max_iter == 0 {
                    return Err(RSpinError::InvalidSpectrum {
                        message: "maximum iterations must be positive".to_owned(),
                    });
                }
                ensure_positive("tolerance", tolerance)
            }
        }
    }

    fn operation_name(self) -> &'static str {
        match self {
            Self::Constant { .. } => "baseline_constant",
            Self::MovingMinimum { .. } => "baseline_moving_minimum",
            Self::WhittakerAsls { .. } => "baseline_whittaker_asls",
        }
    }

    fn details(self) -> String {
        match self {
            Self::Constant { value } => format!("method=constant,value={value}"),
            Self::MovingMinimum { half_window } => {
                format!("method=moving_minimum,half_window={half_window}")
            }
            Self::WhittakerAsls {
                lambda,
                p,
                max_iter,
                tolerance,
            } => format!(
                "method=whittaker_asls,lambda={lambda},p={p},max_iter={max_iter},tolerance={tolerance}"
            ),
        }
    }
}

impl Default for BaselineMethod {
    fn default() -> Self {
        Self::WhittakerAsls {
            lambda: 1.0e6,
            p: 0.01,
            max_iter: 50,
            tolerance: 1.0e-3,
        }
    }
}

/// Baseline fit convergence metadata.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BaselineReport {
    /// Number of iterations performed by the fit.
    pub iterations: usize,
    /// Whether the fit converged within the configured tolerance.
    pub converged: bool,
    /// Final relative weight-change tolerance.
    pub tolerance: f64,
}

impl BaselineReport {
    fn converged_without_iteration() -> Self {
        Self {
            iterations: 0,
            converged: true,
            tolerance: 0.0,
        }
    }
}

/// Baseline fit output.
#[derive(Clone, Debug, PartialEq)]
pub struct BaselineFit {
    /// Estimated baseline.
    pub baseline: Vec<f64>,
    /// Corrected real intensities, computed as observed minus baseline.
    pub corrected: Vec<f64>,
    /// Fit metadata.
    pub report: BaselineReport,
}

/// Processing step that subtracts an estimated baseline.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SubtractBaseline {
    /// Baseline-correction algorithm.
    pub method: BaselineMethod,
}

impl ProcessingStep<Spectrum1D> for SubtractBaseline {
    fn apply(&self, spectrum: &Spectrum1D) -> Result<Spectrum1D> {
        subtract_baseline(spectrum, self.method)
    }
}

/// Fits a one-dimensional baseline and returns corrected intensities.
///
/// # Errors
///
/// Returns an error when the method parameters are invalid or the linear
/// system for Whittaker smoothing cannot be solved.
pub fn fit_baseline(spectrum: &Spectrum1D, method: BaselineMethod) -> Result<BaselineFit> {
    method.validate(spectrum.len())?;
    let (baseline, report) = match method {
        BaselineMethod::Constant { value } => (
            vec![value; spectrum.len()],
            BaselineReport::converged_without_iteration(),
        ),
        BaselineMethod::MovingMinimum { half_window } => (
            moving_minimum_baseline(&spectrum.intensities, half_window),
            BaselineReport::converged_without_iteration(),
        ),
        BaselineMethod::WhittakerAsls {
            lambda,
            p,
            max_iter,
            tolerance,
        } => whittaker_asls_baseline(&spectrum.intensities, lambda, p, max_iter, tolerance)?,
    };
    let corrected = spectrum
        .intensities
        .iter()
        .zip(&baseline)
        .map(|(observed, baseline_value)| observed - baseline_value)
        .collect();

    Ok(BaselineFit {
        baseline,
        corrected,
        report,
    })
}

/// Returns a spectrum with its real intensities baseline-corrected.
///
/// # Errors
///
/// Returns an error when [`fit_baseline`] fails.
pub fn subtract_baseline(spectrum: &Spectrum1D, method: BaselineMethod) -> Result<Spectrum1D> {
    let fit = fit_baseline(spectrum, method)?;
    let mut processed = spectrum.clone();
    processed.intensities = fit.corrected;
    Ok(processed.with_processing_record(
        ProcessingRecord::new(method.operation_name()).with_details(method.details()),
    ))
}

fn moving_minimum_baseline(intensities: &[f64], half_window: usize) -> Vec<f64> {
    let len = intensities.len();
    (0..len)
        .map(|index| {
            let start = index.saturating_sub(half_window);
            let end = (index + half_window + 1).min(len);
            intensities[start..end]
                .iter()
                .copied()
                .fold(f64::INFINITY, f64::min)
        })
        .collect()
}

fn whittaker_asls_baseline(
    intensities: &[f64],
    lambda: f64,
    p: f64,
    max_iter: usize,
    tolerance: f64,
) -> Result<(Vec<f64>, BaselineReport)> {
    let mut weights = vec![1.0; intensities.len()];
    let mut baseline = solve_whittaker(intensities, &weights, lambda)?;
    let mut final_tolerance = f64::INFINITY;

    for iteration in 1..=max_iter {
        let previous = weights.clone();
        for ((weight, observed), fitted) in weights.iter_mut().zip(intensities).zip(&baseline) {
            *weight = if observed > fitted { p } else { 1.0 - p };
        }
        final_tolerance = relative_change(&previous, &weights);
        if final_tolerance <= tolerance {
            return Ok((
                baseline,
                BaselineReport {
                    iterations: iteration,
                    converged: true,
                    tolerance: final_tolerance,
                },
            ));
        }
        baseline = solve_whittaker(intensities, &weights, lambda)?;
    }

    Ok((
        baseline,
        BaselineReport {
            iterations: max_iter,
            converged: false,
            tolerance: final_tolerance,
        },
    ))
}

fn solve_whittaker(intensities: &[f64], weights: &[f64], lambda: f64) -> Result<Vec<f64>> {
    let len = intensities.len();
    let mut matrix = DMatrix::<f64>::zeros(len, len);
    for (index, weight) in weights.iter().copied().enumerate() {
        matrix[(index, index)] = weight;
    }
    add_second_difference_penalty(&mut matrix, lambda);

    let rhs = DVector::from_iterator(
        len,
        intensities
            .iter()
            .copied()
            .zip(weights.iter().copied())
            .map(|(observed, weight)| observed * weight),
    );
    let solution = matrix
        .lu()
        .solve(&rhs)
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "Whittaker baseline linear solve failed".to_owned(),
        })?;
    Ok(solution.iter().copied().collect())
}

fn add_second_difference_penalty(matrix: &mut DMatrix<f64>, lambda: f64) {
    let len = matrix.nrows();
    for row in 0..(len - 2) {
        let positions = [row, row + 1, row + 2];
        let coefficients = [1.0, -2.0, 1.0];
        for (left_index, left_coefficient) in positions.iter().zip(coefficients) {
            for (right_index, right_coefficient) in positions.iter().zip(coefficients) {
                matrix[(*left_index, *right_index)] +=
                    lambda * left_coefficient * right_coefficient;
            }
        }
    }
}

fn relative_change(previous: &[f64], current: &[f64]) -> f64 {
    let numerator = previous
        .iter()
        .zip(current)
        .map(|(old, new)| {
            let difference = new - old;
            difference * difference
        })
        .sum::<f64>()
        .sqrt();
    let denominator = previous
        .iter()
        .map(|value| value * value)
        .sum::<f64>()
        .sqrt();
    numerator / denominator.max(f64::EPSILON)
}

fn ensure_finite(field: &'static str, value: f64) -> Result<()> {
    if !value.is_finite() {
        return Err(RSpinError::NonFinite { field });
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

#[cfg(test)]
mod tests;
