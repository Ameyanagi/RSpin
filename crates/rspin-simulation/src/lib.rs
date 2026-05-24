//! Spectrum simulation.

mod exact;
mod line_shape;
mod traits;

pub use exact::{
    ExactSpectrum2DOptions, ExactSpectrumDecomposition1D, ExactSpectrumDecomposition2D,
    ExactSpectrumOptions, ExactSpinHalfResultWorkflow, ExactSpinHalfSpectrum1DResultWorkflow,
    ExactSpinHalfSpectrum1DWorkflow, ExactSpinHalfSpectrum2DResultWorkflow,
    ExactSpinHalfSpectrum2DWorkflow, ExactSpinHalfWorkflow, ExactSpinOptions, ExactSpinPair,
    ExactTransition, ExactTransitionContribution1D, ExactTransitionContribution2D, MAX_EXACT_SPINS,
    ScalarCoupling, SimulateExactSpinHalf, SimulateExactSpinHalfResult, SpinHalf, SpinHalfSystem,
    decompose_exact_spin_half_1d, decompose_exact_spin_half_2d, exact_spin_half_transitions,
    simulate_exact_spin_half_1d, simulate_exact_spin_half_2d, validate_exact_spectrum_2d_options,
    validate_exact_spectrum_options, validate_exact_spin_half_inputs,
    validate_exact_spin_half_spectrum_2d_inputs, validate_exact_spin_half_spectrum_inputs,
    validate_exact_spin_options, validate_spin_half_system,
};
pub use line_shape::LineShape;
pub use traits::Simulator;
