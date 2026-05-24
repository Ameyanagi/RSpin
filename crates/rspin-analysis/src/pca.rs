//! Principal component analysis for multi-spectrum matrices.

use std::cmp::Ordering;

use nalgebra::DMatrix;

use rspin_core::{RSpinError, Result};

use crate::{BucketMatrix1D, BucketMatrix2D, SpectrumMatrix1D, SpectrumMatrix2D};

mod model;

pub use model::{MatrixPcaOptions, MatrixPcaResult, MatrixScaling};

/// Runs PCA on a row-major numeric matrix.
///
/// # Errors
///
/// Returns an error when dimensions are invalid, values are non-finite, or
/// options are invalid.
pub fn pca_matrix(
    row_ids: &[String],
    values: &[f64],
    row_count: usize,
    column_count: usize,
    options: MatrixPcaOptions,
) -> Result<MatrixPcaResult> {
    options.validate()?;
    validate_matrix(row_ids, values, row_count, column_count)?;
    let row_count_denominator = count_to_f64("PCA row count", row_count)?;
    let sample_denominator = count_to_f64("PCA sample denominator", row_count - 1)?;

    let means = column_means(
        values,
        row_count,
        column_count,
        options.center,
        row_count_denominator,
    );
    let standard_deviations = column_standard_deviations(
        values,
        row_count,
        column_count,
        row_count_denominator,
        sample_denominator,
    );
    let scales = column_scales(&standard_deviations, options.scaling);
    let transformed = transformed_matrix(values, row_count, column_count, &means, &scales);
    let covariance = (&transformed.transpose() * &transformed) / sample_denominator;
    let eigen = covariance.symmetric_eigen();
    let component_count = options.component_count.min(column_count);
    let eigen_indices = sorted_eigen_indices(eigen.eigenvalues.as_slice());
    let total_variance = eigen
        .eigenvalues
        .iter()
        .copied()
        .filter(|value| *value > 0.0)
        .sum::<f64>();

    let mut loadings = Vec::with_capacity(component_count * column_count);
    let mut explained_variance = Vec::with_capacity(component_count);
    let mut explained_variance_ratio = Vec::with_capacity(component_count);

    for source_index in eigen_indices.into_iter().take(component_count) {
        let sign = component_sign(&eigen.eigenvectors, source_index, column_count);
        loadings.extend(
            (0..column_count).map(|row_index| eigen.eigenvectors[(row_index, source_index)] * sign),
        );
        let variance = eigen.eigenvalues[source_index].max(0.0);
        explained_variance.push(variance);
        explained_variance_ratio.push(if total_variance > f64::EPSILON {
            variance / total_variance
        } else {
            0.0
        });
    }

    let scores = scores(
        &transformed,
        row_count,
        column_count,
        component_count,
        &loadings,
    );

    Ok(MatrixPcaResult {
        row_ids: row_ids.to_vec(),
        column_count,
        component_count,
        means,
        scales,
        scores,
        loadings,
        explained_variance,
        explained_variance_ratio,
    })
}

/// Runs PCA on a generated one-dimensional spectrum matrix.
///
/// # Errors
///
/// Returns an error when options or matrix dimensions are invalid.
pub fn pca_spectrum_matrix_1d(
    matrix: &SpectrumMatrix1D,
    options: MatrixPcaOptions,
) -> Result<MatrixPcaResult> {
    let (rows, columns) = matrix.shape();
    pca_matrix(&matrix.row_ids, &matrix.values, rows, columns, options)
}

/// Runs PCA on a generated two-dimensional spectrum matrix.
///
/// # Errors
///
/// Returns an error when options or matrix dimensions are invalid.
pub fn pca_spectrum_matrix_2d(
    matrix: &SpectrumMatrix2D,
    options: MatrixPcaOptions,
) -> Result<MatrixPcaResult> {
    let (layers, y_count, x_count) = matrix.shape();
    let column_count = y_count
        .checked_mul(x_count)
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "PCA matrix column count overflow".to_owned(),
        })?;
    pca_matrix(
        &matrix.spectrum_ids,
        &matrix.values,
        layers,
        column_count,
        options,
    )
}

/// Runs PCA on a one-dimensional bucket matrix.
///
/// # Errors
///
/// Returns an error when options or matrix dimensions are invalid.
pub fn pca_bucket_matrix_1d(
    matrix: &BucketMatrix1D,
    options: MatrixPcaOptions,
) -> Result<MatrixPcaResult> {
    let (rows, columns) = matrix.shape();
    pca_matrix(&matrix.row_ids, &matrix.values, rows, columns, options)
}

/// Runs PCA on a two-dimensional bucket matrix.
///
/// # Errors
///
/// Returns an error when options or matrix dimensions are invalid.
pub fn pca_bucket_matrix_2d(
    matrix: &BucketMatrix2D,
    options: MatrixPcaOptions,
) -> Result<MatrixPcaResult> {
    let (layers, y_count, x_count) = matrix.shape();
    let column_count = y_count
        .checked_mul(x_count)
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "PCA bucket matrix column count overflow".to_owned(),
        })?;
    pca_matrix(
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
    if row_count < 2 {
        return Err(RSpinError::InvalidSpectrum {
            message: "PCA requires at least two rows".to_owned(),
        });
    }
    if column_count == 0 {
        return Err(RSpinError::InvalidSpectrum {
            message: "PCA requires at least one column".to_owned(),
        });
    }
    if row_ids.len() != row_count {
        return Err(RSpinError::InvalidSpectrum {
            message: "PCA row id count must match row count".to_owned(),
        });
    }
    let expected_len =
        row_count
            .checked_mul(column_count)
            .ok_or_else(|| RSpinError::InvalidSpectrum {
                message: "PCA matrix size overflow".to_owned(),
            })?;
    if values.len() != expected_len {
        return Err(RSpinError::InvalidSpectrum {
            message: "PCA matrix value count must match dimensions".to_owned(),
        });
    }
    if !values.iter().all(|value| value.is_finite()) {
        return Err(RSpinError::NonFinite {
            field: "PCA matrix values",
        });
    }
    Ok(())
}

fn column_means(
    values: &[f64],
    row_count: usize,
    column_count: usize,
    center: bool,
    denominator: f64,
) -> Vec<f64> {
    if !center {
        return vec![0.0; column_count];
    }
    (0..column_count)
        .map(|column| {
            (0..row_count)
                .map(|row| values[row * column_count + column])
                .sum::<f64>()
                / denominator
        })
        .collect()
}

fn column_standard_deviations(
    values: &[f64],
    row_count: usize,
    column_count: usize,
    row_count_denominator: f64,
    sample_denominator: f64,
) -> Vec<f64> {
    let means = column_means(values, row_count, column_count, true, row_count_denominator);
    (0..column_count)
        .map(|column| {
            let sum_squares = (0..row_count)
                .map(|row| {
                    let delta = values[row * column_count + column] - means[column];
                    delta * delta
                })
                .sum::<f64>();
            (sum_squares / sample_denominator).sqrt()
        })
        .collect()
}

fn column_scales(standard_deviations: &[f64], scaling: MatrixScaling) -> Vec<f64> {
    standard_deviations
        .iter()
        .copied()
        .map(|standard_deviation| {
            if standard_deviation <= f64::EPSILON {
                return 1.0;
            }
            match scaling {
                MatrixScaling::None => 1.0,
                MatrixScaling::UnitVariance => standard_deviation,
                MatrixScaling::Pareto => standard_deviation.sqrt(),
            }
        })
        .collect()
}

fn transformed_matrix(
    values: &[f64],
    row_count: usize,
    column_count: usize,
    means: &[f64],
    scales: &[f64],
) -> DMatrix<f64> {
    DMatrix::from_fn(row_count, column_count, |row, column| {
        (values[row * column_count + column] - means[column]) / scales[column]
    })
}

fn sorted_eigen_indices(eigenvalues: &[f64]) -> Vec<usize> {
    let mut indices = (0..eigenvalues.len()).collect::<Vec<_>>();
    indices.sort_by(
        |left, right| match eigenvalues[*right].partial_cmp(&eigenvalues[*left]) {
            Some(ordering) => ordering,
            None => Ordering::Equal,
        },
    );
    indices
}

fn component_sign(eigenvectors: &DMatrix<f64>, column: usize, row_count: usize) -> f64 {
    let mut largest_abs = 0.0;
    let mut sign = 1.0;
    for row in 0..row_count {
        let value = eigenvectors[(row, column)];
        let absolute = value.abs();
        if absolute > largest_abs {
            largest_abs = absolute;
            sign = if value < 0.0 { -1.0 } else { 1.0 };
        }
    }
    sign
}

fn scores(
    transformed: &DMatrix<f64>,
    row_count: usize,
    column_count: usize,
    component_count: usize,
    loadings: &[f64],
) -> Vec<f64> {
    let mut scores = Vec::with_capacity(row_count * component_count);
    for row in 0..row_count {
        for component in 0..component_count {
            scores.push(
                (0..column_count)
                    .map(|column| {
                        transformed[(row, column)] * loadings[component * column_count + column]
                    })
                    .sum(),
            );
        }
    }
    scores
}

fn count_to_f64(field: &'static str, value: usize) -> Result<f64> {
    let value = u32::try_from(value).map_err(|_| RSpinError::InvalidSpectrum {
        message: format!("{field} is too large"),
    })?;
    Ok(f64::from(value))
}

#[cfg(test)]
mod tests;
