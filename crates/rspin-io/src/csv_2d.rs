//! CSV support for two-dimensional spectra.

use std::str::FromStr;

use rspin_core::{Axis, Metadata, Nucleus, RSpinError, Result, Spectrum2D, Unit};

use crate::{
    SpectrumReader, SpectrumWriter,
    csv_common::{
        apply_metadata_property_comment, format_float, normalized_key, parse_float, parse_unit,
        push_comment, push_metadata_comments, unit_label, validate_metadata_numbers,
    },
};

/// Reader and writer for simple long-table two-dimensional CSV spectra.
#[derive(Clone, Copy, Debug, Default)]
pub struct CsvSpectrum2D;

impl SpectrumReader for CsvSpectrum2D {
    type Output = Spectrum2D;

    fn read_str(&self, input: &str) -> Result<Self::Output> {
        read_spectrum2d_csv(input)
    }
}

impl SpectrumWriter<Spectrum2D> for CsvSpectrum2D {
    fn write_string(&self, spectrum: &Spectrum2D) -> Result<String> {
        write_spectrum2d_csv(spectrum)
    }
}

#[derive(Default)]
struct Csv2DState {
    metadata: Metadata,
    x_unit: Unit,
    y_unit: Unit,
    saw_header: bool,
    expects_imaginary: Option<bool>,
    points: Vec<Csv2DPoint>,
}

#[derive(Clone, Copy, Debug)]
struct Csv2DPoint {
    x: f64,
    y: f64,
    intensity: f64,
    imaginary: Option<f64>,
}

/// Reads a two-dimensional spectrum from long-table CSV.
///
/// Comment metadata lines are optional and use `# key=value`. Data rows are
/// `x,y,intensity` or `x,y,intensity,imaginary`; one header row is allowed.
/// Rows must be grouped by y value, and each y row must repeat the same x-axis
/// values in the same order.
///
/// # Errors
///
/// Returns an error when rows are malformed, incomplete, non-rectangular, or
/// contain inconsistent imaginary columns.
pub fn read_spectrum2d_csv(input: &str) -> Result<Spectrum2D> {
    let mut state = Csv2DState::default();

    for (line_number, line) in input.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(comment) = trimmed.strip_prefix('#') {
            apply_comment(&mut state, comment)?;
            continue;
        }
        if !state.saw_header && is_header_row(trimmed) {
            state.saw_header = true;
            state.expects_imaginary = Some(header_has_imaginary(trimmed));
            continue;
        }
        parse_data_row(&mut state, trimmed, line_number + 1)?;
    }

    build_spectrum(state)
}

/// Writes a two-dimensional spectrum to long-table CSV.
///
/// # Errors
///
/// Returns an error when the spectrum contains non-finite values or inconsistent
/// internal dimensions.
pub fn write_spectrum2d_csv(spectrum: &Spectrum2D) -> Result<String> {
    validate_spectrum(spectrum)?;

    let mut output = String::new();
    push_comment(&mut output, "format", "RSpin CSV 2D");
    push_metadata_comments(&mut output, &spectrum.metadata);
    push_comment(&mut output, "x_unit", unit_label(spectrum.x.unit));
    push_comment(&mut output, "y_unit", unit_label(spectrum.y.unit));

    if spectrum.imaginary.is_some() {
        output.push_str("x,y,intensity,imaginary\n");
    } else {
        output.push_str("x,y,intensity\n");
    }

    let width = spectrum.x.len();
    for (y_index, y_value) in spectrum.y.values.iter().copied().enumerate() {
        for (x_index, x_value) in spectrum.x.values.iter().copied().enumerate() {
            let index = y_index * width + x_index;
            output.push_str(&format_float(x_value));
            output.push(',');
            output.push_str(&format_float(y_value));
            output.push(',');
            output.push_str(&format_float(spectrum.z[index]));
            if let Some(imaginary) = &spectrum.imaginary {
                output.push(',');
                output.push_str(&format_float(imaginary[index]));
            }
            output.push('\n');
        }
    }

    Ok(output)
}

fn apply_comment(state: &mut Csv2DState, comment: &str) -> Result<()> {
    let Some((key, value)) = comment.split_once('=') else {
        return Ok(());
    };
    if apply_metadata_property_comment(&mut state.metadata, key, value.trim())? {
        return Ok(());
    }
    let value = value.trim();
    match normalized_key(key).as_str() {
        "name" => state.metadata.name = Some(value.to_owned()),
        "nucleus" => state.metadata.nucleus = Some(Nucleus::from_str(value)?),
        "frequencymhz" => {
            state.metadata.frequency_mhz = Some(parse_float("frequency_mhz", value)?);
        }
        "solvent" => state.metadata.solvent = Some(value.to_owned()),
        "temperaturek" => {
            state.metadata.temperature_k = Some(parse_float("temperature_k", value)?);
        }
        "orig" | "origin" => state.metadata.origin = Some(value.to_owned()),
        "xunit" => state.x_unit = parse_unit(value),
        "yunit" => state.y_unit = parse_unit(value),
        _ => {}
    }
    Ok(())
}

fn parse_data_row(state: &mut Csv2DState, line: &str, line_number: usize) -> Result<()> {
    let columns = split_csv_row(line);
    if columns.len() != 3 && columns.len() != 4 {
        return Err(RSpinError::Parse {
            format: "CSV",
            message: format!("line {line_number}: expected 3 or 4 columns"),
        });
    }

    let has_imaginary = columns.len() == 4;
    match state.expects_imaginary {
        Some(expected) if expected != has_imaginary => {
            return Err(RSpinError::Parse {
                format: "CSV",
                message: format!("line {line_number}: inconsistent imaginary column"),
            });
        }
        Some(_) => {}
        None => state.expects_imaginary = Some(has_imaginary),
    }

    state.points.push(Csv2DPoint {
        x: parse_float("x", columns[0].as_str())?,
        y: parse_float("y", columns[1].as_str())?,
        intensity: parse_float("intensity", columns[2].as_str())?,
        imaginary: if has_imaginary {
            Some(parse_float("imaginary", columns[3].as_str())?)
        } else {
            None
        },
    });
    Ok(())
}

fn build_spectrum(state: Csv2DState) -> Result<Spectrum2D> {
    if state.points.is_empty() {
        return Err(RSpinError::Parse {
            format: "CSV",
            message: "missing data rows".to_owned(),
        });
    }

    let mut x_values = Vec::new();
    let mut y_values = Vec::new();
    let mut z = Vec::with_capacity(state.points.len());
    let mut imaginary = state
        .expects_imaginary
        .filter(|has_imaginary| *has_imaginary)
        .map(|_| Vec::with_capacity(state.points.len()));
    let mut current_y: Option<u64> = None;
    let mut row_x_index = 0usize;
    let mut width: Option<usize> = None;

    for point in state.points {
        let y_key = coordinate_key(point.y);
        if current_y != Some(y_key) {
            finish_previous_row(row_x_index, width)?;
            if y_values
                .iter()
                .copied()
                .any(|y_value| coordinate_key(y_value) == y_key)
            {
                return Err(RSpinError::Parse {
                    format: "CSV",
                    message: "2D CSV y rows must be contiguous".to_owned(),
                });
            }
            if width.is_none() && row_x_index > 0 {
                width = Some(row_x_index);
            }
            y_values.push(point.y);
            current_y = Some(y_key);
            row_x_index = 0;
        }

        match width {
            Some(expected_width) => {
                validate_x_position(point.x, &x_values, row_x_index, expected_width)?;
            }
            None => x_values.push(point.x),
        }
        z.push(point.intensity);
        if let Some(imaginary_values) = &mut imaginary {
            imaginary_values.push(point.imaginary.ok_or_else(|| RSpinError::Parse {
                format: "CSV",
                message: "missing imaginary value".to_owned(),
            })?);
        }
        row_x_index += 1;
    }
    finish_previous_row(row_x_index, width)?;

    Spectrum2D::new_complex(
        Axis::new("x", state.x_unit, x_values)?,
        Axis::new("y", state.y_unit, y_values)?,
        z,
        imaginary,
        state.metadata,
    )
}

fn finish_previous_row(row_x_index: usize, width: Option<usize>) -> Result<()> {
    if let Some(expected_width) = width
        && row_x_index != expected_width
    {
        return Err(RSpinError::Parse {
            format: "CSV",
            message: format!("2D CSV row has {row_x_index} points but expected {expected_width}"),
        });
    }
    Ok(())
}

fn validate_x_position(
    x_value: f64,
    x_values: &[f64],
    row_x_index: usize,
    expected_width: usize,
) -> Result<()> {
    if row_x_index >= expected_width {
        return Err(RSpinError::Parse {
            format: "CSV",
            message: format!("2D CSV row has more than {expected_width} points"),
        });
    }
    if coordinate_key(x_value) != coordinate_key(x_values[row_x_index]) {
        return Err(RSpinError::Parse {
            format: "CSV",
            message: "2D CSV rows must repeat the x axis in the same order".to_owned(),
        });
    }
    Ok(())
}

fn split_csv_row(line: &str) -> Vec<String> {
    line.split(',')
        .map(str::trim)
        .filter(|column| !column.is_empty())
        .map(str::to_owned)
        .collect()
}

fn is_header_row(line: &str) -> bool {
    split_csv_row(line)
        .iter()
        .any(|column| column.chars().any(char::is_alphabetic))
}

fn header_has_imaginary(line: &str) -> bool {
    split_csv_row(line)
        .iter()
        .map(|column| normalized_key(column))
        .any(|column| matches!(column.as_str(), "imaginary" | "imag" | "i"))
}

fn validate_spectrum(spectrum: &Spectrum2D) -> Result<()> {
    spectrum.metadata.validate()?;
    validate_metadata_numbers(&spectrum.metadata)?;
    let (width, height) = spectrum.shape();
    let expected_len = width
        .checked_mul(height)
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
    if !spectrum.x.values.iter().all(|value| value.is_finite())
        || !spectrum.y.values.iter().all(|value| value.is_finite())
        || !spectrum.z.iter().all(|value| value.is_finite())
    {
        return Err(RSpinError::NonFinite { field: "spectrum" });
    }
    if let Some(imaginary) = &spectrum.imaginary {
        if imaginary.len() != expected_len {
            return Err(RSpinError::InvalidSpectrum {
                message: format!(
                    "imaginary matrix has {} values but axes require {expected_len}",
                    imaginary.len()
                ),
            });
        }
        if !imaginary.iter().all(|value| value.is_finite()) {
            return Err(RSpinError::NonFinite { field: "imaginary" });
        }
    }
    Ok(())
}

fn coordinate_key(value: f64) -> u64 {
    let bits = value.to_bits();
    if bits == (-0.0_f64).to_bits() {
        0.0_f64.to_bits()
    } else {
        bits
    }
}

#[cfg(test)]
mod tests;
