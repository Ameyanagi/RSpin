//! JSON serialization for assignment and J-coupling payloads.

use rspin_analysis::{AssignmentSet, JCouplingGraph};
use rspin_core::{RSpinError, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{SpectrumReader, SpectrumWriter};

/// Current version of `RSpin`'s assignment-related JSON envelopes.
pub const ASSIGNMENT_JSON_VERSION: u32 = 1;

/// Format identifier for assignment-set JSON.
pub const ASSIGNMENT_SET_JSON_FORMAT: &str = "rspin.assignment_set";

/// Format identifier for J-coupling graph JSON.
pub const J_COUPLING_GRAPH_JSON_FORMAT: &str = "rspin.j_coupling_graph";

/// JSON reader/writer for assignment sets.
#[derive(Clone, Copy, Debug, Default)]
pub struct JsonAssignmentSet;

impl SpectrumReader for JsonAssignmentSet {
    type Output = AssignmentSet;

    fn read_str(&self, input: &str) -> Result<Self::Output> {
        read_assignment_set_json(input)
    }
}

impl SpectrumWriter<AssignmentSet> for JsonAssignmentSet {
    fn write_string(&self, assignments: &AssignmentSet) -> Result<String> {
        write_assignment_set_json(assignments)
    }
}

/// JSON reader/writer for J-coupling graphs.
#[derive(Clone, Copy, Debug, Default)]
pub struct JsonJCouplingGraph;

impl SpectrumReader for JsonJCouplingGraph {
    type Output = JCouplingGraph;

    fn read_str(&self, input: &str) -> Result<Self::Output> {
        read_j_coupling_graph_json(input)
    }
}

impl SpectrumWriter<JCouplingGraph> for JsonJCouplingGraph {
    fn write_string(&self, graph: &JCouplingGraph) -> Result<String> {
        write_j_coupling_graph_json(graph)
    }
}

/// Reads an assignment set from JSON.
///
/// The reader accepts the current versioned envelope and legacy raw
/// `AssignmentSet` JSON payloads.
///
/// # Errors
///
/// Returns an error when JSON deserialization fails, the envelope format is not
/// an assignment set, or the envelope version is unsupported.
pub fn read_assignment_set_json(input: &str) -> Result<AssignmentSet> {
    let value = json_value(input)?;
    if is_assignment_set_document(&value) {
        validate_document_header(&value, ASSIGNMENT_SET_JSON_FORMAT)?;
        let document: AssignmentSetDocument =
            serde_json::from_value(value).map_err(|error| json_error(&error))?;
        return Ok(document.assignment_set);
    }
    serde_json::from_value(value).map_err(|error| json_error(&error))
}

/// Writes an assignment set to compact versioned JSON.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn write_assignment_set_json(assignments: &AssignmentSet) -> Result<String> {
    let document = AssignmentSetDocumentRef {
        format: ASSIGNMENT_SET_JSON_FORMAT,
        version: ASSIGNMENT_JSON_VERSION,
        assignment_set: assignments,
    };
    serde_json::to_string(&document).map_err(|error| json_error(&error))
}

/// Reads a J-coupling graph from JSON.
///
/// The reader accepts the current versioned envelope and legacy raw
/// `JCouplingGraph` JSON payloads.
///
/// # Errors
///
/// Returns an error when JSON deserialization fails, the envelope format is not
/// a J-coupling graph, or the envelope version is unsupported.
pub fn read_j_coupling_graph_json(input: &str) -> Result<JCouplingGraph> {
    let value = json_value(input)?;
    if is_j_coupling_graph_document(&value) {
        validate_document_header(&value, J_COUPLING_GRAPH_JSON_FORMAT)?;
        let document: JCouplingGraphDocument =
            serde_json::from_value(value).map_err(|error| json_error(&error))?;
        return Ok(document.graph);
    }
    serde_json::from_value(value).map_err(|error| json_error(&error))
}

/// Writes a J-coupling graph to compact versioned JSON.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn write_j_coupling_graph_json(graph: &JCouplingGraph) -> Result<String> {
    let document = JCouplingGraphDocumentRef {
        format: J_COUPLING_GRAPH_JSON_FORMAT,
        version: ASSIGNMENT_JSON_VERSION,
        graph,
    };
    serde_json::to_string(&document).map_err(|error| json_error(&error))
}

#[derive(Debug, Deserialize)]
struct AssignmentSetDocument {
    assignment_set: AssignmentSet,
}

#[derive(Debug, Serialize)]
struct AssignmentSetDocumentRef<'a> {
    format: &'static str,
    version: u32,
    assignment_set: &'a AssignmentSet,
}

#[derive(Debug, Deserialize)]
struct JCouplingGraphDocument {
    graph: JCouplingGraph,
}

#[derive(Debug, Serialize)]
struct JCouplingGraphDocumentRef<'a> {
    format: &'static str,
    version: u32,
    graph: &'a JCouplingGraph,
}

#[derive(Debug, Deserialize)]
struct AssignmentDocumentHeader {
    format: String,
    version: u32,
}

fn is_assignment_set_document(value: &Value) -> bool {
    value.as_object().is_some_and(|object| {
        object.contains_key("format") || object.contains_key("assignment_set")
    })
}

fn is_j_coupling_graph_document(value: &Value) -> bool {
    value
        .as_object()
        .is_some_and(|object| object.contains_key("format") || object.contains_key("graph"))
}

fn validate_document_header(value: &Value, expected_format: &'static str) -> Result<()> {
    let header: AssignmentDocumentHeader =
        serde_json::from_value(value.clone()).map_err(|error| json_error(&error))?;
    if header.format != expected_format {
        return Err(RSpinError::Parse {
            format: "JSON",
            message: format!(
                "expected assignment format '{expected_format}' but found '{}'",
                header.format
            ),
        });
    }
    if header.version != ASSIGNMENT_JSON_VERSION {
        return Err(RSpinError::Unsupported {
            feature: "assignment JSON version",
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
