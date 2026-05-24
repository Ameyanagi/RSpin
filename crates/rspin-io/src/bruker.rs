//! Bruker one-dimensional spectrum import.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use rspin_core::{Axis, Metadata, Nucleus, RSpinError, Result, Spectrum1D, Unit};
use serde::{Deserialize, Serialize};

use crate::{JcampDxVersion, SpectrumPathReader, parse_jcamp_dx_version};

mod processed_2d;
mod raw;

pub use processed_2d::{BrukerProcessed2D, read_bruker_processed_2d_dir};
pub use raw::{
    BrukerFid1D, BrukerFid1DBytes, BrukerSer2D, read_bruker_fid_1d_bytes, read_bruker_fid_1d_dir,
    read_bruker_ser_2d_dir,
};

/// Root metadata from a Bruker JCAMP-DX-style parameter file.
///
/// Bruker parameter files such as `acqus`, `acqu2s`, `procs`, and `proc2s`
/// often carry a `##JCAMPDX=` label alongside Bruker-specific `##$...`
/// parameters. `RSpin` exposes the parsed version so callers can inspect format
/// routing decisions before reading binary data.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrukerParameterFileInfo {
    /// Parsed `##JCAMPDX=` label, when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jcamp_dx_version: Option<JcampDxVersion>,
    /// Raw `##DATATYPE=` label, when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_type: Option<String>,
    /// Raw `##ORIGIN=` label, when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
    /// Raw `##OWNER=` label, when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
}

impl BrukerParameterFileInfo {
    /// Returns true when all parsed format versions are supported by current readers.
    #[must_use]
    pub fn is_supported_by_current_readers(&self) -> bool {
        self.jcamp_dx_version
            .as_ref()
            .is_none_or(JcampDxVersion::is_supported_by_current_reader)
    }

    /// Validates that parsed format versions are supported by current readers.
    ///
    /// # Errors
    ///
    /// Returns an unsupported-feature error when a parameter file declares a
    /// future or otherwise unsupported JCAMP-DX major version.
    pub fn validate_supported_by_current_readers(&self) -> Result<()> {
        match self.jcamp_dx_version.as_ref() {
            Some(version) => version.validate_supported_by_current_reader(),
            None => Ok(()),
        }
    }
}

/// Reader for Bruker processed one-dimensional datasets.
///
/// The reader accepts either the dataset root containing `pdata/1` or the
/// processed directory itself. It supports processed `1r` data and optional
/// `1i` data stored as 32-bit integers with Bruker `procs` metadata.
#[derive(Clone, Copy, Debug, Default)]
pub struct BrukerProcessed1D;

impl BrukerProcessed1D {
    /// Reads a processed one-dimensional spectrum from a Bruker dataset path.
    ///
    /// # Errors
    ///
    /// Returns an error when required `procs` or `1r` files are missing,
    /// optional `1i` data is malformed, or binary data uses an unsupported
    /// type.
    pub fn read_dir(self, path: impl AsRef<Path>) -> Result<Spectrum1D> {
        read_bruker_processed_1d_dir(path)
    }
}

impl SpectrumPathReader for BrukerProcessed1D {
    type Output = Spectrum1D;

    fn read_path(&self, path: &Path) -> Result<Self::Output> {
        read_bruker_processed_1d_dir(path)
    }
}

/// Byte-oriented reader for Bruker processed one-dimensional spectra.
///
/// The required inputs are `procs` text and real `1r` bytes. Optional `acqus`
/// text, imaginary `1i` bytes, and title text can be attached with chainable
/// builder methods.
#[derive(Clone, Copy, Debug)]
pub struct BrukerProcessed1DBytes<'a> {
    procs: &'a str,
    real_bytes: &'a [u8],
    acqus: Option<&'a str>,
    imaginary_bytes: Option<&'a [u8]>,
    title: Option<&'a str>,
}

impl<'a> BrukerProcessed1DBytes<'a> {
    /// Creates a byte-oriented Bruker processed 1D reader.
    #[must_use]
    pub fn new(procs: &'a str, real_bytes: &'a [u8]) -> Self {
        Self {
            procs,
            real_bytes,
            acqus: None,
            imaginary_bytes: None,
            title: None,
        }
    }

    /// Attaches optional `acqus` metadata text.
    #[must_use]
    pub fn with_acqus(mut self, acqus: &'a str) -> Self {
        self.acqus = Some(acqus);
        self
    }

    /// Attaches optional imaginary `1i` bytes.
    #[must_use]
    pub fn with_imaginary(mut self, imaginary_bytes: &'a [u8]) -> Self {
        self.imaginary_bytes = Some(imaginary_bytes);
        self
    }

    /// Attaches optional Bruker title text.
    #[must_use]
    pub fn with_title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    /// Reads the supplied bytes into a spectrum.
    ///
    /// # Errors
    ///
    /// Returns an error when metadata or binary data are missing, malformed, or
    /// unsupported.
    pub fn read(self) -> Result<Spectrum1D> {
        let procs = parse_parameter_file_for_reader(self.procs)?;
        let acqus = match self.acqus {
            Some(input) => Some(parse_parameter_file_for_reader(input)?),
            None => None,
        };
        let title = self.title.and_then(first_non_empty_line);
        let point_count = required_usize(&procs, "SI")?;
        let intensities = decode_processed_i32_data(self.real_bytes, point_count, &procs, "1r")?;
        let imaginary = match self.imaginary_bytes {
            Some(bytes) => Some(decode_processed_i32_data(bytes, point_count, &procs, "1i")?),
            None => None,
        };
        let axis = build_axis(&procs, point_count)?;
        let metadata = build_metadata(&procs, acqus.as_ref(), title)?;

        Spectrum1D::new_complex(axis, intensities, imaginary, metadata)
    }
}

/// Reads a processed one-dimensional spectrum from a Bruker dataset path.
///
/// The path may point to the dataset root or directly to `pdata/1`.
///
/// # Errors
///
/// Returns an error when required files are missing, malformed, or unsupported.
pub fn read_bruker_processed_1d_dir(path: impl AsRef<Path>) -> Result<Spectrum1D> {
    let input_path = path.as_ref();
    let processed_dir = locate_processed_dir(input_path);
    let procs_path = processed_dir.join("procs");
    let data_path = processed_dir.join("1r");

    let procs = parse_parameter_file_for_reader(&read_text(&procs_path, "Bruker procs")?)?;
    let acqus = read_acqus(input_path, &processed_dir)?;
    let title = read_title(&processed_dir)?;

    let point_count = required_usize(&procs, "SI")?;
    let intensities = read_processed_i32_data(&data_path, point_count, &procs)?;
    let imaginary = read_optional_processed_1d_imaginary(&processed_dir, point_count, &procs)?;
    let axis = build_axis(&procs, point_count)?;
    let metadata = build_metadata(&procs, acqus.as_ref(), title)?;

    Spectrum1D::new_complex(axis, intensities, imaginary, metadata)
}

/// Reads processed one-dimensional Bruker `1r` bytes with `procs` metadata.
///
/// For optional `acqus`, `1i`, or title metadata, use
/// [`BrukerProcessed1DBytes`].
///
/// # Errors
///
/// Returns an error when `procs` or binary data are malformed or unsupported.
pub fn read_bruker_processed_1d_bytes(procs: &str, real_bytes: &[u8]) -> Result<Spectrum1D> {
    BrukerProcessed1DBytes::new(procs, real_bytes).read()
}

fn locate_processed_dir(path: &Path) -> PathBuf {
    if path.join("procs").is_file() && path.join("1r").is_file() {
        path.to_path_buf()
    } else {
        path.join("pdata").join("1")
    }
}

fn read_acqus(input_path: &Path, processed_dir: &Path) -> Result<Option<BTreeMap<String, String>>> {
    let direct = input_path.join("acqus");
    if direct.is_file() {
        return read_optional_parameters(&direct, "Bruker acqus");
    }

    let root = match processed_dir.parent().and_then(Path::parent) {
        Some(root) => root.to_path_buf(),
        None => return Ok(None),
    };
    read_optional_parameters(&root.join("acqus"), "Bruker acqus")
}

fn read_optional_parameters(
    path: &Path,
    description: &'static str,
) -> Result<Option<BTreeMap<String, String>>> {
    if path.is_file() {
        read_text(path, description)
            .and_then(|text| parse_parameter_file_for_reader(&text).map(Some))
    } else {
        Ok(None)
    }
}

fn read_title(processed_dir: &Path) -> Result<Option<String>> {
    let path = processed_dir.join("title");
    if !path.is_file() {
        return Ok(None);
    }
    let text = read_text(&path, "Bruker title")?;
    Ok(first_non_empty_line(&text))
}

fn first_non_empty_line(text: &str) -> Option<String> {
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(str::to_owned)
}

fn parse_parameter_file(input: &str) -> BTreeMap<String, String> {
    let mut parameters = BTreeMap::new();
    for line in input.lines().map(str::trim) {
        let Some((key, value)) = parse_parameter_line(line) else {
            continue;
        };
        parameters.insert(normalized_key(key), clean_value(value));
    }
    parameters
}

/// Inspects root metadata from a Bruker parameter file.
///
/// # Errors
///
/// Returns an error when a declared `JCAMPDX` version label is malformed.
pub fn inspect_bruker_parameter_file(input: &str) -> Result<BrukerParameterFileInfo> {
    let parameters = parse_parameter_file(input);
    parameter_file_info(&parameters)
}

pub(super) fn parse_parameter_file_for_reader(input: &str) -> Result<BTreeMap<String, String>> {
    let parameters = parse_parameter_file(input);
    parameter_file_info(&parameters)?.validate_supported_by_current_readers()?;
    Ok(parameters)
}

fn parameter_file_info(parameters: &BTreeMap<String, String>) -> Result<BrukerParameterFileInfo> {
    let jcamp_dx_version = match text_parameter(parameters, "JCAMPDX") {
        Some(value) => Some(parse_jcamp_dx_version(&value)?),
        None => None,
    };
    Ok(BrukerParameterFileInfo {
        jcamp_dx_version,
        data_type: text_parameter(parameters, "DATATYPE"),
        origin: text_parameter(parameters, "ORIGIN"),
        owner: text_parameter(parameters, "OWNER"),
    })
}

fn parse_parameter_line(line: &str) -> Option<(&str, &str)> {
    let body = line.strip_prefix("##")?;
    let (key, value) = body.split_once('=')?;
    Some((key.trim_start_matches('$'), value.trim()))
}

fn normalized_key(key: &str) -> String {
    key.chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|character| character.to_ascii_uppercase())
        .collect()
}

fn clean_value(value: &str) -> String {
    let trimmed = value.trim();
    match trimmed
        .strip_prefix('<')
        .and_then(|value| value.strip_suffix('>'))
    {
        Some(inner) => inner.trim().to_owned(),
        None => trimmed.to_owned(),
    }
}

fn read_processed_i32_data(
    path: &Path,
    point_count: usize,
    procs: &BTreeMap<String, String>,
) -> Result<Vec<f64>> {
    let bytes = fs::read(path).map_err(|error| RSpinError::Parse {
        format: "Bruker",
        message: format!("failed to read {}: {error}", path.display()),
    })?;
    let plane = path
        .file_name()
        .and_then(std::ffi::OsStr::to_str)
        .map_or("processed data", |value| value);
    decode_processed_i32_data(&bytes, point_count, procs, plane)
}

fn decode_processed_i32_data(
    bytes: &[u8],
    point_count: usize,
    procs: &BTreeMap<String, String>,
    plane: &str,
) -> Result<Vec<f64>> {
    let data_type = optional_i32(procs, "DTYPP")?;
    if matches!(data_type, Some(value) if value != 0) {
        return Err(RSpinError::Unsupported {
            feature: "Bruker processed non-i32 data",
        });
    }

    let required_len = point_count
        .checked_mul(4)
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "Bruker point count is too large".to_owned(),
        })?;
    if bytes.len() < required_len {
        return Err(RSpinError::Parse {
            format: "Bruker",
            message: format!(
                "processed {plane} has {} bytes but {required_len} are required",
                bytes.len()
            ),
        });
    }

    let byte_order = optional_i32(procs, "BYTORDP")?;
    let scale = optional_i32(procs, "NCPROC")?.map_or(1.0, |value| 2_f64.powi(-value));
    let mut intensities = Vec::with_capacity(point_count);
    for chunk in bytes[..required_len].chunks_exact(4) {
        let raw = match byte_order {
            Some(1) => i32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]),
            _ => i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]),
        };
        intensities.push(f64::from(raw) * scale);
    }
    Ok(intensities)
}

fn read_optional_processed_1d_imaginary(
    processed_dir: &Path,
    point_count: usize,
    procs: &BTreeMap<String, String>,
) -> Result<Option<Vec<f64>>> {
    let path = processed_dir.join("1i");
    if path.is_file() {
        read_processed_i32_data(&path, point_count, procs).map(Some)
    } else {
        Ok(None)
    }
}

fn build_axis(procs: &BTreeMap<String, String>, point_count: usize) -> Result<Axis> {
    let start = optional_f64(procs, "OFFSET")?;
    let sweep_hz = optional_f64(procs, "SWP")?;
    let frequency_mhz = optional_f64(procs, "SF")?;

    match (start, sweep_hz, frequency_mhz) {
        (Some(start_ppm), Some(sweep_hz), Some(frequency_mhz)) if frequency_mhz != 0.0 => {
            let end_ppm = start_ppm - (sweep_hz / frequency_mhz);
            Axis::linear("chemical shift", Unit::Ppm, start_ppm, end_ppm, point_count)
        }
        _ => {
            let end = u32::try_from(point_count.saturating_sub(1)).map_or(0.0, f64::from);
            Axis::linear("point", Unit::Points, 0.0, end, point_count)
        }
    }
}

fn build_metadata(
    procs: &BTreeMap<String, String>,
    acqus: Option<&BTreeMap<String, String>>,
    title: Option<String>,
) -> Result<Metadata> {
    let nucleus = text_parameter(procs, "AXNUC")
        .or_else(|| acqus.and_then(|parameters| text_parameter(parameters, "NUC1")))
        .and_then(|value| Nucleus::from_str(&value).ok());
    let frequency_mhz = match optional_f64(procs, "SF")? {
        Some(value) => Some(value),
        None => match acqus {
            Some(parameters) => optional_f64(parameters, "SFO1")?,
            None => None,
        },
    };
    let solvent = acqus.and_then(|parameters| text_parameter(parameters, "SOLVENT"));
    let temperature_k = match acqus {
        Some(parameters) => optional_f64(parameters, "TE")?,
        None => None,
    };
    let origin = acqus
        .and_then(|parameters| text_parameter(parameters, "ORIGIN"))
        .or_else(|| acqus.and_then(|parameters| text_parameter(parameters, "OWNER")));

    Ok(Metadata {
        name: title,
        nucleus,
        frequency_mhz,
        solvent,
        temperature_k,
        origin,
        properties: processed_metadata_properties(procs, acqus),
        ..Metadata::default()
    })
}

fn processed_metadata_properties(
    procs: &BTreeMap<String, String>,
    acqus: Option<&BTreeMap<String, String>>,
) -> BTreeMap<String, String> {
    let mut properties = prefixed_parameter_properties("bruker.procs", procs);
    if let Some(acqus) = acqus {
        properties.extend(prefixed_parameter_properties("bruker.acqus", acqus));
    }
    properties
}

pub(super) fn prefixed_parameter_properties(
    prefix: &str,
    parameters: &BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    parameters
        .iter()
        .map(|(key, value)| (format!("{prefix}.{key}"), value.clone()))
        .collect()
}

fn text_parameter(parameters: &BTreeMap<String, String>, key: &str) -> Option<String> {
    parameters
        .get(key)
        .filter(|value| !value.trim().is_empty())
        .cloned()
}

fn required_usize(parameters: &BTreeMap<String, String>, key: &'static str) -> Result<usize> {
    let value = parameters.get(key).ok_or_else(|| RSpinError::Parse {
        format: "Bruker",
        message: format!("missing required parameter {key}"),
    })?;
    value.parse::<usize>().map_err(|error| RSpinError::Parse {
        format: "Bruker",
        message: format!("{key}: {error}"),
    })
}

fn optional_i32(parameters: &BTreeMap<String, String>, key: &'static str) -> Result<Option<i32>> {
    match parameters.get(key) {
        Some(value) => value
            .parse::<i32>()
            .map(Some)
            .map_err(|error| RSpinError::Parse {
                format: "Bruker",
                message: format!("{key}: {error}"),
            }),
        None => Ok(None),
    }
}

fn optional_f64(parameters: &BTreeMap<String, String>, key: &'static str) -> Result<Option<f64>> {
    match parameters.get(key) {
        Some(value) => {
            let parsed = value.parse::<f64>().map_err(|error| RSpinError::Parse {
                format: "Bruker",
                message: format!("{key}: {error}"),
            })?;
            if !parsed.is_finite() {
                return Err(RSpinError::NonFinite { field: key });
            }
            Ok(Some(parsed))
        }
        None => Ok(None),
    }
}

fn read_text(path: &Path, description: &'static str) -> Result<String> {
    fs::read_to_string(path).map_err(|error| RSpinError::Parse {
        format: "Bruker",
        message: format!(
            "failed to read {description} at {}: {error}",
            path.display()
        ),
    })
}

#[cfg(test)]
mod tests;
