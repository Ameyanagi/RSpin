//! JSON adapters for two-dimensional WASM processing.

use serde::{Deserialize, Serialize};

use rspin_core::{Axis, Result, Spectrum2D};
use rspin_processing::{
    AutoPhase2DOptions, FftDirection, PhaseCorrection2D, ProjectionMode, auto_phase_correct_2d,
    crop_2d, fft_2d, normalize_2d_max_abs, phase_correct_2d, project_x, project_y, resample_2d,
    scale_2d, slice_x_at_y, slice_x_at_y_index, slice_y_at_x, slice_y_at_x_index, zero_fill_2d,
};

use super::{from_json, to_json};

/// Scales serialized `Spectrum2D` JSON.
///
/// # Errors
///
/// Returns an error when deserialization, processing, or serialization fails.
pub fn scale_spectrum_2d_json(spectrum_json: &str, factor: f64) -> Result<String> {
    let spectrum: Spectrum2D = from_json(spectrum_json)?;
    let processed = scale_2d(&spectrum, factor)?;
    to_json(&processed)
}

/// Normalizes serialized `Spectrum2D` JSON by maximum absolute intensity.
///
/// # Errors
///
/// Returns an error when deserialization, processing, or serialization fails.
pub fn normalize_spectrum_2d_json(spectrum_json: &str) -> Result<String> {
    let spectrum: Spectrum2D = from_json(spectrum_json)?;
    let processed = normalize_2d_max_abs(&spectrum)?;
    to_json(&processed)
}

/// Zero-fills serialized `Spectrum2D` JSON to the requested shape.
///
/// # Errors
///
/// Returns an error when deserialization, processing, or serialization fails.
pub fn zero_fill_spectrum_2d_json(
    spectrum_json: &str,
    target_width: usize,
    target_height: usize,
) -> Result<String> {
    let spectrum: Spectrum2D = from_json(spectrum_json)?;
    let processed = zero_fill_2d(&spectrum, target_width, target_height)?;
    to_json(&processed)
}

/// Crops serialized `Spectrum2D` JSON to inclusive x and y windows.
///
/// # Errors
///
/// Returns an error when deserialization, processing, or serialization fails.
pub fn crop_spectrum_2d_json(
    spectrum_json: &str,
    x_from: f64,
    x_to: f64,
    y_from: f64,
    y_to: f64,
) -> Result<String> {
    let spectrum: Spectrum2D = from_json(spectrum_json)?;
    let processed = crop_2d(&spectrum, x_from, x_to, y_from, y_to)?;
    to_json(&processed)
}

/// Resamples serialized `Spectrum2D` JSON onto serialized target axes.
///
/// # Errors
///
/// Returns an error when deserialization, processing, or serialization fails.
pub fn resample_spectrum_2d_json(
    spectrum_json: &str,
    target_columns_json: &str,
    target_rows_json: &str,
    outside_value: f64,
) -> Result<String> {
    let spectrum: Spectrum2D = from_json(spectrum_json)?;
    let target_x: Axis = from_json(target_columns_json)?;
    let target_y: Axis = from_json(target_rows_json)?;
    let processed = resample_2d(&spectrum, target_x, target_y, outside_value)?;
    to_json(&processed)
}

/// Applies a two-dimensional FFT to serialized `Spectrum2D` JSON.
///
/// `direction_json` is a JSON string: `"forward"` or `"inverse"`.
///
/// # Errors
///
/// Returns an error when deserialization, processing, or serialization fails.
pub fn fft_spectrum_2d_json(spectrum_json: &str, direction_json: &str) -> Result<String> {
    let spectrum: Spectrum2D = from_json(spectrum_json)?;
    let direction: FftDirectionJson = from_json(direction_json)?;
    let processed = fft_2d(&spectrum, direction.into())?;
    to_json(&processed)
}

/// Applies manual x/y phase correction to serialized `Spectrum2D` JSON.
///
/// # Errors
///
/// Returns an error when deserialization, processing, or serialization fails.
pub fn phase_spectrum_2d_json(spectrum_json: &str, correction_json: &str) -> Result<String> {
    let spectrum: Spectrum2D = from_json(spectrum_json)?;
    let correction: PhaseCorrection2DJson = from_json(correction_json)?;
    let processed = phase_correct_2d(&spectrum, correction.into())?;
    to_json(&processed)
}

/// Automatically phases serialized `Spectrum2D` JSON.
///
/// # Errors
///
/// Returns an error when deserialization, processing, or serialization fails.
pub fn auto_phase_spectrum_2d_json(spectrum_json: &str, options_json: &str) -> Result<String> {
    let spectrum: Spectrum2D = from_json(spectrum_json)?;
    let options: AutoPhase2DOptionsJson = from_json(options_json)?;
    let result = auto_phase_correct_2d(&spectrum, options.into())?;
    to_json(&AutoPhase2DResponseJson {
        spectrum: result.spectrum,
        correction: result.correction.into(),
        score: result.score,
    })
}

/// Projects serialized `Spectrum2D` JSON onto the x axis.
///
/// `mode_json` is a JSON string: `"sum"`, `"mean"`, `"max"`, `"min"`, or `"max_abs"`.
///
/// # Errors
///
/// Returns an error when deserialization, processing, or serialization fails.
pub fn project_spectrum_2d_x_json(spectrum_json: &str, mode_json: &str) -> Result<String> {
    let spectrum: Spectrum2D = from_json(spectrum_json)?;
    let mode: ProjectionModeJson = from_json(mode_json)?;
    let projection = project_x(&spectrum, mode.into())?;
    to_json(&projection)
}

/// Projects serialized `Spectrum2D` JSON onto the y axis.
///
/// `mode_json` is a JSON string: `"sum"`, `"mean"`, `"max"`, `"min"`, or `"max_abs"`.
///
/// # Errors
///
/// Returns an error when deserialization, processing, or serialization fails.
pub fn project_spectrum_2d_y_json(spectrum_json: &str, mode_json: &str) -> Result<String> {
    let spectrum: Spectrum2D = from_json(spectrum_json)?;
    let mode: ProjectionModeJson = from_json(mode_json)?;
    let projection = project_y(&spectrum, mode.into())?;
    to_json(&projection)
}

/// Extracts an x-axis row from serialized `Spectrum2D` JSON.
///
/// # Errors
///
/// Returns an error when deserialization, processing, or serialization fails.
pub fn slice_spectrum_2d_x_at_y_index_json(spectrum_json: &str, y_index: usize) -> Result<String> {
    let spectrum: Spectrum2D = from_json(spectrum_json)?;
    let slice = slice_x_at_y_index(&spectrum, y_index)?;
    to_json(&slice)
}

/// Extracts the x-axis row nearest `y` from serialized `Spectrum2D` JSON.
///
/// # Errors
///
/// Returns an error when deserialization, processing, or serialization fails.
pub fn slice_spectrum_2d_x_at_y_json(spectrum_json: &str, y: f64) -> Result<String> {
    let spectrum: Spectrum2D = from_json(spectrum_json)?;
    let slice = slice_x_at_y(&spectrum, y)?;
    to_json(&slice)
}

/// Extracts a y-axis column from serialized `Spectrum2D` JSON.
///
/// # Errors
///
/// Returns an error when deserialization, processing, or serialization fails.
pub fn slice_spectrum_2d_y_at_x_index_json(spectrum_json: &str, x_index: usize) -> Result<String> {
    let spectrum: Spectrum2D = from_json(spectrum_json)?;
    let slice = slice_y_at_x_index(&spectrum, x_index)?;
    to_json(&slice)
}

/// Extracts the y-axis column nearest `x` from serialized `Spectrum2D` JSON.
///
/// # Errors
///
/// Returns an error when deserialization, processing, or serialization fails.
pub fn slice_spectrum_2d_y_at_x_json(spectrum_json: &str, x: f64) -> Result<String> {
    let spectrum: Spectrum2D = from_json(spectrum_json)?;
    let slice = slice_y_at_x(&spectrum, x)?;
    to_json(&slice)
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
enum ProjectionModeJson {
    #[serde(rename = "sum", alias = "Sum")]
    Sum,
    #[serde(rename = "mean", alias = "Mean")]
    Mean,
    #[serde(rename = "max", alias = "Max")]
    Max,
    #[serde(rename = "min", alias = "Min")]
    Min,
    #[serde(rename = "max_abs", alias = "MaxAbs")]
    MaxAbs,
}

impl From<ProjectionModeJson> for ProjectionMode {
    fn from(mode: ProjectionModeJson) -> Self {
        match mode {
            ProjectionModeJson::Sum => Self::Sum,
            ProjectionModeJson::Mean => Self::Mean,
            ProjectionModeJson::Max => Self::Max,
            ProjectionModeJson::Min => Self::Min,
            ProjectionModeJson::MaxAbs => Self::MaxAbs,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(default)]
struct PhaseCorrection2DJson {
    x_zero_order_deg: f64,
    x_first_order_deg: f64,
    x_pivot_fraction: f64,
    y_zero_order_deg: f64,
    y_first_order_deg: f64,
    y_pivot_fraction: f64,
}

impl Default for PhaseCorrection2DJson {
    fn default() -> Self {
        PhaseCorrection2D::default().into()
    }
}

impl From<PhaseCorrection2DJson> for PhaseCorrection2D {
    fn from(correction: PhaseCorrection2DJson) -> Self {
        Self {
            x_zero_order_deg: correction.x_zero_order_deg,
            x_first_order_deg: correction.x_first_order_deg,
            x_pivot_fraction: correction.x_pivot_fraction,
            y_zero_order_deg: correction.y_zero_order_deg,
            y_first_order_deg: correction.y_first_order_deg,
            y_pivot_fraction: correction.y_pivot_fraction,
        }
    }
}

impl From<PhaseCorrection2D> for PhaseCorrection2DJson {
    fn from(correction: PhaseCorrection2D) -> Self {
        Self {
            x_zero_order_deg: correction.x_zero_order_deg,
            x_first_order_deg: correction.x_first_order_deg,
            x_pivot_fraction: correction.x_pivot_fraction,
            y_zero_order_deg: correction.y_zero_order_deg,
            y_first_order_deg: correction.y_first_order_deg,
            y_pivot_fraction: correction.y_pivot_fraction,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(default)]
struct AutoPhase2DOptionsJson {
    x_zero_order_min_deg: f64,
    x_zero_order_max_deg: f64,
    x_zero_order_step_deg: f64,
    x_first_order_min_deg: f64,
    x_first_order_max_deg: f64,
    x_first_order_step_deg: f64,
    x_pivot_fraction: f64,
    y_zero_order_min_deg: f64,
    y_zero_order_max_deg: f64,
    y_zero_order_step_deg: f64,
    y_first_order_min_deg: f64,
    y_first_order_max_deg: f64,
    y_first_order_step_deg: f64,
    y_pivot_fraction: f64,
    imaginary_weight: f64,
    negative_weight: f64,
}

impl Default for AutoPhase2DOptionsJson {
    fn default() -> Self {
        let options = AutoPhase2DOptions::default();
        Self {
            x_zero_order_min_deg: options.x_zero_order_min_deg,
            x_zero_order_max_deg: options.x_zero_order_max_deg,
            x_zero_order_step_deg: options.x_zero_order_step_deg,
            x_first_order_min_deg: options.x_first_order_min_deg,
            x_first_order_max_deg: options.x_first_order_max_deg,
            x_first_order_step_deg: options.x_first_order_step_deg,
            x_pivot_fraction: options.x_pivot_fraction,
            y_zero_order_min_deg: options.y_zero_order_min_deg,
            y_zero_order_max_deg: options.y_zero_order_max_deg,
            y_zero_order_step_deg: options.y_zero_order_step_deg,
            y_first_order_min_deg: options.y_first_order_min_deg,
            y_first_order_max_deg: options.y_first_order_max_deg,
            y_first_order_step_deg: options.y_first_order_step_deg,
            y_pivot_fraction: options.y_pivot_fraction,
            imaginary_weight: options.imaginary_weight,
            negative_weight: options.negative_weight,
        }
    }
}

impl From<AutoPhase2DOptionsJson> for AutoPhase2DOptions {
    fn from(options: AutoPhase2DOptionsJson) -> Self {
        Self {
            x_zero_order_min_deg: options.x_zero_order_min_deg,
            x_zero_order_max_deg: options.x_zero_order_max_deg,
            x_zero_order_step_deg: options.x_zero_order_step_deg,
            x_first_order_min_deg: options.x_first_order_min_deg,
            x_first_order_max_deg: options.x_first_order_max_deg,
            x_first_order_step_deg: options.x_first_order_step_deg,
            x_pivot_fraction: options.x_pivot_fraction,
            y_zero_order_min_deg: options.y_zero_order_min_deg,
            y_zero_order_max_deg: options.y_zero_order_max_deg,
            y_zero_order_step_deg: options.y_zero_order_step_deg,
            y_first_order_min_deg: options.y_first_order_min_deg,
            y_first_order_max_deg: options.y_first_order_max_deg,
            y_first_order_step_deg: options.y_first_order_step_deg,
            y_pivot_fraction: options.y_pivot_fraction,
            imaginary_weight: options.imaginary_weight,
            negative_weight: options.negative_weight,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct AutoPhase2DResponseJson {
    spectrum: Spectrum2D,
    correction: PhaseCorrection2DJson,
    score: f64,
}

#[cfg(test)]
mod tests;
