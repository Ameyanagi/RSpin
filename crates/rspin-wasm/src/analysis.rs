//! WebAssembly bindings for analysis workflows.

use wasm_bindgen::prelude::*;

use crate::{
    align_spectra_by_peak_to_matrix_1d_json, bucket_spectra_1d_json, bucket_spectra_2d_json,
    bucket_spectrum_1d_json, bucket_spectrum_2d_json, detect_ranges_json, detect_zones_json,
    js_error,
};

/// Detects ranges from a serialized one-dimensional spectrum.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, analysis, or
/// serialization fails.
#[wasm_bindgen(js_name = detectRanges1d)]
pub fn detect_ranges_1d(
    spectrum_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    detect_ranges_json(spectrum_json, options_json).map_err(|error| js_error(&error))
}

/// Detects connected zones from a serialized two-dimensional spectrum.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, analysis, or
/// serialization fails.
#[wasm_bindgen(js_name = detectZones2d)]
pub fn detect_zones_2d(
    spectrum_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    detect_zones_json(spectrum_json, options_json).map_err(|error| js_error(&error))
}

/// Aligns one-dimensional spectra by peak and generates a common matrix.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, alignment, matrix
/// generation, or serialization fails.
#[wasm_bindgen(js_name = alignSpectraByPeakToMatrix1d)]
pub fn align_spectra_by_peak_to_matrix_1d(
    spectra_json: &str,
    alignment_options_json: &str,
    matrix_options_json: &str,
) -> std::result::Result<String, JsValue> {
    align_spectra_by_peak_to_matrix_1d_json(
        spectra_json,
        alignment_options_json,
        matrix_options_json,
    )
    .map_err(|error| js_error(&error))
}

/// Buckets a one-dimensional spectrum.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, bucketing, or
/// serialization fails.
#[wasm_bindgen(js_name = bucketSpectrum1d)]
pub fn bucket_spectrum_1d(
    spectrum_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    bucket_spectrum_1d_json(spectrum_json, options_json).map_err(|error| js_error(&error))
}

/// Buckets one-dimensional spectra into a row-major matrix.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, bucketing, or
/// serialization fails.
#[wasm_bindgen(js_name = bucketSpectra1d)]
pub fn bucket_spectra_1d(
    spectra_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    bucket_spectra_1d_json(spectra_json, options_json).map_err(|error| js_error(&error))
}

/// Buckets a two-dimensional spectrum.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, bucketing, or
/// serialization fails.
#[wasm_bindgen(js_name = bucketSpectrum2d)]
pub fn bucket_spectrum_2d(
    spectrum_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    bucket_spectrum_2d_json(spectrum_json, options_json).map_err(|error| js_error(&error))
}

/// Buckets two-dimensional spectra into a layer-major matrix.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, bucketing, or
/// serialization fails.
#[wasm_bindgen(js_name = bucketSpectra2d)]
pub fn bucket_spectra_2d(
    spectra_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    bucket_spectra_2d_json(spectra_json, options_json).map_err(|error| js_error(&error))
}
