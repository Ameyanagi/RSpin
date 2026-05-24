//! Text spectrum writer helpers used by auto IO.

use std::str::FromStr;

use rspin_core::{RSpinError, Result, Spectrum1D, Spectrum2D};

use crate::{
    SpectrumWriter, write_jcamp_dx_1d, write_nmrml_1d, write_nmrml_2d, write_spectrum1d_csv,
    write_spectrum1d_json, write_spectrum2d_csv, write_spectrum2d_json,
};

use super::{Spectrum1DWritePathFormat, Spectrum2DWritePathFormat};

/// Text export formats supported for one-dimensional spectra.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spectrum1DWriteFormat {
    /// `RSpin` JSON spectrum payload.
    Json,
    /// nmrML XML payload.
    NmrMl,
    /// JCAMP-DX text payload.
    JcampDx,
    /// `RSpin` CSV payload.
    Csv,
}

impl FromStr for Spectrum1DWriteFormat {
    type Err = RSpinError;

    fn from_str(input: &str) -> Result<Self> {
        parse_spectrum1d_write_format(input)
    }
}

impl From<Spectrum1DWritePathFormat> for Spectrum1DWriteFormat {
    fn from(value: Spectrum1DWritePathFormat) -> Self {
        match value {
            Spectrum1DWritePathFormat::Json => Self::Json,
            Spectrum1DWritePathFormat::NmrMl => Self::NmrMl,
            Spectrum1DWritePathFormat::JcampDx => Self::JcampDx,
            Spectrum1DWritePathFormat::Csv => Self::Csv,
        }
    }
}

/// Text export formats supported for two-dimensional spectra.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spectrum2DWriteFormat {
    /// `RSpin` JSON spectrum payload.
    Json,
    /// nmrML XML payload.
    NmrMl,
    /// `RSpin` CSV payload.
    Csv,
}

impl FromStr for Spectrum2DWriteFormat {
    type Err = RSpinError;

    fn from_str(input: &str) -> Result<Self> {
        parse_spectrum2d_write_format(input)
    }
}

impl From<Spectrum2DWritePathFormat> for Spectrum2DWriteFormat {
    fn from(value: Spectrum2DWritePathFormat) -> Self {
        match value {
            Spectrum2DWritePathFormat::Json => Self::Json,
            Spectrum2DWritePathFormat::NmrMl => Self::NmrMl,
            Spectrum2DWritePathFormat::Csv => Self::Csv,
        }
    }
}

/// Writer for one-dimensional text spectra in a selected format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spectrum1DTextWriter {
    /// Text export format.
    pub format: Spectrum1DWriteFormat,
}

impl Spectrum1DTextWriter {
    /// Creates a one-dimensional text writer for `format`.
    #[must_use]
    pub fn new(format: Spectrum1DWriteFormat) -> Self {
        Self { format }
    }

    /// Creates a JSON text writer.
    #[must_use]
    pub fn json() -> Self {
        Self::new(Spectrum1DWriteFormat::Json)
    }

    /// Creates an nmrML text writer.
    #[must_use]
    pub fn nmrml() -> Self {
        Self::new(Spectrum1DWriteFormat::NmrMl)
    }

    /// Creates a JCAMP-DX text writer.
    #[must_use]
    pub fn jcamp_dx() -> Self {
        Self::new(Spectrum1DWriteFormat::JcampDx)
    }

    /// Creates a CSV text writer.
    #[must_use]
    pub fn csv() -> Self {
        Self::new(Spectrum1DWriteFormat::Csv)
    }

    /// Writes a spectrum to a string using the selected format.
    ///
    /// # Errors
    ///
    /// Returns an error when the selected writer cannot represent the spectrum.
    pub fn write_string(self, spectrum: &Spectrum1D) -> Result<String> {
        write_spectrum1d_text(spectrum, self.format)
    }
}

impl Default for Spectrum1DTextWriter {
    fn default() -> Self {
        Self::json()
    }
}

impl SpectrumWriter<Spectrum1D> for Spectrum1DTextWriter {
    fn write_string(&self, spectrum: &Spectrum1D) -> Result<String> {
        write_spectrum1d_text(spectrum, self.format)
    }
}

/// Writer for two-dimensional text spectra in a selected format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spectrum2DTextWriter {
    /// Text export format.
    pub format: Spectrum2DWriteFormat,
}

impl Spectrum2DTextWriter {
    /// Creates a two-dimensional text writer for `format`.
    #[must_use]
    pub fn new(format: Spectrum2DWriteFormat) -> Self {
        Self { format }
    }

    /// Creates a JSON text writer.
    #[must_use]
    pub fn json() -> Self {
        Self::new(Spectrum2DWriteFormat::Json)
    }

    /// Creates an nmrML text writer.
    #[must_use]
    pub fn nmrml() -> Self {
        Self::new(Spectrum2DWriteFormat::NmrMl)
    }

    /// Creates a CSV text writer.
    #[must_use]
    pub fn csv() -> Self {
        Self::new(Spectrum2DWriteFormat::Csv)
    }

    /// Writes a spectrum to a string using the selected format.
    ///
    /// # Errors
    ///
    /// Returns an error when the selected writer cannot represent the spectrum.
    pub fn write_string(self, spectrum: &Spectrum2D) -> Result<String> {
        write_spectrum2d_text(spectrum, self.format)
    }
}

impl Default for Spectrum2DTextWriter {
    fn default() -> Self {
        Self::json()
    }
}

impl SpectrumWriter<Spectrum2D> for Spectrum2DTextWriter {
    fn write_string(&self, spectrum: &Spectrum2D) -> Result<String> {
        write_spectrum2d_text(spectrum, self.format)
    }
}

/// Parses a one-dimensional text export format name.
///
/// Accepted names include `json`, `nmrml`, `xml`, `jcamp_dx`, `jdx`, `dx`, and
/// `csv`.
///
/// # Errors
///
/// Returns an unsupported-feature error when `input` is not a supported format.
pub fn parse_spectrum1d_write_format(input: &str) -> Result<Spectrum1DWriteFormat> {
    match normalized_format_name(input).as_str() {
        "json" => Ok(Spectrum1DWriteFormat::Json),
        "nmrml" | "xml" => Ok(Spectrum1DWriteFormat::NmrMl),
        "jcampdx" | "jcamp" | "jdx" | "dx" => Ok(Spectrum1DWriteFormat::JcampDx),
        "csv" => Ok(Spectrum1DWriteFormat::Csv),
        _ => Err(RSpinError::Unsupported {
            feature: "one-dimensional spectrum text writer format",
        }),
    }
}

/// Parses a two-dimensional text export format name.
///
/// Accepted names include `json`, `nmrml`, `xml`, and `csv`.
///
/// # Errors
///
/// Returns an unsupported-feature error when `input` is not a supported format.
pub fn parse_spectrum2d_write_format(input: &str) -> Result<Spectrum2DWriteFormat> {
    match normalized_format_name(input).as_str() {
        "json" => Ok(Spectrum2DWriteFormat::Json),
        "nmrml" | "xml" => Ok(Spectrum2DWriteFormat::NmrMl),
        "csv" => Ok(Spectrum2DWriteFormat::Csv),
        _ => Err(RSpinError::Unsupported {
            feature: "two-dimensional spectrum text writer format",
        }),
    }
}

/// Writes a one-dimensional spectrum to a string in the selected text format.
///
/// # Errors
///
/// Returns an error when the selected writer cannot represent the spectrum.
pub fn write_spectrum1d_text(
    spectrum: &Spectrum1D,
    format: Spectrum1DWriteFormat,
) -> Result<String> {
    match format {
        Spectrum1DWriteFormat::Json => write_spectrum1d_json(spectrum),
        Spectrum1DWriteFormat::NmrMl => write_nmrml_1d(spectrum),
        Spectrum1DWriteFormat::JcampDx => write_jcamp_dx_1d(spectrum),
        Spectrum1DWriteFormat::Csv => write_spectrum1d_csv(spectrum),
    }
}

/// Writes a two-dimensional spectrum to a string in the selected text format.
///
/// # Errors
///
/// Returns an error when the selected writer cannot represent the spectrum.
pub fn write_spectrum2d_text(
    spectrum: &Spectrum2D,
    format: Spectrum2DWriteFormat,
) -> Result<String> {
    match format {
        Spectrum2DWriteFormat::Json => write_spectrum2d_json(spectrum),
        Spectrum2DWriteFormat::NmrMl => write_nmrml_2d(spectrum),
        Spectrum2DWriteFormat::Csv => write_spectrum2d_csv(spectrum),
    }
}

fn normalized_format_name(input: &str) -> String {
    input
        .trim()
        .chars()
        .filter(|character| !matches!(character, '_' | '-' | ' ' | '.'))
        .flat_map(char::to_lowercase)
        .collect()
}

#[cfg(test)]
mod tests {
    use rspin_core::{Axis, Metadata, RSpinError, Spectrum1D, Spectrum2D, Unit};

    use super::*;
    use crate::{SpectrumReader, read_spectrum1d_text, read_spectrum2d_text};

    #[test]
    fn writes_1d_text_by_format() -> Result<()> {
        let spectrum = Spectrum1D::new(
            Axis::linear("shift", Unit::Ppm, 10.0, 8.0, 3)?,
            vec![1.0, -2.0, 3.0],
            Metadata::named("one text"),
        )?;

        assert_eq!(
            parse_spectrum1d_write_format("jcamp_dx")?,
            Spectrum1DWriteFormat::JcampDx
        );
        assert_eq!(
            "nmrML".parse::<Spectrum1DWriteFormat>()?,
            Spectrum1DWriteFormat::NmrMl
        );

        let json = Spectrum1DTextWriter::json().write_string(&spectrum)?;
        assert_eq!(read_spectrum1d_text(&json)?, spectrum);

        let csv = SpectrumWriter::write_string(&Spectrum1DTextWriter::csv(), &spectrum)?;
        let parsed_csv = read_spectrum1d_text(&csv)?;
        assert_eq!(parsed_csv.x.unit, spectrum.x.unit);
        assert_eq!(parsed_csv.x.values, spectrum.x.values);
        assert_eq!(parsed_csv.intensities, spectrum.intensities);

        let jcamp = write_spectrum1d_text(&spectrum, Spectrum1DWriteFormat::JcampDx)?;
        let parsed_jcamp = read_spectrum1d_text(&jcamp)?;
        assert_eq!(parsed_jcamp.x.values, spectrum.x.values);
        assert_eq!(parsed_jcamp.intensities, spectrum.intensities);

        let nmrml = Spectrum1DTextWriter::nmrml().write_string(&spectrum)?;
        let parsed_nmrml = read_spectrum1d_text(&nmrml)?;
        assert_eq!(parsed_nmrml.x.values, spectrum.x.values);
        assert_eq!(parsed_nmrml.intensities, spectrum.intensities);
        Ok(())
    }

    #[test]
    fn writes_2d_text_by_format() -> Result<()> {
        let spectrum = Spectrum2D::new(
            Axis::linear("x", Unit::Ppm, 0.0, 1.0, 2)?,
            Axis::linear("y", Unit::Ppm, 10.0, 11.0, 2)?,
            vec![1.0, 2.0, 3.0, 4.0],
            Metadata::named("two text"),
        )?;

        assert_eq!(
            parse_spectrum2d_write_format("xml")?,
            Spectrum2DWriteFormat::NmrMl
        );
        assert_eq!(
            "csv".parse::<Spectrum2DWriteFormat>()?,
            Spectrum2DWriteFormat::Csv
        );

        let json = Spectrum2DTextWriter::json().write_string(&spectrum)?;
        assert_eq!(read_spectrum2d_text(&json)?, spectrum);

        let csv = SpectrumWriter::write_string(&Spectrum2DTextWriter::csv(), &spectrum)?;
        let parsed_csv = read_spectrum2d_text(&csv)?;
        assert_eq!(parsed_csv.x, spectrum.x);
        assert_eq!(parsed_csv.y, spectrum.y);
        assert_eq!(parsed_csv.z, spectrum.z);

        let nmrml = write_spectrum2d_text(&spectrum, Spectrum2DWriteFormat::NmrMl)?;
        let parsed_nmrml = read_spectrum2d_text(&nmrml)?;
        assert_eq!(parsed_nmrml.x.values, spectrum.x.values);
        assert_eq!(parsed_nmrml.y.values, spectrum.y.values);
        assert_eq!(parsed_nmrml.z, spectrum.z);
        Ok(())
    }

    #[test]
    fn rejects_unsupported_text_writer_formats() {
        let error = parse_spectrum1d_write_format("binary")
            .expect_err("unsupported 1D text writer should fail");
        assert!(matches!(error, RSpinError::Unsupported { .. }));

        let error = parse_spectrum2d_write_format("jdx")
            .expect_err("2D JCAMP-DX text writer should not be supported");
        assert!(matches!(error, RSpinError::Unsupported { .. }));
    }

    #[test]
    fn implements_reader_writer_traits_for_text_formats() -> Result<()> {
        let spectrum = Spectrum1D::new(
            Axis::linear("shift", Unit::Ppm, 0.0, 1.0, 2)?,
            vec![1.0, 2.0],
            Metadata::default(),
        )?;
        let text = SpectrumWriter::write_string(&Spectrum1DTextWriter::csv(), &spectrum)?;
        let parsed = SpectrumReader::read_str(&crate::AutoSpectrum1DText, &text)?;

        assert_eq!(parsed.intensities, spectrum.intensities);
        Ok(())
    }
}
