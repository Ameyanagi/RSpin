//! Pairwise matrix comparison JSON helpers.

use rspin_analysis::{
    BucketMatrix1D, BucketMatrix2D, MatrixPairwiseOptions, SpectrumMatrix1D, SpectrumMatrix2D,
    pairwise_bucket_matrix_1d, pairwise_bucket_matrix_2d, pairwise_spectrum_matrix_1d,
    pairwise_spectrum_matrix_2d,
};
use rspin_core::Result;

use super::{from_json, to_json};

/// Computes pairwise values for a serialized one-dimensional spectrum matrix.
///
/// # Errors
///
/// Returns an error when deserialization, pairwise comparison, or
/// serialization fails.
pub fn pairwise_spectrum_matrix_1d_json(matrix_json: &str, options_json: &str) -> Result<String> {
    let matrix: SpectrumMatrix1D = from_json(matrix_json)?;
    let options: MatrixPairwiseOptions = from_json(options_json)?;
    let result = pairwise_spectrum_matrix_1d(&matrix, options)?;
    to_json(&result)
}

/// Computes pairwise values for a serialized two-dimensional spectrum matrix.
///
/// # Errors
///
/// Returns an error when deserialization, pairwise comparison, or
/// serialization fails.
pub fn pairwise_spectrum_matrix_2d_json(matrix_json: &str, options_json: &str) -> Result<String> {
    let matrix: SpectrumMatrix2D = from_json(matrix_json)?;
    let options: MatrixPairwiseOptions = from_json(options_json)?;
    let result = pairwise_spectrum_matrix_2d(&matrix, options)?;
    to_json(&result)
}

/// Computes pairwise values for a serialized one-dimensional bucket matrix.
///
/// # Errors
///
/// Returns an error when deserialization, pairwise comparison, or
/// serialization fails.
pub fn pairwise_bucket_matrix_1d_json(matrix_json: &str, options_json: &str) -> Result<String> {
    let matrix: BucketMatrix1D = from_json(matrix_json)?;
    let options: MatrixPairwiseOptions = from_json(options_json)?;
    let result = pairwise_bucket_matrix_1d(&matrix, options)?;
    to_json(&result)
}

/// Computes pairwise values for a serialized two-dimensional bucket matrix.
///
/// # Errors
///
/// Returns an error when deserialization, pairwise comparison, or
/// serialization fails.
pub fn pairwise_bucket_matrix_2d_json(matrix_json: &str, options_json: &str) -> Result<String> {
    let matrix: BucketMatrix2D = from_json(matrix_json)?;
    let options: MatrixPairwiseOptions = from_json(options_json)?;
    let result = pairwise_bucket_matrix_2d(&matrix, options)?;
    to_json(&result)
}
