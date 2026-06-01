//! Source-format names used by the unified bundle loader.

use std::{fmt, str::FromStr};

use rspin_core::{RSpinError, Result};
use serde::{Deserialize, Serialize};

/// Known spectrum source formats emitted by the bundle loader.
///
/// `LoadedSource` stores source formats as strings so serialized bundles remain
/// forward-compatible with future readers. Use this enum when callers want the
/// built-in format names without string literals.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LoadedSourceFormat {
    /// `RSpin` JSON spectrum payload.
    Json,
    /// nmrML XML payload.
    NmrMl,
    /// JCAMP-DX text payload.
    JcampDx,
    /// `RSpin` CSV payload.
    Csv,
    /// JEOL Delta `.jdf` file.
    JeolJdf,
    /// Bruker processed spectrum dataset.
    BrukerProcessed,
    /// Bruker raw one-dimensional FID dataset.
    BrukerFid,
    /// Bruker raw two-dimensional SER dataset.
    BrukerSer,
    /// Agilent/Varian processed spectrum dataset.
    AgilentProcessed,
    /// Agilent/Varian raw FID dataset.
    AgilentFid,
}

/// Raw/processed classification for a loaded source format.
///
/// This is intentionally coarse: open exchange formats such as JCAMP-DX,
/// nmrML, JSON, and CSV may contain processed or raw-like data depending on
/// their payload, so they are classified as `Other`. Vendor formats with
/// clear acquisition or processing directory semantics are classified as
/// `Raw` or `Processed`.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoadedSourceDataKind {
    /// Vendor raw acquisition data, such as FID or SER datasets.
    Raw,
    /// Vendor processed data, such as processed Bruker or Agilent/Varian spectra.
    Processed,
    /// Open exchange data or unknown/custom formats without a raw/processed guarantee.
    Other,
}

const LOADED_SOURCE_FORMATS: &[LoadedSourceFormat] = &[
    LoadedSourceFormat::Json,
    LoadedSourceFormat::NmrMl,
    LoadedSourceFormat::JcampDx,
    LoadedSourceFormat::Csv,
    LoadedSourceFormat::JeolJdf,
    LoadedSourceFormat::BrukerProcessed,
    LoadedSourceFormat::BrukerFid,
    LoadedSourceFormat::BrukerSer,
    LoadedSourceFormat::AgilentProcessed,
    LoadedSourceFormat::AgilentFid,
];

const JSON_EXTENSIONS: &[&str] = &["json"];
const NMRML_EXTENSIONS: &[&str] = &["nmrml", "xml"];
const JCAMP_DX_EXTENSIONS: &[&str] = &["jdx", "dx", "jcamp"];
const CSV_EXTENSIONS: &[&str] = &["csv"];
const JEOL_JDF_EXTENSIONS: &[&str] = &["jdf"];
const NO_EXTENSIONS: &[&str] = &[];

const NO_PATH_MARKERS: &[&str] = &[];
const BRUKER_PROCESSED_MARKERS: &[&str] = &["1r", "2rr", "procs"];
const BRUKER_FID_MARKERS: &[&str] = &["fid", "acqus"];
const BRUKER_SER_MARKERS: &[&str] = &["ser", "acqus", "acqu2s"];
const AGILENT_PROCESSED_MARKERS: &[&str] = &["phasefile", "procpar"];
const AGILENT_FID_MARKERS: &[&str] = &["fid", "procpar"];

/// Discovery metadata for one built-in bundle source format.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LoadedSourceFormatInfo {
    /// Canonical source format name.
    pub name: &'static str,
    /// Canonical vendor family name for vendor-specific formats.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor: Option<&'static str>,
    /// Coarse raw/processed classification for the format.
    pub data_kind: LoadedSourceDataKind,
    /// Common standalone file extensions, without leading dots.
    pub extensions: &'static [&'static str],
    /// File names commonly used as directory or direct-file detection markers.
    pub path_markers: &'static [&'static str],
}

/// Vendor families emitted by vendor-specific bundle readers.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LoadedSourceVendor {
    /// Bruker raw or processed datasets.
    Bruker,
    /// JEOL Delta datasets.
    Jeol,
    /// Agilent/Varian raw or processed datasets.
    AgilentVarian,
}

const LOADED_SOURCE_VENDORS: &[LoadedSourceVendor] = &[
    LoadedSourceVendor::Bruker,
    LoadedSourceVendor::Jeol,
    LoadedSourceVendor::AgilentVarian,
];

/// Discovery metadata for one built-in bundle source vendor.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LoadedSourceVendorInfo {
    /// Canonical vendor family name.
    pub name: &'static str,
    /// Canonical source format names belonging to this vendor family.
    pub source_formats: Vec<&'static str>,
}

/// Discovery metadata for one bundle source data kind.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LoadedSourceDataKindInfo {
    /// Canonical source data-kind name.
    pub name: &'static str,
    /// Canonical built-in source format names classified with this data kind.
    pub source_formats: Vec<&'static str>,
}

impl LoadedSourceFormat {
    /// Returns all known built-in source formats in stable display order.
    ///
    /// `LoadedSource` stores source formats as strings for forward-compatible
    /// bundle JSON, so use this list for discovery and filters rather than as
    /// an exhaustiveness check on serialized bundle data.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        LOADED_SOURCE_FORMATS
    }

    /// Returns the canonical snake-case source format name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::NmrMl => "nmrml",
            Self::JcampDx => "jcamp_dx",
            Self::Csv => "csv",
            Self::JeolJdf => "jeol_jdf",
            Self::BrukerProcessed => "bruker_processed",
            Self::BrukerFid => "bruker_fid",
            Self::BrukerSer => "bruker_ser",
            Self::AgilentProcessed => "agilent_processed",
            Self::AgilentFid => "agilent_fid",
        }
    }

    /// Returns file extensions commonly accepted for standalone files.
    ///
    /// Extensions do not include a leading dot. Vendor directory formats usually
    /// return an empty slice because they are detected from required files
    /// rather than from a container extension.
    #[must_use]
    pub const fn file_extensions(self) -> &'static [&'static str] {
        match self {
            Self::Json => JSON_EXTENSIONS,
            Self::NmrMl => NMRML_EXTENSIONS,
            Self::JcampDx => JCAMP_DX_EXTENSIONS,
            Self::Csv => CSV_EXTENSIONS,
            Self::JeolJdf => JEOL_JDF_EXTENSIONS,
            Self::BrukerProcessed
            | Self::BrukerFid
            | Self::BrukerSer
            | Self::AgilentProcessed
            | Self::AgilentFid => NO_EXTENSIONS,
        }
    }

    /// Returns file names used as directory or direct-file detection markers.
    ///
    /// This is discovery metadata for file pickers and diagnostics, not a full
    /// validation schema. Use the reader itself for authoritative routing.
    #[must_use]
    pub const fn path_markers(self) -> &'static [&'static str] {
        match self {
            Self::Json | Self::NmrMl | Self::JcampDx | Self::Csv | Self::JeolJdf => NO_PATH_MARKERS,
            Self::BrukerProcessed => BRUKER_PROCESSED_MARKERS,
            Self::BrukerFid => BRUKER_FID_MARKERS,
            Self::BrukerSer => BRUKER_SER_MARKERS,
            Self::AgilentProcessed => AGILENT_PROCESSED_MARKERS,
            Self::AgilentFid => AGILENT_FID_MARKERS,
        }
    }

    /// Returns discovery metadata for this built-in source format.
    #[must_use]
    pub fn info(self) -> LoadedSourceFormatInfo {
        LoadedSourceFormatInfo {
            name: self.as_str(),
            vendor: self.vendor().map(LoadedSourceVendor::as_str),
            data_kind: self.data_kind(),
            extensions: self.file_extensions(),
            path_markers: self.path_markers(),
        }
    }

    /// Parses a source format name or common alias.
    ///
    /// # Errors
    ///
    /// Returns an unsupported-feature error when `input` is not a known source
    /// format name.
    pub fn parse(input: &str) -> Result<Self> {
        parse_loaded_source_format(input)
    }

    /// Returns the vendor family for vendor-specific source formats.
    #[must_use]
    pub const fn vendor(self) -> Option<LoadedSourceVendor> {
        match self {
            Self::JeolJdf => Some(LoadedSourceVendor::Jeol),
            Self::BrukerProcessed | Self::BrukerFid | Self::BrukerSer => {
                Some(LoadedSourceVendor::Bruker)
            }
            Self::AgilentProcessed | Self::AgilentFid => Some(LoadedSourceVendor::AgilentVarian),
            Self::Json | Self::NmrMl | Self::JcampDx | Self::Csv => None,
        }
    }

    /// Returns the coarse raw/processed classification for this source format.
    #[must_use]
    pub const fn data_kind(self) -> LoadedSourceDataKind {
        match self {
            Self::BrukerFid | Self::BrukerSer | Self::AgilentFid => LoadedSourceDataKind::Raw,
            Self::BrukerProcessed | Self::AgilentProcessed => LoadedSourceDataKind::Processed,
            Self::Json | Self::NmrMl | Self::JcampDx | Self::Csv | Self::JeolJdf => {
                LoadedSourceDataKind::Other
            }
        }
    }

    /// Returns true when this source format represents vendor raw acquisition data.
    #[must_use]
    pub const fn is_raw(self) -> bool {
        matches!(self.data_kind(), LoadedSourceDataKind::Raw)
    }

    /// Returns true when this source format represents vendor processed data.
    #[must_use]
    pub const fn is_processed(self) -> bool {
        matches!(self.data_kind(), LoadedSourceDataKind::Processed)
    }
}

impl AsRef<str> for LoadedSourceFormat {
    fn as_ref(&self) -> &str {
        (*self).as_str()
    }
}

impl fmt::Display for LoadedSourceFormat {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for LoadedSourceFormat {
    type Err = RSpinError;

    fn from_str(input: &str) -> Result<Self> {
        parse_loaded_source_format(input)
    }
}

impl LoadedSourceDataKind {
    /// Returns all known data kinds in stable display order.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[Self::Raw, Self::Processed, Self::Other]
    }

    /// Returns the canonical snake-case data kind name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Raw => "raw",
            Self::Processed => "processed",
            Self::Other => "other",
        }
    }

    /// Parses a source data-kind name.
    ///
    /// # Errors
    ///
    /// Returns an unsupported-feature error when `input` is not a known source
    /// data-kind name.
    pub fn parse(input: &str) -> Result<Self> {
        parse_loaded_source_data_kind(input)
    }

    /// Returns discovery metadata for this source data kind.
    #[must_use]
    pub fn info(self) -> LoadedSourceDataKindInfo {
        LoadedSourceDataKindInfo {
            name: self.as_str(),
            source_formats: LoadedSourceFormat::all()
                .iter()
                .filter(|format| format.data_kind() == self)
                .map(|format| format.as_str())
                .collect(),
        }
    }
}

impl AsRef<str> for LoadedSourceDataKind {
    fn as_ref(&self) -> &str {
        (*self).as_str()
    }
}

impl fmt::Display for LoadedSourceDataKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for LoadedSourceDataKind {
    type Err = RSpinError;

    fn from_str(input: &str) -> Result<Self> {
        parse_loaded_source_data_kind(input)
    }
}

const BRUKER_SOURCE_FORMATS: &[LoadedSourceFormat] = &[
    LoadedSourceFormat::BrukerProcessed,
    LoadedSourceFormat::BrukerFid,
    LoadedSourceFormat::BrukerSer,
];
const JEOL_SOURCE_FORMATS: &[LoadedSourceFormat] = &[LoadedSourceFormat::JeolJdf];
const AGILENT_VARIAN_SOURCE_FORMATS: &[LoadedSourceFormat] = &[
    LoadedSourceFormat::AgilentProcessed,
    LoadedSourceFormat::AgilentFid,
];

impl LoadedSourceVendor {
    /// Returns all known vendor families in stable display order.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        LOADED_SOURCE_VENDORS
    }

    /// Returns the canonical snake-case vendor name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Bruker => "bruker",
            Self::Jeol => "jeol",
            Self::AgilentVarian => "agilent_varian",
        }
    }

    /// Returns the known source formats for this vendor family.
    #[must_use]
    pub const fn source_formats(self) -> &'static [LoadedSourceFormat] {
        match self {
            Self::Bruker => BRUKER_SOURCE_FORMATS,
            Self::Jeol => JEOL_SOURCE_FORMATS,
            Self::AgilentVarian => AGILENT_VARIAN_SOURCE_FORMATS,
        }
    }

    /// Returns discovery metadata for this vendor family.
    #[must_use]
    pub fn info(self) -> LoadedSourceVendorInfo {
        LoadedSourceVendorInfo {
            name: self.as_str(),
            source_formats: self
                .source_formats()
                .iter()
                .map(|format| format.as_str())
                .collect(),
        }
    }

    /// Parses a source vendor name or common alias.
    ///
    /// # Errors
    ///
    /// Returns an unsupported-feature error when `input` is not a known source
    /// vendor name.
    pub fn parse(input: &str) -> Result<Self> {
        parse_loaded_source_vendor(input)
    }
}

impl AsRef<str> for LoadedSourceVendor {
    fn as_ref(&self) -> &str {
        (*self).as_str()
    }
}

impl fmt::Display for LoadedSourceVendor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for LoadedSourceVendor {
    type Err = RSpinError;

    fn from_str(input: &str) -> Result<Self> {
        parse_loaded_source_vendor(input)
    }
}

/// Parses a bundle source format name.
///
/// Accepted aliases include common file extensions and vendor synonyms such as
/// `jdx`, `jdf`, `bruker raw`, `varian fid`, and `xml`.
///
/// # Errors
///
/// Returns an unsupported-feature error when `input` is not a known source
/// format name.
pub fn parse_loaded_source_format(input: &str) -> Result<LoadedSourceFormat> {
    match normalized_source_format_name(input).as_str() {
        "json" | "rspinjson" => Ok(LoadedSourceFormat::Json),
        "nmrml" | "xml" => Ok(LoadedSourceFormat::NmrMl),
        "jcampdx" | "jcamp" | "jdx" | "dx" => Ok(LoadedSourceFormat::JcampDx),
        "csv" => Ok(LoadedSourceFormat::Csv),
        "jeoljdf" | "jeol" | "jdf" => Ok(LoadedSourceFormat::JeolJdf),
        "brukerprocessed" | "brukerpdata" | "bruker1r" | "bruker2rr" => {
            Ok(LoadedSourceFormat::BrukerProcessed)
        }
        "brukerfid" => Ok(LoadedSourceFormat::BrukerFid),
        "brukerser" | "ser" => Ok(LoadedSourceFormat::BrukerSer),
        "agilentprocessed" | "varianprocessed" | "agilentphasefile" | "varianphasefile" => {
            Ok(LoadedSourceFormat::AgilentProcessed)
        }
        "agilentfid" | "varianfid" => Ok(LoadedSourceFormat::AgilentFid),
        _ => Err(RSpinError::Unsupported {
            feature: "bundle source format name",
        }),
    }
}

/// Parses a bundle source data-kind name.
///
/// Accepted names are `raw`, `processed`, and `other`.
///
/// # Errors
///
/// Returns an unsupported-feature error when `input` is not a known source
/// data-kind name.
pub fn parse_loaded_source_data_kind(input: &str) -> Result<LoadedSourceDataKind> {
    match normalized_source_format_name(input).as_str() {
        "raw" => Ok(LoadedSourceDataKind::Raw),
        "processed" => Ok(LoadedSourceDataKind::Processed),
        "other" => Ok(LoadedSourceDataKind::Other),
        _ => Err(RSpinError::Unsupported {
            feature: "bundle source data kind name",
        }),
    }
}

/// Parses a bundle source vendor name.
///
/// Accepted aliases include `bruker`, `jeol`, `agilent`, `varian`, and
/// `agilent_varian`.
///
/// # Errors
///
/// Returns an unsupported-feature error when `input` is not a known source
/// vendor name.
pub fn parse_loaded_source_vendor(input: &str) -> Result<LoadedSourceVendor> {
    match normalized_source_format_name(input).as_str() {
        "bruker" => Ok(LoadedSourceVendor::Bruker),
        "jeol" => Ok(LoadedSourceVendor::Jeol),
        "agilent" | "varian" | "agilentvarian" => Ok(LoadedSourceVendor::AgilentVarian),
        _ => Err(RSpinError::Unsupported {
            feature: "bundle source vendor name",
        }),
    }
}

/// Returns discovery metadata for supported built-in bundle source formats.
#[must_use]
pub fn supported_bundle_source_formats() -> Vec<LoadedSourceFormatInfo> {
    LoadedSourceFormat::all()
        .iter()
        .map(|format| format.info())
        .collect()
}

/// Returns discovery metadata for supported built-in bundle source vendors.
#[must_use]
pub fn supported_bundle_source_vendors() -> Vec<LoadedSourceVendorInfo> {
    LoadedSourceVendor::all()
        .iter()
        .map(|vendor| vendor.info())
        .collect()
}

/// Returns discovery metadata for supported bundle source data kinds.
#[must_use]
pub fn supported_bundle_source_data_kinds() -> Vec<LoadedSourceDataKindInfo> {
    LoadedSourceDataKind::all()
        .iter()
        .map(|data_kind| data_kind.info())
        .collect()
}

fn normalized_source_format_name(input: &str) -> String {
    input
        .trim()
        .chars()
        .filter(|character| !matches!(character, '_' | '-' | ' ' | '.'))
        .flat_map(char::to_lowercase)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_loaded_source_format_names_and_aliases() -> Result<()> {
        assert_eq!(
            LoadedSourceFormat::all()
                .iter()
                .map(|format| format.as_str())
                .collect::<Vec<_>>(),
            vec![
                "json",
                "nmrml",
                "jcamp_dx",
                "csv",
                "jeol_jdf",
                "bruker_processed",
                "bruker_fid",
                "bruker_ser",
                "agilent_processed",
                "agilent_fid"
            ]
        );
        assert_eq!(
            parse_loaded_source_format("jcamp_dx")?,
            LoadedSourceFormat::JcampDx
        );
        assert_eq!(
            "jdx".parse::<LoadedSourceFormat>()?,
            LoadedSourceFormat::JcampDx
        );
        assert_eq!(
            LoadedSourceFormat::parse("varian fid")?,
            LoadedSourceFormat::AgilentFid
        );
        assert_eq!(
            parse_loaded_source_format("bruker 2rr")?,
            LoadedSourceFormat::BrukerProcessed
        );
        assert_eq!(LoadedSourceFormat::JeolJdf.as_str(), "jeol_jdf");
        assert_eq!(
            LoadedSourceFormat::JeolJdf.vendor(),
            Some(LoadedSourceVendor::Jeol)
        );
        assert_eq!(
            LoadedSourceFormat::BrukerFid.data_kind(),
            LoadedSourceDataKind::Raw
        );
        assert!(LoadedSourceFormat::BrukerSer.is_raw());
        assert_eq!(
            LoadedSourceFormat::AgilentProcessed.data_kind(),
            LoadedSourceDataKind::Processed
        );
        assert!(LoadedSourceFormat::BrukerProcessed.is_processed());
        assert_eq!(
            LoadedSourceFormat::JcampDx.data_kind(),
            LoadedSourceDataKind::Other
        );
        assert_eq!(LoadedSourceDataKind::Raw.as_str(), "raw");
        assert_eq!(LoadedSourceDataKind::Processed.to_string(), "processed");
        assert_eq!(
            parse_loaded_source_data_kind("raw")?,
            LoadedSourceDataKind::Raw
        );
        let error = "source-processed"
            .parse::<LoadedSourceDataKind>()
            .expect_err("prefixed data kind should fail");
        assert!(matches!(error, RSpinError::Unsupported { .. }));
        assert_eq!(
            LoadedSourceDataKind::parse("processed")?,
            LoadedSourceDataKind::Processed
        );
        assert_eq!(
            "other".parse::<LoadedSourceDataKind>()?,
            LoadedSourceDataKind::Other
        );
        assert_eq!(
            LoadedSourceDataKind::all(),
            &[
                LoadedSourceDataKind::Raw,
                LoadedSourceDataKind::Processed,
                LoadedSourceDataKind::Other
            ]
        );
        assert_eq!(LoadedSourceFormat::NmrMl.to_string(), "nmrml");
        assert_eq!(
            LoadedSourceFormat::JcampDx.file_extensions(),
            &["jdx", "dx", "jcamp"]
        );
        assert_eq!(LoadedSourceFormat::JeolJdf.file_extensions(), &["jdf"]);
        assert_eq!(LoadedSourceFormat::Json.path_markers(), &[] as &[&str]);
        assert_eq!(
            LoadedSourceFormat::BrukerSer.path_markers(),
            &["ser", "acqus", "acqu2s"]
        );
        assert_eq!(
            LoadedSourceFormat::AgilentFid.path_markers(),
            &["fid", "procpar"]
        );

        let error = parse_loaded_source_format("unknown-format")
            .expect_err("unsupported source format should fail");
        assert!(matches!(error, RSpinError::Unsupported { .. }));
        Ok(())
    }

    #[test]
    fn exposes_supported_source_metadata() -> Result<()> {
        let formats = supported_bundle_source_formats();
        assert!(formats.iter().any(|info| {
            info.name == "jcamp_dx"
                && info.vendor.is_none()
                && info.data_kind == LoadedSourceDataKind::Other
                && info.extensions.contains(&"jdx")
                && info.path_markers.is_empty()
        }));
        assert!(formats.iter().any(|info| {
            info.name == "bruker_ser"
                && info.vendor == Some("bruker")
                && info.data_kind == LoadedSourceDataKind::Raw
                && info.extensions.is_empty()
                && info.path_markers.contains(&"ser")
        }));

        let vendors = supported_bundle_source_vendors();
        let bruker = vendors
            .iter()
            .find(|info| info.name == "bruker")
            .ok_or_else(|| RSpinError::Parse {
                format: "bundle source metadata",
                message: "missing Bruker source vendor metadata".to_owned(),
            })?;
        assert!(bruker.source_formats.contains(&"bruker_fid"));
        assert!(bruker.source_formats.contains(&"bruker_ser"));

        let data_kinds = supported_bundle_source_data_kinds();
        let raw = data_kinds
            .iter()
            .find(|info| info.name == "raw")
            .ok_or_else(|| RSpinError::Parse {
                format: "bundle source metadata",
                message: "missing raw source data-kind metadata".to_owned(),
            })?;
        assert!(raw.source_formats.contains(&"agilent_fid"));
        assert!(!raw.source_formats.contains(&"jcamp_dx"));
        Ok(())
    }

    #[test]
    fn parses_loaded_source_vendor_names_and_lists_formats() -> Result<()> {
        assert_eq!(
            LoadedSourceVendor::all()
                .iter()
                .map(|vendor| vendor.as_str())
                .collect::<Vec<_>>(),
            vec!["bruker", "jeol", "agilent_varian"]
        );
        assert_eq!(
            parse_loaded_source_vendor("bruker")?,
            LoadedSourceVendor::Bruker
        );
        assert_eq!(
            "jeol".parse::<LoadedSourceVendor>()?,
            LoadedSourceVendor::Jeol
        );
        assert_eq!(
            LoadedSourceVendor::parse("varian")?,
            LoadedSourceVendor::AgilentVarian
        );
        assert_eq!(
            LoadedSourceVendor::AgilentVarian.to_string(),
            "agilent_varian"
        );
        assert_eq!(
            LoadedSourceVendor::Bruker.source_formats(),
            &[
                LoadedSourceFormat::BrukerProcessed,
                LoadedSourceFormat::BrukerFid,
                LoadedSourceFormat::BrukerSer
            ]
        );

        let error = parse_loaded_source_vendor("unknown-vendor")
            .expect_err("unsupported source vendor should fail");
        assert!(matches!(error, RSpinError::Unsupported { .. }));
        Ok(())
    }
}
