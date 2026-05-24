//! Shared helpers for `RSpin` CSV codecs.

use rspin_core::{Metadata, RSpinError, Result, Unit};

pub(crate) fn normalized_key(key: &str) -> String {
    key.chars()
        .filter(char::is_ascii_alphanumeric)
        .flat_map(char::to_lowercase)
        .collect()
}

pub(crate) fn parse_float(field: &'static str, value: &str) -> Result<f64> {
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

pub(crate) fn parse_unit(value: &str) -> Unit {
    match normalized_key(value).as_str() {
        "ppm" => Unit::Ppm,
        "hz" | "hertz" => Unit::Hertz,
        "seconds" | "second" | "sec" | "s" => Unit::Seconds,
        "points" | "point" => Unit::Points,
        _ => Unit::Arbitrary,
    }
}

pub(crate) fn unit_label(unit: Unit) -> &'static str {
    match unit {
        Unit::Ppm => "PPM",
        Unit::Hertz => "HZ",
        Unit::Seconds => "SECONDS",
        Unit::Points => "POINTS",
        _ => "ARBITRARY",
    }
}

pub(crate) fn push_comment(output: &mut String, key: &str, value: &str) {
    output.push_str("# ");
    output.push_str(key);
    output.push('=');
    output.push_str(value);
    output.push('\n');
}

pub(crate) fn push_metadata_property_comments(output: &mut String, metadata: &Metadata) {
    for (key, value) in &metadata.properties {
        push_comment(output, &format!("property.{key}"), value);
    }
}

pub(crate) fn push_metadata_comments(output: &mut String, metadata: &Metadata) {
    if let Some(name) = &metadata.name {
        push_comment(output, "name", name);
    }
    if let Some(nucleus) = &metadata.nucleus {
        push_comment(output, "nucleus", nucleus.as_label());
    }
    if let Some(frequency_mhz) = metadata.frequency_mhz {
        push_comment(output, "frequency_mhz", &format_float(frequency_mhz));
    }
    if let Some(solvent) = &metadata.solvent {
        push_comment(output, "solvent", solvent);
    }
    if let Some(temperature_k) = metadata.temperature_k {
        push_comment(output, "temperature_k", &format_float(temperature_k));
    }
    if let Some(origin) = &metadata.origin {
        push_comment(output, "origin", origin);
    }
    push_metadata_property_comments(output, metadata);
}

pub(crate) fn validate_metadata_numbers(metadata: &Metadata) -> Result<()> {
    if !metadata.frequency_mhz.is_none_or(f64::is_finite) {
        return Err(RSpinError::NonFinite {
            field: "frequency_mhz",
        });
    }
    if !metadata.temperature_k.is_none_or(f64::is_finite) {
        return Err(RSpinError::NonFinite {
            field: "temperature_k",
        });
    }
    Ok(())
}

pub(crate) fn apply_metadata_property_comment(
    metadata: &mut Metadata,
    key: &str,
    value: &str,
) -> Result<bool> {
    let key = key.trim();
    let Some(property_key) = key.strip_prefix("property.") else {
        return Ok(false);
    };
    if property_key.trim().is_empty() {
        return Err(RSpinError::Parse {
            format: "CSV",
            message: "metadata property comment requires a non-empty key".to_owned(),
        });
    }
    metadata
        .properties
        .insert(property_key.to_owned(), value.to_owned());
    Ok(true)
}

pub(crate) fn format_float(value: f64) -> String {
    let formatted = format!("{value:.12}");
    formatted
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_owned()
}
