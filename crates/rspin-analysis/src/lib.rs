//! Spectrum analysis operations.

mod integration;
mod peaks;
mod ranges;
mod traits;

pub use integration::{Integral, IntegralRegion, TrapezoidalIntegrator, integrate_region};
pub use peaks::{LocalExtremaPeakPicker, Peak, PeakPickOptions, PeakPolarity, pick_peaks};
pub use ranges::{DetectedRange, RangeDetectionOptions, ThresholdRangeDetector, detect_ranges};
pub use traits::{Integrator, PeakPicker, RangeDetector};
