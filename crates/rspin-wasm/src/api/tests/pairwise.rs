use rspin_core::{Axis, Unit};

use super::super::{
    from_json, pairwise_bucket_matrix_1d_json, pairwise_bucket_matrix_2d_json,
    pairwise_spectrum_matrix_1d_json, pairwise_spectrum_matrix_2d_json, to_json,
};

#[test]
fn computes_pairwise_spectrum_matrix_1d_json() -> anyhow::Result<()> {
    let matrix = rspin_analysis::SpectrumMatrix1D {
        axis: Axis::linear("x", Unit::Ppm, 0.0, 1.0, 2)?,
        row_ids: row_ids(),
        values: vec![1.0, 2.0, 2.0, 4.0, 2.0, 1.0],
    };
    let result_json =
        pairwise_spectrum_matrix_1d_json(&to_json(&matrix)?, r#"{"metric":"PearsonCorrelation"}"#)?;
    let result: rspin_analysis::MatrixPairwiseResult = from_json(&result_json)?;

    assert_eq!(result.shape(), (3, 3));
    assert!((result.values[1] - 1.0).abs() < 1.0e-12);
    assert!((result.values[2] + 1.0).abs() < 1.0e-12);
    Ok(())
}

#[test]
fn computes_pairwise_spectrum_matrix_2d_json() -> anyhow::Result<()> {
    let matrix = rspin_analysis::SpectrumMatrix2D {
        x: Axis::linear("x", Unit::Ppm, 0.0, 1.0, 2)?,
        y: Axis::linear("y", Unit::Ppm, 0.0, 1.0, 1)?,
        spectrum_ids: row_ids(),
        values: vec![1.0, 2.0, 2.0, 4.0, 2.0, 1.0],
    };
    let result_json =
        pairwise_spectrum_matrix_2d_json(&to_json(&matrix)?, r#"{"metric":"EuclideanDistance"}"#)?;
    let result: rspin_analysis::MatrixPairwiseResult = from_json(&result_json)?;

    assert_eq!(result.shape(), (3, 3));
    assert!((result.values[1] - 5.0_f64.sqrt()).abs() < 1.0e-12);
    Ok(())
}

#[test]
fn computes_pairwise_bucket_matrix_1d_json() -> anyhow::Result<()> {
    let matrix = rspin_analysis::BucketMatrix1D {
        regions: vec![
            rspin_analysis::IntegralRegion { from: 0.0, to: 1.0 },
            rspin_analysis::IntegralRegion { from: 1.0, to: 2.0 },
        ],
        row_ids: row_ids(),
        values: vec![3.0, 4.0, 0.0, 0.0, 6.0, 8.0],
    };
    let result_json =
        pairwise_bucket_matrix_1d_json(&to_json(&matrix)?, r#"{"metric":"CosineSimilarity"}"#)?;
    let result: rspin_analysis::MatrixPairwiseResult = from_json(&result_json)?;

    assert_eq!(result.shape(), (3, 3));
    assert!((result.values[1] - 0.0).abs() < 1.0e-12);
    assert!((result.values[2] - 1.0).abs() < 1.0e-12);
    Ok(())
}

#[test]
fn computes_pairwise_bucket_matrix_2d_json() -> anyhow::Result<()> {
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
        layer_ids: row_ids(),
        values: vec![3.0, 4.0, 0.0, 0.0, 6.0, 8.0],
    };
    let result_json =
        pairwise_bucket_matrix_2d_json(&to_json(&matrix)?, r#"{"metric":"ManhattanDistance"}"#)?;
    let result: rspin_analysis::MatrixPairwiseResult = from_json(&result_json)?;

    assert_eq!(result.shape(), (3, 3));
    assert!((result.values[1] - 7.0).abs() < 1.0e-12);
    assert!((result.values[2] - 7.0).abs() < 1.0e-12);
    Ok(())
}

fn row_ids() -> Vec<String> {
    vec!["a".to_owned(), "b".to_owned(), "c".to_owned()]
}
