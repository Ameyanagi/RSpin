use std::collections::BTreeMap;

use rspin_core::{RSpinError, Result};

use super::{
    PARAMETER_HEADER_LEN, PARAMETER_RECORD_LEN, VALUE_TYPE_FLOAT, VALUE_TYPE_INTEGER,
    VALUE_TYPE_STRING,
    binary::{
        BinaryReader, UnitComponent, non_empty_parameter_name, non_empty_string, parse_error,
        read_parameter_units, usize_from_u32,
    },
    header::Header,
};

#[derive(Clone, Debug, Default)]
pub(super) struct Parameters {
    entries: Vec<Parameter>,
}

impl Parameters {
    pub(super) fn parse(bytes: &[u8], header: &Header) -> Result<Self> {
        if header.param_length == 0 {
            return Ok(Self::default());
        }

        let section_end = header
            .param_start
            .checked_add(header.param_length)
            .ok_or_else(|| RSpinError::InvalidSpectrum {
                message: "JEOL JDF parameter section overflow".to_owned(),
            })?;
        let section = bytes
            .get(header.param_start..section_end)
            .ok_or_else(|| parse_error("parameter section is outside the file"))?;
        if section.len() < PARAMETER_HEADER_LEN {
            return Err(parse_error("parameter section is shorter than its header"));
        }

        let mut reader = BinaryReader::new(section, header.endian);
        let _parameter_size = reader.u32("parameter record size")?;
        let _low_index = reader.u32("parameter low index")?;
        let high_index = usize_from_u32(reader.u32("parameter high index")?, "parameter count")?;
        let _total_size = reader.u32("parameter total size")?;

        let record_count =
            high_index
                .checked_add(1)
                .ok_or_else(|| RSpinError::InvalidSpectrum {
                    message: "JEOL JDF parameter count overflow".to_owned(),
                })?;
        let available = section.len().saturating_sub(PARAMETER_HEADER_LEN) / PARAMETER_RECORD_LEN;
        if record_count > available {
            return Err(parse_error("parameter records are truncated"));
        }

        let mut entries = Vec::with_capacity(record_count);
        for index in 0..record_count {
            let offset = PARAMETER_HEADER_LEN
                .checked_add(index.checked_mul(PARAMETER_RECORD_LEN).ok_or_else(|| {
                    RSpinError::InvalidSpectrum {
                        message: "JEOL JDF parameter offset overflow".to_owned(),
                    }
                })?)
                .ok_or_else(|| RSpinError::InvalidSpectrum {
                    message: "JEOL JDF parameter offset overflow".to_owned(),
                })?;
            let record = section
                .get(offset..offset + PARAMETER_RECORD_LEN)
                .ok_or_else(|| parse_error("parameter record is truncated"))?;
            if let Some(parameter) = Parameter::parse(record, header.endian)? {
                entries.push(parameter);
            }
        }
        Ok(Self { entries })
    }

    pub(super) fn string(&self, name: &str) -> Option<&str> {
        match self.get(name).map(|entry| &entry.value) {
            Some(ParameterValue::String(value)) => Some(value.as_str()),
            _ => None,
        }
    }

    pub(super) fn magnitude(&self, name: &str) -> Option<(f64, UnitComponent)> {
        self.get(name).and_then(Parameter::magnitude)
    }

    pub(super) fn properties(&self) -> BTreeMap<String, String> {
        self.entries
            .iter()
            .map(|entry| {
                (
                    format!("jeol.parameter.{}", entry.name),
                    entry.value.as_string(),
                )
            })
            .collect()
    }

    fn get(&self, name: &str) -> Option<&Parameter> {
        self.entries.iter().find(|entry| entry.name == name)
    }
}

#[derive(Clone, Debug)]
struct Parameter {
    name: String,
    units: Vec<UnitComponent>,
    value: ParameterValue,
}

impl Parameter {
    fn parse(record: &[u8], endian: super::binary::Endian) -> Result<Option<Self>> {
        let mut reader = BinaryReader::new(record, endian);
        reader.skip(6, "parameter flags")?;
        let units = read_parameter_units(&mut reader, "parameter units")?;
        let value_type = endian.i32_at(record, 32, "parameter value type")?;
        let value = match value_type {
            VALUE_TYPE_STRING => {
                let mut value = String::new();
                if let Some(parsed) = non_empty_string(
                    record
                        .get(16..32)
                        .ok_or_else(|| parse_error("parameter string value is truncated"))?,
                ) {
                    value = parsed;
                }
                ParameterValue::String(value)
            }
            VALUE_TYPE_INTEGER => ParameterValue::Integer(endian.i32_at(record, 16, "integer")?),
            VALUE_TYPE_FLOAT => ParameterValue::Float(endian.f64_at(record, 16, "float")?),
            _ => return Ok(None),
        };
        let Some(name) = non_empty_parameter_name(
            record
                .get(36..64)
                .ok_or_else(|| parse_error("parameter name is truncated"))?,
        ) else {
            return Ok(None);
        };
        Ok(Some(Self {
            name: name.to_ascii_lowercase(),
            units,
            value,
        }))
    }

    fn magnitude(&self) -> Option<(f64, UnitComponent)> {
        let unit = self.units.first().copied()?;
        match self.value {
            ParameterValue::Integer(value) => {
                Some((f64::from(value) * unit.prefix_multiplier(), unit))
            }
            ParameterValue::Float(value) => Some((value * unit.prefix_multiplier(), unit)),
            ParameterValue::String(_) => None,
        }
    }
}

#[derive(Clone, Debug)]
enum ParameterValue {
    String(String),
    Integer(i32),
    Float(f64),
}

impl ParameterValue {
    fn as_string(&self) -> String {
        match self {
            Self::String(value) => value.clone(),
            Self::Integer(value) => value.to_string(),
            Self::Float(value) => value.to_string(),
        }
    }
}
