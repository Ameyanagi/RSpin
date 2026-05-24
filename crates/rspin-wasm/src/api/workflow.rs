//! High-level analysis workflow JSON helpers.

use rspin_analysis::{
    SpectrumAnalysis1DOptions, SpectrumAnalysis2DOptions, analyze_spectrum_1d, analyze_spectrum_2d,
};
use rspin_core::{Result, Spectrum1D, Spectrum2D};

use super::{from_json, to_json};

/// Runs the default one-dimensional analysis workflow on serialized spectrum JSON.
///
/// # Errors
///
/// Returns an error when deserialization, analysis, or serialization fails.
pub fn analyze_spectrum_1d_json(spectrum_json: &str, options_json: &str) -> Result<String> {
    let spectrum: Spectrum1D = from_json(spectrum_json)?;
    let options: SpectrumAnalysis1DOptions = from_json(options_json)?;
    let analysis = analyze_spectrum_1d(&spectrum, options)?;
    to_json(&analysis)
}

/// Runs the default two-dimensional analysis workflow on serialized spectrum JSON.
///
/// # Errors
///
/// Returns an error when deserialization, analysis, or serialization fails.
pub fn analyze_spectrum_2d_json(spectrum_json: &str, options_json: &str) -> Result<String> {
    let spectrum: Spectrum2D = from_json(spectrum_json)?;
    let options: SpectrumAnalysis2DOptions = from_json(options_json)?;
    let analysis = analyze_spectrum_2d(&spectrum, options)?;
    to_json(&analysis)
}
