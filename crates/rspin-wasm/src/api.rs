//! JSON-oriented API helpers for WASM bindings.

mod contours;
mod csv_io;
mod processing_1d;
mod processing_2d;

use serde::{Serialize, de::DeserializeOwned};

use rspin_analysis::{
    AssignmentSet, DetectedMultiplet, DetectedRange, DetectedZone, IntegralRegion,
    IntegralRegion2D, JCouplingGraph, MatrixGeneration2DOptions, MatrixGenerationOptions,
    MultipletDetectionOptions, PeakOptimizationOptions, PeakPickOptions, RangeDetectionOptions,
    SignalSummary2DOptions, SignalSummaryOptions, ZoneDetectionOptions, detect_multiplets,
    detect_ranges, detect_zones, generate_spectrum_matrix_1d, generate_spectrum_matrix_2d,
    integrate_region, integrate_region_2d, optimize_peaks_quadratic, pick_peaks,
    summarize_signals_1d, summarize_signals_2d,
};
use rspin_core::{RSpinError, Result, Spectrum1D, Spectrum2D};
use rspin_io::read_jcamp_dx_1d;
use rspin_prediction::{
    PredictionSet, PredictionSpectrum2DOptions, PredictionSpectrumOptions, render_prediction_1d,
    render_prediction_2d,
};
use rspin_processing::{AutoPhaseOptions, auto_phase_correct, normalize_max_abs, scale_intensity};
use rspin_simulation::{
    ExactSpectrumOptions, ExactSpinOptions, FirstOrderMultiplet, SimulationOptions, SpinHalfSystem,
    decompose_exact_spin_half_1d, exact_spin_half_transitions, simulate_exact_spin_half_1d,
    simulate_multiplet_1d,
};

pub use contours::extract_contours_2d_json;
pub use csv_io::{
    parse_spectrum_1d_csv_json, parse_spectrum_2d_csv_json, write_spectrum_1d_csv_json,
    write_spectrum_2d_csv_json,
};
pub use processing_1d::{
    abs_spectrum_1d_json, crop_spectrum_1d_json, exponential_apodization_spectrum_1d_json,
    fft_spectrum_1d_json, magnitude_spectrum_1d_json, offset_spectrum_1d_json,
    phase_spectrum_1d_json, resample_spectrum_1d_json, shift_spectrum_1d_axis_json,
    subtract_baseline_spectrum_1d_json, zero_fill_spectrum_1d_json,
};
pub use processing_2d::{
    abs_spectrum_2d_json, auto_phase_spectrum_2d_json, crop_spectrum_2d_json, fft_spectrum_2d_json,
    normalize_spectrum_2d_json, phase_spectrum_2d_json, project_spectrum_2d_x_json,
    project_spectrum_2d_y_json, resample_spectrum_2d_json, scale_spectrum_2d_json,
    slice_spectrum_2d_x_at_y_index_json, slice_spectrum_2d_x_at_y_json,
    slice_spectrum_2d_y_at_x_index_json, slice_spectrum_2d_y_at_x_json, zero_fill_spectrum_2d_json,
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

/// Validates serialized assignment set JSON and returns normalized JSON.
///
/// # Errors
///
/// Returns an error when deserialization, validation, or serialization fails.
pub fn validate_assignment_set_json(assignments_json: &str) -> Result<String> {
    let assignments: AssignmentSet = from_json(assignments_json)?;
    assignments.validate()?;
    to_json(&assignments)
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

/// Simulates a serialized first-order multiplet and options into `Spectrum1D` JSON.
///
/// # Errors
///
/// Returns an error when deserialization, simulation, or serialization fails.
pub fn simulate_first_order_multiplet_json(
    multiplet_json: &str,
    options_json: &str,
) -> Result<String> {
    let multiplet: FirstOrderMultiplet = from_json(multiplet_json)?;
    let options: SimulationOptions = from_json(options_json)?;
    let spectrum = simulate_multiplet_1d(&multiplet, options)?;
    to_json(&spectrum)
}

/// Simulates exact spin-1/2 transitions and returns serialized transition JSON.
///
/// # Errors
///
/// Returns an error when deserialization, exact simulation, or serialization fails.
pub fn simulate_exact_spin_half_transitions_json(
    system_json: &str,
    options_json: &str,
) -> Result<String> {
    let system: SpinHalfSystem = from_json(system_json)?;
    let options: ExactSpinOptions = from_json(options_json)?;
    let transitions = exact_spin_half_transitions(&system, &options)?;
    to_json(&transitions)
}

/// Simulates an exact spin-1/2 system into serialized `Spectrum1D` JSON.
///
/// # Errors
///
/// Returns an error when deserialization, exact simulation, or serialization fails.
pub fn simulate_exact_spin_half_spectrum_json(
    system_json: &str,
    options_json: &str,
) -> Result<String> {
    let system: SpinHalfSystem = from_json(system_json)?;
    let options: ExactSpectrumOptions = from_json(options_json)?;
    let spectrum = simulate_exact_spin_half_1d(&system, &options)?;
    to_json(&spectrum)
}

/// Simulates exact spin-1/2 spectrum JSON with per-transition contributions.
///
/// # Errors
///
/// Returns an error when deserialization, exact simulation, or serialization fails.
pub fn decompose_exact_spin_half_spectrum_json(
    system_json: &str,
    options_json: &str,
) -> Result<String> {
    let system: SpinHalfSystem = from_json(system_json)?;
    let options: ExactSpectrumOptions = from_json(options_json)?;
    let decomposition = decompose_exact_spin_half_1d(&system, &options)?;
    to_json(&decomposition)
}

/// Validates serialized prediction JSON and returns normalized JSON.
///
/// # Errors
///
/// Returns an error when deserialization, validation, or serialization fails.
pub fn validate_prediction_json(prediction_json: &str) -> Result<String> {
    let prediction: PredictionSet = from_json(prediction_json)?;
    prediction.validate()?;
    to_json(&prediction)
}

/// Renders serialized one-dimensional prediction JSON into `Spectrum1D` JSON.
///
/// # Errors
///
/// Returns an error when deserialization, validation, rendering, or serialization fails.
pub fn render_prediction_1d_json(prediction_json: &str, options_json: &str) -> Result<String> {
    let prediction: PredictionSet = from_json(prediction_json)?;
    let options: PredictionSpectrumOptions = from_json(options_json)?;
    let spectrum = render_prediction_1d(&prediction, &options)?;
    to_json(&spectrum)
}

/// Renders serialized two-dimensional prediction JSON into `Spectrum2D` JSON.
///
/// # Errors
///
/// Returns an error when deserialization, validation, rendering, or serialization fails.
pub fn render_prediction_2d_json(prediction_json: &str, options_json: &str) -> Result<String> {
    let prediction: PredictionSet = from_json(prediction_json)?;
    let options: PredictionSpectrum2DOptions = from_json(options_json)?;
    let spectrum = render_prediction_2d(&prediction, &options)?;
    to_json(&spectrum)
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
