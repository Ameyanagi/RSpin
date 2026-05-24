//! WebAssembly bindings for simulation workflows.

use wasm_bindgen::prelude::*;

use crate::{
    decompose_exact_spin_half_spectrum_2d_json, decompose_exact_spin_half_spectrum_json, js_error,
    simulate_exact_spin_half_spectrum_2d_json, simulate_exact_spin_half_spectrum_json,
    simulate_exact_spin_half_transitions_json,
};

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
    crate::simulate_first_order_multiplet_json(multiplet_json, options_json)
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

/// Simulates an exact spin-1/2 system as a serialized two-dimensional spectrum.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, simulation, or
/// serialization fails.
#[wasm_bindgen(js_name = simulateExactSpinHalfSpectrum2d)]
pub fn simulate_exact_spin_half_spectrum_2d(
    system_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    simulate_exact_spin_half_spectrum_2d_json(system_json, options_json)
        .map_err(|error| js_error(&error))
}

/// Simulates exact spin-1/2 2D spectrum and contributions as JSON.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, simulation, or
/// serialization fails.
#[wasm_bindgen(js_name = decomposeExactSpinHalfSpectrum2d)]
pub fn decompose_exact_spin_half_spectrum_2d(
    system_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    decompose_exact_spin_half_spectrum_2d_json(system_json, options_json)
        .map_err(|error| js_error(&error))
}
