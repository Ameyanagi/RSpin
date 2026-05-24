//! Chainable two-dimensional processing pipelines.

use rspin_core::{Result, Spectrum1D, Spectrum2D};

use crate::{
    ExponentialApodization2D, Fft2D, FftDirection, Normalize2DMaxAbs, ProcessingStep,
    ProjectionMode, Scale2D, ZeroFill2D, project_x, project_y, slice_x_at_y_index,
    slice_y_at_x_index,
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
        self.then(Scale2D { factor })
    }

    /// Normalizes intensities by their maximum absolute value.
    #[must_use]
    pub fn normalize_max_abs(self) -> Self {
        self.then(Normalize2DMaxAbs)
    }

    /// Appends zeroes in x and y until the requested shape is reached.
    #[must_use]
    pub fn zero_fill(self, target_width: usize, target_height: usize) -> Self {
        self.then(ZeroFill2D {
            target_x_len: target_width,
            target_y_len: target_height,
        })
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
        self.then(ExponentialApodization2D {
            x_line_broadening_hz,
            y_line_broadening_hz,
            x_dwell_time_s,
            y_dwell_time_s,
        })
    }

    /// Applies a full two-dimensional FFT or inverse FFT.
    #[must_use]
    pub fn fft(self, direction: FftDirection) -> Self {
        self.then(Fft2D { direction })
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

    /// Extracts the column at `x_index` as a 1D spectrum over y.
    ///
    /// # Errors
    ///
    /// Returns the first pipeline error, or an error if `x_index` is out of bounds.
    pub fn slice_y_at_x_index(self, x_index: usize) -> Result<Spectrum1D> {
        self.finish()
            .and_then(|spectrum| slice_y_at_x_index(&spectrum, x_index))
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

#[cfg(test)]
mod tests {
    use rspin_core::{Axis, Metadata, RSpinError, Unit};

    use super::*;

    #[test]
    fn chains_common_2d_processing_steps() -> anyhow::Result<()> {
        let spectrum = demo_spectrum()?;
        let processed = spectrum
            .process()
            .scale(2.0)
            .zero_fill(4, 3)
            .normalize_max_abs()
            .finish()?;

        assert_eq!(processed.shape(), (4, 3));
        assert_vec_close(
            &processed.z,
            &[
                2.0 / 12.0,
                -4.0 / 12.0,
                6.0 / 12.0,
                0.0,
                8.0 / 12.0,
                -10.0 / 12.0,
                1.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
            ],
        );
        assert_eq!(processed.processing.len(), 3);
        assert_eq!(processed.processing[0].operation, "scale_2d");
        assert_eq!(processed.processing[2].operation, "normalize_2d_max_abs");
        Ok(())
    }

    #[test]
    fn borrowed_pipeline_leaves_original_2d_spectrum_unchanged() -> anyhow::Result<()> {
        let spectrum = demo_spectrum()?;
        let processed = (&spectrum).process().scale(3.0).finish()?;

        assert_eq!(spectrum.z, vec![1.0, -2.0, 3.0, 4.0, -5.0, 6.0]);
        assert_eq!(processed.z, vec![3.0, -6.0, 9.0, 12.0, -15.0, 18.0]);
        Ok(())
    }

    #[test]
    fn terminal_projection_includes_prior_processing() -> anyhow::Result<()> {
        let spectrum = demo_spectrum()?;
        let projection = spectrum
            .process()
            .scale(2.0)
            .project_x(ProjectionMode::Sum)?;

        assert_eq!(projection.intensities, vec![10.0, -14.0, 18.0]);
        assert_eq!(projection.processing.len(), 2);
        assert_eq!(projection.processing[0].operation, "scale_2d");
        assert_eq!(projection.processing[1].operation, "project_x");
        Ok(())
    }

    #[test]
    fn terminal_slices_use_processed_data() -> anyhow::Result<()> {
        let spectrum = demo_spectrum()?;
        let row = (&spectrum).process().scale(0.5).slice_x_at_y_index(1)?;
        let column = spectrum.process().scale(0.5).slice_y_at_x_index(1)?;

        assert_eq!(row.intensities, vec![2.0, -2.5, 3.0]);
        assert_eq!(column.intensities, vec![-1.0, -2.5]);
        Ok(())
    }

    #[test]
    fn chains_2d_fft_steps() -> anyhow::Result<()> {
        let spectrum = demo_spectrum()?;
        let processed = spectrum
            .process()
            .fft(FftDirection::Forward)
            .fft(FftDirection::Inverse)
            .finish()?;

        assert_vec_close(&processed.z, &[1.0, -2.0, 3.0, 4.0, -5.0, 6.0]);
        assert!(processed.imaginary.is_some());
        assert_eq!(processed.processing.len(), 2);
        assert_eq!(processed.processing[0].operation, "fft_2d");
        assert_eq!(processed.processing[1].operation, "fft_2d");
        Ok(())
    }

    #[test]
    fn preserves_first_2d_pipeline_error() -> anyhow::Result<()> {
        let spectrum = demo_spectrum()?;
        let error = spectrum
            .process()
            .scale(f64::NAN)
            .zero_fill(4, 3)
            .project_y(ProjectionMode::Sum)
            .expect_err("non-finite scale should fail");

        assert!(matches!(error, RSpinError::NonFinite { .. }));
        Ok(())
    }

    fn demo_spectrum() -> anyhow::Result<Spectrum2D> {
        Ok(Spectrum2D::new(
            Axis::linear("x", Unit::Ppm, 0.0, 2.0, 3)?,
            Axis::linear("y", Unit::Ppm, 10.0, 11.0, 2)?,
            vec![1.0, -2.0, 3.0, 4.0, -5.0, 6.0],
            Metadata::named("2d"),
        )?)
    }

    fn assert_vec_close(actual: &[f64], expected: &[f64]) {
        assert_eq!(actual.len(), expected.len());
        for (left, right) in actual.iter().zip(expected) {
            assert!((left - right).abs() < 1.0e-12, "{left} != {right}");
        }
    }
}
