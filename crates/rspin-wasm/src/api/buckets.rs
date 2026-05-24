//! Bucketing JSON helpers.

use rspin_analysis::{
    BucketOptions1D, BucketOptions2D, bucket_spectra_1d, bucket_spectra_2d, bucket_spectrum_1d,
    bucket_spectrum_2d,
};
use rspin_core::{Result, Spectrum1D, Spectrum2D};

use super::{from_json, to_json};

/// Buckets a serialized one-dimensional spectrum.
///
/// # Errors
///
/// Returns an error when deserialization, bucketing, or serialization fails.
pub fn bucket_spectrum_1d_json(spectrum_json: &str, options_json: &str) -> Result<String> {
    let spectrum: Spectrum1D = from_json(spectrum_json)?;
    let options: BucketOptions1D = from_json(options_json)?;
    let buckets = bucket_spectrum_1d(&spectrum, options)?;
    to_json(&buckets)
}

/// Buckets serialized one-dimensional spectra into a row-major matrix.
///
/// # Errors
///
/// Returns an error when deserialization, bucketing, or serialization fails.
pub fn bucket_spectra_1d_json(spectra_json: &str, options_json: &str) -> Result<String> {
    let spectra: Vec<Spectrum1D> = from_json(spectra_json)?;
    let options: BucketOptions1D = from_json(options_json)?;
    let matrix = bucket_spectra_1d(&spectra, options)?;
    to_json(&matrix)
}

/// Buckets a serialized two-dimensional spectrum.
///
/// # Errors
///
/// Returns an error when deserialization, bucketing, or serialization fails.
pub fn bucket_spectrum_2d_json(spectrum_json: &str, options_json: &str) -> Result<String> {
    let spectrum: Spectrum2D = from_json(spectrum_json)?;
    let options: BucketOptions2D = from_json(options_json)?;
    let buckets = bucket_spectrum_2d(&spectrum, options)?;
    to_json(&buckets)
}

/// Buckets serialized two-dimensional spectra into a layer-major matrix.
///
/// # Errors
///
/// Returns an error when deserialization, bucketing, or serialization fails.
pub fn bucket_spectra_2d_json(spectra_json: &str, options_json: &str) -> Result<String> {
    let spectra: Vec<Spectrum2D> = from_json(spectra_json)?;
    let options: BucketOptions2D = from_json(options_json)?;
    let matrix = bucket_spectra_2d(&spectra, options)?;
    to_json(&matrix)
}
