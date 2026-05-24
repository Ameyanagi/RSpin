//! CSV JSON adapters for WASM bindings.

use rspin_analysis::{SpectrumAnalysis1D, SpectrumAnalysis2D};
use rspin_core::{Result, Spectrum1D, Spectrum2D};
use rspin_io::{
    read_spectrum1d_csv, read_spectrum2d_csv, write_analysis1d_csv, write_analysis2d_csv,
    write_spectrum1d_csv, write_spectrum2d_csv,
};

use super::{from_json, to_json};

/// Parses one-dimensional CSV text into serialized `Spectrum1D` JSON.
///
/// # Errors
///
/// Returns an error when parsing or serialization fails.
pub fn parse_spectrum_1d_csv_json(input: &str) -> Result<String> {
    let spectrum = read_spectrum1d_csv(input)?;
    to_json(&spectrum)
}

/// Serializes `Spectrum1D` JSON into one-dimensional CSV text.
///
/// # Errors
///
/// Returns an error when deserialization or CSV serialization fails.
pub fn write_spectrum_1d_csv_json(spectrum_json: &str) -> Result<String> {
    let spectrum: Spectrum1D = from_json(spectrum_json)?;
    write_spectrum1d_csv(&spectrum)
}

/// Parses two-dimensional CSV text into serialized `Spectrum2D` JSON.
///
/// # Errors
///
/// Returns an error when parsing or serialization fails.
pub fn parse_spectrum_2d_csv_json(input: &str) -> Result<String> {
    let spectrum = read_spectrum2d_csv(input)?;
    to_json(&spectrum)
}

/// Serializes `Spectrum2D` JSON into two-dimensional CSV text.
///
/// # Errors
///
/// Returns an error when deserialization or CSV serialization fails.
pub fn write_spectrum_2d_csv_json(spectrum_json: &str) -> Result<String> {
    let spectrum: Spectrum2D = from_json(spectrum_json)?;
    write_spectrum2d_csv(&spectrum)
}

/// Serializes one-dimensional analysis JSON into multi-section CSV text.
///
/// # Errors
///
/// Returns an error when deserialization or CSV serialization fails.
pub fn write_analysis_1d_csv_json(analysis_json: &str) -> Result<String> {
    let analysis: SpectrumAnalysis1D = from_json(analysis_json)?;
    write_analysis1d_csv(&analysis)
}

/// Serializes two-dimensional analysis JSON into multi-section CSV text.
///
/// # Errors
///
/// Returns an error when deserialization or CSV serialization fails.
pub fn write_analysis_2d_csv_json(analysis_json: &str) -> Result<String> {
    let analysis: SpectrumAnalysis2D = from_json(analysis_json)?;
    write_analysis2d_csv(&analysis)
}

#[cfg(test)]
mod tests;
