//! Spectrum analysis operations.

mod alignment;
mod assignments;
mod integration;
mod matrix;
mod peak_optimization;
mod peaks;
mod ranges;
mod traits;
mod zones;

pub use alignment::{
    AlignmentWindow, PeakAlignmentOptions, PeakAlignmentResult1D, SpectrumAlignmentShift,
    align_spectra_by_peak,
};
pub use assignments::{
    AssignedAtom, Assignment, AssignmentSet, AssignmentTarget, deterministic_assignment_id,
};
pub use integration::{Integral, IntegralRegion, TrapezoidalIntegrator, integrate_region};
pub use matrix::{MatrixGenerationOptions, SpectrumMatrix1D, generate_spectrum_matrix_1d};
pub use peak_optimization::{
    OptimizedPeak, PeakOptimizationOptions, QuadraticPeakOptimizer, optimize_peaks_quadratic,
};
pub use peaks::{LocalExtremaPeakPicker, Peak, PeakPickOptions, PeakPolarity, pick_peaks};
pub use ranges::{DetectedRange, RangeDetectionOptions, ThresholdRangeDetector, detect_ranges};
pub use traits::{Integrator, PeakOptimizer, PeakPicker, RangeDetector, ZoneDetector};
pub use zones::{
    DetectedZone, ThresholdZoneDetector, ZoneConnectivity, ZoneDetectionOptions, detect_zones,
};
