//! WebAssembly bindings for one-dimensional processing workflows.

use wasm_bindgen::prelude::*;

use crate::{
    auto_phase_spectrum_1d_json, exponential_apodization_spectrum_1d_json, fft_spectrum_1d_json,
    js_error, magnitude_spectrum_1d_json, normalize_spectrum_1d_json, offset_spectrum_1d_json,
    phase_spectrum_1d_json, scale_spectrum_1d_json, shift_spectrum_1d_axis_json,
    subtract_baseline_spectrum_1d_json, zero_fill_spectrum_1d_json,
};

/// Scales a serialized one-dimensional spectrum.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, processing, or
/// serialization fails.
#[wasm_bindgen(js_name = scaleSpectrum1d)]
pub fn scale_spectrum_1d(spectrum_json: &str, factor: f64) -> std::result::Result<String, JsValue> {
    scale_spectrum_1d_json(spectrum_json, factor).map_err(|error| js_error(&error))
}

/// Offsets serialized one-dimensional real intensities.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, processing, or
/// serialization fails.
#[wasm_bindgen(js_name = offsetSpectrum1d)]
pub fn offset_spectrum_1d(
    spectrum_json: &str,
    offset: f64,
) -> std::result::Result<String, JsValue> {
    offset_spectrum_1d_json(spectrum_json, offset).map_err(|error| js_error(&error))
}

/// Normalizes a serialized one-dimensional spectrum by maximum absolute value.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, processing, or
/// serialization fails.
#[wasm_bindgen(js_name = normalizeSpectrum1d)]
pub fn normalize_spectrum_1d(spectrum_json: &str) -> std::result::Result<String, JsValue> {
    normalize_spectrum_1d_json(spectrum_json).map_err(|error| js_error(&error))
}

/// Shifts a serialized one-dimensional spectrum axis.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, processing, or
/// serialization fails.
#[wasm_bindgen(js_name = shiftSpectrum1dAxis)]
pub fn shift_spectrum_1d_axis(
    spectrum_json: &str,
    delta: f64,
) -> std::result::Result<String, JsValue> {
    shift_spectrum_1d_axis_json(spectrum_json, delta).map_err(|error| js_error(&error))
}

/// Zero-fills a serialized one-dimensional spectrum.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, processing, or
/// serialization fails.
#[wasm_bindgen(js_name = zeroFillSpectrum1d)]
pub fn zero_fill_spectrum_1d(
    spectrum_json: &str,
    target_len: usize,
) -> std::result::Result<String, JsValue> {
    zero_fill_spectrum_1d_json(spectrum_json, target_len).map_err(|error| js_error(&error))
}

/// Applies a one-dimensional FFT to a serialized spectrum.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, processing, or
/// serialization fails.
#[wasm_bindgen(js_name = fftSpectrum1d)]
pub fn fft_spectrum_1d(
    spectrum_json: &str,
    direction_json: &str,
) -> std::result::Result<String, JsValue> {
    fft_spectrum_1d_json(spectrum_json, direction_json).map_err(|error| js_error(&error))
}

/// Applies manual phase correction to a serialized one-dimensional spectrum.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, processing, or
/// serialization fails.
#[wasm_bindgen(js_name = phaseSpectrum1d)]
pub fn phase_spectrum_1d(
    spectrum_json: &str,
    correction_json: &str,
) -> std::result::Result<String, JsValue> {
    phase_spectrum_1d_json(spectrum_json, correction_json).map_err(|error| js_error(&error))
}

/// Converts a serialized one-dimensional spectrum to magnitude mode.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, processing, or
/// serialization fails.
#[wasm_bindgen(js_name = magnitudeSpectrum1d)]
pub fn magnitude_spectrum_1d(spectrum_json: &str) -> std::result::Result<String, JsValue> {
    magnitude_spectrum_1d_json(spectrum_json).map_err(|error| js_error(&error))
}

/// Applies exponential apodization to a serialized one-dimensional spectrum.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, processing, or
/// serialization fails.
#[wasm_bindgen(js_name = exponentialApodizationSpectrum1d)]
pub fn exponential_apodization_spectrum_1d(
    spectrum_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    exponential_apodization_spectrum_1d_json(spectrum_json, options_json)
        .map_err(|error| js_error(&error))
}

/// Subtracts a fitted baseline from a serialized one-dimensional spectrum.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, processing, or
/// serialization fails.
#[wasm_bindgen(js_name = subtractBaselineSpectrum1d)]
pub fn subtract_baseline_spectrum_1d(
    spectrum_json: &str,
    method_json: &str,
) -> std::result::Result<String, JsValue> {
    subtract_baseline_spectrum_1d_json(spectrum_json, method_json).map_err(|error| js_error(&error))
}

/// Automatically phases a serialized one-dimensional spectrum.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, processing, or
/// serialization fails.
#[wasm_bindgen(js_name = autoPhaseSpectrum1d)]
pub fn auto_phase_spectrum_1d(
    spectrum_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    auto_phase_spectrum_1d_json(spectrum_json, options_json).map_err(|error| js_error(&error))
}
