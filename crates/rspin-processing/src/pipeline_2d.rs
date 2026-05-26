//! Chainable two-dimensional processing pipelines.

use rspin_core::{Axis, Result, Spectrum1D, Spectrum2D};

use crate::{
    Abs2D, AutoPhase2DOptions, AutoPhaseCorrection2D, Crop2D, ExponentialApodization2D, Fft2D,
    FftDirection, GaussianApodization2D, HyperComplex2DOptions, Normalize2DMaxAbs,
    Normalize2DVolume, Offset2D, PhaseCorrection2D, ProcessingStep, ProjectionMode, Resample2D,
    Scale2D, Shift2DAxes, SineBellApodization2D, ZeroFill2D, process_hypercomplex_2d, project_x,
    project_y, slice_x_at_y, slice_x_at_y_index, slice_y_at_x, slice_y_at_x_index,
};

/// Chainable processor for two-dimensional spectra.
///
/// The pipeline stores the first error it encounters and skips later steps.
/// Call [`finish`](Self::finish) to retrieve a processed 2D spectrum, or use a
/// terminal projection/slice method to derive a 1D spectrum.
#[derive(Debug)]
pub struct Spectrum2DPipeline {
    result: Result<Spectrum2D>,
}

impl Spectrum2DPipeline {
    /// Starts a pipeline from an owned spectrum.
    #[must_use]
    pub fn new(spectrum: Spectrum2D) -> Self {
        Self {
            result: Ok(spectrum),
        }
    }

    /// Starts a pipeline from a borrowed spectrum.
    #[must_use]
    pub fn from_spectrum(spectrum: &Spectrum2D) -> Self {
        Self::new(spectrum.clone())
    }

    /// Starts a pipeline from an existing result.
    #[must_use]
    pub fn from_result(result: Result<Spectrum2D>) -> Self {
        Self { result }
    }

    /// Starts a pipeline by processing a raw `ser`-style hypercomplex spectrum
    /// into a phasable spectrum (direct FT, quadrature assembly, indirect FT,
    /// phase), then continues as a normal 2D pipeline.
    #[must_use]
    pub fn from_raw_hypercomplex(raw: &Spectrum2D, options: &HyperComplex2DOptions) -> Self {
        Self::from_result(process_hypercomplex_2d(raw, options))
    }

    /// Applies a reusable processing step.
    #[must_use]
    pub fn then<T>(self, step: T) -> Self
    where
        T: ProcessingStep<Spectrum2D>,
    {
        self.try_then(move |spectrum| step.apply(spectrum))
    }

    /// Applies a custom fallible processing function.
    #[must_use]
    pub fn try_then<F>(self, process: F) -> Self
    where
        F: FnOnce(&Spectrum2D) -> Result<Spectrum2D>,
    {
        let result = match self.result {
            Ok(spectrum) => process(&spectrum),
            Err(error) => Err(error),
        };
        Self { result }
    }

    /// Multiplies all intensities by `factor`.
    #[must_use]
    pub fn scale(self, factor: f64) -> Self {
        self.then(Scale2D::new(factor))
    }

    /// Adds an offset to all real intensities.
    #[must_use]
    pub fn offset(self, offset: f64) -> Self {
        self.then(Offset2D::new(offset))
    }

    /// Normalizes intensities by their maximum absolute value.
    #[must_use]
    pub fn normalize_max_abs(self) -> Self {
        self.then(Normalize2DMaxAbs::new())
    }

    /// Normalizes real and imaginary values by signed bilinear volume.
    #[must_use]
    pub fn normalize_volume(self, target_volume: f64) -> Self {
        self.then(Normalize2DVolume::new(target_volume))
    }

    /// Normalizes real and imaginary values by absolute bilinear volume.
    #[must_use]
    pub fn normalize_abs_volume(self, target_volume: f64) -> Self {
        self.then(Normalize2DVolume::absolute(target_volume))
    }

    /// Shifts x and y axes by constant deltas.
    #[must_use]
    pub fn shift_axes(self, x_delta: f64, y_delta: f64) -> Self {
        self.then(Shift2DAxes::new(x_delta, y_delta))
    }

    /// Shifts only the x axis.
    #[must_use]
    pub fn shift_x_axis(self, delta: f64) -> Self {
        self.then(Shift2DAxes::x(delta))
    }

    /// Shifts only the y axis.
    #[must_use]
    pub fn shift_y_axis(self, delta: f64) -> Self {
        self.then(Shift2DAxes::y(delta))
    }

    /// Applies component-wise absolute value to real and imaginary matrices.
    #[must_use]
    pub fn absolute_value(self) -> Self {
        self.then(Abs2D::new())
    }

    /// Appends zeroes in x and y until the requested shape is reached.
    #[must_use]
    pub fn zero_fill(self, target_width: usize, target_height: usize) -> Self {
        self.then(ZeroFill2D::new(target_width, target_height))
    }

    /// Keeps points inside inclusive x and y coordinate windows.
    #[must_use]
    pub fn crop(self, x_from: f64, x_to: f64, y_from: f64, y_to: f64) -> Self {
        self.then(Crop2D::new(x_from, x_to, y_from, y_to))
    }

    /// Bilinearly resamples real and imaginary matrices onto target axes.
    #[must_use]
    pub fn resample(self, target_x: Axis, target_y: Axis) -> Self {
        self.then(Resample2D::new(target_x, target_y))
    }

    /// Bilinearly resamples onto target axes with an explicit outside value.
    #[must_use]
    pub fn resample_with_outside(self, target_x: Axis, target_y: Axis, outside_value: f64) -> Self {
        self.then(Resample2D::new(target_x, target_y).with_outside_value(outside_value))
    }

    /// Applies separable exponential apodization.
    #[must_use]
    pub fn exponential_apodization(
        self,
        x_line_broadening_hz: f64,
        y_line_broadening_hz: f64,
        x_dwell_time_s: f64,
        y_dwell_time_s: f64,
    ) -> Self {
        self.then(ExponentialApodization2D::new(
            x_line_broadening_hz,
            y_line_broadening_hz,
            x_dwell_time_s,
            y_dwell_time_s,
        ))
    }

    /// Applies separable Gaussian apodization.
    #[must_use]
    pub fn gaussian_apodization(
        self,
        x_gaussian_broadening_hz: f64,
        y_gaussian_broadening_hz: f64,
        x_dwell_time_s: f64,
        y_dwell_time_s: f64,
    ) -> Self {
        self.then(GaussianApodization2D::new(
            x_gaussian_broadening_hz,
            y_gaussian_broadening_hz,
            x_dwell_time_s,
            y_dwell_time_s,
        ))
    }

    /// Applies separable sine-bell apodization.
    #[must_use]
    pub fn sine_bell_apodization(
        self,
        x_start_angle_deg: f64,
        x_end_angle_deg: f64,
        x_exponent: f64,
        y_start_angle_deg: f64,
        y_end_angle_deg: f64,
        y_exponent: f64,
    ) -> Self {
        self.then(SineBellApodization2D::new(
            x_start_angle_deg,
            x_end_angle_deg,
            x_exponent,
            y_start_angle_deg,
            y_end_angle_deg,
            y_exponent,
        ))
    }

    /// Applies a full two-dimensional FFT or inverse FFT.
    #[must_use]
    pub fn fft(self, direction: FftDirection) -> Self {
        self.then(Fft2D::new(direction))
    }

    /// Applies a full two-dimensional phase correction.
    #[must_use]
    pub fn phase(self, correction: PhaseCorrection2D) -> Self {
        self.then(correction)
    }

    /// Applies x-dimension phase correction.
    #[must_use]
    pub fn phase_x(self, zero_order_deg: f64, first_order_deg: f64, pivot_fraction: f64) -> Self {
        self.then(PhaseCorrection2D::new().x_phase(zero_order_deg, first_order_deg, pivot_fraction))
    }

    /// Applies y-dimension phase correction.
    #[must_use]
    pub fn phase_y(self, zero_order_deg: f64, first_order_deg: f64, pivot_fraction: f64) -> Self {
        self.then(PhaseCorrection2D::new().y_phase(zero_order_deg, first_order_deg, pivot_fraction))
    }

    /// Applies automatic two-dimensional phase correction with default options.
    #[must_use]
    pub fn auto_phase(self) -> Self {
        self.auto_phase_with(AutoPhase2DOptions::default())
    }

    /// Applies automatic two-dimensional phase correction with explicit options.
    #[must_use]
    pub fn auto_phase_with(self, options: AutoPhase2DOptions) -> Self {
        self.then(AutoPhaseCorrection2D::with_options(options))
    }

    /// Returns the processed 2D spectrum.
    ///
    /// # Errors
    ///
    /// Returns the first error encountered by any step in the pipeline.
    pub fn finish(self) -> Result<Spectrum2D> {
        self.result
    }

    /// Projects the processed 2D spectrum onto the x axis.
    ///
    /// # Errors
    ///
    /// Returns the first pipeline error, or an error if projection fails.
    pub fn project_x(self, mode: ProjectionMode) -> Result<Spectrum1D> {
        self.finish()
            .and_then(|spectrum| project_x(&spectrum, mode))
    }

    /// Projects the processed 2D spectrum onto the y axis.
    ///
    /// # Errors
    ///
    /// Returns the first pipeline error, or an error if projection fails.
    pub fn project_y(self, mode: ProjectionMode) -> Result<Spectrum1D> {
        self.finish()
            .and_then(|spectrum| project_y(&spectrum, mode))
    }

    /// Extracts the row at `y_index` as a 1D spectrum over x.
    ///
    /// # Errors
    ///
    /// Returns the first pipeline error, or an error if `y_index` is out of bounds.
    pub fn slice_x_at_y_index(self, y_index: usize) -> Result<Spectrum1D> {
        self.finish()
            .and_then(|spectrum| slice_x_at_y_index(&spectrum, y_index))
    }

    /// Extracts the row nearest `y` as a 1D spectrum over x.
    ///
    /// # Errors
    ///
    /// Returns the first pipeline error, or an error if `y` is not finite.
    pub fn slice_x_at_y(self, y: f64) -> Result<Spectrum1D> {
        self.finish()
            .and_then(|spectrum| slice_x_at_y(&spectrum, y))
    }

    /// Extracts the column at `x_index` as a 1D spectrum over y.
    ///
    /// # Errors
    ///
    /// Returns the first pipeline error, or an error if `x_index` is out of bounds.
    pub fn slice_y_at_x_index(self, x_index: usize) -> Result<Spectrum1D> {
        self.finish()
            .and_then(|spectrum| slice_y_at_x_index(&spectrum, x_index))
    }

    /// Extracts the column nearest `x` as a 1D spectrum over y.
    ///
    /// # Errors
    ///
    /// Returns the first pipeline error, or an error if `x` is not finite.
    pub fn slice_y_at_x(self, x: f64) -> Result<Spectrum1D> {
        self.finish()
            .and_then(|spectrum| slice_y_at_x(&spectrum, x))
    }
}

/// Convenience extension trait for starting two-dimensional pipelines.
pub trait ProcessSpectrum2D {
    /// Starts a chainable processing pipeline.
    fn process(self) -> Spectrum2DPipeline;
}

impl ProcessSpectrum2D for Spectrum2D {
    fn process(self) -> Spectrum2DPipeline {
        Spectrum2DPipeline::new(self)
    }
}

impl ProcessSpectrum2D for &Spectrum2D {
    fn process(self) -> Spectrum2DPipeline {
        Spectrum2DPipeline::from_spectrum(self)
    }
}

impl ProcessSpectrum2D for Result<Spectrum2D> {
    fn process(self) -> Spectrum2DPipeline {
        Spectrum2DPipeline::from_result(self)
    }
}

#[cfg(test)]
mod tests;
