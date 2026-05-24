//! Lightweight nmrML document metadata inspection.

use std::{fs, path::Path, str};

use quick_xml::{
    Reader,
    events::{BytesStart, Event},
};
use rspin_core::{RSpinError, Result};
use serde::{Deserialize, Serialize};

const FORMAT: &str = "nmrML";

/// Official nmrML schema and ontology repository.
pub const NMRML_SCHEMA_REPOSITORY: &str = "https://github.com/nmrML/nmrML";

/// Directory containing the official XSD files in the nmrML schema repository.
pub const NMRML_SCHEMA_DIRECTORY: &str = "xml-schemata";

/// One namespace/location pair from an XML schemaLocation attribute.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NmrMlSchemaLocation {
    /// XML namespace URI.
    pub namespace: String,
    /// Schema location URI or relative path.
    pub location: String,
}

/// Parsed nmrML document version.
///
/// The official nmrML repository describes versions as
/// `Major.Minor.Build`, while the current public schema still uses
/// release-candidate text such as `v1.0.rc1`. `RSpin` preserves the raw string
/// and exposes the numeric routing fields separately.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NmrMlVersion {
    /// Raw version string after trimming surrounding whitespace.
    pub raw: String,
    /// Version with a leading `v`/`V` stripped for comparison.
    pub normalized: String,
    /// Major schema version.
    pub major: u32,
    /// Minor schema version.
    pub minor: u32,
    /// Build or qualifier segment after `major.minor`, when present.
    pub build: Option<String>,
}

impl NmrMlVersion {
    /// Returns true when `RSpin`'s current nmrML readers support this version.
    #[must_use]
    pub fn is_supported_by_current_readers(&self) -> bool {
        self.major == 1 && self.minor == 0 && self.build.is_some()
    }
}

/// Root-level nmrML document metadata used for parser routing and compatibility checks.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NmrMlDocumentInfo {
    /// Raw document version attribute, preserving the spelling used by the file.
    pub version: String,
    /// Version with a leading `v` stripped for comparison.
    pub normalized_version: String,
    /// Structured version information, when the version follows nmrML's numeric family.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parsed_version: Option<NmrMlVersion>,
    /// Default XML namespace declared by the root element.
    pub namespace: Option<String>,
    /// Raw `xsi:schemaLocation` value, when declared.
    pub schema_location: Option<String>,
    /// Raw `xsi:noNamespaceSchemaLocation` value, when declared.
    pub no_namespace_schema_location: Option<String>,
    /// Parsed namespace/location pairs from `schema_location`.
    pub schema_locations: Vec<NmrMlSchemaLocation>,
}

impl NmrMlDocumentInfo {
    /// Returns true when `RSpin`'s current nmrML readers support the document version.
    #[must_use]
    pub fn is_supported_by_current_readers(&self) -> bool {
        match self.parsed_version.as_ref() {
            Some(version) => version.is_supported_by_current_readers(),
            None => false,
        }
    }

    /// Validates that the document version is supported by `RSpin`'s current nmrML readers.
    ///
    /// # Errors
    ///
    /// Returns an unsupported-feature error for versions outside the `1.0.*`
    /// family currently handled by the spectrum readers.
    pub fn validate_supported_by_current_readers(&self) -> Result<()> {
        if self.is_supported_by_current_readers() {
            Ok(())
        } else {
            Err(RSpinError::Unsupported {
                feature: "nmrML document version",
            })
        }
    }
}

/// Reads root-level nmrML document metadata from a file.
///
/// # Errors
///
/// Returns an error when the file cannot be read, is not UTF-8, is malformed,
/// or does not contain an nmrML root element with a version.
pub fn read_nmrml_document_info_file(path: impl AsRef<Path>) -> Result<NmrMlDocumentInfo> {
    let path = path.as_ref();
    let input = fs::read_to_string(path).map_err(|error| RSpinError::Parse {
        format: FORMAT,
        message: format!("failed to read {}: {error}", path.display()),
    })?;
    read_nmrml_document_info_str(&input)
}

/// Reads root-level nmrML document metadata from UTF-8 bytes.
///
/// # Errors
///
/// Returns an error when the input is not UTF-8, is malformed, or does not
/// contain an nmrML root element with a version.
pub fn read_nmrml_document_info_bytes(bytes: &[u8]) -> Result<NmrMlDocumentInfo> {
    let input = str::from_utf8(bytes).map_err(|error| RSpinError::Parse {
        format: FORMAT,
        message: format!("input is not valid UTF-8: {error}"),
    })?;
    read_nmrml_document_info_str(input)
}

/// Reads root-level nmrML document metadata from XML text.
///
/// # Errors
///
/// Returns an error when the payload is malformed or does not contain an nmrML
/// root element with a version.
pub fn read_nmrml_document_info_str(input: &str) -> Result<NmrMlDocumentInfo> {
    let mut reader = Reader::from_str(input);
    let mut buffer = Vec::new();

    loop {
        match reader
            .read_event_into(&mut buffer)
            .map_err(|error| xml_error(&error))?
        {
            Event::Start(start) | Event::Empty(start)
                if local_name(start.name().as_ref()) == b"nmrML" =>
            {
                return info_from_root(&start);
            }
            Event::Start(start) | Event::Empty(start) => {
                return Err(RSpinError::Parse {
                    format: FORMAT,
                    message: format!(
                        "expected nmrML root element, found {}",
                        display_name(start.name().as_ref())
                    ),
                });
            }
            Event::Eof => {
                return Err(RSpinError::Parse {
                    format: FORMAT,
                    message: "missing nmrML root element".to_owned(),
                });
            }
            _ => {}
        }
        buffer.clear();
    }
}

fn info_from_root(start: &BytesStart<'_>) -> Result<NmrMlDocumentInfo> {
    let version = attr_value(start, b"version")?.ok_or_else(|| RSpinError::Parse {
        format: FORMAT,
        message: "missing required nmrML version".to_owned(),
    })?;
    let namespace = attr_value_exact(start, b"xmlns")?;
    let schema_location = attr_value(start, b"schemaLocation")?;
    let no_namespace_schema_location = attr_value(start, b"noNamespaceSchemaLocation")?;
    let parsed_version = parse_nmrml_version(&version).ok();
    let normalized_version = match parsed_version.as_ref() {
        Some(version) => version.normalized.clone(),
        None => normalize_version(&version),
    };
    let schema_locations = if let Some(schema_location) = schema_location.as_deref() {
        parse_schema_locations(schema_location)
    } else {
        Vec::new()
    };

    Ok(NmrMlDocumentInfo {
        version,
        normalized_version,
        parsed_version,
        namespace,
        schema_location,
        no_namespace_schema_location,
        schema_locations,
    })
}

/// Parses an nmrML document version into routing-friendly fields.
///
/// # Errors
///
/// Returns a parse error when the version is empty or does not contain numeric
/// major and minor components.
pub fn parse_nmrml_version(version: &str) -> Result<NmrMlVersion> {
    let raw = version.trim();
    if raw.is_empty() {
        return Err(RSpinError::Parse {
            format: FORMAT,
            message: "empty nmrML version".to_owned(),
        });
    }

    let normalized = normalize_version(raw);
    let mut parts = normalized.split('.');
    let major_text = parts.next().ok_or_else(|| RSpinError::Parse {
        format: FORMAT,
        message: "missing nmrML major version".to_owned(),
    })?;
    let minor_text = parts.next().ok_or_else(|| RSpinError::Parse {
        format: FORMAT,
        message: "missing nmrML minor version".to_owned(),
    })?;
    let major = parse_version_number("major", major_text)?;
    let minor = parse_version_number("minor", minor_text)?;
    let build_parts: Vec<&str> = parts.collect();
    let build = if build_parts.is_empty() {
        None
    } else if build_parts.iter().any(|part| part.is_empty()) {
        return Err(RSpinError::Parse {
            format: FORMAT,
            message: "empty nmrML build version component".to_owned(),
        });
    } else {
        Some(build_parts.join("."))
    };

    Ok(NmrMlVersion {
        raw: raw.to_owned(),
        normalized,
        major,
        minor,
        build,
    })
}

pub(crate) fn validate_nmrml_reader_version(version: Option<&str>) -> Result<String> {
    let version = version.ok_or_else(|| RSpinError::Parse {
        format: FORMAT,
        message: "missing required nmrML version".to_owned(),
    })?;
    let parsed = parse_nmrml_version(version)?;
    if parsed.is_supported_by_current_readers() {
        Ok(parsed.raw)
    } else {
        Err(RSpinError::Unsupported {
            feature: "nmrML document version",
        })
    }
}

fn normalize_version(version: &str) -> String {
    let trimmed = version.trim();
    match trimmed.strip_prefix('v') {
        Some(version) => version.to_owned(),
        None => match trimmed.strip_prefix('V') {
            Some(version) => version.to_owned(),
            None => trimmed.to_owned(),
        },
    }
}

fn parse_version_number(field: &'static str, value: &str) -> Result<u32> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(RSpinError::Parse {
            format: FORMAT,
            message: format!("invalid nmrML {field} version component: {value}"),
        });
    }
    value.parse::<u32>().map_err(|error| RSpinError::Parse {
        format: FORMAT,
        message: format!("invalid nmrML {field} version component: {error}"),
    })
}

fn parse_schema_locations(value: &str) -> Vec<NmrMlSchemaLocation> {
    let parts: Vec<&str> = value.split_whitespace().collect();
    parts
        .chunks_exact(2)
        .map(|chunk| NmrMlSchemaLocation {
            namespace: chunk[0].to_owned(),
            location: chunk[1].to_owned(),
        })
        .collect()
}

fn attr_value(start: &BytesStart<'_>, name: &[u8]) -> Result<Option<String>> {
    for attribute in start.attributes().with_checks(false) {
        let attribute = attribute.map_err(|error| RSpinError::Parse {
            format: FORMAT,
            message: format!("invalid XML attribute: {error}"),
        })?;
        if local_name(attribute.key.as_ref()) == name {
            let value =
                str::from_utf8(attribute.value.as_ref()).map_err(|error| RSpinError::Parse {
                    format: FORMAT,
                    message: format!("attribute is not valid UTF-8: {error}"),
                })?;
            return Ok(Some(xml_unescape(value)));
        }
    }
    Ok(None)
}

fn attr_value_exact(start: &BytesStart<'_>, name: &[u8]) -> Result<Option<String>> {
    for attribute in start.attributes().with_checks(false) {
        let attribute = attribute.map_err(|error| RSpinError::Parse {
            format: FORMAT,
            message: format!("invalid XML attribute: {error}"),
        })?;
        if attribute.key.as_ref() == name {
            let value =
                str::from_utf8(attribute.value.as_ref()).map_err(|error| RSpinError::Parse {
                    format: FORMAT,
                    message: format!("attribute is not valid UTF-8: {error}"),
                })?;
            return Ok(Some(xml_unescape(value)));
        }
    }
    Ok(None)
}

fn local_name(name: &[u8]) -> &[u8] {
    match name.iter().rposition(|byte| *byte == b':') {
        Some(index) => &name[index + 1..],
        None => name,
    }
}

fn display_name(name: &[u8]) -> String {
    String::from_utf8_lossy(name).into_owned()
}

fn xml_unescape(value: &str) -> String {
    value
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

fn xml_error(error: &quick_xml::Error) -> RSpinError {
    RSpinError::Parse {
        format: FORMAT,
        message: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_document_version_namespace_and_schema_location() -> anyhow::Result<()> {
        let info = read_nmrml_document_info_str(
            r#"<?xml version="1.0"?>
            <nmrML
                version="v1.0.rc1"
                xmlns="http://nmrml.org/schema"
                xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
                xsi:schemaLocation="http://nmrml.org/schema nmrML.xsd">
            </nmrML>"#,
        )?;

        assert_eq!(info.version, "v1.0.rc1");
        assert_eq!(info.normalized_version, "1.0.rc1");
        let Some(parsed_version) = info.parsed_version.as_ref() else {
            panic!("document version should parse");
        };
        assert_eq!(parsed_version.major, 1);
        assert_eq!(parsed_version.minor, 0);
        assert_eq!(parsed_version.build.as_deref(), Some("rc1"));
        assert_eq!(info.namespace.as_deref(), Some("http://nmrml.org/schema"));
        assert_eq!(
            info.schema_location.as_deref(),
            Some("http://nmrml.org/schema nmrML.xsd")
        );
        assert_eq!(
            info.schema_locations,
            vec![NmrMlSchemaLocation {
                namespace: "http://nmrml.org/schema".to_owned(),
                location: "nmrML.xsd".to_owned(),
            }]
        );
        assert!(info.is_supported_by_current_readers());
        info.validate_supported_by_current_readers()?;
        Ok(())
    }

    #[test]
    fn preserves_future_versions_for_routing() -> anyhow::Result<()> {
        let info = read_nmrml_document_info_str(r#"<nmrML version="2.1.0"/>"#)?;

        assert_eq!(info.version, "2.1.0");
        assert_eq!(info.normalized_version, "2.1.0");
        let Some(parsed_version) = info.parsed_version.as_ref() else {
            panic!("future numeric version should parse");
        };
        assert_eq!(parsed_version.major, 2);
        assert_eq!(parsed_version.minor, 1);
        assert_eq!(parsed_version.build.as_deref(), Some("0"));
        assert!(!info.is_supported_by_current_readers());
        assert!(matches!(
            info.validate_supported_by_current_readers(),
            Err(RSpinError::Unsupported { .. })
        ));
        Ok(())
    }

    #[test]
    fn reads_no_namespace_schema_location_and_bytes() -> anyhow::Result<()> {
        let info = read_nmrml_document_info_bytes(
            br#"<nmrML version="1.0.0" xsi:noNamespaceSchemaLocation="nmrML.xsd"/>"#,
        )?;

        assert_eq!(info.version, "1.0.0");
        assert_eq!(
            info.no_namespace_schema_location.as_deref(),
            Some("nmrML.xsd")
        );
        assert!(info.schema_locations.is_empty());
        Ok(())
    }

    #[test]
    fn rejects_missing_root_or_version() {
        let error = read_nmrml_document_info_str("<notNmrMl/>")
            .expect_err("wrong root element should fail");
        assert!(matches!(error, RSpinError::Parse { .. }));

        let error =
            read_nmrml_document_info_str("<nmrML/>").expect_err("missing version should fail");
        assert!(matches!(error, RSpinError::Parse { .. }));
    }

    #[test]
    fn parses_schema_versions_for_routing() -> anyhow::Result<()> {
        let release = parse_nmrml_version(" V1.0.0 ")?;
        assert_eq!(release.raw, "V1.0.0");
        assert_eq!(release.normalized, "1.0.0");
        assert_eq!(release.major, 1);
        assert_eq!(release.minor, 0);
        assert_eq!(release.build.as_deref(), Some("0"));
        assert!(release.is_supported_by_current_readers());

        let draft = parse_nmrml_version("v1.0.rc1")?;
        assert_eq!(draft.build.as_deref(), Some("rc1"));
        assert!(draft.is_supported_by_current_readers());

        let family = parse_nmrml_version("1.0")?;
        assert_eq!(family.build, None);
        assert!(!family.is_supported_by_current_readers());

        Ok(())
    }

    #[test]
    fn rejects_malformed_schema_versions() {
        let error =
            parse_nmrml_version("version-two").expect_err("non-numeric major version should fail");
        assert!(matches!(error, RSpinError::Parse { .. }));

        let error = parse_nmrml_version("1..0").expect_err("empty component should fail");
        assert!(matches!(error, RSpinError::Parse { .. }));
    }
}
