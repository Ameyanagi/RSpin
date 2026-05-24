//! Simulation JSON helpers.

use rspin_core::Result;
use rspin_io::{
    read_exact_spectrum_2d_options_json, read_exact_spectrum_options_json,
    read_exact_spin_options_json, read_exact_transitions_csv, read_exact_transitions_json,
    read_spin_half_system_json, write_exact_decomposition_1d_json,
    write_exact_decomposition_2d_json, write_exact_spectrum_2d_options_json,
    write_exact_spectrum_options_json, write_exact_spin_options_json, write_exact_transitions_csv,
    write_exact_transitions_json, write_spin_half_system_json,
};
use rspin_simulation::{
    decompose_exact_spin_half_1d, decompose_exact_spin_half_2d, exact_spin_half_transitions,
    simulate_exact_spin_half_1d, simulate_exact_spin_half_2d,
};

use super::{spectrum1d_to_json, spectrum2d_to_json};

/// Validates exact spin-1/2 system JSON and returns normalized JSON.
///
/// # Errors
///
/// Returns an error when deserialization, validation, or serialization fails.
pub fn validate_exact_spin_half_system_json(system_json: &str) -> Result<String> {
    let system = read_spin_half_system_json(system_json)?;
    system.validate()?;
    write_spin_half_system_json(&system)
}

/// Validates exact transition option JSON and returns normalized JSON.
///
/// # Errors
///
/// Returns an error when deserialization, validation, or serialization fails.
pub fn validate_exact_spin_options_json(options_json: &str) -> Result<String> {
    let options = read_exact_spin_options_json(options_json)?;
    options.validate()?;
    write_exact_spin_options_json(&options)
}

/// Validates exact one-dimensional rendering option JSON and returns normalized JSON.
///
/// # Errors
///
/// Returns an error when deserialization, validation, or serialization fails.
pub fn validate_exact_spectrum_options_json(options_json: &str) -> Result<String> {
    let options = read_exact_spectrum_options_json(options_json)?;
    options.validate()?;
    write_exact_spectrum_options_json(&options)
}

/// Validates exact two-dimensional rendering option JSON and returns normalized JSON.
///
/// # Errors
///
/// Returns an error when deserialization, validation, or serialization fails.
pub fn validate_exact_spectrum_2d_options_json(options_json: &str) -> Result<String> {
    let options = read_exact_spectrum_2d_options_json(options_json)?;
    options.validate()?;
    write_exact_spectrum_2d_options_json(&options)
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
    let system = read_spin_half_system_json(system_json)?;
    let options = read_exact_spin_options_json(options_json)?;
    let transitions = exact_spin_half_transitions(&system, &options)?;
    write_exact_transitions_json(&transitions)
}

/// Parses exact transition CSV and returns serialized transition JSON.
///
/// # Errors
///
/// Returns an error when CSV parsing or JSON serialization fails.
pub fn parse_exact_transitions_csv_json(input: &str) -> Result<String> {
    let transitions = read_exact_transitions_csv(input)?;
    write_exact_transitions_json(&transitions)
}

/// Converts serialized exact transition JSON to CSV.
///
/// # Errors
///
/// Returns an error when deserialization or CSV serialization fails.
pub fn write_exact_transitions_csv_json(transitions_json: &str) -> Result<String> {
    let transitions = read_exact_transitions_json(transitions_json)?;
    write_exact_transitions_csv(&transitions)
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
    let system = read_spin_half_system_json(system_json)?;
    let options = read_exact_spectrum_options_json(options_json)?;
    let spectrum = simulate_exact_spin_half_1d(&system, &options)?;
    spectrum1d_to_json(&spectrum)
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
    let system = read_spin_half_system_json(system_json)?;
    let options = read_exact_spectrum_options_json(options_json)?;
    let decomposition = decompose_exact_spin_half_1d(&system, &options)?;
    write_exact_decomposition_1d_json(&decomposition)
}

/// Simulates an exact spin-1/2 system into serialized `Spectrum2D` JSON.
///
/// # Errors
///
/// Returns an error when deserialization, exact simulation, or serialization fails.
pub fn simulate_exact_spin_half_spectrum_2d_json(
    system_json: &str,
    options_json: &str,
) -> Result<String> {
    let system = read_spin_half_system_json(system_json)?;
    let options = read_exact_spectrum_2d_options_json(options_json)?;
    let spectrum = simulate_exact_spin_half_2d(&system, &options)?;
    spectrum2d_to_json(&spectrum)
}

/// Simulates exact spin-1/2 2D spectrum JSON with per-correlation contributions.
///
/// # Errors
///
/// Returns an error when deserialization, exact simulation, or serialization fails.
pub fn decompose_exact_spin_half_spectrum_2d_json(
    system_json: &str,
    options_json: &str,
) -> Result<String> {
    let system = read_spin_half_system_json(system_json)?;
    let options = read_exact_spectrum_2d_options_json(options_json)?;
    let decomposition = decompose_exact_spin_half_2d(&system, &options)?;
    write_exact_decomposition_2d_json(&decomposition)
}
