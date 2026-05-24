//! WebAssembly bindings for contour extraction.

use wasm_bindgen::prelude::*;

use crate::{extract_contours_2d_json, js_error};

/// Extracts contour segments from serialized two-dimensional spectrum JSON.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, contour extraction,
/// or serialization fails.
#[wasm_bindgen(js_name = extractContours2d)]
pub fn extract_contours_2d(
    spectrum_json: &str,
    levels_json: &str,
) -> std::result::Result<String, JsValue> {
    extract_contours_2d_json(spectrum_json, levels_json).map_err(|error| js_error(&error))
}
