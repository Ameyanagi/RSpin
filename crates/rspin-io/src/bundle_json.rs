//! JSON serialization for unified spectrum bundles.

use std::{fs, path::Path};

use rspin_core::{RSpinError, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{SpectrumBundle, SpectrumReader, SpectrumWriter};

/// Current version of `RSpin`'s spectrum bundle JSON envelope.
pub const SPECTRUM_BUNDLE_JSON_VERSION: u32 = 1;

/// Format identifier for spectrum bundle JSON.
pub const SPECTRUM_BUNDLE_JSON_FORMAT: &str = "rspin.spectrum_bundle";

/// JSON reader/writer for unified spectrum bundles.
#[derive(Clone, Copy, Debug, Default)]
pub struct JsonSpectrumBundle;

impl SpectrumReader for JsonSpectrumBundle {
    type Output = SpectrumBundle;

    fn read_str(&self, input: &str) -> Result<Self::Output> {
        read_spectrum_bundle_json(input)
    }
}

impl SpectrumWriter<SpectrumBundle> for JsonSpectrumBundle {
    fn write_string(&self, bundle: &SpectrumBundle) -> Result<String> {
        write_spectrum_bundle_json(bundle)
    }
}

/// Reads a spectrum bundle from JSON.
///
/// The reader accepts the current versioned envelope and legacy raw
/// `SpectrumBundle` JSON payloads.
///
/// # Errors
///
/// Returns an error when JSON deserialization fails, the envelope format is not
/// a spectrum bundle, or the envelope version is unsupported.
pub fn read_spectrum_bundle_json(input: &str) -> Result<SpectrumBundle> {
    let value = json_value(input)?;
    if is_bundle_document(&value) {
        validate_bundle_document_header(&value)?;
        let document: SpectrumBundleDocument =
            serde_json::from_value(value).map_err(|error| json_error(&error))?;
        return Ok(document.bundle);
    }
    serde_json::from_value(value).map_err(|error| json_error(&error))
}

/// Reads a spectrum bundle JSON file.
///
/// # Errors
///
/// Returns an error when the file cannot be read or the JSON payload is invalid.
pub fn read_spectrum_bundle_json_file(path: impl AsRef<Path>) -> Result<SpectrumBundle> {
    let path = path.as_ref();
    let input = fs::read_to_string(path).map_err(|error| RSpinError::Parse {
        format: "spectrum bundle JSON",
        message: format!("failed to read {}: {error}", path.display()),
    })?;
    read_spectrum_bundle_json(&input)
}

/// Writes a spectrum bundle to compact versioned JSON.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn write_spectrum_bundle_json(bundle: &SpectrumBundle) -> Result<String> {
    let document = SpectrumBundleDocumentRef {
        format: SPECTRUM_BUNDLE_JSON_FORMAT,
        version: SPECTRUM_BUNDLE_JSON_VERSION,
        bundle,
    };
    serde_json::to_string(&document).map_err(|error| json_error(&error))
}

#[derive(Debug, Deserialize)]
struct SpectrumBundleDocument {
    bundle: SpectrumBundle,
}

#[derive(Debug, Serialize)]
struct SpectrumBundleDocumentRef<'a> {
    format: &'static str,
    version: u32,
    bundle: &'a SpectrumBundle,
}

#[derive(Debug, Deserialize)]
struct SpectrumBundleDocumentHeader {
    format: String,
    version: u32,
}

fn is_bundle_document(value: &Value) -> bool {
    value.as_object().is_some_and(|object| {
        object.contains_key("format")
            || object.contains_key("version")
            || object.contains_key("bundle")
    })
}

fn validate_bundle_document_header(value: &Value) -> Result<()> {
    let header: SpectrumBundleDocumentHeader =
        serde_json::from_value(value.clone()).map_err(|error| json_error(&error))?;
    if header.format != SPECTRUM_BUNDLE_JSON_FORMAT {
        return Err(RSpinError::Parse {
            format: "JSON",
            message: format!(
                "expected spectrum bundle format '{SPECTRUM_BUNDLE_JSON_FORMAT}' but found '{}'",
                header.format
            ),
        });
    }
    if header.version != SPECTRUM_BUNDLE_JSON_VERSION {
        return Err(RSpinError::Unsupported {
            feature: "spectrum bundle JSON version",
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
