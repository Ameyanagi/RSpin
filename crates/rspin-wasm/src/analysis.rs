//! WebAssembly bindings for analysis workflows.

use wasm_bindgen::prelude::*;

use crate::{
    align_spectra_by_peak_to_matrix_1d_json, bucket_spectra_1d_json, bucket_spectra_2d_json,
    bucket_spectrum_1d_json, bucket_spectrum_2d_json, cluster_bucket_matrix_1d_json,
    cluster_bucket_matrix_2d_json, cluster_spectrum_matrix_1d_json,
    cluster_spectrum_matrix_2d_json, cut_cluster_result_at_distance_json,
    cut_cluster_result_to_count_json, detect_consensus_peaks_1d_json, detect_ranges_json,
    detect_zones_json, js_error, pairwise_bucket_matrix_1d_json, pairwise_bucket_matrix_2d_json,
    pairwise_spectrum_matrix_1d_json, pairwise_spectrum_matrix_2d_json, pca_bucket_matrix_1d_json,
    pca_bucket_matrix_2d_json, pca_spectrum_matrix_1d_json, pca_spectrum_matrix_2d_json,
};

/// Detects ranges from a serialized one-dimensional spectrum.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, analysis, or
/// serialization fails.
#[wasm_bindgen(js_name = detectRanges1d)]
pub fn detect_ranges_1d(
    spectrum_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    detect_ranges_json(spectrum_json, options_json).map_err(|error| js_error(&error))
}

/// Detects connected zones from a serialized two-dimensional spectrum.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, analysis, or
/// serialization fails.
#[wasm_bindgen(js_name = detectZones2d)]
pub fn detect_zones_2d(
    spectrum_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    detect_zones_json(spectrum_json, options_json).map_err(|error| js_error(&error))
}

/// Aligns one-dimensional spectra by peak and generates a common matrix.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, alignment, matrix
/// generation, or serialization fails.
#[wasm_bindgen(js_name = alignSpectraByPeakToMatrix1d)]
pub fn align_spectra_by_peak_to_matrix_1d(
    spectra_json: &str,
    alignment_options_json: &str,
    matrix_options_json: &str,
) -> std::result::Result<String, JsValue> {
    align_spectra_by_peak_to_matrix_1d_json(
        spectra_json,
        alignment_options_json,
        matrix_options_json,
    )
    .map_err(|error| js_error(&error))
}

/// Buckets a one-dimensional spectrum.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, bucketing, or
/// serialization fails.
#[wasm_bindgen(js_name = bucketSpectrum1d)]
pub fn bucket_spectrum_1d(
    spectrum_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    bucket_spectrum_1d_json(spectrum_json, options_json).map_err(|error| js_error(&error))
}

/// Buckets one-dimensional spectra into a row-major matrix.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, bucketing, or
/// serialization fails.
#[wasm_bindgen(js_name = bucketSpectra1d)]
pub fn bucket_spectra_1d(
    spectra_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    bucket_spectra_1d_json(spectra_json, options_json).map_err(|error| js_error(&error))
}

/// Buckets a two-dimensional spectrum.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, bucketing, or
/// serialization fails.
#[wasm_bindgen(js_name = bucketSpectrum2d)]
pub fn bucket_spectrum_2d(
    spectrum_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    bucket_spectrum_2d_json(spectrum_json, options_json).map_err(|error| js_error(&error))
}

/// Buckets two-dimensional spectra into a layer-major matrix.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, bucketing, or
/// serialization fails.
#[wasm_bindgen(js_name = bucketSpectra2d)]
pub fn bucket_spectra_2d(
    spectra_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    bucket_spectra_2d_json(spectra_json, options_json).map_err(|error| js_error(&error))
}

/// Runs hierarchical clustering on a one-dimensional spectrum matrix.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, clustering, or
/// serialization fails.
#[wasm_bindgen(js_name = clusterSpectrumMatrix1d)]
pub fn cluster_spectrum_matrix_1d(
    matrix_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    cluster_spectrum_matrix_1d_json(matrix_json, options_json).map_err(|error| js_error(&error))
}

/// Runs hierarchical clustering on a two-dimensional spectrum matrix.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, clustering, or
/// serialization fails.
#[wasm_bindgen(js_name = clusterSpectrumMatrix2d)]
pub fn cluster_spectrum_matrix_2d(
    matrix_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    cluster_spectrum_matrix_2d_json(matrix_json, options_json).map_err(|error| js_error(&error))
}

/// Runs hierarchical clustering on a one-dimensional bucket matrix.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, clustering, or
/// serialization fails.
#[wasm_bindgen(js_name = clusterBucketMatrix1d)]
pub fn cluster_bucket_matrix_1d(
    matrix_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    cluster_bucket_matrix_1d_json(matrix_json, options_json).map_err(|error| js_error(&error))
}

/// Runs hierarchical clustering on a two-dimensional bucket matrix.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, clustering, or
/// serialization fails.
#[wasm_bindgen(js_name = clusterBucketMatrix2d)]
pub fn cluster_bucket_matrix_2d(
    matrix_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    cluster_bucket_matrix_2d_json(matrix_json, options_json).map_err(|error| js_error(&error))
}

/// Cuts a clustering result to a requested number of clusters.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, dendrogram cutting,
/// or serialization fails.
#[wasm_bindgen(js_name = cutClusterResultToCount)]
pub fn cut_cluster_result_to_count(
    result_json: &str,
    cluster_count: usize,
) -> std::result::Result<String, JsValue> {
    cut_cluster_result_to_count_json(result_json, cluster_count).map_err(|error| js_error(&error))
}

/// Cuts a clustering result at a maximum linkage distance.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, dendrogram cutting,
/// or serialization fails.
#[wasm_bindgen(js_name = cutClusterResultAtDistance)]
pub fn cut_cluster_result_at_distance(
    result_json: &str,
    max_distance: f64,
) -> std::result::Result<String, JsValue> {
    cut_cluster_result_at_distance_json(result_json, max_distance).map_err(|error| js_error(&error))
}

/// Detects consensus peaks across one-dimensional spectra.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, peak detection,
/// grouping, or serialization fails.
#[wasm_bindgen(js_name = detectConsensusPeaks1d)]
pub fn detect_consensus_peaks_1d(
    spectra_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    detect_consensus_peaks_1d_json(spectra_json, options_json).map_err(|error| js_error(&error))
}

/// Runs PCA on a one-dimensional spectrum matrix.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, PCA, or
/// serialization fails.
#[wasm_bindgen(js_name = pcaSpectrumMatrix1d)]
pub fn pca_spectrum_matrix_1d(
    matrix_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    pca_spectrum_matrix_1d_json(matrix_json, options_json).map_err(|error| js_error(&error))
}

/// Runs PCA on a two-dimensional spectrum matrix.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, PCA, or
/// serialization fails.
#[wasm_bindgen(js_name = pcaSpectrumMatrix2d)]
pub fn pca_spectrum_matrix_2d(
    matrix_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    pca_spectrum_matrix_2d_json(matrix_json, options_json).map_err(|error| js_error(&error))
}

/// Runs PCA on a one-dimensional bucket matrix.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, PCA, or
/// serialization fails.
#[wasm_bindgen(js_name = pcaBucketMatrix1d)]
pub fn pca_bucket_matrix_1d(
    matrix_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    pca_bucket_matrix_1d_json(matrix_json, options_json).map_err(|error| js_error(&error))
}

/// Runs PCA on a two-dimensional bucket matrix.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, PCA, or
/// serialization fails.
#[wasm_bindgen(js_name = pcaBucketMatrix2d)]
pub fn pca_bucket_matrix_2d(
    matrix_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    pca_bucket_matrix_2d_json(matrix_json, options_json).map_err(|error| js_error(&error))
}

/// Computes pairwise values for a one-dimensional spectrum matrix.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, pairwise comparison,
/// or serialization fails.
#[wasm_bindgen(js_name = pairwiseSpectrumMatrix1d)]
pub fn pairwise_spectrum_matrix_1d(
    matrix_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    pairwise_spectrum_matrix_1d_json(matrix_json, options_json).map_err(|error| js_error(&error))
}

/// Computes pairwise values for a two-dimensional spectrum matrix.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, pairwise comparison,
/// or serialization fails.
#[wasm_bindgen(js_name = pairwiseSpectrumMatrix2d)]
pub fn pairwise_spectrum_matrix_2d(
    matrix_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    pairwise_spectrum_matrix_2d_json(matrix_json, options_json).map_err(|error| js_error(&error))
}

/// Computes pairwise values for a one-dimensional bucket matrix.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, pairwise comparison,
/// or serialization fails.
#[wasm_bindgen(js_name = pairwiseBucketMatrix1d)]
pub fn pairwise_bucket_matrix_1d(
    matrix_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    pairwise_bucket_matrix_1d_json(matrix_json, options_json).map_err(|error| js_error(&error))
}

/// Computes pairwise values for a two-dimensional bucket matrix.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, pairwise comparison,
/// or serialization fails.
#[wasm_bindgen(js_name = pairwiseBucketMatrix2d)]
pub fn pairwise_bucket_matrix_2d(
    matrix_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    pairwise_bucket_matrix_2d_json(matrix_json, options_json).map_err(|error| js_error(&error))
}
