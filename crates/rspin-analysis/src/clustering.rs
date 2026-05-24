//! Hierarchical clustering for multi-spectrum matrices.

use rspin_core::{RSpinError, Result};

use crate::{
    BucketMatrix1D, BucketMatrix2D, MatrixPairwiseMetric, MatrixPairwiseOptions, SpectrumMatrix1D,
    SpectrumMatrix2D, pairwise_matrix,
};

mod model;

pub use model::{
    ClusterMerge, MatrixClusterMetric, MatrixClusterResult, MatrixClusteringOptions, MatrixLinkage,
};

/// Runs agglomerative hierarchical clustering on a row-major matrix.
///
/// # Errors
///
/// Returns an error when dimensions are invalid, values are non-finite, or
/// distance calculations fail.
pub fn cluster_matrix(
    row_ids: &[String],
    values: &[f64],
    row_count: usize,
    column_count: usize,
    options: MatrixClusteringOptions,
) -> Result<MatrixClusterResult> {
    if row_count < 2 {
        return Err(RSpinError::InvalidSpectrum {
            message: "clustering requires at least two rows".to_owned(),
        });
    }
    let distances = row_distances(row_ids, values, row_count, column_count, options.metric)?;
    let mut active = (0..row_count)
        .map(|row| ClusterState {
            node_id: row,
            members: vec![row],
        })
        .collect::<Vec<_>>();
    let mut merges = Vec::with_capacity(row_count - 1);

    while active.len() > 1 {
        let selected = select_pair(&active, &distances, row_count, options.linkage)?;
        let right = active.remove(selected.right_index);
        let left = active.remove(selected.left_index);
        let size = left
            .members
            .len()
            .checked_add(right.members.len())
            .ok_or_else(|| RSpinError::InvalidSpectrum {
                message: "cluster size overflow".to_owned(),
            })?;
        let node_id =
            row_count
                .checked_add(merges.len())
                .ok_or_else(|| RSpinError::InvalidSpectrum {
                    message: "cluster node id overflow".to_owned(),
                })?;
        let mut members = left.members;
        members.extend(right.members);
        merges.push(ClusterMerge {
            left: left.node_id,
            right: right.node_id,
            distance: selected.distance,
            size,
        });
        active.push(ClusterState { node_id, members });
    }

    Ok(MatrixClusterResult {
        row_ids: row_ids.to_vec(),
        metric: options.metric,
        linkage: options.linkage,
        merges,
    })
}

/// Clusters a one-dimensional spectrum matrix.
///
/// # Errors
///
/// Returns an error when dimensions are invalid or clustering fails.
pub fn cluster_spectrum_matrix_1d(
    matrix: &SpectrumMatrix1D,
    options: MatrixClusteringOptions,
) -> Result<MatrixClusterResult> {
    let (rows, columns) = matrix.shape();
    cluster_matrix(&matrix.row_ids, &matrix.values, rows, columns, options)
}

/// Clusters a two-dimensional spectrum matrix.
///
/// # Errors
///
/// Returns an error when dimensions are invalid or clustering fails.
pub fn cluster_spectrum_matrix_2d(
    matrix: &SpectrumMatrix2D,
    options: MatrixClusteringOptions,
) -> Result<MatrixClusterResult> {
    let (layers, y_count, x_count) = matrix.shape();
    let column_count = y_count
        .checked_mul(x_count)
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "clustering matrix column count overflow".to_owned(),
        })?;
    cluster_matrix(
        &matrix.spectrum_ids,
        &matrix.values,
        layers,
        column_count,
        options,
    )
}

/// Clusters a one-dimensional bucket matrix.
///
/// # Errors
///
/// Returns an error when dimensions are invalid or clustering fails.
pub fn cluster_bucket_matrix_1d(
    matrix: &BucketMatrix1D,
    options: MatrixClusteringOptions,
) -> Result<MatrixClusterResult> {
    let (rows, columns) = matrix.shape();
    cluster_matrix(&matrix.row_ids, &matrix.values, rows, columns, options)
}

/// Clusters a two-dimensional bucket matrix.
///
/// # Errors
///
/// Returns an error when dimensions are invalid or clustering fails.
pub fn cluster_bucket_matrix_2d(
    matrix: &BucketMatrix2D,
    options: MatrixClusteringOptions,
) -> Result<MatrixClusterResult> {
    let (layers, y_count, x_count) = matrix.shape();
    let column_count = y_count
        .checked_mul(x_count)
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "clustering bucket matrix column count overflow".to_owned(),
        })?;
    cluster_matrix(
        &matrix.layer_ids,
        &matrix.values,
        layers,
        column_count,
        options,
    )
}

#[derive(Clone, Debug)]
struct ClusterState {
    node_id: usize,
    members: Vec<usize>,
}

#[derive(Clone, Copy, Debug)]
struct SelectedPair {
    left_index: usize,
    right_index: usize,
    distance: f64,
}

fn row_distances(
    row_ids: &[String],
    values: &[f64],
    row_count: usize,
    column_count: usize,
    metric: MatrixClusterMetric,
) -> Result<Vec<f64>> {
    let pairwise_metric = match metric {
        MatrixClusterMetric::EuclideanDistance => MatrixPairwiseMetric::EuclideanDistance,
        MatrixClusterMetric::ManhattanDistance => MatrixPairwiseMetric::ManhattanDistance,
        MatrixClusterMetric::PearsonDistance => MatrixPairwiseMetric::PearsonCorrelation,
        MatrixClusterMetric::CosineDistance => MatrixPairwiseMetric::CosineSimilarity,
    };
    let pairwise = pairwise_matrix(
        row_ids,
        values,
        row_count,
        column_count,
        MatrixPairwiseOptions::new().with_metric(pairwise_metric),
    )?;
    pairwise
        .values
        .into_iter()
        .map(|value| match metric {
            MatrixClusterMetric::PearsonDistance | MatrixClusterMetric::CosineDistance => {
                1.0 - value
            }
            MatrixClusterMetric::EuclideanDistance | MatrixClusterMetric::ManhattanDistance => {
                value
            }
        })
        .map(|value| {
            if value.is_finite() {
                Ok(value.max(0.0))
            } else {
                Err(RSpinError::NonFinite {
                    field: "cluster distance",
                })
            }
        })
        .collect()
}

fn select_pair(
    active: &[ClusterState],
    distances: &[f64],
    row_count: usize,
    linkage: MatrixLinkage,
) -> Result<SelectedPair> {
    let mut selected: Option<SelectedPair> = None;
    for left_index in 0..active.len() {
        for right_index in (left_index + 1)..active.len() {
            let distance = cluster_distance(
                &active[left_index],
                &active[right_index],
                distances,
                row_count,
                linkage,
            )?;
            let candidate = SelectedPair {
                left_index,
                right_index,
                distance,
            };
            if should_replace(selected, candidate, active) {
                selected = Some(candidate);
            }
        }
    }
    selected.ok_or_else(|| RSpinError::InvalidSpectrum {
        message: "no cluster pair available".to_owned(),
    })
}

fn should_replace(
    selected: Option<SelectedPair>,
    candidate: SelectedPair,
    active: &[ClusterState],
) -> bool {
    let Some(current) = selected else {
        return true;
    };
    if candidate.distance < current.distance - f64::EPSILON {
        return true;
    }
    (candidate.distance - current.distance).abs() <= f64::EPSILON
        && pair_node_ids(candidate, active) < pair_node_ids(current, active)
}

fn pair_node_ids(pair: SelectedPair, active: &[ClusterState]) -> (usize, usize) {
    (
        active[pair.left_index].node_id,
        active[pair.right_index].node_id,
    )
}

fn cluster_distance(
    left: &ClusterState,
    right: &ClusterState,
    distances: &[f64],
    row_count: usize,
    linkage: MatrixLinkage,
) -> Result<f64> {
    match linkage {
        MatrixLinkage::Single => {
            folded_member_distance(left, right, distances, row_count, f64::INFINITY, f64::min)
        }
        MatrixLinkage::Complete => {
            folded_member_distance(left, right, distances, row_count, 0.0, f64::max)
        }
        MatrixLinkage::Average => average_member_distance(left, right, distances, row_count),
    }
}

fn folded_member_distance(
    left: &ClusterState,
    right: &ClusterState,
    distances: &[f64],
    row_count: usize,
    initial: f64,
    fold: fn(f64, f64) -> f64,
) -> Result<f64> {
    let mut distance = initial;
    for &left_member in &left.members {
        for &right_member in &right.members {
            distance = fold(
                distance,
                distance_at(distances, row_count, left_member, right_member)?,
            );
        }
    }
    Ok(distance)
}

fn average_member_distance(
    left: &ClusterState,
    right: &ClusterState,
    distances: &[f64],
    row_count: usize,
) -> Result<f64> {
    let mut sum = 0.0;
    let mut count = 0_usize;
    for &left_member in &left.members {
        for &right_member in &right.members {
            sum += distance_at(distances, row_count, left_member, right_member)?;
            count = count
                .checked_add(1)
                .ok_or_else(|| RSpinError::InvalidSpectrum {
                    message: "cluster average count overflow".to_owned(),
                })?;
        }
    }
    let count = u32::try_from(count).map_err(|_| RSpinError::InvalidSpectrum {
        message: "cluster average count is too large".to_owned(),
    })?;
    Ok(sum / f64::from(count))
}

fn distance_at(distances: &[f64], row_count: usize, left: usize, right: usize) -> Result<f64> {
    distances
        .get(
            left.checked_mul(row_count)
                .and_then(|offset| offset.checked_add(right))
                .ok_or_else(|| RSpinError::InvalidSpectrum {
                    message: "cluster distance index overflow".to_owned(),
                })?,
        )
        .copied()
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "cluster distance index out of bounds".to_owned(),
        })
}

#[cfg(test)]
mod tests;
