use rspin_core::{Axis, Metadata, Spectrum2D, Unit};

use super::*;

#[test]
fn fft_2d_forward_transforms_real_impulse() -> anyhow::Result<()> {
    let spectrum = Spectrum2D::new(
        Axis::linear("x", Unit::Points, 0.0, 1.0, 2)?,
        Axis::linear("y", Unit::Points, 0.0, 1.0, 2)?,
        vec![1.0, 0.0, 0.0, 0.0],
        Metadata::default(),
    )?;

    let transformed = fft_2d(&spectrum, FftDirection::Forward)?;

    assert_vec_close(&transformed.z, &[1.0, 1.0, 1.0, 1.0]);
    assert_vec_close(require_imaginary(&transformed)?, &[0.0, 0.0, 0.0, 0.0]);
    assert_eq!(transformed.processing[0].operation, "fft_2d");
    Ok(())
}

#[test]
fn fft_2d_inverse_roundtrip_recovers_complex_data() -> anyhow::Result<()> {
    let spectrum = complex_spectrum()?;
    let transformed = fft_2d(&spectrum, FftDirection::Forward)?;
    let recovered = fft_2d(&transformed, FftDirection::Inverse)?;

    assert_vec_close(&recovered.z, &spectrum.z);
    assert_vec_close(
        require_imaginary(&recovered)?,
        require_imaginary_2d(&spectrum)?,
    );
    assert_eq!(recovered.processing.len(), 2);
    Ok(())
}

#[test]
fn fft_2d_processing_step_delegates_to_function() -> anyhow::Result<()> {
    let spectrum = complex_spectrum()?;
    let transformed = Fft2D {
        direction: FftDirection::Forward,
    }
    .apply(&spectrum)?;

    assert_eq!(transformed.shape(), spectrum.shape());
    assert!(transformed.imaginary.is_some());
    assert_eq!(transformed.processing[0].operation, "fft_2d");
    Ok(())
}

fn complex_spectrum() -> anyhow::Result<Spectrum2D> {
    Ok(Spectrum2D::new_complex(
        Axis::linear("x", Unit::Points, 0.0, 2.0, 3)?,
        Axis::linear("y", Unit::Points, 0.0, 1.0, 2)?,
        vec![1.0, -2.0, 3.0, 4.0, 0.5, -1.5],
        Some(vec![0.0, 1.0, -1.0, 0.5, -0.25, 0.75]),
        Metadata::named("complex 2d"),
    )?)
}

fn require_imaginary(spectrum: &Spectrum2D) -> anyhow::Result<&[f64]> {
    match &spectrum.imaginary {
        Some(imaginary) => Ok(imaginary),
        None => anyhow::bail!("missing imaginary channel"),
    }
}

fn require_imaginary_2d(spectrum: &Spectrum2D) -> anyhow::Result<&[f64]> {
    require_imaginary(spectrum)
}

fn assert_vec_close(actual: &[f64], expected: &[f64]) {
    assert_eq!(actual.len(), expected.len());
    for (left, right) in actual.iter().zip(expected) {
        assert!((left - right).abs() < 1.0e-10, "{left} != {right}");
    }
}
