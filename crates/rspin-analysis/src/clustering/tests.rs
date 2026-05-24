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

fn row_ids() -> Vec<String> {
    vec!["a".to_owned(), "b".to_owned(), "c".to_owned()]
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1.0e-12,
        "{actual} != {expected}"
    );
}
