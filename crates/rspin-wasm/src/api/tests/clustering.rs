use rspin_core::{Axis, Unit};

use super::super::{
    cluster_bucket_matrix_1d_json, cluster_bucket_matrix_2d_json, cluster_spectrum_matrix_1d_json,
    cluster_spectrum_matrix_2d_json, cut_cluster_result_at_distance_json,
    cut_cluster_result_to_count_json, from_json, to_json,
};

#[test]
fn clusters_spectrum_matrix_1d_json() -> anyhow::Result<()> {
    let matrix = rspin_analysis::SpectrumMatrix1D {
        axis: Axis::linear("x", Unit::Ppm, 0.0, 1.0, 2)?,
        row_ids: row_ids(),
        values: vec![0.0, 0.0, 0.0, 1.0, 4.0, 4.0],
    };
    let result_json = cluster_spectrum_matrix_1d_json(
        &to_json(&matrix)?,
        r#"{"metric":"EuclideanDistance","linkage":"Single"}"#,
    )?;
    let result: rspin_analysis::MatrixClusterResult = from_json(&result_json)?;

    assert_eq!(result.leaf_count(), 3);
    assert_eq!(result.merges.len(), 2);
    assert_eq!(result.merges[0].left, 0);
    assert_eq!(result.merges[0].right, 1);
    assert!((result.merges[0].distance - 1.0).abs() < 1.0e-12);
    Ok(())
}

#[test]
fn clusters_spectrum_matrix_2d_json() -> anyhow::Result<()> {
    let matrix = rspin_analysis::SpectrumMatrix2D {
        x: Axis::linear("x", Unit::Ppm, 0.0, 1.0, 2)?,
        y: Axis::linear("y", Unit::Ppm, 0.0, 1.0, 1)?,
        spectrum_ids: row_ids(),
        values: vec![1.0, 0.0, 1.0, 0.5, -1.0, 0.0],
    };
    let result_json = cluster_spectrum_matrix_2d_json(
        &to_json(&matrix)?,
        r#"{"metric":"CosineDistance","linkage":"Complete"}"#,
    )?;
    let result: rspin_analysis::MatrixClusterResult = from_json(&result_json)?;

    assert_eq!(result.merges.len(), 2);
    assert_eq!(result.merges[0].left, 0);
    assert_eq!(result.merges[0].right, 1);
    Ok(())
}

#[test]
fn clusters_bucket_matrix_1d_json() -> anyhow::Result<()> {
    let matrix = rspin_analysis::BucketMatrix1D {
        regions: vec![
            rspin_analysis::IntegralRegion { from: 0.0, to: 1.0 },
            rspin_analysis::IntegralRegion { from: 1.0, to: 2.0 },
        ],
        row_ids: row_ids(),
        values: vec![0.0, 0.0, 0.0, 2.0, 5.0, 5.0],
    };
    let result_json = cluster_bucket_matrix_1d_json(
        &to_json(&matrix)?,
        r#"{"metric":"ManhattanDistance","linkage":"Average"}"#,
    )?;
    let result: rspin_analysis::MatrixClusterResult = from_json(&result_json)?;

    assert_eq!(result.merges.len(), 2);
    assert_eq!(result.merges[0].left, 0);
    assert_eq!(result.merges[0].right, 1);
    assert!((result.merges[0].distance - 2.0).abs() < 1.0e-12);
    Ok(())
}

#[test]
fn clusters_bucket_matrix_2d_json() -> anyhow::Result<()> {
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
        values: vec![1.0, 2.0, 1.0, 3.0, 6.0, 6.0],
    };
    let result_json = cluster_bucket_matrix_2d_json(
        &to_json(&matrix)?,
        r#"{"metric":"PearsonDistance","linkage":"Single"}"#,
    )?;
    let result: rspin_analysis::MatrixClusterResult = from_json(&result_json)?;

    assert_eq!(result.leaf_count(), 3);
    assert_eq!(result.root_node_id(), Some(4));
    Ok(())
}

#[test]
fn cuts_cluster_result_json() -> anyhow::Result<()> {
    let matrix = rspin_analysis::SpectrumMatrix1D {
        axis: Axis::linear("x", Unit::Ppm, 0.0, 1.0, 1)?,
        row_ids: vec![
            "a".to_owned(),
            "b".to_owned(),
            "c".to_owned(),
            "d".to_owned(),
        ],
        values: vec![0.0, 1.0, 10.0, 11.0],
    };
    let result_json = cluster_spectrum_matrix_1d_json(
        &to_json(&matrix)?,
        r#"{"metric":"EuclideanDistance","linkage":"Single"}"#,
    )?;

    let count_cut_json = cut_cluster_result_to_count_json(&result_json, 2)?;
    let count_cut: rspin_analysis::MatrixClusterCut = from_json(&count_cut_json)?;
    assert_eq!(count_cut.cluster_count, 2);
    assert_eq!(count_cut.cluster_ids, vec![0, 0, 1, 1]);

    let distance_cut_json = cut_cluster_result_at_distance_json(&result_json, 0.5)?;
    let distance_cut: rspin_analysis::MatrixClusterCut = from_json(&distance_cut_json)?;
    assert_eq!(distance_cut.cluster_count, 4);
    assert_eq!(distance_cut.cluster_ids, vec![0, 1, 2, 3]);
    Ok(())
}

fn row_ids() -> Vec<String> {
    vec!["a".to_owned(), "b".to_owned(), "c".to_owned()]
}
