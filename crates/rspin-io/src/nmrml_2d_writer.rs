//! nmrML two-dimensional spectrum export.

use std::{fmt::Write as _, fs, path::Path};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use rspin_core::{RSpinError, Result, Spectrum2D, Unit};

use crate::{SpectrumWriter, nmrml_2d::NmrMl2D};

const FORMAT: &str = "nmrML";
const VERSION: &str = "v1.0.rc1";
const NAMESPACE: &str = "http://nmrml.org/schema";

impl NmrMl2D {
    /// Writes a two-dimensional spectrum to nmrML text.
    ///
    /// # Errors
    ///
    /// Returns an error when the spectrum contains non-finite values or cannot
    /// yet be represented by `RSpin`'s focused nmrML writer.
    pub fn write_string(self, spectrum: &Spectrum2D) -> Result<String> {
        write_nmrml_2d(spectrum)
    }

    /// Writes a two-dimensional spectrum to an nmrML file.
    ///
    /// # Errors
    ///
    /// Returns an error when the spectrum cannot be represented or the file
    /// cannot be written.
    pub fn write_file(self, spectrum: &Spectrum2D, path: impl AsRef<Path>) -> Result<()> {
        write_nmrml_2d_file(spectrum, path)
    }
}

impl SpectrumWriter<Spectrum2D> for NmrMl2D {
    fn write_string(&self, spectrum: &Spectrum2D) -> Result<String> {
        write_nmrml_2d(spectrum)
    }
}

/// Writes a two-dimensional spectrum to nmrML text.
///
/// The focused writer emits processed real spectra as little-endian `float64`
/// row-major matrices with direct and first indirect dimension metadata.
///
/// # Errors
///
/// Returns an error when the spectrum contains non-finite values or cannot yet
/// be represented by `RSpin`'s focused nmrML writer.
pub fn write_nmrml_2d(spectrum: &Spectrum2D) -> Result<String> {
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

/// Writes a two-dimensional spectrum to an nmrML file.
///
/// # Errors
///
/// Returns an error when the spectrum cannot be represented or the file cannot
/// be written.
pub fn write_nmrml_2d_file(spectrum: &Spectrum2D, path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    let payload = write_nmrml_2d(spectrum)?;
    fs::write(path, payload).map_err(|error| RSpinError::Parse {
        format: FORMAT,
        message: format!("failed to write {}: {error}", path.display()),
    })
}

fn validate_exportable(spectrum: &Spectrum2D) -> Result<()> {
    let expected_len = spectrum
        .x
        .len()
        .checked_mul(spectrum.y.len())
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "2D axis size overflow".to_owned(),
        })?;
    if spectrum.z.len() != expected_len {
        return Err(RSpinError::InvalidSpectrum {
            message: format!(
                "matrix has {} values but axes require {expected_len}",
                spectrum.z.len()
            ),
        });
    }
    if spectrum.imaginary.is_some() {
        return Err(RSpinError::Unsupported {
            feature: "complex nmrML 2D spectrum export",
        });
    }
    if !has_uniform_spacing(&spectrum.x.values) || !has_uniform_spacing(&spectrum.y.values) {
        return Err(RSpinError::Unsupported {
            feature: "non-uniform nmrML 2D axis export",
        });
    }
    if !spectrum.x.values.iter().all(|value| value.is_finite())
        || !spectrum.y.values.iter().all(|value| value.is_finite())
        || !spectrum.z.iter().all(|value| value.is_finite())
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

fn write_acquisition(output: &mut String, spectrum: &Spectrum2D) {
    output.push_str("  <acquisition>\n");
    output.push_str("    <acquisitionMultiD>\n");
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
        "        <directDimensionParameterSet decoupled=\"false\" numberOfDataPoints=\"{}\">",
        spectrum.x.len()
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
    output.push_str("        </directDimensionParameterSet>\n");
    let _ = writeln!(
        output,
        "        <indirectDimensionParameterSet decoupled=\"false\" numberOfDataPoints=\"{}\"/>",
        spectrum.y.len()
    );
    if let Some(solvent) = spectrum.metadata.solvent.as_deref() {
        let _ = writeln!(
            output,
            "        <solventType value=\"{}\"/>",
            escape_xml_attr(solvent)
        );
    }
    output.push_str("      </acquisitionParameterSet>\n");
    output.push_str("    </acquisitionMultiD>\n");
    output.push_str("  </acquisition>\n");
}

fn write_spectrum(output: &mut String, spectrum: &Spectrum2D) -> Result<()> {
    output.push_str("  <spectrumList count=\"1\">\n");
    output.push_str("    <spectrumMultiD");
    output.push_str(" id=\"spectrum1\"");
    if let Some(name) = spectrum.metadata.name.as_deref() {
        let _ = write!(output, " name=\"{}\"", escape_xml_attr(name));
    }
    let _ = writeln!(output, " numberOfDataPoints=\"{}\">", spectrum.z.len());

    let encoded = STANDARD.encode(spectrum_bytes(spectrum));
    let _ = write!(
        output,
        "      <spectrumDataArray compressed=\"false\" encodedLength=\"{}\" byteFormat=\"float64\">",
        encoded.len()
    );
    output.push_str(&encoded);
    output.push_str("</spectrumDataArray>\n");
    write_axis(output, "xAxis", spectrum.x.unit, &spectrum.x.values)?;
    write_axis(output, "yAxis", spectrum.y.unit, &spectrum.y.values)?;
    output.push_str("      <firstDimensionProcessingParameterSet/>\n");
    output.push_str("      <higherDimensionProcessingParameterSet/>\n");
    output.push_str("    </spectrumMultiD>\n");
    output.push_str("  </spectrumList>\n");
    Ok(())
}

fn spectrum_bytes(spectrum: &Spectrum2D) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(spectrum.z.len() * 8);
    for value in &spectrum.z {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes
}

fn write_axis(
    output: &mut String,
    element: &'static str,
    unit: Unit,
    values: &[f64],
) -> Result<()> {
    let start = values
        .first()
        .copied()
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: format!("missing {element} values"),
        })?;
    let end = values
        .last()
        .copied()
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: format!("missing {element} values"),
        })?;
    let _ = writeln!(
        output,
        "      <{element} unitName=\"{}\" startValue=\"{}\" endValue=\"{}\"/>",
        axis_unit_label(unit),
        format_float(start),
        format_float(end)
    );
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

    use crate::{SpectrumReader, SpectrumWriter, read_nmrml_2d_str, read_nmrml_document_info_str};

    use super::*;

    #[test]
    fn writes_processed_float64_2d_spectrum_round_trip() -> anyhow::Result<()> {
        let spectrum = Spectrum2D::new(
            Axis::linear_ppm(10.0, 8.0, 3)?,
            Axis::linear_ppm(120.0, 100.0, 2)?,
            vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
            Metadata::named("Two & D")
                .with_nucleus(Nucleus::Hydrogen1)
                .with_frequency_mhz(600.0)
                .with_temperature_k(298.15)
                .with_solvent("D2O"),
        )?;

        let text = write_nmrml_2d(&spectrum)?;
        let info = read_nmrml_document_info_str(&text)?;
        let parsed = read_nmrml_2d_str(&text)?;

        assert_eq!(info.version, VERSION);
        assert!(text.contains("byteFormat=\"float64\""));
        assert_eq!(parsed.shape(), spectrum.shape());
        assert_eq!(parsed.x, spectrum.x);
        assert_eq!(parsed.y, spectrum.y);
        assert_eq!(parsed.z, spectrum.z);
        assert_eq!(parsed.metadata.name.as_deref(), Some("Two & D"));
        assert_eq!(parsed.metadata.nucleus, Some(Nucleus::Hydrogen1));
        assert_eq!(parsed.metadata.frequency_mhz, Some(600.0));
        assert_eq!(parsed.metadata.temperature_k, Some(298.15));
        assert_eq!(parsed.metadata.solvent.as_deref(), Some("D2O"));
        Ok(())
    }

    #[test]
    fn writes_with_trait_api() -> anyhow::Result<()> {
        let spectrum = Spectrum2D::new(
            Axis::linear_ppm(1.0, 3.0, 3)?,
            Axis::linear("frequency", Unit::Hertz, 10.0, 20.0, 2)?,
            vec![1.0, 0.0, 2.0, 3.0, 0.5, -1.0],
            Metadata::named("trait"),
        )?;

        let text = SpectrumWriter::write_string(&NmrMl2D, &spectrum)?;
        let parsed = SpectrumReader::read_str(&NmrMl2D, &text)?;

        assert_eq!(parsed.x, spectrum.x);
        assert_eq!(parsed.y, spectrum.y);
        assert_eq!(parsed.z, spectrum.z);
        Ok(())
    }

    #[test]
    fn writes_file_with_inherent_api() -> anyhow::Result<()> {
        let mut path = std::env::temp_dir();
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        path.push(format!(
            "rspin-nmrml-2d-writer-{}-{nanos}.nmrml",
            std::process::id()
        ));
        let spectrum = Spectrum2D::new(
            Axis::linear_ppm(0.0, 1.0, 2)?,
            Axis::linear_ppm(10.0, 11.0, 2)?,
            vec![1.0, 2.0, 3.0, 4.0],
            Metadata::named("file"),
        )?;

        NmrMl2D.write_file(&spectrum, &path)?;
        let parsed = read_nmrml_2d_str(&fs::read_to_string(&path)?)?;

        assert_eq!(parsed.z, spectrum.z);
        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn rejects_complex_non_uniform_and_non_finite_metadata() -> anyhow::Result<()> {
        let complex = Spectrum2D::new_complex(
            Axis::linear_ppm(0.0, 1.0, 2)?,
            Axis::linear_ppm(10.0, 11.0, 2)?,
            vec![1.0, 2.0, 3.0, 4.0],
            Some(vec![0.1, 0.2, 0.3, 0.4]),
            Metadata::new(),
        )?;
        let error = write_nmrml_2d(&complex).expect_err("complex export should be explicit");
        assert!(matches!(error, RSpinError::Unsupported { .. }));

        let non_uniform = Spectrum2D::new(
            Axis::ppm(vec![0.0, 0.5, 2.0])?,
            Axis::linear_ppm(10.0, 11.0, 2)?,
            vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
            Metadata::new(),
        )?;
        let error = write_nmrml_2d(&non_uniform).expect_err("non-uniform axes should be explicit");
        assert!(matches!(error, RSpinError::Unsupported { .. }));

        let mut invalid = Spectrum2D::new(
            Axis::linear_ppm(0.0, 1.0, 2)?,
            Axis::linear_ppm(10.0, 11.0, 2)?,
            vec![1.0, 2.0, 3.0, 4.0],
            Metadata::new().with_frequency_mhz(f64::NAN),
        )?;
        invalid.metadata.temperature_k = Some(f64::INFINITY);
        let error = write_nmrml_2d(&invalid).expect_err("non-finite metadata should be rejected");
        assert!(matches!(error, RSpinError::NonFinite { .. }));
        Ok(())
    }
}
