use rspin_core::{Axis, Unit};

use super::*;
use crate::{BucketMatrix1D, BucketMatrix2D, IntegralRegion, IntegralRegion2D};

#[test]
fn clusters_rows_with_single_linkage() -> anyhow::Result<()> {
    let result = cluster_matrix(
        &row_ids(),
        &[0.0, 2.0, 5.0],
        3,
        1,
        MatrixClusteringOptions::new(),
    )?;

    assert_eq!(result.leaf_count(), 3);
    assert_eq!(result.root_node_id(), Some(4));
    assert_eq!(result.merges.len(), 2);
    assert_eq!(result.merges[0].left, 0);
    assert_eq!(result.merges[0].right, 1);
    assert_close(result.merges[0].distance, 2.0);
    assert_eq!(result.merges[0].size, 2);
    assert_eq!(result.merges[1].left, 2);
    assert_eq!(result.merges[1].right, 3);
    assert_close(result.merges[1].distance, 3.0);
    assert_eq!(result.merges[1].size, 3);
    Ok(())
}

#[test]
fn supports_complete_and_average_linkage() -> anyhow::Result<()> {
    let complete = cluster_matrix(
        &row_ids(),
        &[0.0, 2.0, 5.0],
        3,
        1,
        MatrixClusteringOptions::new().with_linkage(MatrixLinkage::Complete),
    )?;
    assert_close(complete.merges[1].distance, 5.0);

    let average = cluster_matrix(
        &row_ids(),
        &[0.0, 2.0, 5.0],
        3,
        1,
        MatrixClusteringOptions::new().with_linkage(MatrixLinkage::Average),
    )?;
    assert_close(average.merges[1].distance, 4.0);
    Ok(())
}

#[test]
fn supports_correlation_and_cosine_distances() -> anyhow::Result<()> {
    let correlation = cluster_matrix(
        &row_ids(),
        &[1.0, 2.0, 2.0, 4.0, 2.0, 1.0],
        3,
        2,
        MatrixClusteringOptions::new().with_metric(MatrixClusterMetric::PearsonDistance),
    )?;
    assert_close(correlation.merges[0].distance, 0.0);
    assert_close(correlation.merges[1].distance, 2.0);

    let cosine = cluster_matrix(
        &row_ids(),
        &[3.0, 4.0, 6.0, 8.0, 0.0, 0.0],
        3,
        2,
        MatrixClusteringOptions::new().with_metric(MatrixClusterMetric::CosineDistance),
    )?;
    assert_close(cosine.merges[0].distance, 0.0);
    assert_close(cosine.merges[1].distance, 1.0);
    Ok(())
}

#[test]
fn accepts_spectrum_and_bucket_matrices() -> anyhow::Result<()> {
    let spectrum_matrix = SpectrumMatrix1D {
        axis: Axis::linear("x", Unit::Ppm, 0.0, 1.0, 1)?,
        row_ids: row_ids(),
        values: vec![0.0, 2.0, 5.0],
    };
    let spectrum_result =
        cluster_spectrum_matrix_1d(&spectrum_matrix, MatrixClusteringOptions::new())?;
    assert_eq!(spectrum_result.merges.len(), 2);

    let bucket_matrix = BucketMatrix1D {
        regions: vec![IntegralRegion { from: 0.0, to: 1.0 }],
        row_ids: row_ids(),
        values: vec![0.0, 2.0, 5.0],
    };
    let bucket_result = cluster_bucket_matrix_1d(&bucket_matrix, MatrixClusteringOptions::new())?;
    assert_eq!(bucket_result.merges.len(), 2);
    Ok(())
}

#[test]
fn accepts_two_dimensional_matrix_flats() -> anyhow::Result<()> {
    let bucket_matrix = BucketMatrix2D {
        regions: vec![IntegralRegion2D {
            x_from: 0.0,
            x_to: 1.0,
            y_from: 0.0,
            y_to: 1.0,
        }],
        x_bucket_count: 1,
        y_bucket_count: 1,
        layer_ids: row_ids(),
        values: vec![0.0, 2.0, 5.0],
    };
    let result = cluster_bucket_matrix_2d(&bucket_matrix, MatrixClusteringOptions::new())?;

    assert_eq!(result.leaf_count(), 3);
    assert_eq!(result.merges.len(), 2);
    Ok(())
}

#[test]
fn cuts_dendrogram_to_cluster_count_or_distance() -> anyhow::Result<()> {
    let result = cluster_matrix(
        &four_row_ids(),
        &[0.0, 1.0, 10.0, 11.0],
        4,
        1,
        MatrixClusteringOptions::new(),
    )?;

    let count_cut = result.cut_to_cluster_count(2)?;
    assert_eq!(count_cut.row_count(), 4);
    assert_eq!(count_cut.cluster_count, 2);
    assert_eq!(count_cut.cluster_ids, vec![0, 0, 1, 1]);
    assert_eq!(count_cut.cluster_id_at(2), Some(1));

    let distance_cut = result.cut_at_distance(1.0)?;
    assert_eq!(distance_cut.cluster_ids, vec![0, 0, 1, 1]);

    let leaf_cut = result.cut_at_distance(0.5)?;
    assert_eq!(leaf_cut.cluster_count, 4);
    assert_eq!(leaf_cut.cluster_ids, vec![0, 1, 2, 3]);

    let root_cut = result.cut_to_cluster_count(1)?;
    assert_eq!(root_cut.cluster_ids, vec![0, 0, 0, 0]);
    Ok(())
}

#[test]
fn rejects_invalid_inputs() {
    let one_row_error = cluster_matrix(
        &[String::from("only")],
        &[1.0],
        1,
        1,
        MatrixClusteringOptions::new(),
    )
    .expect_err("single row should fail");
    assert!(matches!(one_row_error, RSpinError::InvalidSpectrum { .. }));

    let non_finite_error = cluster_matrix(
        &[String::from("a"), String::from("b")],
        &[1.0, f64::NAN],
        2,
        1,
        MatrixClusteringOptions::new(),
    )
    .expect_err("non-finite value should fail");
    assert!(matches!(non_finite_error, RSpinError::NonFinite { .. }));
}

#[test]
fn rejects_invalid_cluster_cuts() -> anyhow::Result<()> {
    let result = cluster_matrix(
        &row_ids(),
        &[0.0, 2.0, 5.0],
        3,
        1,
        MatrixClusteringOptions::new(),
    )?;

    let zero_count_error = result
        .cut_to_cluster_count(0)
        .expect_err("zero clusters should fail");
    assert!(matches!(
        zero_count_error,
        RSpinError::InvalidSpectrum { .. }
    ));

    let too_many_error = result
        .cut_to_cluster_count(4)
        .expect_err("too many clusters should fail");
    assert!(matches!(too_many_error, RSpinError::InvalidSpectrum { .. }));

    let non_finite_distance_error = result
        .cut_at_distance(f64::NAN)
        .expect_err("non-finite cut distance should fail");
    assert!(matches!(
        non_finite_distance_error,
        RSpinError::NonFinite { .. }
    ));

    let malformed = MatrixClusterResult {
        row_ids: row_ids(),
        metric: MatrixClusterMetric::EuclideanDistance,
        linkage: MatrixLinkage::Single,
        merges: vec![ClusterMerge {
            left: 0,
            right: 1,
            distance: 1.0,
            size: 3,
        }],
    };
    let malformed_error = malformed
        .cut_to_cluster_count(2)
        .expect_err("malformed merge should fail");
    assert!(matches!(
        malformed_error,
        RSpinError::InvalidSpectrum { .. }
    ));

    Ok(())
}

fn row_ids() -> Vec<String> {
    vec!["a".to_owned(), "b".to_owned(), "c".to_owned()]
}

fn four_row_ids() -> Vec<String> {
    vec![
        "a".to_owned(),
        "b".to_owned(),
        "c".to_owned(),
        "d".to_owned(),
    ]
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1.0e-12,
        "{actual} != {expected}"
    );
}
