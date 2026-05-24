//! `NMReDATA` SDF tag export.

use std::{collections::BTreeMap, fs, path::Path};

use rspin_core::{RSpinError, Result};

use super::{
    FORMAT, NmreDataAssignment, NmreDataCoupling, NmreDataRecord, NmreDataSignal1D,
    NmreDataSignal2D, NmreDataSpectrum, NmreDataTag,
};

/// Writes one `NMReDATA` record as SDF text.
///
/// Existing tags on [`NmreDataRecord::tags`] are written as the canonical
/// source of truth so unknown SDF tags and source ordering are preserved. If a
/// record has no tags, the writer synthesizes a focused `NMReDATA` tag set from
/// the parsed typed fields.
///
/// # Errors
///
/// Returns an error when tag names are empty, tag values contain embedded
/// newlines, or synthesized labels cannot be represented.
pub fn write_nmredata_record(record: &NmreDataRecord) -> Result<String> {
    let mut output = String::new();
    write_record(record, &mut output)?;
    Ok(output)
}

/// Writes multiple `NMReDATA` records as SDF text.
///
/// # Errors
///
/// Returns an error when `records` is empty or any record cannot be serialized.
pub fn write_nmredata_records(records: &[NmreDataRecord]) -> Result<String> {
    if records.is_empty() {
        return Err(RSpinError::Parse {
            format: FORMAT,
            message: "cannot write an empty NMReDATA record list".to_owned(),
        });
    }

    let mut output = String::new();
    for record in records {
        write_record(record, &mut output)?;
    }
    Ok(output)
}

/// Writes one `NMReDATA` record to an SDF file.
///
/// # Errors
///
/// Returns an error when serialization or filesystem writing fails.
pub fn write_nmredata_file(record: &NmreDataRecord, path: impl AsRef<Path>) -> Result<()> {
    let payload = write_nmredata_record(record)?;
    write_text(path.as_ref(), &payload)
}

/// Writes multiple `NMReDATA` records to an SDF file.
///
/// # Errors
///
/// Returns an error when serialization or filesystem writing fails.
pub fn write_nmredata_records_file(
    records: &[NmreDataRecord],
    path: impl AsRef<Path>,
) -> Result<()> {
    let payload = write_nmredata_records(records)?;
    write_text(path.as_ref(), &payload)
}

fn write_text(path: &Path, payload: &str) -> Result<()> {
    fs::write(path, payload).map_err(|error| RSpinError::Parse {
        format: FORMAT,
        message: format!("failed to write {}: {error}", path.display()),
    })
}

fn write_record(record: &NmreDataRecord, output: &mut String) -> Result<()> {
    if let Some(molfile) = record.molfile.as_deref() {
        output.push_str(molfile.trim_end());
        output.push('\n');
    }

    let tags = if record.tags.is_empty() {
        synthesized_tags(record)?
    } else {
        record.tags.clone()
    };
    for tag in &tags {
        write_tag(tag, output)?;
    }
    output.push_str("$$$$\n");
    Ok(())
}

fn write_tag(tag: &NmreDataTag, output: &mut String) -> Result<()> {
    validate_tag_name(&tag.name)?;
    output.push_str(">  <");
    output.push_str(&tag.name);
    output.push_str(">\n");
    for value in &tag.values {
        validate_value_line(value)?;
        output.push_str(value.trim());
        output.push_str("\\\n");
    }
    output.push('\n');
    Ok(())
}

fn synthesized_tags(record: &NmreDataRecord) -> Result<Vec<NmreDataTag>> {
    let mut tags = Vec::new();
    if let Some(version) = record.version.as_ref() {
        push_tag(&mut tags, "NMREDATA_VERSION", vec![version.raw.clone()]);
    }
    if let Some(level) = record.level {
        push_tag(&mut tags, "NMREDATA_LEVEL", vec![level.to_string()]);
    }
    if !record.id.is_empty() {
        push_tag(&mut tags, "NMREDATA_ID", key_value_lines(&record.id));
    }
    if let Some(formula) = record.formula.as_ref() {
        push_tag(&mut tags, "NMREDATA_FORMULA", vec![formula.clone()]);
    }
    if let Some(smiles) = record.smiles.as_ref() {
        push_tag(&mut tags, "NMREDATA_SMILES", vec![smiles.clone()]);
    }
    if let Some(solvent) = record.solvent.as_ref() {
        push_tag(&mut tags, "NMREDATA_SOLVENT", vec![solvent.clone()]);
    }
    if let Some(temperature_k) = record.temperature_k {
        push_tag(
            &mut tags,
            "NMREDATA_TEMPERATURE",
            vec![format!("{temperature_k} K")],
        );
    }
    let assignment_lines = assignment_lines(record)?;
    if !assignment_lines.is_empty() {
        push_tag(&mut tags, "NMREDATA_ASSIGNMENT", assignment_lines);
    }
    let coupling_lines = coupling_lines(record)?;
    if !coupling_lines.is_empty() {
        push_tag(&mut tags, "NMREDATA_J", coupling_lines);
    }
    for spectrum in &record.spectra {
        push_tag(&mut tags, &spectrum.tag, spectrum_lines(spectrum)?);
    }
    Ok(tags)
}

fn push_tag(tags: &mut Vec<NmreDataTag>, name: &str, values: Vec<String>) {
    tags.push(NmreDataTag {
        name: name.to_owned(),
        values,
    });
}

fn key_value_lines(values: &BTreeMap<String, String>) -> Vec<String> {
    values
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect()
}

fn assignment_lines(record: &NmreDataRecord) -> Result<Vec<String>> {
    let mut lines = Vec::new();
    for assignment in &record.assignments {
        lines.push(assignment_line(assignment)?);
    }
    lines.extend(record.assignment_equivalences.iter().cloned());
    Ok(lines)
}

fn assignment_line(assignment: &NmreDataAssignment) -> Result<String> {
    if !assignment.raw_line.trim().is_empty() {
        return Ok(assignment.raw_line.clone());
    }
    let mut fields = Vec::with_capacity(2 + assignment.atom_refs.len());
    fields.push(format_label(&assignment.label)?);
    fields.push(assignment.shift_ppm.to_string());
    for atom_ref in &assignment.atom_refs {
        fields.push(format_label(atom_ref)?);
    }
    Ok(fields.join(", "))
}

fn coupling_lines(record: &NmreDataRecord) -> Result<Vec<String>> {
    let mut lines = Vec::new();
    for coupling in &record.couplings {
        lines.push(coupling_line(coupling)?);
    }
    lines.extend(record.coupling_equivalences.iter().cloned());
    Ok(lines)
}

fn coupling_line(coupling: &NmreDataCoupling) -> Result<String> {
    if !coupling.raw_line.trim().is_empty() {
        return Ok(coupling.raw_line.clone());
    }
    Ok(format!(
        "{}, {}, {}",
        format_label(&coupling.from_label)?,
        format_label(&coupling.to_label)?,
        coupling.j_hz
    ))
}

fn spectrum_lines(spectrum: &NmreDataSpectrum) -> Result<Vec<String>> {
    let mut lines = Vec::new();
    for (key, values) in &spectrum.attributes {
        if let Some((first, rest)) = values.split_first() {
            let mut line = format!("{key}={first}");
            for value in rest {
                line.push_str(", ");
                line.push_str(&format_label(value)?);
            }
            lines.push(line);
        }
    }
    for signal in &spectrum.signals_1d {
        lines.push(signal_1d_line(signal)?);
    }
    for signal in &spectrum.signals_2d {
        lines.push(signal_2d_line(signal)?);
    }
    Ok(lines)
}

fn signal_1d_line(signal: &NmreDataSignal1D) -> Result<String> {
    if !signal.raw_line.trim().is_empty() {
        return Ok(signal.raw_line.clone());
    }
    let mut fields = vec![shift_or_range(signal.from_ppm, signal.to_ppm)];
    append_attribute_fields(&mut fields, &signal.attributes)?;
    for item in &signal.items {
        fields.push(format_label(item)?);
    }
    Ok(fields.join(", "))
}

fn signal_2d_line(signal: &NmreDataSignal2D) -> Result<String> {
    if !signal.raw_line.trim().is_empty() {
        return Ok(signal.raw_line.clone());
    }
    let mut fields = vec![format!(
        "{}/{}",
        format_label(&signal.left)?,
        format_label(&signal.right)?
    )];
    append_attribute_fields(&mut fields, &signal.attributes)?;
    for item in &signal.items {
        fields.push(format_label(item)?);
    }
    Ok(fields.join(", "))
}

fn append_attribute_fields(
    fields: &mut Vec<String>,
    attributes: &BTreeMap<String, Vec<String>>,
) -> Result<()> {
    for (key, values) in attributes {
        if let Some((first, rest)) = values.split_first() {
            fields.push(format!("{key}={first}"));
            for value in rest {
                fields.push(format_label(value)?);
            }
        }
    }
    Ok(())
}

fn shift_or_range(from_ppm: f64, to_ppm: Option<f64>) -> String {
    match to_ppm {
        Some(to_ppm) => format!("{from_ppm}-{to_ppm}"),
        None => from_ppm.to_string(),
    }
}

fn format_label(value: &str) -> Result<String> {
    validate_value_line(value)?;
    if value.contains('"') {
        return Err(RSpinError::Parse {
            format: FORMAT,
            message: "NMReDATA labels containing double quotes cannot be written".to_owned(),
        });
    }
    if value
        .chars()
        .any(|character| matches!(character, ',' | '/' | '\\' | '|' | '(' | ')' | '&'))
    {
        Ok(format!("<\"{value}\">"))
    } else {
        Ok(value.to_owned())
    }
}

fn validate_tag_name(name: &str) -> Result<()> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(RSpinError::Parse {
            format: FORMAT,
            message: "cannot write an empty SDF tag name".to_owned(),
        });
    }
    if trimmed.chars().any(char::is_whitespace) || trimmed.contains('<') || trimmed.contains('>') {
        return Err(RSpinError::Parse {
            format: FORMAT,
            message: format!("invalid SDF tag name: {name}"),
        });
    }
    Ok(())
}

fn validate_value_line(value: &str) -> Result<()> {
    if value.contains('\n') || value.contains('\r') {
        return Err(RSpinError::Parse {
            format: FORMAT,
            message: "NMReDATA tag values must not contain embedded newlines".to_owned(),
        });
    }
    Ok(())
}
