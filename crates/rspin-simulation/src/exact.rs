//! Exact spin-1/2 Hamiltonian simulation.

use std::collections::BTreeSet;

use nalgebra::DMatrix;
use rspin_core::{RSpinError, Result};
use serde::{Deserialize, Serialize};

use crate::Simulator;

/// Maximum number of spin-1/2 particles supported by the dense exact solver.
pub const MAX_EXACT_SPINS: usize = 12;

/// A spin-1/2 nucleus in an exact spin system.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpinHalf {
    /// Chemical shift in ppm relative to the transmitter reference.
    pub shift_ppm: f64,
}

/// An isotropic scalar coupling between two spin-1/2 nuclei.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScalarCoupling {
    /// Zero-based index of the first spin.
    pub spin_a: usize,
    /// Zero-based index of the second spin.
    pub spin_b: usize,
    /// Scalar coupling constant in Hz.
    pub j_hz: f64,
}

/// A spin-1/2 system for exact transition simulation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpinHalfSystem {
    /// Spin definitions.
    pub spins: Vec<SpinHalf>,
    /// Scalar couplings between spins.
    pub couplings: Vec<ScalarCoupling>,
}

/// Options for exact spin-1/2 transition simulation.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExactSpinOptions {
    /// Spectrometer frequency in MHz.
    pub spectrometer_mhz: f64,
    /// Discard transitions with intensity at or below this threshold.
    pub intensity_threshold: f64,
    /// Merge transitions this close in Hz.
    pub frequency_tolerance_hz: f64,
    /// Per-call spin-count limit, capped by [`MAX_EXACT_SPINS`].
    pub max_spins: usize,
}

impl Default for ExactSpinOptions {
    fn default() -> Self {
        Self {
            spectrometer_mhz: 400.0,
            intensity_threshold: 1.0e-12,
            frequency_tolerance_hz: 1.0e-9,
            max_spins: 10,
        }
    }
}

impl Simulator<SpinHalfSystem> for ExactSpinOptions {
    type Output = Vec<ExactTransition>;

    fn simulate(&self, model: &SpinHalfSystem) -> Result<Self::Output> {
        exact_spin_half_transitions(model, *self)
    }
}

/// An observable exact transition line.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExactTransition {
    /// Absolute transition frequency in Hz.
    pub frequency_hz: f64,
    /// Signed transition offset in Hz relative to the transmitter reference.
    pub offset_hz: f64,
    /// Signed transition position in ppm.
    pub center_ppm: f64,
    /// Relative transition intensity.
    pub intensity: f64,
    /// Number of eigenstate transitions merged into this line.
    pub contribution_count: u32,
}

/// Simulates exact spin-1/2 transition lines by dense Hamiltonian diagonalization.
///
/// The Hamiltonian uses chemical-shift offsets and isotropic scalar coupling in
/// Hz. The observable is the sum of transverse single-spin operators.
///
/// # Errors
///
/// Returns an error when the system or options contain invalid values, duplicate
/// couplings, out-of-range spin indices, or too many spins for dense exact
/// diagonalization.
pub fn exact_spin_half_transitions(
    system: &SpinHalfSystem,
    options: ExactSpinOptions,
) -> Result<Vec<ExactTransition>> {
    validate_system(system)?;
    validate_options(options)?;
    validate_spin_count(system.spins.len(), options.max_spins)?;

    let dimension = basis_dimension(system.spins.len());
    let hamiltonian = hamiltonian_matrix(system, options.spectrometer_mhz, dimension);
    let observation = observation_matrix(system.spins.len(), dimension);
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

fn hamiltonian_matrix(
    system: &SpinHalfSystem,
    spectrometer_mhz: f64,
    dimension: usize,
) -> DMatrix<f64> {
    let mut matrix = DMatrix::zeros(dimension, dimension);
    for state in 0..dimension {
        matrix[(state, state)] += chemical_shift_energy(system, spectrometer_mhz, state);
        matrix[(state, state)] += scalar_coupling_z_energy(system, state);
        add_scalar_flip_flop_terms(&mut matrix, system, state);
    }
    matrix
}

fn chemical_shift_energy(system: &SpinHalfSystem, spectrometer_mhz: f64, state: usize) -> f64 {
    system
        .spins
        .iter()
        .enumerate()
        .map(|(spin, definition)| -definition.shift_ppm * spectrometer_mhz * spin_z(state, spin))
        .sum()
}

fn scalar_coupling_z_energy(system: &SpinHalfSystem, state: usize) -> f64 {
    system
        .couplings
        .iter()
        .map(|coupling| {
            coupling.j_hz * spin_z(state, coupling.spin_a) * spin_z(state, coupling.spin_b)
        })
        .sum()
}

fn add_scalar_flip_flop_terms(matrix: &mut DMatrix<f64>, system: &SpinHalfSystem, state: usize) {
    for coupling in &system.couplings {
        if spin_is_up(state, coupling.spin_a) != spin_is_up(state, coupling.spin_b) {
            let flipped = state ^ spin_bit(coupling.spin_a) ^ spin_bit(coupling.spin_b);
            matrix[(flipped, state)] += coupling.j_hz / 2.0;
        }
    }
}

fn observation_matrix(spin_count: usize, dimension: usize) -> DMatrix<f64> {
    let mut matrix = DMatrix::zeros(dimension, dimension);
    for state in 0..dimension {
        for spin in 0..spin_count {
            let flipped = state ^ spin_bit(spin);
            matrix[(flipped, state)] += 0.5;
        }
    }
    matrix
}

fn total_z_expectations(
    eigenvectors: &DMatrix<f64>,
    spin_count: usize,
    dimension: usize,
) -> Vec<f64> {
    (0..dimension)
        .map(|state| {
            (0..dimension)
                .map(|basis| eigenvectors[(basis, state)].powi(2) * total_spin_z(basis, spin_count))
                .sum()
        })
        .collect()
}

fn total_spin_z(state: usize, spin_count: usize) -> f64 {
    (0..spin_count).map(|spin| spin_z(state, spin)).sum()
}

fn signed_offset(energy_delta_hz: f64, lower_magnetization: f64, upper_magnetization: f64) -> f64 {
    if upper_magnetization - lower_magnetization > f64::EPSILON {
        -energy_delta_hz
    } else {
        energy_delta_hz
    }
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

fn validate_options(options: ExactSpinOptions) -> Result<()> {
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

fn spin_z(state: usize, spin: usize) -> f64 {
    if spin_is_up(state, spin) { 0.5 } else { -0.5 }
}

fn spin_is_up(state: usize, spin: usize) -> bool {
    state & spin_bit(spin) != 0
}

fn spin_bit(spin: usize) -> usize {
    1_usize << spin
}

fn basis_dimension(spin_count: usize) -> usize {
    1_usize << spin_count
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
