//! Hierarchical clustering JSON helpers.

use rspin_analysis::{
    BucketMatrix1D, BucketMatrix2D, MatrixClusteringOptions, SpectrumMatrix1D, SpectrumMatrix2D,
    cluster_bucket_matrix_1d, cluster_bucket_matrix_2d, cluster_spectrum_matrix_1d,
    cluster_spectrum_matrix_2d,
};
use rspin_core::Result;

use super::{from_json, to_json};

/// Runs hierarchical clustering on a serialized one-dimensional spectrum matrix.
///
/// # Errors
///
/// Returns an error when deserialization, clustering, or serialization fails.
pub fn cluster_spectrum_matrix_1d_json(matrix_json: &str, options_json: &str) -> Result<String> {
    let matrix: SpectrumMatrix1D = from_json(matrix_json)?;
    let options: MatrixClusteringOptions = from_json(options_json)?;
    let result = cluster_spectrum_matrix_1d(&matrix, options)?;
    to_json(&result)
}

/// Runs hierarchical clustering on a serialized two-dimensional spectrum matrix.
///
/// # Errors
///
/// Returns an error when deserialization, clustering, or serialization fails.
pub fn cluster_spectrum_matrix_2d_json(matrix_json: &str, options_json: &str) -> Result<String> {
    let matrix: SpectrumMatrix2D = from_json(matrix_json)?;
    let options: MatrixClusteringOptions = from_json(options_json)?;
    let result = cluster_spectrum_matrix_2d(&matrix, options)?;
    to_json(&result)
}

/// Runs hierarchical clustering on a serialized one-dimensional bucket matrix.
///
/// # Errors
///
/// Returns an error when deserialization, clustering, or serialization fails.
pub fn cluster_bucket_matrix_1d_json(matrix_json: &str, options_json: &str) -> Result<String> {
    let matrix: BucketMatrix1D = from_json(matrix_json)?;
    let options: MatrixClusteringOptions = from_json(options_json)?;
    let result = cluster_bucket_matrix_1d(&matrix, options)?;
    to_json(&result)
}

/// Runs hierarchical clustering on a serialized two-dimensional bucket matrix.
///
/// # Errors
///
/// Returns an error when deserialization, clustering, or serialization fails.
pub fn cluster_bucket_matrix_2d_json(matrix_json: &str, options_json: &str) -> Result<String> {
    let matrix: BucketMatrix2D = from_json(matrix_json)?;
    let options: MatrixClusteringOptions = from_json(options_json)?;
    let result = cluster_bucket_matrix_2d(&matrix, options)?;
    to_json(&result)
}
