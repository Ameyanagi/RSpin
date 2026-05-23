//! Minimal JCAMP-DX support for one-dimensional spectra.

use std::str::FromStr;

use rspin_core::{Axis, Metadata, Nucleus, RSpinError, Result, Spectrum1D, Unit};

use crate::{SpectrumReader, SpectrumWriter};

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
    x_unit: Unit,
    nucleus: Option<Nucleus>,
    frequency_mhz: Option<f64>,
    xy_values: Vec<f64>,
}

/// Reads a one-dimensional spectrum from a JCAMP-DX string.
///
/// This parser targets the common `XYDATA=(X++(Y..Y))` numeric subset. It is
/// intentionally strict about finite numeric data and leaves richer variants
/// for later format modules.
///
/// # Errors
///
/// Returns an error when required axis/data fields are missing or malformed.
pub fn read_jcamp_dx_1d(input: &str) -> Result<Spectrum1D> {
    let mut raw = RawJcamp::default();
    let mut in_xydata = false;

    for line in input.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if let Some((key, value)) = parse_labeled_line(line) {
            in_xydata = normalized_key(key) == "XYDATA";
            apply_label(&mut raw, key, value)?;
            continue;
        }

        if in_xydata && !line.starts_with('$') {
            parse_xydata_line(line, &mut raw.xy_values)?;
        }
    }

    let intensities = match raw.points {
        Some(points) => raw.xy_values.into_iter().take(points).collect::<Vec<_>>(),
        None => raw.xy_values,
    };

    if intensities.is_empty() {
        return Err(RSpinError::Parse {
            format: "JCAMP-DX",
            message: "missing XYDATA values".to_owned(),
        });
    }

    let first_x = raw.first_x.unwrap_or(0.0);
    let last_x = raw
        .last_x
        .unwrap_or_else(|| u32::try_from(intensities.len() - 1).map_or(0.0, f64::from));
    let axis = Axis::linear("x", raw.x_unit, first_x, last_x, intensities.len())?;
    let metadata = Metadata {
        name: raw.title,
        nucleus: raw.nucleus,
        frequency_mhz: raw.frequency_mhz,
        solvent: None,
        temperature_k: None,
        origin: None,
    };

    Spectrum1D::new(axis, intensities, metadata)
}

/// Writes a one-dimensional spectrum to a JCAMP-DX string.
///
/// # Errors
///
/// Returns an error when the spectrum axis or data contains non-finite values.
pub fn write_jcamp_dx_1d(spectrum: &Spectrum1D) -> Result<String> {
    if !spectrum.x.values.iter().all(|value| value.is_finite())
        || !spectrum.intensities.iter().all(|value| value.is_finite())
    {
        return Err(RSpinError::NonFinite { field: "spectrum" });
    }

    let title = spectrum.metadata.name.as_deref().unwrap_or("untitled");
    let first_x =
        spectrum
            .x
            .values
            .first()
            .copied()
            .ok_or_else(|| RSpinError::InvalidSpectrum {
                message: "missing x axis values".to_owned(),
            })?;
    let last_x = spectrum
        .x
        .values
        .last()
        .copied()
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "missing x axis values".to_owned(),
        })?;

    let mut output = String::new();
    push_label(&mut output, "TITLE", title);
    push_label(&mut output, "JCAMP-DX", "5.00");
    push_label(&mut output, "DATA TYPE", "NMR SPECTRUM");
    if let Some(nucleus) = &spectrum.metadata.nucleus {
        push_label(&mut output, "OBSERVE NUCLEUS", nucleus.as_label());
    }
    if let Some(frequency_mhz) = spectrum.metadata.frequency_mhz {
        push_label(
            &mut output,
            "OBSERVE FREQUENCY",
            &format_float(frequency_mhz),
        );
    }
    push_label(&mut output, "XUNITS", unit_label(spectrum.x.unit));
    push_label(&mut output, "YUNITS", "ARBITRARY UNITS");
    push_label(&mut output, "FIRSTX", &format_float(first_x));
    push_label(&mut output, "LASTX", &format_float(last_x));
    push_label(&mut output, "NPOINTS", &spectrum.len().to_string());
    push_label(&mut output, "XYDATA", "(X++(Y..Y))");
    for (x_value, intensity) in spectrum.points() {
        output.push_str(&format_float(x_value));
        output.push(' ');
        output.push_str(&format_float(intensity));
        output.push('\n');
    }
    output.push_str("##END=\n");

    Ok(output)
}

fn apply_label(raw: &mut RawJcamp, key: &str, value: &str) -> Result<()> {
    match normalized_key(key).as_str() {
        "TITLE" => raw.title = Some(value.trim().to_owned()),
        "FIRSTX" => raw.first_x = Some(parse_float("FIRSTX", value)?),
        "LASTX" => raw.last_x = Some(parse_float("LASTX", value)?),
        "NPOINTS" => raw.points = Some(parse_usize("NPOINTS", value)?),
        "XUNITS" => raw.x_unit = parse_unit(value),
        "OBSERVENUCLEUS" => raw.nucleus = Some(Nucleus::from_str(value.trim())?),
        "OBSERVEFREQUENCY" => raw.frequency_mhz = Some(parse_float("OBSERVE FREQUENCY", value)?),
        _ => {}
    }
    Ok(())
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
    let values = line
        .split(|character: char| {
            character.is_ascii_whitespace() || character == ',' || character == ';'
        })
        .filter(|token| !token.is_empty())
        .map(|token| parse_float("XYDATA", token))
        .collect::<Result<Vec<_>>>()?;

    if values.len() > 1 {
        intensities.extend(values.into_iter().skip(1));
    }
    Ok(())
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

fn parse_unit(value: &str) -> Unit {
    match normalized_key(value).as_str() {
        "PPM" => Unit::Ppm,
        "HZ" | "HERTZ" => Unit::Hertz,
        "SECONDS" | "SECOND" | "SEC" | "S" => Unit::Seconds,
        "POINTS" | "POINT" => Unit::Points,
        _ => Unit::Arbitrary,
    }
}

fn unit_label(unit: Unit) -> &'static str {
    match unit {
        Unit::Ppm => "PPM",
        Unit::Hertz => "HZ",
        Unit::Seconds => "SECONDS",
        Unit::Points => "POINTS",
        _ => "ARBITRARY UNITS",
    }
}

fn push_label(output: &mut String, label: &str, value: &str) {
    output.push_str("##");
    output.push_str(label);
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
    use super::*;

    #[test]
    fn reads_xydata_spectrum() -> anyhow::Result<()> {
        let input = "\
##TITLE=ethyl sample
##JCAMP-DX=5.00
##DATA TYPE=NMR SPECTRUM
##OBSERVE NUCLEUS=1H
##OBSERVE FREQUENCY=400
##XUNITS=PPM
##FIRSTX=10
##LASTX=8
##NPOINTS=4
##XYDATA=(X++(Y..Y))
10 1 2
9 3 4
##END=
";
        let spectrum = read_jcamp_dx_1d(input)?;

        assert_eq!(spectrum.metadata.name.as_deref(), Some("ethyl sample"));
        assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Hydrogen1));
        assert_eq!(spectrum.metadata.frequency_mhz, Some(400.0));
        assert_axis_close(
            &spectrum.x.values,
            &[10.0, 9.333_333_333_333_334, 8.666_666_666_666_666, 8.0],
        );
        assert_eq!(spectrum.intensities, vec![1.0, 2.0, 3.0, 4.0]);
        Ok(())
    }

    #[test]
    fn rejects_missing_xydata() {
        let error = read_jcamp_dx_1d("##TITLE=empty\n").expect_err("missing data should fail");
        assert!(matches!(error, RSpinError::Parse { .. }));
    }

    #[test]
    fn writes_readable_xydata_spectrum() -> anyhow::Result<()> {
        let x = Axis::linear("shift", Unit::Ppm, 1.0, 3.0, 3)?;
        let mut metadata = Metadata::named("demo");
        metadata.nucleus = Some(Nucleus::Hydrogen1);
        metadata.frequency_mhz = Some(400.0);
        let spectrum = Spectrum1D::new(x, vec![2.0, 4.0, 8.0], metadata)?;

        let text = write_jcamp_dx_1d(&spectrum)?;
        assert!(text.contains("##TITLE=demo"));
        assert!(text.contains("##OBSERVE NUCLEUS=1H"));

        let parsed = read_jcamp_dx_1d(&text)?;
        assert_eq!(parsed.metadata.name.as_deref(), Some("demo"));
        assert_eq!(parsed.intensities, spectrum.intensities);
        assert_eq!(parsed.x.values, spectrum.x.values);
        Ok(())
    }

    #[test]
    fn supports_trait_api() -> anyhow::Result<()> {
        let codec = JcampDx;
        let x = Axis::linear("shift", Unit::Ppm, 0.0, 1.0, 2)?;
        let spectrum = Spectrum1D::new(x, vec![5.0, 6.0], Metadata::default())?;
        let text = codec.write_string(&spectrum)?;
        let parsed = codec.read_str(&text)?;
        assert_eq!(parsed.intensities, vec![5.0, 6.0]);
        Ok(())
    }

    fn assert_axis_close(actual: &[f64], expected: &[f64]) {
        assert_eq!(actual.len(), expected.len());
        for (left, right) in actual.iter().zip(expected) {
            assert!((left - right).abs() < 1e-12, "{left} != {right}");
        }
    }
}
