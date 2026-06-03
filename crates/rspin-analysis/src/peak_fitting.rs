//! Local one-dimensional peak line-shape fitting.

use nalgebra::{DMatrix, DVector};
use rspin_core::{RSpinError, Result, Spectrum1D};
use serde::{Deserialize, Serialize};

/// Supported local peak line-shape models.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PeakLineShapeModel {
    /// Lorentzian absorption line.
    #[default]
    Lorentzian,
    /// Gaussian absorption line.
    Gaussian,
    /// Pseudo-Voigt line with fitted Lorentzian mixing fraction.
    PseudoVoigt,
}

/// Options for fitting an isolated one-dimensional peak.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PeakFitOptions {
    /// Line-shape model.
    pub model: PeakLineShapeModel,
    /// Half-width of the local fitting window in x-axis units.
    pub window_half_width: f64,
    /// Maximum damped least-squares iterations.
    pub max_iterations: usize,
    /// Convergence threshold on the parameter-update norm.
    pub tolerance: f64,
}

impl Default for PeakFitOptions {
    fn default() -> Self {
        Self {
            model: PeakLineShapeModel::Lorentzian,
            window_half_width: 0.2,
            max_iterations: 50,
            tolerance: 1.0e-8,
        }
    }
}

impl PeakFitOptions {
    /// Creates fitting options for a line-shape model and local half-window.
    #[must_use]
    pub fn new(model: PeakLineShapeModel, window_half_width: f64) -> Self {
        Self {
            model,
            window_half_width,
            ..Self::default()
        }
    }

    /// Sets the maximum iteration count.
    #[must_use]
    pub fn with_max_iterations(mut self, max_iterations: usize) -> Self {
        self.max_iterations = max_iterations;
        self
    }

    /// Sets the convergence tolerance.
    #[must_use]
    pub fn with_tolerance(mut self, tolerance: f64) -> Self {
        self.tolerance = tolerance;
        self
    }

    fn validate(self) -> Result<()> {
        ensure_finite("peak fit window_half_width", self.window_half_width)?;
        ensure_finite("peak fit tolerance", self.tolerance)?;
        if self.window_half_width <= 0.0 {
            return Err(RSpinError::InvalidSpectrum {
                message: "peak fit window_half_width must be positive".to_owned(),
            });
        }
        if self.max_iterations == 0 {
            return Err(RSpinError::InvalidSpectrum {
                message: "peak fit max_iterations must be positive".to_owned(),
            });
        }
        if self.tolerance <= 0.0 {
            return Err(RSpinError::InvalidSpectrum {
                message: "peak fit tolerance must be positive".to_owned(),
            });
        }
        Ok(())
    }
}

/// Fitted local peak parameters.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PeakFitResult {
    /// Model used for the fit.
    pub model: PeakLineShapeModel,
    /// Fitted peak center.
    pub center: f64,
    /// Fitted signed peak amplitude.
    pub amplitude: f64,
    /// Fitted full width at half maximum.
    pub fwhm: f64,
    /// Fitted constant local baseline.
    pub baseline: f64,
    /// Fitted pseudo-Voigt Lorentzian fraction, when applicable.
    pub eta: Option<f64>,
    /// Analytical area for the fitted line shape above baseline.
    pub area: f64,
    /// Root-mean-square residual in the fitting window.
    pub rms_error: f64,
    /// Number of points in the fitting window.
    pub n_points: usize,
    /// Iterations used by the optimizer.
    pub iterations: usize,
    /// Whether the optimizer met the parameter-update tolerance.
    pub converged: bool,
}

/// Fits one isolated peak in a local window around `initial_center`.
///
/// # Errors
///
/// Returns an error when options are invalid, the fitting window is too small,
/// or the local fit cannot produce finite parameters.
#[allow(clippy::too_many_lines)]
pub fn fit_peak_1d(
    spectrum: &Spectrum1D,
    initial_center: f64,
    options: PeakFitOptions,
) -> Result<PeakFitResult> {
    options.validate()?;
    ensure_finite("initial peak center", initial_center)?;
    let indices = local_window_indices(spectrum, initial_center, options.window_half_width)?;
    if indices.len() < parameter_count(options.model) + 1 {
        return Err(RSpinError::InvalidSpectrum {
            message: "peak fit window has too few points".to_owned(),
        });
    }
    let x = indices
        .iter()
        .map(|index| spectrum.x.values[*index])
        .collect::<Vec<_>>();
    let y = indices
        .iter()
        .map(|index| spectrum.intensities[*index])
        .collect::<Vec<_>>();
    let constraints = FitConstraints::new(&x, options.window_half_width)?;
    let initial = initial_parameters(&x, &y, options.model, &constraints)?;
    let context = FitContext {
        x,
        y,
        model: options.model,
        constraints,
    };
    let (params, iterations, converged) = optimize_parameters(&context, initial, options)?;
    let residuals = residuals(&context, &params)?;
    let rms_error = rms(&residuals)?;
    let area = fitted_area(options.model, &params);
    ensure_finite("peak fit area", area)?;

    Ok(PeakFitResult {
        model: options.model,
        center: params[0],
        amplitude: params[1],
        fwhm: params[2],
        baseline: params[3],
        eta: pseudo_voigt_eta(options.model, &params),
        area,
        rms_error,
        n_points: context.x.len(),
        iterations,
        converged,
    })
}

/// Fits multiple isolated peaks with the same options.
///
/// # Errors
///
/// Returns the first error from an individual local fit.
pub fn fit_peaks_1d(
    spectrum: &Spectrum1D,
    initial_centers: &[f64],
    options: PeakFitOptions,
) -> Result<Vec<PeakFitResult>> {
    initial_centers
        .iter()
        .map(|center| fit_peak_1d(spectrum, *center, options))
        .collect()
}

#[derive(Clone, Debug)]
struct FitContext {
    x: Vec<f64>,
    y: Vec<f64>,
    model: PeakLineShapeModel,
    constraints: FitConstraints,
}

#[derive(Clone, Copy, Debug)]
struct FitConstraints {
    min_center: f64,
    max_center: f64,
    min_fwhm: f64,
    max_fwhm: f64,
}

impl FitConstraints {
    fn new(x: &[f64], window_half_width: f64) -> Result<Self> {
        let min_center = x.iter().copied().fold(f64::INFINITY, f64::min);
        let max_center = x.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let min_spacing = minimum_spacing(x)?;
        Ok(Self {
            min_center,
            max_center,
            min_fwhm: min_spacing,
            max_fwhm: window_half_width * 4.0,
        })
    }

    fn clamp(self, params: &mut [f64]) {
        params[0] = params[0].clamp(self.min_center, self.max_center);
        params[2] = params[2].clamp(self.min_fwhm, self.max_fwhm);
        if params.len() == 5 {
            params[4] = params[4].clamp(0.0, 1.0);
        }
    }
}

fn local_window_indices(
    spectrum: &Spectrum1D,
    initial_center: f64,
    window_half_width: f64,
) -> Result<Vec<usize>> {
    let lower = initial_center - window_half_width;
    let upper = initial_center + window_half_width;
    let indices = spectrum
        .x
        .values
        .iter()
        .enumerate()
        .filter_map(|(index, x)| {
            if *x >= lower && *x <= upper {
                Some(index)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    if indices.is_empty() {
        return Err(RSpinError::InvalidSpectrum {
            message: "peak fit window selected no points".to_owned(),
        });
    }
    Ok(indices)
}

fn initial_parameters(
    x: &[f64],
    y: &[f64],
    model: PeakLineShapeModel,
    constraints: &FitConstraints,
) -> Result<Vec<f64>> {
    let baseline = 0.5 * (y[0] + y[y.len() - 1]);
    let peak_index = strongest_index(y, baseline)?;
    let amplitude = y[peak_index] - baseline;
    if amplitude.abs() <= f64::EPSILON {
        return Err(RSpinError::InvalidSpectrum {
            message: "peak fit requires non-zero local amplitude".to_owned(),
        });
    }
    let estimated_fwhm = match estimate_fwhm(x, y, peak_index, baseline, amplitude) {
        Some(value) => value,
        None => 0.5 * (constraints.min_fwhm + constraints.max_fwhm),
    };
    let fwhm = estimated_fwhm.clamp(constraints.min_fwhm, constraints.max_fwhm);
    let mut params = vec![x[peak_index], amplitude, fwhm, baseline];
    if model == PeakLineShapeModel::PseudoVoigt {
        params.push(0.5);
    }
    Ok(params)
}

fn strongest_index(y: &[f64], baseline: f64) -> Result<usize> {
    let mut best: Option<(usize, f64)> = None;
    for (index, value) in y.iter().enumerate() {
        let amplitude = *value - baseline;
        match best {
            Some((_, best_amplitude)) if amplitude.abs() <= best_amplitude.abs() => {}
            _ => best = Some((index, amplitude)),
        }
    }
    best.map(|(index, _)| index)
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "peak fit window selected no intensities".to_owned(),
        })
}

fn estimate_fwhm(
    x: &[f64],
    y: &[f64],
    peak_index: usize,
    baseline: f64,
    amplitude: f64,
) -> Option<f64> {
    let half_height = baseline + 0.5 * amplitude;
    let left = crossing_left(x, y, peak_index, half_height, amplitude)?;
    let right = crossing_right(x, y, peak_index, half_height, amplitude)?;
    let fwhm = (right - left).abs();
    if fwhm.is_finite() && fwhm > 0.0 {
        Some(fwhm)
    } else {
        None
    }
}

fn crossing_left(
    x: &[f64],
    y: &[f64],
    peak_index: usize,
    half_height: f64,
    amplitude: f64,
) -> Option<f64> {
    for right in (1..=peak_index).rev() {
        let left = right - 1;
        if crosses_half(y[left], y[right], half_height, amplitude) {
            return interpolate_x(x[left], y[left], x[right], y[right], half_height);
        }
    }
    None
}

fn crossing_right(
    x: &[f64],
    y: &[f64],
    peak_index: usize,
    half_height: f64,
    amplitude: f64,
) -> Option<f64> {
    for left in peak_index..(y.len().saturating_sub(1)) {
        let right = left + 1;
        if crosses_half(y[left], y[right], half_height, amplitude) {
            return interpolate_x(x[left], y[left], x[right], y[right], half_height);
        }
    }
    None
}

fn crosses_half(left: f64, right: f64, half_height: f64, _amplitude: f64) -> bool {
    (left <= half_height && right >= half_height) || (left >= half_height && right <= half_height)
}

fn interpolate_x(
    left_x: f64,
    left_y: f64,
    right_x: f64,
    right_y: f64,
    target_y: f64,
) -> Option<f64> {
    let denominator = right_y - left_y;
    if denominator.abs() <= f64::EPSILON {
        return None;
    }
    let fraction = (target_y - left_y) / denominator;
    let x = left_x + fraction * (right_x - left_x);
    if x.is_finite() { Some(x) } else { None }
}

fn optimize_parameters(
    context: &FitContext,
    mut params: Vec<f64>,
    options: PeakFitOptions,
) -> Result<(Vec<f64>, usize, bool)> {
    let mut lambda = 1.0e-3;
    let mut current_rss = rss(&residuals(context, &params)?);
    let mut converged = false;
    let mut iterations = 0_usize;
    for iteration in 0..options.max_iterations {
        iterations = iteration + 1;
        let step = lm_step(context, &params, lambda)?;
        let update_norm = norm(&step);
        let mut candidate = params
            .iter()
            .zip(step.iter())
            .map(|(param, delta)| param + delta)
            .collect::<Vec<_>>();
        context.constraints.clamp(&mut candidate);
        if !valid_params(&candidate) {
            lambda *= 10.0;
            continue;
        }
        let candidate_rss = rss(&residuals(context, &candidate)?);
        if candidate_rss < current_rss {
            params = candidate;
            current_rss = candidate_rss;
            lambda = (lambda * 0.3).max(1.0e-12);
            if update_norm <= options.tolerance {
                converged = true;
                break;
            }
        } else {
            lambda *= 10.0;
        }
    }
    Ok((params, iterations, converged))
}

fn lm_step(context: &FitContext, params: &[f64], lambda: f64) -> Result<Vec<f64>> {
    let base = residuals(context, params)?;
    let rows = base.len();
    let cols = params.len();
    let mut jacobian = DMatrix::<f64>::zeros(rows, cols);
    for col in 0..cols {
        let mut perturbed = params.to_vec();
        let step = finite_difference_step(params[col]);
        perturbed[col] += step;
        context.constraints.clamp(&mut perturbed);
        let actual_step = perturbed[col] - params[col];
        if actual_step.abs() <= f64::EPSILON {
            continue;
        }
        let shifted = residuals(context, &perturbed)?;
        for row in 0..rows {
            jacobian[(row, col)] = (shifted[row] - base[row]) / actual_step;
        }
    }
    let residual_vector = DVector::from_vec(base);
    let jt = jacobian.transpose();
    let mut lhs = &jt * &jacobian;
    for index in 0..cols {
        lhs[(index, index)] += lambda;
    }
    let rhs = -(&jt * residual_vector);
    let solution = lhs
        .lu()
        .solve(&rhs)
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "peak fit normal equations are singular".to_owned(),
        })?;
    Ok(solution.iter().copied().collect())
}

fn residuals(context: &FitContext, params: &[f64]) -> Result<Vec<f64>> {
    if !valid_params(params) {
        return Err(RSpinError::InvalidSpectrum {
            message: "peak fit produced invalid parameters".to_owned(),
        });
    }
    context
        .x
        .iter()
        .zip(context.y.iter())
        .map(|(x, y)| Ok(model_value(context.model, params, *x) - y))
        .collect()
}

fn model_value(model: PeakLineShapeModel, params: &[f64], x: f64) -> f64 {
    let center = params[0];
    let amplitude = params[1];
    let fwhm = params[2];
    let baseline = params[3];
    let shape = match model {
        PeakLineShapeModel::Lorentzian => lorentzian_unit(x, center, fwhm),
        PeakLineShapeModel::Gaussian => gaussian_unit(x, center, fwhm),
        PeakLineShapeModel::PseudoVoigt => {
            let eta = params[4];
            eta * lorentzian_unit(x, center, fwhm) + (1.0 - eta) * gaussian_unit(x, center, fwhm)
        }
    };
    baseline + amplitude * shape
}

fn lorentzian_unit(x: f64, center: f64, fwhm: f64) -> f64 {
    let scaled = (x - center) / fwhm;
    1.0 / (1.0 + 4.0 * scaled * scaled)
}

fn gaussian_unit(x: f64, center: f64, fwhm: f64) -> f64 {
    let scaled = (x - center) / fwhm;
    (-4.0 * std::f64::consts::LN_2 * scaled * scaled).exp()
}

fn fitted_area(model: PeakLineShapeModel, params: &[f64]) -> f64 {
    let amplitude = params[1];
    let fwhm = params[2];
    let lorentzian_area = amplitude * std::f64::consts::PI * fwhm * 0.5;
    let gaussian_area =
        amplitude * fwhm * (std::f64::consts::PI / (4.0 * std::f64::consts::LN_2)).sqrt();
    match model {
        PeakLineShapeModel::Lorentzian => lorentzian_area,
        PeakLineShapeModel::Gaussian => gaussian_area,
        PeakLineShapeModel::PseudoVoigt => {
            let eta = params[4];
            eta * lorentzian_area + (1.0 - eta) * gaussian_area
        }
    }
}

fn pseudo_voigt_eta(model: PeakLineShapeModel, params: &[f64]) -> Option<f64> {
    if model == PeakLineShapeModel::PseudoVoigt {
        Some(params[4])
    } else {
        None
    }
}

fn rss(values: &[f64]) -> f64 {
    values.iter().map(|value| value * value).sum()
}

fn rms(values: &[f64]) -> Result<f64> {
    let count = u32::try_from(values.len()).map_err(|_| RSpinError::InvalidSpectrum {
        message: "peak fit residual count is too large".to_owned(),
    })?;
    let value = (rss(values) / f64::from(count)).sqrt();
    ensure_finite("peak fit rms_error", value)?;
    Ok(value)
}

fn norm(values: &[f64]) -> f64 {
    values.iter().map(|value| value * value).sum::<f64>().sqrt()
}

fn finite_difference_step(value: f64) -> f64 {
    (value.abs() * 1.0e-6).max(1.0e-7)
}

fn valid_params(params: &[f64]) -> bool {
    params.iter().all(|value| value.is_finite()) && params[2] > 0.0
}

fn parameter_count(model: PeakLineShapeModel) -> usize {
    match model {
        PeakLineShapeModel::Lorentzian | PeakLineShapeModel::Gaussian => 4,
        PeakLineShapeModel::PseudoVoigt => 5,
    }
}

fn minimum_spacing(x: &[f64]) -> Result<f64> {
    let mut best = f64::INFINITY;
    for pair in x.windows(2) {
        let spacing = (pair[1] - pair[0]).abs();
        if spacing > f64::EPSILON {
            best = best.min(spacing);
        }
    }
    if !best.is_finite() {
        return Err(RSpinError::InvalidAxis {
            message: "peak fit requires at least two distinct x values".to_owned(),
        });
    }
    Ok(best)
}

fn ensure_finite(field: &'static str, value: f64) -> Result<()> {
    if !value.is_finite() {
        return Err(RSpinError::NonFinite { field });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use rspin_core::{Axis, Metadata, Unit};

    use super::*;

    #[test]
    fn fits_lorentzian_peak() -> anyhow::Result<()> {
        let spectrum = synthetic_spectrum(PeakLineShapeModel::Lorentzian, 7.26, 10.0, 0.08, 0.3)?;
        let result = fit_peak_1d(
            &spectrum,
            7.25,
            PeakFitOptions::new(PeakLineShapeModel::Lorentzian, 0.4),
        )?;

        assert_close(result.center, 7.26, 1.0e-3);
        assert_close(result.fwhm, 0.08, 5.0e-3);
        assert_close(result.baseline, 0.3, 2.0e-2);
        assert!(result.rms_error < 1.0e-6);
        Ok(())
    }

    #[test]
    fn fits_gaussian_peak() -> anyhow::Result<()> {
        let spectrum = synthetic_spectrum(PeakLineShapeModel::Gaussian, 1.23, 5.0, 0.12, -0.1)?;
        let result = fit_peak_1d(
            &spectrum,
            1.2,
            PeakFitOptions::new(PeakLineShapeModel::Gaussian, 0.5),
        )?;

        assert_close(result.center, 1.23, 1.0e-3);
        assert_close(result.fwhm, 0.12, 5.0e-3);
        assert!(result.rms_error < 1.0e-6);
        Ok(())
    }

    #[test]
    fn fits_multiple_peaks() -> anyhow::Result<()> {
        let spectrum = synthetic_spectrum(PeakLineShapeModel::Lorentzian, 0.0, 4.0, 0.1, 0.0)?;
        let results = fit_peaks_1d(
            &spectrum,
            &[0.0],
            PeakFitOptions::new(PeakLineShapeModel::Lorentzian, 0.3),
        )?;

        assert_eq!(results.len(), 1);
        assert_close(results[0].center, 0.0, 1.0e-3);
        Ok(())
    }

    #[test]
    fn rejects_empty_fit_window() -> anyhow::Result<()> {
        let spectrum = synthetic_spectrum(PeakLineShapeModel::Lorentzian, 0.0, 4.0, 0.1, 0.0)?;
        let error = fit_peak_1d(
            &spectrum,
            10.0,
            PeakFitOptions::new(PeakLineShapeModel::Lorentzian, 0.1),
        )
        .expect_err("empty window should fail");

        assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));
        Ok(())
    }

    fn synthetic_spectrum(
        model: PeakLineShapeModel,
        center: f64,
        amplitude: f64,
        fwhm: f64,
        baseline: f64,
    ) -> anyhow::Result<Spectrum1D> {
        let axis = Axis::linear("shift", Unit::Ppm, center - 1.0, center + 1.0, 401)?;
        let params = match model {
            PeakLineShapeModel::PseudoVoigt => vec![center, amplitude, fwhm, baseline, 0.4],
            PeakLineShapeModel::Lorentzian | PeakLineShapeModel::Gaussian => {
                vec![center, amplitude, fwhm, baseline]
            }
        };
        let intensities = axis
            .values
            .iter()
            .map(|x| model_value(model, &params, *x))
            .collect::<Vec<_>>();
        Ok(Spectrum1D::new(axis, intensities, Metadata::default())?)
    }

    fn assert_close(left: f64, right: f64, tolerance: f64) {
        assert!(
            (left - right).abs() <= tolerance,
            "left={left}, right={right}, tolerance={tolerance}"
        );
    }
}
