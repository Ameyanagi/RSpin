//! Exact spin-1/2 Hamiltonian simulation.

use std::collections::BTreeSet;

use rspin_core::{RSpinError, Result};

mod hamiltonian;
mod model;
mod spectrum;
mod spectrum_2d;
mod workflow;

use hamiltonian::{basis_dimension, hamiltonian_matrix, observation_matrix, total_z_expectations};

pub use model::{ExactSpinOptions, ExactTransition, ScalarCoupling, SpinHalf, SpinHalfSystem};
pub use spectrum::{
    ExactSpectrumDecomposition1D, ExactSpectrumOptions, ExactTransitionContribution1D,
    decompose_exact_spin_half_1d, simulate_exact_spin_half_1d, validate_exact_spectrum_options,
    validate_exact_spin_half_spectrum_inputs,
};
pub use spectrum_2d::{
    ExactSpectrum2DOptions, ExactSpectrumDecomposition2D, ExactSpinPair,
    ExactTransitionContribution2D, decompose_exact_spin_half_2d, simulate_exact_spin_half_2d,
    validate_exact_spectrum_2d_options, validate_exact_spin_half_spectrum_2d_inputs,
};
pub use workflow::{
    ExactSpinHalfResultWorkflow, ExactSpinHalfSpectrum1DResultWorkflow,
    ExactSpinHalfSpectrum1DWorkflow, ExactSpinHalfSpectrum2DResultWorkflow,
    ExactSpinHalfSpectrum2DWorkflow, ExactSpinHalfWorkflow, SimulateExactSpinHalf,
    SimulateExactSpinHalfResult,
};

/// Maximum number of spin-1/2 particles supported by the dense exact solver.
pub const MAX_EXACT_SPINS: usize = 12;

/// Simulates exact spin-1/2 transition lines by dense Hamiltonian diagonalization.
///
/// The Hamiltonian uses chemical-shift offsets and the full isotropic scalar
/// coupling term in Hz, including transverse flip-flop terms. The observable is
/// the sum of transverse single-spin operators.
///
/// # Errors
///
/// Returns an error when the system or options contain invalid values, duplicate
/// couplings, out-of-range spin indices, or too many spins for dense exact
/// diagonalization.
pub fn exact_spin_half_transitions(
    system: &SpinHalfSystem,
    options: &ExactSpinOptions,
) -> Result<Vec<ExactTransition>> {
    validate_system(system)?;
    validate_options(options)?;
    validate_spin_count(system.spins.len(), options.max_spins)?;
    let detected_spins = detected_spin_indices(system.spins.len(), &options.detected_spins)?;

    let dimension = basis_dimension(system.spins.len());
    let hamiltonian = hamiltonian_matrix(system, options.spectrometer_mhz, dimension);
    let observation = observation_matrix(&detected_spins, dimension);
    let eigen = hamiltonian.symmetric_eigen();
    let transition_operator = eigen.eigenvectors.transpose() * observation * &eigen.eigenvectors;
    let magnetizations = total_z_expectations(&eigen.eigenvectors, system.spins.len(), dimension);

    let mut transitions = Vec::new();
    for lower in 0..dimension {
        for upper in (lower + 1)..dimension {
            let energy_delta_hz = eigen.eigenvalues[upper] - eigen.eigenvalues[lower];
            let frequency_hz = energy_delta_hz.abs();
            let amplitude = transition_operator[(lower, upper)];
            let intensity = amplitude * amplitude;
            if frequency_hz > f64::EPSILON && intensity > options.intensity_threshold {
                let offset_hz = signed_offset(
                    energy_delta_hz,
                    magnetizations[lower],
                    magnetizations[upper],
                );
                transitions.push(ExactTransition {
                    frequency_hz,
                    offset_hz,
                    center_ppm: offset_hz / options.spectrometer_mhz,
                    intensity,
                    contribution_count: 1,
                });
            }
        }
    }

    Ok(merge_transitions(
        transitions,
        options.frequency_tolerance_hz,
        options.spectrometer_mhz,
    ))
}

/// Validates an exact spin-1/2 system without running simulation.
///
/// # Errors
///
/// Returns an error when the system has no spins, non-finite shifts, invalid
/// couplings, duplicate couplings, or out-of-range coupling indices.
pub fn validate_spin_half_system(system: &SpinHalfSystem) -> Result<()> {
    validate_system(system)
}

/// Validates exact transition simulation options without running simulation.
///
/// This validates option-local constraints. Use
/// [`validate_exact_spin_half_inputs`] when detected-spin indices should also be
/// checked against a concrete system.
///
/// # Errors
///
/// Returns an error when options contain non-finite or invalid values.
pub fn validate_exact_spin_options(options: &ExactSpinOptions) -> Result<()> {
    validate_options(options)
}

/// Validates an exact spin-1/2 system and transition options together.
///
/// # Errors
///
/// Returns an error when the system, options, spin-count limit, or detected-spin
/// indices are invalid.
pub fn validate_exact_spin_half_inputs(
    system: &SpinHalfSystem,
    options: &ExactSpinOptions,
) -> Result<()> {
    validate_system(system)?;
    validate_options(options)?;
    validate_spin_count(system.spins.len(), options.max_spins)?;
    detected_spin_indices(system.spins.len(), &options.detected_spins).map(|_| ())
}

fn signed_offset(energy_delta_hz: f64, lower_magnetization: f64, upper_magnetization: f64) -> f64 {
    if upper_magnetization - lower_magnetization > f64::EPSILON {
        -energy_delta_hz
    } else {
        energy_delta_hz
    }
}

fn detected_spin_indices(spin_count: usize, configured: &[usize]) -> Result<Vec<usize>> {
    if configured.is_empty() {
        return Ok((0..spin_count).collect());
    }

    let mut seen = BTreeSet::new();
    for &spin in configured {
        if spin >= spin_count {
            return Err(RSpinError::InvalidSpectrum {
                message: "detected spin index is outside the system".to_owned(),
            });
        }
        if !seen.insert(spin) {
            return Err(RSpinError::InvalidSpectrum {
                message: "duplicate detected spin index".to_owned(),
            });
        }
    }

    Ok(configured.to_vec())
}

fn merge_transitions(
    mut transitions: Vec<ExactTransition>,
    tolerance_hz: f64,
    spectrometer_mhz: f64,
) -> Vec<ExactTransition> {
    transitions.sort_by(|left, right| left.offset_hz.total_cmp(&right.offset_hz));
    transitions
        .into_iter()
        .fold(Vec::new(), |mut merged, transition| {
            if let Some(last) = merged.last_mut() {
                let distance = (last.offset_hz - transition.offset_hz).abs();
                if distance <= tolerance_hz {
                    merge_transition_line(last, transition, spectrometer_mhz);
                    return merged;
                }
            }
            merged.push(transition);
            merged
        })
}

fn merge_transition_line(
    target: &mut ExactTransition,
    source: ExactTransition,
    spectrometer_mhz: f64,
) {
    let total_intensity = target.intensity + source.intensity;
    target.offset_hz = (target.offset_hz * target.intensity + source.offset_hz * source.intensity)
        / total_intensity;
    target.frequency_hz = target.offset_hz.abs();
    target.center_ppm = target.offset_hz / spectrometer_mhz;
    target.intensity = total_intensity;
    target.contribution_count = target
        .contribution_count
        .saturating_add(source.contribution_count);
}

fn validate_system(system: &SpinHalfSystem) -> Result<()> {
    if system.spins.is_empty() {
        return Err(RSpinError::InvalidSpectrum {
            message: "spin system must contain at least one spin".to_owned(),
        });
    }

    for spin in &system.spins {
        require_finite("shift_ppm", spin.shift_ppm)?;
    }

    let mut seen = BTreeSet::new();
    for coupling in &system.couplings {
        validate_coupling(system.spins.len(), *coupling, &mut seen)?;
    }

    Ok(())
}

fn validate_coupling(
    spin_count: usize,
    coupling: ScalarCoupling,
    seen: &mut BTreeSet<(usize, usize)>,
) -> Result<()> {
    require_finite("j_hz", coupling.j_hz)?;
    if coupling.spin_a >= spin_count || coupling.spin_b >= spin_count {
        return Err(RSpinError::InvalidSpectrum {
            message: "coupling references a spin outside the system".to_owned(),
        });
    }
    if coupling.spin_a == coupling.spin_b {
        return Err(RSpinError::InvalidSpectrum {
            message: "coupling must reference two different spins".to_owned(),
        });
    }
    let pair = ordered_pair(coupling.spin_a, coupling.spin_b);
    if !seen.insert(pair) {
        return Err(RSpinError::InvalidSpectrum {
            message: "duplicate scalar coupling".to_owned(),
        });
    }
    Ok(())
}

fn validate_options(options: &ExactSpinOptions) -> Result<()> {
    require_positive("spectrometer_mhz", options.spectrometer_mhz)?;
    require_finite("intensity_threshold", options.intensity_threshold)?;
    require_finite("frequency_tolerance_hz", options.frequency_tolerance_hz)?;
    if options.intensity_threshold < 0.0 {
        return Err(RSpinError::InvalidSpectrum {
            message: "intensity threshold must be non-negative".to_owned(),
        });
    }
    if options.frequency_tolerance_hz < 0.0 {
        return Err(RSpinError::InvalidSpectrum {
            message: "frequency tolerance must be non-negative".to_owned(),
        });
    }
    if options.max_spins == 0 || options.max_spins > MAX_EXACT_SPINS {
        return Err(RSpinError::InvalidSpectrum {
            message: format!("max spins must be between 1 and {MAX_EXACT_SPINS}"),
        });
    }
    Ok(())
}

fn validate_spin_count(spin_count: usize, max_spins: usize) -> Result<()> {
    if spin_count > max_spins {
        return Err(RSpinError::InvalidSpectrum {
            message: format!(
                "spin system exceeds configured exact simulation limit of {max_spins}"
            ),
        });
    }
    Ok(())
}

fn require_finite(field: &'static str, value: f64) -> Result<()> {
    if !value.is_finite() {
        return Err(RSpinError::NonFinite { field });
    }
    Ok(())
}

fn require_positive(field: &'static str, value: f64) -> Result<()> {
    require_finite(field, value)?;
    if value <= 0.0 {
        return Err(RSpinError::InvalidSpectrum {
            message: format!("{field} must be positive"),
        });
    }
    Ok(())
}

fn ordered_pair(left: usize, right: usize) -> (usize, usize) {
    if left < right {
        (left, right)
    } else {
        (right, left)
    }
}

#[cfg(test)]
mod tests;
