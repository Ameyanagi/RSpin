//! Two-dimensional processing WebAssembly bindings.

use wasm_bindgen::prelude::*;

use crate::{
    abs_spectrum_2d_json, apply_processing_recipe_2d_json, apply_processing_recipe_2d_until_json,
    auto_phase_spectrum_2d_json, crop_spectrum_2d_json, exponential_apodization_spectrum_2d_json,
    fft_spectrum_2d_json, gaussian_apodization_spectrum_2d_json, js_error,
    normalize_spectrum_2d_json, phase_spectrum_2d_json, project_spectrum_2d_x_json,
    project_spectrum_2d_y_json, resample_spectrum_2d_json, scale_spectrum_2d_json,
    sine_bell_apodization_spectrum_2d_json, slice_spectrum_2d_x_at_y_index_json,
    slice_spectrum_2d_x_at_y_json, slice_spectrum_2d_y_at_x_index_json,
    slice_spectrum_2d_y_at_x_json, zero_fill_spectrum_2d_json,
};

/// Scales a serialized two-dimensional spectrum.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, processing, or
/// serialization fails.
#[wasm_bindgen(js_name = scaleSpectrum2d)]
pub fn scale_spectrum_2d(spectrum_json: &str, factor: f64) -> std::result::Result<String, JsValue> {
    scale_spectrum_2d_json(spectrum_json, factor).map_err(|error| js_error(&error))
}

/// Normalizes a serialized two-dimensional spectrum by maximum absolute value.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, processing, or
/// serialization fails.
#[wasm_bindgen(js_name = normalizeSpectrum2d)]
pub fn normalize_spectrum_2d(spectrum_json: &str) -> std::result::Result<String, JsValue> {
    normalize_spectrum_2d_json(spectrum_json).map_err(|error| js_error(&error))
}

/// Zero-fills a serialized two-dimensional spectrum.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, processing, or
/// serialization fails.
#[wasm_bindgen(js_name = zeroFillSpectrum2d)]
pub fn zero_fill_spectrum_2d(
    spectrum_json: &str,
    target_width: usize,
    target_height: usize,
) -> std::result::Result<String, JsValue> {
    zero_fill_spectrum_2d_json(spectrum_json, target_width, target_height)
        .map_err(|error| js_error(&error))
}

/// Crops a serialized two-dimensional spectrum to inclusive x and y windows.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, processing, or
/// serialization fails.
#[wasm_bindgen(js_name = cropSpectrum2d)]
pub fn crop_spectrum_2d(
    spectrum_json: &str,
    x_from: f64,
    x_to: f64,
    y_from: f64,
    y_to: f64,
) -> std::result::Result<String, JsValue> {
    crop_spectrum_2d_json(spectrum_json, x_from, x_to, y_from, y_to)
        .map_err(|error| js_error(&error))
}

/// Applies component-wise absolute value to a serialized two-dimensional spectrum.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, processing, or
/// serialization fails.
#[wasm_bindgen(js_name = absSpectrum2d)]
pub fn abs_spectrum_2d(spectrum_json: &str) -> std::result::Result<String, JsValue> {
    abs_spectrum_2d_json(spectrum_json).map_err(|error| js_error(&error))
}

/// Resamples a serialized two-dimensional spectrum onto serialized target axes.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, processing, or
/// serialization fails.
#[wasm_bindgen(js_name = resampleSpectrum2d)]
pub fn resample_spectrum_2d(
    spectrum_json: &str,
    target_columns_json: &str,
    target_rows_json: &str,
    outside_value: f64,
) -> std::result::Result<String, JsValue> {
    resample_spectrum_2d_json(
        spectrum_json,
        target_columns_json,
        target_rows_json,
        outside_value,
    )
    .map_err(|error| js_error(&error))
}

/// Applies a two-dimensional FFT to a serialized spectrum.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, processing, or
/// serialization fails.
#[wasm_bindgen(js_name = fftSpectrum2d)]
pub fn fft_spectrum_2d(
    spectrum_json: &str,
    direction_json: &str,
) -> std::result::Result<String, JsValue> {
    fft_spectrum_2d_json(spectrum_json, direction_json).map_err(|error| js_error(&error))
}

/// Applies separable exponential apodization to a serialized two-dimensional spectrum.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, processing, or
/// serialization fails.
#[wasm_bindgen(js_name = exponentialApodizationSpectrum2d)]
pub fn exponential_apodization_spectrum_2d(
    spectrum_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    exponential_apodization_spectrum_2d_json(spectrum_json, options_json)
        .map_err(|error| js_error(&error))
}

/// Applies separable Gaussian apodization to a serialized two-dimensional spectrum.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, processing, or
/// serialization fails.
#[wasm_bindgen(js_name = gaussianApodizationSpectrum2d)]
pub fn gaussian_apodization_spectrum_2d(
    spectrum_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    gaussian_apodization_spectrum_2d_json(spectrum_json, options_json)
        .map_err(|error| js_error(&error))
}

/// Applies separable sine-bell apodization to a serialized two-dimensional spectrum.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, processing, or
/// serialization fails.
#[wasm_bindgen(js_name = sineBellApodizationSpectrum2d)]
pub fn sine_bell_apodization_spectrum_2d(
    spectrum_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    sine_bell_apodization_spectrum_2d_json(spectrum_json, options_json)
        .map_err(|error| js_error(&error))
}

/// Applies manual x/y phase correction to a serialized two-dimensional spectrum.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, processing, or
/// serialization fails.
#[wasm_bindgen(js_name = phaseSpectrum2d)]
pub fn phase_spectrum_2d(
    spectrum_json: &str,
    correction_json: &str,
) -> std::result::Result<String, JsValue> {
    phase_spectrum_2d_json(spectrum_json, correction_json).map_err(|error| js_error(&error))
}

/// Automatically phases a serialized two-dimensional spectrum.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, processing, or
/// serialization fails.
#[wasm_bindgen(js_name = autoPhaseSpectrum2d)]
pub fn auto_phase_spectrum_2d(
    spectrum_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    auto_phase_spectrum_2d_json(spectrum_json, options_json).map_err(|error| js_error(&error))
}

/// Applies a serialized processing recipe to a serialized two-dimensional spectrum.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, processing, or
/// serialization fails.
#[wasm_bindgen(js_name = applyProcessingRecipe2d)]
pub fn apply_processing_recipe_2d(
    spectrum_json: &str,
    recipe_json: &str,
) -> std::result::Result<String, JsValue> {
    apply_processing_recipe_2d_json(spectrum_json, recipe_json).map_err(|error| js_error(&error))
}

/// Applies the first operations in a serialized processing recipe.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, processing, or
/// serialization fails.
#[wasm_bindgen(js_name = applyProcessingRecipe2dUntil)]
pub fn apply_processing_recipe_2d_until(
    spectrum_json: &str,
    recipe_json: &str,
    operation_count: usize,
) -> std::result::Result<String, JsValue> {
    apply_processing_recipe_2d_until_json(spectrum_json, recipe_json, operation_count)
        .map_err(|error| js_error(&error))
}

/// Projects a serialized two-dimensional spectrum onto the x axis.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, processing, or
/// serialization fails.
#[wasm_bindgen(js_name = projectSpectrum2dX)]
pub fn project_spectrum_2d_x(
    spectrum_json: &str,
    mode_json: &str,
) -> std::result::Result<String, JsValue> {
    project_spectrum_2d_x_json(spectrum_json, mode_json).map_err(|error| js_error(&error))
}

/// Projects a serialized two-dimensional spectrum onto the y axis.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, processing, or
/// serialization fails.
#[wasm_bindgen(js_name = projectSpectrum2dY)]
pub fn project_spectrum_2d_y(
    spectrum_json: &str,
    mode_json: &str,
) -> std::result::Result<String, JsValue> {
    project_spectrum_2d_y_json(spectrum_json, mode_json).map_err(|error| js_error(&error))
}

/// Extracts an x-axis row from a serialized two-dimensional spectrum by y index.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, processing, or
/// serialization fails.
#[wasm_bindgen(js_name = sliceSpectrum2dXAtYIndex)]
pub fn slice_spectrum_2d_x_at_y_index(
    spectrum_json: &str,
    y_index: usize,
) -> std::result::Result<String, JsValue> {
    slice_spectrum_2d_x_at_y_index_json(spectrum_json, y_index).map_err(|error| js_error(&error))
}

/// Extracts the x-axis row nearest a y coordinate.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, processing, or
/// serialization fails.
#[wasm_bindgen(js_name = sliceSpectrum2dXAtY)]
pub fn slice_spectrum_2d_x_at_y(
    spectrum_json: &str,
    y: f64,
) -> std::result::Result<String, JsValue> {
    slice_spectrum_2d_x_at_y_json(spectrum_json, y).map_err(|error| js_error(&error))
}

/// Extracts a y-axis column from a serialized two-dimensional spectrum by x index.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, processing, or
/// serialization fails.
#[wasm_bindgen(js_name = sliceSpectrum2dYAtXIndex)]
pub fn slice_spectrum_2d_y_at_x_index(
    spectrum_json: &str,
    x_index: usize,
) -> std::result::Result<String, JsValue> {
    slice_spectrum_2d_y_at_x_index_json(spectrum_json, x_index).map_err(|error| js_error(&error))
}

/// Extracts the y-axis column nearest an x coordinate.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, processing, or
/// serialization fails.
#[wasm_bindgen(js_name = sliceSpectrum2dYAtX)]
pub fn slice_spectrum_2d_y_at_x(
    spectrum_json: &str,
    x: f64,
) -> std::result::Result<String, JsValue> {
    slice_spectrum_2d_y_at_x_json(spectrum_json, x).map_err(|error| js_error(&error))
}
