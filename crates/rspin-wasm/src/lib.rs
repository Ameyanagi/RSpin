//! WebAssembly bindings.

mod analysis;
mod api;
mod contours;
mod io;
mod prediction;
mod processing_1d;
mod processing_2d;
mod simulation;
mod workflow;

use rspin_core::RSpinError;
use wasm_bindgen::prelude::*;

pub use analysis::{
    align_spectra_by_peak_to_matrix_1d, align_spectra_by_zone_to_matrix_2d, bucket_spectra_1d,
    bucket_spectra_2d, bucket_spectrum_1d, bucket_spectrum_2d, cluster_bucket_matrix_1d,
    cluster_bucket_matrix_2d, cluster_spectrum_matrix_1d, cluster_spectrum_matrix_2d,
    cut_cluster_result_at_distance, cut_cluster_result_to_count, detect_consensus_peaks_1d,
    detect_consensus_ranges_1d, detect_consensus_zones_2d, detect_ranges_1d, detect_zones_2d,
    pairwise_bucket_matrix_1d, pairwise_bucket_matrix_2d, pairwise_spectrum_matrix_1d,
    pairwise_spectrum_matrix_2d, pca_bucket_matrix_1d, pca_bucket_matrix_2d,
    pca_spectrum_matrix_1d, pca_spectrum_matrix_2d,
};
pub use api::{
    abs_spectrum_1d_json, abs_spectrum_2d_json, align_spectra_by_peak_to_matrix_1d_json,
    align_spectra_by_zone_to_matrix_2d_json, analyze_spectrum_1d_json, analyze_spectrum_2d_json,
    annotate_spectrum_1d_with_assignments_json, annotate_spectrum_2d_with_assignments_json,
    apply_processing_recipe_1d_json, apply_processing_recipe_1d_until_json,
    apply_processing_recipe_2d_json, apply_processing_recipe_2d_until_json,
    auto_phase_spectrum_1d_json, auto_phase_spectrum_2d_json, bucket_spectra_1d_json,
    bucket_spectra_2d_json, bucket_spectrum_1d_json, bucket_spectrum_2d_json,
    cluster_bucket_matrix_1d_json, cluster_bucket_matrix_2d_json, cluster_spectrum_matrix_1d_json,
    cluster_spectrum_matrix_2d_json, create_spectrum_bundle_json, crop_spectrum_1d_json,
    crop_spectrum_2d_json, cut_cluster_result_at_distance_json, cut_cluster_result_to_count_json,
    decompose_exact_spin_half_spectrum_2d_json, decompose_exact_spin_half_spectrum_json,
    detect_consensus_peaks_1d_json, detect_consensus_ranges_1d_json,
    detect_consensus_zones_2d_json, detect_multiplets_json, detect_ranges_json, detect_zones_json,
    exponential_apodization_spectrum_1d_json, exponential_apodization_spectrum_2d_json,
    extract_contours_2d_json, fft_spectrum_1d_json, fft_spectrum_2d_json,
    gauss_multiply_bruker_apodization_spectrum_1d_json, gaussian_apodization_spectrum_1d_json,
    gaussian_apodization_spectrum_2d_json, generate_spectrum_matrix_1d_json,
    generate_spectrum_matrix_2d_json, inspect_agilent_procpar_json,
    inspect_bruker_parameter_file_json, inspect_jeol_jdf_bytes_json, inspect_nmrml_document_json,
    integrate_ranges_json, integrate_region_2d_json, integrate_region_json,
    integrate_regions_2d_json, integrate_regions_json, integrate_zones_2d_json,
    lorentz_to_gauss_apodization_spectrum_1d_json, magnitude_spectrum_1d_json,
    nmredata_1d_signals_to_assignment_set_json, nmredata_2d_signals_to_assignment_set_json,
    nmredata_assignments_to_assignment_set_json, nmredata_couplings_to_j_coupling_graph_json,
    nmredata_to_analysis_json, normalize_spectrum_1d_area_json, normalize_spectrum_1d_json,
    normalize_spectrum_2d_json, normalize_spectrum_2d_volume_json, offset_spectrum_1d_json,
    offset_spectrum_2d_json, optimize_peaks_json, pairwise_bucket_matrix_1d_json,
    pairwise_bucket_matrix_2d_json, pairwise_spectrum_matrix_1d_json,
    pairwise_spectrum_matrix_2d_json, parse_agilent_arrayed_fid_1d_bytes_json,
    parse_agilent_arrayed_fid_2d_bytes_json, parse_agilent_fid_1d_bytes_json,
    parse_agilent_fid_2d_bytes_json, parse_agilent_processed_1d_bytes_json,
    parse_agilent_processed_2d_bytes_json, parse_bruker_fid_1d_bytes_json,
    parse_bruker_processed_1d_bytes_json, parse_bruker_processed_2d_bytes_json,
    parse_bruker_ser_2d_bytes_json, parse_exact_transitions_csv_json, parse_jcamp_dx_1d_json,
    parse_jcamp_dx_2d_json, parse_jcamp_dx_version_json, parse_jeol_jdf_1d_bytes_json,
    parse_jeol_jdf_2d_bytes_json, parse_nmredata_json, parse_nmredata_records_json,
    parse_nmrml_1d_json, parse_nmrml_2d_json, parse_nmrml_version_json, parse_prediction_csv_json,
    parse_spectrum_1d_bytes_as_json, parse_spectrum_1d_csv_json, parse_spectrum_1d_text_as_json,
    parse_spectrum_1d_text_json, parse_spectrum_2d_bytes_as_json, parse_spectrum_2d_csv_json,
    parse_spectrum_2d_text_as_json, parse_spectrum_2d_text_json, pca_bucket_matrix_1d_json,
    pca_bucket_matrix_2d_json, pca_spectrum_matrix_1d_json, pca_spectrum_matrix_2d_json,
    phase_spectrum_1d_json, phase_spectrum_2d_json, pick_peaks_json,
    predict_formula_with_element_rules_json, predict_molecule_with_element_rules_json,
    project_spectrum_2d_x_json, project_spectrum_2d_y_json, render_prediction_1d_json,
    render_prediction_2d_json, resample_spectrum_1d_json, resample_spectrum_2d_json,
    scale_spectrum_1d_json, scale_spectrum_2d_json, shift_spectrum_1d_axis_json,
    shift_spectrum_2d_axes_json, simulate_exact_spin_half_spectrum_2d_json,
    simulate_exact_spin_half_spectrum_json, simulate_exact_spin_half_transitions_json,
    sine_bell_apodization_spectrum_1d_json, sine_bell_apodization_spectrum_2d_json,
    slice_spectrum_2d_x_at_y_index_json, slice_spectrum_2d_x_at_y_json,
    slice_spectrum_2d_y_at_x_index_json, slice_spectrum_2d_y_at_x_json,
    spectrum_bundle_counts_json, spectrum_bundle_only_1d_json, spectrum_bundle_only_2d_json,
    subtract_baseline_spectrum_1d_json, summarize_signals_1d_json, summarize_signals_2d_json,
    traf_apodization_spectrum_1d_json, trapezoidal_apodization_spectrum_1d_json,
    validate_assignment_set_json, validate_exact_spectrum_2d_options_json,
    validate_exact_spectrum_options_json, validate_exact_spin_half_system_json,
    validate_exact_spin_options_json, validate_j_coupling_graph_json, validate_prediction_json,
    validate_spectrum_bundle_json, write_analysis_1d_csv_json, write_analysis_2d_csv_json,
    write_exact_transitions_csv_json, write_jcamp_dx_1d_json, write_jcamp_dx_2d_json,
    write_nmredata_json, write_nmredata_records_json, write_nmrml_1d_json, write_nmrml_2d_json,
    write_prediction_csv_json, write_spectrum_1d_csv_json, write_spectrum_1d_text_json,
    write_spectrum_2d_csv_json, write_spectrum_2d_text_json, zero_fill_spectrum_1d_json,
    zero_fill_spectrum_2d_json,
};
pub use contours::extract_contours_2d;
pub use io::{
    create_spectrum_bundle, inspect_agilent_procpar, inspect_bruker_parameter_file,
    inspect_jeol_jdf_bytes, inspect_nmrml_document, nmredata_1d_signals_to_assignment_set,
    nmredata_2d_signals_to_assignment_set, nmredata_assignments_to_assignment_set,
    nmredata_couplings_to_j_coupling_graph, nmredata_to_analysis,
    parse_agilent_arrayed_fid_1d_bytes, parse_agilent_arrayed_fid_2d_bytes,
    parse_agilent_fid_1d_bytes, parse_agilent_fid_2d_bytes, parse_agilent_processed_1d_bytes,
    parse_agilent_processed_2d_bytes, parse_bruker_fid_1d_bytes, parse_bruker_processed_1d_bytes,
    parse_bruker_processed_2d_bytes, parse_bruker_ser_2d_bytes, parse_jcamp_dx_2d,
    parse_jcamp_dx_version, parse_jeol_jdf_1d_bytes, parse_jeol_jdf_2d_bytes, parse_nmredata,
    parse_nmredata_records, parse_nmrml_1d, parse_nmrml_2d, parse_nmrml_version,
    parse_spectrum_1d_bytes_as, parse_spectrum_1d_csv, parse_spectrum_1d_text,
    parse_spectrum_1d_text_as, parse_spectrum_2d_bytes_as, parse_spectrum_2d_csv,
    parse_spectrum_2d_text, parse_spectrum_2d_text_as, spectrum_bundle_counts,
    spectrum_bundle_only_1d, spectrum_bundle_only_2d, validate_spectrum_bundle,
    write_analysis_1d_csv, write_analysis_2d_csv, write_jcamp_dx_2d, write_nmredata,
    write_nmredata_records, write_nmrml_1d, write_nmrml_2d, write_spectrum_1d_csv,
    write_spectrum_1d_text, write_spectrum_2d_csv, write_spectrum_2d_text,
};
pub use prediction::{
    parse_prediction_csv, predict_formula_with_element_rules, predict_molecule_with_element_rules,
    render_prediction_1d, render_prediction_2d, validate_prediction, write_prediction_csv,
};
pub use processing_1d::{
    abs_spectrum_1d, apply_processing_recipe_1d, apply_processing_recipe_1d_until,
    auto_phase_spectrum_1d, crop_spectrum_1d, exponential_apodization_spectrum_1d, fft_spectrum_1d,
    gaussian_apodization_spectrum_1d, magnitude_spectrum_1d, normalize_spectrum_1d,
    normalize_spectrum_1d_area, offset_spectrum_1d, phase_spectrum_1d, resample_spectrum_1d,
    scale_spectrum_1d, shift_spectrum_1d_axis, sine_bell_apodization_spectrum_1d,
    subtract_baseline_spectrum_1d, zero_fill_spectrum_1d,
};
pub use processing_2d::{
    abs_spectrum_2d, apply_processing_recipe_2d, apply_processing_recipe_2d_until,
    auto_phase_spectrum_2d, crop_spectrum_2d, exponential_apodization_spectrum_2d, fft_spectrum_2d,
    gaussian_apodization_spectrum_2d, normalize_spectrum_2d, normalize_spectrum_2d_volume,
    offset_spectrum_2d, phase_spectrum_2d, project_spectrum_2d_x, project_spectrum_2d_y,
    resample_spectrum_2d, scale_spectrum_2d, shift_spectrum_2d_axes,
    sine_bell_apodization_spectrum_2d, slice_spectrum_2d_x_at_y, slice_spectrum_2d_x_at_y_index,
    slice_spectrum_2d_y_at_x, slice_spectrum_2d_y_at_x_index, zero_fill_spectrum_2d,
};
pub use simulation::{
    decompose_exact_spin_half_spectrum, decompose_exact_spin_half_spectrum_2d,
    parse_exact_transitions_csv, simulate_exact_spin_half_spectrum,
    simulate_exact_spin_half_spectrum_2d, simulate_exact_spin_half_transitions,
    validate_exact_spectrum_2d_options, validate_exact_spectrum_options,
    validate_exact_spin_half_system, validate_exact_spin_options, write_exact_transitions_csv,
};
pub use workflow::{analyze_spectrum_1d, analyze_spectrum_2d};

/// Parses JCAMP-DX text into a serialized one-dimensional spectrum.
///
/// # Errors
///
/// Returns a JavaScript error string when parsing or serialization fails.
#[wasm_bindgen(js_name = parseJcampDx1d)]
pub fn parse_jcamp_dx_1d(input: &str) -> std::result::Result<String, JsValue> {
    parse_jcamp_dx_1d_json(input).map_err(|error| js_error(&error))
}

/// Serializes one-dimensional spectrum JSON into JCAMP-DX text.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization or serialization fails.
#[wasm_bindgen(js_name = writeJcampDx1d)]
pub fn write_jcamp_dx_1d(spectrum_json: &str) -> std::result::Result<String, JsValue> {
    write_jcamp_dx_1d_json(spectrum_json).map_err(|error| js_error(&error))
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

/// Integrates a serialized one-dimensional spectrum over regions.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, analysis, or
/// serialization fails.
#[wasm_bindgen(js_name = integrateRegions1d)]
pub fn integrate_regions_1d(
    spectrum_json: &str,
    regions_json: &str,
) -> std::result::Result<String, JsValue> {
    integrate_regions_json(spectrum_json, regions_json).map_err(|error| js_error(&error))
}

/// Integrates a serialized one-dimensional spectrum over detected ranges.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, analysis, or
/// serialization fails.
#[wasm_bindgen(js_name = integrateRanges1d)]
pub fn integrate_ranges_1d(
    spectrum_json: &str,
    ranges_json: &str,
) -> std::result::Result<String, JsValue> {
    integrate_ranges_json(spectrum_json, ranges_json).map_err(|error| js_error(&error))
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

/// Integrates a serialized two-dimensional spectrum over rectangular regions.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, analysis, or
/// serialization fails.
#[wasm_bindgen(js_name = integrateRegions2d)]
pub fn integrate_regions_2d(
    spectrum_json: &str,
    regions_json: &str,
) -> std::result::Result<String, JsValue> {
    integrate_regions_2d_json(spectrum_json, regions_json).map_err(|error| js_error(&error))
}

/// Integrates a serialized two-dimensional spectrum over detected zones.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, analysis, or
/// serialization fails.
#[wasm_bindgen(js_name = integrateZones2d)]
pub fn integrate_zones_2d(
    spectrum_json: &str,
    zones_json: &str,
) -> std::result::Result<String, JsValue> {
    integrate_zones_2d_json(spectrum_json, zones_json).map_err(|error| js_error(&error))
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

pub(crate) fn js_error(error: &RSpinError) -> JsValue {
    JsValue::from_str(&error.to_string())
}
