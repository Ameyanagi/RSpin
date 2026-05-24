use rspin_core::{Axis, Metadata, Spectrum2D, Unit};

use super::super::{from_json, spectrum1d_from_json, spectrum2d_from_json, to_json};
use super::*;

#[test]
fn scales_and_normalizes_2d_spectrum_json() -> anyhow::Result<()> {
    let spectrum_json = to_json(&complex_spectrum()?)?;

    let scaled_json = scale_spectrum_2d_json(&spectrum_json, -2.0)?;
    let scaled = spectrum2d_from_json(&scaled_json)?;
    assert_vec_close(&scaled.z, &[-2.0, 4.0, -6.0, -8.0]);
    assert_option_vec_close(scaled.imaginary.as_deref(), &[-1.0, 2.0, -3.0, -4.0]);

    let normalized_json = normalize_spectrum_2d_json(&spectrum_json)?;
    let normalized = spectrum2d_from_json(&normalized_json)?;
    assert_vec_close(&normalized.z, &[0.25, -0.5, 0.75, 1.0]);
    assert_option_vec_close(normalized.imaginary.as_deref(), &[0.125, -0.25, 0.375, 0.5]);
    assert_eq!(
        normalized
            .processing
            .last()
            .map(|record| record.operation.as_str()),
        Some("normalize_2d_max_abs")
    );
    Ok(())
}

#[test]
fn takes_absolute_value_2d_spectrum_json() -> anyhow::Result<()> {
    let spectrum_json = to_json(&complex_spectrum()?)?;
    let abs_json = abs_spectrum_2d_json(&spectrum_json)?;
    let processed = spectrum2d_from_json(&abs_json)?;

    assert_vec_close(&processed.z, &[1.0, 2.0, 3.0, 4.0]);
    assert_option_vec_close(processed.imaginary.as_deref(), &[0.5, 1.0, 1.5, 2.0]);
    assert_eq!(
        processed
            .processing
            .last()
            .map(|record| record.operation.as_str()),
        Some("abs_2d")
    );
    Ok(())
}

#[test]
fn zero_fills_2d_spectrum_json() -> anyhow::Result<()> {
    let spectrum_json = to_json(&complex_spectrum()?)?;
    let filled_json = zero_fill_spectrum_2d_json(&spectrum_json, 3, 3)?;
    let filled = spectrum2d_from_json(&filled_json)?;

    assert_eq!(filled.shape(), (3, 3));
    assert_vec_close(&filled.z, &[1.0, -2.0, 0.0, 3.0, 4.0, 0.0, 0.0, 0.0, 0.0]);
    assert_option_vec_close(
        filled.imaginary.as_deref(),
        &[0.5, -1.0, 0.0, 1.5, 2.0, 0.0, 0.0, 0.0, 0.0],
    );
    Ok(())
}

#[test]
fn crops_2d_spectrum_json() -> anyhow::Result<()> {
    let spectrum_json = to_json(&grid_spectrum()?)?;
    let cropped_json = crop_spectrum_2d_json(&spectrum_json, 1.0, 2.0, 1.0, 1.0)?;
    let cropped = spectrum2d_from_json(&cropped_json)?;

    assert_eq!(cropped.shape(), (2, 1));
    assert_vec_close(&cropped.x.values, &[1.0, 2.0]);
    assert_vec_close(&cropped.y.values, &[1.0]);
    assert_vec_close(&cropped.z, &[5.0, 6.0]);
    assert_option_vec_close(cropped.imaginary.as_deref(), &[15.0, 16.0]);
    assert_eq!(
        cropped
            .processing
            .last()
            .map(|record| record.operation.as_str()),
        Some("crop_2d")
    );
    Ok(())
}

#[test]
fn resamples_2d_spectrum_json() -> anyhow::Result<()> {
    let spectrum_json = to_json(&grid_spectrum()?)?;
    let columns_json = to_json(&Axis::linear("x", Unit::Ppm, 0.0, 2.0, 5)?)?;
    let rows_json = to_json(&Axis::linear("y", Unit::Ppm, -1.0, 1.0, 3)?)?;
    let resampled_json =
        resample_spectrum_2d_json(&spectrum_json, &columns_json, &rows_json, -1.0)?;
    let resampled = spectrum2d_from_json(&resampled_json)?;

    assert_eq!(resampled.shape(), (5, 3));
    assert_vec_close(
        &resampled.z,
        &[
            -1.0, -1.0, -1.0, -1.0, -1.0, 1.0, 1.5, 2.0, 2.5, 3.0, 4.0, 4.5, 5.0, 5.5, 6.0,
        ],
    );
    assert_option_vec_close(
        resampled.imaginary.as_deref(),
        &[
            -1.0, -1.0, -1.0, -1.0, -1.0, 10.0, 10.5, 11.0, 11.5, 12.0, 13.0, 14.0, 15.0, 15.5,
            16.0,
        ],
    );
    assert_eq!(
        resampled
            .processing
            .last()
            .map(|record| record.operation.as_str()),
        Some("resample_2d")
    );
    Ok(())
}

#[test]
fn roundtrips_2d_fft_json() -> anyhow::Result<()> {
    let spectrum = complex_spectrum()?;
    let spectrum_json = to_json(&spectrum)?;
    let forward_json = fft_spectrum_2d_json(&spectrum_json, r#""forward""#)?;
    let inverse_json = fft_spectrum_2d_json(&forward_json, r#""inverse""#)?;
    let inverse = spectrum2d_from_json(&inverse_json)?;

    assert_vec_close(&inverse.z, &spectrum.z);
    match (&inverse.imaginary, &spectrum.imaginary) {
        (Some(actual), Some(expected)) => assert_vec_close(actual, expected),
        _ => panic!("roundtrip should preserve the imaginary channel"),
    }
    Ok(())
}

#[test]
fn apodizes_2d_spectrum_json() -> anyhow::Result<()> {
    let spectrum_json = to_json(&complex_spectrum()?)?;
    let exponential_json = exponential_apodization_spectrum_2d_json(
        &spectrum_json,
        r#"{"x_line_broadening_hz":1.0,"y_line_broadening_hz":1.0,"x_dwell_time_s":0.1,"y_dwell_time_s":0.1}"#,
    )?;
    let exponential = spectrum2d_from_json(&exponential_json)?;
    assert!(exponential.z[1].abs() < 2.0);
    assert!(exponential.z[2].abs() < 3.0);
    assert_eq!(
        exponential
            .processing
            .last()
            .map(|record| record.operation.as_str()),
        Some("exponential_apodization_2d")
    );

    let apodized_json = gaussian_apodization_spectrum_2d_json(
        &spectrum_json,
        r#"{"x_gaussian_broadening_hz":1.0,"y_gaussian_broadening_hz":1.0,"x_dwell_time_s":0.1,"y_dwell_time_s":0.1}"#,
    )?;
    let apodized = spectrum2d_from_json(&apodized_json)?;

    assert!(apodized.z[1].abs() < 2.0);
    assert!(apodized.z[2].abs() < 3.0);
    let Some(imaginary) = apodized.imaginary.as_deref() else {
        panic!("expected an imaginary channel");
    };
    assert!(imaginary[1].abs() < 1.0);
    assert!(imaginary[2].abs() < 1.5);
    assert_eq!(
        apodized
            .processing
            .last()
            .map(|record| record.operation.as_str()),
        Some("gaussian_apodization_2d")
    );

    let sine_bell_json = sine_bell_apodization_spectrum_2d_json(
        &spectrum_json,
        r#"{"x_start_angle_deg":90.0,"x_end_angle_deg":90.0,"x_exponent":1.0,"y_start_angle_deg":90.0,"y_end_angle_deg":90.0,"y_exponent":1.0}"#,
    )?;
    let sine_bell = spectrum2d_from_json(&sine_bell_json)?;
    assert_vec_close(&sine_bell.z, &complex_spectrum()?.z);
    assert_eq!(
        sine_bell
            .processing
            .last()
            .map(|record| record.operation.as_str()),
        Some("sine_bell_apodization_2d")
    );
    Ok(())
}

#[test]
fn phases_and_auto_phases_2d_spectrum_json() -> anyhow::Result<()> {
    let spectrum_json = to_json(&positive_spectrum()?)?;
    let phased_json = phase_spectrum_2d_json(&spectrum_json, r#"{"x_zero_order_deg":45.0}"#)?;
    let result_json = auto_phase_spectrum_2d_json(
        &phased_json,
        r#"{"x_zero_order_min_deg":-90.0,"x_zero_order_max_deg":90.0,"x_zero_order_step_deg":5.0,"x_first_order_min_deg":0.0,"x_first_order_max_deg":0.0,"x_first_order_step_deg":5.0,"y_zero_order_min_deg":0.0,"y_zero_order_max_deg":0.0,"y_zero_order_step_deg":5.0,"y_first_order_min_deg":0.0,"y_first_order_max_deg":0.0,"y_first_order_step_deg":5.0}"#,
    )?;
    let result: AutoPhase2DResponseJson = from_json(&result_json)?;

    assert!((result.correction.x_zero_order_deg + 45.0).abs() < 1.0e-12);
    assert!(result.score <= 1.0e-12);
    assert!(result.spectrum.z.iter().any(|value| *value > 1.99));
    assert_eq!(
        result
            .spectrum
            .processing
            .last()
            .map(|record| record.operation.as_str()),
        Some("auto_phase_correct_2d")
    );
    Ok(())
}

#[test]
fn applies_processing_recipe_2d_json() -> anyhow::Result<()> {
    let spectrum_json = to_json(&complex_spectrum()?)?;
    let processed_json = apply_processing_recipe_2d_json(
        &spectrum_json,
        r#"{"format":"rspin.processing_recipe_2d","version":1,"recipe":{"operations":[{"operation":"scale","factor":2.0},{"operation":"absolute_value"},{"operation":"zero_fill","target_x_len":3,"target_y_len":2},{"operation":"normalize_max_abs"}]}}"#,
    )?;
    let processed = spectrum2d_from_json(&processed_json)?;

    assert_eq!(processed.shape(), (3, 2));
    assert_vec_close(&processed.z, &[0.25, 0.5, 0.0, 0.75, 1.0, 0.0]);
    assert_option_vec_close(
        processed.imaginary.as_deref(),
        &[0.125, 0.25, 0.0, 0.375, 0.5, 0.0],
    );
    assert_eq!(
        processed
            .processing
            .iter()
            .map(|record| record.operation.as_str())
            .collect::<Vec<_>>(),
        vec!["scale_2d", "abs_2d", "zero_fill_2d", "normalize_2d_max_abs"]
    );
    Ok(())
}

#[test]
fn applies_processing_recipe_2d_prefix_json() -> anyhow::Result<()> {
    let spectrum_json = to_json(&complex_spectrum()?)?;
    let processed_json = apply_processing_recipe_2d_until_json(
        &spectrum_json,
        r#"{"operations":[{"operation":"scale","factor":2.0},{"operation":"absolute_value"},{"operation":"zero_fill","target_x_len":3,"target_y_len":2},{"operation":"normalize_max_abs"}]}"#,
        2,
    )?;
    let processed = spectrum2d_from_json(&processed_json)?;

    assert_eq!(processed.shape(), (2, 2));
    assert_vec_close(&processed.z, &[2.0, 4.0, 6.0, 8.0]);
    assert_eq!(processed.processing.len(), 2);

    let error = apply_processing_recipe_2d_until_json(
        &spectrum_json,
        r#"{"operations":[{"operation":"scale","factor":2.0}]}"#,
        2,
    )
    .expect_err("too many operations should fail");
    assert!(error.to_string().contains("requested"));
    Ok(())
}

#[test]
fn projects_and_slices_2d_spectrum_json() -> anyhow::Result<()> {
    let spectrum_json = to_json(&grid_spectrum()?)?;

    let x_projection_json = project_spectrum_2d_x_json(&spectrum_json, r#""sum""#)?;
    let x_projection = spectrum1d_from_json(&x_projection_json)?;
    assert_vec_close(&x_projection.intensities, &[5.0, 7.0, 9.0]);
    assert_option_vec_close(x_projection.imaginary.as_deref(), &[23.0, 26.0, 28.0]);

    let y_projection_json = project_spectrum_2d_y_json(&spectrum_json, r#""mean""#)?;
    let y_projection = spectrum1d_from_json(&y_projection_json)?;
    assert_vec_close(&y_projection.intensities, &[2.0, 5.0]);
    assert_option_vec_close(y_projection.imaginary.as_deref(), &[11.0, 44.0 / 3.0]);

    let x_slice_json = slice_spectrum_2d_x_at_y_index_json(&spectrum_json, 1)?;
    let x_slice = spectrum1d_from_json(&x_slice_json)?;
    assert_vec_close(&x_slice.intensities, &[4.0, 5.0, 6.0]);

    let x_slice_json = slice_spectrum_2d_x_at_y_json(&spectrum_json, 0.6)?;
    let x_slice = spectrum1d_from_json(&x_slice_json)?;
    assert_vec_close(&x_slice.intensities, &[4.0, 5.0, 6.0]);
    assert_eq!(
        x_slice
            .processing
            .last()
            .map(|record| record.operation.as_str()),
        Some("slice_x_at_y")
    );

    let y_slice_json = slice_spectrum_2d_y_at_x_index_json(&spectrum_json, 2)?;
    let y_slice = spectrum1d_from_json(&y_slice_json)?;
    assert_vec_close(&y_slice.intensities, &[3.0, 6.0]);

    let y_slice_json = slice_spectrum_2d_y_at_x_json(&spectrum_json, 1.6)?;
    let y_slice = spectrum1d_from_json(&y_slice_json)?;
    assert_vec_close(&y_slice.intensities, &[3.0, 6.0]);
    Ok(())
}

#[test]
fn rejects_invalid_2d_processing_json_options() -> anyhow::Result<()> {
    let spectrum_json = to_json(&grid_spectrum()?)?;

    let error = project_spectrum_2d_x_json(&spectrum_json, r#""median""#)
        .expect_err("invalid projection mode should fail");
    assert!(error.to_string().contains("unknown variant"));

    let error = phase_spectrum_2d_json(&spectrum_json, r#"{"x_pivot_fraction":1.5}"#)
        .expect_err("invalid phase pivot should fail");
    assert!(error.to_string().contains("pivot"));

    let error = exponential_apodization_spectrum_2d_json(
        &spectrum_json,
        r#"{"x_line_broadening_hz":-1.0,"y_line_broadening_hz":1.0,"x_dwell_time_s":0.1,"y_dwell_time_s":0.1}"#,
    )
    .expect_err("negative exponential broadening should fail");
    assert!(error.to_string().contains("x_line_broadening_hz"));
    Ok(())
}

fn complex_spectrum() -> anyhow::Result<Spectrum2D> {
    Spectrum2D::new_complex(
        Axis::linear("x", Unit::Ppm, 0.0, 1.0, 2)?,
        Axis::linear("y", Unit::Ppm, 0.0, 1.0, 2)?,
        vec![1.0, -2.0, 3.0, 4.0],
        Some(vec![0.5, -1.0, 1.5, 2.0]),
        Metadata::default(),
    )
    .map_err(Into::into)
}

fn positive_spectrum() -> anyhow::Result<Spectrum2D> {
    Spectrum2D::new(
        Axis::linear("x", Unit::Ppm, 0.0, 1.0, 2)?,
        Axis::linear("y", Unit::Ppm, 0.0, 1.0, 2)?,
        vec![0.0, 1.0, 2.0, 0.0],
        Metadata::default(),
    )
    .map_err(Into::into)
}

fn grid_spectrum() -> anyhow::Result<Spectrum2D> {
    Spectrum2D::new_complex(
        Axis::linear("x", Unit::Ppm, 0.0, 2.0, 3)?,
        Axis::linear("y", Unit::Ppm, 0.0, 1.0, 2)?,
        vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
        Some(vec![10.0, 11.0, 12.0, 13.0, 15.0, 16.0]),
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
