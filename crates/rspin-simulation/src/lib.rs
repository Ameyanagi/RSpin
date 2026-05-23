//! Spectrum simulation.

mod first_order;
mod line_shape;
mod traits;

pub use first_order::{
    CouplingGroup, FirstOrderMultiplet, FirstOrderOptions, SimulationOptions, Transition,
    multiplet_transitions, simulate_multiplet_1d,
};
pub use line_shape::LineShape;
pub use traits::Simulator;
