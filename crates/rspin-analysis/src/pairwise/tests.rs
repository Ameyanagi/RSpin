use rspin_core::{Axis, Unit};

use super::*;
use crate::{BucketMatrix1D, BucketMatrix2D, IntegralRegion, IntegralRegion2D};

#[test]
fn computes_pearson_correlation_matrix() -> anyhow::Result<()> {
    let result = pairwise_matrix(
        &row_ids(),
        &[1.0, 2.0, 2.0, 4.0, 2.0, 1.0],
        3,
        2,
        MatrixPairwiseOptions::new(),
    )?;

    assert_eq!(result.shape(), (3, 3));
    assert_eq!(result.metric, MatrixPairwiseMetric::PearsonCorrelation);
    assert_close(
        result
            .value_at(0, 0)
            .ok_or(anyhow::anyhow!("missing value"))?,
        1.0,
    );
    assert_close(
        result
            .value_at(0, 1)
            .ok_or(anyhow::anyhow!("missing value"))?,
        1.0,
    );
    assert_close(
        result
            .value_at(0, 2)
            .ok_or(anyhow::anyhow!("missing value"))?,
        -1.0,
    );
    assert_eq!(result.value_at(3, 0), None);
    Ok(())
}

#[test]
fn computes_cosine_and_distances() -> anyhow::Result<()> {
    let values = [3.0, 4.0, 0.0, 0.0];
    let row_ids = [String::from("a"), String::from("zero")];
    let cosine = pairwise_matrix(
        &row_ids,
        &values,
        2,
        2,
        MatrixPairwiseOptions::new().with_metric(MatrixPairwiseMetric::CosineSimilarity),
    )?;
    assert_close(
        cosine
            .value_at(0, 0)
            .ok_or(anyhow::anyhow!("missing value"))?,
        1.0,
    );
    assert_close(
        cosine
            .value_at(1, 1)
            .ok_or(anyhow::anyhow!("missing value"))?,
        1.0,
    );
    assert_close(
        cosine
            .value_at(0, 1)
            .ok_or(anyhow::anyhow!("missing value"))?,
        0.0,
    );

    let euclidean = pairwise_matrix(
        &row_ids,
        &values,
        2,
        2,
        MatrixPairwiseOptions::new().with_metric(MatrixPairwiseMetric::EuclideanDistance),
    )?;
    assert_close(
        euclidean
            .value_at(0, 1)
            .ok_or(anyhow::anyhow!("missing value"))?,
        5.0,
    );

    let manhattan = pairwise_matrix(
        &row_ids,
        &values,
        2,
        2,
        MatrixPairwiseOptions::new().with_metric(MatrixPairwiseMetric::ManhattanDistance),
    )?;
    assert_close(
        manhattan
            .value_at(0, 1)
            .ok_or(anyhow::anyhow!("missing value"))?,
        7.0,
    );
    Ok(())
}

#[test]
fn accepts_spectrum_and_bucket_matrices() -> anyhow::Result<()> {
    let spectrum_matrix = SpectrumMatrix1D {
        axis: Axis::linear("x", Unit::Ppm, 0.0, 1.0, 2)?,
        row_ids: row_ids(),
        values: vec![1.0, 2.0, 2.0, 4.0, 2.0, 1.0],
    };
    let spectrum_result = pairwise_spectrum_matrix_1d(
        &spectrum_matrix,
        MatrixPairwiseOptions::new().with_metric(MatrixPairwiseMetric::PearsonCorrelation),
    )?;
    assert_eq!(spectrum_result.shape(), (3, 3));

    let bucket_matrix = BucketMatrix1D {
        regions: vec![
            IntegralRegion { from: 0.0, to: 1.0 },
            IntegralRegion { from: 1.0, to: 2.0 },
        ],
        row_ids: row_ids(),
        values: vec![1.0, 2.0, 2.0, 4.0, 2.0, 1.0],
    };
    let bucket_result = pairwise_bucket_matrix_1d(
        &bucket_matrix,
        MatrixPairwiseOptions::new().with_metric(MatrixPairwiseMetric::CosineSimilarity),
    )?;
    assert_eq!(bucket_result.shape(), (3, 3));
    Ok(())
}

#[test]
fn accepts_two_dimensional_matrix_flats() -> anyhow::Result<()> {
    let bucket_matrix = BucketMatrix2D {
        regions: vec![
            IntegralRegion2D {
                x_from: 0.0,
                x_to: 1.0,
                y_from: 0.0,
                y_to: 1.0,
            },
            IntegralRegion2D {
                x_from: 1.0,
                x_to: 2.0,
                y_from: 0.0,
                y_to: 1.0,
            },
        ],
        x_bucket_count: 2,
        y_bucket_count: 1,
        layer_ids: row_ids(),
        values: vec![1.0, 2.0, 2.0, 4.0, 2.0, 1.0],
    };
    let result = pairwise_bucket_matrix_2d(
        &bucket_matrix,
        MatrixPairwiseOptions::new().with_metric(MatrixPairwiseMetric::EuclideanDistance),
    )?;

    assert_eq!(result.shape(), (3, 3));
    assert_close(
        result
            .value_at(0, 1)
            .ok_or(anyhow::anyhow!("missing value"))?,
        5.0_f64.sqrt(),
    );
    Ok(())
}

#[test]
fn rejects_invalid_inputs() {
    let empty_error = pairwise_matrix(&[], &[], 0, 1, MatrixPairwiseOptions::new())
        .expect_err("empty rows should fail");
    assert!(matches!(empty_error, RSpinError::InvalidSpectrum { .. }));

    let row_id_error = pairwise_matrix(
        &[String::from("a")],
        &[1.0, 2.0, 3.0, 4.0],
        2,
        2,
        MatrixPairwiseOptions::new(),
    )
    .expect_err("row id mismatch should fail");
    assert!(matches!(row_id_error, RSpinError::InvalidSpectrum { .. }));

    let non_finite_error = pairwise_matrix(
        &[String::from("a"), String::from("b")],
        &[1.0, f64::NAN, 3.0, 4.0],
        2,
        2,
        MatrixPairwiseOptions::new(),
    )
    .expect_err("non-finite values should fail");
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
