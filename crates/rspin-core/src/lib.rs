//! Core data model and shared types.

mod chemistry;
mod error;
mod marks;
mod metadata;
mod nucleus;
mod spectrum;
mod units;

pub use chemistry::{Atom, Bond, BondOrder, Molecule, atoms_from_formula};
pub use error::{RSpinError, Result};
pub use marks::{AnnotationTarget, SpectrumAnnotation};
pub use metadata::Metadata;
pub use nucleus::Nucleus;
pub use spectrum::{Axis, ProcessingRecord, Spectrum1D, Spectrum2D};
pub use units::Unit;

/// Returns the crate version.
#[must_use]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
