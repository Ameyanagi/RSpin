//! Text spectrum format detection and convenience readers.

use rspin_core::{RSpinError, Result, Spectrum1D, Spectrum2D};

use crate::{
    SpectrumReader, read_jcamp_dx_1d, read_nmrml_1d_str, read_nmrml_2d_str, read_spectrum1d_csv,
    read_spectrum1d_json, read_spectrum2d_csv, read_spectrum2d_json,
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

/// Reads a one-dimensional spectrum from JSON, nmrML, JCAMP-DX, or CSV text.
///
/// # Errors
///
/// Returns an error when the format cannot be detected or the selected parser
/// rejects the payload.
pub fn read_spectrum1d_text(input: &str) -> Result<Spectrum1D> {
    match detect_spectrum_text_format(input)? {
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
    match detect_spectrum_text_format(input)? {
        SpectrumTextFormat::Json => read_spectrum2d_json(input),
        SpectrumTextFormat::NmrMl => read_nmrml_2d_str(input),
        SpectrumTextFormat::Csv => read_spectrum2d_csv(input),
        SpectrumTextFormat::JcampDx => Err(RSpinError::Unsupported {
            feature: "two-dimensional JCAMP-DX text reader",
        }),
    }
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

#[cfg(test)]
mod tests {
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
}
