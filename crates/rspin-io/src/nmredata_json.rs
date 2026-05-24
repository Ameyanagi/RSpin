//! JSON serialization for `NMReDATA` records.

use rspin_core::{RSpinError, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{NmreDataRecord, SpectrumReader, SpectrumWriter};

/// Current version of `RSpin`'s `NMReDATA` JSON envelopes.
pub const NMREDATA_JSON_VERSION: u32 = 1;

/// Format identifier for one `NMReDATA` record JSON envelope.
pub const NMREDATA_RECORD_JSON_FORMAT: &str = "rspin.nmredata_record";

/// Format identifier for multi-record `NMReDATA` JSON envelopes.
pub const NMREDATA_RECORDS_JSON_FORMAT: &str = "rspin.nmredata_records";

/// JSON reader/writer for one `NMReDATA` record.
#[derive(Clone, Copy, Debug, Default)]
pub struct JsonNmreDataRecord;

impl SpectrumReader for JsonNmreDataRecord {
    type Output = NmreDataRecord;

    fn read_str(&self, input: &str) -> Result<Self::Output> {
        read_nmredata_record_json(input)
    }
}

impl SpectrumWriter<NmreDataRecord> for JsonNmreDataRecord {
    fn write_string(&self, record: &NmreDataRecord) -> Result<String> {
        write_nmredata_record_json(record)
    }
}

/// JSON reader/writer for multi-record `NMReDATA` payloads.
#[derive(Clone, Copy, Debug, Default)]
pub struct JsonNmreDataRecords;

impl SpectrumReader for JsonNmreDataRecords {
    type Output = Vec<NmreDataRecord>;

    fn read_str(&self, input: &str) -> Result<Self::Output> {
        read_nmredata_records_json(input)
    }
}

impl SpectrumWriter<[NmreDataRecord]> for JsonNmreDataRecords {
    fn write_string(&self, records: &[NmreDataRecord]) -> Result<String> {
        write_nmredata_records_json(records)
    }
}

/// Reads one `NMReDATA` record from JSON.
///
/// The reader accepts the current versioned envelope and legacy raw
/// `NmreDataRecord` JSON payloads.
///
/// # Errors
///
/// Returns an error when JSON deserialization fails, the envelope format is not
/// a single `NMReDATA` record, or the envelope version is unsupported.
pub fn read_nmredata_record_json(input: &str) -> Result<NmreDataRecord> {
    let value = json_value(input)?;
    if is_record_document(&value) {
        validate_document_header(&value, NMREDATA_RECORD_JSON_FORMAT)?;
        let document: NmreDataRecordDocument =
            serde_json::from_value(value).map_err(|error| json_error(&error))?;
        return Ok(document.record);
    }
    serde_json::from_value(value).map_err(|error| json_error(&error))
}

/// Writes one `NMReDATA` record to compact versioned JSON.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn write_nmredata_record_json(record: &NmreDataRecord) -> Result<String> {
    let document = NmreDataRecordDocumentRef {
        format: NMREDATA_RECORD_JSON_FORMAT,
        version: NMREDATA_JSON_VERSION,
        record,
    };
    serde_json::to_string(&document).map_err(|error| json_error(&error))
}

/// Reads multiple `NMReDATA` records from JSON.
///
/// The reader accepts the current versioned envelope and legacy raw
/// `Vec<NmreDataRecord>` JSON payloads.
///
/// # Errors
///
/// Returns an error when JSON deserialization fails, the envelope format is not
/// a multi-record `NMReDATA` payload, or the envelope version is unsupported.
pub fn read_nmredata_records_json(input: &str) -> Result<Vec<NmreDataRecord>> {
    let value = json_value(input)?;
    if is_records_document(&value) {
        validate_document_header(&value, NMREDATA_RECORDS_JSON_FORMAT)?;
        let document: NmreDataRecordsDocument =
            serde_json::from_value(value).map_err(|error| json_error(&error))?;
        return Ok(document.records);
    }
    serde_json::from_value(value).map_err(|error| json_error(&error))
}

/// Writes multiple `NMReDATA` records to compact versioned JSON.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn write_nmredata_records_json(records: &[NmreDataRecord]) -> Result<String> {
    let document = NmreDataRecordsDocumentRef {
        format: NMREDATA_RECORDS_JSON_FORMAT,
        version: NMREDATA_JSON_VERSION,
        records,
    };
    serde_json::to_string(&document).map_err(|error| json_error(&error))
}

#[derive(Debug, Deserialize)]
struct NmreDataRecordDocument {
    record: NmreDataRecord,
}

#[derive(Debug, Serialize)]
struct NmreDataRecordDocumentRef<'a> {
    format: &'static str,
    version: u32,
    record: &'a NmreDataRecord,
}

#[derive(Debug, Deserialize)]
struct NmreDataRecordsDocument {
    records: Vec<NmreDataRecord>,
}

#[derive(Debug, Serialize)]
struct NmreDataRecordsDocumentRef<'a> {
    format: &'static str,
    version: u32,
    records: &'a [NmreDataRecord],
}

#[derive(Debug, Deserialize)]
struct NmreDataDocumentHeader {
    format: String,
    version: u32,
}

fn is_record_document(value: &Value) -> bool {
    value
        .as_object()
        .is_some_and(|object| object.contains_key("format") || object.contains_key("record"))
}

fn is_records_document(value: &Value) -> bool {
    value
        .as_object()
        .is_some_and(|object| object.contains_key("format") || object.contains_key("records"))
}

fn validate_document_header(value: &Value, expected_format: &'static str) -> Result<()> {
    let header: NmreDataDocumentHeader =
        serde_json::from_value(value.clone()).map_err(|error| json_error(&error))?;
    if header.format != expected_format {
        return Err(RSpinError::Parse {
            format: "JSON",
            message: format!(
                "expected NMReDATA format '{expected_format}' but found '{}'",
                header.format
            ),
        });
    }
    if header.version != NMREDATA_JSON_VERSION {
        return Err(RSpinError::Unsupported {
            feature: "NMReDATA JSON version",
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
