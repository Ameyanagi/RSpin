//! One-dimensional baseline correction.

use rspin_core::{ProcessingRecord, RSpinError, Result, Spectrum1D};
use serde::{Deserialize, Serialize};

use crate::ProcessingStep;

mod polynomial;
mod whittaker;

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
    /// Least-squares polynomial fit over the spectrum x axis.
    Polynomial {
        /// Polynomial degree. Degree `0` fits a constant mean baseline.
        degree: usize,
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
    /// Asymmetric least-squares smoothing from the optional `baselines` crate.
    #[cfg(feature = "external-baselines")]
    BaselinesAsls {
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
            Self::Polynomial { degree } => validate_polynomial(len, degree),
            Self::WhittakerAsls {
                lambda,
                p,
                max_iter,
                tolerance,
            } => validate_asls(
                "Whittaker baseline correction",
                len,
                lambda,
                p,
                max_iter,
                tolerance,
            ),
            #[cfg(feature = "external-baselines")]
            Self::BaselinesAsls {
                lambda,
                p,
                max_iter,
                tolerance,
            } => validate_asls(
                "baselines AsLS correction",
                len,
                lambda,
                p,
                max_iter,
                tolerance,
            ),
        }
    }

    fn operation_name(self) -> &'static str {
        match self {
            Self::Constant { .. } => "baseline_constant",
            Self::MovingMinimum { .. } => "baseline_moving_minimum",
            Self::Polynomial { .. } => "baseline_polynomial",
            Self::WhittakerAsls { .. } => "baseline_whittaker_asls",
            #[cfg(feature = "external-baselines")]
            Self::BaselinesAsls { .. } => "baseline_baselines_asls",
        }
    }

    fn details(self) -> String {
        match self {
            Self::Constant { value } => format!("method=constant,value={value}"),
            Self::MovingMinimum { half_window } => {
                format!("method=moving_minimum,half_window={half_window}")
            }
            Self::Polynomial { degree } => format!("method=polynomial,degree={degree}"),
            Self::WhittakerAsls {
                lambda,
                p,
                max_iter,
                tolerance,
            } => format!(
                "method=whittaker_asls,lambda={lambda},p={p},max_iter={max_iter},tolerance={tolerance}"
            ),
            #[cfg(feature = "external-baselines")]
            Self::BaselinesAsls {
                lambda,
                p,
                max_iter,
                tolerance,
            } => format!(
                "method=baselines_asls,lambda={lambda},p={p},max_iter={max_iter},tolerance={tolerance}"
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
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BaselineFit {
    /// Estimated baseline.
    pub baseline: Vec<f64>,
    /// Corrected real intensities, computed as observed minus baseline.
    pub corrected: Vec<f64>,
    /// Fit metadata.
    pub report: BaselineReport,
}

/// Processing step that subtracts an estimated baseline.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SubtractBaseline {
    /// Baseline-correction algorithm.
    pub method: BaselineMethod,
}

impl SubtractBaseline {
    /// Creates a baseline-subtraction processing step.
    #[must_use]
    pub fn new(method: BaselineMethod) -> Self {
        Self { method }
    }

    /// Returns this step with a different baseline method.
    #[must_use]
    pub fn with_method(mut self, method: BaselineMethod) -> Self {
        self.method = method;
        self
    }
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
        BaselineMethod::Polynomial { degree } => (
            polynomial::polynomial_baseline(&spectrum.x.values, &spectrum.intensities, degree)?,
            BaselineReport::converged_without_iteration(),
        ),
        BaselineMethod::WhittakerAsls {
            lambda,
            p,
            max_iter,
            tolerance,
        } => whittaker::whittaker_asls_baseline(
            &spectrum.intensities,
            lambda,
            p,
            max_iter,
            tolerance,
        )?,
        #[cfg(feature = "external-baselines")]
        BaselineMethod::BaselinesAsls {
            lambda,
            p,
            max_iter,
            tolerance,
        } => baselines_asls_baseline(&spectrum.intensities, lambda, p, max_iter, tolerance)?,
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

#[cfg(feature = "external-baselines")]
fn baselines_asls_baseline(
    intensities: &[f64],
    lambda: f64,
    p: f64,
    max_iter: usize,
    tolerance: f64,
) -> Result<(Vec<f64>, BaselineReport)> {
    let fit = baselines::Baseline::new(intensities)
        .asls()
        .lambda(lambda)
        .p(p)
        .max_iter(max_iter)
        .tol(tolerance)
        .fit()
        .map_err(|error| RSpinError::InvalidSpectrum {
            message: format!("baselines AsLS correction failed: {error}"),
        })?;

    Ok((
        fit.baseline,
        BaselineReport {
            iterations: fit.report.iterations,
            converged: fit.report.converged,
            tolerance: fit.report.tolerance,
        },
    ))
}

fn validate_asls(
    method: &'static str,
    len: usize,
    lambda: f64,
    p: f64,
    max_iter: usize,
    tolerance: f64,
) -> Result<()> {
    if len < 3 {
        return Err(RSpinError::InvalidSpectrum {
            message: format!("{method} requires at least three points"),
        });
    }
    ensure_positive("lambda", lambda)?;
    if !p.is_finite() || p <= 0.0 || p >= 1.0 {
        return Err(RSpinError::InvalidSpectrum {
            message: "asymmetry parameter p must be finite and between 0 and 1".to_owned(),
        });
    }
    if max_iter == 0 {
        return Err(RSpinError::InvalidSpectrum {
            message: "maximum iterations must be positive".to_owned(),
        });
    }
    ensure_positive("tolerance", tolerance)
}

fn validate_polynomial(len: usize, degree: usize) -> Result<()> {
    if len == 0 {
        return Err(RSpinError::InvalidSpectrum {
            message: "polynomial baseline correction requires at least one point".to_owned(),
        });
    }
    let coefficient_count = degree
        .checked_add(1)
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "polynomial degree is too large".to_owned(),
        })?;
    if coefficient_count > len {
        return Err(RSpinError::InvalidSpectrum {
            message: format!(
                "polynomial degree {degree} requires at least {coefficient_count} points"
            ),
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
