//! JSON serialization for exact simulation payloads.

use rspin_core::{RSpinError, Result};
use rspin_simulation::{
    ExactSpectrum2DOptions, ExactSpectrumDecomposition1D, ExactSpectrumDecomposition2D,
    ExactSpectrumOptions, ExactSpinOptions, ExactTransition, SpinHalfSystem,
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Map, Value};

use crate::{SpectrumReader, SpectrumWriter};

/// Current version of `RSpin`'s exact simulation JSON envelopes.
pub const SIMULATION_JSON_VERSION: u32 = 1;

/// Format identifier for exact spin-1/2 system JSON.
pub const SPIN_HALF_SYSTEM_JSON_FORMAT: &str = "rspin.spin_half_system";

/// Format identifier for exact transition option JSON.
pub const EXACT_SPIN_OPTIONS_JSON_FORMAT: &str = "rspin.exact_spin_options";

/// Format identifier for exact one-dimensional rendering option JSON.
pub const EXACT_SPECTRUM_1D_OPTIONS_JSON_FORMAT: &str = "rspin.exact_spectrum_1d_options";

/// Format identifier for exact two-dimensional rendering option JSON.
pub const EXACT_SPECTRUM_2D_OPTIONS_JSON_FORMAT: &str = "rspin.exact_spectrum_2d_options";

/// Format identifier for exact transition JSON.
pub const EXACT_TRANSITIONS_JSON_FORMAT: &str = "rspin.exact_transitions";

/// Format identifier for exact one-dimensional decomposition JSON.
pub const EXACT_DECOMPOSITION_1D_JSON_FORMAT: &str = "rspin.exact_decomposition_1d";

/// Format identifier for exact two-dimensional decomposition JSON.
pub const EXACT_DECOMPOSITION_2D_JSON_FORMAT: &str = "rspin.exact_decomposition_2d";

/// JSON reader/writer for exact spin-1/2 systems.
#[derive(Clone, Copy, Debug, Default)]
pub struct JsonSpinHalfSystem;

impl SpectrumReader for JsonSpinHalfSystem {
    type Output = SpinHalfSystem;

    fn read_str(&self, input: &str) -> Result<Self::Output> {
        read_spin_half_system_json(input)
    }
}

impl SpectrumWriter<SpinHalfSystem> for JsonSpinHalfSystem {
    fn write_string(&self, system: &SpinHalfSystem) -> Result<String> {
        write_spin_half_system_json(system)
    }
}

/// JSON reader/writer for exact transition simulation options.
#[derive(Clone, Copy, Debug, Default)]
pub struct JsonExactSpinOptions;

impl SpectrumReader for JsonExactSpinOptions {
    type Output = ExactSpinOptions;

    fn read_str(&self, input: &str) -> Result<Self::Output> {
        read_exact_spin_options_json(input)
    }
}

impl SpectrumWriter<ExactSpinOptions> for JsonExactSpinOptions {
    fn write_string(&self, options: &ExactSpinOptions) -> Result<String> {
        write_exact_spin_options_json(options)
    }
}

/// JSON reader/writer for exact one-dimensional rendering options.
#[derive(Clone, Copy, Debug, Default)]
pub struct JsonExactSpectrumOptions;

impl SpectrumReader for JsonExactSpectrumOptions {
    type Output = ExactSpectrumOptions;

    fn read_str(&self, input: &str) -> Result<Self::Output> {
        read_exact_spectrum_options_json(input)
    }
}

impl SpectrumWriter<ExactSpectrumOptions> for JsonExactSpectrumOptions {
    fn write_string(&self, options: &ExactSpectrumOptions) -> Result<String> {
        write_exact_spectrum_options_json(options)
    }
}

/// JSON reader/writer for exact two-dimensional rendering options.
#[derive(Clone, Copy, Debug, Default)]
pub struct JsonExactSpectrum2DOptions;

impl SpectrumReader for JsonExactSpectrum2DOptions {
    type Output = ExactSpectrum2DOptions;

    fn read_str(&self, input: &str) -> Result<Self::Output> {
        read_exact_spectrum_2d_options_json(input)
    }
}

impl SpectrumWriter<ExactSpectrum2DOptions> for JsonExactSpectrum2DOptions {
    fn write_string(&self, options: &ExactSpectrum2DOptions) -> Result<String> {
        write_exact_spectrum_2d_options_json(options)
    }
}

/// JSON reader/writer for exact transition lists.
#[derive(Clone, Copy, Debug, Default)]
pub struct JsonExactTransitions;

impl SpectrumReader for JsonExactTransitions {
    type Output = Vec<ExactTransition>;

    fn read_str(&self, input: &str) -> Result<Self::Output> {
        read_exact_transitions_json(input)
    }
}

impl SpectrumWriter<[ExactTransition]> for JsonExactTransitions {
    fn write_string(&self, transitions: &[ExactTransition]) -> Result<String> {
        write_exact_transitions_json(transitions)
    }
}

/// JSON reader/writer for exact one-dimensional decompositions.
#[derive(Clone, Copy, Debug, Default)]
pub struct JsonExactDecomposition1D;

impl SpectrumReader for JsonExactDecomposition1D {
    type Output = ExactSpectrumDecomposition1D;

    fn read_str(&self, input: &str) -> Result<Self::Output> {
        read_exact_decomposition_1d_json(input)
    }
}

impl SpectrumWriter<ExactSpectrumDecomposition1D> for JsonExactDecomposition1D {
    fn write_string(&self, decomposition: &ExactSpectrumDecomposition1D) -> Result<String> {
        write_exact_decomposition_1d_json(decomposition)
    }
}

/// JSON reader/writer for exact two-dimensional decompositions.
#[derive(Clone, Copy, Debug, Default)]
pub struct JsonExactDecomposition2D;

impl SpectrumReader for JsonExactDecomposition2D {
    type Output = ExactSpectrumDecomposition2D;

    fn read_str(&self, input: &str) -> Result<Self::Output> {
        read_exact_decomposition_2d_json(input)
    }
}

impl SpectrumWriter<ExactSpectrumDecomposition2D> for JsonExactDecomposition2D {
    fn write_string(&self, decomposition: &ExactSpectrumDecomposition2D) -> Result<String> {
        write_exact_decomposition_2d_json(decomposition)
    }
}

/// Reads an exact spin-1/2 system from JSON.
///
/// The reader accepts the current versioned envelope and legacy raw
/// `SpinHalfSystem` JSON payloads.
///
/// # Errors
///
/// Returns an error when JSON deserialization fails, the envelope format is not
/// a spin system, or the envelope version is unsupported.
pub fn read_spin_half_system_json(input: &str) -> Result<SpinHalfSystem> {
    read_document_or_raw(input, SPIN_HALF_SYSTEM_JSON_FORMAT, "system")
}

/// Writes an exact spin-1/2 system to compact versioned JSON.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn write_spin_half_system_json(system: &SpinHalfSystem) -> Result<String> {
    write_document(SPIN_HALF_SYSTEM_JSON_FORMAT, "system", system)
}

/// Reads exact transition simulation options from JSON.
///
/// The reader accepts the current versioned envelope and legacy raw
/// `ExactSpinOptions` JSON payloads.
///
/// # Errors
///
/// Returns an error when JSON deserialization fails, the envelope format is not
/// exact transition options, or the envelope version is unsupported.
pub fn read_exact_spin_options_json(input: &str) -> Result<ExactSpinOptions> {
    read_document_or_raw(input, EXACT_SPIN_OPTIONS_JSON_FORMAT, "options")
}

/// Writes exact transition simulation options to compact versioned JSON.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn write_exact_spin_options_json(options: &ExactSpinOptions) -> Result<String> {
    write_document(EXACT_SPIN_OPTIONS_JSON_FORMAT, "options", options)
}

/// Reads exact one-dimensional rendering options from JSON.
///
/// The reader accepts the current versioned envelope and legacy raw
/// `ExactSpectrumOptions` JSON payloads.
///
/// # Errors
///
/// Returns an error when JSON deserialization fails, the envelope format is not
/// exact one-dimensional rendering options, or the envelope version is
/// unsupported.
pub fn read_exact_spectrum_options_json(input: &str) -> Result<ExactSpectrumOptions> {
    read_document_or_raw(input, EXACT_SPECTRUM_1D_OPTIONS_JSON_FORMAT, "options")
}

/// Writes exact one-dimensional rendering options to compact versioned JSON.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn write_exact_spectrum_options_json(options: &ExactSpectrumOptions) -> Result<String> {
    write_document(EXACT_SPECTRUM_1D_OPTIONS_JSON_FORMAT, "options", options)
}

/// Reads exact two-dimensional rendering options from JSON.
///
/// The reader accepts the current versioned envelope and legacy raw
/// `ExactSpectrum2DOptions` JSON payloads.
///
/// # Errors
///
/// Returns an error when JSON deserialization fails, the envelope format is not
/// exact two-dimensional rendering options, or the envelope version is
/// unsupported.
pub fn read_exact_spectrum_2d_options_json(input: &str) -> Result<ExactSpectrum2DOptions> {
    read_document_or_raw(input, EXACT_SPECTRUM_2D_OPTIONS_JSON_FORMAT, "options")
}

/// Writes exact two-dimensional rendering options to compact versioned JSON.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn write_exact_spectrum_2d_options_json(options: &ExactSpectrum2DOptions) -> Result<String> {
    write_document(EXACT_SPECTRUM_2D_OPTIONS_JSON_FORMAT, "options", options)
}

/// Reads exact transition lines from JSON.
///
/// The reader accepts the current versioned envelope and legacy raw transition
/// array JSON payloads.
///
/// # Errors
///
/// Returns an error when JSON deserialization fails, the envelope format is not
/// exact transitions, or the envelope version is unsupported.
pub fn read_exact_transitions_json(input: &str) -> Result<Vec<ExactTransition>> {
    read_document_or_raw(input, EXACT_TRANSITIONS_JSON_FORMAT, "transitions")
}

/// Writes exact transition lines to compact versioned JSON.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn write_exact_transitions_json(transitions: &[ExactTransition]) -> Result<String> {
    write_document(EXACT_TRANSITIONS_JSON_FORMAT, "transitions", transitions)
}

/// Reads an exact one-dimensional decomposition from JSON.
///
/// The reader accepts the current versioned envelope and legacy raw
/// `ExactSpectrumDecomposition1D` JSON payloads.
///
/// # Errors
///
/// Returns an error when JSON deserialization fails, the envelope format is not
/// an exact one-dimensional decomposition, or the envelope version is
/// unsupported.
pub fn read_exact_decomposition_1d_json(input: &str) -> Result<ExactSpectrumDecomposition1D> {
    read_document_or_raw(input, EXACT_DECOMPOSITION_1D_JSON_FORMAT, "decomposition")
}

/// Writes an exact one-dimensional decomposition to compact versioned JSON.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn write_exact_decomposition_1d_json(
    decomposition: &ExactSpectrumDecomposition1D,
) -> Result<String> {
    write_document(
        EXACT_DECOMPOSITION_1D_JSON_FORMAT,
        "decomposition",
        decomposition,
    )
}

/// Reads an exact two-dimensional decomposition from JSON.
///
/// The reader accepts the current versioned envelope and legacy raw
/// `ExactSpectrumDecomposition2D` JSON payloads.
///
/// # Errors
///
/// Returns an error when JSON deserialization fails, the envelope format is not
/// an exact two-dimensional decomposition, or the envelope version is
/// unsupported.
pub fn read_exact_decomposition_2d_json(input: &str) -> Result<ExactSpectrumDecomposition2D> {
    read_document_or_raw(input, EXACT_DECOMPOSITION_2D_JSON_FORMAT, "decomposition")
}

/// Writes an exact two-dimensional decomposition to compact versioned JSON.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn write_exact_decomposition_2d_json(
    decomposition: &ExactSpectrumDecomposition2D,
) -> Result<String> {
    write_document(
        EXACT_DECOMPOSITION_2D_JSON_FORMAT,
        "decomposition",
        decomposition,
    )
}

#[derive(Debug, serde::Deserialize)]
struct SimulationDocumentHeader {
    format: String,
    version: u32,
}

fn read_document_or_raw<T>(
    input: &str,
    expected_format: &'static str,
    payload_key: &'static str,
) -> Result<T>
where
    T: DeserializeOwned,
{
    let value = json_value(input)?;
    if is_versioned_document(&value, payload_key) {
        validate_document_header(&value, expected_format)?;
        let payload = document_payload(value, payload_key)?;
        return serde_json::from_value(payload).map_err(|error| json_error(&error));
    }
    serde_json::from_value(value).map_err(|error| json_error(&error))
}

fn write_document<T>(
    expected_format: &'static str,
    payload_key: &'static str,
    payload: &T,
) -> Result<String>
where
    T: Serialize + ?Sized,
{
    let mut document = Map::new();
    document.insert(
        "format".to_owned(),
        Value::String(expected_format.to_owned()),
    );
    document.insert(
        "version".to_owned(),
        Value::Number(serde_json::Number::from(u64::from(SIMULATION_JSON_VERSION))),
    );
    document.insert(
        payload_key.to_owned(),
        serde_json::to_value(payload).map_err(|error| json_error(&error))?,
    );
    serde_json::to_string(&Value::Object(document)).map_err(|error| json_error(&error))
}

fn is_versioned_document(value: &Value, payload_key: &'static str) -> bool {
    value.as_object().is_some_and(|object| {
        object.contains_key("format")
            || object.contains_key("version")
            || object.contains_key(payload_key)
    })
}

fn validate_document_header(value: &Value, expected_format: &'static str) -> Result<()> {
    let header: SimulationDocumentHeader =
        serde_json::from_value(value.clone()).map_err(|error| json_error(&error))?;
    if header.format != expected_format {
        return Err(RSpinError::Parse {
            format: "JSON",
            message: format!(
                "expected simulation format '{expected_format}' but found '{}'",
                header.format
            ),
        });
    }
    if header.version != SIMULATION_JSON_VERSION {
        return Err(RSpinError::Unsupported {
            feature: "simulation JSON version",
        });
    }
    Ok(())
}

fn document_payload(mut value: Value, payload_key: &'static str) -> Result<Value> {
    match value {
        Value::Object(ref mut object) => {
            object.remove(payload_key).ok_or_else(|| RSpinError::Parse {
                format: "JSON",
                message: format!("missing simulation payload field '{payload_key}'"),
            })
        }
        _ => Err(RSpinError::Parse {
            format: "JSON",
            message: "expected simulation JSON object".to_owned(),
        }),
    }
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
