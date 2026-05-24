//! JSON serialization for processing recipes.

use rspin_core::{RSpinError, Result};
use rspin_processing::{ProcessingRecipe1D, ProcessingRecipe2D};

use crate::{SpectrumReader, SpectrumWriter};

/// JSON reader/writer for one-dimensional processing recipes.
#[derive(Clone, Copy, Debug, Default)]
pub struct JsonProcessingRecipe1D;

impl SpectrumReader for JsonProcessingRecipe1D {
    type Output = ProcessingRecipe1D;

    fn read_str(&self, input: &str) -> Result<Self::Output> {
        read_processing_recipe_1d_json(input)
    }
}

impl SpectrumWriter<ProcessingRecipe1D> for JsonProcessingRecipe1D {
    fn write_string(&self, recipe: &ProcessingRecipe1D) -> Result<String> {
        write_processing_recipe_1d_json(recipe)
    }
}

/// JSON reader/writer for two-dimensional processing recipes.
#[derive(Clone, Copy, Debug, Default)]
pub struct JsonProcessingRecipe2D;

impl SpectrumReader for JsonProcessingRecipe2D {
    type Output = ProcessingRecipe2D;

    fn read_str(&self, input: &str) -> Result<Self::Output> {
        read_processing_recipe_2d_json(input)
    }
}

impl SpectrumWriter<ProcessingRecipe2D> for JsonProcessingRecipe2D {
    fn write_string(&self, recipe: &ProcessingRecipe2D) -> Result<String> {
        write_processing_recipe_2d_json(recipe)
    }
}

/// Reads a one-dimensional processing recipe from JSON.
///
/// # Errors
///
/// Returns an error when JSON deserialization fails.
pub fn read_processing_recipe_1d_json(input: &str) -> Result<ProcessingRecipe1D> {
    serde_json::from_str(input).map_err(|error| json_error(&error))
}

/// Writes a one-dimensional processing recipe to compact JSON.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn write_processing_recipe_1d_json(recipe: &ProcessingRecipe1D) -> Result<String> {
    serde_json::to_string(recipe).map_err(|error| json_error(&error))
}

/// Reads a two-dimensional processing recipe from JSON.
///
/// # Errors
///
/// Returns an error when JSON deserialization fails.
pub fn read_processing_recipe_2d_json(input: &str) -> Result<ProcessingRecipe2D> {
    serde_json::from_str(input).map_err(|error| json_error(&error))
}

/// Writes a two-dimensional processing recipe to compact JSON.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn write_processing_recipe_2d_json(recipe: &ProcessingRecipe2D) -> Result<String> {
    serde_json::to_string(recipe).map_err(|error| json_error(&error))
}

fn json_error(error: &serde_json::Error) -> RSpinError {
    RSpinError::Parse {
        format: "JSON",
        message: error.to_string(),
    }
}

#[cfg(test)]
mod tests;
