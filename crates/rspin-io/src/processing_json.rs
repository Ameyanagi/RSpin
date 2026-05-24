//! JSON serialization for processing recipes.

use rspin_core::{RSpinError, Result};
use rspin_processing::{ProcessingRecipe1D, ProcessingRecipe2D};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{SpectrumReader, SpectrumWriter};

/// Current version of `RSpin`'s processing-recipe JSON envelope.
pub const PROCESSING_RECIPE_JSON_VERSION: u32 = 1;

/// Format identifier for one-dimensional processing-recipe JSON.
pub const PROCESSING_RECIPE_1D_FORMAT: &str = "rspin.processing_recipe_1d";

/// Format identifier for two-dimensional processing-recipe JSON.
pub const PROCESSING_RECIPE_2D_FORMAT: &str = "rspin.processing_recipe_2d";

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
/// The reader accepts the current versioned envelope and legacy raw recipe
/// payloads shaped as `{"operations":[...]}`.
///
/// # Errors
///
/// Returns an error when JSON deserialization fails, the envelope format is not
/// one-dimensional, or the envelope version is unsupported.
pub fn read_processing_recipe_1d_json(input: &str) -> Result<ProcessingRecipe1D> {
    let value = json_value(input)?;
    if is_versioned_recipe_document(&value) {
        validate_recipe_document_header(&value, PROCESSING_RECIPE_1D_FORMAT)?;
        let document: ProcessingRecipe1DDocument =
            serde_json::from_value(value).map_err(|error| json_error(&error))?;
        return Ok(document.recipe);
    }
    serde_json::from_value(value).map_err(|error| json_error(&error))
}

/// Writes a one-dimensional processing recipe to compact versioned JSON.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn write_processing_recipe_1d_json(recipe: &ProcessingRecipe1D) -> Result<String> {
    let document = ProcessingRecipe1DDocumentRef {
        format: PROCESSING_RECIPE_1D_FORMAT,
        version: PROCESSING_RECIPE_JSON_VERSION,
        recipe,
    };
    serde_json::to_string(&document).map_err(|error| json_error(&error))
}

/// Reads a two-dimensional processing recipe from JSON.
///
/// The reader accepts the current versioned envelope and legacy raw recipe
/// payloads shaped as `{"operations":[...]}`.
///
/// # Errors
///
/// Returns an error when JSON deserialization fails, the envelope format is not
/// two-dimensional, or the envelope version is unsupported.
pub fn read_processing_recipe_2d_json(input: &str) -> Result<ProcessingRecipe2D> {
    let value = json_value(input)?;
    if is_versioned_recipe_document(&value) {
        validate_recipe_document_header(&value, PROCESSING_RECIPE_2D_FORMAT)?;
        let document: ProcessingRecipe2DDocument =
            serde_json::from_value(value).map_err(|error| json_error(&error))?;
        return Ok(document.recipe);
    }
    serde_json::from_value(value).map_err(|error| json_error(&error))
}

/// Writes a two-dimensional processing recipe to compact versioned JSON.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn write_processing_recipe_2d_json(recipe: &ProcessingRecipe2D) -> Result<String> {
    let document = ProcessingRecipe2DDocumentRef {
        format: PROCESSING_RECIPE_2D_FORMAT,
        version: PROCESSING_RECIPE_JSON_VERSION,
        recipe,
    };
    serde_json::to_string(&document).map_err(|error| json_error(&error))
}

#[derive(Debug, Deserialize)]
struct ProcessingRecipe1DDocument {
    recipe: ProcessingRecipe1D,
}

#[derive(Debug, Serialize)]
struct ProcessingRecipe1DDocumentRef<'a> {
    format: &'static str,
    version: u32,
    recipe: &'a ProcessingRecipe1D,
}

#[derive(Debug, Deserialize)]
struct ProcessingRecipe2DDocument {
    recipe: ProcessingRecipe2D,
}

#[derive(Debug, Serialize)]
struct ProcessingRecipe2DDocumentRef<'a> {
    format: &'static str,
    version: u32,
    recipe: &'a ProcessingRecipe2D,
}

#[derive(Debug, Deserialize)]
struct ProcessingRecipeDocumentHeader {
    format: String,
    version: u32,
}

fn json_value(input: &str) -> Result<Value> {
    serde_json::from_str(input).map_err(|error| json_error(&error))
}

fn is_versioned_recipe_document(value: &Value) -> bool {
    match value {
        Value::Object(object) => {
            object.contains_key("format")
                || object.contains_key("version")
                || object.contains_key("recipe")
        }
        _ => false,
    }
}

fn validate_recipe_document(
    format: &str,
    version: u32,
    expected_format: &'static str,
) -> Result<()> {
    if format != expected_format {
        return Err(RSpinError::Parse {
            format: "JSON",
            message: format!(
                "expected processing recipe format '{expected_format}' but found '{format}'"
            ),
        });
    }
    if version != PROCESSING_RECIPE_JSON_VERSION {
        return Err(RSpinError::Unsupported {
            feature: "processing recipe JSON version",
        });
    }
    Ok(())
}

fn validate_recipe_document_header(value: &Value, expected_format: &'static str) -> Result<()> {
    let header: ProcessingRecipeDocumentHeader =
        serde_json::from_value(value.clone()).map_err(|error| json_error(&error))?;
    validate_recipe_document(&header.format, header.version, expected_format)
}

fn json_error(error: &serde_json::Error) -> RSpinError {
    RSpinError::Parse {
        format: "JSON",
        message: error.to_string(),
    }
}

#[cfg(test)]
mod tests;
