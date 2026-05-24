use rspin_core::{Axis, Metadata, Spectrum1D, Spectrum2D, Unit};

use super::super::{
    bucket_spectra_1d_json, bucket_spectra_2d_json, bucket_spectrum_1d_json,
    bucket_spectrum_2d_json, from_json, to_json,
};

#[test]
fn buckets_spectrum_1d_json() -> anyhow::Result<()> {
    let spectrum_json = to_json(&Spectrum1D::new(
        Axis::linear("x", Unit::Ppm, 0.0, 2.0, 3)?,
        vec![1.0, 1.0, 1.0],
        Metadata::default(),
    )?)?;
    let buckets_json =
        bucket_spectrum_1d_json(&spectrum_json, r#"{"from":0.0,"to":2.0,"bucket_count":2}"#)?;
    let buckets: Vec<rspin_analysis::SpectralBucket1D> = from_json(&buckets_json)?;

    assert_eq!(buckets.len(), 2);
    assert!((buckets[0].area - 1.0).abs() < 1.0e-12);
    assert!((buckets[1].area - 1.0).abs() < 1.0e-12);
    Ok(())
}

#[test]
fn buckets_spectra_1d_matrix_json() -> anyhow::Result<()> {
    let spectra_json = to_json(&vec![
        Spectrum1D::new(
            Axis::linear("x", Unit::Ppm, 0.0, 2.0, 3)?,
            vec![1.0, 1.0, 1.0],
            Metadata::named("a"),
        )?,
        Spectrum1D::new(
            Axis::linear("x", Unit::Ppm, 0.0, 2.0, 3)?,
            vec![2.0, 2.0, 2.0],
            Metadata::named("b b"),
        )?,
    ])?;
    let matrix_json =
        bucket_spectra_1d_json(&spectra_json, r#"{"from":0.0,"to":2.0,"bucket_count":2}"#)?;
    let matrix: rspin_analysis::BucketMatrix1D = from_json(&matrix_json)?;

    assert_eq!(matrix.shape(), (2, 2));
    assert_eq!(matrix.row_ids, vec!["0:a", "1:b_b"]);
    assert_eq!(matrix.values, vec![1.0, 1.0, 2.0, 2.0]);
    Ok(())
}

#[test]
fn buckets_spectrum_2d_json() -> anyhow::Result<()> {
    let spectrum_json = to_json(&Spectrum2D::new(
        Axis::linear("x", Unit::Ppm, 0.0, 2.0, 3)?,
        Axis::linear("y", Unit::Ppm, 0.0, 2.0, 3)?,
        vec![1.0; 9],
        Metadata::default(),
    )?)?;
    let buckets_json = bucket_spectrum_2d_json(
        &spectrum_json,
        r#"{
            "x_from":0.0,
            "x_to":2.0,
            "y_from":0.0,
            "y_to":2.0,
            "x_bucket_count":2,
            "y_bucket_count":2
        }"#,
    )?;
    let buckets: Vec<rspin_analysis::SpectralBucket2D> = from_json(&buckets_json)?;

    assert_eq!(buckets.len(), 4);
    assert!((buckets[0].volume - 1.0).abs() < 1.0e-12);
    assert_eq!(buckets[3].x_index, 1);
    assert_eq!(buckets[3].y_index, 1);
    Ok(())
}

#[test]
fn buckets_spectra_2d_matrix_json() -> anyhow::Result<()> {
    let spectra_json = to_json(&vec![
        Spectrum2D::new(
            Axis::linear("x", Unit::Ppm, 0.0, 2.0, 3)?,
            Axis::linear("y", Unit::Ppm, 0.0, 2.0, 3)?,
            vec![1.0; 9],
            Metadata::named("a"),
        )?,
        Spectrum2D::new(
            Axis::linear("x", Unit::Ppm, 0.0, 2.0, 3)?,
            Axis::linear("y", Unit::Ppm, 0.0, 2.0, 3)?,
            vec![2.0; 9],
            Metadata::named("b b"),
        )?,
    ])?;
    let matrix_json = bucket_spectra_2d_json(
        &spectra_json,
        r#"{
            "x_from":0.0,
            "x_to":2.0,
            "y_from":0.0,
            "y_to":2.0,
            "x_bucket_count":2,
            "y_bucket_count":2
        }"#,
    )?;
    let matrix: rspin_analysis::BucketMatrix2D = from_json(&matrix_json)?;

    assert_eq!(matrix.shape(), (2, 2, 2));
    assert_eq!(matrix.layer_ids, vec!["0:a", "1:b_b"]);
    assert_eq!(matrix.values, vec![1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0, 2.0]);
    Ok(())
}
