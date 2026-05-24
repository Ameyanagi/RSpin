//! Chainable prediction rendering workflows.

use rspin_core::{Nucleus, Result, Spectrum1D, Spectrum2D};

use crate::{
    Experiment, PredictionLineShape, PredictionSet, PredictionSpectrum2DOptions,
    PredictionSpectrumOptions, render_prediction_1d, render_prediction_2d,
};

/// Extension trait for chainable one-dimensional prediction rendering.
pub trait RenderPrediction1D {
    /// Creates a borrowed one-dimensional prediction rendering workflow.
    #[must_use]
    fn render_1d(&self) -> PredictionSpectrum1DWorkflow<'_>;
}

impl RenderPrediction1D for PredictionSet {
    fn render_1d(&self) -> PredictionSpectrum1DWorkflow<'_> {
        PredictionSpectrum1DWorkflow::new(self)
    }
}

/// Extension trait for chainable one-dimensional rendering from fallible inputs.
pub trait RenderPrediction1DResult {
    /// Creates an owned one-dimensional prediction rendering workflow.
    #[must_use]
    fn render_1d(self) -> PredictionSpectrum1DResultWorkflow;
}

impl RenderPrediction1DResult for Result<PredictionSet> {
    fn render_1d(self) -> PredictionSpectrum1DResultWorkflow {
        PredictionSpectrum1DResultWorkflow::from_result(self)
    }
}

/// Extension trait for chainable two-dimensional prediction rendering.
pub trait RenderPrediction2D {
    /// Creates a borrowed two-dimensional prediction rendering workflow.
    #[must_use]
    fn render_2d(&self) -> PredictionSpectrum2DWorkflow<'_>;
}

impl RenderPrediction2D for PredictionSet {
    fn render_2d(&self) -> PredictionSpectrum2DWorkflow<'_> {
        PredictionSpectrum2DWorkflow::new(self)
    }
}

/// Extension trait for chainable two-dimensional rendering from fallible inputs.
pub trait RenderPrediction2DResult {
    /// Creates an owned two-dimensional prediction rendering workflow.
    #[must_use]
    fn render_2d(self) -> PredictionSpectrum2DResultWorkflow;
}

impl RenderPrediction2DResult for Result<PredictionSet> {
    fn render_2d(self) -> PredictionSpectrum2DResultWorkflow {
        PredictionSpectrum2DResultWorkflow::from_result(self)
    }
}

/// Borrowed builder for one-dimensional prediction rendering.
#[derive(Clone, Debug)]
pub struct PredictionSpectrum1DWorkflow<'a> {
    prediction: &'a PredictionSet,
    options: PredictionSpectrumOptions,
}

impl<'a> PredictionSpectrum1DWorkflow<'a> {
    /// Creates a one-dimensional rendering workflow for `prediction`.
    #[must_use]
    pub fn new(prediction: &'a PredictionSet) -> Self {
        Self {
            prediction,
            options: PredictionSpectrumOptions::default(),
        }
    }

    /// Replaces all rendering options.
    #[must_use]
    pub fn with_options(mut self, options: PredictionSpectrumOptions) -> Self {
        self.options = options;
        self
    }

    /// Sets the experiment filter.
    #[must_use]
    pub fn with_experiment(mut self, experiment: Experiment) -> Self {
        self.options.experiment = Some(experiment);
        self
    }

    /// Clears the experiment filter.
    #[must_use]
    pub fn without_experiment(mut self) -> Self {
        self.options.experiment = None;
        self
    }

    /// Sets the nucleus filter.
    #[must_use]
    pub fn with_nucleus(mut self, nucleus: Nucleus) -> Self {
        self.options.nucleus = Some(nucleus);
        self
    }

    /// Clears the nucleus filter.
    #[must_use]
    pub fn without_nucleus(mut self) -> Self {
        self.options.nucleus = None;
        self
    }

    /// Sets the output ppm range.
    #[must_use]
    pub fn with_ppm_range(mut self, from_ppm: f64, to_ppm: f64) -> Self {
        self.options.from_ppm = from_ppm;
        self.options.to_ppm = to_ppm;
        self
    }

    /// Sets the number of output points.
    #[must_use]
    pub fn with_points(mut self, points: usize) -> Self {
        self.options.points = points;
        self
    }

    /// Sets the spectrometer frequency in MHz.
    #[must_use]
    pub fn with_spectrometer_mhz(mut self, spectrometer_mhz: f64) -> Self {
        self.options.spectrometer_mhz = spectrometer_mhz;
        self
    }

    /// Sets the full width at half maximum in Hz.
    #[must_use]
    pub fn with_line_width_hz(mut self, line_width_hz: f64) -> Self {
        self.options.line_width_hz = line_width_hz;
        self
    }

    /// Sets the line shape used for each predicted signal.
    #[must_use]
    pub fn with_line_shape(mut self, line_shape: PredictionLineShape) -> Self {
        self.options.line_shape = line_shape;
        self
    }

    /// Sets the multiplicative area scale.
    #[must_use]
    pub fn with_area_scale(mut self, area_scale: f64) -> Self {
        self.options.area_scale = area_scale;
        self
    }

    /// Runs the configured rendering workflow.
    ///
    /// # Errors
    ///
    /// Returns an error when the prediction payload or options are invalid.
    pub fn run(self) -> Result<Spectrum1D> {
        render_prediction_1d(self.prediction, &self.options)
    }
}

/// Owned builder for one-dimensional prediction rendering from fallible inputs.
#[derive(Debug)]
pub struct PredictionSpectrum1DResultWorkflow {
    prediction: Result<PredictionSet>,
    options: PredictionSpectrumOptions,
}

impl PredictionSpectrum1DResultWorkflow {
    /// Creates a one-dimensional rendering workflow from an existing result.
    #[must_use]
    pub fn from_result(prediction: Result<PredictionSet>) -> Self {
        Self {
            prediction,
            options: PredictionSpectrumOptions::default(),
        }
    }

    /// Replaces all rendering options.
    #[must_use]
    pub fn with_options(mut self, options: PredictionSpectrumOptions) -> Self {
        self.options = options;
        self
    }

    /// Sets the experiment filter.
    #[must_use]
    pub fn with_experiment(mut self, experiment: Experiment) -> Self {
        self.options.experiment = Some(experiment);
        self
    }

    /// Clears the experiment filter.
    #[must_use]
    pub fn without_experiment(mut self) -> Self {
        self.options.experiment = None;
        self
    }

    /// Sets the nucleus filter.
    #[must_use]
    pub fn with_nucleus(mut self, nucleus: Nucleus) -> Self {
        self.options.nucleus = Some(nucleus);
        self
    }

    /// Clears the nucleus filter.
    #[must_use]
    pub fn without_nucleus(mut self) -> Self {
        self.options.nucleus = None;
        self
    }

    /// Sets the output ppm range.
    #[must_use]
    pub fn with_ppm_range(mut self, from_ppm: f64, to_ppm: f64) -> Self {
        self.options.from_ppm = from_ppm;
        self.options.to_ppm = to_ppm;
        self
    }

    /// Sets the number of output points.
    #[must_use]
    pub fn with_points(mut self, points: usize) -> Self {
        self.options.points = points;
        self
    }

    /// Sets the spectrometer frequency in MHz.
    #[must_use]
    pub fn with_spectrometer_mhz(mut self, spectrometer_mhz: f64) -> Self {
        self.options.spectrometer_mhz = spectrometer_mhz;
        self
    }

    /// Sets the full width at half maximum in Hz.
    #[must_use]
    pub fn with_line_width_hz(mut self, line_width_hz: f64) -> Self {
        self.options.line_width_hz = line_width_hz;
        self
    }

    /// Sets the line shape used for each predicted signal.
    #[must_use]
    pub fn with_line_shape(mut self, line_shape: PredictionLineShape) -> Self {
        self.options.line_shape = line_shape;
        self
    }

    /// Sets the multiplicative area scale.
    #[must_use]
    pub fn with_area_scale(mut self, area_scale: f64) -> Self {
        self.options.area_scale = area_scale;
        self
    }

    /// Runs the configured rendering workflow.
    ///
    /// # Errors
    ///
    /// Returns the initial prediction error, or an error from rendering.
    pub fn run(self) -> Result<Spectrum1D> {
        let prediction = self.prediction?;
        render_prediction_1d(&prediction, &self.options)
    }
}

/// Borrowed builder for two-dimensional prediction rendering.
#[derive(Clone, Debug)]
pub struct PredictionSpectrum2DWorkflow<'a> {
    prediction: &'a PredictionSet,
    options: PredictionSpectrum2DOptions,
}

impl<'a> PredictionSpectrum2DWorkflow<'a> {
    /// Creates a two-dimensional rendering workflow for `prediction`.
    #[must_use]
    pub fn new(prediction: &'a PredictionSet) -> Self {
        Self {
            prediction,
            options: PredictionSpectrum2DOptions::default(),
        }
    }

    /// Replaces all rendering options.
    #[must_use]
    pub fn with_options(mut self, options: PredictionSpectrum2DOptions) -> Self {
        self.options = options;
        self
    }

    /// Sets the experiment filter.
    #[must_use]
    pub fn with_experiment(mut self, experiment: Experiment) -> Self {
        self.options.experiment = Some(experiment);
        self
    }

    /// Clears the experiment filter.
    #[must_use]
    pub fn without_experiment(mut self) -> Self {
        self.options.experiment = None;
        self
    }

    /// Sets the x-axis nucleus filter.
    #[must_use]
    pub fn with_x_nucleus(mut self, nucleus: Nucleus) -> Self {
        self.options.x_nucleus = Some(nucleus);
        self
    }

    /// Sets the y-axis nucleus filter.
    #[must_use]
    pub fn with_y_nucleus(mut self, nucleus: Nucleus) -> Self {
        self.options.y_nucleus = Some(nucleus);
        self
    }

    /// Clears both nucleus filters.
    #[must_use]
    pub fn without_nuclei(mut self) -> Self {
        self.options.x_nucleus = None;
        self.options.y_nucleus = None;
        self
    }

    /// Sets the x-axis ppm range and point count.
    #[must_use]
    pub fn with_x_axis(mut self, from_ppm: f64, to_ppm: f64, points: usize) -> Self {
        self.options.x_from_ppm = from_ppm;
        self.options.x_to_ppm = to_ppm;
        self.options.x_points = points;
        self
    }

    /// Sets the y-axis ppm range and point count.
    #[must_use]
    pub fn with_y_axis(mut self, from_ppm: f64, to_ppm: f64, points: usize) -> Self {
        self.options.y_from_ppm = from_ppm;
        self.options.y_to_ppm = to_ppm;
        self.options.y_points = points;
        self
    }

    /// Sets the x-axis spectrometer frequency in MHz.
    #[must_use]
    pub fn with_x_spectrometer_mhz(mut self, spectrometer_mhz: f64) -> Self {
        self.options.x_spectrometer_mhz = spectrometer_mhz;
        self
    }

    /// Sets the y-axis spectrometer frequency in MHz.
    #[must_use]
    pub fn with_y_spectrometer_mhz(mut self, spectrometer_mhz: f64) -> Self {
        self.options.y_spectrometer_mhz = spectrometer_mhz;
        self
    }

    /// Sets the x-axis full width at half maximum in Hz.
    #[must_use]
    pub fn with_x_line_width_hz(mut self, line_width_hz: f64) -> Self {
        self.options.x_line_width_hz = line_width_hz;
        self
    }

    /// Sets the y-axis full width at half maximum in Hz.
    #[must_use]
    pub fn with_y_line_width_hz(mut self, line_width_hz: f64) -> Self {
        self.options.y_line_width_hz = line_width_hz;
        self
    }

    /// Sets the line shape used in both dimensions.
    #[must_use]
    pub fn with_line_shape(mut self, line_shape: PredictionLineShape) -> Self {
        self.options.line_shape = line_shape;
        self
    }

    /// Sets the multiplicative volume scale.
    #[must_use]
    pub fn with_volume_scale(mut self, volume_scale: f64) -> Self {
        self.options.volume_scale = volume_scale;
        self
    }

    /// Runs the configured rendering workflow.
    ///
    /// # Errors
    ///
    /// Returns an error when the prediction payload or options are invalid.
    pub fn run(self) -> Result<Spectrum2D> {
        render_prediction_2d(self.prediction, &self.options)
    }
}

/// Owned builder for two-dimensional prediction rendering from fallible inputs.
#[derive(Debug)]
pub struct PredictionSpectrum2DResultWorkflow {
    prediction: Result<PredictionSet>,
    options: PredictionSpectrum2DOptions,
}

impl PredictionSpectrum2DResultWorkflow {
    /// Creates a two-dimensional rendering workflow from an existing result.
    #[must_use]
    pub fn from_result(prediction: Result<PredictionSet>) -> Self {
        Self {
            prediction,
            options: PredictionSpectrum2DOptions::default(),
        }
    }

    /// Replaces all rendering options.
    #[must_use]
    pub fn with_options(mut self, options: PredictionSpectrum2DOptions) -> Self {
        self.options = options;
        self
    }

    /// Sets the experiment filter.
    #[must_use]
    pub fn with_experiment(mut self, experiment: Experiment) -> Self {
        self.options.experiment = Some(experiment);
        self
    }

    /// Clears the experiment filter.
    #[must_use]
    pub fn without_experiment(mut self) -> Self {
        self.options.experiment = None;
        self
    }

    /// Sets the x-axis nucleus filter.
    #[must_use]
    pub fn with_x_nucleus(mut self, nucleus: Nucleus) -> Self {
        self.options.x_nucleus = Some(nucleus);
        self
    }

    /// Sets the y-axis nucleus filter.
    #[must_use]
    pub fn with_y_nucleus(mut self, nucleus: Nucleus) -> Self {
        self.options.y_nucleus = Some(nucleus);
        self
    }

    /// Clears both nucleus filters.
    #[must_use]
    pub fn without_nuclei(mut self) -> Self {
        self.options.x_nucleus = None;
        self.options.y_nucleus = None;
        self
    }

    /// Sets the x-axis ppm range and point count.
    #[must_use]
    pub fn with_x_axis(mut self, from_ppm: f64, to_ppm: f64, points: usize) -> Self {
        self.options.x_from_ppm = from_ppm;
        self.options.x_to_ppm = to_ppm;
        self.options.x_points = points;
        self
    }

    /// Sets the y-axis ppm range and point count.
    #[must_use]
    pub fn with_y_axis(mut self, from_ppm: f64, to_ppm: f64, points: usize) -> Self {
        self.options.y_from_ppm = from_ppm;
        self.options.y_to_ppm = to_ppm;
        self.options.y_points = points;
        self
    }

    /// Sets the x-axis spectrometer frequency in MHz.
    #[must_use]
    pub fn with_x_spectrometer_mhz(mut self, spectrometer_mhz: f64) -> Self {
        self.options.x_spectrometer_mhz = spectrometer_mhz;
        self
    }

    /// Sets the y-axis spectrometer frequency in MHz.
    #[must_use]
    pub fn with_y_spectrometer_mhz(mut self, spectrometer_mhz: f64) -> Self {
        self.options.y_spectrometer_mhz = spectrometer_mhz;
        self
    }

    /// Sets the x-axis full width at half maximum in Hz.
    #[must_use]
    pub fn with_x_line_width_hz(mut self, line_width_hz: f64) -> Self {
        self.options.x_line_width_hz = line_width_hz;
        self
    }

    /// Sets the y-axis full width at half maximum in Hz.
    #[must_use]
    pub fn with_y_line_width_hz(mut self, line_width_hz: f64) -> Self {
        self.options.y_line_width_hz = line_width_hz;
        self
    }

    /// Sets the line shape used in both dimensions.
    #[must_use]
    pub fn with_line_shape(mut self, line_shape: PredictionLineShape) -> Self {
        self.options.line_shape = line_shape;
        self
    }

    /// Sets the multiplicative volume scale.
    #[must_use]
    pub fn with_volume_scale(mut self, volume_scale: f64) -> Self {
        self.options.volume_scale = volume_scale;
        self
    }

    /// Runs the configured rendering workflow.
    ///
    /// # Errors
    ///
    /// Returns the initial prediction error, or an error from rendering.
    pub fn run(self) -> Result<Spectrum2D> {
        let prediction = self.prediction?;
        render_prediction_2d(&prediction, &self.options)
    }
}

#[cfg(test)]
mod tests;
