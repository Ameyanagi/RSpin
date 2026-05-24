//! WebAssembly bindings for spectrum IO workflows.

use wasm_bindgen::prelude::*;

use crate::{
    inspect_nmrml_document_json, js_error, parse_nmredata_json, parse_nmredata_records_json,
    parse_nmrml_1d_json, parse_nmrml_2d_json, parse_spectrum_1d_csv_json,
    parse_spectrum_1d_text_json, parse_spectrum_2d_csv_json, parse_spectrum_2d_text_json,
    write_analysis_1d_csv_json, write_analysis_2d_csv_json, write_nmredata_json,
    write_nmredata_records_json, write_nmrml_1d_json, write_nmrml_2d_json,
    write_spectrum_1d_csv_json, write_spectrum_2d_csv_json,
};

/// Parses one-dimensional CSV text into serialized spectrum JSON.
///
/// # Errors
///
/// Returns a JavaScript error string when parsing or serialization fails.
#[wasm_bindgen(js_name = parseSpectrum1dCsv)]
pub fn parse_spectrum_1d_csv(input: &str) -> std::result::Result<String, JsValue> {
    parse_spectrum_1d_csv_json(input).map_err(|error| js_error(&error))
}

/// Parses auto-detected one-dimensional spectrum text into serialized spectrum JSON.
///
/// # Errors
///
/// Returns a JavaScript error string when parsing or serialization fails.
#[wasm_bindgen(js_name = parseSpectrum1dText)]
pub fn parse_spectrum_1d_text(input: &str) -> std::result::Result<String, JsValue> {
    parse_spectrum_1d_text_json(input).map_err(|error| js_error(&error))
}

/// Parses nmrML text into serialized spectrum JSON.
///
/// # Errors
///
/// Returns a JavaScript error string when parsing or serialization fails.
#[wasm_bindgen(js_name = parseNmrMl1d)]
pub fn parse_nmrml_1d(input: &str) -> std::result::Result<String, JsValue> {
    parse_nmrml_1d_json(input).map_err(|error| js_error(&error))
}

/// Parses two-dimensional nmrML text into serialized spectrum JSON.
///
/// # Errors
///
/// Returns a JavaScript error string when parsing or serialization fails.
#[wasm_bindgen(js_name = parseNmrMl2d)]
pub fn parse_nmrml_2d(input: &str) -> std::result::Result<String, JsValue> {
    parse_nmrml_2d_json(input).map_err(|error| js_error(&error))
}

/// Parses `NMReDATA` SDF text into serialized record JSON.
///
/// # Errors
///
/// Returns a JavaScript error string when parsing or serialization fails.
#[wasm_bindgen(js_name = parseNmreData)]
pub fn parse_nmredata(input: &str) -> std::result::Result<String, JsValue> {
    parse_nmredata_json(input).map_err(|error| js_error(&error))
}

/// Parses all `NMReDATA` SDF records into serialized record-list JSON.
///
/// # Errors
///
/// Returns a JavaScript error string when parsing or serialization fails.
#[wasm_bindgen(js_name = parseNmreDataRecords)]
pub fn parse_nmredata_records(input: &str) -> std::result::Result<String, JsValue> {
    parse_nmredata_records_json(input).map_err(|error| js_error(&error))
}

/// Serializes `NMReDATA` record JSON into SDF text.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization or serialization fails.
#[wasm_bindgen(js_name = writeNmreData)]
pub fn write_nmredata(record_json: &str) -> std::result::Result<String, JsValue> {
    write_nmredata_json(record_json).map_err(|error| js_error(&error))
}

/// Serializes `NMReDATA` record-list JSON into SDF text.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization or serialization fails.
#[wasm_bindgen(js_name = writeNmreDataRecords)]
pub fn write_nmredata_records(records_json: &str) -> std::result::Result<String, JsValue> {
    write_nmredata_records_json(records_json).map_err(|error| js_error(&error))
}

/// Parses root-level nmrML document metadata into JSON.
///
/// # Errors
///
/// Returns a JavaScript error string when parsing or serialization fails.
#[wasm_bindgen(js_name = inspectNmrMlDocument)]
pub fn inspect_nmrml_document(input: &str) -> std::result::Result<String, JsValue> {
    inspect_nmrml_document_json(input).map_err(|error| js_error(&error))
}

/// Serializes one-dimensional spectrum JSON into nmrML text.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization or nmrML serialization fails.
#[wasm_bindgen(js_name = writeNmrMl1d)]
pub fn write_nmrml_1d(spectrum_json: &str) -> std::result::Result<String, JsValue> {
    write_nmrml_1d_json(spectrum_json).map_err(|error| js_error(&error))
}

/// Serializes two-dimensional spectrum JSON into nmrML text.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization or nmrML serialization fails.
#[wasm_bindgen(js_name = writeNmrMl2d)]
pub fn write_nmrml_2d(spectrum_json: &str) -> std::result::Result<String, JsValue> {
    write_nmrml_2d_json(spectrum_json).map_err(|error| js_error(&error))
}

/// Serializes one-dimensional spectrum JSON into CSV text.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization or CSV serialization fails.
#[wasm_bindgen(js_name = writeSpectrum1dCsv)]
pub fn write_spectrum_1d_csv(spectrum_json: &str) -> std::result::Result<String, JsValue> {
    write_spectrum_1d_csv_json(spectrum_json).map_err(|error| js_error(&error))
}

/// Parses two-dimensional CSV text into serialized spectrum JSON.
///
/// # Errors
///
/// Returns a JavaScript error string when parsing or serialization fails.
#[wasm_bindgen(js_name = parseSpectrum2dCsv)]
pub fn parse_spectrum_2d_csv(input: &str) -> std::result::Result<String, JsValue> {
    parse_spectrum_2d_csv_json(input).map_err(|error| js_error(&error))
}

/// Parses auto-detected two-dimensional spectrum text into serialized spectrum JSON.
///
/// # Errors
///
/// Returns a JavaScript error string when parsing or serialization fails.
#[wasm_bindgen(js_name = parseSpectrum2dText)]
pub fn parse_spectrum_2d_text(input: &str) -> std::result::Result<String, JsValue> {
    parse_spectrum_2d_text_json(input).map_err(|error| js_error(&error))
}

/// Serializes two-dimensional spectrum JSON into CSV text.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization or CSV serialization fails.
#[wasm_bindgen(js_name = writeSpectrum2dCsv)]
pub fn write_spectrum_2d_csv(spectrum_json: &str) -> std::result::Result<String, JsValue> {
    write_spectrum_2d_csv_json(spectrum_json).map_err(|error| js_error(&error))
}

/// Serializes one-dimensional analysis JSON into multi-section CSV text.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization or CSV serialization fails.
#[wasm_bindgen(js_name = writeAnalysis1dCsv)]
pub fn write_analysis_1d_csv(analysis_json: &str) -> std::result::Result<String, JsValue> {
    write_analysis_1d_csv_json(analysis_json).map_err(|error| js_error(&error))
}

/// Serializes two-dimensional analysis JSON into multi-section CSV text.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization or CSV serialization fails.
#[wasm_bindgen(js_name = writeAnalysis2dCsv)]
pub fn write_analysis_2d_csv(analysis_json: &str) -> std::result::Result<String, JsValue> {
    write_analysis_2d_csv_json(analysis_json).map_err(|error| js_error(&error))
}
