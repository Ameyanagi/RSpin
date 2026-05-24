//! Contour extraction JSON adapters for WASM bindings.

use rspin_core::Result;
use rspin_processing::extract_contours;

use super::{from_json, spectrum2d_from_json, to_json};

/// Extracts contour segments from serialized `Spectrum2D` JSON.
///
/// `levels_json` is a JSON array of finite contour levels.
///
/// # Errors
///
/// Returns an error when deserialization, contour extraction, or serialization fails.
pub fn extract_contours_2d_json(spectrum_json: &str, levels_json: &str) -> Result<String> {
    let spectrum = spectrum2d_from_json(spectrum_json)?;
    let levels: Vec<f64> = from_json(levels_json)?;
    let contours = extract_contours(&spectrum, &levels)?;
    to_json(&contours)
}

#[cfg(test)]
mod tests;
