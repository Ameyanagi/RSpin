//! WebAssembly bindings for high-level analysis workflows.

use wasm_bindgen::prelude::*;

use crate::{analyze_spectrum_1d_json, analyze_spectrum_2d_json, js_error};

/// Runs the default one-dimensional analysis workflow.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, analysis, or
/// serialization fails.
#[wasm_bindgen(js_name = analyzeSpectrum1d)]
pub fn analyze_spectrum_1d(
    spectrum_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    analyze_spectrum_1d_json(spectrum_json, options_json).map_err(|error| js_error(&error))
}

/// Runs the default two-dimensional analysis workflow.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, analysis, or
/// serialization fails.
#[wasm_bindgen(js_name = analyzeSpectrum2d)]
pub fn analyze_spectrum_2d(
    spectrum_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    analyze_spectrum_2d_json(spectrum_json, options_json).map_err(|error| js_error(&error))
}
