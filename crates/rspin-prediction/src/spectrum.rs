//! Rendering prediction payloads into spectra.

use std::f64::consts::{LN_2, PI};

use serde::{Deserialize, Serialize};

use rspin_core::{Axis, Metadata, Nucleus, ProcessingRecord, RSpinError, Result, Spectrum1D, Unit};

use crate::{Experiment, PredictedSignal1D, PredictionSet};

/// Line shape used when rendering one-dimensional predictions.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PredictionLineShape {
    /// Lorentzian peak shape.
    #[default]
    Lorentzian,
    /// Gaussian peak shape.
    Gaussian,
    /// Equal-mixture pseudo-Voigt peak shape.
    PseudoVoigt,
}

/// Options for rendering predicted one-dimensional signals into a spectrum.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PredictionSpectrumOptions {
    /// Optional experiment filter.
    pub experiment: Option<Experiment>,
    /// Optional nucleus filter.
    pub nucleus: Option<Nucleus>,
    /// Left axis bound in ppm.
    pub from_ppm: f64,
    /// Right axis bound in ppm.
    pub to_ppm: f64,
    /// Number of output points.
    pub points: usize,
    /// Spectrometer frequency in MHz.
    pub spectrometer_mhz: f64,
    /// Full width at half maximum in Hz.
    pub line_width_hz: f64,
    /// Line shape used for each predicted signal.
    pub line_shape: PredictionLineShape,
    /// Multiplicative area scale applied to predicted intensities.
    pub area_scale: f64,
}

impl Default for PredictionSpectrumOptions {
    fn default() -> Self {
        Self {
            experiment: None,
            nucleus: None,
            from_ppm: -1.0,
            to_ppm: 12.0,
            points: 16_384,
            spectrometer_mhz: 400.0,
            line_width_hz: 1.0,
            line_shape: PredictionLineShape::Lorentzian,
            area_scale: 1.0,
        }
    }
}

impl PredictionSpectrumOptions {
    /// Creates default prediction spectrum rendering options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the experiment filter.
    #[must_use]
    pub fn with_experiment(mut self, experiment: Experiment) -> Self {
        self.experiment = Some(experiment);
        self
    }

    /// Clears the experiment filter.
    #[must_use]
    pub fn without_experiment(mut self) -> Self {
        self.experiment = None;
        self
    }

    /// Sets the nucleus filter.
    #[must_use]
    pub fn with_nucleus(mut self, nucleus: Nucleus) -> Self {
        self.nucleus = Some(nucleus);
        self
    }

    /// Clears the nucleus filter.
    #[must_use]
    pub fn without_nucleus(mut self) -> Self {
        self.nucleus = None;
        self
    }

    /// Sets the output ppm range.
    #[must_use]
    pub fn with_ppm_range(mut self, from_ppm: f64, to_ppm: f64) -> Self {
        self.from_ppm = from_ppm;
        self.to_ppm = to_ppm;
        self
    }

    /// Sets the number of output points.
    #[must_use]
    pub fn with_points(mut self, points: usize) -> Self {
        self.points = points;
        self
    }

    /// Sets the spectrometer frequency in MHz.
    #[must_use]
    pub fn with_spectrometer_mhz(mut self, spectrometer_mhz: f64) -> Self {
        self.spectrometer_mhz = spectrometer_mhz;
        self
    }

    /// Sets the full width at half maximum in Hz.
    #[must_use]
    pub fn with_line_width_hz(mut self, line_width_hz: f64) -> Self {
        self.line_width_hz = line_width_hz;
        self
    }

    /// Sets the line shape used for each predicted signal.
    #[must_use]
    pub fn with_line_shape(mut self, line_shape: PredictionLineShape) -> Self {
        self.line_shape = line_shape;
        self
    }

    /// Sets the multiplicative area scale.
    #[must_use]
    pub fn with_area_scale(mut self, area_scale: f64) -> Self {
        self.area_scale = area_scale;
        self
    }

    fn validate(&self) -> Result<()> {
        require_finite("from_ppm", self.from_ppm)?;
        require_finite("to_ppm", self.to_ppm)?;
        require_positive("spectrometer_mhz", self.spectrometer_mhz)?;
        require_positive("line_width_hz", self.line_width_hz)?;
        require_finite("area_scale", self.area_scale)?;
        if (self.from_ppm - self.to_ppm).abs() <= f64::EPSILON {
            return Err(RSpinError::InvalidSpectrum {
                message: "prediction ppm window must have distinct bounds".to_owned(),
            });
        }
        if self.points == 0 {
            return Err(RSpinError::InvalidSpectrum {
                message: "prediction spectrum point count must be positive".to_owned(),
            });
        }
        Ok(())
    }
}

/// Renders predicted one-dimensional signals into a dense spectrum.
///
/// Signals are filtered by [`PredictionSpectrumOptions::experiment`] and
/// [`PredictionSpectrumOptions::nucleus`] when those options are set. Empty
/// selections produce a valid zero-valued spectrum on the requested axis.
///
/// # Errors
///
/// Returns an error when the prediction payload or rendering options are invalid.
pub fn render_prediction_1d(
    prediction: &PredictionSet,
    options: &PredictionSpectrumOptions,
) -> Result<Spectrum1D> {
    prediction.validate()?;
    options.validate()?;

    let signals = selected_signals(prediction, options);
    let axis = Axis::linear(
        "chemical shift",
        Unit::Ppm,
        options.from_ppm,
        options.to_ppm,
        options.points,
    )?;
    let intensities = render_signals(&axis.values, &signals, options);
    let metadata = Metadata {
        name: spectrum_name(prediction),
        nucleus: infer_nucleus(&signals, options),
        frequency_mhz: Some(options.spectrometer_mhz),
        origin: prediction
            .provenance
            .as_ref()
            .map(|provenance| provenance.source.clone()),
        ..Metadata::default()
    };

    let spectrum = Spectrum1D::new(axis, intensities, metadata)?.with_processing_record(
        ProcessingRecord::new("render_prediction_1d")
            .with_details(format!("{} signals rendered", signals.len())),
    );
    Ok(spectrum)
}

fn selected_signals<'a>(
    prediction: &'a PredictionSet,
    options: &PredictionSpectrumOptions,
) -> Vec<&'a PredictedSignal1D> {
    prediction
        .signals_1d
        .iter()
        .filter(|signal| {
            options
                .experiment
                .as_ref()
                .is_none_or(|experiment| signal.experiment == *experiment)
                && options
                    .nucleus
                    .as_ref()
                    .is_none_or(|nucleus| signal.nucleus == *nucleus)
        })
        .collect()
}

fn render_signals(
    axis: &[f64],
    signals: &[&PredictedSignal1D],
    options: &PredictionSpectrumOptions,
) -> Vec<f64> {
    axis.iter()
        .copied()
        .map(|x_ppm| {
            signals
                .iter()
                .map(|signal| {
                    line_shape_value(
                        options.line_shape,
                        x_ppm,
                        signal.delta_ppm,
                        options.line_width_hz,
                        options.spectrometer_mhz,
                        signal.intensity * options.area_scale,
                    )
                })
                .sum()
        })
        .collect()
}

fn line_shape_value(
    line_shape: PredictionLineShape,
    x_ppm: f64,
    center_ppm: f64,
    line_width_hz: f64,
    spectrometer_mhz: f64,
    area: f64,
) -> f64 {
    let fwhm_ppm = line_width_hz / spectrometer_mhz;
    match line_shape {
        PredictionLineShape::Lorentzian => lorentzian(x_ppm, center_ppm, fwhm_ppm, area),
        PredictionLineShape::Gaussian => gaussian(x_ppm, center_ppm, fwhm_ppm, area),
        PredictionLineShape::PseudoVoigt => pseudo_voigt(x_ppm, center_ppm, fwhm_ppm, area),
    }
}

const PSEUDO_VOIGT_LORENTZIAN_FRACTION: f64 = 0.5;

fn lorentzian(x_ppm: f64, center_ppm: f64, fwhm_ppm: f64, area: f64) -> f64 {
    let half_width = fwhm_ppm / 2.0;
    area * half_width / (PI * ((x_ppm - center_ppm).powi(2) + half_width.powi(2)))
}

fn gaussian(x_ppm: f64, center_ppm: f64, fwhm_ppm: f64, area: f64) -> f64 {
    let sigma = fwhm_ppm / (2.0 * (2.0 * LN_2).sqrt());
    let normalizer = sigma * (2.0 * PI).sqrt();
    area * (-(x_ppm - center_ppm).powi(2) / (2.0 * sigma.powi(2))).exp() / normalizer
}

fn pseudo_voigt(x_ppm: f64, center_ppm: f64, fwhm_ppm: f64, area: f64) -> f64 {
    let lorentzian_value = lorentzian(x_ppm, center_ppm, fwhm_ppm, area);
    let gaussian_value = gaussian(x_ppm, center_ppm, fwhm_ppm, area);
    PSEUDO_VOIGT_LORENTZIAN_FRACTION * lorentzian_value
        + (1.0 - PSEUDO_VOIGT_LORENTZIAN_FRACTION) * gaussian_value
}

fn spectrum_name(prediction: &PredictionSet) -> Option<String> {
    prediction
        .name
        .as_ref()
        .map(|name| format!("{name} predicted spectrum"))
}

fn infer_nucleus(
    signals: &[&PredictedSignal1D],
    options: &PredictionSpectrumOptions,
) -> Option<Nucleus> {
    if let Some(nucleus) = &options.nucleus {
        return Some(nucleus.clone());
    }

    let mut nuclei = signals.iter().map(|signal| &signal.nucleus);
    let first = nuclei.next()?;
    if nuclei.all(|nucleus| nucleus == first) {
        Some(first.clone())
    } else {
        None
    }
}

fn require_finite(field: &'static str, value: f64) -> Result<()> {
    if !value.is_finite() {
        return Err(RSpinError::NonFinite { field });
    }
    Ok(())
}

fn require_positive(field: &'static str, value: f64) -> Result<()> {
    require_finite(field, value)?;
    if value <= 0.0 {
        return Err(RSpinError::InvalidSpectrum {
            message: format!("{field} must be positive"),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests;
