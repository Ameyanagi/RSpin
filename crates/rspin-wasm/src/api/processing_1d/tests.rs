use rspin_core::{Axis, Metadata, Spectrum1D, Unit};

use super::super::{from_json, to_json};
use super::*;

#[test]
fn offsets_shifts_and_zero_fills_1d_spectrum_json() -> anyhow::Result<()> {
    let spectrum_json = to_json(&real_spectrum()?)?;
    let offset_json = offset_spectrum_1d_json(&spectrum_json, 1.0)?;
    let shifted_json = shift_spectrum_1d_axis_json(&offset_json, -0.5)?;
    let filled_json = zero_fill_spectrum_1d_json(&shifted_json, 5)?;
    let processed: Spectrum1D = from_json(&filled_json)?;

    assert_vec_close(&processed.x.values, &[-0.5, 0.5, 1.5, 2.5, 3.5]);
    assert_vec_close(&processed.intensities, &[2.0, -1.0, 5.0, 0.0, 0.0]);
    assert_eq!(processed.processing.len(), 3);
    assert_eq!(processed.processing[0].operation, "offset_intensity");
    assert_eq!(processed.processing[1].operation, "shift_axis");
    assert_eq!(processed.processing[2].operation, "zero_fill");
    Ok(())
}

#[test]
fn crops_1d_spectrum_json() -> anyhow::Result<()> {
    let spectrum_json = to_json(&complex_spectrum()?)?;
    let cropped_json = crop_spectrum_1d_json(&spectrum_json, 1.0, 2.0)?;
    let cropped: Spectrum1D = from_json(&cropped_json)?;

    assert_vec_close(&cropped.x.values, &[1.0, 2.0]);
    assert_vec_close(&cropped.intensities, &[-2.0, 3.0]);
    assert_option_vec_close(cropped.imaginary.as_deref(), &[-1.0, 1.5]);
    assert_eq!(
        cropped
            .processing
            .last()
            .map(|record| record.operation.as_str()),
        Some("crop_1d")
    );
    Ok(())
}

#[test]
fn takes_absolute_value_1d_spectrum_json() -> anyhow::Result<()> {
    let spectrum_json = to_json(&complex_spectrum()?)?;
    let abs_json = abs_spectrum_1d_json(&spectrum_json)?;
    let processed: Spectrum1D = from_json(&abs_json)?;

    assert_vec_close(&processed.intensities, &[1.0, 2.0, 3.0, 4.0]);
    assert_option_vec_close(processed.imaginary.as_deref(), &[0.5, 1.0, 1.5, 2.0]);
    assert_eq!(
        processed
            .processing
            .last()
            .map(|record| record.operation.as_str()),
        Some("abs_1d")
    );
    Ok(())
}

#[test]
fn resamples_1d_spectrum_json() -> anyhow::Result<()> {
    let spectrum_json = to_json(&complex_spectrum()?)?;
    let target_axis_json = to_json(&Axis::linear("x", Unit::Ppm, 0.0, 3.0, 7)?)?;
    let resampled_json = resample_spectrum_1d_json(&spectrum_json, &target_axis_json, -1.0)?;
    let resampled: Spectrum1D = from_json(&resampled_json)?;

    assert_vec_close(&resampled.x.values, &[0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0]);
    assert_vec_close(
        &resampled.intensities,
        &[1.0, -0.5, -2.0, 0.5, 3.0, 3.5, 4.0],
    );
    assert_option_vec_close(
        resampled.imaginary.as_deref(),
        &[0.5, -0.25, -1.0, 0.25, 1.5, 1.75, 2.0],
    );
    assert_eq!(
        resampled
            .processing
            .last()
            .map(|record| record.operation.as_str()),
        Some("resample_1d")
    );
    Ok(())
}

#[test]
fn roundtrips_1d_fft_json() -> anyhow::Result<()> {
    let spectrum = complex_spectrum()?;
    let spectrum_json = to_json(&spectrum)?;
    let forward_json = fft_spectrum_1d_json(&spectrum_json, r#""forward""#)?;
    let inverse_json = fft_spectrum_1d_json(&forward_json, r#""inverse""#)?;
    let inverse: Spectrum1D = from_json(&inverse_json)?;

    assert_vec_close(&inverse.intensities, &spectrum.intensities);
    match (&inverse.imaginary, &spectrum.imaginary) {
        (Some(actual), Some(expected)) => assert_vec_close(actual, expected),
        _ => panic!("roundtrip should preserve the imaginary channel"),
    }
    Ok(())
}

#[test]
fn phases_and_magnitudes_1d_spectrum_json() -> anyhow::Result<()> {
    let spectrum_json = to_json(&Spectrum1D::new_complex(
        Axis::linear("x", Unit::Ppm, 0.0, 1.0, 2)?,
        vec![1.0, 0.0],
        Some(vec![0.0, 2.0]),
        Metadata::default(),
    )?)?;
    let phased_json = phase_spectrum_1d_json(&spectrum_json, r#"{"zero_order_deg":90.0}"#)?;
    let phased: Spectrum1D = from_json(&phased_json)?;
    assert_vec_close(&phased.intensities, &[0.0, -2.0]);
    assert_option_vec_close(phased.imaginary.as_deref(), &[1.0, 0.0]);

    let magnitude_json = magnitude_spectrum_1d_json(&phased_json)?;
    let magnitude: Spectrum1D = from_json(&magnitude_json)?;
    assert_vec_close(&magnitude.intensities, &[1.0, 2.0]);
    assert!(magnitude.imaginary.is_none());
    Ok(())
}

#[test]
fn apodizes_and_subtracts_baseline_json() -> anyhow::Result<()> {
    let spectrum_json = to_json(&Spectrum1D::new(
        Axis::linear("x", Unit::Ppm, 0.0, 2.0, 3)?,
        vec![2.0, 4.0, 6.0],
        Metadata::default(),
    )?)?;
    let apodized_json = exponential_apodization_spectrum_1d_json(
        &spectrum_json,
        r#"{"line_broadening_hz":1.0,"dwell_time_s":0.1}"#,
    )?;
    let apodized: Spectrum1D = from_json(&apodized_json)?;
    assert!(apodized.intensities[1] < 4.0);
    assert!(apodized.intensities[2] < 6.0);
    assert_eq!(
        apodized
            .processing
            .last()
            .map(|record| record.operation.as_str()),
        Some("exponential_apodization")
    );
    let gaussian_json = gaussian_apodization_spectrum_1d_json(
        &spectrum_json,
        r#"{"gaussian_broadening_hz":1.0,"dwell_time_s":0.1}"#,
    )?;
    let gaussian: Spectrum1D = from_json(&gaussian_json)?;
    assert!(gaussian.intensities[1] < 4.0);
    assert!(gaussian.intensities[2] < 6.0);
    assert_eq!(
        gaussian
            .processing
            .last()
            .map(|record| record.operation.as_str()),
        Some("gaussian_apodization")
    );
    let sine_bell_json = sine_bell_apodization_spectrum_1d_json(
        &spectrum_json,
        r#"{"start_angle_deg":0.0,"end_angle_deg":180.0,"exponent":1.0}"#,
    )?;
    let sine_bell: Spectrum1D = from_json(&sine_bell_json)?;
    assert_vec_close(&sine_bell.intensities, &[0.0, 4.0, 0.0]);
    assert_eq!(
        sine_bell
            .processing
            .last()
            .map(|record| record.operation.as_str()),
        Some("sine_bell_apodization")
    );

    let corrected_json =
        subtract_baseline_spectrum_1d_json(&spectrum_json, r#"{"method":"constant","value":1.5}"#)?;
    let corrected: Spectrum1D = from_json(&corrected_json)?;
    assert_vec_close(&corrected.intensities, &[0.5, 2.5, 4.5]);
    assert_eq!(
        corrected
            .processing
            .last()
            .map(|record| record.operation.as_str()),
        Some("baseline_constant")
    );
    Ok(())
}

#[cfg(feature = "external-baselines")]
#[test]
fn subtracts_external_baselines_asls_json() -> anyhow::Result<()> {
    let spectrum_json = to_json(&Spectrum1D::new(
        Axis::linear("x", Unit::Ppm, 0.0, 4.0, 5)?,
        vec![1.0, 1.0, 4.0, 1.0, 1.0],
        Metadata::default(),
    )?)?;
    let corrected_json = subtract_baseline_spectrum_1d_json(
        &spectrum_json,
        r#"{"method":"baselines_asls","lambda":10000.0,"p":0.01,"max_iter":25,"tolerance":0.000001}"#,
    )?;
    let corrected: Spectrum1D = from_json(&corrected_json)?;

    assert!(corrected.intensities[2] > 2.0);
    assert_eq!(
        corrected
            .processing
            .last()
            .map(|record| record.operation.as_str()),
        Some("baseline_baselines_asls")
    );
    Ok(())
}

#[test]
fn applies_processing_recipe_1d_json() -> anyhow::Result<()> {
    let spectrum_json = to_json(&real_spectrum()?)?;
    let processed_json = apply_processing_recipe_1d_json(
        &spectrum_json,
        r#"{"format":"rspin.processing_recipe_1d","version":1,"recipe":{"operations":[{"operation":"scale","factor":2.0},{"operation":"offset","offset":-2.0},{"operation":"absolute_value"},{"operation":"normalize_max_abs"}]}}"#,
    )?;
    let processed: Spectrum1D = from_json(&processed_json)?;

    assert_vec_close(&processed.x.values, &[0.0, 1.0, 2.0]);
    assert_vec_close(&processed.intensities, &[0.0, 1.0, 1.0]);
    assert_eq!(
        processed
            .processing
            .iter()
            .map(|record| record.operation.as_str())
            .collect::<Vec<_>>(),
        vec![
            "scale_intensity",
            "offset_intensity",
            "abs_1d",
            "normalize_max_abs"
        ]
    );
    Ok(())
}

#[test]
fn applies_processing_recipe_1d_prefix_json() -> anyhow::Result<()> {
    let spectrum_json = to_json(&real_spectrum()?)?;
    let processed_json = apply_processing_recipe_1d_until_json(
        &spectrum_json,
        r#"{"operations":[{"operation":"scale","factor":2.0},{"operation":"offset","offset":-2.0},{"operation":"absolute_value"},{"operation":"normalize_max_abs"}]}"#,
        2,
    )?;
    let processed: Spectrum1D = from_json(&processed_json)?;

    assert_vec_close(&processed.intensities, &[0.0, -6.0, 6.0]);
    assert_eq!(processed.processing.len(), 2);

    let error = apply_processing_recipe_1d_until_json(
        &spectrum_json,
        r#"{"operations":[{"operation":"scale","factor":2.0}]}"#,
        2,
    )
    .expect_err("too many operations should fail");
    assert!(error.to_string().contains("requested"));
    Ok(())
}

#[test]
fn rejects_invalid_1d_processing_json_options() -> anyhow::Result<()> {
    let spectrum_json = to_json(&real_spectrum()?)?;

    let error = zero_fill_spectrum_1d_json(&spectrum_json, 2)
        .expect_err("short zero-fill target should fail");
    assert!(error.to_string().contains("zero-fill"));

    let error = fft_spectrum_1d_json(&spectrum_json, r#""backward""#)
        .expect_err("invalid FFT direction should fail");
    assert!(error.to_string().contains("unknown variant"));

    let error = phase_spectrum_1d_json(&spectrum_json, r#"{"pivot_fraction":1.5}"#)
        .expect_err("invalid phase pivot should fail");
    assert!(error.to_string().contains("pivot"));
    Ok(())
}

fn real_spectrum() -> anyhow::Result<Spectrum1D> {
    Spectrum1D::new(
        Axis::linear("x", Unit::Ppm, 0.0, 2.0, 3)?,
        vec![1.0, -2.0, 4.0],
        Metadata::default(),
    )
    .map_err(Into::into)
}

fn complex_spectrum() -> anyhow::Result<Spectrum1D> {
    Spectrum1D::new_complex(
        Axis::linear("x", Unit::Ppm, 0.0, 3.0, 4)?,
        vec![1.0, -2.0, 3.0, 4.0],
        Some(vec![0.5, -1.0, 1.5, 2.0]),
        Metadata::default(),
    )
    .map_err(Into::into)
}

fn assert_option_vec_close(actual: Option<&[f64]>, expected: &[f64]) {
    match actual {
        Some(values) => assert_vec_close(values, expected),
        None => panic!("expected an imaginary channel"),
    }
}

fn assert_vec_close(actual: &[f64], expected: &[f64]) {
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(expected) {
        assert!((actual - expected).abs() < 1.0e-10);
    }
}
