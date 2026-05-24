//! `NMReDATA` SDF tag import.

use std::{collections::BTreeMap, fs, path::Path, str::FromStr};

use rspin_core::{Nucleus, RSpinError, Result};
use serde::{Deserialize, Serialize};

use crate::{SpectrumReader, SpectrumWriter};

mod analysis;
mod writer;

pub use analysis::{
    NmreDataAnalysis, nmredata_assignments_to_assignment_set,
    nmredata_couplings_to_j_coupling_graph, nmredata_to_analysis,
};
pub use writer::{
    write_nmredata_file, write_nmredata_record, write_nmredata_records, write_nmredata_records_file,
};

const FORMAT: &str = "NMReDATA";

/// Reader for `NMReDATA` SDF tag records.
#[derive(Clone, Copy, Debug, Default)]
pub struct NmreData;

/// Reader for multi-record `NMReDATA` SDF payloads.
#[derive(Clone, Copy, Debug, Default)]
pub struct NmreDataRecords;

impl NmreData {
    /// Reads the first `NMReDATA` record from an SDF string.
    ///
    /// # Errors
    ///
    /// Returns an error when the payload contains no records or the first
    /// record has malformed `NMReDATA` tags.
    pub fn read_str(self, input: &str) -> Result<NmreDataRecord> {
        read_nmredata_str(input)
    }

    /// Reads all `NMReDATA` records from an SDF string.
    ///
    /// # Errors
    ///
    /// Returns an error when the payload contains no records or any record has
    /// malformed `NMReDATA` tags.
    pub fn read_records_str(self, input: &str) -> Result<Vec<NmreDataRecord>> {
        read_nmredata_records_str(input)
    }

    /// Reads the first `NMReDATA` record from a file.
    ///
    /// # Errors
    ///
    /// Returns an error when the file cannot be read or its first record has
    /// malformed `NMReDATA` tags.
    pub fn read_file(self, path: impl AsRef<Path>) -> Result<NmreDataRecord> {
        read_nmredata_file(path)
    }

    /// Reads the first `NMReDATA` record from UTF-8 bytes.
    ///
    /// # Errors
    ///
    /// Returns an error when the payload is not UTF-8, contains no records, or
    /// has malformed `NMReDATA` tags.
    pub fn read_bytes(self, bytes: &[u8]) -> Result<NmreDataRecord> {
        read_nmredata_bytes(bytes)
    }

    /// Writes one `NMReDATA` record as SDF text.
    ///
    /// # Errors
    ///
    /// Returns an error when the record contains invalid tag names or values.
    pub fn write_string(self, record: &NmreDataRecord) -> Result<String> {
        write_nmredata_record(record)
    }

    /// Writes multiple `NMReDATA` records as SDF text.
    ///
    /// # Errors
    ///
    /// Returns an error when the record list is empty or any record contains
    /// invalid tag names or values.
    pub fn write_records_string(self, records: &[NmreDataRecord]) -> Result<String> {
        write_nmredata_records(records)
    }

    /// Writes one `NMReDATA` record to an SDF file.
    ///
    /// # Errors
    ///
    /// Returns an error when serialization or filesystem writing fails.
    pub fn write_file(self, record: &NmreDataRecord, path: impl AsRef<Path>) -> Result<()> {
        write_nmredata_file(record, path)
    }

    /// Writes multiple `NMReDATA` records to an SDF file.
    ///
    /// # Errors
    ///
    /// Returns an error when serialization or filesystem writing fails.
    pub fn write_records_file(
        self,
        records: &[NmreDataRecord],
        path: impl AsRef<Path>,
    ) -> Result<()> {
        write_nmredata_records_file(records, path)
    }
}

impl SpectrumReader for NmreData {
    type Output = NmreDataRecord;

    fn read_str(&self, input: &str) -> Result<Self::Output> {
        read_nmredata_str(input)
    }
}

impl SpectrumWriter<NmreDataRecord> for NmreData {
    fn write_string(&self, record: &NmreDataRecord) -> Result<String> {
        write_nmredata_record(record)
    }
}

impl SpectrumWriter<[NmreDataRecord]> for NmreData {
    fn write_string(&self, records: &[NmreDataRecord]) -> Result<String> {
        write_nmredata_records(records)
    }
}

impl NmreDataRecords {
    /// Reads all `NMReDATA` records from an SDF string.
    ///
    /// # Errors
    ///
    /// Returns an error when the payload contains no records or any record has
    /// malformed `NMReDATA` tags.
    pub fn read_str(self, input: &str) -> Result<Vec<NmreDataRecord>> {
        read_nmredata_records_str(input)
    }

    /// Reads all `NMReDATA` records from a file.
    ///
    /// # Errors
    ///
    /// Returns an error when the file cannot be read, contains no records, or
    /// any record has malformed `NMReDATA` tags.
    pub fn read_file(self, path: impl AsRef<Path>) -> Result<Vec<NmreDataRecord>> {
        read_nmredata_records_file(path)
    }

    /// Reads all `NMReDATA` records from UTF-8 bytes.
    ///
    /// # Errors
    ///
    /// Returns an error when the payload is not UTF-8, contains no records, or
    /// any record has malformed `NMReDATA` tags.
    pub fn read_bytes(self, bytes: &[u8]) -> Result<Vec<NmreDataRecord>> {
        read_nmredata_records_bytes(bytes)
    }

    /// Writes multiple `NMReDATA` records as SDF text.
    ///
    /// # Errors
    ///
    /// Returns an error when the record list is empty or any record contains
    /// invalid tag names or values.
    pub fn write_string(self, records: &[NmreDataRecord]) -> Result<String> {
        write_nmredata_records(records)
    }

    /// Writes multiple `NMReDATA` records to an SDF file.
    ///
    /// # Errors
    ///
    /// Returns an error when serialization or filesystem writing fails.
    pub fn write_file(self, records: &[NmreDataRecord], path: impl AsRef<Path>) -> Result<()> {
        write_nmredata_records_file(records, path)
    }
}

impl SpectrumReader for NmreDataRecords {
    type Output = Vec<NmreDataRecord>;

    fn read_str(&self, input: &str) -> Result<Self::Output> {
        read_nmredata_records_str(input)
    }
}

impl SpectrumWriter<[NmreDataRecord]> for NmreDataRecords {
    fn write_string(&self, records: &[NmreDataRecord]) -> Result<String> {
        write_nmredata_records(records)
    }
}

/// Parsed `NMReDATA` version.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NmreDataVersion {
    /// Raw version string after trimming.
    pub raw: String,
    /// Major version component.
    pub major: u32,
    /// Minor version component, when present.
    pub minor: Option<u32>,
    /// Remaining qualifier/build text, when present.
    pub qualifier: Option<String>,
}

/// One SDF tag and its cleaned `NMReDATA` values.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NmreDataTag {
    /// Raw tag name, preserving any suffix such as `#2`.
    pub name: String,
    /// Cleaned non-empty tag lines with comments and trailing continuations removed.
    pub values: Vec<String>,
}

impl NmreDataTag {
    /// Returns the tag name without any `#number` suffix.
    #[must_use]
    pub fn base_name(&self) -> &str {
        base_tag_name(&self.name)
    }
}

/// One parsed `NMReDATA` SDF record.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NmreDataRecord {
    /// Molfile block before the first SDF tag, when present.
    pub molfile: Option<String>,
    /// All `NMReDATA` and non-`NMReDATA` tags in source order.
    pub tags: Vec<NmreDataTag>,
    /// Parsed `<NMREDATA_VERSION>`, when present.
    pub version: Option<NmreDataVersion>,
    /// Parsed `<NMREDATA_LEVEL>`, when present.
    pub level: Option<u8>,
    /// Parsed key/value lines from `<NMREDATA_ID>`.
    pub id: BTreeMap<String, String>,
    /// Parsed `<NMREDATA_FORMULA>`, when present.
    pub formula: Option<String>,
    /// Parsed `<NMREDATA_SMILES>`, when present.
    pub smiles: Option<String>,
    /// Parsed `<NMREDATA_SOLVENT>`, when present.
    pub solvent: Option<String>,
    /// Parsed sample temperature in kelvin, when present.
    pub temperature_k: Option<f64>,
    /// Parsed chemical-shift assignments from `<NMREDATA_ASSIGNMENT>`.
    pub assignments: Vec<NmreDataAssignment>,
    /// Raw equivalence declarations from `<NMREDATA_ASSIGNMENT>`.
    pub assignment_equivalences: Vec<String>,
    /// Parsed scalar couplings from `<NMREDATA_J>`.
    pub couplings: Vec<NmreDataCoupling>,
    /// Raw equivalence declarations from `<NMREDATA_J>`.
    pub coupling_equivalences: Vec<String>,
    /// Parsed 1D/2D spectrum tag summaries.
    pub spectra: Vec<NmreDataSpectrum>,
}

impl NmreDataRecord {
    /// Finds the first tag with a case-insensitive exact name.
    #[must_use]
    pub fn tag(&self, name: &str) -> Option<&NmreDataTag> {
        self.tags
            .iter()
            .find(|tag| tag.name.eq_ignore_ascii_case(name))
    }

    /// Returns all tags whose base name matches case-insensitively.
    #[must_use]
    pub fn tags_named(&self, base_name: &str) -> Vec<&NmreDataTag> {
        self.tags
            .iter()
            .filter(|tag| tag.base_name().eq_ignore_ascii_case(base_name))
            .collect()
    }
}

/// One `NMReDATA` chemical-shift assignment.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NmreDataAssignment {
    /// Assignment label.
    pub label: String,
    /// Chemical shift in ppm.
    pub shift_ppm: f64,
    /// Atom references as written by the file, for example `1`, `H3`, or `C1`.
    pub atom_refs: Vec<String>,
    /// Cleaned source line.
    pub raw_line: String,
}

/// One `NMReDATA` scalar coupling.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NmreDataCoupling {
    /// First assigned label.
    pub from_label: String,
    /// Second assigned label.
    pub to_label: String,
    /// Scalar coupling in Hz.
    pub j_hz: f64,
    /// Cleaned source line.
    pub raw_line: String,
}

/// Parsed `NMReDATA` spectrum tag kind.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NmreDataSpectrumKind {
    /// One-dimensional spectrum tag.
    OneD {
        /// Raw observed-nucleus label from the tag.
        observed_label: String,
        /// Parsed observed nucleus, when recognized by `RSpin`.
        observed_nucleus: Option<Nucleus>,
    },
    /// Two-dimensional spectrum tag.
    TwoD {
        /// Raw indirect-dimension nucleus label.
        indirect_label: String,
        /// Parsed indirect-dimension nucleus, when recognized by `RSpin`.
        indirect_nucleus: Option<Nucleus>,
        /// Raw mixing/code segment from the tag.
        mixing: String,
        /// Raw direct-dimension nucleus label.
        direct_label: String,
        /// Parsed direct-dimension nucleus, when recognized by `RSpin`.
        direct_nucleus: Option<Nucleus>,
    },
    /// `NMReDATA` spectrum-like tag that does not match the standard 1D/2D shapes yet.
    Other {
        /// Descriptor text after `NMREDATA_`.
        descriptor: String,
    },
}

/// Parsed `NMReDATA` spectrum tag summary.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NmreDataSpectrum {
    /// Raw SDF tag name.
    pub tag: String,
    /// Parsed spectrum kind.
    pub kind: NmreDataSpectrumKind,
    /// Spectrum-level attributes such as `Larmor`, `Pulseprogram`, and `Spectrum_Location`.
    pub attributes: BTreeMap<String, Vec<String>>,
    /// Parsed larmor frequency in MHz from the `Larmor` attribute, when present.
    pub larmor_mhz: Option<f64>,
    /// Spectrum locations from `Spectrum_Location`, when present.
    pub spectrum_locations: Vec<String>,
    /// Parsed one-dimensional signals.
    pub signals_1d: Vec<NmreDataSignal1D>,
    /// Parsed two-dimensional correlations.
    pub signals_2d: Vec<NmreDataSignal2D>,
}

/// Parsed one-dimensional `NMReDATA` signal.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NmreDataSignal1D {
    /// Signal start or scalar shift in ppm.
    pub from_ppm: f64,
    /// Signal range end in ppm, when the source line used a range.
    pub to_ppm: Option<f64>,
    /// Signal attributes such as `S`, `N`, `L`, `J`, and `I`.
    pub attributes: BTreeMap<String, Vec<String>>,
    /// Positional values that were not attached to an attribute key.
    pub items: Vec<String>,
    /// Cleaned source line.
    pub raw_line: String,
}

/// Parsed two-dimensional `NMReDATA` signal/correlation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NmreDataSignal2D {
    /// Left side of the `left/right` correlation field.
    pub left: String,
    /// Right side of the `left/right` correlation field.
    pub right: String,
    /// Signal attributes such as `I`, `S`, and coupling annotations.
    pub attributes: BTreeMap<String, Vec<String>>,
    /// Positional values that were not attached to an attribute key.
    pub items: Vec<String>,
    /// Cleaned source line.
    pub raw_line: String,
}

/// Reads the first `NMReDATA` SDF record from a file.
///
/// # Errors
///
/// Returns an error when the file cannot be read, contains no records, or has
/// malformed `NMReDATA` tags.
pub fn read_nmredata_file(path: impl AsRef<Path>) -> Result<NmreDataRecord> {
    let records = read_nmredata_records_file(path)?;
    first_record(records)
}

/// Reads the first `NMReDATA` SDF record from UTF-8 bytes.
///
/// # Errors
///
/// Returns an error when the payload is not UTF-8, contains no records, or has
/// malformed `NMReDATA` tags.
pub fn read_nmredata_bytes(bytes: &[u8]) -> Result<NmreDataRecord> {
    let records = read_nmredata_records_bytes(bytes)?;
    first_record(records)
}

/// Reads the first `NMReDATA` SDF record from a string.
///
/// # Errors
///
/// Returns an error when the payload contains no records or has malformed
/// `NMReDATA` tags.
pub fn read_nmredata_str(input: &str) -> Result<NmreDataRecord> {
    let records = read_nmredata_records_str(input)?;
    first_record(records)
}

fn first_record(records: Vec<NmreDataRecord>) -> Result<NmreDataRecord> {
    records.into_iter().next().ok_or_else(|| RSpinError::Parse {
        format: FORMAT,
        message: "missing NMReDATA record".to_owned(),
    })
}

/// Reads all `NMReDATA` SDF records from a file.
///
/// # Errors
///
/// Returns an error when the file cannot be read, contains no records, or any
/// record has malformed `NMReDATA` tags.
pub fn read_nmredata_records_file(path: impl AsRef<Path>) -> Result<Vec<NmreDataRecord>> {
    let path = path.as_ref();
    let input = fs::read_to_string(path).map_err(|error| RSpinError::Parse {
        format: FORMAT,
        message: format!("failed to read {}: {error}", path.display()),
    })?;
    read_nmredata_records_str(&input)
}

/// Reads all `NMReDATA` SDF records from UTF-8 bytes.
///
/// # Errors
///
/// Returns an error when the payload is not UTF-8, contains no records, or any
/// record has malformed `NMReDATA` tags.
pub fn read_nmredata_records_bytes(bytes: &[u8]) -> Result<Vec<NmreDataRecord>> {
    let input = std::str::from_utf8(bytes).map_err(|error| RSpinError::Parse {
        format: FORMAT,
        message: format!("input is not valid UTF-8: {error}"),
    })?;
    read_nmredata_records_str(input)
}

/// Reads all `NMReDATA` SDF records from a string.
///
/// # Errors
///
/// Returns an error when the payload contains no records or any record has
/// malformed `NMReDATA` tags.
pub fn read_nmredata_records_str(input: &str) -> Result<Vec<NmreDataRecord>> {
    let mut records = Vec::new();
    let mut lines = Vec::new();

    for line in input.lines() {
        if line.trim() == "$$$$" {
            push_record(&mut records, &lines)?;
            lines.clear();
        } else {
            lines.push(line.to_owned());
        }
    }

    if lines.iter().any(|line| !line.trim().is_empty()) {
        push_record(&mut records, &lines)?;
    }

    if records.is_empty() {
        return Err(RSpinError::Parse {
            format: FORMAT,
            message: "missing NMReDATA record".to_owned(),
        });
    }

    Ok(records)
}

/// Parses an `NMReDATA` version string.
///
/// # Errors
///
/// Returns a parse error when the major component is absent or non-numeric.
pub fn parse_nmredata_version(version: &str) -> Result<NmreDataVersion> {
    let raw = version.trim();
    if raw.is_empty() {
        return Err(RSpinError::Parse {
            format: FORMAT,
            message: "empty NMReDATA version".to_owned(),
        });
    }

    let mut parts = raw.split('.');
    let major_text = parts.next().ok_or_else(|| RSpinError::Parse {
        format: FORMAT,
        message: "missing NMReDATA major version".to_owned(),
    })?;
    let major = parse_u32_component("major", major_text)?;
    let minor = match parts.next() {
        Some(minor_text) => Some(parse_u32_component("minor", minor_text)?),
        None => None,
    };
    let rest: Vec<&str> = parts.collect();
    let qualifier = if rest.is_empty() {
        None
    } else if rest.iter().any(|part| part.trim().is_empty()) {
        return Err(RSpinError::Parse {
            format: FORMAT,
            message: "empty NMReDATA version qualifier".to_owned(),
        });
    } else {
        Some(rest.join("."))
    };

    Ok(NmreDataVersion {
        raw: raw.to_owned(),
        major,
        minor,
        qualifier,
    })
}

fn push_record(records: &mut Vec<NmreDataRecord>, lines: &[String]) -> Result<()> {
    if lines.iter().all(|line| line.trim().is_empty()) {
        return Ok(());
    }
    records.push(parse_record(lines)?);
    Ok(())
}

fn parse_record(lines: &[String]) -> Result<NmreDataRecord> {
    let mut molfile_lines = Vec::new();
    let mut tags = Vec::new();
    let mut active_name: Option<String> = None;
    let mut active_values = Vec::new();

    for line in lines {
        if line.trim_start().starts_with('>') {
            flush_tag(&mut tags, &mut active_name, &mut active_values);
            active_name = Some(parse_tag_header(line)?);
        } else if active_name.is_some() {
            active_values.push(line.clone());
        } else {
            molfile_lines.push(line.clone());
        }
    }

    flush_tag(&mut tags, &mut active_name, &mut active_values);
    let molfile = if molfile_lines.iter().any(|line| !line.trim().is_empty()) {
        Some(molfile_lines.join("\n"))
    } else {
        None
    };

    let mut record = NmreDataRecord {
        molfile,
        tags,
        ..NmreDataRecord::default()
    };
    apply_known_tags(&mut record)?;
    Ok(record)
}

fn flush_tag(
    tags: &mut Vec<NmreDataTag>,
    active_name: &mut Option<String>,
    active_values: &mut Vec<String>,
) {
    if let Some(name) = active_name.take() {
        let values = active_values
            .drain(..)
            .map(|line| clean_nmredata_line(&line))
            .filter(|line| !line.is_empty())
            .collect();
        tags.push(NmreDataTag { name, values });
    }
}

fn parse_tag_header(line: &str) -> Result<String> {
    let start = line.find('<').ok_or_else(|| RSpinError::Parse {
        format: FORMAT,
        message: format!("malformed SDF tag header: {}", line.trim()),
    })?;
    let end = line[start + 1..]
        .find('>')
        .map(|offset| start + 1 + offset)
        .ok_or_else(|| RSpinError::Parse {
            format: FORMAT,
            message: format!("malformed SDF tag header: {}", line.trim()),
        })?;
    let name = line[start + 1..end].trim();
    if name.is_empty() {
        return Err(RSpinError::Parse {
            format: FORMAT,
            message: "empty SDF tag name".to_owned(),
        });
    }
    Ok(name.to_owned())
}

fn apply_known_tags(record: &mut NmreDataRecord) -> Result<()> {
    for tag in &record.tags {
        let base = base_tag_name(&tag.name).to_ascii_uppercase();
        match base.as_str() {
            "NMREDATA_VERSION" => {
                if let Some(value) = first_value(tag) {
                    record.version = Some(parse_nmredata_version(value)?);
                }
            }
            "NMREDATA_LEVEL" => {
                if let Some(value) = first_value(tag) {
                    record.level = Some(parse_u8_field("NMReDATA level", value)?);
                }
            }
            "NMREDATA_ID" => {
                record.id.extend(parse_key_value_lines(&tag.values));
            }
            "NMREDATA_FORMULA" => record.formula = first_value(tag).map(ToOwned::to_owned),
            "NMREDATA_SMILES" => record.smiles = first_value(tag).map(ToOwned::to_owned),
            "NMREDATA_SOLVENT" => record.solvent = first_value(tag).map(ToOwned::to_owned),
            "NMREDATA_TEMPERATURE" => {
                record.temperature_k = first_value(tag).and_then(parse_first_f64);
            }
            "NMREDATA_ASSIGNMENT" | "NMREDATA_SIGNALS" => {
                let parsed = parse_assignments(&tag.values)?;
                record.assignments.extend(parsed.0);
                record.assignment_equivalences.extend(parsed.1);
            }
            "NMREDATA_J" => {
                let parsed = parse_couplings(&tag.values)?;
                record.couplings.extend(parsed.0);
                record.coupling_equivalences.extend(parsed.1);
            }
            _ if is_nmredata_spectrum_tag(&base) => {
                record.spectra.push(parse_spectrum_tag(tag)?);
            }
            _ => {}
        }
    }
    Ok(())
}

fn parse_assignments(values: &[String]) -> Result<(Vec<NmreDataAssignment>, Vec<String>)> {
    let mut assignments = Vec::new();
    let mut equivalences = Vec::new();

    for line in values {
        if is_equivalence_line(line) {
            equivalences.push(line.clone());
            continue;
        }
        let fields = split_nmredata_fields(line);
        if fields.len() < 2 {
            return Err(RSpinError::Parse {
                format: FORMAT,
                message: format!("assignment requires label and shift: {line}"),
            });
        }
        let shift_ppm = parse_f64_field("assignment shift", &fields[1])?;
        let atom_refs = fields[2..].iter().map(|field| parse_label(field)).collect();
        assignments.push(NmreDataAssignment {
            label: parse_label(&fields[0]),
            shift_ppm,
            atom_refs,
            raw_line: line.clone(),
        });
    }

    Ok((assignments, equivalences))
}

fn parse_couplings(values: &[String]) -> Result<(Vec<NmreDataCoupling>, Vec<String>)> {
    let mut couplings = Vec::new();
    let mut equivalences = Vec::new();

    for line in values {
        if is_equivalence_line(line) {
            equivalences.push(line.clone());
            continue;
        }
        let fields = split_nmredata_fields(line);
        if fields.len() < 3 {
            return Err(RSpinError::Parse {
                format: FORMAT,
                message: format!("coupling requires two labels and a J value: {line}"),
            });
        }
        couplings.push(NmreDataCoupling {
            from_label: parse_label(&fields[0]),
            to_label: parse_label(&fields[1]),
            j_hz: parse_f64_field("scalar coupling", &fields[2])?,
            raw_line: line.clone(),
        });
    }

    Ok((couplings, equivalences))
}

fn parse_spectrum_tag(tag: &NmreDataTag) -> Result<NmreDataSpectrum> {
    let kind = parse_spectrum_kind(tag.base_name());
    let mut attributes = BTreeMap::new();
    let mut signals_1d = Vec::new();
    let mut signals_2d = Vec::new();

    for line in &tag.values {
        let fields = split_nmredata_fields(line);
        if fields.is_empty() {
            continue;
        }
        if let Some((key, value)) = parse_key_value(&fields[0]) {
            append_attribute_values(&mut attributes, key, value, &fields[1..]);
        } else {
            match kind {
                NmreDataSpectrumKind::OneD { .. } => {
                    signals_1d.push(parse_signal_1d(line, &fields)?);
                }
                NmreDataSpectrumKind::TwoD { .. } => {
                    signals_2d.push(parse_signal_2d(line, &fields)?);
                }
                NmreDataSpectrumKind::Other { .. } => {}
            }
        }
    }

    let larmor_mhz = first_attribute_value(&attributes, "Larmor").and_then(parse_first_f64);
    let spectrum_locations = attribute_values(&attributes, "Spectrum_Location");

    Ok(NmreDataSpectrum {
        tag: tag.name.clone(),
        kind,
        attributes,
        larmor_mhz,
        spectrum_locations,
        signals_1d,
        signals_2d,
    })
}

fn parse_spectrum_kind(base_name: &str) -> NmreDataSpectrumKind {
    let Some(descriptor) = base_name.strip_prefix("NMREDATA_") else {
        return NmreDataSpectrumKind::Other {
            descriptor: base_name.to_owned(),
        };
    };
    if let Some(observed) = descriptor.strip_prefix("1D_") {
        return NmreDataSpectrumKind::OneD {
            observed_label: observed.to_owned(),
            observed_nucleus: parse_nucleus(observed),
        };
    }
    if let Some(descriptor) = descriptor.strip_prefix("2D_") {
        let parts: Vec<&str> = descriptor.split('_').collect();
        if parts.len() >= 3 {
            let indirect = parts[0];
            let direct = parts[parts.len() - 1];
            let mixing = parts[1..parts.len() - 1].join("_");
            return NmreDataSpectrumKind::TwoD {
                indirect_label: indirect.to_owned(),
                indirect_nucleus: parse_nucleus(indirect),
                mixing,
                direct_label: direct.to_owned(),
                direct_nucleus: parse_nucleus(direct),
            };
        }
    }

    NmreDataSpectrumKind::Other {
        descriptor: descriptor.to_owned(),
    }
}

fn parse_signal_1d(line: &str, fields: &[String]) -> Result<NmreDataSignal1D> {
    let (from_ppm, to_ppm) = parse_shift_or_range(&fields[0])?;
    let (attributes, items) = parse_attributes(&fields[1..]);
    Ok(NmreDataSignal1D {
        from_ppm,
        to_ppm,
        attributes,
        items,
        raw_line: line.to_owned(),
    })
}

fn parse_signal_2d(line: &str, fields: &[String]) -> Result<NmreDataSignal2D> {
    let (left, right) = split_pair(&fields[0], '/')?;
    let (attributes, items) = parse_attributes(&fields[1..]);
    Ok(NmreDataSignal2D {
        left: parse_label(left),
        right: parse_label(right),
        attributes,
        items,
        raw_line: line.to_owned(),
    })
}

fn parse_attributes(fields: &[String]) -> (BTreeMap<String, Vec<String>>, Vec<String>) {
    let mut attributes: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut items = Vec::new();
    let mut current_key: Option<String> = None;

    for field in fields {
        if let Some((key, value)) = parse_key_value(field) {
            attributes
                .entry(key.to_owned())
                .or_default()
                .push(value.to_owned());
            current_key = Some(key.to_owned());
        } else if let Some(key) = current_key.as_ref() {
            attributes
                .entry(key.clone())
                .or_default()
                .push(parse_label(field));
        } else {
            items.push(parse_label(field));
        }
    }

    (attributes, items)
}

fn append_attribute_values(
    attributes: &mut BTreeMap<String, Vec<String>>,
    key: &str,
    first_value: &str,
    additional_values: &[String],
) {
    let values = attributes.entry(key.to_owned()).or_default();
    values.push(first_value.to_owned());
    values.extend(additional_values.iter().map(|value| parse_label(value)));
}

fn parse_key_value_lines(values: &[String]) -> BTreeMap<String, String> {
    values
        .iter()
        .filter_map(|line| {
            parse_key_value(line).map(|(key, value)| (key.to_owned(), value.to_owned()))
        })
        .collect()
}

fn parse_key_value(input: &str) -> Option<(&str, &str)> {
    let (key, value) = input.split_once('=')?;
    let key = key.trim();
    if key.is_empty() {
        return None;
    }
    Some((key, value.trim()))
}

fn parse_shift_or_range(input: &str) -> Result<(f64, Option<f64>)> {
    let input = input.trim();
    if let Some(separator) = range_separator(input) {
        let from = parse_f64_field("signal range start", &input[..separator])?;
        let to = parse_f64_field("signal range end", &input[separator + 1..])?;
        Ok((from, Some(to)))
    } else {
        Ok((parse_f64_field("signal shift", input)?, None))
    }
}

fn range_separator(input: &str) -> Option<usize> {
    input.char_indices().skip(1).find_map(
        |(index, character)| {
            if character == '-' { Some(index) } else { None }
        },
    )
}

fn parse_u32_component(field: &'static str, value: &str) -> Result<u32> {
    if value.trim().is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(RSpinError::Parse {
            format: FORMAT,
            message: format!("invalid NMReDATA {field} version component: {value}"),
        });
    }
    value.parse::<u32>().map_err(|error| RSpinError::Parse {
        format: FORMAT,
        message: format!("invalid NMReDATA {field} version component: {error}"),
    })
}

fn parse_u8_field(field: &'static str, value: &str) -> Result<u8> {
    value
        .trim()
        .parse::<u8>()
        .map_err(|error| RSpinError::Parse {
            format: FORMAT,
            message: format!("invalid {field}: {error}"),
        })
}

fn parse_f64_field(field: &'static str, value: &str) -> Result<f64> {
    let parsed = value
        .trim()
        .parse::<f64>()
        .map_err(|error| RSpinError::Parse {
            format: FORMAT,
            message: format!("invalid {field}: {error}"),
        })?;
    if !parsed.is_finite() {
        return Err(RSpinError::NonFinite { field });
    }
    Ok(parsed)
}

fn parse_first_f64(value: &str) -> Option<f64> {
    value
        .split_whitespace()
        .next()
        .and_then(|value| value.parse::<f64>().ok())
        .filter(|value| value.is_finite())
}

fn parse_nucleus(value: &str) -> Option<Nucleus> {
    Nucleus::from_str(value).ok()
}

fn first_value(tag: &NmreDataTag) -> Option<&str> {
    tag.values.first().map(String::as_str)
}

fn first_attribute_value<'a>(
    attributes: &'a BTreeMap<String, Vec<String>>,
    key: &str,
) -> Option<&'a str> {
    attributes
        .iter()
        .find(|(candidate, _)| candidate.eq_ignore_ascii_case(key))
        .and_then(|(_, values)| values.first())
        .map(String::as_str)
}

fn attribute_values(attributes: &BTreeMap<String, Vec<String>>, key: &str) -> Vec<String> {
    attributes
        .iter()
        .filter(|(candidate, _)| candidate.eq_ignore_ascii_case(key))
        .flat_map(|(_, values)| values.iter().cloned())
        .collect()
}

fn clean_nmredata_line(line: &str) -> String {
    let without_comment = strip_comment(line);
    without_comment
        .trim()
        .trim_end_matches('\\')
        .trim()
        .to_owned()
}

fn strip_comment(line: &str) -> String {
    let mut output = String::new();
    let mut state = SplitState::default();
    let mut previous = '\0';

    for character in line.chars() {
        state.update_quote_state(previous, character);
        if !state.in_quoted_label && character == ';' {
            break;
        }
        output.push(character);
        previous = character;
    }

    output
}

fn split_nmredata_fields(input: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut state = SplitState::default();
    let mut previous = '\0';

    for character in input.chars() {
        state.update_quote_state(previous, character);
        if !state.in_quoted_label {
            match character {
                '(' => state.parentheses += 1,
                ')' => state.parentheses = state.parentheses.saturating_sub(1),
                ',' if state.parentheses == 0 => {
                    push_field(&mut fields, &mut current);
                    previous = character;
                    continue;
                }
                _ => {}
            }
        }
        current.push(character);
        previous = character;
    }

    push_field(&mut fields, &mut current);
    fields
}

fn split_pair(input: &str, separator: char) -> Result<(&str, &str)> {
    let mut state = SplitState::default();
    let mut previous = '\0';
    for (index, character) in input.char_indices() {
        state.update_quote_state(previous, character);
        if !state.in_quoted_label && state.parentheses == 0 && character == separator {
            let left = input[..index].trim();
            let right = input[index + separator.len_utf8()..].trim();
            if left.is_empty() || right.is_empty() {
                return Err(RSpinError::Parse {
                    format: FORMAT,
                    message: format!("empty 2D correlation label: {input}"),
                });
            }
            return Ok((left, right));
        }
        if !state.in_quoted_label {
            match character {
                '(' => state.parentheses += 1,
                ')' => state.parentheses = state.parentheses.saturating_sub(1),
                _ => {}
            }
        }
        previous = character;
    }
    Err(RSpinError::Parse {
        format: FORMAT,
        message: format!("2D correlation is missing '/': {input}"),
    })
}

fn push_field(fields: &mut Vec<String>, current: &mut String) {
    let field = current.trim();
    if !field.is_empty() {
        fields.push(field.to_owned());
    }
    current.clear();
}

fn parse_label(input: &str) -> String {
    let trimmed = input.trim();
    match trimmed
        .strip_prefix("<\"")
        .and_then(|value| value.strip_suffix("\">"))
    {
        Some(label) => label.to_owned(),
        None => trimmed.to_owned(),
    }
}

fn base_tag_name(name: &str) -> &str {
    match name.split_once('#') {
        Some((base, _)) => base,
        None => name,
    }
}

fn is_nmredata_spectrum_tag(base: &str) -> bool {
    base.starts_with("NMREDATA_1D_")
        || base.starts_with("NMREDATA_2D_")
        || base.starts_with("NMREDATA_19F_")
}

fn is_equivalence_line(line: &str) -> bool {
    line.trim_start()
        .get(..10)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("Equivalent"))
}

#[derive(Default)]
struct SplitState {
    in_quoted_label: bool,
    parentheses: usize,
}

impl SplitState {
    fn update_quote_state(&mut self, previous: char, current: char) {
        if !self.in_quoted_label && previous == '<' && current == '"' {
            self.in_quoted_label = true;
        } else if self.in_quoted_label && previous == '"' && current == '>' {
            self.in_quoted_label = false;
        }
    }
}

#[cfg(test)]
mod tests;
