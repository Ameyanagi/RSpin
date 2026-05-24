//! Prediction JSON helpers.

use rspin_core::{Molecule, Result, Spectrum1D, Spectrum2D};
use rspin_prediction::{
    ElementShiftPredictor, PredictionSet, PredictionSpectrum2DOptions, PredictionSpectrumOptions,
    predict_formula_with_rules, predict_molecule_with_rules, render_prediction_1d,
    render_prediction_2d,
};

use super::{from_json, spectrum1d_to_json, spectrum2d_to_json, to_json};

/// Predicts molecule signals with a serialized element shift rule table.
///
/// # Errors
///
/// Returns an error when deserialization, prediction, validation, or
/// serialization fails.
pub fn predict_molecule_with_element_rules_json(
    molecule_json: &str,
    predictor_json: &str,
) -> Result<String> {
    let molecule: Molecule = from_json(molecule_json)?;
    let predictor: ElementShiftPredictor = from_json(predictor_json)?;
    let prediction = predict_molecule_with_rules(&molecule, &predictor)?;
    to_json(&prediction)
}

/// Predicts formula-expanded molecule signals with serialized element shift rules.
///
/// # Errors
///
/// Returns an error when deserialization, formula expansion, prediction,
/// validation, or serialization fails.
pub fn predict_formula_with_element_rules_json(
    molecule_id: &str,
    formula: &str,
    predictor_json: &str,
) -> Result<String> {
    let predictor: ElementShiftPredictor = from_json(predictor_json)?;
    let prediction = predict_formula_with_rules(molecule_id, formula, &predictor)?;
    to_json(&prediction)
}

/// Validates serialized prediction JSON and returns normalized JSON.
///
/// # Errors
///
/// Returns an error when deserialization, validation, or serialization fails.
pub fn validate_prediction_json(prediction_json: &str) -> Result<String> {
    let prediction: PredictionSet = from_json(prediction_json)?;
    prediction.validate()?;
    to_json(&prediction)
}

/// Renders serialized one-dimensional prediction JSON into `Spectrum1D` JSON.
///
/// # Errors
///
/// Returns an error when deserialization, validation, rendering, or serialization fails.
pub fn render_prediction_1d_json(prediction_json: &str, options_json: &str) -> Result<String> {
    let prediction: PredictionSet = from_json(prediction_json)?;
    let options: PredictionSpectrumOptions = from_json(options_json)?;
    let spectrum: Spectrum1D = render_prediction_1d(&prediction, &options)?;
    spectrum1d_to_json(&spectrum)
}

/// Renders serialized two-dimensional prediction JSON into `Spectrum2D` JSON.
///
/// # Errors
///
/// Returns an error when deserialization, validation, rendering, or serialization fails.
pub fn render_prediction_2d_json(prediction_json: &str, options_json: &str) -> Result<String> {
    let prediction: PredictionSet = from_json(prediction_json)?;
    let options: PredictionSpectrum2DOptions = from_json(options_json)?;
    let spectrum: Spectrum2D = render_prediction_2d(&prediction, &options)?;
    spectrum2d_to_json(&spectrum)
}
