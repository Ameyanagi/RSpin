use rspin_core::{Axis, Metadata, RSpinError, Spectrum1D, Unit};
use rspin_processing::{FftDirection, PhaseCorrection2D, ProcessingRecipe1D, ProcessingRecipe2D};

use crate::{
    JsonProcessingRecipe1D, JsonProcessingRecipe2D, PROCESSING_RECIPE_1D_FORMAT,
    PROCESSING_RECIPE_2D_FORMAT, PROCESSING_RECIPE_JSON_VERSION, SpectrumReader, SpectrumWriter,
    read_processing_recipe_1d_json, read_processing_recipe_2d_json,
    write_processing_recipe_1d_json, write_processing_recipe_2d_json,
};

#[test]
fn round_trips_one_dimensional_recipe_json() -> anyhow::Result<()> {
    let recipe = ProcessingRecipe1D::new()
        .scale(2.0)
        .offset(-1.0)
        .absolute_value()
        .normalize_max_abs();
    let codec = JsonProcessingRecipe1D;

    let json = codec.write_string(&recipe)?;
    let decoded = codec.read_str(&json)?;
    let processed = decoded.apply(&spectrum_1d()?)?;

    assert!(json.contains(PROCESSING_RECIPE_1D_FORMAT));
    assert!(json.contains(&format!("\"version\":{PROCESSING_RECIPE_JSON_VERSION}")));
    assert_eq!(read_processing_recipe_1d_json(&json)?, recipe);
    assert_eq!(write_processing_recipe_1d_json(&decoded)?, json);
    assert_eq!(decoded, recipe);
    assert_eq!(processed.intensities, vec![0.2, 1.0, 1.0]);
    Ok(())
}

#[test]
fn round_trips_two_dimensional_recipe_json() -> anyhow::Result<()> {
    let recipe = ProcessingRecipe2D::new()
        .scale(2.0)
        .phase(PhaseCorrection2D::new().x_phase(0.0, 0.0, 0.5))
        .fft(FftDirection::Inverse)
        .normalize_max_abs();
    let codec = JsonProcessingRecipe2D;

    let json = codec.write_string(&recipe)?;
    let decoded = codec.read_str(&json)?;

    assert!(json.contains(PROCESSING_RECIPE_2D_FORMAT));
    assert!(json.contains(&format!("\"version\":{PROCESSING_RECIPE_JSON_VERSION}")));
    assert_eq!(read_processing_recipe_2d_json(&json)?, recipe);
    assert_eq!(write_processing_recipe_2d_json(&decoded)?, json);
    assert_eq!(decoded.len(), 4);
    Ok(())
}

#[test]
fn reads_legacy_raw_processing_recipe_json() -> anyhow::Result<()> {
    let recipe_1d = ProcessingRecipe1D::new().scale(2.0).normalize_max_abs();
    let raw_1d = serde_json::to_string(&recipe_1d)?;
    assert_eq!(read_processing_recipe_1d_json(&raw_1d)?, recipe_1d);

    let recipe_2d = ProcessingRecipe2D::new().scale(2.0).normalize_max_abs();
    let raw_2d = serde_json::to_string(&recipe_2d)?;
    assert_eq!(read_processing_recipe_2d_json(&raw_2d)?, recipe_2d);
    Ok(())
}

#[test]
fn rejects_unsupported_processing_recipe_json_version() {
    let error = read_processing_recipe_1d_json(
        r#"{"format":"rspin.processing_recipe_1d","version":2,"recipe":{"operations":[]}}"#,
    )
    .expect_err("unsupported recipe JSON version should fail");

    assert!(matches!(error, RSpinError::Unsupported { .. }));
}

#[test]
fn rejects_wrong_processing_recipe_json_format() {
    let error = read_processing_recipe_1d_json(
        r#"{"format":"rspin.processing_recipe_2d","version":1,"recipe":{"operations":[]}}"#,
    )
    .expect_err("wrong recipe JSON format should fail");

    assert!(matches!(error, RSpinError::Parse { format: "JSON", .. }));
}

#[test]
fn rejects_malformed_processing_recipe_json() {
    let error = read_processing_recipe_1d_json("{").expect_err("malformed JSON should be rejected");

    assert!(matches!(error, RSpinError::Parse { format: "JSON", .. }));
}

fn spectrum_1d() -> anyhow::Result<Spectrum1D> {
    Ok(Spectrum1D::new(
        Axis::linear("shift", Unit::Ppm, 0.0, 2.0, 3)?,
        vec![1.0, -2.0, 3.0],
        Metadata::default(),
    )?)
}
