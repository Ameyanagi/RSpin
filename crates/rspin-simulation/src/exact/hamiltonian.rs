//! Dense spin-1/2 Hamiltonian and detection operators.

use nalgebra::DMatrix;

use super::SpinHalfSystem;

pub(super) fn hamiltonian_matrix(
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

pub(super) fn observation_matrix(detected_spins: &[usize], dimension: usize) -> DMatrix<f64> {
    let mut matrix = DMatrix::zeros(dimension, dimension);
    for state in 0..dimension {
        for &spin in detected_spins {
            let flipped = state ^ spin_bit(spin);
            matrix[(flipped, state)] += 0.5;
        }
    }
    matrix
}

pub(super) fn total_z_expectations(
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

pub(super) fn basis_dimension(spin_count: usize) -> usize {
    1_usize << spin_count
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

fn total_spin_z(state: usize, spin_count: usize) -> f64 {
    (0..spin_count).map(|spin| spin_z(state, spin)).sum()
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
