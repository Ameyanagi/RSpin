//! Prediction traits.

use rspin_core::Result;

use crate::PredictionSet;

/// Predicts spectra or signals from an input model.
pub trait Predictor<I> {
    /// Predicts a set of one-dimensional and two-dimensional signals.
    ///
    /// # Errors
    ///
    /// Returns an error when the input cannot be predicted or the produced
    /// payload is invalid.
    fn predict(&self, input: &I) -> Result<PredictionSet>;
}
