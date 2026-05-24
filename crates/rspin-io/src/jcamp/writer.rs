//! JCAMP-DX one-dimensional spectrum writer.

use rspin_core::{RSpinError, Result, Spectrum1D, Unit};

/// Writes a one-dimensional spectrum to a JCAMP-DX string.
///
/// Linear axes are represented as numeric `XYDATA`; non-uniform axes use
/// explicit `XYPOINTS` pairs so coordinates round-trip without resampling.
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

    let title = option_ref_or(spectrum.metadata.name.as_deref(), "untitled");
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
    if has_uniform_spacing(&spectrum.x.values) {
        write_xydata_block(&mut output, spectrum);
    } else {
        write_xypoints_block(&mut output, spectrum);
    }
    output.push_str("##END=\n");

    Ok(output)
}

fn write_xydata_block(output: &mut String, spectrum: &Spectrum1D) {
    push_label(output, "XYDATA", "(X++(Y..Y))");
    for (x_value, intensity) in spectrum.points() {
        output.push_str(&format_float(x_value));
        output.push(' ');
        output.push_str(&format_float(intensity));
        output.push('\n');
    }
}

fn write_xypoints_block(output: &mut String, spectrum: &Spectrum1D) {
    push_label(output, "XYPOINTS", "(XY..XY)");
    for (x_value, intensity) in spectrum.points() {
        output.push_str(&format_float(x_value));
        output.push(' ');
        output.push_str(&format_float(intensity));
        output.push('\n');
    }
}

fn has_uniform_spacing(values: &[f64]) -> bool {
    if values.len() <= 2 {
        return true;
    }
    let first_step = values[1] - values[0];
    values.windows(2).skip(1).all(|pair| {
        let step = pair[1] - pair[0];
        let tolerance = 1.0e-10 * first_step.abs().max(step.abs()).max(1.0);
        (step - first_step).abs() <= tolerance
    })
}

fn option_ref_or<'a>(value: Option<&'a str>, default: &'a str) -> &'a str {
    let mut values = value.into_iter();
    if let Some(value) = values.next() {
        value
    } else {
        default
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
    use super::has_uniform_spacing;

    #[test]
    fn detects_uniform_spacing_with_tolerance() {
        assert!(has_uniform_spacing(&[1.0, 2.0]));
        assert!(has_uniform_spacing(&[0.0, 0.1, 0.2 + 1.0e-13, 0.3]));
        assert!(!has_uniform_spacing(&[0.0, 0.1, 0.22, 0.3]));
    }
}
