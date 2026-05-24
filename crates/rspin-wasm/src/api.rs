//! JSON-oriented API helpers for WASM bindings.

mod assignments;
mod buckets;
mod contours;
mod csv_io;
mod pairwise;
mod pca;
mod prediction;
mod processing_1d;
mod processing_2d;
mod simulation;

use serde::{Serialize, de::DeserializeOwned};

use rspin_analysis::{
    AssignmentSet, DetectedMultiplet, DetectedRange, DetectedZone, IntegralRegion,
    IntegralRegion2D, JCouplingGraph, MatrixGeneration2DOptions, MatrixGenerationOptions,
    MultipletDetectionOptions, PeakAlignmentOptions, PeakOptimizationOptions, PeakPickOptions,
    RangeDetectionOptions, SignalSummary2DOptions, SignalSummaryOptions, ZoneDetectionOptions,
    align_spectra_by_peak_to_matrix, detect_multiplets, detect_ranges, detect_zones,
    generate_spectrum_matrix_1d, generate_spectrum_matrix_2d, integrate_region,
    integrate_region_2d, optimize_peaks_quadratic, pick_peaks, summarize_signals_1d,
    summarize_signals_2d,
};
use rspin_core::{RSpinError, Result, Spectrum1D, Spectrum2D};
use rspin_io::read_jcamp_dx_1d;
use rspin_processing::{AutoPhaseOptions, auto_phase_correct, normalize_max_abs, scale_intensity};

pub use assignments::{
    annotate_spectrum_1d_with_assignments_json, annotate_spectrum_2d_with_assignments_json,
    validate_assignment_set_json,
};
pub use buckets::{
    bucket_spectra_1d_json, bucket_spectra_2d_json, bucket_spectrum_1d_json,
    bucket_spectrum_2d_json,
};
pub use contours::extract_contours_2d_json;
pub use csv_io::{
    parse_spectrum_1d_csv_json, parse_spectrum_2d_csv_json, write_spectrum_1d_csv_json,
    write_spectrum_2d_csv_json,
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
    predict_molecule_with_element_rules_json, render_prediction_1d_json, render_prediction_2d_json,
    validate_prediction_json,
};
pub use processing_1d::{
    abs_spectrum_1d_json, apply_processing_recipe_1d_json, apply_processing_recipe_1d_until_json,
    crop_spectrum_1d_json, exponential_apodization_spectrum_1d_json, fft_spectrum_1d_json,
    magnitude_spectrum_1d_json, offset_spectrum_1d_json, phase_spectrum_1d_json,
    resample_spectrum_1d_json, shift_spectrum_1d_axis_json, subtract_baseline_spectrum_1d_json,
    zero_fill_spectrum_1d_json,
};
pub use processing_2d::{
    abs_spectrum_2d_json, apply_processing_recipe_2d_json, apply_processing_recipe_2d_until_json,
    auto_phase_spectrum_2d_json, crop_spectrum_2d_json, fft_spectrum_2d_json,
    normalize_spectrum_2d_json, phase_spectrum_2d_json, project_spectrum_2d_x_json,
    project_spectrum_2d_y_json, resample_spectrum_2d_json, scale_spectrum_2d_json,
    slice_spectrum_2d_x_at_y_index_json, slice_spectrum_2d_x_at_y_json,
    slice_spectrum_2d_y_at_x_index_json, slice_spectrum_2d_y_at_x_json, zero_fill_spectrum_2d_json,
};
#[cfg(feature = "first-order")]
pub use simulation::simulate_first_order_multiplet_json;
pub use simulation::{
    decompose_exact_spin_half_spectrum_json, simulate_exact_spin_half_spectrum_json,
    simulate_exact_spin_half_transitions_json,
};

/// Parses JCAMP-DX text into serialized `Spectrum1D` JSON.
///
/// # Errors
///
/// Returns an error when parsing or serialization fails.
pub fn parse_jcamp_dx_1d_json(input: &str) -> Result<String> {
    let spectrum = read_jcamp_dx_1d(input)?;
    to_json(&spectrum)
}

/// Scales serialized `Spectrum1D` JSON.
///
/// # Errors
///
/// Returns an error when deserialization, processing, or serialization fails.
pub fn scale_spectrum_1d_json(spectrum_json: &str, factor: f64) -> Result<String> {
    let spectrum: Spectrum1D = from_json(spectrum_json)?;
    let processed = scale_intensity(&spectrum, factor)?;
    to_json(&processed)
}

/// Normalizes serialized `Spectrum1D` JSON by maximum absolute intensity.
///
/// # Errors
///
/// Returns an error when deserialization, processing, or serialization fails.
pub fn normalize_spectrum_1d_json(spectrum_json: &str) -> Result<String> {
    let spectrum: Spectrum1D = from_json(spectrum_json)?;
    let processed = normalize_max_abs(&spectrum)?;
    to_json(&processed)
}

/// Automatically phases serialized `Spectrum1D` JSON.
///
/// # Errors
///
/// Returns an error when deserialization, processing, or serialization fails.
pub fn auto_phase_spectrum_1d_json(spectrum_json: &str, options_json: &str) -> Result<String> {
    let spectrum: Spectrum1D = from_json(spectrum_json)?;
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
    let spectrum: Spectrum1D = from_json(spectrum_json)?;
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
    let spectrum: Spectrum1D = from_json(spectrum_json)?;
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
    let spectrum: Spectrum1D = from_json(spectrum_json)?;
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
    let spectrum: Spectrum1D = from_json(spectrum_json)?;
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
    let spectrum: Spectrum2D = from_json(spectrum_json)?;
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
    let graph: JCouplingGraph = from_json(graph_json)?;
    graph.validate()?;
    to_json(&graph)
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
    let spectrum: Spectrum1D = from_json(spectrum_json)?;
    let ranges: Vec<DetectedRange> = from_json(ranges_json)?;
    let multiplets: Vec<DetectedMultiplet> = from_json(multiplets_json)?;
    let assignments: AssignmentSet = from_json(assignments_json)?;
    let coupling_graph: JCouplingGraph = from_json(coupling_graph_json)?;
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
    let spectrum: Spectrum2D = from_json(spectrum_json)?;
    let zones: Vec<DetectedZone> = from_json(zones_json)?;
    let assignments: AssignmentSet = from_json(assignments_json)?;
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
    let spectrum: Spectrum1D = from_json(spectrum_json)?;
    let region: IntegralRegion = from_json(region_json)?;
    let integral = integrate_region(&spectrum, region)?;
    to_json(&integral)
}

/// Integrates serialized `Spectrum2D` JSON over a serialized rectangular region.
///
/// # Errors
///
/// Returns an error when deserialization, analysis, or serialization fails.
pub fn integrate_region_2d_json(spectrum_json: &str, region_json: &str) -> Result<String> {
    let spectrum: Spectrum2D = from_json(spectrum_json)?;
    let region: IntegralRegion2D = from_json(region_json)?;
    let integral = integrate_region_2d(&spectrum, region)?;
    to_json(&integral)
}

/// Generates a row-major matrix from serialized `Spectrum1D` JSON values.
///
/// # Errors
///
/// Returns an error when deserialization, analysis, or serialization fails.
pub fn generate_spectrum_matrix_1d_json(spectra_json: &str, options_json: &str) -> Result<String> {
    let spectra: Vec<Spectrum1D> = from_json(spectra_json)?;
    let options: MatrixGenerationOptions = from_json(options_json)?;
    let matrix = generate_spectrum_matrix_1d(&spectra, options)?;
    to_json(&matrix)
}

/// Aligns serialized `Spectrum1D` JSON values by peak and generates a matrix.
///
/// # Errors
///
/// Returns an error when deserialization, alignment, matrix generation, or
/// serialization fails.
pub fn align_spectra_by_peak_to_matrix_1d_json(
    spectra_json: &str,
    alignment_options_json: &str,
    matrix_options_json: &str,
) -> Result<String> {
    let spectra: Vec<Spectrum1D> = from_json(spectra_json)?;
    let alignment_options: PeakAlignmentOptions = from_json(alignment_options_json)?;
    let matrix_options: MatrixGenerationOptions = from_json(matrix_options_json)?;
    let result = align_spectra_by_peak_to_matrix(&spectra, alignment_options, matrix_options)?;
    to_json(&result)
}

/// Generates a layer-major matrix from serialized `Spectrum2D` JSON values.
///
/// # Errors
///
/// Returns an error when deserialization, analysis, or serialization fails.
pub fn generate_spectrum_matrix_2d_json(spectra_json: &str, options_json: &str) -> Result<String> {
    let spectra: Vec<Spectrum2D> = from_json(spectra_json)?;
    let options: MatrixGeneration2DOptions = from_json(options_json)?;
    let matrix = generate_spectrum_matrix_2d(&spectra, options)?;
    to_json(&matrix)
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
