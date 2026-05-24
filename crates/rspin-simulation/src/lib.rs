//! Spectrum simulation.

mod exact;
#[cfg(feature = "first-order")]
mod first_order;
mod line_shape;
mod traits;

pub use exact::{
    ExactSpectrum2DOptions, ExactSpectrumDecomposition1D, ExactSpectrumDecomposition2D,
    ExactSpectrumOptions, ExactSpinOptions, ExactSpinPair, ExactTransition,
    ExactTransitionContribution1D, ExactTransitionContribution2D, MAX_EXACT_SPINS, ScalarCoupling,
    SpinHalf, SpinHalfSystem, decompose_exact_spin_half_1d, decompose_exact_spin_half_2d,
    exact_spin_half_transitions, simulate_exact_spin_half_1d, simulate_exact_spin_half_2d,
};
#[cfg(feature = "first-order")]
pub use first_order::{
    CouplingGroup, FirstOrderMultiplet, FirstOrderOptions, SimulationOptions, Transition,
    multiplet_transitions, simulate_multiplet_1d,
};
pub use line_shape::LineShape;
pub use traits::Simulator;
