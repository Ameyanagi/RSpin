use rspin_core::{Axis, Unit};

use super::super::{
    from_json, pca_bucket_matrix_1d_json, pca_bucket_matrix_2d_json, pca_spectrum_matrix_1d_json,
    pca_spectrum_matrix_2d_json, to_json,
};

#[test]
fn runs_pca_on_spectrum_matrix_1d_json() -> anyhow::Result<()> {
    let matrix = rspin_analysis::SpectrumMatrix1D {
        axis: Axis::linear("x", Unit::Ppm, 0.0, 1.0, 2)?,
        row_ids: vec!["a".to_owned(), "b".to_owned(), "c".to_owned()],
        values: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
    };
    let result_json = pca_spectrum_matrix_1d_json(
        &to_json(&matrix)?,
        r#"{"component_count":1,"center":true,"scaling":"None"}"#,
    )?;
    let result: rspin_analysis::MatrixPcaResult = from_json(&result_json)?;

    assert_eq!(result.score_shape(), (3, 1));
    assert_eq!(result.loading_shape(), (1, 2));
    assert!((result.explained_variance_ratio[0] - 1.0).abs() < 1.0e-12);
    Ok(())
}

#[test]
fn runs_pca_on_spectrum_matrix_2d_json() -> anyhow::Result<()> {
    let matrix = rspin_analysis::SpectrumMatrix2D {
        x: Axis::linear("x", Unit::Ppm, 0.0, 1.0, 2)?,
        y: Axis::linear("y", Unit::Ppm, 0.0, 1.0, 1)?,
        spectrum_ids: vec!["a".to_owned(), "b".to_owned(), "c".to_owned()],
        values: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
    };
    let result_json = pca_spectrum_matrix_2d_json(
        &to_json(&matrix)?,
        r#"{"component_count":1,"center":true,"scaling":"None"}"#,
    )?;
    let result: rspin_analysis::MatrixPcaResult = from_json(&result_json)?;

    assert_eq!(result.score_shape(), (3, 1));
    assert_eq!(result.loading_shape(), (1, 2));
    Ok(())
}

#[test]
fn runs_pca_on_bucket_matrix_1d_json() -> anyhow::Result<()> {
    let matrix = rspin_analysis::BucketMatrix1D {
        regions: vec![
            rspin_analysis::IntegralRegion { from: 0.0, to: 1.0 },
            rspin_analysis::IntegralRegion { from: 1.0, to: 2.0 },
        ],
        row_ids: vec!["a".to_owned(), "b".to_owned(), "c".to_owned()],
        values: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
    };
    let result_json = pca_bucket_matrix_1d_json(
        &to_json(&matrix)?,
        r#"{"component_count":1,"center":true,"scaling":"UnitVariance"}"#,
    )?;
    let result: rspin_analysis::MatrixPcaResult = from_json(&result_json)?;

    assert_eq!(result.score_shape(), (3, 1));
    assert_eq!(result.scales, vec![2.0, 2.0]);
    Ok(())
}

#[test]
fn runs_pca_on_bucket_matrix_2d_json() -> anyhow::Result<()> {
    let matrix = rspin_analysis::BucketMatrix2D {
        regions: vec![
            rspin_analysis::IntegralRegion2D {
                x_from: 0.0,
                x_to: 1.0,
                y_from: 0.0,
                y_to: 1.0,
            },
            rspin_analysis::IntegralRegion2D {
                x_from: 1.0,
                x_to: 2.0,
                y_from: 0.0,
                y_to: 1.0,
            },
        ],
        x_bucket_count: 2,
        y_bucket_count: 1,
        layer_ids: vec!["a".to_owned(), "b".to_owned(), "c".to_owned()],
        values: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
    };
    let result_json = pca_bucket_matrix_2d_json(
        &to_json(&matrix)?,
        r#"{"component_count":1,"center":true,"scaling":"None"}"#,
    )?;
    let result: rspin_analysis::MatrixPcaResult = from_json(&result_json)?;

    assert_eq!(result.score_shape(), (3, 1));
    assert_eq!(result.loading_shape(), (1, 2));
    Ok(())
}
