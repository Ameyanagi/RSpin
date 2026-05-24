//! Serializable one-dimensional processing recipes.

use serde::{Deserialize, Serialize};

use rspin_core::{Axis, Result, Spectrum1D};

use crate::{
    AutoPhaseOptions, BaselineMethod, FftDirection, ProcessingStep, abs_1d, auto_phase_correct,
    crop_1d, exponential_apodization, fft_1d, magnitude_spectrum, normalize_max_abs,
    offset_intensity, phase_correct, resample_1d, scale_intensity, shift_axis, subtract_baseline,
    zero_fill,
};

/// A serializable one-dimensional processing operation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum ProcessingOperation1D {
    /// Multiplies all real and imaginary intensities by `factor`.
    Scale {
        /// Multiplicative factor.
        factor: f64,
    },
    /// Adds `offset` to all real intensities.
    Offset {
        /// Additive offset.
        offset: f64,
    },
    /// Normalizes real intensities by their maximum absolute value.
    NormalizeMaxAbs,
    /// Applies component-wise absolute value to real and imaginary channels.
    AbsoluteValue,
    /// Shifts the x-axis values by `delta`.
    ShiftAxis {
        /// Shift amount in the x-axis unit.
        delta: f64,
    },
    /// Appends zeroes until the spectrum reaches `target_len` points.
    ZeroFill {
        /// Desired total point count.
        target_len: usize,
    },
    /// Keeps points inside an inclusive x-axis window.
    Crop {
        /// First requested x coordinate.
        from: f64,
        /// Second requested x coordinate.
        to: f64,
    },
    /// Linearly resamples real and imaginary channels onto `target_axis`.
    Resample {
        /// Target x axis.
        target_axis: Axis,
        /// Value used outside the source axis domain.
        outside_value: f64,
    },
    /// Applies exponential apodization to real and imaginary channels.
    ExponentialApodization {
        /// Line broadening in hertz.
        line_broadening_hz: f64,
        /// Dwell time in seconds.
        dwell_time_s: f64,
    },
    /// Applies a forward or inverse FFT.
    Fft {
        /// Transform direction.
        direction: FftDirection,
    },
    /// Converts the spectrum to magnitude mode.
    Magnitude,
    /// Applies manual zero- and first-order phase correction.
    Phase {
        /// Zero-order phase in degrees.
        zero_order_deg: f64,
        /// First-order phase in degrees across the full spectrum.
        first_order_deg: f64,
        /// Pivot position as a fraction of the index range.
        pivot_fraction: f64,
    },
    /// Applies automatic phase correction.
    AutoPhase {
        /// Search options.
        options: AutoPhaseOptions,
    },
    /// Subtracts a fitted baseline.
    SubtractBaseline {
        /// Baseline-correction algorithm.
        method: BaselineMethod,
    },
}

impl ProcessingStep<Spectrum1D> for ProcessingOperation1D {
    fn apply(&self, spectrum: &Spectrum1D) -> Result<Spectrum1D> {
        match self {
            Self::Scale { factor } => scale_intensity(spectrum, *factor),
            Self::Offset { offset } => offset_intensity(spectrum, *offset),
            Self::NormalizeMaxAbs => normalize_max_abs(spectrum),
            Self::AbsoluteValue => abs_1d(spectrum),
            Self::ShiftAxis { delta } => shift_axis(spectrum, *delta),
            Self::ZeroFill { target_len } => zero_fill(spectrum, *target_len),
            Self::Crop { from, to } => crop_1d(spectrum, *from, *to),
            Self::Resample {
                target_axis,
                outside_value,
            } => resample_1d(spectrum, target_axis.clone(), *outside_value),
            Self::ExponentialApodization {
                line_broadening_hz,
                dwell_time_s,
            } => exponential_apodization(spectrum, *line_broadening_hz, *dwell_time_s),
            Self::Fft { direction } => fft_1d(spectrum, *direction),
            Self::Magnitude => magnitude_spectrum(spectrum),
            Self::Phase {
                zero_order_deg,
                first_order_deg,
                pivot_fraction,
            } => phase_correct(spectrum, *zero_order_deg, *first_order_deg, *pivot_fraction),
            Self::AutoPhase { options } => {
                auto_phase_correct(spectrum, *options).map(|result| result.spectrum)
            }
            Self::SubtractBaseline { method } => subtract_baseline(spectrum, *method),
        }
    }
}

/// A serializable one-dimensional processing recipe.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ProcessingRecipe1D {
    /// Operations applied in order.
    pub operations: Vec<ProcessingOperation1D>,
}

impl ProcessingRecipe1D {
    /// Creates an empty processing recipe.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a recipe with one operation appended.
    #[must_use]
    pub fn with_operation(mut self, operation: ProcessingOperation1D) -> Self {
        self.operations.push(operation);
        self
    }

    /// Returns the number of operations in the recipe.
    #[must_use]
    pub fn len(&self) -> usize {
        self.operations.len()
    }

    /// Returns true when the recipe has no operations.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    /// Applies the recipe to a spectrum.
    ///
    /// # Errors
    ///
    /// Returns the first processing error produced by an operation.
    pub fn apply(&self, spectrum: &Spectrum1D) -> Result<Spectrum1D> {
        apply_processing_recipe_1d(spectrum, self)
    }

    /// Appends a scale operation.
    #[must_use]
    pub fn scale(self, factor: f64) -> Self {
        self.with_operation(ProcessingOperation1D::Scale { factor })
    }

    /// Appends an offset operation.
    #[must_use]
    pub fn offset(self, offset: f64) -> Self {
        self.with_operation(ProcessingOperation1D::Offset { offset })
    }

    /// Appends a maximum-absolute normalization operation.
    #[must_use]
    pub fn normalize_max_abs(self) -> Self {
        self.with_operation(ProcessingOperation1D::NormalizeMaxAbs)
    }

    /// Appends a component-wise absolute-value operation.
    #[must_use]
    pub fn absolute_value(self) -> Self {
        self.with_operation(ProcessingOperation1D::AbsoluteValue)
    }

    /// Appends an axis-shift operation.
    #[must_use]
    pub fn shift_axis(self, delta: f64) -> Self {
        self.with_operation(ProcessingOperation1D::ShiftAxis { delta })
    }

    /// Appends a zero-fill operation.
    #[must_use]
    pub fn zero_fill(self, target_len: usize) -> Self {
        self.with_operation(ProcessingOperation1D::ZeroFill { target_len })
    }

    /// Appends a crop operation.
    #[must_use]
    pub fn crop(self, from: f64, to: f64) -> Self {
        self.with_operation(ProcessingOperation1D::Crop { from, to })
    }

    /// Appends a resampling operation.
    #[must_use]
    pub fn resample(self, target_axis: Axis) -> Self {
        self.resample_with_outside(target_axis, 0.0)
    }

    /// Appends a resampling operation with an explicit outside value.
    #[must_use]
    pub fn resample_with_outside(self, target_axis: Axis, outside_value: f64) -> Self {
        self.with_operation(ProcessingOperation1D::Resample {
            target_axis,
            outside_value,
        })
    }

    /// Appends an exponential apodization operation.
    #[must_use]
    pub fn exponential_apodization(self, line_broadening_hz: f64, dwell_time_s: f64) -> Self {
        self.with_operation(ProcessingOperation1D::ExponentialApodization {
            line_broadening_hz,
            dwell_time_s,
        })
    }

    /// Appends an FFT operation.
    #[must_use]
    pub fn fft(self, direction: FftDirection) -> Self {
        self.with_operation(ProcessingOperation1D::Fft { direction })
    }

    /// Appends a magnitude operation.
    #[must_use]
    pub fn magnitude(self) -> Self {
        self.with_operation(ProcessingOperation1D::Magnitude)
    }

    /// Appends a manual phase-correction operation.
    #[must_use]
    pub fn phase(self, zero_order_deg: f64, first_order_deg: f64, pivot_fraction: f64) -> Self {
        self.with_operation(ProcessingOperation1D::Phase {
            zero_order_deg,
            first_order_deg,
            pivot_fraction,
        })
    }

    /// Appends automatic phase correction with default options.
    #[must_use]
    pub fn auto_phase(self) -> Self {
        self.auto_phase_with(AutoPhaseOptions::default())
    }

    /// Appends automatic phase correction with explicit options.
    #[must_use]
    pub fn auto_phase_with(self, options: AutoPhaseOptions) -> Self {
        self.with_operation(ProcessingOperation1D::AutoPhase { options })
    }

    /// Appends baseline subtraction with the default method.
    #[must_use]
    pub fn subtract_baseline(self) -> Self {
        self.subtract_baseline_with(BaselineMethod::default())
    }

    /// Appends baseline subtraction with an explicit method.
    #[must_use]
    pub fn subtract_baseline_with(self, method: BaselineMethod) -> Self {
        self.with_operation(ProcessingOperation1D::SubtractBaseline { method })
    }
}

impl ProcessingStep<Spectrum1D> for ProcessingRecipe1D {
    fn apply(&self, spectrum: &Spectrum1D) -> Result<Spectrum1D> {
        apply_processing_recipe_1d(spectrum, self)
    }
}

/// Applies a one-dimensional processing recipe to a spectrum.
///
/// # Errors
///
/// Returns the first processing error produced by an operation.
pub fn apply_processing_recipe_1d(
    spectrum: &Spectrum1D,
    recipe: &ProcessingRecipe1D,
) -> Result<Spectrum1D> {
    let mut processed = spectrum.clone();
    for operation in &recipe.operations {
        processed = operation.apply(&processed)?;
    }
    Ok(processed)
}

#[cfg(test)]
mod tests {
    use rspin_core::{Axis, Metadata, RSpinError, Unit};

    use super::*;

    #[test]
    fn applies_chainable_recipe_operations() -> anyhow::Result<()> {
        let spectrum = demo_spectrum()?;
        let recipe = ProcessingRecipe1D::new()
            .scale(2.0)
            .offset(-2.0)
            .absolute_value()
            .crop(0.0, 1.0)
            .resample(Axis::linear("shift", Unit::Ppm, 0.0, 1.0, 3)?)
            .zero_fill(5)
            .normalize_max_abs();

        let processed = recipe.apply(&spectrum)?;

        assert_eq!(recipe.len(), 7);
        assert_eq!(processed.len(), 5);
        assert_eq!(processed.intensities, vec![0.0, 0.5, 1.0, 0.0, 0.0]);
        assert_eq!(processed.processing.len(), 7);
        assert_eq!(processed.processing[0].operation, "scale_intensity");
        assert_eq!(processed.processing[6].operation, "normalize_max_abs");
        Ok(())
    }

    #[test]
    fn round_trips_recipe_json_and_applies_step_trait() -> anyhow::Result<()> {
        let recipe = ProcessingRecipe1D::new()
            .scale(2.0)
            .subtract_baseline_with(BaselineMethod::Constant { value: 1.0 });
        let json = serde_json::to_string(&recipe)?;
        let decoded: ProcessingRecipe1D = serde_json::from_str(&json)?;
        let processed = decoded.apply(&demo_spectrum()?)?;

        assert_eq!(decoded.len(), 2);
        assert_eq!(processed.intensities, vec![1.0, -5.0, 7.0]);
        assert_eq!(processed.processing[1].operation, "baseline_constant");
        Ok(())
    }

    #[test]
    fn preserves_first_recipe_error() -> anyhow::Result<()> {
        let recipe = ProcessingRecipe1D::new().scale(f64::NAN).offset(10.0);
        let error = recipe
            .apply(&demo_spectrum()?)
            .expect_err("non-finite scale should fail");

        assert!(matches!(error, RSpinError::NonFinite { .. }));
        Ok(())
    }

    fn demo_spectrum() -> anyhow::Result<Spectrum1D> {
        Ok(Spectrum1D::new(
            Axis::linear("shift", Unit::Ppm, 0.0, 2.0, 3)?,
            vec![1.0, -2.0, 4.0],
            Metadata::default(),
        )?)
    }
}
