use rspin_core::{Axis, Metadata, RSpinError, Spectrum1D, Unit};

use super::*;

#[test]
fn buckets_spectrum_into_equal_width_integrals() -> anyhow::Result<()> {
    let spectrum = spectrum("demo", &[0.0, 1.0, 2.0, 3.0], &[1.0, 1.0, 1.0, 1.0])?;
    let buckets = bucket_spectrum_1d(&spectrum, BucketOptions1D::new(0.0, 3.0, 3))?;

    assert_eq!(buckets.len(), 3);
    assert_close(buckets[0].area, 1.0);
    assert_close(buckets[1].area, 1.0);
    assert_close(buckets[2].area, 1.0);
    assert_eq!(buckets[2].region, IntegralRegion { from: 2.0, to: 3.0 });
    Ok(())
}

#[test]
fn buckets_support_descending_requested_range() -> anyhow::Result<()> {
    let spectrum = spectrum("descending", &[0.0, 1.0, 2.0, 3.0], &[1.0, 1.0, 1.0, 1.0])?;
    let buckets = bucket_spectrum_1d(&spectrum, BucketOptions1D::new(3.0, 0.0, 3))?;

    assert_eq!(buckets[0].region, IntegralRegion { from: 3.0, to: 2.0 });
    assert_close(buckets[0].area, 1.0);
    assert_close(buckets[2].area, 1.0);
    Ok(())
}

#[test]
fn generates_bucket_matrix() -> anyhow::Result<()> {
    let first = spectrum("a", &[0.0, 1.0, 2.0], &[1.0, 1.0, 1.0])?;
    let second = spectrum("b b", &[0.0, 1.0, 2.0], &[2.0, 2.0, 2.0])?;
    let matrix = bucket_spectra_1d(&[first, second], BucketOptions1D::new(0.0, 2.0, 2))?;

    assert_eq!(matrix.shape(), (2, 2));
    assert_eq!(matrix.row_ids, vec!["0:a", "1:b_b"]);
    assert_vec_close(&matrix.values, &[1.0, 1.0, 2.0, 2.0]);
    assert_eq!(matrix.value_at(1, 0), Some(2.0));
    assert_eq!(matrix.value_at(2, 0), None);
    Ok(())
}

#[test]
fn bucket_options_builders_work() {
    let options = BucketOptions1D::new(0.0, 1.0, 1)
        .with_range(-1.0, 1.0)
        .with_bucket_count(2);

    assert_close(options.from, -1.0);
    assert_close(options.to, 1.0);
    assert_eq!(options.bucket_count, 2);
}

#[test]
fn rejects_invalid_bucket_options_and_empty_matrix() -> anyhow::Result<()> {
    let spectrum = spectrum("demo", &[0.0, 1.0], &[1.0, 1.0])?;
    let zero_count_error = bucket_spectrum_1d(&spectrum, BucketOptions1D::new(0.0, 1.0, 0))
        .expect_err("zero bucket count should fail");
    let zero_width_error = bucket_spectrum_1d(&spectrum, BucketOptions1D::new(1.0, 1.0, 1))
        .expect_err("zero-width range should fail");
    let empty_matrix_error = bucket_spectra_1d(&[], BucketOptions1D::new(0.0, 1.0, 1))
        .expect_err("empty matrix input should fail");

    assert!(matches!(
        zero_count_error,
        RSpinError::InvalidSpectrum { .. }
    ));
    assert!(matches!(
        zero_width_error,
        RSpinError::InvalidSpectrum { .. }
    ));
    assert!(matches!(
        empty_matrix_error,
        RSpinError::InvalidSpectrum { .. }
    ));
    Ok(())
}

fn spectrum(name: &str, x: &[f64], intensities: &[f64]) -> anyhow::Result<Spectrum1D> {
    Ok(Spectrum1D::new(
        Axis::new("x", Unit::Ppm, x.to_vec())?,
        intensities.to_vec(),
        Metadata::named(name),
    )?)
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1.0e-12,
        "{actual} != {expected}"
    );
}

fn assert_vec_close(actual: &[f64], expected: &[f64]) {
    assert_eq!(actual.len(), expected.len());
    for (left, right) in actual.iter().zip(expected) {
        assert_close(*left, *right);
    }
}
