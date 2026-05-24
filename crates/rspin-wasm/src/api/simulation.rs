//! Simulation JSON helpers.

use rspin_core::Result;
use rspin_io::{
    read_exact_spectrum_2d_options_json, read_exact_spectrum_options_json,
    read_exact_spin_options_json, read_spin_half_system_json, write_exact_decomposition_1d_json,
    write_exact_decomposition_2d_json, write_exact_transitions_json,
};
use rspin_simulation::{
    decompose_exact_spin_half_1d, decompose_exact_spin_half_2d, exact_spin_half_transitions,
    simulate_exact_spin_half_1d, simulate_exact_spin_half_2d,
};

use super::{spectrum1d_to_json, spectrum2d_to_json};

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
