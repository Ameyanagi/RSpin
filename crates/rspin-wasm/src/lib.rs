//! WebAssembly bindings.

mod analysis;
mod api;
mod contours;
mod io;

use rspin_core::RSpinError;
use wasm_bindgen::prelude::*;

pub use analysis::{detect_ranges_1d, detect_zones_2d};
pub use api::{
    auto_phase_spectrum_1d_json, auto_phase_spectrum_2d_json,
    decompose_exact_spin_half_spectrum_json, detect_multiplets_json, detect_ranges_json,
    detect_zones_json, extract_contours_2d_json, fft_spectrum_2d_json, integrate_region_json,
    normalize_spectrum_1d_json, normalize_spectrum_2d_json, optimize_peaks_json,
    parse_jcamp_dx_1d_json, parse_spectrum_1d_csv_json, parse_spectrum_2d_csv_json,
    phase_spectrum_2d_json, pick_peaks_json, project_spectrum_2d_x_json,
    project_spectrum_2d_y_json, render_prediction_1d_json, scale_spectrum_1d_json,
    scale_spectrum_2d_json, simulate_exact_spin_half_spectrum_json,
    simulate_exact_spin_half_transitions_json, simulate_first_order_multiplet_json,
    slice_spectrum_2d_x_at_y_index_json, slice_spectrum_2d_y_at_x_index_json,
    summarize_signals_1d_json, validate_j_coupling_graph_json, validate_prediction_json,
    write_spectrum_1d_csv_json, write_spectrum_2d_csv_json, zero_fill_spectrum_2d_json,
};
pub use contours::extract_contours_2d;
pub use io::{
    parse_spectrum_1d_csv, parse_spectrum_2d_csv, write_spectrum_1d_csv, write_spectrum_2d_csv,
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

/// Extracts an x-axis row from a serialized two-dimensional spectrum.
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

/// Extracts a y-axis column from a serialized two-dimensional spectrum.
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

pub(crate) fn js_error(error: &RSpinError) -> JsValue {
    JsValue::from_str(&error.to_string())
}
