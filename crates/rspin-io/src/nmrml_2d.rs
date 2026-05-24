//! nmrML two-dimensional FID import.

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
use rspin_core::{Axis, Metadata, Nucleus, RSpinError, Result, Spectrum2D, Unit};

const FORMAT: &str = "nmrML";

/// Reader for raw two-dimensional nmrML FID payloads.
///
/// This focused reader supports schema version `1.0.*`, `acquisitionMultiD`,
/// direct and first indirect dimension metadata, and little-endian
/// `complex128`/`complex64` `fidData`.
#[derive(Clone, Copy, Debug, Default)]
pub struct NmrMl2D;

impl NmrMl2D {
    /// Reads a raw two-dimensional FID from an nmrML file.
    ///
    /// # Errors
    ///
    /// Returns an error when the file is missing, malformed, not
    /// two-dimensional, uses an unsupported schema version, or stores
    /// unsupported binary data.
    pub fn read_file(self, path: impl AsRef<Path>) -> Result<Spectrum2D> {
        read_nmrml_2d_file(path)
    }

    /// Reads a raw two-dimensional FID from UTF-8 nmrML bytes.
    ///
    /// # Errors
    ///
    /// Returns an error when the payload is not UTF-8, malformed, not
    /// two-dimensional, uses an unsupported schema version, or stores
    /// unsupported binary data.
    pub fn read_bytes(self, bytes: &[u8]) -> Result<Spectrum2D> {
        read_nmrml_2d_bytes(bytes)
    }
}

/// Reads a raw two-dimensional FID from an nmrML file.
///
/// # Errors
///
/// Returns an error when the file is missing, malformed, not two-dimensional,
/// uses an unsupported schema version, or stores unsupported binary data.
pub fn read_nmrml_2d_file(path: impl AsRef<Path>) -> Result<Spectrum2D> {
    let path = path.as_ref();
    let input = fs::read_to_string(path).map_err(|error| RSpinError::Parse {
        format: FORMAT,
        message: format!("failed to read {}: {error}", path.display()),
    })?;
    read_nmrml_2d_str(&input)
}

/// Reads a raw two-dimensional FID from UTF-8 nmrML bytes.
///
/// # Errors
///
/// Returns an error when the payload is not UTF-8, malformed, not
/// two-dimensional, uses an unsupported schema version, or stores unsupported
/// binary data.
pub fn read_nmrml_2d_bytes(bytes: &[u8]) -> Result<Spectrum2D> {
    let input = str::from_utf8(bytes).map_err(|error| RSpinError::Parse {
        format: FORMAT,
        message: format!("input is not valid UTF-8: {error}"),
    })?;
    read_nmrml_2d_str(input)
}

/// Reads a raw two-dimensional FID from nmrML XML text.
///
/// # Errors
///
/// Returns an error when the payload is malformed, not two-dimensional, uses an
/// unsupported schema version, or stores unsupported binary data.
pub fn read_nmrml_2d_str(input: &str) -> Result<Spectrum2D> {
    let raw = parse_nmrml_2d(input)?;
    spectrum_from_raw(raw)
}

#[derive(Default)]
struct RawNmrMl2D {
    version: Option<String>,
    name: Option<String>,
    id: Option<String>,
    fid_data: Option<BinaryDataArray>,
    direct: DimensionSpec,
    indirect: DimensionSpec,
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

#[derive(Clone, Debug, Default)]
struct DimensionSpec {
    point_count: Option<usize>,
    nucleus: Option<Nucleus>,
    frequency_mhz: Option<f64>,
    sweep_width_hz: Option<f64>,
}

#[derive(Clone, Copy)]
enum DimensionContext {
    Direct,
    Indirect,
}

fn parse_nmrml_2d(input: &str) -> Result<RawNmrMl2D> {
    let mut reader = Reader::from_str(input);
    reader.config_mut().trim_text(true);

    let mut raw = RawNmrMl2D::default();
    let mut buffer = Vec::new();
    let mut active_binary: Option<BinaryDataArray> = None;
    let mut active_text = String::new();
    let mut dimension_context = None;

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
                } else if name == b"spectrumMultiD" {
                    apply_spectrum(&mut raw, &start)?;
                } else if name == b"directDimensionParameterSet" {
                    dimension_context = Some(DimensionContext::Direct);
                    apply_dimension_start(&mut raw.direct, &start)?;
                } else if name == b"indirectDimensionParameterSet" {
                    dimension_context = Some(DimensionContext::Indirect);
                    apply_dimension_start(&mut raw.indirect, &start)?;
                } else if name == b"fidData" && active_binary.is_none() {
                    active_binary = Some(binary_from_start(&start)?);
                    active_text.clear();
                } else if active_binary.is_none() {
                    apply_metadata(&mut raw, &start, name, dimension_context)?;
                }
            }
            Event::Empty(start) => {
                let qualified_name = start.name();
                let name = local_name(qualified_name.as_ref());
                if name == b"nmrML" {
                    apply_root(&mut raw, &start)?;
                } else if name == b"spectrumMultiD" {
                    apply_spectrum(&mut raw, &start)?;
                } else if name == b"directDimensionParameterSet" {
                    apply_dimension_start(&mut raw.direct, &start)?;
                } else if name == b"indirectDimensionParameterSet" {
                    apply_dimension_start(&mut raw.indirect, &start)?;
                } else if name == b"fidData" && raw.fid_data.is_none() {
                    raw.fid_data = Some(binary_from_start(&start)?);
                } else {
                    apply_metadata(&mut raw, &start, name, dimension_context)?;
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
            Event::End(end) => {
                let qualified_name = end.name();
                let name = local_name(qualified_name.as_ref());
                if name == b"fidData" {
                    if let Some(mut binary) = active_binary.take() {
                        if raw.fid_data.is_none() {
                            binary.text.clone_from(&active_text);
                            raw.fid_data = Some(binary);
                        }
                    }
                    active_text.clear();
                } else if matches!(
                    name,
                    b"directDimensionParameterSet" | b"indirectDimensionParameterSet"
                ) {
                    dimension_context = None;
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }

    Ok(raw)
}

fn spectrum_from_raw(raw: RawNmrMl2D) -> Result<Spectrum2D> {
    let version = validate_version(raw.version.as_deref())?;
    let x_count = raw.direct.point_count.ok_or_else(|| RSpinError::Parse {
        format: FORMAT,
        message: "missing direct dimension point count".to_owned(),
    })?;
    let y_count = raw.indirect.point_count.ok_or_else(|| RSpinError::Parse {
        format: FORMAT,
        message: "missing indirect dimension point count".to_owned(),
    })?;
    let binary = raw.fid_data.ok_or_else(|| RSpinError::Parse {
        format: FORMAT,
        message: "missing multidimensional fidData".to_owned(),
    })?;

    let (z, imaginary) = decode_fid_matrix(&binary, x_count, y_count)?;
    let x = build_time_axis("time", x_count, raw.direct.sweep_width_hz)?;
    let y = build_time_axis("indirect time", y_count, raw.indirect.sweep_width_hz)?;
    let metadata = Metadata {
        name: raw.name.or(raw.id),
        nucleus: raw.direct.nucleus,
        frequency_mhz: raw.direct.frequency_mhz,
        solvent: raw.solvent,
        temperature_k: raw.temperature_k,
        origin: Some(format!("nmrML {version}")),
        molecules: Vec::new(),
    };

    Spectrum2D::new_complex(x, y, z, imaginary, metadata)
}

fn apply_root(raw: &mut RawNmrMl2D, start: &BytesStart<'_>) -> Result<()> {
    if raw.version.is_none() {
        raw.version = attr_value(start, b"version")?;
    }
    if raw.id.is_none() {
        raw.id = attr_value(start, b"id")?.or(attr_value(start, b"accession")?);
    }
    Ok(())
}

fn apply_spectrum(raw: &mut RawNmrMl2D, start: &BytesStart<'_>) -> Result<()> {
    if raw.name.is_none() {
        raw.name = attr_value(start, b"name")?;
    }
    if raw.id.is_none() {
        raw.id = attr_value(start, b"id")?;
    }
    Ok(())
}

fn apply_dimension_start(dimension: &mut DimensionSpec, start: &BytesStart<'_>) -> Result<()> {
    if dimension.point_count.is_none() {
        dimension.point_count =
            optional_usize_attr(start, b"numberOfDataPoints", "numberOfDataPoints")?;
    }
    Ok(())
}

fn apply_metadata(
    raw: &mut RawNmrMl2D,
    start: &BytesStart<'_>,
    name: &[u8],
    dimension_context: Option<DimensionContext>,
) -> Result<()> {
    match name {
        b"acquisitionNucleus" => {
            if let Some(dimension) = dimension_mut(raw, dimension_context) {
                if dimension.nucleus.is_none() {
                    dimension.nucleus = attr_value(start, b"name")?
                        .as_deref()
                        .map(parse_nucleus)
                        .transpose()?;
                }
            }
        }
        b"irradiationFrequency" => {
            if let Some(dimension) = dimension_mut(raw, dimension_context) {
                let value = optional_f64_attr(start, b"value", "frequency value")?;
                let unit_name = attr_value(start, b"unitName")?;
                if let Some(frequency) =
                    value.and_then(|frequency| frequency_to_mhz(frequency, unit_name.as_deref()))
                {
                    dimension.frequency_mhz = Some(frequency);
                }
            }
        }
        b"effectiveExcitationField" => {
            if let Some(dimension) = dimension_mut(raw, dimension_context) {
                if dimension.frequency_mhz.is_none() {
                    let value = optional_f64_attr(start, b"value", "frequency value")?;
                    let unit_name = attr_value(start, b"unitName")?;
                    dimension.frequency_mhz = value
                        .and_then(|frequency| frequency_to_mhz(frequency, unit_name.as_deref()));
                }
            }
        }
        b"sweepWidth" => {
            if let Some(dimension) = dimension_mut(raw, dimension_context) {
                if dimension.sweep_width_hz.is_none() {
                    let value = optional_f64_attr(start, b"value", "sweep width value")?;
                    let unit_name = attr_value(start, b"unitName")?;
                    dimension.sweep_width_hz = value
                        .and_then(|sweep_width| frequency_to_hz(sweep_width, unit_name.as_deref()));
                }
            }
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

fn dimension_mut(
    raw: &mut RawNmrMl2D,
    context: Option<DimensionContext>,
) -> Option<&mut DimensionSpec> {
    match context {
        Some(DimensionContext::Direct) => Some(&mut raw.direct),
        Some(DimensionContext::Indirect) => Some(&mut raw.indirect),
        None => None,
    }
}

fn binary_from_start(start: &BytesStart<'_>) -> Result<BinaryDataArray> {
    let compressed = required_bool_attr(start, b"compressed", "fidData compressed")?;
    let encoded_length = optional_usize_attr(start, b"encodedLength", "encodedLength")?;
    let byte_format = attr_value(start, b"byteFormat")?.ok_or_else(|| RSpinError::Parse {
        format: FORMAT,
        message: "missing fidData byteFormat".to_owned(),
    })?;

    Ok(BinaryDataArray {
        compressed,
        encoded_length,
        byte_format,
        text: String::new(),
    })
}

fn decode_fid_matrix(
    binary: &BinaryDataArray,
    x_count: usize,
    y_count: usize,
) -> Result<(Vec<f64>, Option<Vec<f64>>)> {
    let payload = binary_payload(binary)?;
    let expected_points =
        x_count
            .checked_mul(y_count)
            .ok_or_else(|| RSpinError::InvalidSpectrum {
                message: "nmrML 2D matrix size is too large".to_owned(),
            })?;

    match normalize_token(&binary.byte_format).as_str() {
        "complex128" if payload.len() == expected_points.saturating_mul(8) => {
            decode_f32_pairs(&payload, "fidData")
        }
        "complex128" if payload.len() == expected_points.saturating_mul(16) => {
            decode_f64_pairs(&payload, "fidData")
        }
        "complex64" => decode_f32_pairs(&payload, "fidData"),
        "float64" => decode_f64_values(&payload, "fidData").map(|z| (z, None)),
        "float32" => decode_f32_values(&payload, "fidData").map(|z| (z, None)),
        _ => Err(RSpinError::Unsupported {
            feature: "nmrML multidimensional fidData byteFormat",
        }),
    }
    .and_then(|(z, imaginary)| validate_matrix_length(z, imaginary, expected_points))
}

fn validate_matrix_length(
    z: Vec<f64>,
    imaginary: Option<Vec<f64>>,
    expected_points: usize,
) -> Result<(Vec<f64>, Option<Vec<f64>>)> {
    if z.len() != expected_points {
        return Err(RSpinError::InvalidSpectrum {
            message: format!(
                "nmrML 2D fidData has {} real points but dimensions require {expected_points}",
                z.len()
            ),
        });
    }
    if let Some(values) = imaginary.as_deref() {
        if values.len() != expected_points {
            return Err(RSpinError::InvalidSpectrum {
                message: format!(
                    "nmrML 2D fidData has {} imaginary points but dimensions require {expected_points}",
                    values.len()
                ),
            });
        }
    }
    Ok((z, imaginary))
}

fn binary_payload(binary: &BinaryDataArray) -> Result<Vec<u8>> {
    let encoded = binary
        .text
        .bytes()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect::<Vec<_>>();
    validate_encoded_length(binary.encoded_length, encoded.len())?;

    let decoded_bytes = STANDARD
        .decode(&encoded)
        .map_err(|error| RSpinError::Parse {
            format: FORMAT,
            message: format!("invalid base64 fidData: {error}"),
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
            message: format!("failed to zlib-decompress fidData: {error}"),
        })?;
    Ok(inflated)
}

fn validate_encoded_length(expected: Option<usize>, actual: usize) -> Result<()> {
    let Some(expected) = expected else {
        return Ok(());
    };
    if actual == expected || (actual < expected && expected - actual <= 4) {
        return Ok(());
    }
    Err(RSpinError::Parse {
        format: FORMAT,
        message: format!("encodedLength is {expected} but fidData contains {actual} characters"),
    })
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

fn decode_f64_pairs(bytes: &[u8], field: &'static str) -> Result<(Vec<f64>, Option<Vec<f64>>)> {
    let values = decode_f64_values(bytes, field)?;
    split_pairs(&values, field).map(|(z, imaginary)| (z, Some(imaginary)))
}

fn decode_f32_pairs(bytes: &[u8], field: &'static str) -> Result<(Vec<f64>, Option<Vec<f64>>)> {
    let values = decode_f32_values(bytes, field)?;
    split_pairs(&values, field).map(|(z, imaginary)| (z, Some(imaginary)))
}

fn split_pairs(values: &[f64], field: &'static str) -> Result<(Vec<f64>, Vec<f64>)> {
    if values.len() % 2 != 0 {
        return Err(RSpinError::Parse {
            format: FORMAT,
            message: format!("{field} pair data has an odd number of values"),
        });
    }

    let mut z = Vec::with_capacity(values.len() / 2);
    let mut imaginary = Vec::with_capacity(values.len() / 2);
    for pair in values.chunks_exact(2) {
        z.push(pair[0]);
        imaginary.push(pair[1]);
    }
    Ok((z, imaginary))
}

fn build_time_axis(
    label: &'static str,
    points: usize,
    sweep_width_hz: Option<f64>,
) -> Result<Axis> {
    match sweep_width_hz {
        Some(sweep_width) if sweep_width > 0.0 => {
            let end = if points <= 1 {
                0.0
            } else {
                let segments = u32::try_from(points - 1).map_err(|_| RSpinError::InvalidAxis {
                    message: "nmrML FID point count is too large".to_owned(),
                })?;
                f64::from(segments) / sweep_width
            };
            Axis::linear(label, Unit::Seconds, 0.0, end, points)
        }
        _ => {
            let end = u32::try_from(points.saturating_sub(1)).map_or(0.0, f64::from);
            Axis::linear(label, Unit::Points, 0.0, end, points)
        }
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

fn frequency_to_mhz(value: f64, unit_name: Option<&str>) -> Option<f64> {
    match unit_name.map(normalize_token).as_deref() {
        Some("megahertz" | "mhz") | None => Some(value),
        Some("kilohertz" | "khz") => Some(value / 1_000.0),
        Some("hertz" | "hz") if value < 100_000.0 => Some(value),
        Some("hertz" | "hz") => Some(value / 1_000_000.0),
        Some("gigahertz" | "ghz") => Some(value * 1_000.0),
        _ => None,
    }
}

fn frequency_to_hz(value: f64, unit_name: Option<&str>) -> Option<f64> {
    match unit_name.map(normalize_token).as_deref() {
        Some("hertz" | "hz") | None => Some(value),
        Some("kilohertz" | "khz") => Some(value * 1_000.0),
        Some("megahertz" | "mhz") => Some(value * 1_000_000.0),
        Some("gigahertz" | "ghz") => Some(value * 1_000_000_000.0),
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

fn finite_value(field: &'static str, value: f64) -> Result<f64> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(RSpinError::NonFinite { field })
    }
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
    use rspin_core::Unit;

    use super::*;

    #[test]
    fn reads_compressed_complex64_2d_fid() -> Result<()> {
        let input = r#"
            <nmrML version="v1.0.rc1" id="two-d">
              <acquisition>
                <acquisitionMultiD>
                  <acquisitionParameterSet numberOfScans="1" numberOfSteadyStateScans="0">
                    <sampleAcquisitionTemperature value="25.0" unitName="degree celsius"/>
                    <directDimensionParameterSet decoupled="false" numberOfDataPoints="2">
                      <acquisitionNucleus cvRef="NMR" accession="NMR:1400151" name="1H"/>
                      <irradiationFrequency value="600.0" unitName="megaHertz"/>
                      <sweepWidth value="2.0" unitName="hertz"/>
                    </directDimensionParameterSet>
                    <indirectDimensionParameterSet decoupled="false" numberOfDataPoints="2">
                      <acquisitionNucleus cvRef="NMR" accession="NMR:1400154" name="13C"/>
                      <irradiationFrequency value="150.0" unitName="megaHertz"/>
                      <sweepWidth value="4.0" unitName="hertz"/>
                    </indirectDimensionParameterSet>
                  </acquisitionParameterSet>
                  <fidData compressed="true" encodedLength="44" byteFormat="complex64">
                    eJxjYGiwZ2Bo2M/AwOAAxAeAFJB2ANINQLrhAABd6gZ/
                  </fidData>
                </acquisitionMultiD>
              </acquisition>
            </nmrML>
        "#;

        let spectrum = read_nmrml_2d_str(input)?;

        assert_eq!(spectrum.shape(), (2, 2));
        assert_eq!(spectrum.x.unit, Unit::Seconds);
        assert_eq!(spectrum.y.unit, Unit::Seconds);
        assert_eq!(spectrum.x.values, vec![0.0, 0.5]);
        assert_eq!(spectrum.y.values, vec![0.0, 0.25]);
        assert_eq!(spectrum.z, vec![1.0, 2.0, 3.0, 4.0]);
        assert_eq!(spectrum.imaginary, Some(vec![-1.0, -2.0, -3.0, -4.0]));
        assert_eq!(spectrum.metadata.name.as_deref(), Some("two-d"));
        assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Hydrogen1));
        assert_eq!(spectrum.metadata.frequency_mhz, Some(600.0));
        assert_eq!(spectrum.metadata.temperature_k, Some(298.15));
        Ok(())
    }

    #[test]
    fn rejects_length_mismatch() {
        let input = r#"
            <nmrML version="v1.0.rc1">
              <acquisition>
                <acquisitionMultiD>
                  <acquisitionParameterSet>
                    <directDimensionParameterSet decoupled="false" numberOfDataPoints="3"/>
                    <indirectDimensionParameterSet decoupled="false" numberOfDataPoints="2"/>
                  </acquisitionParameterSet>
                  <fidData compressed="true" encodedLength="44" byteFormat="complex64">
                    eJxjYGiwZ2Bo2M/AwOAAxAeAFJB2ANINQLrhAABd6gZ/
                  </fidData>
                </acquisitionMultiD>
              </acquisition>
            </nmrML>
        "#;

        let error = read_nmrml_2d_str(input).expect_err("dimension mismatch should fail");

        assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));
    }
}
