//! Spectrum analysis operations.

mod integration;
mod peaks;
mod ranges;
mod traits;
mod zones;

pub use integration::{Integral, IntegralRegion, TrapezoidalIntegrator, integrate_region};
pub use peaks::{LocalExtremaPeakPicker, Peak, PeakPickOptions, PeakPolarity, pick_peaks};
pub use ranges::{DetectedRange, RangeDetectionOptions, ThresholdRangeDetector, detect_ranges};
pub use traits::{Integrator, PeakPicker, RangeDetector, ZoneDetector};
pub use zones::{
    DetectedZone, ThresholdZoneDetector, ZoneConnectivity, ZoneDetectionOptions, detect_zones,
};
