//! Hierarchical clustering JSON helpers.

use rspin_analysis::{
    BucketMatrix1D, BucketMatrix2D, MatrixClusterResult, MatrixClusteringOptions, SpectrumMatrix1D,
    SpectrumMatrix2D, cluster_bucket_matrix_1d, cluster_bucket_matrix_2d,
    cluster_spectrum_matrix_1d, cluster_spectrum_matrix_2d,
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

/// Cuts a serialized clustering result to a requested number of clusters.
///
/// # Errors
///
/// Returns an error when deserialization, dendrogram cutting, or serialization
/// fails.
pub fn cut_cluster_result_to_count_json(result_json: &str, cluster_count: usize) -> Result<String> {
    let result: MatrixClusterResult = from_json(result_json)?;
    let cut = result.cut_to_cluster_count(cluster_count)?;
    to_json(&cut)
}

/// Cuts a serialized clustering result at a maximum linkage distance.
///
/// # Errors
///
/// Returns an error when deserialization, dendrogram cutting, or serialization
/// fails.
pub fn cut_cluster_result_at_distance_json(result_json: &str, max_distance: f64) -> Result<String> {
    let result: MatrixClusterResult = from_json(result_json)?;
    let cut = result.cut_at_distance(max_distance)?;
    to_json(&cut)
}
