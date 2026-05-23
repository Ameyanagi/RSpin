//! Core data model and shared types.

mod error;
mod metadata;
mod nucleus;
mod spectrum;
mod units;

pub use error::{RSpinError, Result};
pub use metadata::Metadata;
pub use nucleus::Nucleus;
pub use spectrum::{Axis, ProcessingRecord, Spectrum1D, Spectrum2D};
pub use units::Unit;

/// Returns the crate version.
#[must_use]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
