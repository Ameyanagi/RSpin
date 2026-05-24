//! CSV JSON adapters for WASM bindings.

use rspin_core::Result;
use rspin_io::{
    read_analysis1d_json, read_analysis2d_json, read_spectrum1d_csv, read_spectrum2d_csv,
    write_analysis1d_csv, write_analysis2d_csv, write_spectrum1d_csv, write_spectrum2d_csv,
};

use super::{spectrum1d_from_json, spectrum1d_to_json, spectrum2d_from_json, spectrum2d_to_json};

/// Parses one-dimensional CSV text into serialized `Spectrum1D` JSON.
///
/// # Errors
///
/// Returns an error when parsing or serialization fails.
pub fn parse_spectrum_1d_csv_json(input: &str) -> Result<String> {
    let spectrum = read_spectrum1d_csv(input)?;
    spectrum1d_to_json(&spectrum)
}

/// Serializes `Spectrum1D` JSON into one-dimensional CSV text.
///
/// # Errors
///
/// Returns an error when deserialization or CSV serialization fails.
pub fn write_spectrum_1d_csv_json(spectrum_json: &str) -> Result<String> {
    let spectrum = spectrum1d_from_json(spectrum_json)?;
    write_spectrum1d_csv(&spectrum)
}

/// Parses two-dimensional CSV text into serialized `Spectrum2D` JSON.
///
/// # Errors
///
/// Returns an error when parsing or serialization fails.
pub fn parse_spectrum_2d_csv_json(input: &str) -> Result<String> {
    let spectrum = read_spectrum2d_csv(input)?;
    spectrum2d_to_json(&spectrum)
}

/// Serializes `Spectrum2D` JSON into two-dimensional CSV text.
///
/// # Errors
///
/// Returns an error when deserialization or CSV serialization fails.
pub fn write_spectrum_2d_csv_json(spectrum_json: &str) -> Result<String> {
    let spectrum = spectrum2d_from_json(spectrum_json)?;
    write_spectrum2d_csv(&spectrum)
}

/// Serializes one-dimensional analysis JSON into multi-section CSV text.
///
/// # Errors
///
/// Returns an error when deserialization or CSV serialization fails.
pub fn write_analysis_1d_csv_json(analysis_json: &str) -> Result<String> {
    let analysis = read_analysis1d_json(analysis_json)?;
    write_analysis1d_csv(&analysis)
}

/// Serializes two-dimensional analysis JSON into multi-section CSV text.
///
/// # Errors
///
/// Returns an error when deserialization or CSV serialization fails.
pub fn write_analysis_2d_csv_json(analysis_json: &str) -> Result<String> {
    let analysis = read_analysis2d_json(analysis_json)?;
    write_analysis2d_csv(&analysis)
}

#[cfg(test)]
mod tests;
