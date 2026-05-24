//! Pairwise comparison for multi-spectrum matrices.

use serde::{Deserialize, Serialize};

use rspin_core::{RSpinError, Result};

use crate::{BucketMatrix1D, BucketMatrix2D, SpectrumMatrix1D, SpectrumMatrix2D};

/// Pairwise metric computed between matrix rows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatrixPairwiseMetric {
    /// Pearson correlation between row vectors.
    #[default]
    PearsonCorrelation,
    /// Cosine similarity between row vectors.
    CosineSimilarity,
    /// Euclidean distance between row vectors.
    EuclideanDistance,
    /// Manhattan distance between row vectors.
    ManhattanDistance,
}

/// Options for pairwise matrix comparison.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatrixPairwiseOptions {
    /// Metric computed between every pair of rows.
    pub metric: MatrixPairwiseMetric,
}

impl MatrixPairwiseOptions {
    /// Creates default pairwise comparison options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the pairwise metric.
    #[must_use]
    pub fn with_metric(mut self, metric: MatrixPairwiseMetric) -> Self {
        self.metric = metric;
        self
    }
}

/// Row-major pairwise matrix.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MatrixPairwiseResult {
    /// Row identifiers copied from the source matrix.
    pub row_ids: Vec<String>,
    /// Metric used to compute pairwise values.
    pub metric: MatrixPairwiseMetric,
    /// Row-major pairwise values: `row_ids.len() * row_ids.len()`.
    pub values: Vec<f64>,
}

impl MatrixPairwiseResult {
    /// Returns the pairwise matrix shape as `(rows, rows)`.
    #[must_use]
    pub fn shape(&self) -> (usize, usize) {
        (self.row_ids.len(), self.row_ids.len())
    }

    /// Returns one pairwise value, or `None` when out of bounds.
    #[must_use]
    pub fn value_at(&self, row_index: usize, column_index: usize) -> Option<f64> {
        let (rows, columns) = self.shape();
        if row_index >= rows || column_index >= columns {
            return None;
        }
        self.values
            .get(row_index.checked_mul(columns)?.checked_add(column_index)?)
            .copied()
    }
}

/// Computes pairwise values for a row-major matrix.
///
/// # Errors
///
/// Returns an error when dimensions are invalid or values are non-finite.
pub fn pairwise_matrix(
    row_ids: &[String],
    values: &[f64],
    row_count: usize,
    column_count: usize,
    options: MatrixPairwiseOptions,
) -> Result<MatrixPairwiseResult> {
    validate_matrix(row_ids, values, row_count, column_count)?;
    let column_count_f64 = count_to_f64("pairwise column count", column_count)?;
    let output_len =
        row_count
            .checked_mul(row_count)
            .ok_or_else(|| RSpinError::InvalidSpectrum {
                message: "pairwise matrix size overflow".to_owned(),
            })?;
    let mut pairwise_values = Vec::with_capacity(output_len);

    for row_index in 0..row_count {
        let left = row(values, row_index, column_count);
        for column_index in 0..row_count {
            let right = row(values, column_index, column_count);
            pairwise_values.push(metric_value(left, right, options.metric, column_count_f64));
        }
    }

    Ok(MatrixPairwiseResult {
        row_ids: row_ids.to_vec(),
        metric: options.metric,
        values: pairwise_values,
    })
}

/// Computes pairwise values for a one-dimensional spectrum matrix.
///
/// # Errors
///
/// Returns an error when dimensions are invalid or values are non-finite.
pub fn pairwise_spectrum_matrix_1d(
    matrix: &SpectrumMatrix1D,
    options: MatrixPairwiseOptions,
) -> Result<MatrixPairwiseResult> {
    let (rows, columns) = matrix.shape();
    pairwise_matrix(&matrix.row_ids, &matrix.values, rows, columns, options)
}

/// Computes pairwise values for a two-dimensional spectrum matrix.
///
/// # Errors
///
/// Returns an error when dimensions are invalid or values are non-finite.
pub fn pairwise_spectrum_matrix_2d(
    matrix: &SpectrumMatrix2D,
    options: MatrixPairwiseOptions,
) -> Result<MatrixPairwiseResult> {
    let (layers, y_count, x_count) = matrix.shape();
    let column_count = y_count
        .checked_mul(x_count)
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "pairwise matrix column count overflow".to_owned(),
        })?;
    pairwise_matrix(
        &matrix.spectrum_ids,
        &matrix.values,
        layers,
        column_count,
        options,
    )
}

/// Computes pairwise values for a one-dimensional bucket matrix.
///
/// # Errors
///
/// Returns an error when dimensions are invalid or values are non-finite.
pub fn pairwise_bucket_matrix_1d(
    matrix: &BucketMatrix1D,
    options: MatrixPairwiseOptions,
) -> Result<MatrixPairwiseResult> {
    let (rows, columns) = matrix.shape();
    pairwise_matrix(&matrix.row_ids, &matrix.values, rows, columns, options)
}

/// Computes pairwise values for a two-dimensional bucket matrix.
///
/// # Errors
///
/// Returns an error when dimensions are invalid or values are non-finite.
pub fn pairwise_bucket_matrix_2d(
    matrix: &BucketMatrix2D,
    options: MatrixPairwiseOptions,
) -> Result<MatrixPairwiseResult> {
    let (layers, y_count, x_count) = matrix.shape();
    let column_count = y_count
        .checked_mul(x_count)
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "pairwise bucket matrix column count overflow".to_owned(),
        })?;
    pairwise_matrix(
        &matrix.layer_ids,
        &matrix.values,
        layers,
        column_count,
        options,
    )
}

fn validate_matrix(
    row_ids: &[String],
    values: &[f64],
    row_count: usize,
    column_count: usize,
) -> Result<()> {
    if row_count == 0 {
        return Err(RSpinError::InvalidSpectrum {
            message: "pairwise comparison requires at least one row".to_owned(),
        });
    }
    if column_count == 0 {
        return Err(RSpinError::InvalidSpectrum {
            message: "pairwise comparison requires at least one column".to_owned(),
        });
    }
    if row_ids.len() != row_count {
        return Err(RSpinError::InvalidSpectrum {
            message: "pairwise row id count must match row count".to_owned(),
        });
    }
    let expected_len =
        row_count
            .checked_mul(column_count)
            .ok_or_else(|| RSpinError::InvalidSpectrum {
                message: "pairwise matrix size overflow".to_owned(),
            })?;
    if values.len() != expected_len {
        return Err(RSpinError::InvalidSpectrum {
            message: "pairwise matrix value count must match dimensions".to_owned(),
        });
    }
    if !values.iter().all(|value| value.is_finite()) {
        return Err(RSpinError::NonFinite {
            field: "pairwise matrix values",
        });
    }
    Ok(())
}

fn metric_value(
    left: &[f64],
    right: &[f64],
    metric: MatrixPairwiseMetric,
    column_count: f64,
) -> f64 {
    match metric {
        MatrixPairwiseMetric::PearsonCorrelation => pearson_correlation(left, right, column_count),
        MatrixPairwiseMetric::CosineSimilarity => cosine_similarity(left, right),
        MatrixPairwiseMetric::EuclideanDistance => euclidean_distance(left, right),
        MatrixPairwiseMetric::ManhattanDistance => manhattan_distance(left, right),
    }
}

fn pearson_correlation(left: &[f64], right: &[f64], column_count: f64) -> f64 {
    let left_mean = left.iter().sum::<f64>() / column_count;
    let right_mean = right.iter().sum::<f64>() / column_count;
    let mut dot = 0.0;
    let mut left_norm = 0.0;
    let mut right_norm = 0.0;
    for (&left_value, &right_value) in left.iter().zip(right) {
        let left_delta = left_value - left_mean;
        let right_delta = right_value - right_mean;
        dot += left_delta * right_delta;
        left_norm += left_delta * left_delta;
        right_norm += right_delta * right_delta;
    }
    normalized_dot(dot, left_norm, right_norm, left, right)
}

fn cosine_similarity(left: &[f64], right: &[f64]) -> f64 {
    let mut dot = 0.0;
    let mut left_norm = 0.0;
    let mut right_norm = 0.0;
    for (&left_value, &right_value) in left.iter().zip(right) {
        dot += left_value * right_value;
        left_norm += left_value * left_value;
        right_norm += right_value * right_value;
    }
    normalized_dot(dot, left_norm, right_norm, left, right)
}

fn normalized_dot(dot: f64, left_norm: f64, right_norm: f64, left: &[f64], right: &[f64]) -> f64 {
    let denominator = left_norm.sqrt() * right_norm.sqrt();
    if denominator > f64::EPSILON {
        return dot / denominator;
    }
    if rows_equal(left, right) { 1.0 } else { 0.0 }
}

fn euclidean_distance(left: &[f64], right: &[f64]) -> f64 {
    left.iter()
        .zip(right)
        .map(|(&left_value, &right_value)| {
            let delta = left_value - right_value;
            delta * delta
        })
        .sum::<f64>()
        .sqrt()
}

fn manhattan_distance(left: &[f64], right: &[f64]) -> f64 {
    left.iter()
        .zip(right)
        .map(|(&left_value, &right_value)| (left_value - right_value).abs())
        .sum()
}

fn rows_equal(left: &[f64], right: &[f64]) -> bool {
    left.iter()
        .zip(right)
        .all(|(&left_value, &right_value)| (left_value - right_value).abs() <= f64::EPSILON)
}

fn row(values: &[f64], row_index: usize, column_count: usize) -> &[f64] {
    let start = row_index * column_count;
    &values[start..start + column_count]
}

fn count_to_f64(field: &'static str, value: usize) -> Result<f64> {
    let value = u32::try_from(value).map_err(|_| RSpinError::InvalidSpectrum {
        message: format!("{field} is too large"),
    })?;
    Ok(f64::from(value))
}

#[cfg(test)]
mod tests;
