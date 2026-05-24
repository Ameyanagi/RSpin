use rspin_core::{Axis, Metadata, RSpinError, Spectrum1D, Unit};

use super::*;

#[test]
fn applies_chainable_recipe_operations() -> anyhow::Result<()> {
    let spectrum = demo_spectrum()?;
    let recipe = ProcessingRecipe1D::new()
        .scale(2.0)
        .offset(-2.0)
        .absolute_value()
        .crop(0.0, 1.0)
        .resample(Axis::linear("shift", Unit::Ppm, 0.0, 1.0, 3)?)
        .zero_fill(5)
        .normalize_max_abs();

    let processed = recipe.apply(&spectrum)?;

    assert_eq!(recipe.len(), 7);
    assert_eq!(processed.len(), 5);
    assert_eq!(processed.intensities, vec![0.0, 0.5, 1.0, 0.0, 0.0]);
    assert_eq!(processed.processing.len(), 7);
    assert_eq!(processed.processing[0].operation, "scale_intensity");
    assert_eq!(processed.processing[6].operation, "normalize_max_abs");
    Ok(())
}

#[test]
fn applies_recipe_prefix_for_rollback_reapply() -> anyhow::Result<()> {
    let spectrum = demo_spectrum()?;
    let recipe = ProcessingRecipe1D::new()
        .scale(2.0)
        .offset(-2.0)
        .absolute_value()
        .normalize_max_abs();

    let rolled_back = recipe.apply_until(&spectrum, 2)?;
    let replayed = recipe.prefix(3)?.apply(&spectrum)?;
    let without_last = recipe.without_last();

    assert_eq!(rolled_back.intensities, vec![0.0, -6.0, 6.0]);
    assert_eq!(rolled_back.processing.len(), 2);
    assert_eq!(replayed.intensities, vec![0.0, 6.0, 6.0]);
    assert_eq!(without_last.len(), 3);
    assert_eq!(
        without_last.apply(&spectrum)?.intensities,
        replayed.intensities
    );
    Ok(())
}

#[test]
fn rejects_recipe_prefix_past_end() -> anyhow::Result<()> {
    let recipe = ProcessingRecipe1D::new().scale(2.0);
    let error = apply_processing_recipe_1d_until(&demo_spectrum()?, &recipe, 2)
        .expect_err("too many operations should fail");

    assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));
    Ok(())
}

#[test]
fn round_trips_recipe_json_and_applies_step_trait() -> anyhow::Result<()> {
    let recipe = ProcessingRecipe1D::new()
        .scale(2.0)
        .subtract_baseline_with(BaselineMethod::Constant { value: 1.0 });
    let json = serde_json::to_string(&recipe)?;
    let decoded: ProcessingRecipe1D = serde_json::from_str(&json)?;
    let processed = decoded.apply(&demo_spectrum()?)?;

    assert_eq!(decoded.len(), 2);
    assert_eq!(processed.intensities, vec![1.0, -5.0, 7.0]);
    assert_eq!(processed.processing[1].operation, "baseline_constant");
    Ok(())
}

#[test]
fn preserves_first_recipe_error() -> anyhow::Result<()> {
    let recipe = ProcessingRecipe1D::new().scale(f64::NAN).offset(10.0);
    let error = recipe
        .apply(&demo_spectrum()?)
        .expect_err("non-finite scale should fail");

    assert!(matches!(error, RSpinError::NonFinite { .. }));
    Ok(())
}

fn demo_spectrum() -> anyhow::Result<Spectrum1D> {
    Ok(Spectrum1D::new(
        Axis::linear("shift", Unit::Ppm, 0.0, 2.0, 3)?,
        vec![1.0, -2.0, 4.0],
        Metadata::default(),
    )?)
}
