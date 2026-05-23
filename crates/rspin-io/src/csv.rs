//! CSV support for one-dimensional spectra.

use std::str::FromStr;

use rspin_core::{Axis, Metadata, Nucleus, RSpinError, Result, Spectrum1D, Unit};

use crate::{SpectrumReader, SpectrumWriter};

/// Reader and writer for simple one-dimensional CSV spectra.
#[derive(Clone, Copy, Debug, Default)]
pub struct CsvSpectrum1D;

impl SpectrumReader for CsvSpectrum1D {
    type Output = Spectrum1D;

    fn read_str(&self, input: &str) -> Result<Self::Output> {
        read_spectrum1d_csv(input)
    }
}

impl SpectrumWriter<Spectrum1D> for CsvSpectrum1D {
    fn write_string(&self, spectrum: &Spectrum1D) -> Result<String> {
        write_spectrum1d_csv(spectrum)
    }
}

#[derive(Default)]
struct CsvState {
    metadata: Metadata,
    x_unit: Unit,
    saw_header: bool,
    expects_imaginary: Option<bool>,
    x_values: Vec<f64>,
    intensities: Vec<f64>,
    imaginary: Vec<f64>,
}

/// Reads a one-dimensional spectrum from CSV.
///
/// Comment metadata lines are optional and use `# key=value`. Data rows are
/// `x,intensity` or `x,intensity,imaginary`; one header row is allowed.
///
/// # Errors
///
/// Returns an error when rows are malformed, numeric values are non-finite, or
/// imaginary columns are inconsistent.
pub fn read_spectrum1d_csv(input: &str) -> Result<Spectrum1D> {
    let mut state = CsvState::default();

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

    if state.x_values.is_empty() {
        return Err(RSpinError::Parse {
            format: "CSV",
            message: "missing data rows".to_owned(),
        });
    }

    let imaginary = if matches!(state.expects_imaginary, Some(true)) {
        Some(state.imaginary)
    } else {
        None
    };
    Spectrum1D::new_complex(
        Axis::new("x", state.x_unit, state.x_values)?,
        state.intensities,
        imaginary,
        state.metadata,
    )
}

/// Writes a one-dimensional spectrum to CSV.
///
/// # Errors
///
/// Returns an error when the spectrum contains non-finite values.
pub fn write_spectrum1d_csv(spectrum: &Spectrum1D) -> Result<String> {
    validate_spectrum(spectrum)?;

    let mut output = String::new();
    push_comment(&mut output, "format", "RSpin CSV 1D");
    if let Some(name) = &spectrum.metadata.name {
        push_comment(&mut output, "name", name);
    }
    if let Some(nucleus) = &spectrum.metadata.nucleus {
        push_comment(&mut output, "nucleus", nucleus.as_label());
    }
    if let Some(frequency_mhz) = spectrum.metadata.frequency_mhz {
        push_comment(&mut output, "frequency_mhz", &format_float(frequency_mhz));
    }
    push_comment(&mut output, "x_unit", unit_label(spectrum.x.unit));

    if spectrum.imaginary.is_some() {
        output.push_str("x,intensity,imaginary\n");
    } else {
        output.push_str("x,intensity\n");
    }

    for (index, (x_value, intensity)) in spectrum.points().enumerate() {
        output.push_str(&format_float(x_value));
        output.push(',');
        output.push_str(&format_float(intensity));
        if let Some(imaginary) = &spectrum.imaginary {
            output.push(',');
            output.push_str(&format_float(imaginary[index]));
        }
        output.push('\n');
    }

    Ok(output)
}

fn apply_comment(state: &mut CsvState, comment: &str) -> Result<()> {
    let Some((key, value)) = comment.split_once('=') else {
        return Ok(());
    };
    let value = value.trim();
    match normalized_key(key).as_str() {
        "name" => state.metadata.name = Some(value.to_owned()),
        "nucleus" => state.metadata.nucleus = Some(Nucleus::from_str(value)?),
        "frequencymhz" => {
            state.metadata.frequency_mhz = Some(parse_float("frequency_mhz", value)?);
        }
        "solvent" => state.metadata.solvent = Some(value.to_owned()),
        "orig" | "origin" => state.metadata.origin = Some(value.to_owned()),
        "xunit" => state.x_unit = parse_unit(value),
        _ => {}
    }
    Ok(())
}

fn parse_data_row(state: &mut CsvState, line: &str, line_number: usize) -> Result<()> {
    let columns = split_csv_row(line);
    if columns.len() != 2 && columns.len() != 3 {
        return Err(RSpinError::Parse {
            format: "CSV",
            message: format!("line {line_number}: expected 2 or 3 columns"),
        });
    }

    let has_imaginary = columns.len() == 3;
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

    state.x_values.push(parse_float("x", columns[0].as_str())?);
    state
        .intensities
        .push(parse_float("intensity", columns[1].as_str())?);
    if has_imaginary {
        state
            .imaginary
            .push(parse_float("imaginary", columns[2].as_str())?);
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

fn validate_spectrum(spectrum: &Spectrum1D) -> Result<()> {
    if !spectrum.x.values.iter().all(|value| value.is_finite())
        || !spectrum.intensities.iter().all(|value| value.is_finite())
    {
        return Err(RSpinError::NonFinite { field: "spectrum" });
    }
    if let Some(imaginary) = &spectrum.imaginary {
        if !imaginary.iter().all(|value| value.is_finite()) {
            return Err(RSpinError::NonFinite { field: "imaginary" });
        }
    }
    Ok(())
}

fn normalized_key(key: &str) -> String {
    key.chars()
        .filter(char::is_ascii_alphanumeric)
        .flat_map(char::to_lowercase)
        .collect()
}

fn parse_float(field: &'static str, value: &str) -> Result<f64> {
    let parsed = value
        .trim()
        .parse::<f64>()
        .map_err(|error| RSpinError::Parse {
            format: "CSV",
            message: format!("{field}: {error}"),
        })?;
    if !parsed.is_finite() {
        return Err(RSpinError::NonFinite { field });
    }
    Ok(parsed)
}

fn parse_unit(value: &str) -> Unit {
    match normalized_key(value).as_str() {
        "ppm" => Unit::Ppm,
        "hz" | "hertz" => Unit::Hertz,
        "seconds" | "second" | "sec" | "s" => Unit::Seconds,
        "points" | "point" => Unit::Points,
        _ => Unit::Arbitrary,
    }
}

fn unit_label(unit: Unit) -> &'static str {
    match unit {
        Unit::Ppm => "PPM",
        Unit::Hertz => "HZ",
        Unit::Seconds => "SECONDS",
        Unit::Points => "POINTS",
        _ => "ARBITRARY",
    }
}

fn push_comment(output: &mut String, key: &str, value: &str) {
    output.push_str("# ");
    output.push_str(key);
    output.push('=');
    output.push_str(value);
    output.push('\n');
}

fn format_float(value: f64) -> String {
    let formatted = format!("{value:.12}");
    formatted
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_owned()
}

#[cfg(test)]
mod tests {
    use rspin_core::{Metadata, Unit};

    use super::*;

    #[test]
    fn round_trips_real_spectrum_with_trait_api() -> anyhow::Result<()> {
        let mut metadata = Metadata::named("demo");
        metadata.nucleus = Some(Nucleus::Hydrogen1);
        metadata.frequency_mhz = Some(400.0);
        let spectrum = Spectrum1D::new(
            Axis::linear("shift", Unit::Ppm, 10.0, 8.0, 3)?,
            vec![1.0, 2.5, 3.0],
            metadata,
        )?;

        let codec = CsvSpectrum1D;
        let text = codec.write_string(&spectrum)?;
        let parsed = codec.read_str(&text)?;

        assert_eq!(parsed.metadata.name.as_deref(), Some("demo"));
        assert_eq!(parsed.metadata.nucleus, Some(Nucleus::Hydrogen1));
        assert_eq!(parsed.metadata.frequency_mhz, Some(400.0));
        assert_eq!(parsed.x.unit, Unit::Ppm);
        assert_eq!(parsed.x.values, spectrum.x.values);
        assert_eq!(parsed.intensities, spectrum.intensities);
        assert_eq!(parsed.imaginary, None);
        Ok(())
    }

    #[test]
    fn reads_imaginary_column() -> anyhow::Result<()> {
        let input = "\
# name=complex
# x_unit=HZ
x,intensity,imaginary
1,2,0.5
2,3,-0.25
";
        let spectrum = read_spectrum1d_csv(input)?;

        assert_eq!(spectrum.metadata.name.as_deref(), Some("complex"));
        assert_eq!(spectrum.x.unit, Unit::Hertz);
        assert_eq!(spectrum.x.values, vec![1.0, 2.0]);
        assert_eq!(spectrum.intensities, vec![2.0, 3.0]);
        assert_eq!(spectrum.imaginary, Some(vec![0.5, -0.25]));
        Ok(())
    }

    #[test]
    fn reads_data_without_header() -> anyhow::Result<()> {
        let spectrum = read_spectrum1d_csv("1,2\n2,3\n")?;

        assert_eq!(spectrum.x.values, vec![1.0, 2.0]);
        assert_eq!(spectrum.intensities, vec![2.0, 3.0]);
        Ok(())
    }

    #[test]
    fn rejects_inconsistent_imaginary_columns() {
        let error = read_spectrum1d_csv("x,intensity\n1,2\n2,3,4\n")
            .expect_err("mixed columns should fail");
        assert!(matches!(error, RSpinError::Parse { .. }));
    }
}
