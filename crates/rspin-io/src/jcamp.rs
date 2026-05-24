//! Minimal JCAMP-DX support for one-dimensional spectra.

use std::str::FromStr;

use rspin_core::{Axis, Metadata, Nucleus, RSpinError, Result, Spectrum1D, Unit};

use crate::{SpectrumReader, SpectrumWriter};

mod writer;

pub use writer::write_jcamp_dx_1d;

/// Reader and writer for a narrow, numeric JCAMP-DX 1D subset.
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

#[derive(Default)]
struct RawJcamp {
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
    xy_values: Vec<f64>,
    imaginary_values: Vec<f64>,
    xy_points: Vec<(f64, f64)>,
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
/// This parser targets numeric `XYDATA=(X++(Y..Y))`, `XYPOINTS=(XY..XY)`,
/// `PEAK TABLE=(XY..XY)`, and numeric NTUPLES `DATA TABLE` real/imaginary
/// pages. It applies JCAMP scaling factors to tabulated ordinates, and to
/// explicit `XYPOINTS`/peak-table abscissae. Richer compressed variants are
/// left for later format modules.
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
            apply_label(&mut raw, key, value)?;
            continue;
        }

        if line.starts_with("##") {
            data_block = None;
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

    let metadata = metadata_from_raw(&raw);
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
    let metadata = metadata_from_raw(&raw);
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

fn apply_label(raw: &mut RawJcamp, key: &str, value: &str) -> Result<()> {
    match normalized_key(key).as_str() {
        "TITLE" => raw.title = Some(value.trim().to_owned()),
        "FIRSTX" => raw.first_x = Some(parse_float("FIRSTX", value)?),
        "LASTX" => raw.last_x = Some(parse_float("LASTX", value)?),
        "NPOINTS" => raw.points = Some(parse_usize("NPOINTS", value)?),
        "XFACTOR" => raw.x_factor = Some(parse_float("XFACTOR", value)?),
        "YFACTOR" => raw.y_factor = Some(parse_float("YFACTOR", value)?),
        "XUNITS" => raw.x_unit = parse_unit(value),
        "UNITS" => raw.x_unit = parse_unit(first_list_value(value)),
        "FACTOR" => apply_factor_label(raw, value)?,
        "FIRST" => apply_first_or_last_label(&mut raw.first_x, value)?,
        "LAST" => apply_first_or_last_label(&mut raw.last_x, value)?,
        "OBSERVENUCLEUS" => raw.nucleus = Some(Nucleus::from_str(value.trim())?),
        "OBSERVEFREQUENCY" => raw.frequency_mhz = Some(parse_float("OBSERVE FREQUENCY", value)?),
        _ => {}
    }
    Ok(())
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

fn apply_factor_label(raw: &mut RawJcamp, value: &str) -> Result<()> {
    let values = parse_numeric_list("FACTOR", value)?;
    if let Some(value) = list_value(&values, 0) {
        raw.x_factor = Some(value);
    }
    if let Some(value) = list_value(&values, 1) {
        raw.y_factor = Some(value);
    }
    if let Some(value) = list_value(&values, 2) {
        raw.imaginary_y_factor = Some(value);
    }
    Ok(())
}

fn apply_first_or_last_label(target: &mut Option<f64>, value: &str) -> Result<()> {
    let values = parse_numeric_list("FIRST/LAST", value)?;
    if let Some(value) = list_value(&values, 0) {
        *target = Some(value);
    }
    Ok(())
}

fn parse_numeric_list(field: &'static str, value: &str) -> Result<Vec<f64>> {
    value
        .split(',')
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(|token| parse_float(field, token))
        .collect()
}

fn list_value(values: &[f64], index: usize) -> Option<f64> {
    values.iter().copied().nth(index)
}

fn first_list_value(value: &str) -> &str {
    let mut values = value.split(',');
    if let Some(first) = values.next() {
        first.trim()
    } else {
        value
    }
}

fn parse_labeled_line(line: &str) -> Option<(&str, &str)> {
    let without_prefix = line.strip_prefix("##")?;
    without_prefix.split_once('=')
}

fn normalized_key(key: &str) -> String {
    key.chars()
        .filter(char::is_ascii_alphanumeric)
        .flat_map(char::to_uppercase)
        .collect()
}

fn parse_xydata_line(line: &str, intensities: &mut Vec<f64>) -> Result<()> {
    let values = parse_numeric_tokens("XYDATA", line)?;

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
    value
        .trim()
        .parse::<usize>()
        .map_err(|error| RSpinError::Parse {
            format: "JCAMP-DX",
            message: format!("{field}: {error}"),
        })
}

fn scale_value(field: &'static str, value: f64, factor: f64) -> Result<f64> {
    let scaled = value * factor;
    if !scaled.is_finite() {
        return Err(RSpinError::NonFinite { field });
    }
    Ok(scaled)
}

fn metadata_from_raw(raw: &RawJcamp) -> Metadata {
    Metadata {
        name: raw.title.clone(),
        nucleus: raw.nucleus.clone(),
        frequency_mhz: raw.frequency_mhz,
        solvent: None,
        temperature_k: None,
        origin: None,
        molecules: Vec::new(),
    }
}

fn option_or<T>(value: Option<T>, default: T) -> T {
    let mut values = value.into_iter();
    if let Some(value) = values.next() {
        value
    } else {
        default
    }
}

fn parse_unit(value: &str) -> Unit {
    match normalized_key(value).as_str() {
        "PPM" => Unit::Ppm,
        "HZ" | "HERTZ" => Unit::Hertz,
        "SECONDS" | "SECOND" | "SEC" | "S" => Unit::Seconds,
        "POINTS" | "POINT" => Unit::Points,
        _ => Unit::Arbitrary,
    }
}

#[cfg(test)]
mod tests;
