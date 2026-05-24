//! JSON adapters for one-dimensional WASM processing.

use serde::Deserialize;

use rspin_core::{Axis, Result, Spectrum1D};
use rspin_io::read_processing_recipe_1d_json;
use rspin_processing::{
    BaselineMethod, FftDirection, abs_1d, apply_processing_recipe_1d,
    apply_processing_recipe_1d_until, crop_1d, exponential_apodization, fft_1d,
    gaussian_apodization, magnitude_spectrum, offset_intensity, phase_correct, resample_1d,
    shift_axis, sine_bell_apodization, subtract_baseline, zero_fill,
};

use super::{from_json, to_json};

/// Offsets serialized `Spectrum1D` real intensities.
///
/// # Errors
///
/// Returns an error when deserialization, processing, or serialization fails.
pub fn offset_spectrum_1d_json(spectrum_json: &str, offset: f64) -> Result<String> {
    let spectrum: Spectrum1D = from_json(spectrum_json)?;
    let processed = offset_intensity(&spectrum, offset)?;
    to_json(&processed)
}

/// Shifts the x axis of serialized `Spectrum1D` JSON.
///
/// # Errors
///
/// Returns an error when deserialization, processing, or serialization fails.
pub fn shift_spectrum_1d_axis_json(spectrum_json: &str, delta: f64) -> Result<String> {
    let spectrum: Spectrum1D = from_json(spectrum_json)?;
    let processed = shift_axis(&spectrum, delta)?;
    to_json(&processed)
}

/// Zero-fills serialized `Spectrum1D` JSON to the requested length.
///
/// # Errors
///
/// Returns an error when deserialization, processing, or serialization fails.
pub fn zero_fill_spectrum_1d_json(spectrum_json: &str, target_len: usize) -> Result<String> {
    let spectrum: Spectrum1D = from_json(spectrum_json)?;
    let processed = zero_fill(&spectrum, target_len)?;
    to_json(&processed)
}

/// Crops serialized `Spectrum1D` JSON to an inclusive x-axis window.
///
/// # Errors
///
/// Returns an error when deserialization, processing, or serialization fails.
pub fn crop_spectrum_1d_json(spectrum_json: &str, from: f64, to: f64) -> Result<String> {
    let spectrum: Spectrum1D = from_json(spectrum_json)?;
    let processed = crop_1d(&spectrum, from, to)?;
    to_json(&processed)
}

/// Applies component-wise absolute value to serialized `Spectrum1D` JSON.
///
/// # Errors
///
/// Returns an error when deserialization, processing, or serialization fails.
pub fn abs_spectrum_1d_json(spectrum_json: &str) -> Result<String> {
    let spectrum: Spectrum1D = from_json(spectrum_json)?;
    let processed = abs_1d(&spectrum)?;
    to_json(&processed)
}

/// Resamples serialized `Spectrum1D` JSON onto a serialized target axis.
///
/// # Errors
///
/// Returns an error when deserialization, processing, or serialization fails.
pub fn resample_spectrum_1d_json(
    spectrum_json: &str,
    target_axis_json: &str,
    outside_value: f64,
) -> Result<String> {
    let spectrum: Spectrum1D = from_json(spectrum_json)?;
    let target_axis: Axis = from_json(target_axis_json)?;
    let processed = resample_1d(&spectrum, target_axis, outside_value)?;
    to_json(&processed)
}

/// Applies a one-dimensional FFT to serialized `Spectrum1D` JSON.
///
/// `direction_json` is a JSON string: `"forward"` or `"inverse"`.
///
/// # Errors
///
/// Returns an error when deserialization, processing, or serialization fails.
pub fn fft_spectrum_1d_json(spectrum_json: &str, direction_json: &str) -> Result<String> {
    let spectrum: Spectrum1D = from_json(spectrum_json)?;
    let direction: FftDirectionJson = from_json(direction_json)?;
    let processed = fft_1d(&spectrum, direction.into())?;
    to_json(&processed)
}

/// Applies manual phase correction to serialized `Spectrum1D` JSON.
///
/// # Errors
///
/// Returns an error when deserialization, processing, or serialization fails.
pub fn phase_spectrum_1d_json(spectrum_json: &str, correction_json: &str) -> Result<String> {
    let spectrum: Spectrum1D = from_json(spectrum_json)?;
    let correction: PhaseCorrectionJson = from_json(correction_json)?;
    let processed = phase_correct(
        &spectrum,
        correction.zero_order_deg,
        correction.first_order_deg,
        correction.pivot_fraction,
    )?;
    to_json(&processed)
}

/// Converts serialized `Spectrum1D` JSON to magnitude mode.
///
/// # Errors
///
/// Returns an error when deserialization, processing, or serialization fails.
pub fn magnitude_spectrum_1d_json(spectrum_json: &str) -> Result<String> {
    let spectrum: Spectrum1D = from_json(spectrum_json)?;
    let processed = magnitude_spectrum(&spectrum)?;
    to_json(&processed)
}

/// Applies exponential apodization to serialized `Spectrum1D` JSON.
///
/// # Errors
///
/// Returns an error when deserialization, processing, or serialization fails.
pub fn exponential_apodization_spectrum_1d_json(
    spectrum_json: &str,
    options_json: &str,
) -> Result<String> {
    let spectrum: Spectrum1D = from_json(spectrum_json)?;
    let options: ExponentialApodizationJson = from_json(options_json)?;
    let processed =
        exponential_apodization(&spectrum, options.line_broadening_hz, options.dwell_time_s)?;
    to_json(&processed)
}

/// Applies Gaussian apodization to serialized `Spectrum1D` JSON.
///
/// # Errors
///
/// Returns an error when deserialization, processing, or serialization fails.
pub fn gaussian_apodization_spectrum_1d_json(
    spectrum_json: &str,
    options_json: &str,
) -> Result<String> {
    let spectrum: Spectrum1D = from_json(spectrum_json)?;
    let options: GaussianApodizationJson = from_json(options_json)?;
    let processed = gaussian_apodization(
        &spectrum,
        options.gaussian_broadening_hz,
        options.dwell_time_s,
    )?;
    to_json(&processed)
}

/// Applies sine-bell apodization to serialized `Spectrum1D` JSON.
///
/// # Errors
///
/// Returns an error when deserialization, processing, or serialization fails.
pub fn sine_bell_apodization_spectrum_1d_json(
    spectrum_json: &str,
    options_json: &str,
) -> Result<String> {
    let spectrum: Spectrum1D = from_json(spectrum_json)?;
    let options: SineBellApodizationJson = from_json(options_json)?;
    let processed = sine_bell_apodization(
        &spectrum,
        options.start_angle_deg,
        options.end_angle_deg,
        options.exponent,
    )?;
    to_json(&processed)
}

/// Subtracts a fitted baseline from serialized `Spectrum1D` JSON.
///
/// # Errors
///
/// Returns an error when deserialization, processing, or serialization fails.
pub fn subtract_baseline_spectrum_1d_json(
    spectrum_json: &str,
    method_json: &str,
) -> Result<String> {
    let spectrum: Spectrum1D = from_json(spectrum_json)?;
    let method: BaselineMethodJson = from_json(method_json)?;
    let processed = subtract_baseline(&spectrum, method.into())?;
    to_json(&processed)
}

/// Applies a serialized one-dimensional processing recipe to serialized `Spectrum1D` JSON.
///
/// # Errors
///
/// Returns an error when deserialization, processing, or serialization fails.
pub fn apply_processing_recipe_1d_json(spectrum_json: &str, recipe_json: &str) -> Result<String> {
    let spectrum: Spectrum1D = from_json(spectrum_json)?;
    let recipe = read_processing_recipe_1d_json(recipe_json)?;
    let processed = apply_processing_recipe_1d(&spectrum, &recipe)?;
    to_json(&processed)
}

/// Applies the first operations in a serialized one-dimensional recipe.
///
/// # Errors
///
/// Returns an error when deserialization, processing, or serialization fails.
pub fn apply_processing_recipe_1d_until_json(
    spectrum_json: &str,
    recipe_json: &str,
    operation_count: usize,
) -> Result<String> {
    let spectrum: Spectrum1D = from_json(spectrum_json)?;
    let recipe = read_processing_recipe_1d_json(recipe_json)?;
    let processed = apply_processing_recipe_1d_until(&spectrum, &recipe, operation_count)?;
    to_json(&processed)
}

#[derive(Clone, Copy, Debug, Deserialize)]
enum FftDirectionJson {
    #[serde(rename = "forward", alias = "Forward")]
    Forward,
    #[serde(rename = "inverse", alias = "Inverse")]
    Inverse,
}

impl From<FftDirectionJson> for FftDirection {
    fn from(direction: FftDirectionJson) -> Self {
        match direction {
            FftDirectionJson::Forward => Self::Forward,
            FftDirectionJson::Inverse => Self::Inverse,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(default)]
struct PhaseCorrectionJson {
    zero_order_deg: f64,
    first_order_deg: f64,
    pivot_fraction: f64,
}

impl Default for PhaseCorrectionJson {
    fn default() -> Self {
        Self {
            zero_order_deg: 0.0,
            first_order_deg: 0.0,
            pivot_fraction: 0.5,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize)]
struct ExponentialApodizationJson {
    line_broadening_hz: f64,
    dwell_time_s: f64,
}

#[derive(Clone, Copy, Debug, Deserialize)]
struct GaussianApodizationJson {
    gaussian_broadening_hz: f64,
    dwell_time_s: f64,
}

#[derive(Clone, Copy, Debug, Deserialize)]
struct SineBellApodizationJson {
    start_angle_deg: f64,
    end_angle_deg: f64,
    exponent: f64,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(tag = "method", rename_all = "snake_case")]
enum BaselineMethodJson {
    Constant {
        value: f64,
    },
    MovingMinimum {
        half_window: usize,
    },
    WhittakerAsls {
        lambda: f64,
        p: f64,
        max_iter: usize,
        tolerance: f64,
    },
    #[cfg(feature = "external-baselines")]
    BaselinesAsls {
        lambda: f64,
        p: f64,
        max_iter: usize,
        tolerance: f64,
    },
}

impl From<BaselineMethodJson> for BaselineMethod {
    fn from(method: BaselineMethodJson) -> Self {
        match method {
            BaselineMethodJson::Constant { value } => Self::Constant { value },
            BaselineMethodJson::MovingMinimum { half_window } => {
                Self::MovingMinimum { half_window }
            }
            BaselineMethodJson::WhittakerAsls {
                lambda,
                p,
                max_iter,
                tolerance,
            } => Self::WhittakerAsls {
                lambda,
                p,
                max_iter,
                tolerance,
            },
            #[cfg(feature = "external-baselines")]
            BaselineMethodJson::BaselinesAsls {
                lambda,
                p,
                max_iter,
                tolerance,
            } => Self::BaselinesAsls {
                lambda,
                p,
                max_iter,
                tolerance,
            },
        }
    }
}

#[cfg(test)]
mod tests;
