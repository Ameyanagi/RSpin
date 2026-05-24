//! JSON serialization for prediction payloads.

use rspin_core::{RSpinError, Result};
use rspin_prediction::PredictionSet;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{SpectrumReader, SpectrumWriter};

/// Current version of `RSpin`'s prediction JSON envelope.
pub const PREDICTION_JSON_VERSION: u32 = 1;

/// Format identifier for prediction JSON.
pub const PREDICTION_JSON_FORMAT: &str = "rspin.prediction";

/// JSON reader/writer for prediction payloads.
#[derive(Clone, Copy, Debug, Default)]
pub struct JsonPrediction;

impl SpectrumReader for JsonPrediction {
    type Output = PredictionSet;

    fn read_str(&self, input: &str) -> Result<Self::Output> {
        read_prediction_json(input)
    }
}

impl SpectrumWriter<PredictionSet> for JsonPrediction {
    fn write_string(&self, prediction: &PredictionSet) -> Result<String> {
        write_prediction_json(prediction)
    }
}

/// Reads a prediction payload from JSON.
///
/// The reader accepts the current versioned envelope and legacy raw
/// `PredictionSet` JSON payloads.
///
/// # Errors
///
/// Returns an error when JSON deserialization fails, the envelope format is not
/// a prediction payload, or the envelope version is unsupported.
pub fn read_prediction_json(input: &str) -> Result<PredictionSet> {
    let value = json_value(input)?;
    if is_versioned_prediction_document(&value) {
        validate_prediction_document_header(&value)?;
        let document: PredictionDocument =
            serde_json::from_value(value).map_err(|error| json_error(&error))?;
        return Ok(document.prediction);
    }
    serde_json::from_value(value).map_err(|error| json_error(&error))
}

/// Writes a prediction payload to compact versioned JSON.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn write_prediction_json(prediction: &PredictionSet) -> Result<String> {
    let document = PredictionDocumentRef {
        format: PREDICTION_JSON_FORMAT,
        version: PREDICTION_JSON_VERSION,
        prediction,
    };
    serde_json::to_string(&document).map_err(|error| json_error(&error))
}

#[derive(Debug, Deserialize)]
struct PredictionDocument {
    prediction: PredictionSet,
}

#[derive(Debug, Serialize)]
struct PredictionDocumentRef<'a> {
    format: &'static str,
    version: u32,
    prediction: &'a PredictionSet,
}

#[derive(Debug, Deserialize)]
struct PredictionDocumentHeader {
    format: String,
    version: u32,
}

fn is_versioned_prediction_document(value: &Value) -> bool {
    value.as_object().is_some_and(|object| {
        object.contains_key("format")
            || object.contains_key("version")
            || object.contains_key("prediction")
    })
}

fn validate_prediction_document_header(value: &Value) -> Result<()> {
    let header: PredictionDocumentHeader =
        serde_json::from_value(value.clone()).map_err(|error| json_error(&error))?;
    if header.format != PREDICTION_JSON_FORMAT {
        return Err(RSpinError::Parse {
            format: "JSON",
            message: format!(
                "expected prediction format '{PREDICTION_JSON_FORMAT}' but found '{}'",
                header.format
            ),
        });
    }
    if header.version != PREDICTION_JSON_VERSION {
        return Err(RSpinError::Unsupported {
            feature: "prediction JSON version",
        });
    }
    Ok(())
}

fn json_value(input: &str) -> Result<Value> {
    serde_json::from_str(input).map_err(|error| json_error(&error))
}

fn json_error(error: &serde_json::Error) -> RSpinError {
    RSpinError::Parse {
        format: "JSON",
        message: error.to_string(),
    }
}

#[cfg(test)]
mod tests;
