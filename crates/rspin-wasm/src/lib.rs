//! WebAssembly bindings.

mod api;

use rspin_core::RSpinError;
use wasm_bindgen::prelude::*;

pub use api::{
    decompose_exact_spin_half_spectrum_json, detect_multiplets_json, integrate_region_json,
    normalize_spectrum_1d_json, optimize_peaks_json, parse_jcamp_dx_1d_json, pick_peaks_json,
    render_prediction_1d_json, scale_spectrum_1d_json, simulate_exact_spin_half_spectrum_json,
    simulate_exact_spin_half_transitions_json, simulate_first_order_multiplet_json,
    summarize_signals_1d_json, validate_j_coupling_graph_json, validate_prediction_json,
};

/// Parses JCAMP-DX text into a serialized one-dimensional spectrum.
///
/// # Errors
///
/// Returns a JavaScript error string when parsing or serialization fails.
#[wasm_bindgen(js_name = parseJcampDx1d)]
pub fn parse_jcamp_dx_1d(input: &str) -> std::result::Result<String, JsValue> {
    parse_jcamp_dx_1d_json(input).map_err(|error| js_error(&error))
}

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

/// Picks peaks from a serialized one-dimensional spectrum.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, analysis, or
/// serialization fails.
#[wasm_bindgen(js_name = pickPeaks1d)]
pub fn pick_peaks_1d(
    spectrum_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    pick_peaks_json(spectrum_json, options_json).map_err(|error| js_error(&error))
}

/// Optimizes serialized one-dimensional peaks.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, analysis, or
/// serialization fails.
#[wasm_bindgen(js_name = optimizePeaks1d)]
pub fn optimize_peaks_1d(
    spectrum_json: &str,
    peaks_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    optimize_peaks_json(spectrum_json, peaks_json, options_json).map_err(|error| js_error(&error))
}

/// Detects serialized one-dimensional multiplets.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, analysis, or
/// serialization fails.
#[wasm_bindgen(js_name = detectMultiplets1d)]
pub fn detect_multiplets_1d(
    spectrum_json: &str,
    peaks_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    detect_multiplets_json(spectrum_json, peaks_json, options_json)
        .map_err(|error| js_error(&error))
}

/// Validates a serialized J-coupling graph and returns its normalized JSON.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, validation, or
/// serialization fails.
#[wasm_bindgen(js_name = validateJCouplingGraph)]
pub fn validate_j_coupling_graph(graph_json: &str) -> std::result::Result<String, JsValue> {
    validate_j_coupling_graph_json(graph_json).map_err(|error| js_error(&error))
}

/// Assembles serialized one-dimensional signal summaries.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, validation, analysis,
/// or serialization fails.
#[wasm_bindgen(js_name = summarizeSignals1d)]
pub fn summarize_signals_1d(
    spectrum_json: &str,
    ranges_json: &str,
    multiplets_json: &str,
    assignments_json: &str,
    coupling_graph_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    summarize_signals_1d_json(
        spectrum_json,
        ranges_json,
        multiplets_json,
        assignments_json,
        coupling_graph_json,
        options_json,
    )
    .map_err(|error| js_error(&error))
}

/// Integrates a serialized one-dimensional spectrum over a region.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, analysis, or
/// serialization fails.
#[wasm_bindgen(js_name = integrateRegion1d)]
pub fn integrate_region_1d(
    spectrum_json: &str,
    region_json: &str,
) -> std::result::Result<String, JsValue> {
    integrate_region_json(spectrum_json, region_json).map_err(|error| js_error(&error))
}

/// Simulates a first-order multiplet as a serialized one-dimensional spectrum.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, simulation, or
/// serialization fails.
#[wasm_bindgen(js_name = simulateFirstOrderMultiplet)]
pub fn simulate_first_order_multiplet(
    multiplet_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    simulate_first_order_multiplet_json(multiplet_json, options_json)
        .map_err(|error| js_error(&error))
}

/// Simulates exact spin-1/2 transitions as serialized JSON.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, simulation, or
/// serialization fails.
#[wasm_bindgen(js_name = simulateExactSpinHalfTransitions)]
pub fn simulate_exact_spin_half_transitions(
    system_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    simulate_exact_spin_half_transitions_json(system_json, options_json)
        .map_err(|error| js_error(&error))
}

/// Simulates an exact spin-1/2 system as a serialized one-dimensional spectrum.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, simulation, or
/// serialization fails.
#[wasm_bindgen(js_name = simulateExactSpinHalfSpectrum)]
pub fn simulate_exact_spin_half_spectrum(
    system_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    simulate_exact_spin_half_spectrum_json(system_json, options_json)
        .map_err(|error| js_error(&error))
}

/// Simulates exact spin-1/2 spectrum and per-transition contributions as JSON.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, simulation, or
/// serialization fails.
#[wasm_bindgen(js_name = decomposeExactSpinHalfSpectrum)]
pub fn decompose_exact_spin_half_spectrum(
    system_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    decompose_exact_spin_half_spectrum_json(system_json, options_json)
        .map_err(|error| js_error(&error))
}

/// Validates a serialized prediction payload and returns its normalized JSON.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, validation, or
/// serialization fails.
#[wasm_bindgen(js_name = validatePrediction)]
pub fn validate_prediction(prediction_json: &str) -> std::result::Result<String, JsValue> {
    validate_prediction_json(prediction_json).map_err(|error| js_error(&error))
}

/// Renders a serialized one-dimensional prediction as a spectrum.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, validation,
/// rendering, or serialization fails.
#[wasm_bindgen(js_name = renderPrediction1d)]
pub fn render_prediction_1d(
    prediction_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    render_prediction_1d_json(prediction_json, options_json).map_err(|error| js_error(&error))
}

fn js_error(error: &RSpinError) -> JsValue {
    JsValue::from_str(&error.to_string())
}
