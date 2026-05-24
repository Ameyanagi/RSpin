//! PCA JSON helpers.

use rspin_analysis::{
    BucketMatrix1D, BucketMatrix2D, MatrixPcaOptions, SpectrumMatrix1D, SpectrumMatrix2D,
    pca_bucket_matrix_1d, pca_bucket_matrix_2d, pca_spectrum_matrix_1d, pca_spectrum_matrix_2d,
};
use rspin_core::Result;

use super::{from_json, to_json};

/// Runs PCA on a serialized one-dimensional spectrum matrix.
///
/// # Errors
///
/// Returns an error when deserialization, PCA, or serialization fails.
pub fn pca_spectrum_matrix_1d_json(matrix_json: &str, options_json: &str) -> Result<String> {
    let matrix: SpectrumMatrix1D = from_json(matrix_json)?;
    let options: MatrixPcaOptions = from_json(options_json)?;
    let result = pca_spectrum_matrix_1d(&matrix, options)?;
    to_json(&result)
}

/// Runs PCA on a serialized two-dimensional spectrum matrix.
///
/// # Errors
///
/// Returns an error when deserialization, PCA, or serialization fails.
pub fn pca_spectrum_matrix_2d_json(matrix_json: &str, options_json: &str) -> Result<String> {
    let matrix: SpectrumMatrix2D = from_json(matrix_json)?;
    let options: MatrixPcaOptions = from_json(options_json)?;
    let result = pca_spectrum_matrix_2d(&matrix, options)?;
    to_json(&result)
}

/// Runs PCA on a serialized one-dimensional bucket matrix.
///
/// # Errors
///
/// Returns an error when deserialization, PCA, or serialization fails.
pub fn pca_bucket_matrix_1d_json(matrix_json: &str, options_json: &str) -> Result<String> {
    let matrix: BucketMatrix1D = from_json(matrix_json)?;
    let options: MatrixPcaOptions = from_json(options_json)?;
    let result = pca_bucket_matrix_1d(&matrix, options)?;
    to_json(&result)
}

/// Runs PCA on a serialized two-dimensional bucket matrix.
///
/// # Errors
///
/// Returns an error when deserialization, PCA, or serialization fails.
pub fn pca_bucket_matrix_2d_json(matrix_json: &str, options_json: &str) -> Result<String> {
    let matrix: BucketMatrix2D = from_json(matrix_json)?;
    let options: MatrixPcaOptions = from_json(options_json)?;
    let result = pca_bucket_matrix_2d(&matrix, options)?;
    to_json(&result)
}
