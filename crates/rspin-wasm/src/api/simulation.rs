//! Simulation JSON helpers.

use rspin_core::Result;
use rspin_simulation::{
    ExactSpectrumOptions, ExactSpinOptions, FirstOrderMultiplet, SimulationOptions, SpinHalfSystem,
    decompose_exact_spin_half_1d, exact_spin_half_transitions, simulate_exact_spin_half_1d,
    simulate_multiplet_1d,
};

use super::{from_json, to_json};

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
