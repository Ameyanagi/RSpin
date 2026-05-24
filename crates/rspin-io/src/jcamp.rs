//! Minimal JCAMP-DX support for one- and two-dimensional spectra.

use rspin_core::{Axis, Nucleus, RSpinError, Result, Spectrum1D, Spectrum2D, Unit};
use serde::{Deserialize, Serialize};

use crate::{SpectrumReader, SpectrumWriter};

mod asdf;
mod labels;
mod two_d;
mod writer;

pub use two_d::{JcampDx2D, read_jcamp_dx_2d};
pub use writer::{write_jcamp_dx_1d, write_jcamp_dx_2d};

/// Reader and writer for a focused JCAMP-DX 1D subset.
#[derive(Clone, Copy, Debug, Default)]
pub struct JcampDx;

impl SpectrumReader for JcampDx {
    type Output = Spectrum1D;

    fn read_str(&self, input: &str) -> Result<Self::Output> {
        read_jcamp_dx_1d(input)
    }
}

impl SpectrumWriter<Spectrum1D> for JcampDx {
    fn write_string(&self, spectrum: &Spectrum1D) -> Result<String> {
        write_jcamp_dx_1d(spectrum)
    }
}

impl SpectrumWriter<Spectrum2D> for JcampDx2D {
    fn write_string(&self, spectrum: &Spectrum2D) -> Result<String> {
        write_jcamp_dx_2d(spectrum)
    }
}

#[derive(Default)]
struct RawJcamp {
    version: Option<JcampDxVersion>,
    title: Option<String>,
    first_x: Option<f64>,
    last_x: Option<f64>,
    points: Option<usize>,
    x_factor: Option<f64>,
    y_factor: Option<f64>,
    imaginary_y_factor: Option<f64>,
    x_unit: Unit,
    nucleus: Option<Nucleus>,
    frequency_mhz: Option<f64>,
    solvent: Option<String>,
    temperature_k: Option<f64>,
    origin: Option<String>,
    xy_values: Vec<f64>,
    imaginary_values: Vec<f64>,
    xy_points: Vec<(f64, f64)>,
}

/// Parsed JCAMP-DX version label.
///
/// JCAMP-DX files commonly use values such as `4.24` or `5.00`. `RSpin`
/// preserves the raw label value and exposes numeric components so readers can
/// reject unsupported future versions before interpreting data blocks.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JcampDxVersion {
    /// Raw version string after trimming comments and whitespace.
    pub raw: String,
    /// Major JCAMP-DX version.
    pub major: u32,
    /// Minor JCAMP-DX version.
    pub minor: u32,
    /// Optional patch/build component after `major.minor`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub patch: Option<u32>,
}

impl JcampDxVersion {
    /// Returns true when `RSpin`'s current JCAMP-DX reader supports this version.
    #[must_use]
    pub fn is_supported_by_current_reader(&self) -> bool {
        matches!(self.major, 4 | 5)
    }

    /// Validates that the version is supported by `RSpin`'s current reader.
    ///
    /// # Errors
    ///
    /// Returns an unsupported-feature error for future or otherwise unsupported
    /// JCAMP-DX major versions.
    pub fn validate_supported_by_current_reader(&self) -> Result<()> {
        if self.is_supported_by_current_reader() {
            Ok(())
        } else {
            Err(RSpinError::Unsupported {
                feature: "JCAMP-DX version",
            })
        }
    }
}

#[derive(Clone, Copy)]
enum DataBlock {
    XyData(Channel),
    XyPoints,
}

#[derive(Clone, Copy)]
enum Channel {
    Real,
    Imaginary,
}

/// Reads a one-dimensional spectrum from a JCAMP-DX string.
///
/// This parser targets numeric or ASDF-compressed `XYDATA=(X++(Y..Y))`,
/// `XYPOINTS=(XY..XY)`, `PEAK TABLE=(XY..XY)`, and numeric or
/// ASDF-compressed NTUPLES `DATA TABLE` real/imaginary pages. It applies JCAMP
/// scaling factors to tabulated ordinates, and to explicit `XYPOINTS`/peak-table
/// abscissae.
///
/// # Errors
///
/// Returns an error when required axis/data fields are missing or malformed.
pub fn read_jcamp_dx_1d(input: &str) -> Result<Spectrum1D> {
    let mut raw = RawJcamp::default();
    let mut data_block = None;

    for line in input.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if let Some((key, value)) = parse_labeled_line(line) {
            data_block = match normalized_key(key).as_str() {
                "XYDATA" => Some(DataBlock::XyData(Channel::Real)),
                "XYPOINTS" | "PEAKTABLE" => Some(DataBlock::XyPoints),
                "DATATABLE" => data_table_block(value),
                _ => None,
            };
            labels::apply_label(&mut raw, key, value)?;
            continue;
        }

        if line.starts_with("##") {
            data_block = None;
            continue;
        }

        if let Some((key, value)) = parse_comment_assignment(line) {
            labels::apply_comment_assignment(&mut raw, key, value)?;
            continue;
        }

        if line.starts_with('$') {
            continue;
        }

        match data_block {
            Some(DataBlock::XyData(Channel::Real)) => {
                parse_xydata_line(line, &mut raw.xy_values)?;
            }
            Some(DataBlock::XyData(Channel::Imaginary)) => {
                parse_xydata_line(line, &mut raw.imaginary_values)?;
            }
            Some(DataBlock::XyPoints) => parse_xypoints_line(line, &mut raw.xy_points)?,
            None => {}
        }
    }

    if !raw.xy_points.is_empty() {
        return spectrum_from_xypoints(raw);
    }

    let metadata = labels::metadata_from_raw(&raw);
    let y_factor = option_or(raw.y_factor, 1.0);
    let imaginary_y_factor = option_or(raw.imaginary_y_factor, y_factor);
    let intensity_limit = option_or(raw.points, raw.xy_values.len()).min(raw.xy_values.len());
    let intensities = raw
        .xy_values
        .into_iter()
        .take(intensity_limit)
        .map(|value| scale_value("XYDATA Y value", value, y_factor))
        .collect::<Result<Vec<_>>>()?;

    if intensities.is_empty() {
        return Err(RSpinError::Parse {
            format: "JCAMP-DX",
            message: "missing XYDATA values".to_owned(),
        });
    }

    let imaginary = if raw.imaginary_values.is_empty() {
        None
    } else {
        Some(
            raw.imaginary_values
                .into_iter()
                .take(intensities.len())
                .map(|value| scale_value("XYDATA imaginary value", value, imaginary_y_factor))
                .collect::<Result<Vec<_>>>()?,
        )
    };

    let first_x = option_or(raw.first_x, 0.0);
    let last_x = match raw.last_x {
        Some(value) => value,
        None => u32::try_from(intensities.len() - 1).map_or(0.0, f64::from),
    };
    let axis = Axis::linear("x", raw.x_unit, first_x, last_x, intensities.len())?;
    Spectrum1D::new_complex(axis, intensities, imaginary, metadata)
}

fn spectrum_from_xypoints(raw: RawJcamp) -> Result<Spectrum1D> {
    let metadata = labels::metadata_from_raw(&raw);
    let x_factor = option_or(raw.x_factor, 1.0);
    let y_factor = option_or(raw.y_factor, 1.0);
    let point_limit = option_or(raw.points, raw.xy_points.len()).min(raw.xy_points.len());
    let mut x_values = Vec::with_capacity(point_limit);
    let mut intensities = Vec::with_capacity(point_limit);

    for (x_value, intensity) in raw.xy_points.into_iter().take(point_limit) {
        x_values.push(scale_value("XYPOINTS X value", x_value, x_factor)?);
        intensities.push(scale_value("XYPOINTS Y value", intensity, y_factor)?);
    }

    if intensities.is_empty() {
        return Err(RSpinError::Parse {
            format: "JCAMP-DX",
            message: "missing XYPOINTS values".to_owned(),
        });
    }

    let axis = Axis::new("x", raw.x_unit, x_values)?;
    Spectrum1D::new(axis, intensities, metadata)
}

/// Parses a JCAMP-DX version label.
///
/// The parser accepts `major`, `major.minor`, and `major.minor.patch` numeric
/// labels and strips a trailing `$$` comment before parsing.
///
/// # Errors
///
/// Returns a parse error when the label is empty, contains a non-numeric
/// component, or has more than three numeric components.
pub fn parse_jcamp_dx_version(input: &str) -> Result<JcampDxVersion> {
    let raw = clean_label_value(input);
    if raw.is_empty() {
        return Err(RSpinError::Parse {
            format: "JCAMP-DX",
            message: "JCAMP-DX version: empty version label".to_owned(),
        });
    }

    let mut components = raw.split('.');
    let major = parse_version_component("JCAMP-DX major version", components.next())?;
    let minor = match components.next() {
        Some(component) => parse_version_component("JCAMP-DX minor version", Some(component))?,
        None => 0,
    };
    let patch = match components.next() {
        Some(component) => Some(parse_version_component(
            "JCAMP-DX patch version",
            Some(component),
        )?),
        None => None,
    };
    if components.next().is_some() {
        return Err(RSpinError::Parse {
            format: "JCAMP-DX",
            message: "JCAMP-DX version: expected at most three numeric components".to_owned(),
        });
    }

    Ok(JcampDxVersion {
        raw: raw.to_owned(),
        major,
        minor,
        patch,
    })
}

fn parse_version_component(field: &'static str, component: Option<&str>) -> Result<u32> {
    let component = component.ok_or_else(|| RSpinError::Parse {
        format: "JCAMP-DX",
        message: format!("{field}: missing component"),
    })?;
    if component.is_empty()
        || !component
            .chars()
            .all(|character| character.is_ascii_digit())
    {
        return Err(RSpinError::Parse {
            format: "JCAMP-DX",
            message: format!("{field}: expected a non-negative integer"),
        });
    }
    component.parse::<u32>().map_err(|error| RSpinError::Parse {
        format: "JCAMP-DX",
        message: format!("{field}: {error}"),
    })
}

fn clean_label_value(value: &str) -> &str {
    let mut parts = value.split("$$");
    if let Some(cleaned) = parts.next() {
        cleaned.trim()
    } else {
        value.trim()
    }
}

fn data_table_block(value: &str) -> Option<DataBlock> {
    let upper = value.to_ascii_uppercase();
    if !upper.contains("XYDATA") {
        return None;
    }
    if upper.contains("I..I") {
        Some(DataBlock::XyData(Channel::Imaginary))
    } else if upper.contains("R..R") || upper.contains("Y..Y") {
        Some(DataBlock::XyData(Channel::Real))
    } else {
        None
    }
}

fn parse_labeled_line(line: &str) -> Option<(&str, &str)> {
    let without_prefix = line.strip_prefix("##")?;
    without_prefix.split_once('=')
}

fn parse_comment_assignment(line: &str) -> Option<(&str, &str)> {
    let without_prefix = line.strip_prefix("$$")?;
    without_prefix.split_once('=')
}

fn normalized_key(key: &str) -> String {
    key.chars()
        .filter(char::is_ascii_alphanumeric)
        .flat_map(char::to_uppercase)
        .collect()
}

fn parse_xydata_line(line: &str, intensities: &mut Vec<f64>) -> Result<()> {
    let values = asdf::decode_values("XYDATA", line)?;

    if values.len() > 1 {
        intensities.extend(values.into_iter().skip(1));
    }
    Ok(())
}

fn parse_xypoints_line(line: &str, points: &mut Vec<(f64, f64)>) -> Result<()> {
    let values = parse_numeric_tokens("XYPOINTS", line)?;
    let mut pairs = values.chunks_exact(2);
    if !pairs.remainder().is_empty() {
        return Err(RSpinError::Parse {
            format: "JCAMP-DX",
            message: "XYPOINTS requires x/y numeric pairs".to_owned(),
        });
    }

    for pair in &mut pairs {
        points.push((pair[0], pair[1]));
    }
    Ok(())
}

fn parse_numeric_tokens(field: &'static str, line: &str) -> Result<Vec<f64>> {
    line.split(|character: char| {
        character.is_ascii_whitespace() || character == ',' || character == ';'
    })
    .filter(|token| !token.is_empty())
    .map(|token| parse_float(field, token))
    .collect()
}

fn parse_float(field: &'static str, value: &str) -> Result<f64> {
    let parsed = value
        .trim()
        .parse::<f64>()
        .map_err(|error| RSpinError::Parse {
            format: "JCAMP-DX",
            message: format!("{field}: {error}"),
        })?;
    if !parsed.is_finite() {
        return Err(RSpinError::NonFinite { field });
    }
    Ok(parsed)
}

fn parse_usize(field: &'static str, value: &str) -> Result<usize> {
    let value = value.trim();
    if let Ok(parsed) = value.parse::<usize>() {
        return Ok(parsed);
    }

    parse_decimal_usize(value).ok_or_else(|| RSpinError::Parse {
        format: "JCAMP-DX",
        message: format!("{field}: expected a non-negative integer"),
    })
}

fn parse_decimal_usize(value: &str) -> Option<usize> {
    let unsigned = if let Some(value) = value.strip_prefix('+') {
        value
    } else {
        value
    };
    let (whole, fraction) = unsigned.split_once('.')?;
    if whole.is_empty() || !fraction.chars().all(|character| character == '0') {
        return None;
    }
    whole.parse::<usize>().ok()
}

fn scale_value(field: &'static str, value: f64, factor: f64) -> Result<f64> {
    let scaled = value * factor;
    if !scaled.is_finite() {
        return Err(RSpinError::NonFinite { field });
    }
    Ok(scaled)
}

fn option_or<T>(value: Option<T>, default: T) -> T {
    let mut values = value.into_iter();
    if let Some(value) = values.next() {
        value
    } else {
        default
    }
}

#[cfg(test)]
mod tests;
