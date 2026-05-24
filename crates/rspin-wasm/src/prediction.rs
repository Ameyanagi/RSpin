//! WebAssembly bindings for prediction workflows.

use wasm_bindgen::prelude::*;

use crate::{
    js_error, predict_formula_with_element_rules_json, predict_molecule_with_element_rules_json,
    render_prediction_1d_json, render_prediction_2d_json, validate_prediction_json,
};

/// Validates a serialized prediction payload and returns its normalized JSON.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, validation, or
/// serialization fails.
#[wasm_bindgen(js_name = validatePrediction)]
pub fn validate_prediction(prediction_json: &str) -> std::result::Result<String, JsValue> {
    validate_prediction_json(prediction_json).map_err(|error| js_error(&error))
}

/// Predicts molecule signals with serialized element shift rules.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, prediction,
/// validation, or serialization fails.
#[wasm_bindgen(js_name = predictMoleculeWithElementRules)]
pub fn predict_molecule_with_element_rules(
    molecule_json: &str,
    predictor_json: &str,
) -> std::result::Result<String, JsValue> {
    predict_molecule_with_element_rules_json(molecule_json, predictor_json)
        .map_err(|error| js_error(&error))
}

/// Predicts formula-expanded molecule signals with serialized element shift rules.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, formula expansion,
/// prediction, validation, or serialization fails.
#[wasm_bindgen(js_name = predictFormulaWithElementRules)]
pub fn predict_formula_with_element_rules(
    molecule_id: &str,
    formula: &str,
    predictor_json: &str,
) -> std::result::Result<String, JsValue> {
    predict_formula_with_element_rules_json(molecule_id, formula, predictor_json)
        .map_err(|error| js_error(&error))
}

/// Renders a serialized one-dimensional prediction as a spectrum.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, validation,
/// rendering, or serialization fails.
#[wasm_bindgen(js_name = renderPrediction1d)]
pub fn render_prediction_1d(
    prediction_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    render_prediction_1d_json(prediction_json, options_json).map_err(|error| js_error(&error))
}

/// Renders a serialized two-dimensional prediction as a spectrum.
///
/// # Errors
///
/// Returns a JavaScript error string when deserialization, validation,
/// rendering, or serialization fails.
#[wasm_bindgen(js_name = renderPrediction2d)]
pub fn render_prediction_2d(
    prediction_json: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    render_prediction_2d_json(prediction_json, options_json).map_err(|error| js_error(&error))
}
