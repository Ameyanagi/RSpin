//! Assignment JSON helpers.

use rspin_analysis::AssignmentSet;
use rspin_core::{Result, Spectrum1D, Spectrum2D};

use super::{from_json, to_json};

/// Validates serialized assignment set JSON and returns normalized JSON.
///
/// # Errors
///
/// Returns an error when deserialization, validation, or serialization fails.
pub fn validate_assignment_set_json(assignments_json: &str) -> Result<String> {
    let assignments: AssignmentSet = from_json(assignments_json)?;
    assignments.validate()?;
    to_json(&assignments)
}

/// Appends assignment annotations to serialized `Spectrum1D` JSON.
///
/// # Errors
///
/// Returns an error when deserialization, assignment validation, annotation
/// validation, or serialization fails.
pub fn annotate_spectrum_1d_with_assignments_json(
    spectrum_json: &str,
    assignments_json: &str,
) -> Result<String> {
    let spectrum: Spectrum1D = from_json(spectrum_json)?;
    let assignments: AssignmentSet = from_json(assignments_json)?;
    let annotated = assignments.annotate_spectrum_1d(spectrum)?;
    to_json(&annotated)
}

/// Appends assignment annotations to serialized `Spectrum2D` JSON.
///
/// # Errors
///
/// Returns an error when deserialization, assignment validation, annotation
/// validation, or serialization fails.
pub fn annotate_spectrum_2d_with_assignments_json(
    spectrum_json: &str,
    assignments_json: &str,
) -> Result<String> {
    let spectrum: Spectrum2D = from_json(spectrum_json)?;
    let assignments: AssignmentSet = from_json(assignments_json)?;
    let annotated = assignments.annotate_spectrum_2d(spectrum)?;
    to_json(&annotated)
}
