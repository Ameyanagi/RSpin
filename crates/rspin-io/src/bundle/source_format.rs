//! Source-format names used by the unified bundle loader.

use std::{fmt, str::FromStr};

use rspin_core::{RSpinError, Result};

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
