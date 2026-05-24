//! WebAssembly bindings for analysis workflows.

use wasm_bindgen::prelude::*;

use crate::{detect_ranges_json, detect_zones_json, js_error};

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
