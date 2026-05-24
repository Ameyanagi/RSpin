use rspin_core::{Axis, Unit};

use super::*;
use crate::{BucketMatrix1D, BucketMatrix2D, IntegralRegion, IntegralRegion2D};

#[test]
fn pca_extracts_dominant_component_from_centered_matrix() -> anyhow::Result<()> {
    let result = pca_matrix(
        &row_ids(),
        &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
        3,
        2,
        MatrixPcaOptions::new(),
    )?;

    assert_eq!(result.score_shape(), (3, 2));
    assert_eq!(result.loading_shape(), (2, 2));
    assert_vec_close(&result.means, &[3.0, 4.0]);
    assert_vec_close(&result.scales, &[1.0, 1.0]);
    assert_close(result.explained_variance[0], 8.0);
    assert_close(result.explained_variance_ratio[0], 1.0);
    assert_close(
        result
            .loading_at(0, 0)
            .ok_or(anyhow::anyhow!("missing loading"))?,
        2.0_f64.sqrt() / 2.0,
    );
    assert_close(
        result
            .loading_at(0, 1)
            .ok_or(anyhow::anyhow!("missing loading"))?,
        2.0_f64.sqrt() / 2.0,
    );
    assert_close(
        result
            .score_at(0, 0)
            .ok_or(anyhow::anyhow!("missing score"))?,
        -8.0_f64.sqrt(),
    );
    assert_close(
        result
            .score_at(1, 0)
            .ok_or(anyhow::anyhow!("missing score"))?,
        0.0,
    );
    assert_close(
        result
            .score_at(2, 0)
            .ok_or(anyhow::anyhow!("missing score"))?,
        8.0_f64.sqrt(),
    );
    assert_eq!(result.score_at(3, 0), None);
    assert_eq!(result.loading_at(0, 2), None);
    Ok(())
}

#[test]
fn pca_options_support_scaling_and_component_limits() -> anyhow::Result<()> {
    let result = pca_matrix(
        &row_ids(),
        &[1.0, 10.0, 2.0, 20.0, 3.0, 30.0],
        3,
        2,
        MatrixPcaOptions::new()
            .with_component_count(5)
            .with_scaling(MatrixScaling::UnitVariance),
    )?;

    assert_eq!(result.component_count, 2);
    assert_vec_close(&result.means, &[2.0, 20.0]);
    assert_vec_close(&result.scales, &[1.0, 10.0]);
    assert_close(result.explained_variance[0], 2.0);
    assert_close(result.explained_variance_ratio[0], 1.0);

    let uncentered = pca_matrix(
        &row_ids(),
        &[1.0, 10.0, 2.0, 20.0, 3.0, 30.0],
        3,
        2,
        MatrixPcaOptions::new()
            .without_centering()
            .with_scaling(MatrixScaling::Pareto)
            .with_component_count(1),
    )?;
    assert_vec_close(&uncentered.means, &[0.0, 0.0]);
    assert_vec_close(&uncentered.scales, &[1.0, 10.0_f64.sqrt()]);
    assert_eq!(uncentered.component_count, 1);
    Ok(())
}

#[test]
fn pca_accepts_spectrum_and_bucket_matrices() -> anyhow::Result<()> {
    let spectrum_matrix = SpectrumMatrix1D {
        axis: Axis::linear("x", Unit::Ppm, 0.0, 1.0, 2)?,
        row_ids: row_ids(),
        values: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
    };
    let spectrum_result = pca_spectrum_matrix_1d(
        &spectrum_matrix,
        MatrixPcaOptions::new().with_component_count(1),
    )?;
    assert_eq!(spectrum_result.score_shape(), (3, 1));

    let bucket_matrix = BucketMatrix1D {
        regions: vec![
            IntegralRegion { from: 0.0, to: 1.0 },
            IntegralRegion { from: 1.0, to: 2.0 },
        ],
        row_ids: row_ids(),
        values: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
    };
    let bucket_result = pca_bucket_matrix_1d(
        &bucket_matrix,
        MatrixPcaOptions::new().with_component_count(1),
    )?;
    assert_eq!(bucket_result.loading_shape(), (1, 2));
    Ok(())
}

#[test]
fn pca_accepts_two_dimensional_matrix_flats() -> anyhow::Result<()> {
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
        values: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
    };
    let result = pca_bucket_matrix_2d(
        &bucket_matrix,
        MatrixPcaOptions::new().with_component_count(1),
    )?;

    assert_eq!(result.score_shape(), (3, 1));
    assert_eq!(result.loading_shape(), (1, 2));
    Ok(())
}

#[test]
fn pca_rejects_invalid_inputs() {
    let zero_component_error = pca_matrix(
        &row_ids(),
        &[1.0, 2.0, 3.0, 4.0],
        2,
        2,
        MatrixPcaOptions::new().with_component_count(0),
    )
    .expect_err("zero components should fail");
    assert!(matches!(
        zero_component_error,
        RSpinError::InvalidSpectrum { .. }
    ));

    let one_row_error = pca_matrix(
        &[String::from("a")],
        &[1.0, 2.0],
        1,
        2,
        MatrixPcaOptions::new(),
    )
    .expect_err("single row should fail");
    assert!(matches!(one_row_error, RSpinError::InvalidSpectrum { .. }));

    let non_finite_error = pca_matrix(
        &[String::from("a"), String::from("b")],
        &[1.0, f64::NAN, 3.0, 4.0],
        2,
        2,
        MatrixPcaOptions::new(),
    )
    .expect_err("non-finite values should fail");
    assert!(matches!(non_finite_error, RSpinError::NonFinite { .. }));
}

fn row_ids() -> Vec<String> {
    vec!["a".to_owned(), "b".to_owned(), "c".to_owned()]
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1.0e-10,
        "{actual} != {expected}"
    );
}

fn assert_vec_close(actual: &[f64], expected: &[f64]) {
    assert_eq!(actual.len(), expected.len());
    for (actual_value, expected_value) in actual.iter().zip(expected) {
        assert_close(*actual_value, *expected_value);
    }
}
