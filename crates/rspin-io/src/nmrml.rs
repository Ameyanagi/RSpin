//! nmrML one-dimensional spectrum import.

use std::{
    fs,
    io::Read,
    path::Path,
    str::{self, FromStr},
};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use flate2::read::ZlibDecoder;
use quick_xml::{
    Reader,
    events::{BytesStart, Event},
};
use rspin_core::{Axis, Metadata, Nucleus, RSpinError, Result, Spectrum1D, Unit};

use crate::SpectrumReader;

const FORMAT: &str = "nmrML";

/// Reader for processed one-dimensional nmrML spectra.
///
/// This focused reader supports schema version `1.0.*`, the 1D
/// `spectrumDataArray` element, little-endian `float64`/`float32` y-value
/// arrays, and little-endian `complex128`/`complex64` x-y pair arrays.
#[derive(Clone, Copy, Debug, Default)]
pub struct NmrMl1D;

impl NmrMl1D {
    /// Reads a processed one-dimensional spectrum from an nmrML file.
    ///
    /// # Errors
    ///
    /// Returns an error when the file is missing, malformed, uses an
    /// unsupported schema version, or stores unsupported binary data.
    pub fn read_file(self, path: impl AsRef<Path>) -> Result<Spectrum1D> {
        read_nmrml_1d_file(path)
    }

    /// Reads a processed one-dimensional spectrum from UTF-8 nmrML bytes.
    ///
    /// # Errors
    ///
    /// Returns an error when the payload is not UTF-8, malformed, uses an
    /// unsupported schema version, or stores unsupported binary data.
    pub fn read_bytes(self, bytes: &[u8]) -> Result<Spectrum1D> {
        read_nmrml_1d_bytes(bytes)
    }
}

impl SpectrumReader for NmrMl1D {
    type Output = Spectrum1D;

    fn read_str(&self, input: &str) -> Result<Self::Output> {
        read_nmrml_1d_str(input)
    }
}

/// Reads a processed one-dimensional spectrum from an nmrML file.
///
/// # Errors
///
/// Returns an error when the file is missing, malformed, uses an unsupported
/// schema version, or stores unsupported binary data.
pub fn read_nmrml_1d_file(path: impl AsRef<Path>) -> Result<Spectrum1D> {
    let path = path.as_ref();
    let input = fs::read_to_string(path).map_err(|error| RSpinError::Parse {
        format: FORMAT,
        message: format!("failed to read {}: {error}", path.display()),
    })?;
    read_nmrml_1d_str(&input)
}

/// Reads a processed one-dimensional spectrum from UTF-8 nmrML bytes.
///
/// # Errors
///
/// Returns an error when the payload is not UTF-8, malformed, uses an
/// unsupported schema version, or stores unsupported binary data.
pub fn read_nmrml_1d_bytes(bytes: &[u8]) -> Result<Spectrum1D> {
    let input = str::from_utf8(bytes).map_err(|error| RSpinError::Parse {
        format: FORMAT,
        message: format!("input is not valid UTF-8: {error}"),
    })?;
    read_nmrml_1d_str(input)
}

/// Reads a processed one-dimensional spectrum from nmrML XML text.
///
/// # Errors
///
/// Returns an error when the payload is malformed, uses an unsupported schema
/// version, or stores unsupported binary data.
pub fn read_nmrml_1d_str(input: &str) -> Result<Spectrum1D> {
    let raw = parse_nmrml_1d(input)?;
    spectrum_from_raw(raw)
}

#[derive(Default)]
struct RawNmrMl1D {
    version: Option<String>,
    spectrum_name: Option<String>,
    spectrum_id: Option<String>,
    spectrum_data: Option<BinaryDataArray>,
    x_axis: Option<AxisSpec>,
    nucleus: Option<Nucleus>,
    frequency_mhz: Option<f64>,
    temperature_k: Option<f64>,
    solvent: Option<String>,
}

#[derive(Clone, Debug)]
struct BinaryDataArray {
    compressed: bool,
    encoded_length: Option<usize>,
    byte_format: String,
    text: String,
}

#[derive(Clone, Debug)]
struct AxisSpec {
    unit: Unit,
    start: Option<f64>,
    end: Option<f64>,
}

enum DecodedSpectrumData {
    Intensities(Vec<f64>),
    Points { x: Vec<f64>, intensities: Vec<f64> },
}

fn parse_nmrml_1d(input: &str) -> Result<RawNmrMl1D> {
    let mut reader = Reader::from_str(input);
    reader.config_mut().trim_text(true);

    let mut raw = RawNmrMl1D::default();
    let mut buffer = Vec::new();
    let mut active_binary: Option<BinaryDataArray> = None;
    let mut active_text = String::new();

    loop {
        match reader
            .read_event_into(&mut buffer)
            .map_err(|error| xml_error(&error))?
        {
            Event::Start(start) => {
                let qualified_name = start.name();
                let name = local_name(qualified_name.as_ref());
                if name == b"nmrML" {
                    apply_root(&mut raw, &start)?;
                } else if name == b"spectrum1D" {
                    apply_spectrum_1d(&mut raw, &start)?;
                } else if name == b"spectrumDataArray" && active_binary.is_none() {
                    active_binary = Some(binary_from_start(&start)?);
                    active_text.clear();
                } else {
                    apply_empty_metadata(&mut raw, &start, name)?;
                }
            }
            Event::Empty(start) => {
                let qualified_name = start.name();
                let name = local_name(qualified_name.as_ref());
                if name == b"nmrML" {
                    apply_root(&mut raw, &start)?;
                } else if name == b"spectrum1D" {
                    apply_spectrum_1d(&mut raw, &start)?;
                } else if name == b"spectrumDataArray" && raw.spectrum_data.is_none() {
                    raw.spectrum_data = Some(binary_from_start(&start)?);
                } else {
                    apply_empty_metadata(&mut raw, &start, name)?;
                }
            }
            Event::Text(text) if active_binary.is_some() => {
                active_text.push_str(str::from_utf8(text.as_ref()).map_err(|error| {
                    RSpinError::Parse {
                        format: FORMAT,
                        message: format!("binary text is not valid UTF-8: {error}"),
                    }
                })?);
            }
            Event::CData(text) if active_binary.is_some() => {
                active_text.push_str(str::from_utf8(text.as_ref()).map_err(|error| {
                    RSpinError::Parse {
                        format: FORMAT,
                        message: format!("binary CDATA is not valid UTF-8: {error}"),
                    }
                })?);
            }
            Event::End(end) if local_name(end.name().as_ref()) == b"spectrumDataArray" => {
                if let Some(mut binary) = active_binary.take() {
                    if raw.spectrum_data.is_none() {
                        binary.text.clone_from(&active_text);
                        raw.spectrum_data = Some(binary);
                    }
                }
                active_text.clear();
            }
            Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }

    Ok(raw)
}

fn spectrum_from_raw(raw: RawNmrMl1D) -> Result<Spectrum1D> {
    let version = validate_version(raw.version.as_deref())?;
    let binary = raw.spectrum_data.ok_or_else(|| RSpinError::Parse {
        format: FORMAT,
        message: "missing spectrumDataArray".to_owned(),
    })?;
    let decoded = decode_spectrum_data(&binary)?;

    let (axis, intensities) = match decoded {
        DecodedSpectrumData::Intensities(intensities) => {
            let axis_spec = raw.x_axis.ok_or_else(|| RSpinError::Parse {
                format: FORMAT,
                message: "missing xAxis for y-value spectrumDataArray".to_owned(),
            })?;
            let start = axis_spec.start.ok_or_else(|| RSpinError::Parse {
                format: FORMAT,
                message: "missing xAxis startValue".to_owned(),
            })?;
            let end = axis_spec.end.ok_or_else(|| RSpinError::Parse {
                format: FORMAT,
                message: "missing xAxis endValue".to_owned(),
            })?;
            let label = axis_label(axis_spec.unit);
            (
                Axis::linear(label, axis_spec.unit, start, end, intensities.len())?,
                intensities,
            )
        }
        DecodedSpectrumData::Points { x, intensities } => {
            let unit = match raw.x_axis.as_ref() {
                Some(axis_spec) => axis_spec.unit,
                None => Unit::Points,
            };
            (Axis::new(axis_label(unit), unit, x)?, intensities)
        }
    };

    let name = match raw.spectrum_name {
        Some(name) => Some(name),
        None => raw.spectrum_id,
    };
    let metadata = Metadata {
        name,
        nucleus: raw.nucleus,
        frequency_mhz: raw.frequency_mhz,
        solvent: raw.solvent,
        temperature_k: raw.temperature_k,
        origin: Some(format!("nmrML {version}")),
        molecules: Vec::new(),
    };

    Spectrum1D::new(axis, intensities, metadata)
}

fn apply_root(raw: &mut RawNmrMl1D, start: &BytesStart<'_>) -> Result<()> {
    if raw.version.is_none() {
        raw.version = attr_value(start, b"version")?;
    }
    Ok(())
}

fn apply_spectrum_1d(raw: &mut RawNmrMl1D, start: &BytesStart<'_>) -> Result<()> {
    if raw.spectrum_name.is_none() {
        raw.spectrum_name = attr_value(start, b"name")?;
    }
    if raw.spectrum_id.is_none() {
        raw.spectrum_id = attr_value(start, b"id")?;
    }
    Ok(())
}

fn apply_empty_metadata(raw: &mut RawNmrMl1D, start: &BytesStart<'_>, name: &[u8]) -> Result<()> {
    match name {
        b"xAxis" if raw.x_axis.is_none() => {
            raw.x_axis = Some(axis_from_start(start)?);
        }
        b"acquisitionNucleus" if raw.nucleus.is_none() => {
            raw.nucleus = attr_value(start, b"name")?
                .as_deref()
                .map(parse_nucleus)
                .transpose()?;
        }
        b"effectiveExcitationField" | b"irradiationFrequency" if raw.frequency_mhz.is_none() => {
            let value = optional_f64_attr(start, b"value", "frequency value")?;
            let unit_name = attr_value(start, b"unitName")?;
            raw.frequency_mhz =
                value.and_then(|frequency| frequency_to_mhz(frequency, unit_name.as_deref()));
        }
        b"sampleAcquisitionTemperature" if raw.temperature_k.is_none() => {
            let value = optional_f64_attr(start, b"value", "temperature value")?;
            let unit_name = attr_value(start, b"unitName")?;
            raw.temperature_k =
                value.map(|temperature| temperature_to_kelvin(temperature, unit_name.as_deref()));
        }
        b"solventType" if raw.solvent.is_none() => {
            raw.solvent = match attr_value(start, b"value")? {
                Some(value) => Some(value),
                None => attr_value(start, b"name")?,
            };
        }
        _ => {}
    }
    Ok(())
}

fn binary_from_start(start: &BytesStart<'_>) -> Result<BinaryDataArray> {
    let compressed = required_bool_attr(start, b"compressed", "spectrumDataArray compressed")?;
    let encoded_length = optional_usize_attr(start, b"encodedLength", "encodedLength")?;
    let byte_format = attr_value(start, b"byteFormat")?.ok_or_else(|| RSpinError::Parse {
        format: FORMAT,
        message: "missing spectrumDataArray byteFormat".to_owned(),
    })?;

    Ok(BinaryDataArray {
        compressed,
        encoded_length,
        byte_format,
        text: String::new(),
    })
}

fn axis_from_start(start: &BytesStart<'_>) -> Result<AxisSpec> {
    let unit = match attr_value(start, b"unitName")? {
        Some(unit_name) => axis_unit(&unit_name),
        None => Unit::Arbitrary,
    };
    let start_value = optional_f64_attr(start, b"startValue", "xAxis startValue")?;
    let end_value = optional_f64_attr(start, b"endValue", "xAxis endValue")?;
    Ok(AxisSpec {
        unit,
        start: start_value,
        end: end_value,
    })
}

fn decode_spectrum_data(binary: &BinaryDataArray) -> Result<DecodedSpectrumData> {
    let payload = binary_payload(binary)?;
    match normalize_token(&binary.byte_format).as_str() {
        "float64" => {
            decode_f64_values(&payload, "spectrumDataArray").map(DecodedSpectrumData::Intensities)
        }
        "float32" => {
            decode_f32_values(&payload, "spectrumDataArray").map(DecodedSpectrumData::Intensities)
        }
        "complex128" => decode_f64_pairs(&payload, "spectrumDataArray")
            .map(|(x, intensities)| DecodedSpectrumData::Points { x, intensities }),
        "complex64" => decode_f32_pairs(&payload, "spectrumDataArray")
            .map(|(x, intensities)| DecodedSpectrumData::Points { x, intensities }),
        _ => Err(RSpinError::Unsupported {
            feature: "nmrML spectrumDataArray byteFormat",
        }),
    }
}

fn binary_payload(binary: &BinaryDataArray) -> Result<Vec<u8>> {
    let encoded = binary
        .text
        .bytes()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect::<Vec<_>>();
    if let Some(expected) = binary.encoded_length {
        if encoded.len() != expected {
            return Err(RSpinError::Parse {
                format: FORMAT,
                message: format!(
                    "encodedLength is {expected} but spectrumDataArray contains {} characters",
                    encoded.len()
                ),
            });
        }
    }

    let decoded_bytes = STANDARD
        .decode(&encoded)
        .map_err(|error| RSpinError::Parse {
            format: FORMAT,
            message: format!("invalid base64 spectrumDataArray: {error}"),
        })?;
    if !binary.compressed {
        return Ok(decoded_bytes);
    }

    let mut zlib_reader = ZlibDecoder::new(decoded_bytes.as_slice());
    let mut inflated = Vec::new();
    zlib_reader
        .read_to_end(&mut inflated)
        .map_err(|error| RSpinError::Parse {
            format: FORMAT,
            message: format!("failed to zlib-decompress spectrumDataArray: {error}"),
        })?;
    Ok(inflated)
}

fn decode_f64_values(bytes: &[u8], field: &'static str) -> Result<Vec<f64>> {
    if bytes.len() % 8 != 0 {
        return Err(RSpinError::Parse {
            format: FORMAT,
            message: format!("{field} byte length is not divisible by 8"),
        });
    }

    bytes
        .chunks_exact(8)
        .map(|chunk| {
            let value = f64::from_le_bytes([
                chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5], chunk[6], chunk[7],
            ]);
            finite_value(field, value)
        })
        .collect()
}

fn decode_f32_values(bytes: &[u8], field: &'static str) -> Result<Vec<f64>> {
    if bytes.len() % 4 != 0 {
        return Err(RSpinError::Parse {
            format: FORMAT,
            message: format!("{field} byte length is not divisible by 4"),
        });
    }

    bytes
        .chunks_exact(4)
        .map(|chunk| {
            let value = f64::from(f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
            finite_value(field, value)
        })
        .collect()
}

fn decode_f64_pairs(bytes: &[u8], field: &'static str) -> Result<(Vec<f64>, Vec<f64>)> {
    let values = decode_f64_values(bytes, field)?;
    split_pairs(&values, field)
}

fn decode_f32_pairs(bytes: &[u8], field: &'static str) -> Result<(Vec<f64>, Vec<f64>)> {
    let values = decode_f32_values(bytes, field)?;
    split_pairs(&values, field)
}

fn split_pairs(values: &[f64], field: &'static str) -> Result<(Vec<f64>, Vec<f64>)> {
    if values.len() % 2 != 0 {
        return Err(RSpinError::Parse {
            format: FORMAT,
            message: format!("{field} pair data has an odd number of values"),
        });
    }

    let mut x = Vec::with_capacity(values.len() / 2);
    let mut intensities = Vec::with_capacity(values.len() / 2);
    for pair in values.chunks_exact(2) {
        x.push(pair[0]);
        intensities.push(pair[1]);
    }
    Ok((x, intensities))
}

fn finite_value(field: &'static str, value: f64) -> Result<f64> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(RSpinError::NonFinite { field })
    }
}

fn validate_version(version: Option<&str>) -> Result<String> {
    let version = version.ok_or_else(|| RSpinError::Parse {
        format: FORMAT,
        message: "missing required nmrML version".to_owned(),
    })?;
    let normalized = match version.trim().strip_prefix('v') {
        Some(trimmed) => trimmed,
        None => version.trim(),
    };
    if normalized.starts_with("1.0.") {
        Ok(version.trim().to_owned())
    } else {
        Err(RSpinError::Unsupported {
            feature: "nmrML document version",
        })
    }
}

fn parse_nucleus(value: &str) -> Result<Nucleus> {
    match normalize_token(value).as_str() {
        "hydrogenatom" | "proton" | "h1" | "1h" => Ok(Nucleus::Hydrogen1),
        "carbon13" | "carbon13atom" | "c13" | "13c" => Ok(Nucleus::Carbon13),
        "nitrogen15" | "nitrogen15atom" | "n15" | "15n" => Ok(Nucleus::Nitrogen15),
        "fluorine19" | "fluorine19atom" | "f19" | "19f" => Ok(Nucleus::Fluorine19),
        "phosphorus31" | "phosphorus31atom" | "p31" | "31p" => Ok(Nucleus::Phosphorus31),
        _ => Nucleus::from_str(value),
    }
}

fn axis_unit(value: &str) -> Unit {
    match normalize_token(value).as_str() {
        "partspermillion" | "ppm" => Unit::Ppm,
        "hertz" | "hz" => Unit::Hertz,
        "second" | "seconds" | "s" => Unit::Seconds,
        "point" | "points" => Unit::Points,
        _ => Unit::Arbitrary,
    }
}

fn axis_label(unit: Unit) -> &'static str {
    match unit {
        Unit::Ppm => "chemical shift",
        Unit::Hertz => "frequency",
        Unit::Seconds => "time",
        Unit::Points => "point",
        _ => "x",
    }
}

fn frequency_to_mhz(value: f64, unit_name: Option<&str>) -> Option<f64> {
    match unit_name.map(normalize_token).as_deref() {
        Some("megahertz" | "mhz") | None => Some(value),
        Some("kilohertz" | "khz") => Some(value / 1_000.0),
        Some("hertz" | "hz") => Some(value / 1_000_000.0),
        Some("gigahertz" | "ghz") => Some(value * 1_000.0),
        _ => None,
    }
}

fn temperature_to_kelvin(value: f64, unit_name: Option<&str>) -> f64 {
    match unit_name.map(normalize_token).as_deref() {
        Some("celsius" | "degreecelsius") => value + 273.15,
        _ => value,
    }
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

fn required_bool_attr(start: &BytesStart<'_>, name: &[u8], field: &'static str) -> Result<bool> {
    let value = attr_value(start, name)?.ok_or_else(|| RSpinError::Parse {
        format: FORMAT,
        message: format!("missing {field}"),
    })?;
    match value.trim() {
        "true" | "1" => Ok(true),
        "false" | "0" => Ok(false),
        _ => Err(RSpinError::Parse {
            format: FORMAT,
            message: format!("{field} must be true or false"),
        }),
    }
}

fn optional_f64_attr(
    start: &BytesStart<'_>,
    name: &[u8],
    field: &'static str,
) -> Result<Option<f64>> {
    attr_value(start, name)?
        .as_deref()
        .map(|value| parse_float(field, value))
        .transpose()
}

fn optional_usize_attr(
    start: &BytesStart<'_>,
    name: &[u8],
    field: &'static str,
) -> Result<Option<usize>> {
    attr_value(start, name)?
        .as_deref()
        .map(|value| {
            value
                .trim()
                .parse::<usize>()
                .map_err(|error| RSpinError::Parse {
                    format: FORMAT,
                    message: format!("{field}: {error}"),
                })
        })
        .transpose()
}

fn parse_float(field: &'static str, value: &str) -> Result<f64> {
    let parsed = value
        .trim()
        .parse::<f64>()
        .map_err(|error| RSpinError::Parse {
            format: FORMAT,
            message: format!("{field}: {error}"),
        })?;
    finite_value(field, parsed)
}

fn local_name(name: &[u8]) -> &[u8] {
    match name.iter().rposition(|byte| *byte == b':') {
        Some(index) => &name[index + 1..],
        None => name,
    }
}

fn normalize_token(value: &str) -> String {
    value
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .flat_map(char::to_lowercase)
        .collect()
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
    fn reads_compressed_float64_spectrum() -> Result<()> {
        let input = r#"
            <nmrML version="v1.0.rc1" xmlns="http://nmrml.org/schema">
              <acquisition>
                <acquisition1D>
                  <acquisitionParameterSet numberOfScans="1" numberOfSteadyStateScans="0">
                    <sampleAcquisitionTemperature value="25.0" unitName="degree celsius"/>
                    <DirectDimensionParameterSet decoupled="false" numberOfDataPoints="3">
                      <acquisitionNucleus cvRef="CHEBI" accession="CHEBI:49637" name="hydrogen atom"/>
                      <effectiveExcitationField value="600.0" unitName="megaHertz"/>
                    </DirectDimensionParameterSet>
                  </acquisitionParameterSet>
                </acquisition1D>
              </acquisition>
              <spectrumList>
                <spectrum1D id="s1" numberOfDataPoints="3">
                  <spectrumDataArray compressed="true" encodedLength="28" byteFormat="float64">eJxjYACBD/YMEHAAQvE4AAAcPwI8</spectrumDataArray>
                  <xAxis unitName="parts per million" startValue="10.0" endValue="8.0"/>
                </spectrum1D>
              </spectrumList>
            </nmrML>
        "#;

        let spectrum = read_nmrml_1d_str(input)?;

        assert_eq!(spectrum.len(), 3);
        assert_eq!(spectrum.x.unit, Unit::Ppm);
        assert_eq!(spectrum.x.values, vec![10.0, 9.0, 8.0]);
        assert_eq!(spectrum.intensities, vec![1.0, -2.0, 3.5]);
        assert_eq!(spectrum.metadata.name.as_deref(), Some("s1"));
        assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Hydrogen1));
        assert_eq!(spectrum.metadata.frequency_mhz, Some(600.0));
        assert_eq!(spectrum.metadata.temperature_k, Some(298.15));
        assert_eq!(spectrum.metadata.origin.as_deref(), Some("nmrML v1.0.rc1"));
        Ok(())
    }

    #[test]
    fn reads_complex128_as_xy_pairs() -> Result<()> {
        let input = r#"
            <nmrML version="1.0.rc1">
              <spectrumList>
                <spectrum1D id="pairs" numberOfDataPoints="3">
                  <spectrumDataArray compressed="false" encodedLength="64" byteFormat="complex128">
                    AAAAAAAAJEAAAAAAAADwPwAAAAAAACJAAAAAAAAAAMAAAAAAAAAgQAAAAAAAAAxA
                  </spectrumDataArray>
                  <xAxis unitName="parts per million" startValue="10.0" endValue="8.0"/>
                </spectrum1D>
              </spectrumList>
            </nmrML>
        "#;

        let spectrum = read_nmrml_1d_str(input)?;

        assert_eq!(spectrum.x.values, vec![10.0, 9.0, 8.0]);
        assert_eq!(spectrum.intensities, vec![1.0, -2.0, 3.5]);
        assert_eq!(spectrum.metadata.origin.as_deref(), Some("nmrML 1.0.rc1"));
        Ok(())
    }

    #[test]
    fn rejects_unsupported_version() {
        let error = read_nmrml_1d_str(
            r#"<nmrML version="2.0.0"><spectrumDataArray compressed="false" encodedLength="0" byteFormat="float64"/></nmrML>"#,
        )
        .expect_err("unsupported versions should be rejected");

        assert!(matches!(error, RSpinError::Unsupported { .. }));
    }

    #[test]
    fn rejects_encoded_length_mismatch() {
        let error = read_nmrml_1d_str(
            r#"
            <nmrML version="1.0.rc1">
              <spectrumList>
                <spectrum1D>
                  <spectrumDataArray compressed="false" encodedLength="3" byteFormat="float64">AAAA</spectrumDataArray>
                  <xAxis unitName="parts per million" startValue="1" endValue="0"/>
                </spectrum1D>
              </spectrumList>
            </nmrML>
            "#,
        )
        .expect_err("encodedLength mismatches should be rejected");

        assert!(matches!(error, RSpinError::Parse { .. }));
    }
}
