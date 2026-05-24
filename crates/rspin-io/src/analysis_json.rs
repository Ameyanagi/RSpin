//! JSON serialization for analysis workflow results.

use rspin_analysis::{SpectrumAnalysis1D, SpectrumAnalysis2D};
use rspin_core::{RSpinError, Result};

use crate::{SpectrumReader, SpectrumWriter};

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
/// # Errors
///
/// Returns an error when JSON deserialization fails.
pub fn read_analysis1d_json(input: &str) -> Result<SpectrumAnalysis1D> {
    serde_json::from_str(input).map_err(|error| json_error(&error))
}

/// Writes a one-dimensional analysis workflow result to compact JSON.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn write_analysis1d_json(analysis: &SpectrumAnalysis1D) -> Result<String> {
    serde_json::to_string(analysis).map_err(|error| json_error(&error))
}

/// Reads a two-dimensional analysis workflow result from JSON.
///
/// # Errors
///
/// Returns an error when JSON deserialization fails.
pub fn read_analysis2d_json(input: &str) -> Result<SpectrumAnalysis2D> {
    serde_json::from_str(input).map_err(|error| json_error(&error))
}

/// Writes a two-dimensional analysis workflow result to compact JSON.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn write_analysis2d_json(analysis: &SpectrumAnalysis2D) -> Result<String> {
    serde_json::to_string(analysis).map_err(|error| json_error(&error))
}

fn json_error(error: &serde_json::Error) -> RSpinError {
    RSpinError::Parse {
        format: "JSON",
        message: error.to_string(),
    }
}

#[cfg(test)]
mod tests;
