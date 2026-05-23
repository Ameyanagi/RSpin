//! Analysis traits.

use rspin_core::{Result, Spectrum1D};

use crate::{Integral, IntegralRegion, Peak};

/// Picks peaks from a one-dimensional spectrum.
pub trait PeakPicker {
    /// Returns detected peaks.
    ///
    /// # Errors
    ///
    /// Returns an error when picker options are invalid for the spectrum.
    fn pick(&self, spectrum: &Spectrum1D) -> Result<Vec<Peak>>;
}

/// Integrates a one-dimensional spectrum over a region.
pub trait Integrator {
    /// Returns the integral over `region`.
    ///
    /// # Errors
    ///
    /// Returns an error when the region is invalid for the spectrum.
    fn integrate(&self, spectrum: &Spectrum1D, region: IntegralRegion) -> Result<Integral>;
}
