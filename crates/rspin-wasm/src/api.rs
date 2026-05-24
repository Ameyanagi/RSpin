//! JSON-oriented API helpers for WASM bindings.

mod assignments;
mod buckets;
mod clustering;
mod consensus;
mod contours;
mod csv_io;
mod matrix;
mod pairwise;
mod pca;
mod prediction;
mod processing_1d;
mod processing_2d;
mod simulation;
mod workflow;

use serde::{Serialize, de::DeserializeOwned};

use rspin_analysis::{
    DetectedMultiplet, DetectedRange, DetectedZone, IntegralRegion, IntegralRegion2D,
    MultipletDetectionOptions, PeakOptimizationOptions, PeakPickOptions, RangeDetectionOptions,
    SignalSummary2DOptions, SignalSummaryOptions, ZoneDetectionOptions, detect_multiplets,
    detect_ranges, detect_zones, integrate_ranges, integrate_region, integrate_region_2d,
    integrate_regions, integrate_regions_2d, integrate_zones_2d, optimize_peaks_quadratic,
    pick_peaks, summarize_signals_1d, summarize_signals_2d,
};
use rspin_core::{Nucleus, RSpinError, Result, Spectrum1D, Spectrum2D};
use rspin_io::{
    inspect_agilent_procpar, inspect_bruker_parameter_file, inspect_jeol_jdf_bytes,
    parse_jcamp_dx_version, parse_spectrum_text_format, parse_spectrum1d_write_format,
    parse_spectrum2d_write_format, read_assignment_set_json, read_j_coupling_graph_json,
    read_jcamp_dx_1d, read_nmredata_record_json, read_nmredata_records_json,
    read_nmredata_records_str, read_nmredata_str, read_nmrml_1d_str, read_nmrml_2d_str,
    read_nmrml_document_info_str, read_spectrum1d_json, read_spectrum1d_text,
    read_spectrum1d_text_as, read_spectrum2d_json, read_spectrum2d_text, read_spectrum2d_text_as,
    write_assignment_set_json, write_j_coupling_graph_json, write_jcamp_dx_1d,
    write_nmredata_record, write_nmredata_record_json, write_nmredata_records,
    write_nmredata_records_json as write_nmredata_records_json_io, write_nmrml_1d, write_nmrml_2d,
    write_spectrum1d_json, write_spectrum1d_text, write_spectrum2d_json, write_spectrum2d_text,
};
use rspin_processing::{AutoPhaseOptions, auto_phase_correct, normalize_max_abs, scale_intensity};

pub use assignments::{
    annotate_spectrum_1d_with_assignments_json, annotate_spectrum_2d_with_assignments_json,
    validate_assignment_set_json,
};
pub use buckets::{
    bucket_spectra_1d_json, bucket_spectra_2d_json, bucket_spectrum_1d_json,
    bucket_spectrum_2d_json,
};
pub use clustering::{
    cluster_bucket_matrix_1d_json, cluster_bucket_matrix_2d_json, cluster_spectrum_matrix_1d_json,
    cluster_spectrum_matrix_2d_json, cut_cluster_result_at_distance_json,
    cut_cluster_result_to_count_json,
};
pub use consensus::{
    detect_consensus_peaks_1d_json, detect_consensus_ranges_1d_json, detect_consensus_zones_2d_json,
};
pub use contours::extract_contours_2d_json;
pub use csv_io::{
    parse_spectrum_1d_csv_json, parse_spectrum_2d_csv_json, write_analysis_1d_csv_json,
    write_analysis_2d_csv_json, write_spectrum_1d_csv_json, write_spectrum_2d_csv_json,
};
pub use matrix::{
    align_spectra_by_peak_to_matrix_1d_json, align_spectra_by_zone_to_matrix_2d_json,
    generate_spectrum_matrix_1d_json, generate_spectrum_matrix_2d_json,
};
pub use pairwise::{
    pairwise_bucket_matrix_1d_json, pairwise_bucket_matrix_2d_json,
    pairwise_spectrum_matrix_1d_json, pairwise_spectrum_matrix_2d_json,
};
pub use pca::{
    pca_bucket_matrix_1d_json, pca_bucket_matrix_2d_json, pca_spectrum_matrix_1d_json,
    pca_spectrum_matrix_2d_json,
};
pub use prediction::{
    parse_prediction_csv_json, predict_formula_with_element_rules_json,
    predict_molecule_with_element_rules_json, render_prediction_1d_json, render_prediction_2d_json,
    validate_prediction_json, write_prediction_csv_json,
};
pub use processing_1d::{
    abs_spectrum_1d_json, apply_processing_recipe_1d_json, apply_processing_recipe_1d_until_json,
    crop_spectrum_1d_json, exponential_apodization_spectrum_1d_json, fft_spectrum_1d_json,
    gaussian_apodization_spectrum_1d_json, magnitude_spectrum_1d_json,
    normalize_spectrum_1d_area_json, offset_spectrum_1d_json, phase_spectrum_1d_json,
    resample_spectrum_1d_json, shift_spectrum_1d_axis_json, sine_bell_apodization_spectrum_1d_json,
    subtract_baseline_spectrum_1d_json, zero_fill_spectrum_1d_json,
};
pub use processing_2d::{
    abs_spectrum_2d_json, apply_processing_recipe_2d_json, apply_processing_recipe_2d_until_json,
    auto_phase_spectrum_2d_json, crop_spectrum_2d_json, exponential_apodization_spectrum_2d_json,
    fft_spectrum_2d_json, gaussian_apodization_spectrum_2d_json, normalize_spectrum_2d_json,
    normalize_spectrum_2d_volume_json, phase_spectrum_2d_json, project_spectrum_2d_x_json,
    project_spectrum_2d_y_json, resample_spectrum_2d_json, scale_spectrum_2d_json,
    sine_bell_apodization_spectrum_2d_json, slice_spectrum_2d_x_at_y_index_json,
    slice_spectrum_2d_x_at_y_json, slice_spectrum_2d_y_at_x_index_json,
    slice_spectrum_2d_y_at_x_json, zero_fill_spectrum_2d_json,
};
pub use simulation::{
    decompose_exact_spin_half_spectrum_2d_json, decompose_exact_spin_half_spectrum_json,
    parse_exact_transitions_csv_json, simulate_exact_spin_half_spectrum_2d_json,
    simulate_exact_spin_half_spectrum_json, simulate_exact_spin_half_transitions_json,
    validate_exact_spectrum_2d_options_json, validate_exact_spectrum_options_json,
    validate_exact_spin_half_system_json, validate_exact_spin_options_json,
    write_exact_transitions_csv_json,
};
pub use workflow::{analyze_spectrum_1d_json, analyze_spectrum_2d_json};

/// Parses JCAMP-DX text into serialized `Spectrum1D` JSON.
///
/// # Errors
///
/// Returns an error when parsing or serialization fails.
pub fn parse_jcamp_dx_1d_json(input: &str) -> Result<String> {
    let spectrum = read_jcamp_dx_1d(input)?;
    spectrum1d_to_json(&spectrum)
}

/// Serializes `Spectrum1D` JSON into JCAMP-DX text.
///
/// # Errors
///
/// Returns an error when deserialization or JCAMP-DX serialization fails.
pub fn write_jcamp_dx_1d_json(spectrum_json: &str) -> Result<String> {
    let spectrum = spectrum1d_from_json(spectrum_json)?;
    write_jcamp_dx_1d(&spectrum)
}

/// Parses a JCAMP-DX version label into serialized metadata JSON.
///
/// # Errors
///
/// Returns an error when the version label is malformed.
pub fn parse_jcamp_dx_version_json(input: &str) -> Result<String> {
    let version = parse_jcamp_dx_version(input)?;
    to_json(&version)
}

/// Parses Bruker parameter-file routing metadata into serialized JSON.
///
/// # Errors
///
/// Returns an error when a declared format version is malformed.
pub fn inspect_bruker_parameter_file_json(input: &str) -> Result<String> {
    let info = inspect_bruker_parameter_file(input)?;
    to_json(&info)
}

/// Parses Agilent/Varian `procpar` routing metadata into serialized JSON.
///
/// # Errors
///
/// Returns an error when routing fields contain malformed numeric values.
pub fn inspect_agilent_procpar_json(input: &str) -> Result<String> {
    let info = inspect_agilent_procpar(input)?;
    to_json(&info)
}

/// Parses JEOL Delta `.jdf` header routing metadata into serialized JSON.
///
/// # Errors
///
/// Returns an error when the payload does not contain a valid JDF header.
pub fn inspect_jeol_jdf_bytes_json(input: &[u8]) -> Result<String> {
    let info = inspect_jeol_jdf_bytes(input)?;
    to_json(&info)
}

/// Serializes `Spectrum1D` JSON into the requested text format.
///
/// Supported formats are `json`, `nmrml`, `xml`, `jcamp_dx`, `jdx`, `dx`, and
/// `csv`.
///
/// # Errors
///
/// Returns an error when deserialization, format parsing, or serialization fails.
pub fn write_spectrum_1d_text_json(spectrum_json: &str, format: &str) -> Result<String> {
    let spectrum = spectrum1d_from_json(spectrum_json)?;
    write_spectrum1d_text(&spectrum, parse_spectrum1d_write_format(format)?)
}

/// Serializes `Spectrum2D` JSON into the requested text format.
///
/// Supported formats are `json`, `nmrml`, `xml`, and `csv`.
///
/// # Errors
///
/// Returns an error when deserialization, format parsing, or serialization fails.
pub fn write_spectrum_2d_text_json(spectrum_json: &str, format: &str) -> Result<String> {
    let spectrum = spectrum2d_from_json(spectrum_json)?;
    write_spectrum2d_text(&spectrum, parse_spectrum2d_write_format(format)?)
}

/// Parses nmrML text into serialized `Spectrum1D` JSON.
///
/// # Errors
///
/// Returns an error when parsing or serialization fails.
pub fn parse_nmrml_1d_json(input: &str) -> Result<String> {
    let spectrum = read_nmrml_1d_str(input)?;
    spectrum1d_to_json(&spectrum)
}

/// Parses two-dimensional nmrML text into serialized `Spectrum2D` JSON.
///
/// # Errors
///
/// Returns an error when parsing or serialization fails.
pub fn parse_nmrml_2d_json(input: &str) -> Result<String> {
    let spectrum = read_nmrml_2d_str(input)?;
    spectrum2d_to_json(&spectrum)
}

/// Parses `NMReDATA` SDF text into serialized record JSON.
///
/// # Errors
///
/// Returns an error when parsing or serialization fails.
pub fn parse_nmredata_json(input: &str) -> Result<String> {
    let record = read_nmredata_str(input)?;
    write_nmredata_record_json(&record)
}

/// Parses all `NMReDATA` SDF records into serialized record-list JSON.
///
/// # Errors
///
/// Returns an error when parsing or serialization fails.
pub fn parse_nmredata_records_json(input: &str) -> Result<String> {
    let records = read_nmredata_records_str(input)?;
    write_nmredata_records_json_io(&records)
}

/// Serializes `NMReDATA` record JSON into SDF text.
///
/// # Errors
///
/// Returns an error when deserialization or `NMReDATA` serialization fails.
pub fn write_nmredata_json(record_json: &str) -> Result<String> {
    let record = read_nmredata_record_json(record_json)?;
    write_nmredata_record(&record)
}

/// Serializes `NMReDATA` record-list JSON into SDF text.
///
/// # Errors
///
/// Returns an error when deserialization or `NMReDATA` serialization fails.
pub fn write_nmredata_records_json(records_json: &str) -> Result<String> {
    let records = read_nmredata_records_json(records_json)?;
    write_nmredata_records(&records)
}

/// Converts `NMReDATA` record JSON into serialized [`AssignmentSet`] JSON.
///
/// # Errors
///
/// Returns an error when deserialization, nucleus parsing, conversion, or
/// serialization fails.
pub fn nmredata_assignments_to_assignment_set_json(
    record_json: &str,
    nucleus_label: &str,
) -> Result<String> {
    let record = read_nmredata_record_json(record_json)?;
    let assignments = record.to_assignment_set(parse_nucleus_label(nucleus_label)?)?;
    write_assignment_set_json(&assignments)
}

/// Converts `NMReDATA` 1D signal labels into serialized [`AssignmentSet`] JSON.
///
/// # Errors
///
/// Returns an error when deserialization, nucleus parsing, conversion, or
/// serialization fails.
pub fn nmredata_1d_signals_to_assignment_set_json(
    record_json: &str,
    nucleus_label: &str,
) -> Result<String> {
    let record = read_nmredata_record_json(record_json)?;
    let assignments = record.to_signal_assignment_set(parse_nucleus_label(nucleus_label)?)?;
    write_assignment_set_json(&assignments)
}

/// Converts `NMReDATA` 2D signal labels into serialized [`AssignmentSet`] JSON.
///
/// # Errors
///
/// Returns an error when deserialization, conversion, or serialization fails.
pub fn nmredata_2d_signals_to_assignment_set_json(record_json: &str) -> Result<String> {
    let record = read_nmredata_record_json(record_json)?;
    let assignments = record.to_2d_signal_assignment_set()?;
    write_assignment_set_json(&assignments)
}

/// Converts `NMReDATA` record JSON into serialized [`JCouplingGraph`] JSON.
///
/// # Errors
///
/// Returns an error when deserialization, nucleus parsing, conversion, or
/// serialization fails.
pub fn nmredata_couplings_to_j_coupling_graph_json(
    record_json: &str,
    nucleus_label: &str,
) -> Result<String> {
    let record = read_nmredata_record_json(record_json)?;
    let graph = record.to_j_coupling_graph(parse_nucleus_label(nucleus_label)?)?;
    write_j_coupling_graph_json(&graph)
}

/// Converts `NMReDATA` record JSON into serialized combined analysis JSON.
///
/// # Errors
///
/// Returns an error when deserialization, nucleus parsing, conversion, or
/// serialization fails.
pub fn nmredata_to_analysis_json(record_json: &str, nucleus_label: &str) -> Result<String> {
    let record = read_nmredata_record_json(record_json)?;
    let analysis = record.to_analysis(parse_nucleus_label(nucleus_label)?)?;
    to_json(&analysis)
}

/// Parses root-level nmrML document metadata into JSON.
///
/// # Errors
///
/// Returns an error when parsing or serialization fails.
pub fn inspect_nmrml_document_json(input: &str) -> Result<String> {
    let info = read_nmrml_document_info_str(input)?;
    to_json(&info)
}

/// Serializes `Spectrum1D` JSON into nmrML text.
///
/// # Errors
///
/// Returns an error when deserialization or nmrML serialization fails.
pub fn write_nmrml_1d_json(spectrum_json: &str) -> Result<String> {
    let spectrum = spectrum1d_from_json(spectrum_json)?;
    write_nmrml_1d(&spectrum)
}

/// Serializes `Spectrum2D` JSON into nmrML text.
///
/// # Errors
///
/// Returns an error when deserialization or nmrML serialization fails.
pub fn write_nmrml_2d_json(spectrum_json: &str) -> Result<String> {
    let spectrum = spectrum2d_from_json(spectrum_json)?;
    write_nmrml_2d(&spectrum)
}

/// Parses auto-detected one-dimensional spectrum text into serialized `Spectrum1D` JSON.
///
/// # Errors
///
/// Returns an error when parsing or serialization fails.
pub fn parse_spectrum_1d_text_json(input: &str) -> Result<String> {
    let spectrum = read_spectrum1d_text(input)?;
    spectrum1d_to_json(&spectrum)
}

/// Parses one-dimensional spectrum text in an explicit format into serialized `Spectrum1D` JSON.
///
/// # Errors
///
/// Returns an error when format parsing, spectrum parsing, or serialization fails.
pub fn parse_spectrum_1d_text_as_json(input: &str, format: &str) -> Result<String> {
    let spectrum = read_spectrum1d_text_as(input, parse_spectrum_text_format(format)?)?;
    spectrum1d_to_json(&spectrum)
}

/// Parses auto-detected two-dimensional spectrum text into serialized `Spectrum2D` JSON.
///
/// # Errors
///
/// Returns an error when parsing or serialization fails.
pub fn parse_spectrum_2d_text_json(input: &str) -> Result<String> {
    let spectrum = read_spectrum2d_text(input)?;
    spectrum2d_to_json(&spectrum)
}

/// Parses two-dimensional spectrum text in an explicit format into serialized `Spectrum2D` JSON.
///
/// # Errors
///
/// Returns an error when format parsing, spectrum parsing, or serialization fails.
pub fn parse_spectrum_2d_text_as_json(input: &str, format: &str) -> Result<String> {
    let spectrum = read_spectrum2d_text_as(input, parse_spectrum_text_format(format)?)?;
    spectrum2d_to_json(&spectrum)
}

/// Scales serialized `Spectrum1D` JSON.
///
/// # Errors
///
/// Returns an error when deserialization, processing, or serialization fails.
pub fn scale_spectrum_1d_json(spectrum_json: &str, factor: f64) -> Result<String> {
    let spectrum = spectrum1d_from_json(spectrum_json)?;
    let processed = scale_intensity(&spectrum, factor)?;
    spectrum1d_to_json(&processed)
}

/// Normalizes serialized `Spectrum1D` JSON by maximum absolute intensity.
///
/// # Errors
///
/// Returns an error when deserialization, processing, or serialization fails.
pub fn normalize_spectrum_1d_json(spectrum_json: &str) -> Result<String> {
    let spectrum = spectrum1d_from_json(spectrum_json)?;
    let processed = normalize_max_abs(&spectrum)?;
    spectrum1d_to_json(&processed)
}

/// Automatically phases serialized `Spectrum1D` JSON.
///
/// # Errors
///
/// Returns an error when deserialization, processing, or serialization fails.
pub fn auto_phase_spectrum_1d_json(spectrum_json: &str, options_json: &str) -> Result<String> {
    let spectrum = spectrum1d_from_json(spectrum_json)?;
    let options: AutoPhaseOptionsJson = from_json(options_json)?;
    let result = auto_phase_correct(&spectrum, options.into())?;
    to_json(&AutoPhaseResponseJson {
        spectrum: result.spectrum,
        zero_order_deg: result.zero_order_deg,
        first_order_deg: result.first_order_deg,
        score: result.score,
    })
}

/// Picks peaks from serialized `Spectrum1D` JSON.
///
/// # Errors
///
/// Returns an error when deserialization, analysis, or serialization fails.
pub fn pick_peaks_json(spectrum_json: &str, options_json: &str) -> Result<String> {
    let spectrum = spectrum1d_from_json(spectrum_json)?;
    let options: PeakPickOptions = from_json(options_json)?;
    let peaks = pick_peaks(&spectrum, options)?;
    to_json(&peaks)
}

/// Optimizes serialized peaks from serialized `Spectrum1D` JSON.
///
/// # Errors
///
/// Returns an error when deserialization, analysis, or serialization fails.
pub fn optimize_peaks_json(
    spectrum_json: &str,
    peaks_json: &str,
    options_json: &str,
) -> Result<String> {
    let spectrum = spectrum1d_from_json(spectrum_json)?;
    let peaks: Vec<rspin_analysis::Peak> = from_json(peaks_json)?;
    let options: PeakOptimizationOptions = from_json(options_json)?;
    let optimized = optimize_peaks_quadratic(&spectrum, &peaks, options)?;
    to_json(&optimized)
}

/// Detects serialized multiplets from serialized `Spectrum1D` and peak JSON.
///
/// # Errors
///
/// Returns an error when deserialization, analysis, or serialization fails.
pub fn detect_multiplets_json(
    spectrum_json: &str,
    peaks_json: &str,
    options_json: &str,
) -> Result<String> {
    let spectrum = spectrum1d_from_json(spectrum_json)?;
    let peaks: Vec<rspin_analysis::Peak> = from_json(peaks_json)?;
    let options: MultipletDetectionOptions = from_json(options_json)?;
    let multiplets = detect_multiplets(&spectrum, &peaks, options)?;
    to_json(&multiplets)
}

/// Detects ranges from serialized `Spectrum1D` JSON.
///
/// # Errors
///
/// Returns an error when deserialization, analysis, or serialization fails.
pub fn detect_ranges_json(spectrum_json: &str, options_json: &str) -> Result<String> {
    let spectrum = spectrum1d_from_json(spectrum_json)?;
    let options: RangeDetectionOptions = from_json(options_json)?;
    let ranges = detect_ranges(&spectrum, options)?;
    to_json(&ranges)
}

/// Detects connected zones from serialized `Spectrum2D` JSON.
///
/// # Errors
///
/// Returns an error when deserialization, analysis, or serialization fails.
pub fn detect_zones_json(spectrum_json: &str, options_json: &str) -> Result<String> {
    let spectrum = spectrum2d_from_json(spectrum_json)?;
    let options: ZoneDetectionOptions = from_json(options_json)?;
    let zones = detect_zones(&spectrum, options)?;
    to_json(&zones)
}

/// Validates serialized J-coupling graph JSON and returns normalized JSON.
///
/// # Errors
///
/// Returns an error when deserialization, validation, or serialization fails.
pub fn validate_j_coupling_graph_json(graph_json: &str) -> Result<String> {
    let graph = read_j_coupling_graph_json(graph_json)?;
    graph.validate()?;
    write_j_coupling_graph_json(&graph)
}

/// Assembles one-dimensional signal summary JSON from analysis payloads.
///
/// # Errors
///
/// Returns an error when deserialization, validation, analysis, or serialization fails.
pub fn summarize_signals_1d_json(
    spectrum_json: &str,
    ranges_json: &str,
    multiplets_json: &str,
    assignments_json: &str,
    coupling_graph_json: &str,
    options_json: &str,
) -> Result<String> {
    let spectrum = spectrum1d_from_json(spectrum_json)?;
    let ranges: Vec<DetectedRange> = from_json(ranges_json)?;
    let multiplets: Vec<DetectedMultiplet> = from_json(multiplets_json)?;
    let assignments = read_assignment_set_json(assignments_json)?;
    let coupling_graph = read_j_coupling_graph_json(coupling_graph_json)?;
    let options: SignalSummaryOptions = from_json(options_json)?;
    let signals = summarize_signals_1d(
        &spectrum,
        &ranges,
        &multiplets,
        &assignments,
        &coupling_graph,
        options,
    )?;
    to_json(&signals)
}

/// Assembles two-dimensional signal summary JSON from zone and assignment payloads.
///
/// # Errors
///
/// Returns an error when deserialization, validation, analysis, or serialization fails.
pub fn summarize_signals_2d_json(
    spectrum_json: &str,
    zones_json: &str,
    assignments_json: &str,
    options_json: &str,
) -> Result<String> {
    let spectrum = spectrum2d_from_json(spectrum_json)?;
    let zones: Vec<DetectedZone> = from_json(zones_json)?;
    let assignments = read_assignment_set_json(assignments_json)?;
    let options: SignalSummary2DOptions = from_json(options_json)?;
    let signals = summarize_signals_2d(&spectrum, &zones, &assignments, options)?;
    to_json(&signals)
}

/// Integrates serialized `Spectrum1D` JSON over a serialized region.
///
/// # Errors
///
/// Returns an error when deserialization, analysis, or serialization fails.
pub fn integrate_region_json(spectrum_json: &str, region_json: &str) -> Result<String> {
    let spectrum = spectrum1d_from_json(spectrum_json)?;
    let region: IntegralRegion = from_json(region_json)?;
    let integral = integrate_region(&spectrum, region)?;
    to_json(&integral)
}

/// Integrates serialized `Spectrum1D` JSON over serialized regions.
///
/// # Errors
///
/// Returns an error when deserialization, analysis, or serialization fails.
pub fn integrate_regions_json(spectrum_json: &str, regions_json: &str) -> Result<String> {
    let spectrum = spectrum1d_from_json(spectrum_json)?;
    let regions: Vec<IntegralRegion> = from_json(regions_json)?;
    let integrals = integrate_regions(&spectrum, &regions)?;
    to_json(&integrals)
}

/// Integrates serialized `Spectrum1D` JSON over serialized detected ranges.
///
/// # Errors
///
/// Returns an error when deserialization, analysis, or serialization fails.
pub fn integrate_ranges_json(spectrum_json: &str, ranges_json: &str) -> Result<String> {
    let spectrum = spectrum1d_from_json(spectrum_json)?;
    let ranges: Vec<DetectedRange> = from_json(ranges_json)?;
    let integrals = integrate_ranges(&spectrum, &ranges)?;
    to_json(&integrals)
}

/// Integrates serialized `Spectrum2D` JSON over a serialized rectangular region.
///
/// # Errors
///
/// Returns an error when deserialization, analysis, or serialization fails.
pub fn integrate_region_2d_json(spectrum_json: &str, region_json: &str) -> Result<String> {
    let spectrum = spectrum2d_from_json(spectrum_json)?;
    let region: IntegralRegion2D = from_json(region_json)?;
    let integral = integrate_region_2d(&spectrum, region)?;
    to_json(&integral)
}

/// Integrates serialized `Spectrum2D` JSON over serialized rectangular regions.
///
/// # Errors
///
/// Returns an error when deserialization, analysis, or serialization fails.
pub fn integrate_regions_2d_json(spectrum_json: &str, regions_json: &str) -> Result<String> {
    let spectrum = spectrum2d_from_json(spectrum_json)?;
    let regions: Vec<IntegralRegion2D> = from_json(regions_json)?;
    let integrals = integrate_regions_2d(&spectrum, &regions)?;
    to_json(&integrals)
}

/// Integrates serialized `Spectrum2D` JSON over serialized detected zones.
///
/// # Errors
///
/// Returns an error when deserialization, analysis, or serialization fails.
pub fn integrate_zones_2d_json(spectrum_json: &str, zones_json: &str) -> Result<String> {
    let spectrum = spectrum2d_from_json(spectrum_json)?;
    let zones: Vec<DetectedZone> = from_json(zones_json)?;
    let integrals = integrate_zones_2d(&spectrum, &zones)?;
    to_json(&integrals)
}

fn from_json<T: DeserializeOwned>(input: &str) -> Result<T> {
    serde_json::from_str(input).map_err(|error| RSpinError::Parse {
        format: "JSON",
        message: error.to_string(),
    })
}

fn to_json<T: Serialize>(value: &T) -> Result<String> {
    serde_json::to_string(value).map_err(|error| RSpinError::Parse {
        format: "JSON",
        message: error.to_string(),
    })
}

fn spectrum1d_from_json(input: &str) -> Result<Spectrum1D> {
    read_spectrum1d_json(input)
}

fn spectrum2d_from_json(input: &str) -> Result<Spectrum2D> {
    read_spectrum2d_json(input)
}

fn spectrum1d_to_json(spectrum: &Spectrum1D) -> Result<String> {
    write_spectrum1d_json(spectrum)
}

fn spectrum2d_to_json(spectrum: &Spectrum2D) -> Result<String> {
    write_spectrum2d_json(spectrum)
}

fn parse_nucleus_label(label: &str) -> Result<Nucleus> {
    label.parse()
}

#[derive(Clone, Copy, Debug, serde::Deserialize)]
#[serde(default)]
struct AutoPhaseOptionsJson {
    zero_order_min_deg: f64,
    zero_order_max_deg: f64,
    zero_order_step_deg: f64,
    first_order_min_deg: f64,
    first_order_max_deg: f64,
    first_order_step_deg: f64,
    pivot_fraction: f64,
    imaginary_weight: f64,
    negative_weight: f64,
}

impl Default for AutoPhaseOptionsJson {
    fn default() -> Self {
        let options = AutoPhaseOptions::default();
        Self {
            zero_order_min_deg: options.zero_order_min_deg,
            zero_order_max_deg: options.zero_order_max_deg,
            zero_order_step_deg: options.zero_order_step_deg,
            first_order_min_deg: options.first_order_min_deg,
            first_order_max_deg: options.first_order_max_deg,
            first_order_step_deg: options.first_order_step_deg,
            pivot_fraction: options.pivot_fraction,
            imaginary_weight: options.imaginary_weight,
            negative_weight: options.negative_weight,
        }
    }
}

impl From<AutoPhaseOptionsJson> for AutoPhaseOptions {
    fn from(options: AutoPhaseOptionsJson) -> Self {
        Self {
            zero_order_min_deg: options.zero_order_min_deg,
            zero_order_max_deg: options.zero_order_max_deg,
            zero_order_step_deg: options.zero_order_step_deg,
            first_order_min_deg: options.first_order_min_deg,
            first_order_max_deg: options.first_order_max_deg,
            first_order_step_deg: options.first_order_step_deg,
            pivot_fraction: options.pivot_fraction,
            imaginary_weight: options.imaginary_weight,
            negative_weight: options.negative_weight,
        }
    }
}

#[derive(Clone, Debug, Serialize, serde::Deserialize)]
struct AutoPhaseResponseJson {
    spectrum: Spectrum1D,
    zero_order_deg: f64,
    first_order_deg: f64,
    score: f64,
}

#[cfg(test)]
mod tests;
