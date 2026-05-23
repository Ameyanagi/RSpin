//! Simulation traits.

use rspin_core::Result;

/// Simulates an output from a model.
pub trait Simulator<M> {
    /// Simulation output.
    type Output;

    /// Simulates `model`.
    ///
    /// # Errors
    ///
    /// Returns an error when model or simulation options are invalid.
    fn simulate(&self, model: &M) -> Result<Self::Output>;
}
