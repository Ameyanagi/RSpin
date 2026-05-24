//! JSON serialization for analysis workflow results.

use rspin_analysis::{SpectrumAnalysis1D, SpectrumAnalysis2D};
use rspin_core::{RSpinError, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{SpectrumReader, SpectrumWriter};

/// Current version of `RSpin`'s analysis JSON envelope.
pub const ANALYSIS_JSON_VERSION: u32 = 1;

/// Format identifier for one-dimensional analysis JSON.
pub const ANALYSIS_1D_JSON_FORMAT: &str = "rspin.analysis_1d";

/// Format identifier for two-dimensional analysis JSON.
pub const ANALYSIS_2D_JSON_FORMAT: &str = "rspin.analysis_2d";

/// JSON reader/writer for one-dimensional analysis workflow results.
#[derive(Clone, Copy, Debug, Default)]
pub struct JsonAnalysis1D;

impl SpectrumReader for JsonAnalysis1D {
    type Output = SpectrumAnalysis1D;

    fn read_str(&self, input: &str) -> Result<Self::Output> {
        read_analysis1d_json(input)
    }
}

impl SpectrumWriter<SpectrumAnalysis1D> for JsonAnalysis1D {
    fn write_string(&self, analysis: &SpectrumAnalysis1D) -> Result<String> {
        write_analysis1d_json(analysis)
    }
}

/// JSON reader/writer for two-dimensional analysis workflow results.
#[derive(Clone, Copy, Debug, Default)]
pub struct JsonAnalysis2D;

impl SpectrumReader for JsonAnalysis2D {
    type Output = SpectrumAnalysis2D;

    fn read_str(&self, input: &str) -> Result<Self::Output> {
        read_analysis2d_json(input)
    }
}

impl SpectrumWriter<SpectrumAnalysis2D> for JsonAnalysis2D {
    fn write_string(&self, analysis: &SpectrumAnalysis2D) -> Result<String> {
        write_analysis2d_json(analysis)
    }
}

/// Reads a one-dimensional analysis workflow result from JSON.
///
/// The reader accepts the current versioned envelope and legacy raw
/// `SpectrumAnalysis1D` JSON payloads.
///
/// # Errors
///
/// Returns an error when JSON deserialization fails, the envelope format is not
/// one-dimensional, or the envelope version is unsupported.
pub fn read_analysis1d_json(input: &str) -> Result<SpectrumAnalysis1D> {
    let value = json_value(input)?;
    if is_versioned_analysis_document(&value) {
        validate_analysis_document_header(&value, ANALYSIS_1D_JSON_FORMAT)?;
        let document: Analysis1DDocument =
            serde_json::from_value(value).map_err(|error| json_error(&error))?;
        return Ok(document.analysis);
    }
    serde_json::from_value(value).map_err(|error| json_error(&error))
}

/// Writes a one-dimensional analysis workflow result to compact versioned JSON.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn write_analysis1d_json(analysis: &SpectrumAnalysis1D) -> Result<String> {
    let document = Analysis1DDocumentRef {
        format: ANALYSIS_1D_JSON_FORMAT,
        version: ANALYSIS_JSON_VERSION,
        analysis,
    };
    serde_json::to_string(&document).map_err(|error| json_error(&error))
}

/// Reads a two-dimensional analysis workflow result from JSON.
///
/// The reader accepts the current versioned envelope and legacy raw
/// `SpectrumAnalysis2D` JSON payloads.
///
/// # Errors
///
/// Returns an error when JSON deserialization fails, the envelope format is not
/// two-dimensional, or the envelope version is unsupported.
pub fn read_analysis2d_json(input: &str) -> Result<SpectrumAnalysis2D> {
    let value = json_value(input)?;
    if is_versioned_analysis_document(&value) {
        validate_analysis_document_header(&value, ANALYSIS_2D_JSON_FORMAT)?;
        let document: Analysis2DDocument =
            serde_json::from_value(value).map_err(|error| json_error(&error))?;
        return Ok(document.analysis);
    }
    serde_json::from_value(value).map_err(|error| json_error(&error))
}

/// Writes a two-dimensional analysis workflow result to compact versioned JSON.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn write_analysis2d_json(analysis: &SpectrumAnalysis2D) -> Result<String> {
    let document = Analysis2DDocumentRef {
        format: ANALYSIS_2D_JSON_FORMAT,
        version: ANALYSIS_JSON_VERSION,
        analysis,
    };
    serde_json::to_string(&document).map_err(|error| json_error(&error))
}

#[derive(Debug, Deserialize)]
struct Analysis1DDocument {
    analysis: SpectrumAnalysis1D,
}

#[derive(Debug, Serialize)]
struct Analysis1DDocumentRef<'a> {
    format: &'static str,
    version: u32,
    analysis: &'a SpectrumAnalysis1D,
}

#[derive(Debug, Deserialize)]
struct Analysis2DDocument {
    analysis: SpectrumAnalysis2D,
}

#[derive(Debug, Serialize)]
struct Analysis2DDocumentRef<'a> {
    format: &'static str,
    version: u32,
    analysis: &'a SpectrumAnalysis2D,
}

#[derive(Debug, Deserialize)]
struct AnalysisDocumentHeader {
    format: String,
    version: u32,
}

fn is_versioned_analysis_document(value: &Value) -> bool {
    value.as_object().is_some_and(|object| {
        object.contains_key("format")
            || object.contains_key("version")
            || object.contains_key("analysis")
    })
}

fn validate_analysis_document_header(value: &Value, expected_format: &'static str) -> Result<()> {
    let header: AnalysisDocumentHeader =
        serde_json::from_value(value.clone()).map_err(|error| json_error(&error))?;
    if header.format != expected_format {
        return Err(RSpinError::Parse {
            format: "JSON",
            message: format!(
                "expected analysis format '{expected_format}' but found '{}'",
                header.format
            ),
        });
    }
    if header.version != ANALYSIS_JSON_VERSION {
        return Err(RSpinError::Unsupported {
            feature: "analysis JSON version",
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
