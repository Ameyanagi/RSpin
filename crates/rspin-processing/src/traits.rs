//! Processing traits.

use rspin_core::Result;

/// A reusable processing step.
pub trait ProcessingStep<S> {
    /// Applies this step to `spectrum`, returning a processed copy.
    ///
    /// # Errors
    ///
    /// Returns an error when the step parameters are invalid for the spectrum.
    fn apply(&self, spectrum: &S) -> Result<S>;
}
