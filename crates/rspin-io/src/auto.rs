//! Text spectrum format detection and convenience readers/writers.

use std::{fmt, fs, path::Path, str::FromStr};

use rspin_core::{RSpinError, Result, Spectrum1D, Spectrum2D};

use crate::{
    SpectrumPathReader, SpectrumReader, read_agilent_fid_1d_dir, read_agilent_fid_2d_dir,
    read_agilent_processed_1d_dir, read_agilent_processed_2d_dir, read_bruker_fid_1d_dir,
    read_bruker_processed_1d_dir, read_bruker_processed_2d_dir, read_bruker_ser_2d_dir,
    read_jcamp_dx_1d, read_jeol_jdf_1d_file, read_jeol_jdf_2d_file, read_nmrml_1d_str,
    read_nmrml_2d_str, read_spectrum1d_csv, read_spectrum1d_json, read_spectrum2d_csv,
    read_spectrum2d_json,
};

mod writer;

pub use writer::{
    Spectrum1DTextWriter, Spectrum1DWriteFormat, Spectrum2DTextWriter, Spectrum2DWriteFormat,
    parse_spectrum1d_write_format, parse_spectrum2d_write_format, write_spectrum1d_text,
    write_spectrum2d_text,
};

/// Text spectrum formats supported by the auto-detecting readers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpectrumTextFormat {
    /// `RSpin` JSON spectrum payload.
    Json,
    /// nmrML XML payload.
    NmrMl,
    /// JCAMP-DX text payload.
    JcampDx,
    /// `RSpin` CSV payload.
    Csv,
}

impl SpectrumTextFormat {
    /// Returns the canonical snake-case format name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::NmrMl => "nmrml",
            Self::JcampDx => "jcamp_dx",
            Self::Csv => "csv",
        }
    }
}

impl fmt::Display for SpectrumTextFormat {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for SpectrumTextFormat {
    type Err = RSpinError;

    fn from_str(input: &str) -> Result<Self> {
        parse_spectrum_text_format(input)
    }
}

/// Filesystem formats supported by the one-dimensional auto reader.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spectrum1DPathFormat {
    /// Text payload detected as JSON.
    Json,
    /// Text payload detected as nmrML.
    NmrMl,
    /// Text payload detected as JCAMP-DX.
    JcampDx,
    /// Text payload detected as CSV.
    Csv,
    /// JEOL Delta `.jdf` file.
    JeolJdf,
    /// Bruker processed one-dimensional dataset directory.
    BrukerProcessed,
    /// Bruker raw one-dimensional FID dataset directory or `fid` file.
    BrukerFid,
    /// Agilent/Varian processed one-dimensional `phasefile` dataset.
    AgilentProcessed,
    /// Agilent/Varian raw one-dimensional FID dataset directory or `fid` file.
    AgilentFid,
}

impl Spectrum1DPathFormat {
    /// Returns the canonical snake-case format name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::NmrMl => "nmrml",
            Self::JcampDx => "jcamp_dx",
            Self::Csv => "csv",
            Self::JeolJdf => "jeol_jdf",
            Self::BrukerProcessed => "bruker_processed",
            Self::BrukerFid => "bruker_fid",
            Self::AgilentProcessed => "agilent_processed",
            Self::AgilentFid => "agilent_fid",
        }
    }
}

impl fmt::Display for Spectrum1DPathFormat {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for Spectrum1DPathFormat {
    type Err = RSpinError;

    fn from_str(input: &str) -> Result<Self> {
        parse_spectrum1d_path_format(input)
    }
}

/// Filesystem formats supported by the two-dimensional auto reader.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spectrum2DPathFormat {
    /// Text payload detected as JSON.
    Json,
    /// Text payload detected as nmrML.
    NmrMl,
    /// Text payload detected as CSV.
    Csv,
    /// JEOL Delta `.jdf` file.
    JeolJdf,
    /// Bruker processed two-dimensional dataset directory.
    BrukerProcessed,
    /// Bruker raw two-dimensional `ser` dataset directory or `ser` file.
    BrukerSer,
    /// Agilent/Varian processed two-dimensional `phasefile` dataset.
    AgilentProcessed,
    /// Agilent/Varian raw two-dimensional FID dataset directory or `fid` file.
    AgilentFid,
}

impl Spectrum2DPathFormat {
    /// Returns the canonical snake-case format name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::NmrMl => "nmrml",
            Self::Csv => "csv",
            Self::JeolJdf => "jeol_jdf",
            Self::BrukerProcessed => "bruker_processed",
            Self::BrukerSer => "bruker_ser",
            Self::AgilentProcessed => "agilent_processed",
            Self::AgilentFid => "agilent_fid",
        }
    }
}

impl fmt::Display for Spectrum2DPathFormat {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for Spectrum2DPathFormat {
    type Err = RSpinError;

    fn from_str(input: &str) -> Result<Self> {
        parse_spectrum2d_path_format(input)
    }
}

/// Filesystem text formats supported by the one-dimensional auto writer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spectrum1DWritePathFormat {
    /// `RSpin` JSON spectrum payload.
    Json,
    /// nmrML XML payload.
    NmrMl,
    /// JCAMP-DX text payload.
    JcampDx,
    /// `RSpin` CSV payload.
    Csv,
}

impl Spectrum1DWritePathFormat {
    /// Returns the canonical snake-case format name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::NmrMl => "nmrml",
            Self::JcampDx => "jcamp_dx",
            Self::Csv => "csv",
        }
    }
}

impl fmt::Display for Spectrum1DWritePathFormat {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for Spectrum1DWritePathFormat {
    type Err = RSpinError;

    fn from_str(input: &str) -> Result<Self> {
        parse_spectrum1d_write_path_format(input)
    }
}

/// Filesystem text formats supported by the two-dimensional auto writer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spectrum2DWritePathFormat {
    /// `RSpin` JSON spectrum payload.
    Json,
    /// nmrML XML payload.
    NmrMl,
    /// `RSpin` CSV payload.
    Csv,
}

impl Spectrum2DWritePathFormat {
    /// Returns the canonical snake-case format name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::NmrMl => "nmrml",
            Self::Csv => "csv",
        }
    }
}

impl fmt::Display for Spectrum2DWritePathFormat {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for Spectrum2DWritePathFormat {
    type Err = RSpinError;

    fn from_str(input: &str) -> Result<Self> {
        parse_spectrum2d_write_path_format(input)
    }
}

/// Auto-detecting reader for one-dimensional text spectra.
#[derive(Clone, Copy, Debug, Default)]
pub struct AutoSpectrum1DText;

impl SpectrumReader for AutoSpectrum1DText {
    type Output = Spectrum1D;

    fn read_str(&self, input: &str) -> Result<Self::Output> {
        read_spectrum1d_text(input)
    }
}

/// Auto-detecting reader for two-dimensional text spectra.
#[derive(Clone, Copy, Debug, Default)]
pub struct AutoSpectrum2DText;

impl SpectrumReader for AutoSpectrum2DText {
    type Output = Spectrum2D;

    fn read_str(&self, input: &str) -> Result<Self::Output> {
        read_spectrum2d_text(input)
    }
}

/// Auto-detecting reader for one-dimensional spectrum paths.
#[derive(Clone, Copy, Debug, Default)]
pub struct AutoSpectrum1DPath;

impl AutoSpectrum1DPath {
    /// Reads a one-dimensional spectrum from an auto-detected path.
    ///
    /// # Errors
    ///
    /// Returns an error when the path is missing, unsupported, or malformed.
    pub fn read_path(self, path: impl AsRef<Path>) -> Result<Spectrum1D> {
        read_spectrum1d_path(path)
    }

    /// Reads a one-dimensional spectrum from `path` using an explicit format.
    ///
    /// # Errors
    ///
    /// Returns an error when the selected reader rejects the path contents.
    pub fn read_path_as(
        self,
        path: impl AsRef<Path>,
        format: Spectrum1DPathFormat,
    ) -> Result<Spectrum1D> {
        read_spectrum1d_path_as(path, format)
    }
}

impl SpectrumPathReader for AutoSpectrum1DPath {
    type Output = Spectrum1D;

    fn read_path(&self, path: &Path) -> Result<Self::Output> {
        read_spectrum1d_path(path)
    }
}

/// Auto-detecting reader for two-dimensional spectrum paths.
#[derive(Clone, Copy, Debug, Default)]
pub struct AutoSpectrum2DPath;

impl AutoSpectrum2DPath {
    /// Reads a two-dimensional spectrum from an auto-detected path.
    ///
    /// # Errors
    ///
    /// Returns an error when the path is missing, unsupported, or malformed.
    pub fn read_path(self, path: impl AsRef<Path>) -> Result<Spectrum2D> {
        read_spectrum2d_path(path)
    }

    /// Reads a two-dimensional spectrum from `path` using an explicit format.
    ///
    /// # Errors
    ///
    /// Returns an error when the selected reader rejects the path contents.
    pub fn read_path_as(
        self,
        path: impl AsRef<Path>,
        format: Spectrum2DPathFormat,
    ) -> Result<Spectrum2D> {
        read_spectrum2d_path_as(path, format)
    }
}

impl SpectrumPathReader for AutoSpectrum2DPath {
    type Output = Spectrum2D;

    fn read_path(&self, path: &Path) -> Result<Self::Output> {
        read_spectrum2d_path(path)
    }
}

/// Extension-selecting writer for one-dimensional spectrum paths.
#[derive(Clone, Copy, Debug, Default)]
pub struct AutoSpectrum1DPathWriter;

impl AutoSpectrum1DPathWriter {
    /// Writes a one-dimensional spectrum to a path using the path extension.
    ///
    /// # Errors
    ///
    /// Returns an error when the path extension is unsupported, the spectrum
    /// cannot be represented by the selected writer, or the file cannot be
    /// written.
    pub fn write_path(self, spectrum: &Spectrum1D, path: impl AsRef<Path>) -> Result<()> {
        write_spectrum1d_path(spectrum, path)
    }

    /// Writes a one-dimensional spectrum to `path` using an explicit format.
    ///
    /// # Errors
    ///
    /// Returns an error when the spectrum cannot be represented by the selected
    /// writer or the file cannot be written.
    pub fn write_path_as(
        self,
        spectrum: &Spectrum1D,
        path: impl AsRef<Path>,
        format: Spectrum1DWritePathFormat,
    ) -> Result<()> {
        write_spectrum1d_path_as(spectrum, path, format)
    }
}

/// Extension-selecting writer for two-dimensional spectrum paths.
#[derive(Clone, Copy, Debug, Default)]
pub struct AutoSpectrum2DPathWriter;

impl AutoSpectrum2DPathWriter {
    /// Writes a two-dimensional spectrum to a path using the path extension.
    ///
    /// # Errors
    ///
    /// Returns an error when the path extension is unsupported, the spectrum
    /// cannot be represented by the selected writer, or the file cannot be
    /// written.
    pub fn write_path(self, spectrum: &Spectrum2D, path: impl AsRef<Path>) -> Result<()> {
        write_spectrum2d_path(spectrum, path)
    }

    /// Writes a two-dimensional spectrum to `path` using an explicit format.
    ///
    /// # Errors
    ///
    /// Returns an error when the spectrum cannot be represented by the selected
    /// writer or the file cannot be written.
    pub fn write_path_as(
        self,
        spectrum: &Spectrum2D,
        path: impl AsRef<Path>,
        format: Spectrum2DWritePathFormat,
    ) -> Result<()> {
        write_spectrum2d_path_as(spectrum, path, format)
    }
}

/// Detects a supported text spectrum format from the payload shape.
///
/// # Errors
///
/// Returns an error when the payload is empty or does not look like a
/// supported spectrum text format.
pub fn detect_spectrum_text_format(input: &str) -> Result<SpectrumTextFormat> {
    let trimmed = input.trim_start();
    if trimmed.is_empty() {
        return Err(RSpinError::Parse {
            format: "text spectrum",
            message: "empty input".to_owned(),
        });
    }

    if trimmed.starts_with('{') {
        return Ok(SpectrumTextFormat::Json);
    }

    if looks_like_nmrml(trimmed) {
        return Ok(SpectrumTextFormat::NmrMl);
    }

    if looks_like_jcamp_dx(input) {
        return Ok(SpectrumTextFormat::JcampDx);
    }

    if looks_like_csv(input) {
        return Ok(SpectrumTextFormat::Csv);
    }

    Err(RSpinError::Unsupported {
        feature: "text spectrum format",
    })
}

/// Parses a text spectrum format name.
///
/// Accepted names include `json`, `nmrml`, `xml`, `jcamp_dx`, `jcamp`, `jdx`,
/// `dx`, and `csv`.
///
/// # Errors
///
/// Returns an unsupported-feature error when `input` is not a supported text
/// spectrum format name.
pub fn parse_spectrum_text_format(input: &str) -> Result<SpectrumTextFormat> {
    match normalized_format_name(input).as_str() {
        "json" => Ok(SpectrumTextFormat::Json),
        "nmrml" | "xml" => Ok(SpectrumTextFormat::NmrMl),
        "jcampdx" | "jcamp" | "jdx" | "dx" => Ok(SpectrumTextFormat::JcampDx),
        "csv" => Ok(SpectrumTextFormat::Csv),
        _ => Err(RSpinError::Unsupported {
            feature: "text spectrum format name",
        }),
    }
}

/// Reads a one-dimensional spectrum from JSON, nmrML, JCAMP-DX, or CSV text.
///
/// # Errors
///
/// Returns an error when the format cannot be detected or the selected parser
/// rejects the payload.
pub fn read_spectrum1d_text(input: &str) -> Result<Spectrum1D> {
    read_spectrum1d_text_as(input, detect_spectrum_text_format(input)?)
}

/// Reads a one-dimensional spectrum from text using an explicit format.
///
/// # Errors
///
/// Returns an error when the selected parser rejects the payload.
pub fn read_spectrum1d_text_as(input: &str, format: SpectrumTextFormat) -> Result<Spectrum1D> {
    match format {
        SpectrumTextFormat::Json => read_spectrum1d_json(input),
        SpectrumTextFormat::NmrMl => read_nmrml_1d_str(input),
        SpectrumTextFormat::JcampDx => read_jcamp_dx_1d(input),
        SpectrumTextFormat::Csv => read_spectrum1d_csv(input),
    }
}

/// Reads a two-dimensional spectrum from JSON, nmrML, or CSV text.
///
/// # Errors
///
/// Returns an error when the format cannot be detected, the selected parser
/// rejects the payload, or the payload is a one-dimensional-only format.
pub fn read_spectrum2d_text(input: &str) -> Result<Spectrum2D> {
    read_spectrum2d_text_as(input, detect_spectrum_text_format(input)?)
}

/// Reads a two-dimensional spectrum from text using an explicit format.
///
/// # Errors
///
/// Returns an error when the selected parser rejects the payload, or when the
/// selected format is one-dimensional-only.
pub fn read_spectrum2d_text_as(input: &str, format: SpectrumTextFormat) -> Result<Spectrum2D> {
    match format {
        SpectrumTextFormat::Json => read_spectrum2d_json(input),
        SpectrumTextFormat::NmrMl => read_nmrml_2d_str(input),
        SpectrumTextFormat::Csv => read_spectrum2d_csv(input),
        SpectrumTextFormat::JcampDx => Err(RSpinError::Unsupported {
            feature: "two-dimensional JCAMP-DX text reader",
        }),
    }
}

/// Detects a supported one-dimensional spectrum format from a filesystem path.
///
/// # Errors
///
/// Returns an error when the path is missing or does not look like a supported
/// one-dimensional spectrum path.
pub fn detect_spectrum1d_path_format(path: impl AsRef<Path>) -> Result<Spectrum1DPathFormat> {
    let path = path.as_ref();
    ensure_path_exists(path)?;

    if looks_like_bruker_processed_1d(path) {
        return Ok(Spectrum1DPathFormat::BrukerProcessed);
    }
    if looks_like_bruker_fid(path) {
        return Ok(Spectrum1DPathFormat::BrukerFid);
    }
    if looks_like_agilent_processed_1d(path) {
        return Ok(Spectrum1DPathFormat::AgilentProcessed);
    }
    if looks_like_agilent_fid(path) {
        return Ok(Spectrum1DPathFormat::AgilentFid);
    }
    if is_extension(path, &["jdf"]) {
        return Ok(Spectrum1DPathFormat::JeolJdf);
    }

    match text_format_from_path(path)? {
        SpectrumTextFormat::Json => Ok(Spectrum1DPathFormat::Json),
        SpectrumTextFormat::NmrMl => Ok(Spectrum1DPathFormat::NmrMl),
        SpectrumTextFormat::JcampDx => Ok(Spectrum1DPathFormat::JcampDx),
        SpectrumTextFormat::Csv => Ok(Spectrum1DPathFormat::Csv),
    }
}

/// Parses a one-dimensional path format name.
///
/// Accepted names include text format names plus `jeol_jdf`, `jdf`,
/// `bruker_processed`, `bruker_fid`, `agilent_processed`, `agilent_fid`,
/// `varian_processed`, and `varian_fid`.
///
/// # Errors
///
/// Returns an unsupported-feature error when `input` is not a supported
/// one-dimensional path format name.
pub fn parse_spectrum1d_path_format(input: &str) -> Result<Spectrum1DPathFormat> {
    match normalized_format_name(input).as_str() {
        "json" => Ok(Spectrum1DPathFormat::Json),
        "nmrml" | "xml" => Ok(Spectrum1DPathFormat::NmrMl),
        "jcampdx" | "jcamp" | "jdx" | "dx" => Ok(Spectrum1DPathFormat::JcampDx),
        "csv" => Ok(Spectrum1DPathFormat::Csv),
        "jeoljdf" | "jeol" | "jdf" => Ok(Spectrum1DPathFormat::JeolJdf),
        "brukerprocessed" | "brukerpdata" | "bruker1r" => Ok(Spectrum1DPathFormat::BrukerProcessed),
        "brukerfid" | "brukerraw" => Ok(Spectrum1DPathFormat::BrukerFid),
        "agilentprocessed" | "varianprocessed" | "agilentphasefile" | "varianphasefile" => {
            Ok(Spectrum1DPathFormat::AgilentProcessed)
        }
        "agilentfid" | "varianfid" => Ok(Spectrum1DPathFormat::AgilentFid),
        _ => Err(RSpinError::Unsupported {
            feature: "one-dimensional spectrum path format name",
        }),
    }
}

/// Detects a supported two-dimensional spectrum format from a filesystem path.
///
/// # Errors
///
/// Returns an error when the path is missing or does not look like a supported
/// two-dimensional spectrum path.
pub fn detect_spectrum2d_path_format(path: impl AsRef<Path>) -> Result<Spectrum2DPathFormat> {
    let path = path.as_ref();
    ensure_path_exists(path)?;

    if looks_like_bruker_processed_2d(path) {
        return Ok(Spectrum2DPathFormat::BrukerProcessed);
    }
    if looks_like_bruker_ser(path) {
        return Ok(Spectrum2DPathFormat::BrukerSer);
    }
    if looks_like_agilent_processed(path) {
        return Ok(Spectrum2DPathFormat::AgilentProcessed);
    }
    if looks_like_agilent_fid(path) {
        return Ok(Spectrum2DPathFormat::AgilentFid);
    }
    if is_extension(path, &["jdf"]) {
        return Ok(Spectrum2DPathFormat::JeolJdf);
    }

    match text_format_from_path(path)? {
        SpectrumTextFormat::Json => Ok(Spectrum2DPathFormat::Json),
        SpectrumTextFormat::NmrMl => Ok(Spectrum2DPathFormat::NmrMl),
        SpectrumTextFormat::Csv => Ok(Spectrum2DPathFormat::Csv),
        SpectrumTextFormat::JcampDx => Err(RSpinError::Unsupported {
            feature: "two-dimensional JCAMP-DX path reader",
        }),
    }
}

/// Parses a two-dimensional path format name.
///
/// Accepted names include `json`, `nmrml`, `xml`, `csv`, `jeol_jdf`, `jdf`,
/// `bruker_processed`, `bruker_ser`, `agilent_processed`, `agilent_fid`,
/// `varian_processed`, and `varian_fid`.
///
/// # Errors
///
/// Returns an unsupported-feature error when `input` is not a supported
/// two-dimensional path format name.
pub fn parse_spectrum2d_path_format(input: &str) -> Result<Spectrum2DPathFormat> {
    match normalized_format_name(input).as_str() {
        "json" => Ok(Spectrum2DPathFormat::Json),
        "nmrml" | "xml" => Ok(Spectrum2DPathFormat::NmrMl),
        "csv" => Ok(Spectrum2DPathFormat::Csv),
        "jeoljdf" | "jeol" | "jdf" => Ok(Spectrum2DPathFormat::JeolJdf),
        "brukerprocessed" | "brukerpdata" | "bruker2rr" => {
            Ok(Spectrum2DPathFormat::BrukerProcessed)
        }
        "brukerser" | "ser" | "brukerraw" => Ok(Spectrum2DPathFormat::BrukerSer),
        "agilentprocessed" | "varianprocessed" | "agilentphasefile" | "varianphasefile" => {
            Ok(Spectrum2DPathFormat::AgilentProcessed)
        }
        "agilentfid" | "varianfid" => Ok(Spectrum2DPathFormat::AgilentFid),
        _ => Err(RSpinError::Unsupported {
            feature: "two-dimensional spectrum path format name",
        }),
    }
}

/// Detects the one-dimensional text writer format selected by a destination path.
///
/// Unlike reader detection, writer detection uses only the path extension and
/// does not require the destination path to exist.
///
/// # Errors
///
/// Returns an unsupported-feature error when the extension is not a supported
/// one-dimensional text export format.
pub fn detect_spectrum1d_write_path_format(
    path: impl AsRef<Path>,
) -> Result<Spectrum1DWritePathFormat> {
    let path = path.as_ref();
    if is_extension(path, &["json"]) {
        return Ok(Spectrum1DWritePathFormat::Json);
    }
    if is_extension(path, &["nmrml", "xml"]) {
        return Ok(Spectrum1DWritePathFormat::NmrMl);
    }
    if is_extension(path, &["jdx", "dx"]) {
        return Ok(Spectrum1DWritePathFormat::JcampDx);
    }
    if is_extension(path, &["csv"]) {
        return Ok(Spectrum1DWritePathFormat::Csv);
    }
    Err(RSpinError::Unsupported {
        feature: "one-dimensional spectrum path writer format",
    })
}

/// Parses a one-dimensional path writer format name.
///
/// Accepted names include `json`, `nmrml`, `xml`, `jcamp_dx`, `jcamp`, `jdx`,
/// `dx`, and `csv`.
///
/// # Errors
///
/// Returns an unsupported-feature error when `input` is not a supported
/// one-dimensional path writer format name.
pub fn parse_spectrum1d_write_path_format(input: &str) -> Result<Spectrum1DWritePathFormat> {
    match normalized_format_name(input).as_str() {
        "json" => Ok(Spectrum1DWritePathFormat::Json),
        "nmrml" | "xml" => Ok(Spectrum1DWritePathFormat::NmrMl),
        "jcampdx" | "jcamp" | "jdx" | "dx" => Ok(Spectrum1DWritePathFormat::JcampDx),
        "csv" => Ok(Spectrum1DWritePathFormat::Csv),
        _ => Err(RSpinError::Unsupported {
            feature: "one-dimensional spectrum path writer format name",
        }),
    }
}

/// Detects the two-dimensional text writer format selected by a destination path.
///
/// Unlike reader detection, writer detection uses only the path extension and
/// does not require the destination path to exist.
///
/// # Errors
///
/// Returns an unsupported-feature error when the extension is not a supported
/// two-dimensional text export format.
pub fn detect_spectrum2d_write_path_format(
    path: impl AsRef<Path>,
) -> Result<Spectrum2DWritePathFormat> {
    let path = path.as_ref();
    if is_extension(path, &["json"]) {
        return Ok(Spectrum2DWritePathFormat::Json);
    }
    if is_extension(path, &["nmrml", "xml"]) {
        return Ok(Spectrum2DWritePathFormat::NmrMl);
    }
    if is_extension(path, &["csv"]) {
        return Ok(Spectrum2DWritePathFormat::Csv);
    }
    Err(RSpinError::Unsupported {
        feature: "two-dimensional spectrum path writer format",
    })
}

/// Parses a two-dimensional path writer format name.
///
/// Accepted names include `json`, `nmrml`, `xml`, and `csv`.
///
/// # Errors
///
/// Returns an unsupported-feature error when `input` is not a supported
/// two-dimensional path writer format name.
pub fn parse_spectrum2d_write_path_format(input: &str) -> Result<Spectrum2DWritePathFormat> {
    match normalized_format_name(input).as_str() {
        "json" => Ok(Spectrum2DWritePathFormat::Json),
        "nmrml" | "xml" => Ok(Spectrum2DWritePathFormat::NmrMl),
        "csv" => Ok(Spectrum2DWritePathFormat::Csv),
        _ => Err(RSpinError::Unsupported {
            feature: "two-dimensional spectrum path writer format name",
        }),
    }
}

/// Reads a one-dimensional spectrum from an auto-detected path.
///
/// # Errors
///
/// Returns an error when the format cannot be detected or the selected parser
/// rejects the path contents.
pub fn read_spectrum1d_path(path: impl AsRef<Path>) -> Result<Spectrum1D> {
    let path = path.as_ref();
    read_spectrum1d_path_as(path, detect_spectrum1d_path_format(path)?)
}

/// Reads a one-dimensional spectrum from a path using an explicit format.
///
/// # Errors
///
/// Returns an error when the selected reader rejects the path contents.
pub fn read_spectrum1d_path_as(
    path: impl AsRef<Path>,
    format: Spectrum1DPathFormat,
) -> Result<Spectrum1D> {
    let path = path.as_ref();
    match format {
        Spectrum1DPathFormat::Json => read_spectrum1d_json(&read_text_file(path)?),
        Spectrum1DPathFormat::NmrMl => read_nmrml_1d_str(&read_text_file(path)?),
        Spectrum1DPathFormat::JcampDx => read_jcamp_dx_1d(&read_text_file(path)?),
        Spectrum1DPathFormat::Csv => read_spectrum1d_csv(&read_text_file(path)?),
        Spectrum1DPathFormat::JeolJdf => read_jeol_jdf_1d_file(path),
        Spectrum1DPathFormat::BrukerProcessed => read_bruker_processed_1d_dir(path),
        Spectrum1DPathFormat::BrukerFid => read_bruker_fid_1d_dir(path),
        Spectrum1DPathFormat::AgilentProcessed => read_agilent_processed_1d_dir(path),
        Spectrum1DPathFormat::AgilentFid => read_agilent_fid_1d_dir(path),
    }
}

/// Reads a two-dimensional spectrum from an auto-detected path.
///
/// # Errors
///
/// Returns an error when the format cannot be detected or the selected parser
/// rejects the path contents.
pub fn read_spectrum2d_path(path: impl AsRef<Path>) -> Result<Spectrum2D> {
    let path = path.as_ref();
    read_spectrum2d_path_as(path, detect_spectrum2d_path_format(path)?)
}

/// Reads a two-dimensional spectrum from a path using an explicit format.
///
/// # Errors
///
/// Returns an error when the selected reader rejects the path contents.
pub fn read_spectrum2d_path_as(
    path: impl AsRef<Path>,
    format: Spectrum2DPathFormat,
) -> Result<Spectrum2D> {
    let path = path.as_ref();
    match format {
        Spectrum2DPathFormat::Json => read_spectrum2d_json(&read_text_file(path)?),
        Spectrum2DPathFormat::NmrMl => read_nmrml_2d_str(&read_text_file(path)?),
        Spectrum2DPathFormat::Csv => read_spectrum2d_csv(&read_text_file(path)?),
        Spectrum2DPathFormat::JeolJdf => read_jeol_jdf_2d_file(path),
        Spectrum2DPathFormat::BrukerProcessed => read_bruker_processed_2d_dir(path),
        Spectrum2DPathFormat::BrukerSer => read_bruker_ser_2d_dir(path),
        Spectrum2DPathFormat::AgilentProcessed => read_agilent_processed_2d_dir(path),
        Spectrum2DPathFormat::AgilentFid => read_agilent_fid_2d_dir(path),
    }
}

/// Writes a one-dimensional spectrum to JSON, nmrML, JCAMP-DX, or CSV by extension.
///
/// # Errors
///
/// Returns an error when the extension is unsupported, the spectrum cannot be
/// represented by the selected writer, or the file cannot be written.
pub fn write_spectrum1d_path(spectrum: &Spectrum1D, path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    write_spectrum1d_path_as(spectrum, path, detect_spectrum1d_write_path_format(path)?)
}

/// Writes a one-dimensional spectrum to a path using an explicit text format.
///
/// # Errors
///
/// Returns an error when the spectrum cannot be represented by the selected
/// writer or the file cannot be written.
pub fn write_spectrum1d_path_as(
    spectrum: &Spectrum1D,
    path: impl AsRef<Path>,
    format: Spectrum1DWritePathFormat,
) -> Result<()> {
    let path = path.as_ref();
    let payload = write_spectrum1d_text(spectrum, format.into())?;
    write_text_file(path, &payload)
}

/// Writes a two-dimensional spectrum to JSON, nmrML, or CSV by extension.
///
/// # Errors
///
/// Returns an error when the extension is unsupported, the spectrum cannot be
/// represented by the selected writer, or the file cannot be written.
pub fn write_spectrum2d_path(spectrum: &Spectrum2D, path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    write_spectrum2d_path_as(spectrum, path, detect_spectrum2d_write_path_format(path)?)
}

/// Writes a two-dimensional spectrum to a path using an explicit text format.
///
/// # Errors
///
/// Returns an error when the spectrum cannot be represented by the selected
/// writer or the file cannot be written.
pub fn write_spectrum2d_path_as(
    spectrum: &Spectrum2D,
    path: impl AsRef<Path>,
    format: Spectrum2DWritePathFormat,
) -> Result<()> {
    let path = path.as_ref();
    let payload = write_spectrum2d_text(spectrum, format.into())?;
    write_text_file(path, &payload)
}

fn looks_like_nmrml(input: &str) -> bool {
    (input.starts_with("<nmrML") || input.starts_with("<?xml"))
        && contains_ascii_case_insensitive(input, "<nmrML")
}

fn looks_like_jcamp_dx(input: &str) -> bool {
    contains_ascii_case_insensitive(input, "##TITLE=")
        || contains_ascii_case_insensitive(input, "##JCAMP-DX=")
        || contains_ascii_case_insensitive(input, "##XYDATA=")
        || contains_ascii_case_insensitive(input, "##DATA TABLE=")
}

fn looks_like_csv(input: &str) -> bool {
    input
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .any(|line| line.contains(','))
}

fn contains_ascii_case_insensitive(input: &str, needle: &str) -> bool {
    let needle = needle.as_bytes();
    input
        .as_bytes()
        .windows(needle.len())
        .any(|window| window.eq_ignore_ascii_case(needle))
}

fn normalized_format_name(input: &str) -> String {
    input
        .trim()
        .chars()
        .filter(|character| !matches!(character, '_' | '-' | ' ' | '.'))
        .flat_map(char::to_lowercase)
        .collect()
}

fn text_format_from_path(path: &Path) -> Result<SpectrumTextFormat> {
    if is_extension(path, &["json"]) {
        return Ok(SpectrumTextFormat::Json);
    }
    if is_extension(path, &["nmrml", "xml"]) {
        return Ok(SpectrumTextFormat::NmrMl);
    }
    if is_extension(path, &["jdx", "dx"]) {
        return Ok(SpectrumTextFormat::JcampDx);
    }
    if is_extension(path, &["csv"]) {
        return Ok(SpectrumTextFormat::Csv);
    }
    detect_spectrum_text_format(&read_text_file(path)?)
}

fn ensure_path_exists(path: &Path) -> Result<()> {
    if path.exists() {
        Ok(())
    } else {
        Err(RSpinError::Parse {
            format: "spectrum path",
            message: format!("{} does not exist", path.display()),
        })
    }
}

fn read_text_file(path: &Path) -> Result<String> {
    fs::read_to_string(path).map_err(|error| RSpinError::Parse {
        format: "text spectrum file",
        message: format!("failed to read {}: {error}", path.display()),
    })
}

fn write_text_file(path: &Path, payload: &str) -> Result<()> {
    fs::write(path, payload).map_err(|error| RSpinError::Parse {
        format: "text spectrum file",
        message: format!("failed to write {}: {error}", path.display()),
    })
}

fn looks_like_bruker_processed_1d(path: &Path) -> bool {
    let processed = processed_dir(path);
    processed.join("procs").is_file() && processed.join("1r").is_file()
}

fn looks_like_bruker_processed_2d(path: &Path) -> bool {
    let processed = processed_dir(path);
    processed.join("procs").is_file()
        && processed.join("proc2s").is_file()
        && processed.join("2rr").is_file()
}

fn looks_like_bruker_fid(path: &Path) -> bool {
    let dataset = dataset_dir(path);
    dataset.join("fid").is_file() && dataset.join("acqus").is_file()
}

fn looks_like_bruker_ser(path: &Path) -> bool {
    let dataset = dataset_dir(path);
    dataset.join("ser").is_file()
        && dataset.join("acqus").is_file()
        && dataset.join("acqu2s").is_file()
}

fn looks_like_agilent_fid(path: &Path) -> bool {
    let dataset = dataset_dir(path);
    dataset.join("fid").is_file() && dataset.join("procpar").is_file()
}

fn looks_like_agilent_processed_1d(path: &Path) -> bool {
    looks_like_agilent_processed(path)
}

fn looks_like_agilent_processed(path: &Path) -> bool {
    if path.is_file() {
        return path
            .file_name()
            .and_then(std::ffi::OsStr::to_str)
            .is_some_and(|name| name.eq_ignore_ascii_case("phasefile"))
            && agilent_phasefile_procpar_candidates(path)
                .iter()
                .any(|candidate| candidate.is_file());
    }

    (path.join("datdir").join("phasefile").is_file() || path.join("phasefile").is_file())
        && path.join("procpar").is_file()
}

fn agilent_phasefile_procpar_candidates(path: &Path) -> [std::path::PathBuf; 2] {
    [
        path.parent()
            .and_then(Path::parent)
            .map_or_else(std::path::PathBuf::new, |parent| parent.join("procpar")),
        path.parent()
            .map_or_else(std::path::PathBuf::new, |parent| parent.join("procpar")),
    ]
}

fn processed_dir(path: &Path) -> std::path::PathBuf {
    if path.join("pdata").join("1").is_dir() {
        path.join("pdata").join("1")
    } else if path.is_file() {
        path.parent()
            .map_or_else(std::path::PathBuf::new, Path::to_path_buf)
    } else {
        path.to_path_buf()
    }
}

fn dataset_dir(path: &Path) -> std::path::PathBuf {
    if path.is_file() {
        path.parent()
            .map_or_else(std::path::PathBuf::new, Path::to_path_buf)
    } else {
        path.to_path_buf()
    }
}

fn is_extension(path: &Path, candidates: &[&str]) -> bool {
    path.extension()
        .and_then(std::ffi::OsStr::to_str)
        .is_some_and(|extension| {
            candidates
                .iter()
                .any(|candidate| extension.eq_ignore_ascii_case(candidate))
        })
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use rspin_core::{Axis, Metadata, Unit};

    use super::*;
    use crate::{write_spectrum1d_json, write_spectrum2d_json};

    #[test]
    fn detects_text_formats() -> Result<()> {
        assert_eq!(
            detect_spectrum_text_format(r#"{"x":{"label":"x"}}"#)?,
            SpectrumTextFormat::Json
        );
        assert_eq!(
            detect_spectrum_text_format(
                r#"<?xml version="1.0"?><nmrML version="v1.0.rc1"></nmrML>"#
            )?,
            SpectrumTextFormat::NmrMl
        );
        assert_eq!(
            detect_spectrum_text_format("##TITLE=demo\n##XYDATA=(X++(Y..Y))\n")?,
            SpectrumTextFormat::JcampDx
        );
        assert_eq!(
            detect_spectrum_text_format("# name=demo\nx,intensity\n0,1\n")?,
            SpectrumTextFormat::Csv
        );
        Ok(())
    }

    #[test]
    fn parses_and_displays_format_names() -> Result<()> {
        assert_eq!(
            "JSON".parse::<SpectrumTextFormat>()?,
            SpectrumTextFormat::Json
        );
        assert_eq!(
            parse_spectrum_text_format("jcamp-dx")?,
            SpectrumTextFormat::JcampDx
        );
        assert_eq!(SpectrumTextFormat::JcampDx.as_str(), "jcamp_dx");
        assert_eq!(SpectrumTextFormat::NmrMl.to_string(), "nmrml");

        assert_eq!(
            "bruker-pdata".parse::<Spectrum1DPathFormat>()?,
            Spectrum1DPathFormat::BrukerProcessed
        );
        assert_eq!(
            parse_spectrum1d_path_format("varian phasefile")?,
            Spectrum1DPathFormat::AgilentProcessed
        );
        assert_eq!(Spectrum1DPathFormat::JeolJdf.to_string(), "jeol_jdf");

        assert_eq!(
            "ser".parse::<Spectrum2DPathFormat>()?,
            Spectrum2DPathFormat::BrukerSer
        );
        assert_eq!(
            parse_spectrum2d_path_format("agilent_fid")?,
            Spectrum2DPathFormat::AgilentFid
        );
        assert_eq!(
            Spectrum2DPathFormat::AgilentProcessed.as_str(),
            "agilent_processed"
        );

        assert_eq!(
            "jdx".parse::<Spectrum1DWritePathFormat>()?,
            Spectrum1DWritePathFormat::JcampDx
        );
        assert_eq!(
            parse_spectrum2d_write_path_format("xml")?,
            Spectrum2DWritePathFormat::NmrMl
        );
        assert_eq!(Spectrum1DWritePathFormat::Csv.to_string(), "csv");
        assert_eq!(Spectrum2DWritePathFormat::NmrMl.as_str(), "nmrml");

        let error = parse_spectrum2d_path_format("jcamp_dx")
            .expect_err("2D JCAMP-DX path format should not parse");
        assert!(matches!(error, RSpinError::Unsupported { .. }));

        Ok(())
    }

    #[test]
    fn routes_explicit_text_and_path_formats() -> anyhow::Result<()> {
        let root = temp_dir("explicit-format-routing")?;
        let one = Spectrum1D::new(
            Axis::linear("shift", Unit::Ppm, 10.0, 8.0, 3)?,
            vec![1.0, -2.0, 3.0],
            Metadata::named("explicit one"),
        )?;
        let one_json = write_spectrum1d_json(&one)?;
        assert_eq!(
            read_spectrum1d_text_as(&one_json, SpectrumTextFormat::Json)?,
            one
        );

        let one_csv_path = root.join("one.payload");
        AutoSpectrum1DPathWriter.write_path_as(
            &one,
            &one_csv_path,
            Spectrum1DWritePathFormat::Csv,
        )?;
        let parsed_one =
            AutoSpectrum1DPath.read_path_as(&one_csv_path, Spectrum1DPathFormat::Csv)?;
        assert_eq!(parsed_one.x.unit, one.x.unit);
        assert_eq!(parsed_one.x.values, one.x.values);
        assert_eq!(parsed_one.intensities, one.intensities);
        assert_eq!(parsed_one.metadata.name, one.metadata.name);

        let two = Spectrum2D::new(
            Axis::linear("x", Unit::Ppm, 0.0, 1.0, 2)?,
            Axis::linear("y", Unit::Ppm, 10.0, 11.0, 2)?,
            vec![1.0, 2.0, 3.0, 4.0],
            Metadata::named("explicit two"),
        )?;
        let two_json = write_spectrum2d_json(&two)?;
        assert_eq!(
            read_spectrum2d_text_as(&two_json, SpectrumTextFormat::Json)?,
            two
        );

        let two_json_path = root.join("two.payload");
        write_spectrum2d_path_as(&two, &two_json_path, Spectrum2DWritePathFormat::Json)?;
        assert_eq!(
            read_spectrum2d_path_as(&two_json_path, Spectrum2DPathFormat::Json)?,
            two
        );

        let error = read_spectrum2d_text_as("##TITLE=demo\n", SpectrumTextFormat::JcampDx)
            .expect_err("2D JCAMP-DX text routing should fail");
        assert!(matches!(error, RSpinError::Unsupported { .. }));

        remove_dir(root)?;
        Ok(())
    }

    #[test]
    fn reads_json_1d_text() -> Result<()> {
        let spectrum = Spectrum1D::new(
            Axis::linear("shift", Unit::Ppm, 0.0, 2.0, 3)?,
            vec![1.0, 2.0, 3.0],
            Metadata::named("one"),
        )?;
        let text = write_spectrum1d_json(&spectrum)?;
        let parsed = read_spectrum1d_text(&text)?;
        assert_eq!(parsed, spectrum);
        Ok(())
    }

    #[test]
    fn reads_csv_1d_text_with_trait_api() -> Result<()> {
        let input = "# name=csv\n# x_unit=ppm\nx,intensity\n0.0,1.0\n1.0,2.0\n";
        let parsed = SpectrumReader::read_str(&AutoSpectrum1DText, input)?;
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed.metadata.name.as_deref(), Some("csv"));
        assert_eq!(parsed.intensities, vec![1.0, 2.0]);
        Ok(())
    }

    #[test]
    fn reads_nmrml_2d_text() -> Result<()> {
        let input = r#"
            <?xml version="1.0" encoding="UTF-8"?>
            <nmrML version="v1.0.rc1">
              <acquisition>
                <acquisitionMultiD>
                  <acquisitionParameterSet>
                    <directDimensionParameterSet decoupled="false" numberOfDataPoints="2"/>
                    <indirectDimensionParameterSet decoupled="false" numberOfDataPoints="1"/>
                  </acquisitionParameterSet>
                </acquisitionMultiD>
              </acquisition>
              <spectrumList>
                <spectrumMultiD id="processed" numberOfDataPoints="2">
                  <spectrumDataArray compressed="false" encodedLength="24" byteFormat="float64">
                    AAAAAAAA8D8AAAAAAAAAQA==
                  </spectrumDataArray>
                  <xAxis unitName="parts per million" startValue="1.0" endValue="0.0"/>
                  <firstDimensionProcessingParameterSet/>
                  <higherDimensionProcessingParameterSet/>
                </spectrumMultiD>
              </spectrumList>
            </nmrML>
        "#;

        let parsed = read_spectrum2d_text(input)?;
        assert_eq!(parsed.shape(), (2, 1));
        assert_eq!(parsed.z, vec![1.0, 2.0]);
        Ok(())
    }

    #[test]
    fn reads_json_2d_text_with_trait_api() -> Result<()> {
        let spectrum = Spectrum2D::new(
            Axis::linear("x", Unit::Ppm, 0.0, 1.0, 2)?,
            Axis::linear("y", Unit::Ppm, 10.0, 10.0, 1)?,
            vec![1.0, 2.0],
            Metadata::named("two"),
        )?;
        let text = write_spectrum2d_json(&spectrum)?;
        let parsed = SpectrumReader::read_str(&AutoSpectrum2DText, &text)?;
        assert_eq!(parsed, spectrum);
        Ok(())
    }

    #[test]
    fn rejects_empty_text() {
        let error = detect_spectrum_text_format(" \n\t").expect_err("empty input should fail");
        assert!(matches!(error, RSpinError::Parse { .. }));
    }

    #[test]
    fn rejects_jcamp_as_2d_text() {
        let error = read_spectrum2d_text("##TITLE=demo\n##XYDATA=(X++(Y..Y))\n")
            .expect_err("2D JCAMP text is not supported");
        assert!(matches!(error, RSpinError::Unsupported { .. }));
    }

    #[test]
    fn reads_csv_1d_path_with_trait_api() -> anyhow::Result<()> {
        let root = temp_dir("csv-1d")?;
        let path = root.join("one.csv");
        fs::write(
            &path,
            "\
# name=path one
# x_unit=PPM
x,intensity
0,1
1,2
",
        )?;

        assert_eq!(
            detect_spectrum1d_path_format(&path)?,
            Spectrum1DPathFormat::Csv
        );
        let parsed = SpectrumPathReader::read_path(&AutoSpectrum1DPath, &path)?;

        assert_eq!(parsed.metadata.name.as_deref(), Some("path one"));
        assert_eq!(parsed.x.unit, Unit::Ppm);
        assert_eq!(parsed.intensities, vec![1.0, 2.0]);

        remove_dir(root)?;
        Ok(())
    }

    #[test]
    fn reads_json_2d_path_by_content() -> anyhow::Result<()> {
        let root = temp_dir("json-2d")?;
        let path = root.join("two.spectrum");
        let spectrum = Spectrum2D::new(
            Axis::linear("x", Unit::Ppm, 0.0, 1.0, 2)?,
            Axis::linear("y", Unit::Ppm, 10.0, 10.0, 1)?,
            vec![1.0, 2.0],
            Metadata::named("path two"),
        )?;
        fs::write(&path, write_spectrum2d_json(&spectrum)?)?;

        assert_eq!(
            detect_spectrum2d_path_format(&path)?,
            Spectrum2DPathFormat::Json
        );
        let parsed = AutoSpectrum2DPath.read_path(&path)?;

        assert_eq!(parsed, spectrum);

        remove_dir(root)?;
        Ok(())
    }

    #[test]
    fn writes_1d_paths_by_extension() -> anyhow::Result<()> {
        let root = temp_dir("write-1d")?;
        let spectrum = Spectrum1D::new(
            Axis::linear("x", Unit::Ppm, 0.0, 2.0, 3)?,
            vec![1.0, 2.0, 3.0],
            Metadata::named("auto write one"),
        )?;

        let json_path = root.join("one.json");
        let csv_path = root.join("one.csv");
        let nmrml_path = root.join("one.nmrml");
        let jcamp_path = root.join("one.jdx");

        assert_eq!(
            detect_spectrum1d_write_path_format(&json_path)?,
            Spectrum1DWritePathFormat::Json
        );
        assert_eq!(
            detect_spectrum1d_write_path_format(&csv_path)?,
            Spectrum1DWritePathFormat::Csv
        );
        assert_eq!(
            detect_spectrum1d_write_path_format(&nmrml_path)?,
            Spectrum1DWritePathFormat::NmrMl
        );
        assert_eq!(
            detect_spectrum1d_write_path_format(&jcamp_path)?,
            Spectrum1DWritePathFormat::JcampDx
        );

        AutoSpectrum1DPathWriter.write_path(&spectrum, &json_path)?;
        write_spectrum1d_path(&spectrum, &csv_path)?;
        write_spectrum1d_path(&spectrum, &nmrml_path)?;
        write_spectrum1d_path(&spectrum, &jcamp_path)?;

        assert_eq!(read_spectrum1d_path(&json_path)?, spectrum);

        let csv = read_spectrum1d_path(&csv_path)?;
        assert_eq!(csv.x, spectrum.x);
        assert_eq!(csv.intensities, spectrum.intensities);
        assert_eq!(csv.metadata.name, spectrum.metadata.name);

        let nmrml = read_spectrum1d_path(&nmrml_path)?;
        assert_eq!(nmrml.x.unit, spectrum.x.unit);
        assert_eq!(nmrml.x.values, spectrum.x.values);
        assert_eq!(nmrml.intensities, spectrum.intensities);
        assert_eq!(nmrml.metadata.name, spectrum.metadata.name);

        let jcamp = read_spectrum1d_path(&jcamp_path)?;
        assert_eq!(jcamp.x, spectrum.x);
        assert_eq!(jcamp.intensities, spectrum.intensities);
        assert_eq!(jcamp.metadata.name, spectrum.metadata.name);

        remove_dir(root)?;
        Ok(())
    }

    #[test]
    fn writes_2d_paths_by_extension() -> anyhow::Result<()> {
        let root = temp_dir("write-2d")?;
        let spectrum = Spectrum2D::new(
            Axis::linear("x", Unit::Ppm, 0.0, 1.0, 2)?,
            Axis::linear("y", Unit::Ppm, 10.0, 11.0, 2)?,
            vec![1.0, 2.0, 3.0, 4.0],
            Metadata::named("auto write two"),
        )?;

        let json_path = root.join("two.json");
        let csv_path = root.join("two.csv");
        let nmrml_path = root.join("two.nmrml");

        assert_eq!(
            detect_spectrum2d_write_path_format(&json_path)?,
            Spectrum2DWritePathFormat::Json
        );
        assert_eq!(
            detect_spectrum2d_write_path_format(&csv_path)?,
            Spectrum2DWritePathFormat::Csv
        );
        assert_eq!(
            detect_spectrum2d_write_path_format(&nmrml_path)?,
            Spectrum2DWritePathFormat::NmrMl
        );

        AutoSpectrum2DPathWriter.write_path(&spectrum, &json_path)?;
        write_spectrum2d_path(&spectrum, &csv_path)?;
        write_spectrum2d_path(&spectrum, &nmrml_path)?;

        assert_eq!(read_spectrum2d_path(&json_path)?, spectrum);

        let csv = read_spectrum2d_path(&csv_path)?;
        assert_eq!(csv.x, spectrum.x);
        assert_eq!(csv.y, spectrum.y);
        assert_eq!(csv.z, spectrum.z);
        assert_eq!(csv.metadata.name, spectrum.metadata.name);

        let nmrml = read_spectrum2d_path(&nmrml_path)?;
        assert_eq!(nmrml.x.unit, spectrum.x.unit);
        assert_eq!(nmrml.x.values, spectrum.x.values);
        assert_eq!(nmrml.y.unit, spectrum.y.unit);
        assert_eq!(nmrml.y.values, spectrum.y.values);
        assert_eq!(nmrml.z, spectrum.z);
        assert_eq!(nmrml.metadata.name, spectrum.metadata.name);

        remove_dir(root)?;
        Ok(())
    }

    #[test]
    fn rejects_unsupported_write_path_extensions() {
        let error = detect_spectrum1d_write_path_format("one.bin")
            .expect_err("unsupported 1D write extension should fail");
        assert!(matches!(error, RSpinError::Unsupported { .. }));

        let error = detect_spectrum2d_write_path_format("two.dx")
            .expect_err("2D JCAMP-DX path writer should not be supported");
        assert!(matches!(error, RSpinError::Unsupported { .. }));
    }

    #[test]
    fn reads_bruker_processed_1d_path() -> anyhow::Result<()> {
        let root = temp_dir("bruker-1d")?;
        let processed = root.join("pdata/1");
        fs::create_dir_all(&processed)?;
        fs::write(
            processed.join("procs"),
            "\
##$SI= 3
##$BYTORDP= 0
##$DTYPP= 0
##$NC_proc= 0
",
        )?;
        fs::write(processed.join("1r"), i32_bytes(&[1, -2, 3]))?;

        assert_eq!(
            detect_spectrum1d_path_format(&root)?,
            Spectrum1DPathFormat::BrukerProcessed
        );
        let parsed = read_spectrum1d_path(&root)?;

        assert_eq!(parsed.x.unit, Unit::Points);
        assert_eq!(parsed.x.values, vec![0.0, 1.0, 2.0]);
        assert_eq!(parsed.intensities, vec![1.0, -2.0, 3.0]);

        remove_dir(root)?;
        Ok(())
    }

    #[test]
    fn reads_bruker_processed_2d_path() -> anyhow::Result<()> {
        let root = temp_dir("bruker-2d")?;
        let processed = root.join("pdata/1");
        fs::create_dir_all(&processed)?;
        fs::write(
            processed.join("procs"),
            "\
##$SI= 2
##$BYTORDP= 0
##$DTYPP= 0
##$NC_proc= 0
",
        )?;
        fs::write(
            processed.join("proc2s"),
            "\
##$SI= 2
",
        )?;
        fs::write(processed.join("2rr"), i32_bytes(&[1, 2, 3, 4]))?;

        assert_eq!(
            detect_spectrum2d_path_format(&root)?,
            Spectrum2DPathFormat::BrukerProcessed
        );
        let parsed = read_spectrum2d_path(&root)?;

        assert_eq!(parsed.shape(), (2, 2));
        assert_eq!(parsed.x.unit, Unit::Points);
        assert_eq!(parsed.y.unit, Unit::Points);
        assert_eq!(parsed.z, vec![1.0, 2.0, 3.0, 4.0]);

        remove_dir(root)?;
        Ok(())
    }

    #[test]
    fn reads_agilent_processed_1d_path() -> anyhow::Result<()> {
        let root = temp_dir("agilent-processed-1d")?;
        fs::create_dir_all(root.join("datdir"))?;
        fs::write(
            root.join("procpar"),
            "\
sw 1 1 5 5 5 2 1 8203 1 64
1 1000
0
sfrq 1 1 1000000000 0 0 2 1 11 1 64
1 500
0
rfl 1 1 1000000000 -1000000000 0 2 1 11 1 64
1 500
0
rfp 1 1 1000000000 -1000000000 0 2 1 11 1 64
1 0
0
",
        )?;
        fs::write(
            root.join("datdir/phasefile"),
            agilent_phasefile_i32_le(&[4, -2])?,
        )?;

        assert_eq!(
            detect_spectrum1d_path_format(&root)?,
            Spectrum1DPathFormat::AgilentProcessed
        );
        let parsed = read_spectrum1d_path(root.join("datdir/phasefile"))?;

        assert_eq!(parsed.x.unit, Unit::Ppm);
        assert_eq!(parsed.x.values, vec![1.0, -1.0]);
        assert_eq!(parsed.intensities, vec![4.0, -2.0]);

        remove_dir(root)?;
        Ok(())
    }

    #[test]
    fn reads_agilent_processed_2d_path() -> anyhow::Result<()> {
        let root = temp_dir("agilent-processed-2d")?;
        fs::create_dir_all(root.join("datdir"))?;
        fs::write(
            root.join("procpar"),
            "\
acqdim 7 1 32767 0 0 2 1 0 1 64
1 2
0
sw 1 1 5 5 5 2 1 8203 1 64
1 1000
0
sfrq 1 1 1000000000 0 0 2 1 11 1 64
1 500
0
rfl 1 1 1000000000 -1000000000 0 2 1 11 1 64
1 500
0
rfp 1 1 1000000000 -1000000000 0 2 1 11 1 64
1 0
0
sw1 1 1 5000000 1 -1.25e-08 2 1 0 1 64
1 400
0
dfrq 1 1 1000000000 0 0 2 1 11 1 64
1 100
0
rfl1 1 1 1000000000 -1000000000 0 2 1 11 1 64
1 300
0
rfp1 1 1 1000000000 -1000000000 0 2 1 11 1 64
1 100
0
",
        )?;
        fs::write(
            root.join("datdir/phasefile"),
            agilent_phasefile_i32_le_matrix(&[1, 2, 3, 4], 2, 1)?,
        )?;

        assert_eq!(
            detect_spectrum2d_path_format(&root)?,
            Spectrum2DPathFormat::AgilentProcessed
        );
        let parsed = read_spectrum2d_path(root.join("datdir/phasefile"))?;

        assert_eq!(parsed.shape(), (2, 2));
        assert_eq!(parsed.x.unit, Unit::Ppm);
        assert_eq!(parsed.x.values, vec![1.0, -1.0]);
        assert_eq!(parsed.y.unit, Unit::Ppm);
        assert_eq!(parsed.y.values, vec![2.0, -2.0]);
        assert_eq!(parsed.z, vec![1.0, 2.0, 3.0, 4.0]);

        remove_dir(root)?;
        Ok(())
    }

    fn temp_dir(name: &str) -> anyhow::Result<PathBuf> {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let mut path = std::env::temp_dir();
        path.push(format!("rspin-auto-{name}-{}-{nanos}", std::process::id()));
        fs::create_dir_all(&path)?;
        Ok(path)
    }

    fn remove_dir(path: PathBuf) -> anyhow::Result<()> {
        fs::remove_dir_all(path)?;
        Ok(())
    }

    fn i32_bytes(values: &[i32]) -> Vec<u8> {
        values
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect::<Vec<_>>()
    }

    fn agilent_phasefile_i32_le(values: &[i32]) -> anyhow::Result<Vec<u8>> {
        agilent_phasefile_i32_le_matrix(values, 1, 1)
    }

    fn agilent_phasefile_i32_le_matrix(
        values: &[i32],
        nblocks: i32,
        ntraces: i32,
    ) -> anyhow::Result<Vec<u8>> {
        let row_count = usize::try_from(nblocks)?
            .checked_mul(usize::try_from(ntraces)?)
            .ok_or_else(|| anyhow::anyhow!("synthetic phasefile row count overflow"))?;
        let trace_value_count = values.len() / row_count;
        let tbytes = trace_value_count
            .checked_mul(4)
            .ok_or_else(|| anyhow::anyhow!("synthetic phasefile trace size overflow"))?;
        let block_data_len = usize::try_from(ntraces)?
            .checked_mul(tbytes)
            .ok_or_else(|| anyhow::anyhow!("synthetic phasefile block data size overflow"))?;
        let bbytes = 28_usize
            .checked_add(block_data_len)
            .ok_or_else(|| anyhow::anyhow!("synthetic phasefile block size overflow"))?;
        let mut payload = Vec::new();
        push_i32_le(&mut payload, nblocks);
        push_i32_le(&mut payload, ntraces);
        push_i32_le(&mut payload, i32::try_from(trace_value_count)?);
        push_i32_le(&mut payload, 4);
        push_i32_le(&mut payload, i32::try_from(tbytes)?);
        push_i32_le(&mut payload, i32::try_from(bbytes)?);
        push_i16_le(&mut payload, 0);
        push_i16_le(&mut payload, 0x0001 | 0x0004);
        push_i32_le(&mut payload, 1);

        for block_index in 0..usize::try_from(nblocks)? {
            push_i16_le(&mut payload, 0);
            push_i16_le(&mut payload, 0x0001 | 0x0004);
            push_i16_le(&mut payload, 1);
            push_i16_le(&mut payload, 0);
            push_i32_le(&mut payload, i32::try_from(block_index + 1)?);
            for _ in 0..4 {
                payload.extend(0.0_f32.to_le_bytes());
            }
            let value_start = block_index
                .checked_mul(usize::try_from(ntraces)?)
                .and_then(|index| index.checked_mul(trace_value_count))
                .ok_or_else(|| anyhow::anyhow!("synthetic phasefile value offset overflow"))?;
            let value_end = value_start
                .checked_add(block_data_len / 4)
                .ok_or_else(|| anyhow::anyhow!("synthetic phasefile value end overflow"))?;
            for value in &values[value_start..value_end] {
                payload.extend(value.to_le_bytes());
            }
        }
        Ok(payload)
    }

    fn push_i32_le(bytes: &mut Vec<u8>, value: i32) {
        bytes.extend(value.to_le_bytes());
    }

    fn push_i16_le(bytes: &mut Vec<u8>, value: i16) {
        bytes.extend(value.to_le_bytes());
    }
}
