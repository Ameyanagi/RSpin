//! JCAMP-DX one- and two-dimensional spectrum writers.

use rspin_core::{RSpinError, Result, Spectrum1D, Spectrum2D, Unit};

/// Writes a one-dimensional spectrum to a JCAMP-DX string.
///
/// Linear axes are represented as numeric `XYDATA`; non-uniform axes use
/// explicit `XYPOINTS` pairs so coordinates round-trip without resampling.
///
/// # Errors
///
/// Returns an error when the spectrum axis or data contains non-finite values.
pub fn write_jcamp_dx_1d(spectrum: &Spectrum1D) -> Result<String> {
    validate_finite_spectrum(spectrum)?;

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
    if let Some(origin) = spectrum.metadata.origin.as_deref() {
        push_label(&mut output, "ORIGIN", origin);
    }
    if let Some(solvent) = spectrum.metadata.solvent.as_deref() {
        push_label(&mut output, ".SOLVENT NAME", solvent);
    }
    if let Some(temperature_k) = spectrum.metadata.temperature_k {
        push_label(
            &mut output,
            "TEMPERATURE",
            &format!("{} K", format_float(temperature_k)),
        );
    }
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
    if spectrum.imaginary.is_some() {
        write_complex_data_tables(&mut output, spectrum, first_x, last_x)?;
    } else {
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
    }
    output.push_str("##END=\n");

    Ok(output)
}

/// Writes a two-dimensional spectrum to a JCAMP-DX string.
///
/// This writer emits a focused NTUPLES/page representation with one real page
/// per matrix row, and one matching imaginary page per row when the spectrum is
/// complex. The direct axis must be uniformly spaced because the matching
/// reader interprets the direct dimension from `FIRST` and `LAST` labels.
/// Non-uniform indirect axes are preserved through `PAGE=F1=...` labels.
///
/// # Errors
///
/// Returns an error when the spectrum contains non-finite values or uses a
/// non-uniform direct axis.
pub fn write_jcamp_dx_2d(spectrum: &Spectrum2D) -> Result<String> {
    validate_finite_spectrum_2d(spectrum)?;
    if !has_uniform_spacing(&spectrum.x.values) {
        return Err(RSpinError::InvalidSpectrum {
            message: "2D JCAMP-DX export requires a uniform x axis".to_owned(),
        });
    }

    let (width, height) = spectrum.shape();
    let first_x = axis_endpoint("first x value", &spectrum.x.values, Endpoint::First)?;
    let last_x = axis_endpoint("last x value", &spectrum.x.values, Endpoint::Last)?;
    let first_y = axis_endpoint("first y value", &spectrum.y.values, Endpoint::First)?;
    let last_y = axis_endpoint("last y value", &spectrum.y.values, Endpoint::Last)?;
    let first_z = data_endpoint("first z value", &spectrum.z, Endpoint::First)?;
    let last_z = data_endpoint("last z value", &spectrum.z, Endpoint::Last)?;
    let title = option_ref_or(spectrum.metadata.name.as_deref(), "untitled");

    let mut output = String::new();
    push_label(&mut output, "TITLE", title);
    push_label(&mut output, "JCAMP-DX", "5.00");
    push_label(&mut output, "DATA TYPE", "NMR SPECTRUM");
    push_label(&mut output, "DATA CLASS", "NTUPLES");
    write_common_metadata_labels(
        &mut output,
        spectrum.metadata.origin.as_deref(),
        spectrum.metadata.solvent.as_deref(),
        spectrum.metadata.temperature_k,
        spectrum.metadata.nucleus.as_ref(),
        spectrum.metadata.frequency_mhz,
    );
    push_label(
        &mut output,
        "UNITS",
        &format!(
            "{}, {}, ARBITRARY UNITS",
            unit_label(spectrum.x.unit),
            unit_label(spectrum.y.unit)
        ),
    );
    push_label(&mut output, "FACTOR", "1, 1, 1");
    push_label(
        &mut output,
        "FIRST",
        &format!(
            "{}, {}, {}",
            format_float(first_x),
            format_float(first_y),
            format_float(first_z)
        ),
    );
    push_label(
        &mut output,
        "LAST",
        &format!(
            "{}, {}, {}",
            format_float(last_x),
            format_float(last_y),
            format_float(last_z)
        ),
    );
    push_label(
        &mut output,
        "VAR_DIM",
        &format!("{width}, {height}, {width}"),
    );
    write_2d_data_pages(&mut output, spectrum, width, height);
    output.push_str("##END=\n");

    Ok(output)
}

fn validate_finite_spectrum(spectrum: &Spectrum1D) -> Result<()> {
    if !spectrum.x.values.iter().all(|value| value.is_finite())
        || !spectrum.intensities.iter().all(|value| value.is_finite())
    {
        return Err(RSpinError::NonFinite { field: "spectrum" });
    }
    if let Some(imaginary) = spectrum.imaginary.as_deref() {
        if imaginary.len() != spectrum.len() {
            return Err(RSpinError::InvalidSpectrum {
                message: format!(
                    "imaginary data has {} points but intensities have {} points",
                    imaginary.len(),
                    spectrum.len()
                ),
            });
        }
        if !imaginary.iter().all(|value| value.is_finite()) {
            return Err(RSpinError::NonFinite { field: "imaginary" });
        }
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

fn validate_finite_spectrum_2d(spectrum: &Spectrum2D) -> Result<()> {
    if !spectrum.x.values.iter().all(|value| value.is_finite())
        || !spectrum.y.values.iter().all(|value| value.is_finite())
        || !spectrum.z.iter().all(|value| value.is_finite())
    {
        return Err(RSpinError::NonFinite { field: "spectrum" });
    }
    if let Some(imaginary) = spectrum.imaginary.as_deref() {
        if imaginary.len() != spectrum.z.len() {
            return Err(RSpinError::InvalidSpectrum {
                message: format!(
                    "imaginary matrix has {} values but real matrix has {} values",
                    imaginary.len(),
                    spectrum.z.len()
                ),
            });
        }
        if !imaginary.iter().all(|value| value.is_finite()) {
            return Err(RSpinError::NonFinite { field: "imaginary" });
        }
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

fn write_common_metadata_labels(
    output: &mut String,
    origin: Option<&str>,
    solvent: Option<&str>,
    temperature_k: Option<f64>,
    nucleus: Option<&rspin_core::Nucleus>,
    frequency_mhz: Option<f64>,
) {
    if let Some(origin) = origin {
        push_label(output, "ORIGIN", origin);
    }
    if let Some(solvent) = solvent {
        push_label(output, ".SOLVENT NAME", solvent);
    }
    if let Some(temperature_k) = temperature_k {
        push_label(
            output,
            "TEMPERATURE",
            &format!("{} K", format_float(temperature_k)),
        );
    }
    if let Some(nucleus) = nucleus {
        push_label(output, "OBSERVE NUCLEUS", nucleus.as_label());
    }
    if let Some(frequency_mhz) = frequency_mhz {
        push_label(output, "OBSERVE FREQUENCY", &format_float(frequency_mhz));
    }
}

fn write_complex_data_tables(
    output: &mut String,
    spectrum: &Spectrum1D,
    first_x: f64,
    last_x: f64,
) -> Result<()> {
    if !has_uniform_spacing(&spectrum.x.values) {
        return Err(RSpinError::InvalidSpectrum {
            message: "complex JCAMP-DX export requires a uniform x axis".to_owned(),
        });
    }

    let imaginary = spectrum
        .imaginary
        .as_deref()
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "missing imaginary data for complex JCAMP-DX export".to_owned(),
        })?;
    let first_real = data_endpoint("first real value", &spectrum.intensities, Endpoint::First)?;
    let last_real = data_endpoint("last real value", &spectrum.intensities, Endpoint::Last)?;
    let first_imaginary = data_endpoint("first imaginary value", imaginary, Endpoint::First)?;
    let last_imaginary = data_endpoint("last imaginary value", imaginary, Endpoint::Last)?;

    push_label(output, "DATA CLASS", "NTUPLES");
    push_label(output, "NPOINTS", &spectrum.len().to_string());
    push_label(
        output,
        "UNITS",
        &format!(
            "{}, ARBITRARY UNITS, ARBITRARY UNITS",
            unit_label(spectrum.x.unit)
        ),
    );
    push_label(output, "FACTOR", "1, 1, 1");
    push_label(
        output,
        "FIRST",
        &format!(
            "{}, {}, {}",
            format_float(first_x),
            format_float(first_real),
            format_float(first_imaginary)
        ),
    );
    push_label(
        output,
        "LAST",
        &format!(
            "{}, {}, {}",
            format_float(last_x),
            format_float(last_real),
            format_float(last_imaginary)
        ),
    );
    write_data_table_page(
        output,
        "N=1",
        "R",
        &spectrum.x.values,
        &spectrum.intensities,
    );
    write_data_table_page(output, "N=2", "I", &spectrum.x.values, imaginary);
    Ok(())
}

fn write_2d_data_pages(output: &mut String, spectrum: &Spectrum2D, width: usize, height: usize) {
    let imaginary = spectrum.imaginary.as_deref();
    let real_channel = if imaginary.is_some() { "R" } else { "Y" };
    for y_index in 0..height {
        let y_value = spectrum.y.values[y_index];
        let row_start = y_index * width;
        let row_end = row_start + width;
        write_2d_data_page(
            output,
            y_value,
            real_channel,
            &spectrum.x.values,
            &spectrum.z[row_start..row_end],
        );
        if let Some(imaginary) = imaginary {
            write_2d_data_page(
                output,
                y_value,
                "I",
                &spectrum.x.values,
                &imaginary[row_start..row_end],
            );
        }
    }
}

fn write_2d_data_page(
    output: &mut String,
    y_value: f64,
    channel: &str,
    x_values: &[f64],
    values: &[f64],
) {
    push_label(output, "PAGE", &format!("F1={}", format_float(y_value)));
    push_label(
        output,
        "DATA TABLE",
        &format!("(X++({channel}..{channel})), XYDATA"),
    );
    for (x_value, value) in x_values.iter().copied().zip(values.iter().copied()) {
        output.push_str(&format_float(x_value));
        output.push(' ');
        output.push_str(&format_float(value));
        output.push('\n');
    }
}

fn write_data_table_page(
    output: &mut String,
    page: &str,
    channel: &str,
    x_values: &[f64],
    values: &[f64],
) {
    push_label(output, "PAGE", page);
    push_label(
        output,
        "DATA TABLE",
        &format!("(X++({channel}..{channel})), XYDATA"),
    );
    for (x_value, value) in x_values.iter().copied().zip(values.iter().copied()) {
        output.push_str(&format_float(x_value));
        output.push(' ');
        output.push_str(&format_float(value));
        output.push('\n');
    }
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

#[derive(Clone, Copy)]
enum Endpoint {
    First,
    Last,
}

fn data_endpoint(field: &'static str, values: &[f64], endpoint: Endpoint) -> Result<f64> {
    axis_endpoint(field, values, endpoint)
}

fn axis_endpoint(field: &'static str, values: &[f64], endpoint: Endpoint) -> Result<f64> {
    let value = match endpoint {
        Endpoint::First => values.first().copied(),
        Endpoint::Last => values.last().copied(),
    };
    value.ok_or_else(|| RSpinError::InvalidSpectrum {
        message: format!("missing {field}"),
    })
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
