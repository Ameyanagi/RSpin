//! nmrML one-dimensional spectrum export.

use std::{fmt::Write as _, fs, path::Path};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use rspin_core::{RSpinError, Result, Spectrum1D, Unit};

use crate::{SpectrumWriter, nmrml::NmrMl1D};

const FORMAT: &str = "nmrML";
const VERSION: &str = "v1.0.rc1";
const NAMESPACE: &str = "http://nmrml.org/schema";

impl NmrMl1D {
    /// Writes a one-dimensional spectrum to nmrML text.
    ///
    /// # Errors
    ///
    /// Returns an error when the spectrum contains non-finite values or cannot
    /// yet be represented by `RSpin`'s focused nmrML writer.
    pub fn write_string(self, spectrum: &Spectrum1D) -> Result<String> {
        write_nmrml_1d(spectrum)
    }

    /// Writes a one-dimensional spectrum to an nmrML file.
    ///
    /// # Errors
    ///
    /// Returns an error when the spectrum cannot be represented or the file
    /// cannot be written.
    pub fn write_file(self, spectrum: &Spectrum1D, path: impl AsRef<Path>) -> Result<()> {
        write_nmrml_1d_file(spectrum, path)
    }
}

impl SpectrumWriter<Spectrum1D> for NmrMl1D {
    fn write_string(&self, spectrum: &Spectrum1D) -> Result<String> {
        write_nmrml_1d(spectrum)
    }
}

/// Writes a one-dimensional spectrum to nmrML text.
///
/// Uniform real spectra are emitted as little-endian `float64` y-value arrays.
/// Non-uniform real spectra are emitted as little-endian `complex128` x/y
/// pairs so coordinates round-trip without resampling.
///
/// # Errors
///
/// Returns an error when the spectrum contains non-finite values or cannot yet
/// be represented by `RSpin`'s focused nmrML writer.
pub fn write_nmrml_1d(spectrum: &Spectrum1D) -> Result<String> {
    validate_exportable(spectrum)?;

    let mut output = String::new();
    output.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    let _ = writeln!(
        output,
        "<nmrML version=\"{VERSION}\" xmlns=\"{NAMESPACE}\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xsi:schemaLocation=\"{NAMESPACE} nmrML.xsd\">"
    );
    write_acquisition(&mut output, spectrum);
    write_spectrum(&mut output, spectrum)?;
    output.push_str("</nmrML>\n");
    Ok(output)
}

/// Writes a one-dimensional spectrum to an nmrML file.
///
/// # Errors
///
/// Returns an error when the spectrum cannot be represented or the file cannot
/// be written.
pub fn write_nmrml_1d_file(spectrum: &Spectrum1D, path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    let payload = write_nmrml_1d(spectrum)?;
    fs::write(path, payload).map_err(|error| RSpinError::Parse {
        format: FORMAT,
        message: format!("failed to write {}: {error}", path.display()),
    })
}

fn validate_exportable(spectrum: &Spectrum1D) -> Result<()> {
    if spectrum.imaginary.is_some() {
        return Err(RSpinError::Unsupported {
            feature: "complex nmrML 1D spectrum export",
        });
    }
    if !spectrum.x.values.iter().all(|value| value.is_finite())
        || !spectrum.intensities.iter().all(|value| value.is_finite())
    {
        return Err(RSpinError::NonFinite { field: "spectrum" });
    }
    if !spectrum.metadata.frequency_mhz.is_none_or(f64::is_finite) {
        return Err(RSpinError::NonFinite {
            field: "frequency_mhz",
        });
    }
    if !spectrum.metadata.temperature_k.is_none_or(f64::is_finite) {
        return Err(RSpinError::NonFinite {
            field: "temperature_k",
        });
    }
    Ok(())
}

fn write_acquisition(output: &mut String, spectrum: &Spectrum1D) {
    output.push_str("  <acquisition>\n");
    output.push_str("    <acquisition1D>\n");
    output.push_str("      <acquisitionParameterSet>\n");
    if let Some(temperature_k) = spectrum.metadata.temperature_k {
        let _ = writeln!(
            output,
            "        <sampleAcquisitionTemperature value=\"{}\" unitName=\"kelvin\"/>",
            format_float(temperature_k)
        );
    }
    let _ = writeln!(
        output,
        "        <DirectDimensionParameterSet decoupled=\"false\" numberOfDataPoints=\"{}\">",
        spectrum.len()
    );
    if let Some(nucleus) = &spectrum.metadata.nucleus {
        let _ = writeln!(
            output,
            "          <acquisitionNucleus name=\"{}\"/>",
            escape_xml_attr(nucleus.as_label())
        );
    }
    if let Some(frequency_mhz) = spectrum.metadata.frequency_mhz {
        let _ = writeln!(
            output,
            "          <effectiveExcitationField value=\"{}\" unitName=\"megaHertz\"/>",
            format_float(frequency_mhz)
        );
    }
    output.push_str("        </DirectDimensionParameterSet>\n");
    if let Some(solvent) = spectrum.metadata.solvent.as_deref() {
        let _ = writeln!(
            output,
            "        <solventType value=\"{}\"/>",
            escape_xml_attr(solvent)
        );
    }
    output.push_str("      </acquisitionParameterSet>\n");
    output.push_str("    </acquisition1D>\n");
    output.push_str("  </acquisition>\n");
}

fn write_spectrum(output: &mut String, spectrum: &Spectrum1D) -> Result<()> {
    output.push_str("  <spectrumList count=\"1\">\n");
    output.push_str("    <spectrum1D");
    output.push_str(" id=\"spectrum1\"");
    if let Some(name) = spectrum.metadata.name.as_deref() {
        let _ = write!(output, " name=\"{}\"", escape_xml_attr(name));
    }
    let _ = writeln!(output, " numberOfDataPoints=\"{}\">", spectrum.len());

    let binary = spectrum_binary(spectrum);
    let encoded = STANDARD.encode(&binary.bytes);
    let _ = write!(
        output,
        "      <spectrumDataArray compressed=\"false\" encodedLength=\"{}\" byteFormat=\"{}\">",
        encoded.len(),
        binary.byte_format
    );
    output.push_str(&encoded);
    output.push_str("</spectrumDataArray>\n");
    write_axis(output, spectrum)?;
    output.push_str("    </spectrum1D>\n");
    output.push_str("  </spectrumList>\n");
    Ok(())
}

struct SpectrumBinary {
    byte_format: &'static str,
    bytes: Vec<u8>,
}

fn spectrum_binary(spectrum: &Spectrum1D) -> SpectrumBinary {
    if has_uniform_spacing(&spectrum.x.values) {
        let mut bytes = Vec::with_capacity(spectrum.len() * 8);
        for value in &spectrum.intensities {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        SpectrumBinary {
            byte_format: "float64",
            bytes,
        }
    } else {
        let mut bytes = Vec::with_capacity(spectrum.len() * 16);
        for (x, intensity) in spectrum.points() {
            bytes.extend_from_slice(&x.to_le_bytes());
            bytes.extend_from_slice(&intensity.to_le_bytes());
        }
        SpectrumBinary {
            byte_format: "complex128",
            bytes,
        }
    }
}

fn write_axis(output: &mut String, spectrum: &Spectrum1D) -> Result<()> {
    let _ = write!(
        output,
        "      <xAxis unitName=\"{}\"",
        axis_unit_label(spectrum.x.unit)
    );
    if has_uniform_spacing(&spectrum.x.values) {
        let start =
            spectrum
                .x
                .values
                .first()
                .copied()
                .ok_or_else(|| RSpinError::InvalidSpectrum {
                    message: "missing x axis values".to_owned(),
                })?;
        let end = spectrum
            .x
            .values
            .last()
            .copied()
            .ok_or_else(|| RSpinError::InvalidSpectrum {
                message: "missing x axis values".to_owned(),
            })?;
        let _ = write!(
            output,
            " startValue=\"{}\" endValue=\"{}\"",
            format_float(start),
            format_float(end)
        );
    }
    output.push_str("/>\n");
    Ok(())
}

fn has_uniform_spacing(values: &[f64]) -> bool {
    if values.len() <= 2 {
        return true;
    }
    let expected_step = values[1] - values[0];
    let tolerance = (expected_step.abs().max(1.0)) * 1.0e-10;
    values
        .windows(2)
        .all(|pair| ((pair[1] - pair[0]) - expected_step).abs() <= tolerance)
}

fn axis_unit_label(unit: Unit) -> &'static str {
    match unit {
        Unit::Ppm => "parts per million",
        Unit::Hertz => "hertz",
        Unit::Seconds => "second",
        Unit::Points => "point",
        _ => "arbitrary",
    }
}

fn format_float(value: f64) -> String {
    let mut formatted = format!("{value:.15}");
    while formatted.contains('.') && formatted.ends_with('0') {
        formatted.pop();
    }
    if formatted.ends_with('.') {
        formatted.push('0');
    }
    formatted
}

fn escape_xml_attr(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use rspin_core::{Axis, Metadata, Nucleus};

    use crate::{SpectrumReader, SpectrumWriter, read_nmrml_1d_str, read_nmrml_document_info_str};

    use super::*;

    #[test]
    fn writes_uniform_float64_spectrum_round_trip() -> anyhow::Result<()> {
        let spectrum = Spectrum1D::new(
            Axis::linear_ppm(10.0, 8.0, 3)?,
            vec![1.0, -2.0, 3.5],
            Metadata::named("One & Two")
                .with_nucleus(Nucleus::Hydrogen1)
                .with_frequency_mhz(600.0)
                .with_temperature_k(298.15)
                .with_solvent("CDCl3"),
        )?;

        let text = write_nmrml_1d(&spectrum)?;
        let info = read_nmrml_document_info_str(&text)?;
        let parsed = read_nmrml_1d_str(&text)?;

        assert_eq!(info.version, VERSION);
        assert!(text.contains("byteFormat=\"float64\""));
        assert_eq!(parsed.x, spectrum.x);
        assert_eq!(parsed.intensities, spectrum.intensities);
        assert_eq!(parsed.metadata.name.as_deref(), Some("One & Two"));
        assert_eq!(parsed.metadata.nucleus, Some(Nucleus::Hydrogen1));
        assert_eq!(parsed.metadata.frequency_mhz, Some(600.0));
        assert_eq!(parsed.metadata.temperature_k, Some(298.15));
        assert_eq!(parsed.metadata.solvent.as_deref(), Some("CDCl3"));
        Ok(())
    }

    #[test]
    fn writes_non_uniform_xy_pairs_round_trip_with_trait_api() -> anyhow::Result<()> {
        let spectrum = Spectrum1D::new(
            Axis::ppm(vec![10.0, 9.5, 7.0])?,
            vec![1.0, 0.5, -2.0],
            Metadata::named("nonuniform"),
        )?;

        let text = SpectrumWriter::write_string(&NmrMl1D, &spectrum)?;
        let parsed = SpectrumReader::read_str(&NmrMl1D, &text)?;

        assert!(text.contains("byteFormat=\"complex128\""));
        assert_eq!(parsed.x, spectrum.x);
        assert_eq!(parsed.intensities, spectrum.intensities);
        Ok(())
    }

    #[test]
    fn writes_file_with_inherent_api() -> anyhow::Result<()> {
        let mut path = std::env::temp_dir();
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        path.push(format!(
            "rspin-nmrml-writer-{}-{nanos}.nmrml",
            std::process::id()
        ));
        let spectrum = Spectrum1D::new(
            Axis::linear_ppm(0.0, 1.0, 2)?,
            vec![1.0, 2.0],
            Metadata::named("file"),
        )?;

        NmrMl1D.write_file(&spectrum, &path)?;
        let parsed = read_nmrml_1d_str(&fs::read_to_string(&path)?)?;

        assert_eq!(parsed.intensities, spectrum.intensities);
        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn rejects_complex_and_non_finite_metadata() -> anyhow::Result<()> {
        let complex = Spectrum1D::new_complex(
            Axis::linear_ppm(0.0, 1.0, 2)?,
            vec![1.0, 2.0],
            Some(vec![0.1, 0.2]),
            Metadata::new(),
        )?;
        let error = write_nmrml_1d(&complex).expect_err("complex export should be explicit");
        assert!(matches!(error, RSpinError::Unsupported { .. }));

        let mut invalid = Spectrum1D::new(
            Axis::linear_ppm(0.0, 1.0, 2)?,
            vec![1.0, 2.0],
            Metadata::new().with_frequency_mhz(f64::NAN),
        )?;
        invalid.metadata.temperature_k = Some(f64::INFINITY);
        let error = write_nmrml_1d(&invalid).expect_err("non-finite metadata should be rejected");
        assert!(matches!(error, RSpinError::NonFinite { .. }));
        Ok(())
    }
}
