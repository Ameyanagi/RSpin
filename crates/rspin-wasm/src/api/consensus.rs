//! Consensus peak JSON helpers.

use rspin_analysis::{
    ConsensusPeakOptions, ConsensusRangeOptions, detect_consensus_peaks_1d,
    detect_consensus_ranges_1d,
};
use rspin_core::{Result, Spectrum1D};

use super::{from_json, to_json};

/// Detects consensus peaks from serialized one-dimensional spectra.
///
/// # Errors
///
/// Returns an error when deserialization, peak detection, grouping, or
/// serialization fails.
pub fn detect_consensus_peaks_1d_json(spectra_json: &str, options_json: &str) -> Result<String> {
    let spectra: Vec<Spectrum1D> = from_json(spectra_json)?;
    let options: ConsensusPeakOptions = from_json(options_json)?;
    let result = detect_consensus_peaks_1d(&spectra, options)?;
    to_json(&result)
}

/// Detects consensus ranges from serialized one-dimensional spectra.
///
/// # Errors
///
/// Returns an error when deserialization, range detection, grouping, or
/// serialization fails.
pub fn detect_consensus_ranges_1d_json(spectra_json: &str, options_json: &str) -> Result<String> {
    let spectra: Vec<Spectrum1D> = from_json(spectra_json)?;
    let options: ConsensusRangeOptions = from_json(options_json)?;
    let result = detect_consensus_ranges_1d(&spectra, options)?;
    to_json(&result)
}
