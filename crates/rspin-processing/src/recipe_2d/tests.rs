use rspin_core::{Axis, Metadata, RSpinError, Spectrum2D, Unit};

use super::*;

#[test]
fn applies_chainable_recipe_operations() -> anyhow::Result<()> {
    let spectrum = demo_spectrum()?;
    let recipe = ProcessingRecipe2D::new()
        .scale(2.0)
        .absolute_value()
        .zero_fill(3, 2)
        .normalize_max_abs()
        .normalize_abs_volume(2.0);

    let processed = recipe.apply(&spectrum)?;

    assert_eq!(recipe.len(), 5);
    assert_eq!(processed.shape(), (3, 2));
    assert_eq!(processed.z, vec![0.5, 1.0, 0.0, 1.5, 2.0, 0.0]);
    assert_eq!(processed.processing.len(), 5);
    assert_eq!(processed.processing[0].operation, "scale_2d");
    assert_eq!(processed.processing[3].operation, "normalize_2d_max_abs");
    assert_eq!(processed.processing[4].operation, "normalize_2d_volume");
    Ok(())
}

#[test]
fn applies_recipe_prefix_for_rollback_reapply() -> anyhow::Result<()> {
    let spectrum = demo_spectrum()?;
    let recipe = ProcessingRecipe2D::new()
        .scale(2.0)
        .absolute_value()
        .zero_fill(3, 2)
        .normalize_max_abs();

    let rolled_back = recipe.apply_until(&spectrum, 2)?;
    let replayed = recipe.prefix(3)?.apply(&spectrum)?;
    let without_last = recipe.without_last();

    assert_eq!(rolled_back.shape(), (2, 2));
    assert_eq!(rolled_back.z, vec![2.0, 4.0, 6.0, 8.0]);
    assert_eq!(rolled_back.processing.len(), 2);
    assert_eq!(replayed.shape(), (3, 2));
    assert_eq!(replayed.z, vec![2.0, 4.0, 0.0, 6.0, 8.0, 0.0]);
    assert_eq!(without_last.len(), 3);
    assert_eq!(without_last.apply(&spectrum)?.z, replayed.z);
    Ok(())
}

#[test]
fn rejects_recipe_prefix_past_end() -> anyhow::Result<()> {
    let recipe = ProcessingRecipe2D::new().scale(2.0);
    let error = apply_processing_recipe_2d_until(&demo_spectrum()?, &recipe, 2)
        .expect_err("too many operations should fail");

    assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));
    Ok(())
}

#[test]
fn round_trips_recipe_json_and_applies_step_trait() -> anyhow::Result<()> {
    let recipe = ProcessingRecipe2D::new()
        .normalize_volume(-1.0)
        .crop(0.0, 1.0, 1.0, 1.0)
        .gaussian_apodization(0.0, 0.0, 0.1, 0.1)
        .sine_bell_apodization(90.0, 90.0, 1.0, 90.0, 90.0, 1.0)
        .resample(
            Axis::linear("x", Unit::Ppm, 0.0, 1.0, 3)?,
            Axis::ppm(vec![1.0])?,
        );
    let json = serde_json::to_string(&recipe)?;
    let decoded: ProcessingRecipe2D = serde_json::from_str(&json)?;
    let processed = ProcessingStep::apply(&decoded, &demo_spectrum()?)?;

    assert_eq!(decoded.len(), 5);
    assert_eq!(processed.shape(), (3, 1));
    assert_eq!(processed.z, vec![6.0, -1.0, -8.0]);
    assert_eq!(processed.processing[0].operation, "normalize_2d_volume");
    assert_eq!(
        processed.processing[3].operation,
        "sine_bell_apodization_2d"
    );
    assert_eq!(processed.processing[4].operation, "resample_2d");
    assert!(json.contains("normalize_volume"));
    Ok(())
}

#[test]
fn preserves_first_recipe_error() -> anyhow::Result<()> {
    let recipe = ProcessingRecipe2D::new()
        .scale(f64::NAN)
        .normalize_max_abs();
    let error = recipe
        .apply(&demo_spectrum()?)
        .expect_err("non-finite scale should fail");

    assert!(matches!(error, RSpinError::NonFinite { .. }));
    Ok(())
}

fn demo_spectrum() -> anyhow::Result<Spectrum2D> {
    Ok(Spectrum2D::new(
        Axis::linear("x", Unit::Ppm, 0.0, 1.0, 2)?,
        Axis::linear("y", Unit::Ppm, 0.0, 1.0, 2)?,
        vec![1.0, -2.0, 3.0, -4.0],
        Metadata::default(),
    )?)
}
