//! Analysis traits.

use rspin_core::{Result, Spectrum1D, Spectrum2D};

use crate::{
    DetectedMultiplet, DetectedRange, DetectedZone, Integral, IntegralRegion, OptimizedPeak, Peak,
};

/// Picks peaks from a one-dimensional spectrum.
pub trait PeakPicker {
    /// Returns detected peaks.
    ///
    /// # Errors
    ///
    /// Returns an error when picker options are invalid for the spectrum.
    fn pick(&self, spectrum: &Spectrum1D) -> Result<Vec<Peak>>;
}

/// Optimizes picked peaks from a one-dimensional spectrum.
pub trait PeakOptimizer {
    /// Returns optimized peaks corresponding to `peaks`.
    ///
    /// # Errors
    ///
    /// Returns an error when peaks cannot be mapped onto the spectrum.
    fn optimize(&self, spectrum: &Spectrum1D, peaks: &[Peak]) -> Result<Vec<OptimizedPeak>>;
}

/// Detects one-dimensional multiplets from picked peaks.
pub trait MultipletDetector {
    /// Returns detected multiplets corresponding to `peaks`.
    ///
    /// # Errors
    ///
    /// Returns an error when detector options or peak data are invalid.
    fn detect(&self, spectrum: &Spectrum1D, peaks: &[Peak]) -> Result<Vec<DetectedMultiplet>>;
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

/// Detects ranges from a one-dimensional spectrum.
pub trait RangeDetector {
    /// Returns detected ranges.
    ///
    /// # Errors
    ///
    /// Returns an error when detector options are invalid for the spectrum.
    fn detect(&self, spectrum: &Spectrum1D) -> Result<Vec<DetectedRange>>;
}

/// Detects zones from a two-dimensional spectrum.
pub trait ZoneDetector {
    /// Returns detected zones.
    ///
    /// # Errors
    ///
    /// Returns an error when detector options are invalid for the spectrum.
    fn detect(&self, spectrum: &Spectrum2D) -> Result<Vec<DetectedZone>>;
}
