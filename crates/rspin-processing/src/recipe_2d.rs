//! Serializable two-dimensional processing recipes.

use serde::{Deserialize, Serialize};

use rspin_core::{Axis, RSpinError, Result, Spectrum2D};

use crate::{
    AutoPhase2DOptions, FftDirection, PhaseCorrection2D, ProcessingStep, abs_2d,
    auto_phase_correct_2d, crop_2d, exponential_apodization_2d, fft_2d, gaussian_apodization_2d,
    normalize_2d_max_abs, normalize_2d_volume, offset_2d, resample_2d, scale_2d, shift_2d_axes,
    sine_bell_apodization_2d, zero_fill_2d,
};

/// A serializable two-dimensional processing operation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum ProcessingOperation2D {
    /// Multiplies all real and imaginary intensities by `factor`.
    Scale {
        /// Multiplicative factor.
        factor: f64,
    },
    /// Adds an offset to all real intensities.
    Offset {
        /// Additive offset.
        offset: f64,
    },
    /// Normalizes real intensities by their maximum absolute value.
    NormalizeMaxAbs,
    /// Normalizes real and imaginary intensities by bilinear volume.
    NormalizeVolume {
        /// Desired integrated volume after normalization.
        target_volume: f64,
        /// Use absolute real intensities when measuring the volume.
        use_absolute_intensity: bool,
    },
    /// Shifts x and y axes by constant deltas.
    ShiftAxes {
        /// Shift amount in the x-axis unit.
        x_delta: f64,
        /// Shift amount in the y-axis unit.
        y_delta: f64,
    },
    /// Applies component-wise absolute value to real and imaginary matrices.
    AbsoluteValue,
    /// Appends zeroes until the spectrum reaches the requested shape.
    ZeroFill {
        /// Desired x-axis point count.
        target_x_len: usize,
        /// Desired y-axis point count.
        target_y_len: usize,
    },
    /// Keeps points inside inclusive x and y axis windows.
    Crop {
        /// First requested x coordinate.
        x_from: f64,
        /// Second requested x coordinate.
        x_to: f64,
        /// First requested y coordinate.
        y_from: f64,
        /// Second requested y coordinate.
        y_to: f64,
    },
    /// Bilinearly resamples real and imaginary matrices onto target axes.
    Resample {
        /// Target x axis.
        target_x: Axis,
        /// Target y axis.
        target_y: Axis,
        /// Value used outside the source axis domain.
        outside_value: f64,
    },
    /// Applies separable exponential apodization.
    ExponentialApodization {
        /// X-dimension line broadening in hertz.
        x_line_broadening_hz: f64,
        /// Y-dimension line broadening in hertz.
        y_line_broadening_hz: f64,
        /// X-dimension dwell time in seconds.
        x_dwell_time_s: f64,
        /// Y-dimension dwell time in seconds.
        y_dwell_time_s: f64,
    },
    /// Applies separable Gaussian apodization.
    GaussianApodization {
        /// X-dimension Gaussian broadening full width at half maximum in hertz.
        x_gaussian_broadening_hz: f64,
        /// Y-dimension Gaussian broadening full width at half maximum in hertz.
        y_gaussian_broadening_hz: f64,
        /// X-dimension dwell time in seconds.
        x_dwell_time_s: f64,
        /// Y-dimension dwell time in seconds.
        y_dwell_time_s: f64,
    },
    /// Applies separable sine-bell apodization.
    SineBellApodization {
        /// X-dimension start angle in degrees.
        x_start_angle_deg: f64,
        /// X-dimension end angle in degrees.
        x_end_angle_deg: f64,
        /// X-dimension positive exponent.
        x_exponent: f64,
        /// Y-dimension start angle in degrees.
        y_start_angle_deg: f64,
        /// Y-dimension end angle in degrees.
        y_end_angle_deg: f64,
        /// Y-dimension positive exponent.
        y_exponent: f64,
    },
    /// Applies a forward or inverse 2D FFT.
    Fft {
        /// Transform direction.
        direction: FftDirection,
    },
    /// Applies manual separable x/y phase correction.
    Phase {
        /// Phase correction parameters.
        correction: PhaseCorrection2D,
    },
    /// Applies automatic 2D phase correction.
    AutoPhase {
        /// Search options.
        options: AutoPhase2DOptions,
    },
}

impl ProcessingStep<Spectrum2D> for ProcessingOperation2D {
    fn apply(&self, spectrum: &Spectrum2D) -> Result<Spectrum2D> {
        match self {
            Self::Scale { factor } => scale_2d(spectrum, *factor),
            Self::Offset { offset } => offset_2d(spectrum, *offset),
            Self::NormalizeMaxAbs => normalize_2d_max_abs(spectrum),
            Self::NormalizeVolume {
                target_volume,
                use_absolute_intensity,
            } => normalize_2d_volume(spectrum, *target_volume, *use_absolute_intensity),
            Self::ShiftAxes { x_delta, y_delta } => shift_2d_axes(spectrum, *x_delta, *y_delta),
            Self::AbsoluteValue => abs_2d(spectrum),
            Self::ZeroFill {
                target_x_len,
                target_y_len,
            } => zero_fill_2d(spectrum, *target_x_len, *target_y_len),
            Self::Crop {
                x_from,
                x_to,
                y_from,
                y_to,
            } => crop_2d(spectrum, *x_from, *x_to, *y_from, *y_to),
            Self::Resample {
                target_x,
                target_y,
                outside_value,
            } => resample_2d(spectrum, target_x.clone(), target_y.clone(), *outside_value),
            Self::ExponentialApodization {
                x_line_broadening_hz,
                y_line_broadening_hz,
                x_dwell_time_s,
                y_dwell_time_s,
            } => exponential_apodization_2d(
                spectrum,
                *x_line_broadening_hz,
                *y_line_broadening_hz,
                *x_dwell_time_s,
                *y_dwell_time_s,
            ),
            Self::GaussianApodization {
                x_gaussian_broadening_hz,
                y_gaussian_broadening_hz,
                x_dwell_time_s,
                y_dwell_time_s,
            } => gaussian_apodization_2d(
                spectrum,
                *x_gaussian_broadening_hz,
                *y_gaussian_broadening_hz,
                *x_dwell_time_s,
                *y_dwell_time_s,
            ),
            Self::SineBellApodization {
                x_start_angle_deg,
                x_end_angle_deg,
                x_exponent,
                y_start_angle_deg,
                y_end_angle_deg,
                y_exponent,
            } => sine_bell_apodization_2d(
                spectrum,
                *x_start_angle_deg,
                *x_end_angle_deg,
                *x_exponent,
                *y_start_angle_deg,
                *y_end_angle_deg,
                *y_exponent,
            ),
            Self::Fft { direction } => fft_2d(spectrum, *direction),
            Self::Phase { correction } => crate::phase_correct_2d(spectrum, *correction),
            Self::AutoPhase { options } => {
                auto_phase_correct_2d(spectrum, *options).map(|result| result.spectrum)
            }
        }
    }
}

/// A serializable two-dimensional processing recipe.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ProcessingRecipe2D {
    /// Operations applied in order.
    pub operations: Vec<ProcessingOperation2D>,
}

impl ProcessingRecipe2D {
    /// Creates an empty processing recipe.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a recipe with one operation appended.
    #[must_use]
    pub fn with_operation(mut self, operation: ProcessingOperation2D) -> Self {
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
    pub fn apply(&self, spectrum: &Spectrum2D) -> Result<Spectrum2D> {
        apply_processing_recipe_2d(spectrum, self)
    }

    /// Returns a recipe containing the first `operation_count` operations.
    ///
    /// This is useful for rollback/reapply workflows that rebuild a processed
    /// spectrum from the original input.
    ///
    /// # Errors
    ///
    /// Returns an error when `operation_count` is larger than the recipe length.
    pub fn prefix(&self, operation_count: usize) -> Result<Self> {
        validate_operation_count(
            operation_count,
            self.operations.len(),
            "2D processing recipe",
        )?;
        Ok(Self {
            operations: self.operations[..operation_count].to_vec(),
        })
    }

    /// Returns a recipe with the final operation removed.
    #[must_use]
    pub fn without_last(&self) -> Self {
        let operation_count = self.operations.len().saturating_sub(1);
        Self {
            operations: self.operations[..operation_count].to_vec(),
        }
    }

    /// Applies the first `operation_count` operations to a spectrum.
    ///
    /// # Errors
    ///
    /// Returns an error when `operation_count` is too large or when a selected
    /// operation fails.
    pub fn apply_until(&self, spectrum: &Spectrum2D, operation_count: usize) -> Result<Spectrum2D> {
        apply_processing_recipe_2d_until(spectrum, self, operation_count)
    }

    /// Appends a scale operation.
    #[must_use]
    pub fn scale(self, factor: f64) -> Self {
        self.with_operation(ProcessingOperation2D::Scale { factor })
    }

    /// Appends a real-intensity offset operation.
    #[must_use]
    pub fn offset(self, offset: f64) -> Self {
        self.with_operation(ProcessingOperation2D::Offset { offset })
    }

    /// Appends a maximum-absolute normalization operation.
    #[must_use]
    pub fn normalize_max_abs(self) -> Self {
        self.with_operation(ProcessingOperation2D::NormalizeMaxAbs)
    }

    /// Appends a signed-volume normalization operation.
    #[must_use]
    pub fn normalize_volume(self, target_volume: f64) -> Self {
        self.normalize_volume_with(target_volume, false)
    }

    /// Appends an absolute-volume normalization operation.
    #[must_use]
    pub fn normalize_abs_volume(self, target_volume: f64) -> Self {
        self.normalize_volume_with(target_volume, true)
    }

    /// Appends a volume normalization operation with explicit volume mode.
    #[must_use]
    pub fn normalize_volume_with(self, target_volume: f64, use_absolute_intensity: bool) -> Self {
        self.with_operation(ProcessingOperation2D::NormalizeVolume {
            target_volume,
            use_absolute_intensity,
        })
    }

    /// Appends an x/y axis-shift operation.
    #[must_use]
    pub fn shift_axes(self, x_delta: f64, y_delta: f64) -> Self {
        self.with_operation(ProcessingOperation2D::ShiftAxes { x_delta, y_delta })
    }

    /// Appends an x-axis-only shift operation.
    #[must_use]
    pub fn shift_x_axis(self, delta: f64) -> Self {
        self.shift_axes(delta, 0.0)
    }

    /// Appends a y-axis-only shift operation.
    #[must_use]
    pub fn shift_y_axis(self, delta: f64) -> Self {
        self.shift_axes(0.0, delta)
    }

    /// Appends a component-wise absolute-value operation.
    #[must_use]
    pub fn absolute_value(self) -> Self {
        self.with_operation(ProcessingOperation2D::AbsoluteValue)
    }

    /// Appends a zero-fill operation.
    #[must_use]
    pub fn zero_fill(self, x_points: usize, y_points: usize) -> Self {
        self.with_operation(ProcessingOperation2D::ZeroFill {
            target_x_len: x_points,
            target_y_len: y_points,
        })
    }

    /// Appends a crop operation.
    #[must_use]
    pub fn crop(self, x_from: f64, x_to: f64, y_from: f64, y_to: f64) -> Self {
        self.with_operation(ProcessingOperation2D::Crop {
            x_from,
            x_to,
            y_from,
            y_to,
        })
    }

    /// Appends a resampling operation.
    #[must_use]
    pub fn resample(self, target_x: Axis, target_y: Axis) -> Self {
        self.resample_with_outside(target_x, target_y, 0.0)
    }

    /// Appends a resampling operation with an explicit outside value.
    #[must_use]
    pub fn resample_with_outside(self, target_x: Axis, target_y: Axis, outside_value: f64) -> Self {
        self.with_operation(ProcessingOperation2D::Resample {
            target_x,
            target_y,
            outside_value,
        })
    }

    /// Appends a separable exponential apodization operation.
    #[must_use]
    pub fn exponential_apodization(
        self,
        x_line_broadening_hz: f64,
        y_line_broadening_hz: f64,
        x_dwell_time_s: f64,
        y_dwell_time_s: f64,
    ) -> Self {
        self.with_operation(ProcessingOperation2D::ExponentialApodization {
            x_line_broadening_hz,
            y_line_broadening_hz,
            x_dwell_time_s,
            y_dwell_time_s,
        })
    }

    /// Appends a separable Gaussian apodization operation.
    #[must_use]
    pub fn gaussian_apodization(
        self,
        x_gaussian_broadening_hz: f64,
        y_gaussian_broadening_hz: f64,
        x_dwell_time_s: f64,
        y_dwell_time_s: f64,
    ) -> Self {
        self.with_operation(ProcessingOperation2D::GaussianApodization {
            x_gaussian_broadening_hz,
            y_gaussian_broadening_hz,
            x_dwell_time_s,
            y_dwell_time_s,
        })
    }

    /// Appends a separable sine-bell apodization operation.
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
        self.with_operation(ProcessingOperation2D::SineBellApodization {
            x_start_angle_deg,
            x_end_angle_deg,
            x_exponent,
            y_start_angle_deg,
            y_end_angle_deg,
            y_exponent,
        })
    }

    /// Appends a 2D FFT operation.
    #[must_use]
    pub fn fft(self, direction: FftDirection) -> Self {
        self.with_operation(ProcessingOperation2D::Fft { direction })
    }

    /// Appends a manual phase-correction operation.
    #[must_use]
    pub fn phase(self, correction: PhaseCorrection2D) -> Self {
        self.with_operation(ProcessingOperation2D::Phase { correction })
    }

    /// Appends a manual x-dimension phase-correction operation.
    #[must_use]
    pub fn phase_x(self, zero_order_deg: f64, first_order_deg: f64, pivot_fraction: f64) -> Self {
        self.phase(PhaseCorrection2D::new().x_phase(
            zero_order_deg,
            first_order_deg,
            pivot_fraction,
        ))
    }

    /// Appends a manual y-dimension phase-correction operation.
    #[must_use]
    pub fn phase_y(self, zero_order_deg: f64, first_order_deg: f64, pivot_fraction: f64) -> Self {
        self.phase(PhaseCorrection2D::new().y_phase(
            zero_order_deg,
            first_order_deg,
            pivot_fraction,
        ))
    }

    /// Appends automatic 2D phase correction with default options.
    #[must_use]
    pub fn auto_phase(self) -> Self {
        self.auto_phase_with(AutoPhase2DOptions::default())
    }

    /// Appends automatic 2D phase correction with explicit options.
    #[must_use]
    pub fn auto_phase_with(self, options: AutoPhase2DOptions) -> Self {
        self.with_operation(ProcessingOperation2D::AutoPhase { options })
    }
}

impl ProcessingStep<Spectrum2D> for ProcessingRecipe2D {
    fn apply(&self, spectrum: &Spectrum2D) -> Result<Spectrum2D> {
        apply_processing_recipe_2d(spectrum, self)
    }
}

/// Applies a two-dimensional processing recipe to a spectrum.
///
/// # Errors
///
/// Returns the first processing error produced by an operation.
pub fn apply_processing_recipe_2d(
    spectrum: &Spectrum2D,
    recipe: &ProcessingRecipe2D,
) -> Result<Spectrum2D> {
    apply_processing_recipe_2d_until(spectrum, recipe, recipe.operations.len())
}

/// Applies the first `operation_count` operations in a two-dimensional recipe.
///
/// # Errors
///
/// Returns an error when `operation_count` is larger than the recipe length or
/// when a selected operation fails.
pub fn apply_processing_recipe_2d_until(
    spectrum: &Spectrum2D,
    recipe: &ProcessingRecipe2D,
    operation_count: usize,
) -> Result<Spectrum2D> {
    validate_operation_count(
        operation_count,
        recipe.operations.len(),
        "2D processing recipe",
    )?;
    let mut processed = spectrum.clone();
    for operation in recipe.operations.iter().take(operation_count) {
        processed = operation.apply(&processed)?;
    }
    Ok(processed)
}

#[cfg(test)]
mod tests;

fn validate_operation_count(
    operation_count: usize,
    available: usize,
    context: &'static str,
) -> Result<()> {
    if operation_count > available {
        return Err(RSpinError::InvalidSpectrum {
            message: format!(
                "{context} has {available} operations but {operation_count} were requested"
            ),
        });
    }
    Ok(())
}
