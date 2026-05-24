//! JSON-oriented API helpers for WASM bindings.

use serde::{Serialize, de::DeserializeOwned};

use rspin_analysis::{
    AssignmentSet, DetectedMultiplet, DetectedRange, IntegralRegion, JCouplingGraph,
    MultipletDetectionOptions, PeakOptimizationOptions, PeakPickOptions, SignalSummaryOptions,
    detect_multiplets, integrate_region, optimize_peaks_quadratic, pick_peaks,
    summarize_signals_1d,
};
use rspin_core::{RSpinError, Result, Spectrum1D};
use rspin_io::read_jcamp_dx_1d;
use rspin_prediction::PredictionSet;
use rspin_processing::{normalize_max_abs, scale_intensity};
use rspin_simulation::{
    ExactSpectrumOptions, ExactSpinOptions, FirstOrderMultiplet, SimulationOptions, SpinHalfSystem,
    decompose_exact_spin_half_1d, exact_spin_half_transitions, simulate_exact_spin_half_1d,
    simulate_multiplet_1d,
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

#[cfg(test)]
mod tests;
