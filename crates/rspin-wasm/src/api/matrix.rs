//! Matrix JSON helpers.

use rspin_analysis::{
    MatrixGeneration2DOptions, MatrixGenerationOptions, PeakAlignmentOptions, ZoneAlignmentOptions,
    align_spectra_by_peak_to_matrix, align_spectra_by_zone_to_matrix, generate_spectrum_matrix_1d,
    generate_spectrum_matrix_2d,
};
use rspin_core::{Result, Spectrum1D, Spectrum2D};

use super::{from_json, to_json};

/// Generates a row-major matrix from serialized `Spectrum1D` JSON values.
///
/// # Errors
///
/// Returns an error when deserialization, analysis, or serialization fails.
pub fn generate_spectrum_matrix_1d_json(spectra_json: &str, options_json: &str) -> Result<String> {
    let spectra: Vec<Spectrum1D> = from_json(spectra_json)?;
    let options: MatrixGenerationOptions = from_json(options_json)?;
    let matrix = generate_spectrum_matrix_1d(&spectra, options)?;
    to_json(&matrix)
}

/// Aligns serialized `Spectrum1D` JSON values by peak and generates a matrix.
///
/// # Errors
///
/// Returns an error when deserialization, alignment, matrix generation, or
/// serialization fails.
pub fn align_spectra_by_peak_to_matrix_1d_json(
    spectra_json: &str,
    alignment_options_json: &str,
    matrix_options_json: &str,
) -> Result<String> {
    let spectra: Vec<Spectrum1D> = from_json(spectra_json)?;
    let alignment_options: PeakAlignmentOptions = from_json(alignment_options_json)?;
    let matrix_options: MatrixGenerationOptions = from_json(matrix_options_json)?;
    let result = align_spectra_by_peak_to_matrix(&spectra, alignment_options, matrix_options)?;
    to_json(&result)
}

/// Generates a layer-major matrix from serialized `Spectrum2D` JSON values.
///
/// # Errors
///
/// Returns an error when deserialization, analysis, or serialization fails.
pub fn generate_spectrum_matrix_2d_json(spectra_json: &str, options_json: &str) -> Result<String> {
    let spectra: Vec<Spectrum2D> = from_json(spectra_json)?;
    let options: MatrixGeneration2DOptions = from_json(options_json)?;
    let matrix = generate_spectrum_matrix_2d(&spectra, options)?;
    to_json(&matrix)
}

/// Aligns serialized `Spectrum2D` JSON values by zone and generates a matrix.
///
/// # Errors
///
/// Returns an error when deserialization, alignment, matrix generation, or
/// serialization fails.
pub fn align_spectra_by_zone_to_matrix_2d_json(
    spectra_json: &str,
    alignment_options_json: &str,
    matrix_options_json: &str,
) -> Result<String> {
    let spectra: Vec<Spectrum2D> = from_json(spectra_json)?;
    let alignment_options: ZoneAlignmentOptions = from_json(alignment_options_json)?;
    let matrix_options: MatrixGeneration2DOptions = from_json(matrix_options_json)?;
    let result = align_spectra_by_zone_to_matrix(&spectra, alignment_options, matrix_options)?;
    to_json(&result)
}
