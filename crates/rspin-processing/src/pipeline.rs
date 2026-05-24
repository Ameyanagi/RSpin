//! Chainable one-dimensional processing pipelines.

use rspin_core::{Axis, Result, Spectrum1D};

use crate::{
    Abs1D, AutoPhaseOptions, BaselineMethod, Crop1D, ExponentialApodization, Fft1D, FftDirection,
    GaussianApodization, Magnitude, NormalizeArea, NormalizeMaxAbs, OffsetIntensity,
    PhaseCorrection, ProcessingStep, Resample1D, ScaleIntensity, ShiftAxis, SineBellApodization,
    SubtractBaseline, ZeroFill,
};

/// Chainable processor for one-dimensional spectra.
///
/// The pipeline stores the first error it encounters and skips later steps.
/// Call [`finish`](Self::finish) to retrieve the processed spectrum or error.
#[derive(Debug)]
pub struct Spectrum1DPipeline {
    result: Result<Spectrum1D>,
}

impl Spectrum1DPipeline {
    /// Starts a pipeline from an owned spectrum.
    #[must_use]
    pub fn new(spectrum: Spectrum1D) -> Self {
        Self {
            result: Ok(spectrum),
        }
    }

    /// Starts a pipeline from a borrowed spectrum.
    #[must_use]
    pub fn from_spectrum(spectrum: &Spectrum1D) -> Self {
        Self::new(spectrum.clone())
    }

    /// Starts a pipeline from an existing result.
    #[must_use]
    pub fn from_result(result: Result<Spectrum1D>) -> Self {
        Self { result }
    }

    /// Applies a reusable processing step.
    #[must_use]
    pub fn then<T>(self, step: T) -> Self
    where
        T: ProcessingStep<Spectrum1D>,
    {
        self.try_then(move |spectrum| step.apply(spectrum))
    }

    /// Applies a custom fallible processing function.
    #[must_use]
    pub fn try_then<F>(self, process: F) -> Self
    where
        F: FnOnce(&Spectrum1D) -> Result<Spectrum1D>,
    {
        let result = match self.result {
            Ok(spectrum) => process(&spectrum),
            Err(error) => Err(error),
        };
        Self { result }
    }

    /// Multiplies all real and imaginary intensities by `factor`.
    #[must_use]
    pub fn scale(self, factor: f64) -> Self {
        self.then(ScaleIntensity::new(factor))
    }

    /// Adds `offset` to all real intensities.
    #[must_use]
    pub fn offset(self, offset: f64) -> Self {
        self.then(OffsetIntensity::new(offset))
    }

    /// Normalizes real intensities by their maximum absolute value.
    #[must_use]
    pub fn normalize_max_abs(self) -> Self {
        self.then(NormalizeMaxAbs::new())
    }

    /// Normalizes real and imaginary intensities by signed trapezoidal area.
    #[must_use]
    pub fn normalize_area(self, target_area: f64) -> Self {
        self.then(NormalizeArea::new(target_area))
    }

    /// Normalizes real and imaginary intensities by absolute trapezoidal area.
    #[must_use]
    pub fn normalize_abs_area(self, target_area: f64) -> Self {
        self.then(NormalizeArea::absolute(target_area))
    }

    /// Applies component-wise absolute value to real and imaginary channels.
    #[must_use]
    pub fn absolute_value(self) -> Self {
        self.then(Abs1D::new())
    }

    /// Shifts the x-axis values by `delta`.
    #[must_use]
    pub fn shift_axis(self, delta: f64) -> Self {
        self.then(ShiftAxis::new(delta))
    }

    /// Appends zeroes until the spectrum reaches `target_len` points.
    #[must_use]
    pub fn zero_fill(self, target_len: usize) -> Self {
        self.then(ZeroFill::new(target_len))
    }

    /// Keeps points whose x coordinates fall inside an inclusive window.
    #[must_use]
    pub fn crop(self, from: f64, to: f64) -> Self {
        self.then(Crop1D::new(from, to))
    }

    /// Linearly resamples real and imaginary channels onto `target_axis`.
    #[must_use]
    pub fn resample(self, target_axis: Axis) -> Self {
        self.then(Resample1D::new(target_axis))
    }

    /// Linearly resamples onto `target_axis` with an explicit outside value.
    #[must_use]
    pub fn resample_with_outside(self, target_axis: Axis, outside_value: f64) -> Self {
        self.then(Resample1D::new(target_axis).with_outside_value(outside_value))
    }

    /// Applies exponential apodization to real and imaginary channels.
    #[must_use]
    pub fn exponential_apodization(self, line_broadening_hz: f64, dwell_time_s: f64) -> Self {
        self.then(ExponentialApodization::new(
            line_broadening_hz,
            dwell_time_s,
        ))
    }

    /// Applies Gaussian apodization to real and imaginary channels.
    #[must_use]
    pub fn gaussian_apodization(self, gaussian_broadening_hz: f64, dwell_time_s: f64) -> Self {
        self.then(GaussianApodization::new(
            gaussian_broadening_hz,
            dwell_time_s,
        ))
    }

    /// Applies sine-bell apodization to real and imaginary channels.
    #[must_use]
    pub fn sine_bell_apodization(
        self,
        start_angle_deg: f64,
        end_angle_deg: f64,
        exponent: f64,
    ) -> Self {
        self.then(SineBellApodization::new(
            start_angle_deg,
            end_angle_deg,
            exponent,
        ))
    }

    /// Applies a forward or inverse FFT.
    #[must_use]
    pub fn fft(self, direction: FftDirection) -> Self {
        self.then(Fft1D::new(direction))
    }

    /// Converts the spectrum to magnitude mode.
    #[must_use]
    pub fn magnitude(self) -> Self {
        self.then(Magnitude::new())
    }

    /// Applies manual zero- and first-order phase correction.
    #[must_use]
    pub fn phase(self, zero_order_deg: f64, first_order_deg: f64, pivot_fraction: f64) -> Self {
        self.then(PhaseCorrection::from_degrees(
            zero_order_deg,
            first_order_deg,
            pivot_fraction,
        ))
    }

    /// Applies automatic phase correction with default options.
    #[must_use]
    pub fn auto_phase(self) -> Self {
        self.auto_phase_with(AutoPhaseOptions::default())
    }

    /// Applies automatic phase correction with explicit options.
    #[must_use]
    pub fn auto_phase_with(self, options: AutoPhaseOptions) -> Self {
        self.then(crate::AutoPhaseCorrection::with_options(options))
    }

    /// Subtracts a fitted baseline using the default method.
    #[must_use]
    pub fn subtract_baseline(self) -> Self {
        self.subtract_baseline_with(BaselineMethod::default())
    }

    /// Subtracts a fitted baseline using an explicit method.
    #[must_use]
    pub fn subtract_baseline_with(self, method: BaselineMethod) -> Self {
        self.then(SubtractBaseline::new(method))
    }

    /// Returns the processed spectrum.
    ///
    /// # Errors
    ///
    /// Returns the first error encountered by any step in the pipeline.
    pub fn finish(self) -> Result<Spectrum1D> {
        self.result
    }
}

/// Convenience extension trait for starting one-dimensional pipelines.
pub trait ProcessSpectrum1D {
    /// Starts a chainable processing pipeline.
    fn process(self) -> Spectrum1DPipeline;
}

impl ProcessSpectrum1D for Spectrum1D {
    fn process(self) -> Spectrum1DPipeline {
        Spectrum1DPipeline::new(self)
    }
}

impl ProcessSpectrum1D for &Spectrum1D {
    fn process(self) -> Spectrum1DPipeline {
        Spectrum1DPipeline::from_spectrum(self)
    }
}

impl ProcessSpectrum1D for Result<Spectrum1D> {
    fn process(self) -> Spectrum1DPipeline {
        Spectrum1DPipeline::from_result(self)
    }
}

#[cfg(test)]
mod tests {
    use rspin_core::{Axis, Metadata, RSpinError, Unit};

    use super::*;

    #[test]
    fn chains_common_processing_steps() -> anyhow::Result<()> {
        let spectrum = demo_spectrum()?;
        let processed = spectrum
            .process()
            .scale(2.0)
            .offset(-2.0)
            .absolute_value()
            .crop(0.0, 1.0)
            .resample(Axis::linear("shift", Unit::Ppm, 0.0, 1.0, 3)?)
            .zero_fill(5)
            .gaussian_apodization(0.0, 0.1)
            .sine_bell_apodization(90.0, 90.0, 1.0)
            .normalize_max_abs()
            .normalize_abs_area(1.5)
            .finish()?;

        assert_eq!(processed.len(), 5);
        assert_eq!(processed.intensities, vec![0.0, 1.0, 2.0, 0.0, 0.0]);
        assert_eq!(processed.processing.len(), 10);
        assert_eq!(processed.processing[0].operation, "scale_intensity");
        assert_eq!(processed.processing[2].operation, "abs_1d");
        assert_eq!(processed.processing[3].operation, "crop_1d");
        assert_eq!(processed.processing[4].operation, "resample_1d");
        assert_eq!(processed.processing[6].operation, "gaussian_apodization");
        assert_eq!(processed.processing[7].operation, "sine_bell_apodization");
        assert_eq!(processed.processing[8].operation, "normalize_max_abs");
        assert_eq!(processed.processing[9].operation, "normalize_area");
        Ok(())
    }

    #[test]
    fn borrowed_pipeline_leaves_original_spectrum_unchanged() -> anyhow::Result<()> {
        let spectrum = demo_spectrum()?;
        let processed = (&spectrum).process().shift_axis(1.0).finish()?;

        assert_eq!(spectrum.x.values, vec![0.0, 1.0, 2.0]);
        assert_eq!(processed.x.values, vec![1.0, 2.0, 3.0]);
        Ok(())
    }

    #[test]
    fn chains_from_fallible_spectrum_result() -> anyhow::Result<()> {
        let spectrum_result: rspin_core::Result<Spectrum1D> = Ok(demo_spectrum()?);
        let processed = spectrum_result.process().scale(2.0).finish()?;

        assert_eq!(processed.intensities, vec![2.0, -2.0, 6.0]);
        assert_eq!(processed.processing.len(), 1);
        assert_eq!(processed.processing[0].operation, "scale_intensity");
        Ok(())
    }

    #[test]
    fn result_pipeline_preserves_initial_error() {
        let spectrum_result: rspin_core::Result<Spectrum1D> = Err(RSpinError::InvalidSpectrum {
            message: "initial failure".to_owned(),
        });
        let error = spectrum_result
            .process()
            .scale(2.0)
            .finish()
            .expect_err("initial error should be preserved");

        assert_eq!(
            error,
            RSpinError::InvalidSpectrum {
                message: "initial failure".to_owned()
            }
        );
    }

    #[test]
    fn accepts_custom_steps_and_preserves_first_error() -> anyhow::Result<()> {
        let spectrum = demo_spectrum()?;
        let error = spectrum
            .process()
            .then(ScaleIntensity { factor: f64::NAN })
            .offset(10.0)
            .finish()
            .expect_err("non-finite scale should fail");

        assert!(matches!(error, RSpinError::NonFinite { .. }));
        Ok(())
    }

    #[test]
    fn chains_phase_and_magnitude_steps() -> anyhow::Result<()> {
        let spectrum = Spectrum1D::new_complex(
            Axis::linear("shift", Unit::Ppm, 0.0, 1.0, 2)?,
            vec![1.0, 0.0],
            Some(vec![0.0, 1.0]),
            Metadata::default(),
        )?;
        let processed = spectrum
            .process()
            .phase(90.0, 0.0, 0.5)
            .magnitude()
            .finish()?;

        assert_eq!(processed.imaginary, None);
        assert!((processed.intensities[0] - 1.0).abs() < 1.0e-12);
        assert!((processed.intensities[1] - 1.0).abs() < 1.0e-12);
        Ok(())
    }

    fn demo_spectrum() -> anyhow::Result<Spectrum1D> {
        Ok(Spectrum1D::new(
            Axis::linear("shift", Unit::Ppm, 0.0, 2.0, 3)?,
            vec![1.0, -1.0, 3.0],
            Metadata::default(),
        )?)
    }
}
