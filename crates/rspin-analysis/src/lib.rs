//! Spectrum analysis operations.

mod integration;
mod peaks;
mod traits;

pub use integration::{Integral, IntegralRegion, TrapezoidalIntegrator, integrate_region};
pub use peaks::{LocalExtremaPeakPicker, Peak, PeakPickOptions, PeakPolarity, pick_peaks};
pub use traits::{Integrator, PeakPicker};
