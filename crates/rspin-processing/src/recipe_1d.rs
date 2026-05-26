//! Serializable one-dimensional processing recipes.

use serde::{Deserialize, Serialize};

use rspin_core::{Axis, RSpinError, Result, Spectrum1D};

use crate::{
    AutoPhaseOptions, BaselineMethod, FftDirection, ProcessingStep, abs_1d, auto_phase_correct,
    convolution_difference_apodization, crop_1d, exponential_apodization, fft_1d,
    gauss_multiply_bruker_apodization, gaussian_apodization, lorentz_to_gauss_apodization,
    magnitude_spectrum, normalize_area, normalize_max_abs, offset_intensity, phase_correct,
    resample_1d, scale_intensity, shift_axis, sine_bell_apodization, subtract_baseline,
    traf_apodization, trapezoidal_apodization, zero_fill,
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
    /// Normalizes real and imaginary intensities by trapezoidal area.
    NormalizeArea {
        /// Desired integrated area after normalization.
        target_area: f64,
        /// Use absolute real intensities when measuring the area.
        use_absolute_intensity: bool,
    },
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
    /// Applies Gaussian apodization to real and imaginary channels.
    GaussianApodization {
        /// Gaussian broadening full width at half maximum in hertz.
        gaussian_broadening_hz: f64,
        /// Dwell time in seconds.
        dwell_time_s: f64,
    },
    /// Applies Lorentz-to-Gauss (resolution-enhancement) apodization.
    LorentzToGaussApodization {
        /// Lorentzian linewidth to undo, in hertz (≥ 0).
        lorentz_to_undo_hz: f64,
        /// Gaussian full-width-at-half-maximum to impose, in hertz (≥ 0).
        gauss_fwhm_hz: f64,
        /// Gaussian-peak shift in `[0, 1]`.
        gauss_shift: f64,
        /// Dwell time in seconds.
        dwell_time_s: f64,
    },
    /// Applies convolution-difference apodization.
    ConvolutionDifferenceApodization {
        /// Narrow line broadening in hertz (≥ 0).
        narrow_line_broadening_hz: f64,
        /// Broad line broadening in hertz (≥ 0).
        broad_line_broadening_hz: f64,
        /// Mixing coefficient in `[0, 1]`.
        mixing: f64,
        /// Dwell time in seconds.
        dwell_time_s: f64,
    },
    /// Applies Bruker-style two-parameter Gaussian (`procs` GMB) apodization.
    GaussMultiplyBrukerApodization {
        /// Signed Bruker `LB` line broadening, in hertz.
        line_broadening_hz: f64,
        /// Bruker `GB` Gaussian peak position as a fraction of the FID, in `[0, 1]`.
        gauss_position_fraction: f64,
        /// Dwell time in seconds.
        dwell_time_s: f64,
    },
    /// Applies TRAF (Traficante) apodization.
    TrafApodization {
        /// Line broadening in hertz (≥ 0).
        line_broadening_hz: f64,
        /// Dwell time in seconds (> 0).
        dwell_time_s: f64,
    },
    /// Applies trapezoidal apodization (ramp-in, plateau, ramp-out).
    TrapezoidalApodization {
        /// Fraction of the FID where the ramp-up reaches 1, in `[0, 1]`.
        rise_end_fraction: f64,
        /// Fraction of the FID where the ramp-down begins, in `[0, 1]`.
        fall_start_fraction: f64,
    },
    /// Applies sine-bell apodization to real and imaginary channels.
    SineBellApodization {
        /// Start angle in degrees.
        start_angle_deg: f64,
        /// End angle in degrees.
        end_angle_deg: f64,
        /// Positive exponent applied to the sine-bell weights.
        exponent: f64,
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
            Self::NormalizeArea {
                target_area,
                use_absolute_intensity,
            } => normalize_area(spectrum, *target_area, *use_absolute_intensity),
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
            Self::GaussianApodization {
                gaussian_broadening_hz,
                dwell_time_s,
            } => gaussian_apodization(spectrum, *gaussian_broadening_hz, *dwell_time_s),
            Self::LorentzToGaussApodization {
                lorentz_to_undo_hz,
                gauss_fwhm_hz,
                gauss_shift,
                dwell_time_s,
            } => lorentz_to_gauss_apodization(
                spectrum,
                *lorentz_to_undo_hz,
                *gauss_fwhm_hz,
                *gauss_shift,
                *dwell_time_s,
            ),
            Self::ConvolutionDifferenceApodization {
                narrow_line_broadening_hz,
                broad_line_broadening_hz,
                mixing,
                dwell_time_s,
            } => convolution_difference_apodization(
                spectrum,
                *narrow_line_broadening_hz,
                *broad_line_broadening_hz,
                *mixing,
                *dwell_time_s,
            ),
            Self::GaussMultiplyBrukerApodization {
                line_broadening_hz,
                gauss_position_fraction,
                dwell_time_s,
            } => gauss_multiply_bruker_apodization(
                spectrum,
                *line_broadening_hz,
                *gauss_position_fraction,
                *dwell_time_s,
            ),
            Self::TrafApodization {
                line_broadening_hz,
                dwell_time_s,
            } => traf_apodization(spectrum, *line_broadening_hz, *dwell_time_s),
            Self::TrapezoidalApodization {
                rise_end_fraction,
                fall_start_fraction,
            } => trapezoidal_apodization(spectrum, *rise_end_fraction, *fall_start_fraction),
            Self::SineBellApodization {
                start_angle_deg,
                end_angle_deg,
                exponent,
            } => sine_bell_apodization(spectrum, *start_angle_deg, *end_angle_deg, *exponent),
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
            "1D processing recipe",
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
    pub fn apply_until(&self, spectrum: &Spectrum1D, operation_count: usize) -> Result<Spectrum1D> {
        apply_processing_recipe_1d_until(spectrum, self, operation_count)
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

    /// Appends a signed-area normalization operation.
    #[must_use]
    pub fn normalize_area(self, target_area: f64) -> Self {
        self.normalize_area_with(target_area, false)
    }

    /// Appends an absolute-area normalization operation.
    #[must_use]
    pub fn normalize_abs_area(self, target_area: f64) -> Self {
        self.normalize_area_with(target_area, true)
    }

    /// Appends an area normalization operation with explicit area mode.
    #[must_use]
    pub fn normalize_area_with(self, target_area: f64, use_absolute_intensity: bool) -> Self {
        self.with_operation(ProcessingOperation1D::NormalizeArea {
            target_area,
            use_absolute_intensity,
        })
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

    /// Appends a Gaussian apodization operation.
    #[must_use]
    pub fn gaussian_apodization(self, gaussian_broadening_hz: f64, dwell_time_s: f64) -> Self {
        self.with_operation(ProcessingOperation1D::GaussianApodization {
            gaussian_broadening_hz,
            dwell_time_s,
        })
    }

    /// Appends a Lorentz-to-Gauss apodization operation.
    #[must_use]
    pub fn lorentz_to_gauss_apodization(
        self,
        lorentz_to_undo_hz: f64,
        gauss_fwhm_hz: f64,
        gauss_shift: f64,
        dwell_time_s: f64,
    ) -> Self {
        self.with_operation(ProcessingOperation1D::LorentzToGaussApodization {
            lorentz_to_undo_hz,
            gauss_fwhm_hz,
            gauss_shift,
            dwell_time_s,
        })
    }

    /// Appends a convolution-difference apodization operation.
    #[must_use]
    pub fn convolution_difference_apodization(
        self,
        narrow_line_broadening_hz: f64,
        broad_line_broadening_hz: f64,
        mixing: f64,
        dwell_time_s: f64,
    ) -> Self {
        self.with_operation(ProcessingOperation1D::ConvolutionDifferenceApodization {
            narrow_line_broadening_hz,
            broad_line_broadening_hz,
            mixing,
            dwell_time_s,
        })
    }

    /// Appends a Bruker-style two-parameter Gaussian (GMB) apodization operation.
    #[must_use]
    pub fn gauss_multiply_bruker_apodization(
        self,
        line_broadening_hz: f64,
        gauss_position_fraction: f64,
        dwell_time_s: f64,
    ) -> Self {
        self.with_operation(ProcessingOperation1D::GaussMultiplyBrukerApodization {
            line_broadening_hz,
            gauss_position_fraction,
            dwell_time_s,
        })
    }

    /// Appends a TRAF apodization operation.
    #[must_use]
    pub fn traf_apodization(self, line_broadening_hz: f64, dwell_time_s: f64) -> Self {
        self.with_operation(ProcessingOperation1D::TrafApodization {
            line_broadening_hz,
            dwell_time_s,
        })
    }

    /// Appends a trapezoidal apodization operation.
    #[must_use]
    pub fn trapezoidal_apodization(self, rise_end_fraction: f64, fall_start_fraction: f64) -> Self {
        self.with_operation(ProcessingOperation1D::TrapezoidalApodization {
            rise_end_fraction,
            fall_start_fraction,
        })
    }

    /// Appends a sine-bell apodization operation.
    #[must_use]
    pub fn sine_bell_apodization(
        self,
        start_angle_deg: f64,
        end_angle_deg: f64,
        exponent: f64,
    ) -> Self {
        self.with_operation(ProcessingOperation1D::SineBellApodization {
            start_angle_deg,
            end_angle_deg,
            exponent,
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
    apply_processing_recipe_1d_until(spectrum, recipe, recipe.operations.len())
}

/// Applies the first `operation_count` operations in a one-dimensional recipe.
///
/// # Errors
///
/// Returns an error when `operation_count` is larger than the recipe length or
/// when a selected operation fails.
pub fn apply_processing_recipe_1d_until(
    spectrum: &Spectrum1D,
    recipe: &ProcessingRecipe1D,
    operation_count: usize,
) -> Result<Spectrum1D> {
    validate_operation_count(
        operation_count,
        recipe.operations.len(),
        "1D processing recipe",
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
