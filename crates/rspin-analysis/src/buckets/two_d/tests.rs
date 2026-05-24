use rspin_core::{Axis, Metadata, Spectrum2D, Unit};

use super::*;

#[test]
fn buckets_spectrum_into_rectangular_tiles() -> anyhow::Result<()> {
    let spectrum = constant_spectrum("demo", 1.0, 0.0, 2.0, 0.0, 2.0)?;
    let buckets = bucket_spectrum_2d(&spectrum, BucketOptions2D::new(0.0, 2.0, 0.0, 2.0, 2, 2))?;

    assert_eq!(buckets.len(), 4);
    assert_eq!(buckets[0].x_index, 0);
    assert_eq!(buckets[0].y_index, 0);
    assert_eq!(
        buckets[3].region,
        IntegralRegion2D {
            x_from: 1.0,
            x_to: 2.0,
            y_from: 1.0,
            y_to: 2.0,
        }
    );
    for bucket in &buckets {
        assert_close(bucket.volume, 1.0);
        assert_eq!(bucket.cells, 1);
    }
    Ok(())
}

#[test]
fn buckets_support_descending_requested_ranges() -> anyhow::Result<()> {
    let spectrum = constant_spectrum("descending", 1.0, 0.0, 2.0, 0.0, 2.0)?;
    let buckets = bucket_spectrum_2d(&spectrum, BucketOptions2D::new(2.0, 0.0, 2.0, 0.0, 2, 2))?;

    assert_eq!(
        buckets[0].region,
        IntegralRegion2D {
            x_from: 2.0,
            x_to: 1.0,
            y_from: 2.0,
            y_to: 1.0,
        }
    );
    assert_close(buckets[0].volume, 1.0);
    assert_close(buckets[3].volume, 1.0);
    Ok(())
}

#[test]
fn generates_2d_bucket_matrix() -> anyhow::Result<()> {
    let first = constant_spectrum("a", 1.0, 0.0, 2.0, 0.0, 2.0)?;
    let second = constant_spectrum("b b", 2.0, 0.0, 2.0, 0.0, 2.0)?;
    let matrix = bucket_spectra_2d(
        &[first, second],
        BucketOptions2D::new(0.0, 2.0, 0.0, 2.0, 2, 2),
    )?;

    assert_eq!(matrix.shape(), (2, 2, 2));
    assert_eq!(matrix.layer_ids, vec!["0:a", "1:b_b"]);
    assert_vec_close(&matrix.values, &[1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0, 2.0]);
    assert_eq!(matrix.value_at(1, 0, 1), Some(2.0));
    assert_eq!(matrix.value_at(2, 0, 0), None);
    Ok(())
}

#[test]
fn bucket_2d_options_builders_work() {
    let options = BucketOptions2D::new(0.0, 1.0, 0.0, 1.0, 1, 1)
        .with_x_range(-1.0, 1.0)
        .with_y_range(5.0, 3.0)
        .with_bucket_counts(3, 2)
        .with_x_bucket_count(4)
        .with_y_bucket_count(5);

    assert_close(options.x_from, -1.0);
    assert_close(options.x_to, 1.0);
    assert_close(options.y_from, 5.0);
    assert_close(options.y_to, 3.0);
    assert_eq!(options.x_bucket_count, 4);
    assert_eq!(options.y_bucket_count, 5);
}

#[test]
fn rejects_invalid_2d_bucket_options_and_empty_matrix() -> anyhow::Result<()> {
    let spectrum = constant_spectrum("demo", 1.0, 0.0, 2.0, 0.0, 2.0)?;
    let zero_x_count_error =
        bucket_spectrum_2d(&spectrum, BucketOptions2D::new(0.0, 1.0, 0.0, 1.0, 0, 1))
            .expect_err("zero x bucket count should fail");
    let zero_y_width_error =
        bucket_spectrum_2d(&spectrum, BucketOptions2D::new(0.0, 1.0, 1.0, 1.0, 1, 1))
            .expect_err("zero-width y range should fail");
    let empty_matrix_error = bucket_spectra_2d(&[], BucketOptions2D::new(0.0, 1.0, 0.0, 1.0, 1, 1))
        .expect_err("empty matrix input should fail");

    assert!(matches!(
        zero_x_count_error,
        RSpinError::InvalidSpectrum { .. }
    ));
    assert!(matches!(
        zero_y_width_error,
        RSpinError::InvalidSpectrum { .. }
    ));
    assert!(matches!(
        empty_matrix_error,
        RSpinError::InvalidSpectrum { .. }
    ));
    Ok(())
}

fn constant_spectrum(
    name: &str,
    value: f64,
    x_start: f64,
    x_end: f64,
    y_start: f64,
    y_end: f64,
) -> anyhow::Result<Spectrum2D> {
    let x = Axis::linear("x", Unit::Ppm, x_start, x_end, 3)?;
    let y = Axis::linear("y", Unit::Ppm, y_start, y_end, 3)?;
    Ok(Spectrum2D::new(
        x,
        y,
        vec![value; 9],
        Metadata::new().with_name(name),
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
    for (actual_value, expected_value) in actual.iter().zip(expected) {
        assert_close(*actual_value, *expected_value);
    }
}
