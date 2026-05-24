//! Rendering two-dimensional prediction payloads into spectra.

use serde::{Deserialize, Serialize};

use rspin_core::{Axis, Metadata, Nucleus, ProcessingRecord, RSpinError, Result, Spectrum2D, Unit};

use crate::{
    Experiment, PredictedCorrelation2D, PredictionLineShape, PredictionSet,
    spectrum::{line_shape_value, require_finite, require_positive},
};

/// Options for rendering predicted two-dimensional correlations into a spectrum.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PredictionSpectrum2DOptions {
    /// Optional experiment filter.
    pub experiment: Option<Experiment>,
    /// Optional x-axis nucleus filter.
    pub x_nucleus: Option<Nucleus>,
    /// Optional y-axis nucleus filter.
    pub y_nucleus: Option<Nucleus>,
    /// Left x-axis bound in ppm.
    pub x_from_ppm: f64,
    /// Right x-axis bound in ppm.
    pub x_to_ppm: f64,
    /// Number of x-axis output points.
    pub x_points: usize,
    /// Lower y-axis bound in ppm.
    pub y_from_ppm: f64,
    /// Upper y-axis bound in ppm.
    pub y_to_ppm: f64,
    /// Number of y-axis output points.
    pub y_points: usize,
    /// X-axis spectrometer frequency in MHz.
    pub x_spectrometer_mhz: f64,
    /// Y-axis spectrometer frequency in MHz.
    pub y_spectrometer_mhz: f64,
    /// X-axis full width at half maximum in Hz.
    pub x_line_width_hz: f64,
    /// Y-axis full width at half maximum in Hz.
    pub y_line_width_hz: f64,
    /// Line shape used in both dimensions.
    pub line_shape: PredictionLineShape,
    /// Multiplicative volume scale applied to predicted intensities.
    pub volume_scale: f64,
}

impl Default for PredictionSpectrum2DOptions {
    fn default() -> Self {
        Self {
            experiment: None,
            x_nucleus: None,
            y_nucleus: None,
            x_from_ppm: -1.0,
            x_to_ppm: 12.0,
            x_points: 512,
            y_from_ppm: -10.0,
            y_to_ppm: 220.0,
            y_points: 512,
            x_spectrometer_mhz: 400.0,
            y_spectrometer_mhz: 100.0,
            x_line_width_hz: 2.0,
            y_line_width_hz: 8.0,
            line_shape: PredictionLineShape::Lorentzian,
            volume_scale: 1.0,
        }
    }
}

impl PredictionSpectrum2DOptions {
    /// Creates default two-dimensional prediction rendering options.
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

    /// Sets the x-axis nucleus filter.
    #[must_use]
    pub fn with_x_nucleus(mut self, nucleus: Nucleus) -> Self {
        self.x_nucleus = Some(nucleus);
        self
    }

    /// Sets the y-axis nucleus filter.
    #[must_use]
    pub fn with_y_nucleus(mut self, nucleus: Nucleus) -> Self {
        self.y_nucleus = Some(nucleus);
        self
    }

    /// Clears both nucleus filters.
    #[must_use]
    pub fn without_nuclei(mut self) -> Self {
        self.x_nucleus = None;
        self.y_nucleus = None;
        self
    }

    /// Sets the x-axis ppm range and point count.
    #[must_use]
    pub fn with_x_axis(mut self, from_ppm: f64, to_ppm: f64, points: usize) -> Self {
        self.x_from_ppm = from_ppm;
        self.x_to_ppm = to_ppm;
        self.x_points = points;
        self
    }

    /// Sets the y-axis ppm range and point count.
    #[must_use]
    pub fn with_y_axis(mut self, from_ppm: f64, to_ppm: f64, points: usize) -> Self {
        self.y_from_ppm = from_ppm;
        self.y_to_ppm = to_ppm;
        self.y_points = points;
        self
    }

    /// Sets the x-axis spectrometer frequency in MHz.
    #[must_use]
    pub fn with_x_spectrometer_mhz(mut self, spectrometer_mhz: f64) -> Self {
        self.x_spectrometer_mhz = spectrometer_mhz;
        self
    }

    /// Sets the y-axis spectrometer frequency in MHz.
    #[must_use]
    pub fn with_y_spectrometer_mhz(mut self, spectrometer_mhz: f64) -> Self {
        self.y_spectrometer_mhz = spectrometer_mhz;
        self
    }

    /// Sets the x-axis full width at half maximum in Hz.
    #[must_use]
    pub fn with_x_line_width_hz(mut self, line_width_hz: f64) -> Self {
        self.x_line_width_hz = line_width_hz;
        self
    }

    /// Sets the y-axis full width at half maximum in Hz.
    #[must_use]
    pub fn with_y_line_width_hz(mut self, line_width_hz: f64) -> Self {
        self.y_line_width_hz = line_width_hz;
        self
    }

    /// Sets the line shape used in both dimensions.
    #[must_use]
    pub fn with_line_shape(mut self, line_shape: PredictionLineShape) -> Self {
        self.line_shape = line_shape;
        self
    }

    /// Sets the multiplicative volume scale.
    #[must_use]
    pub fn with_volume_scale(mut self, volume_scale: f64) -> Self {
        self.volume_scale = volume_scale;
        self
    }

    fn validate(&self) -> Result<()> {
        require_finite("x_from_ppm", self.x_from_ppm)?;
        require_finite("x_to_ppm", self.x_to_ppm)?;
        require_finite("y_from_ppm", self.y_from_ppm)?;
        require_finite("y_to_ppm", self.y_to_ppm)?;
        require_positive("x_spectrometer_mhz", self.x_spectrometer_mhz)?;
        require_positive("y_spectrometer_mhz", self.y_spectrometer_mhz)?;
        require_positive("x_line_width_hz", self.x_line_width_hz)?;
        require_positive("y_line_width_hz", self.y_line_width_hz)?;
        require_finite("volume_scale", self.volume_scale)?;
        if (self.x_from_ppm - self.x_to_ppm).abs() <= f64::EPSILON {
            return Err(RSpinError::InvalidSpectrum {
                message: "prediction x ppm window must have distinct bounds".to_owned(),
            });
        }
        if (self.y_from_ppm - self.y_to_ppm).abs() <= f64::EPSILON {
            return Err(RSpinError::InvalidSpectrum {
                message: "prediction y ppm window must have distinct bounds".to_owned(),
            });
        }
        if self.x_points == 0 || self.y_points == 0 {
            return Err(RSpinError::InvalidSpectrum {
                message: "prediction 2D point counts must be positive".to_owned(),
            });
        }
        Ok(())
    }
}

/// Renders predicted two-dimensional correlations into a dense spectrum.
///
/// Correlations are filtered by experiment and axis nuclei when those options
/// are set. Empty selections produce a valid zero-valued matrix on the
/// requested axes.
///
/// # Errors
///
/// Returns an error when the prediction payload or rendering options are invalid.
pub fn render_prediction_2d(
    prediction: &PredictionSet,
    options: &PredictionSpectrum2DOptions,
) -> Result<Spectrum2D> {
    prediction.validate()?;
    options.validate()?;

    let correlations = selected_correlations(prediction, options);
    let x_axis = Axis::linear(
        "chemical shift x",
        Unit::Ppm,
        options.x_from_ppm,
        options.x_to_ppm,
        options.x_points,
    )?;
    let y_axis = Axis::linear(
        "chemical shift y",
        Unit::Ppm,
        options.y_from_ppm,
        options.y_to_ppm,
        options.y_points,
    )?;
    let z = render_correlations(&x_axis.values, &y_axis.values, &correlations, options);
    let metadata = Metadata {
        name: spectrum_name(prediction),
        nucleus: infer_common_nucleus(&correlations, options),
        frequency_mhz: infer_common_frequency(options),
        origin: prediction
            .provenance
            .as_ref()
            .map(|provenance| provenance.source.clone()),
        ..Metadata::default()
    };

    let spectrum = Spectrum2D::new(x_axis, y_axis, z, metadata)?.with_processing_record(
        ProcessingRecord::new("render_prediction_2d")
            .with_details(format!("{} correlations rendered", correlations.len())),
    );
    Ok(spectrum)
}

fn selected_correlations<'a>(
    prediction: &'a PredictionSet,
    options: &PredictionSpectrum2DOptions,
) -> Vec<&'a PredictedCorrelation2D> {
    prediction
        .correlations_2d
        .iter()
        .filter(|correlation| {
            options
                .experiment
                .as_ref()
                .is_none_or(|experiment| correlation.experiment == *experiment)
                && options
                    .x_nucleus
                    .as_ref()
                    .is_none_or(|nucleus| correlation.x_nucleus == *nucleus)
                && options
                    .y_nucleus
                    .as_ref()
                    .is_none_or(|nucleus| correlation.y_nucleus == *nucleus)
        })
        .collect()
}

fn render_correlations(
    x_axis: &[f64],
    y_axis: &[f64],
    correlations: &[&PredictedCorrelation2D],
    options: &PredictionSpectrum2DOptions,
) -> Vec<f64> {
    let mut z = vec![0.0; x_axis.len() * y_axis.len()];
    for correlation in correlations {
        let x_profile = x_axis
            .iter()
            .copied()
            .map(|x_ppm| {
                line_shape_value(
                    options.line_shape,
                    x_ppm,
                    correlation.x_ppm,
                    options.x_line_width_hz,
                    options.x_spectrometer_mhz,
                    1.0,
                )
            })
            .collect::<Vec<_>>();
        let y_profile = y_axis
            .iter()
            .copied()
            .map(|y_ppm| {
                line_shape_value(
                    options.line_shape,
                    y_ppm,
                    correlation.y_ppm,
                    options.y_line_width_hz,
                    options.y_spectrometer_mhz,
                    1.0,
                )
            })
            .collect::<Vec<_>>();
        let volume = correlation.intensity * options.volume_scale;

        for (y_index, y_value) in y_profile.iter().copied().enumerate() {
            let row_offset = y_index * x_axis.len();
            for (x_index, x_value) in x_profile.iter().copied().enumerate() {
                z[row_offset + x_index] += volume * x_value * y_value;
            }
        }
    }
    z
}

fn spectrum_name(prediction: &PredictionSet) -> Option<String> {
    prediction
        .name
        .as_ref()
        .map(|name| format!("{name} predicted 2D spectrum"))
}

fn infer_common_frequency(options: &PredictionSpectrum2DOptions) -> Option<f64> {
    if (options.x_spectrometer_mhz - options.y_spectrometer_mhz).abs() <= f64::EPSILON {
        Some(options.x_spectrometer_mhz)
    } else {
        None
    }
}

fn infer_common_nucleus(
    correlations: &[&PredictedCorrelation2D],
    options: &PredictionSpectrum2DOptions,
) -> Option<Nucleus> {
    if let (Some(x_nucleus), Some(y_nucleus)) = (&options.x_nucleus, &options.y_nucleus)
        && x_nucleus == y_nucleus
    {
        return Some(x_nucleus.clone());
    }

    let mut nuclei = correlations
        .iter()
        .flat_map(|correlation| [&correlation.x_nucleus, &correlation.y_nucleus]);
    let first = nuclei.next()?;
    if nuclei.all(|nucleus| nucleus == first) {
        Some(first.clone())
    } else {
        None
    }
}

#[cfg(test)]
mod tests;
