use std::str::FromStr;

use rspin_core::{Metadata, Nucleus, RSpinError, Result, Unit};

use super::{RawJcamp, parse_float, parse_usize};

pub(super) fn apply_label(raw: &mut RawJcamp, key: &str, value: &str) -> Result<()> {
    match super::normalized_key(key).as_str() {
        "TITLE" => set_text(&mut raw.title, value),
        "ORIGIN" => set_text(&mut raw.origin, value),
        "SOLVENT" | "SOLVENTNAME" => set_text(&mut raw.solvent, value),
        "TEMPERATURE" | "TEMP" => raw.temperature_k = Some(parse_temperature_k(value)?),
        "FIRSTX" => raw.first_x = Some(parse_label_float("FIRSTX", value)?),
        "LASTX" => raw.last_x = Some(parse_label_float("LASTX", value)?),
        "NPOINTS" => raw.points = Some(parse_usize("NPOINTS", clean_label_value(value))?),
        "XFACTOR" => raw.x_factor = Some(parse_label_float("XFACTOR", value)?),
        "YFACTOR" => raw.y_factor = Some(parse_label_float("YFACTOR", value)?),
        "XUNITS" => raw.x_unit = parse_unit(value),
        "UNITS" => raw.x_unit = parse_unit(first_list_value(value)),
        "FACTOR" => apply_factor_label(raw, value)?,
        "FIRST" => apply_first_or_last_label(&mut raw.first_x, value)?,
        "LAST" => apply_first_or_last_label(&mut raw.last_x, value)?,
        "OBSERVENUCLEUS" => raw.nucleus = Some(parse_nucleus(value)?),
        "OBSERVEFREQUENCY" => {
            raw.frequency_mhz = Some(parse_label_float("OBSERVE FREQUENCY", value)?);
        }
        _ => {}
    }
    Ok(())
}

pub(super) fn apply_comment_assignment(raw: &mut RawJcamp, key: &str, value: &str) -> Result<()> {
    match super::normalized_key(key).as_str() {
        "ORIGIN" => set_text(&mut raw.origin, value),
        "SOLVENT" | "SOLVENTNAME" => set_text(&mut raw.solvent, value),
        "TEMPERATURE" | "TEMP" => raw.temperature_k = Some(parse_temperature_k(value)?),
        _ => {}
    }
    Ok(())
}

pub(super) fn metadata_from_raw(raw: &RawJcamp) -> Metadata {
    Metadata {
        name: raw.title.clone(),
        nucleus: raw.nucleus.clone(),
        frequency_mhz: raw.frequency_mhz,
        solvent: raw.solvent.clone(),
        temperature_k: raw.temperature_k,
        origin: raw.origin.clone(),
        ..Metadata::default()
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

fn set_text(target: &mut Option<String>, value: &str) {
    let value = clean_label_value(value);
    if !value.is_empty() {
        *target = Some(value.to_owned());
    }
}

fn parse_nucleus(value: &str) -> Result<Nucleus> {
    let value = clean_label_value(value).trim_start_matches('^');
    Nucleus::from_str(value)
}

fn parse_temperature_k(value: &str) -> Result<f64> {
    let numeric_value = parse_label_float("TEMPERATURE", value)?;
    let normalized = super::normalized_key(clean_label_value(value));
    let temperature_k = if normalized.contains("KELVIN") || normalized.ends_with('K') {
        numeric_value
    } else if normalized.contains("CELSIUS") || normalized.ends_with('C') || numeric_value < 170.0 {
        numeric_value + 273.15
    } else {
        numeric_value
    };

    if temperature_k.is_finite() {
        Ok(temperature_k)
    } else {
        Err(RSpinError::NonFinite {
            field: "TEMPERATURE",
        })
    }
}

fn parse_label_float(field: &'static str, value: &str) -> Result<f64> {
    let token = numeric_prefix(field, clean_label_value(value))?;
    parse_float(field, token)
}

fn numeric_prefix<'a>(field: &'static str, value: &'a str) -> Result<&'a str> {
    let value = value.trim();
    let end = match value.char_indices().find_map(|(index, character)| {
        if is_numeric_prefix_character(character) {
            None
        } else {
            Some(index)
        }
    }) {
        Some(index) => index,
        None => value.len(),
    };
    let token = value[..end].trim();
    if token.is_empty() {
        return Err(RSpinError::Parse {
            format: "JCAMP-DX",
            message: format!("{field}: expected numeric value"),
        });
    }
    Ok(token)
}

fn parse_numeric_list(field: &'static str, value: &str) -> Result<Vec<f64>> {
    clean_label_value(value)
        .split(',')
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(|token| parse_label_float(field, token))
        .collect()
}

fn list_value(values: &[f64], index: usize) -> Option<f64> {
    values.iter().copied().nth(index)
}

fn first_list_value(value: &str) -> &str {
    let value = clean_label_value(value);
    let mut values = value.split(',');
    if let Some(first) = values.next() {
        first.trim()
    } else {
        value
    }
}

fn clean_label_value(value: &str) -> &str {
    let mut parts = value.split("$$");
    if let Some(cleaned) = parts.next() {
        cleaned.trim()
    } else {
        value.trim()
    }
}

fn parse_unit(value: &str) -> Unit {
    match super::normalized_key(clean_label_value(value)).as_str() {
        "PPM" => Unit::Ppm,
        "HZ" | "HERTZ" => Unit::Hertz,
        "SECONDS" | "SECOND" | "SEC" | "S" => Unit::Seconds,
        "POINTS" | "POINT" => Unit::Points,
        _ => Unit::Arbitrary,
    }
}

fn is_numeric_prefix_character(character: char) -> bool {
    character.is_ascii_digit()
        || character == '.'
        || character == '+'
        || character == '-'
        || character == 'E'
        || character == 'e'
}
