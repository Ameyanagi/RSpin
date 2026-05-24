//! WebAssembly bindings.

mod analysis;
mod api;
mod contours;
mod io;
mod processing_1d;
mod processing_2d;

use rspin_core::RSpinError;
use wasm_bindgen::prelude::*;

pub use analysis::{
    align_spectra_by_peak_to_matrix_1d, bucket_spectra_1d, bucket_spectra_2d, bucket_spectrum_1d,
    bucket_spectrum_2d, cluster_bucket_matrix_1d, cluster_bucket_matrix_2d,
    cluster_spectrum_matrix_1d, cluster_spectrum_matrix_2d, detect_ranges_1d, detect_zones_2d,
    pairwise_bucket_matrix_1d, pairwise_bucket_matrix_2d, pairwise_spectrum_matrix_1d,
    pairwise_spectrum_matrix_2d, pca_bucket_matrix_1d, pca_bucket_matrix_2d,
    pca_spectrum_matrix_1d, pca_spectrum_matrix_2d,
};
#[cfg(feature = "first-order")]
pub use api::simulate_first_order_multiplet_json;
pub use api::{
    abs_spectrum_1d_json, abs_spectrum_2d_json, align_spectra_by_peak_to_matrix_1d_json,
    annotate_spectrum_1d_with_assignments_json, annotate_spectrum_2d_with_assignments_json,
    apply_processing_recipe_1d_json, apply_processing_recipe_1d_until_json,
    apply_processing_recipe_2d_json, apply_processing_recipe_2d_until_json,
    auto_phase_spectrum_1d_json, auto_phase_spectrum_2d_json, bucket_spectra_1d_json,
    bucket_spectra_2d_json, bucket_spectrum_1d_json, bucket_spectrum_2d_json,
    cluster_bucket_matrix_1d_json, cluster_bucket_matrix_2d_json, cluster_spectrum_matrix_1d_json,
    cluster_spectrum_matrix_2d_json, crop_spectrum_1d_json, crop_spectrum_2d_json,
    decompose_exact_spin_half_spectrum_json, detect_multiplets_json, detect_ranges_json,
    detect_zones_json, exponential_apodization_spectrum_1d_json, extract_contours_2d_json,
    fft_spectrum_1d_json, fft_spectrum_2d_json, generate_spectrum_matrix_1d_json,
    generate_spectrum_matrix_2d_json, integrate_region_2d_json, integrate_region_json,
    magnitude_spectrum_1d_json, normalize_spectrum_1d_json, normalize_spectrum_2d_json,
    offset_spectrum_1d_json, optimize_peaks_json, pairwise_bucket_matrix_1d_json,
    pairwise_bucket_matrix_2d_json, pairwise_spectrum_matrix_1d_json,
    pairwise_spectrum_matrix_2d_json, parse_jcamp_dx_1d_json, parse_spectrum_1d_csv_json,
    parse_spectrum_2d_csv_json, pca_bucket_matrix_1d_json, pca_bucket_matrix_2d_json,
    pca_spectrum_matrix_1d_json, pca_spectrum_matrix_2d_json, phase_spectrum_1d_json,
    phase_spectrum_2d_json, pick_peaks_json, predict_molecule_with_element_rules_json,
    project_spectrum_2d_x_json, project_spectrum_2d_y_json, render_prediction_1d_json,
    render_prediction_2d_json, resample_spectrum_1d_json, resample_spectrum_2d_json,
    scale_spectrum_1d_json, scale_spectrum_2d_json, shift_spectrum_1d_axis_json,
    simulate_exact_spin_half_spectrum_json, simulate_exact_spin_half_transitions_json,
    slice_spectrum_2d_x_at_y_index_json, slice_spectrum_2d_x_at_y_json,
    slice_spectrum_2d_y_at_x_index_json, slice_spectrum_2d_y_at_x_json,
    subtract_baseline_spectrum_1d_json, summarize_signals_1d_json, summarize_signals_2d_json,
    validate_assignment_set_json, validate_j_coupling_graph_json, validate_prediction_json,
    write_spectrum_1d_csv_json, write_spectrum_2d_csv_json, zero_fill_spectrum_1d_json,
    zero_fill_spectrum_2d_json,
};
pub use contours::extract_contours_2d;
pub use io::{
    parse_spectrum_1d_csv, parse_spectrum_2d_csv, write_spectrum_1d_csv, write_spectrum_2d_csv,
};
pub use processing_1d::{
    abs_spectrum_1d, apply_processing_recipe_1d, apply_processing_recipe_1d_until,
    auto_phase_spectrum_1d, crop_spectrum_1d, exponential_apodization_spectrum_1d, fft_spectrum_1d,
    magnitude_spectrum_1d, normalize_spectrum_1d, offset_spectrum_1d, phase_spectrum_1d,
    resample_spectrum_1d, scale_spectrum_1d, shift_spectrum_1d_axis, subtract_baseline_spectrum_1d,
    zero_fill_spectrum_1d,
};
pub use processing_2d::{
    abs_spectrum_2d, apply_processing_recipe_2d, apply_processing_recipe_2d_until,
    auto_phase_spectrum_2d, crop_spectrum_2d, fft_spectrum_2d, normalize_spectrum_2d,
    phase_spectrum_2d, project_spectrum_2d_x, project_spectrum_2d_y, resample_spectrum_2d,
    scale_spectrum_2d, slice_spectrum_2d_x_at_y, slice_spectrum_2d_x_at_y_index,
    slice_spectrum_2d_y_at_x, slice_spectrum_2d_y_at_x_index, zero_fill_spectrum_2d,
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

/// Validates a serialized assignment set and returns its normalized JSON.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, validation, or
/// serialization fails.
#[wasm_bindgen(js_name = validateAssignmentSet)]
pub fn validate_assignment_set(assignments_json: &str) -> std::result::Result<String, JsValue> {
    validate_assignment_set_json(assignments_json).map_err(|error| js_error(&error))
}

/// Appends assignment annotations to a serialized one-dimensional spectrum.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, assignment
/// validation, annotation validation, or serialization fails.
#[wasm_bindgen(js_name = annotateSpectrum1dWithAssignments)]
pub fn annotate_spectrum_1d_with_assignments(
    spectrum_json: &str,
    assignments_json: &str,
) -> std::result::Result<String, JsValue> {
    annotate_spectrum_1d_with_assignments_json(spectrum_json, assignments_json)
        .map_err(|error| js_error(&error))
}

/// Appends assignment annotations to a serialized two-dimensional spectrum.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, assignment
/// validation, annotation validation, or serialization fails.
#[wasm_bindgen(js_name = annotateSpectrum2dWithAssignments)]
pub fn annotate_spectrum_2d_with_assignments(
    spectrum_json: &str,
    assignments_json: &str,
) -> std::result::Result<String, JsValue> {
    annotate_spectrum_2d_with_assignments_json(spectrum_json, assignments_json)
        .map_err(|error| js_error(&error))
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

/// Assembles serialized two-dimensional signal summaries.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, validation, analysis,
/// or serialization fails.
#[wasm_bindgen(js_name = summarizeSignals2d)]
pub fn summarize_signals_2d(
    spectrum_json: &str,
    zones_json: &str,
    assignments_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    summarize_signals_2d_json(spectrum_json, zones_json, assignments_json, options_json)
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

/// Integrates a serialized two-dimensional spectrum over a rectangular region.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, analysis, or
/// serialization fails.
#[wasm_bindgen(js_name = integrateRegion2d)]
pub fn integrate_region_2d(
    spectrum_json: &str,
    region_json: &str,
) -> std::result::Result<String, JsValue> {
    integrate_region_2d_json(spectrum_json, region_json).map_err(|error| js_error(&error))
}

/// Generates a row-major matrix from serialized one-dimensional spectra.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, analysis, or
/// serialization fails.
#[wasm_bindgen(js_name = generateSpectrumMatrix1d)]
pub fn generate_spectrum_matrix_1d(
    spectra_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    generate_spectrum_matrix_1d_json(spectra_json, options_json).map_err(|error| js_error(&error))
}

/// Generates a layer-major matrix from serialized two-dimensional spectra.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, analysis, or
/// serialization fails.
#[wasm_bindgen(js_name = generateSpectrumMatrix2d)]
pub fn generate_spectrum_matrix_2d(
    spectra_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    generate_spectrum_matrix_2d_json(spectra_json, options_json).map_err(|error| js_error(&error))
}

/// Simulates a first-order multiplet as a serialized one-dimensional spectrum.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, simulation, or
/// serialization fails.
#[cfg(feature = "first-order")]
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

/// Predicts molecule signals with serialized element shift rules.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, prediction,
/// validation, or serialization fails.
#[wasm_bindgen(js_name = predictMoleculeWithElementRules)]
pub fn predict_molecule_with_element_rules(
    molecule_json: &str,
    predictor_json: &str,
) -> std::result::Result<String, JsValue> {
    predict_molecule_with_element_rules_json(molecule_json, predictor_json)
        .map_err(|error| js_error(&error))
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

/// Renders a serialized two-dimensional prediction as a spectrum.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, validation,
/// rendering, or serialization fails.
#[wasm_bindgen(js_name = renderPrediction2d)]
pub fn render_prediction_2d(
    prediction_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    render_prediction_2d_json(prediction_json, options_json).map_err(|error| js_error(&error))
}

pub(crate) fn js_error(error: &RSpinError) -> JsValue {
    JsValue::from_str(&error.to_string())
}
