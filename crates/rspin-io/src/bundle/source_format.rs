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

impl LoadedSourceFormat {
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

        let error = parse_loaded_source_format("unknown-format")
            .expect_err("unsupported source format should fail");
        assert!(matches!(error, RSpinError::Unsupported { .. }));
        Ok(())
    }

    #[test]
    fn parses_loaded_source_vendor_names_and_lists_formats() -> Result<()> {
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
