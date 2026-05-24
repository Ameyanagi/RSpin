use rspin_core::{RSpinError, Result, Unit};

#[derive(Clone, Copy, Debug)]
pub(super) enum Endian {
    Big,
    Little,
}

impl Endian {
    pub(super) fn u16_at(self, bytes: &[u8], offset: usize, field: &'static str) -> Result<u16> {
        let bytes = bytes_at(bytes, offset, 2, field)?;
        Ok(match self {
            Self::Big => u16::from_be_bytes([bytes[0], bytes[1]]),
            Self::Little => u16::from_le_bytes([bytes[0], bytes[1]]),
        })
    }

    pub(super) fn u32_at(self, bytes: &[u8], offset: usize, field: &'static str) -> Result<u32> {
        let bytes = bytes_at(bytes, offset, 4, field)?;
        Ok(match self {
            Self::Big => u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            Self::Little => u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        })
    }

    pub(super) fn i32_at(self, bytes: &[u8], offset: usize, field: &'static str) -> Result<i32> {
        let bytes = bytes_at(bytes, offset, 4, field)?;
        Ok(match self {
            Self::Big => i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            Self::Little => i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        })
    }

    pub(super) fn f32_at(self, bytes: &[u8], offset: usize, field: &'static str) -> Result<f32> {
        let bytes = bytes_at(bytes, offset, 4, field)?;
        Ok(match self {
            Self::Big => f32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            Self::Little => f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        })
    }

    pub(super) fn f64_at(self, bytes: &[u8], offset: usize, field: &'static str) -> Result<f64> {
        let bytes = bytes_at(bytes, offset, 8, field)?;
        Ok(match self {
            Self::Big => f64::from_be_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]),
            Self::Little => f64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]),
        })
    }
}

pub(super) struct BinaryReader<'a> {
    bytes: &'a [u8],
    endian: Endian,
    offset: usize,
}

impl<'a> BinaryReader<'a> {
    pub(super) fn new(bytes: &'a [u8], endian: Endian) -> Self {
        Self {
            bytes,
            endian,
            offset: 0,
        }
    }

    pub(super) fn bytes(&mut self, len: usize, field: &'static str) -> Result<&'a [u8]> {
        let bytes = bytes_at(self.bytes, self.offset, len, field)?;
        self.offset = self
            .offset
            .checked_add(len)
            .ok_or_else(|| parse_error("reader offset overflow"))?;
        Ok(bytes)
    }

    pub(super) fn skip(&mut self, len: usize, field: &'static str) -> Result<()> {
        self.bytes(len, field).map(|_| ())
    }

    pub(super) fn u8(&mut self, field: &'static str) -> Result<u8> {
        let bytes = self.bytes(1, field)?;
        Ok(bytes[0])
    }

    pub(super) fn u16(&mut self, field: &'static str) -> Result<u16> {
        let value = self.endian.u16_at(self.bytes, self.offset, field)?;
        self.advance(2)?;
        Ok(value)
    }

    pub(super) fn u32(&mut self, field: &'static str) -> Result<u32> {
        let value = self.endian.u32_at(self.bytes, self.offset, field)?;
        self.advance(4)?;
        Ok(value)
    }

    pub(super) fn f64(&mut self, field: &'static str) -> Result<f64> {
        let value = self.endian.f64_at(self.bytes, self.offset, field)?;
        self.advance(8)?;
        Ok(value)
    }

    pub(super) fn u8_array<const N: usize>(&mut self, field: &'static str) -> Result<[u8; N]> {
        let mut values = [0u8; N];
        for value in &mut values {
            *value = self.u8(field)?;
        }
        Ok(values)
    }

    pub(super) fn u32_array<const N: usize>(&mut self, field: &'static str) -> Result<[u32; N]> {
        let mut values = [0u32; N];
        for value in &mut values {
            *value = self.u32(field)?;
        }
        Ok(values)
    }

    pub(super) fn f64_array<const N: usize>(&mut self, field: &'static str) -> Result<[f64; N]> {
        let mut values = [0.0; N];
        for value in &mut values {
            *value = self.f64(field)?;
        }
        Ok(values)
    }

    fn advance(&mut self, len: usize) -> Result<()> {
        self.offset = self
            .offset
            .checked_add(len)
            .ok_or_else(|| parse_error("reader offset overflow"))?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct UnitComponent {
    pub(super) prefix: i8,
    pub(super) base: u8,
}

impl UnitComponent {
    pub(super) fn prefix_multiplier(self) -> f64 {
        10_f64.powi(-i32::from(self.prefix) * 3)
    }

    pub(super) fn axis_unit(self) -> Unit {
        match self.base {
            13 => Unit::Hertz,
            25 => Unit::Points,
            26 => Unit::Ppm,
            28 => Unit::Seconds,
            _ => Unit::Arbitrary,
        }
    }
}

pub(super) fn read_unit_array(
    reader: &mut BinaryReader<'_>,
    field: &'static str,
) -> Result<[UnitComponent; 8]> {
    let mut units = [UnitComponent::default(); 8];
    for unit in &mut units {
        *unit = read_unit(reader, field)?;
    }
    Ok(units)
}

pub(super) fn read_parameter_units(
    reader: &mut BinaryReader<'_>,
    field: &'static str,
) -> Result<Vec<UnitComponent>> {
    let mut units = Vec::with_capacity(5);
    for _ in 0..5 {
        units.push(read_unit(reader, field)?);
    }
    Ok(units)
}

pub(super) fn non_empty_string(bytes: &[u8]) -> Option<String> {
    let value = bytes
        .iter()
        .copied()
        .filter(|byte| *byte != 0)
        .map(char::from)
        .collect::<String>()
        .trim()
        .to_owned();
    if value.is_empty() { None } else { Some(value) }
}

pub(super) fn non_empty_parameter_name(bytes: &[u8]) -> Option<String> {
    let value = bytes
        .iter()
        .copied()
        .filter(|byte| *byte != b' ' && *byte != 0)
        .map(char::from)
        .collect::<String>();
    if value.is_empty() { None } else { Some(value) }
}

pub(super) fn bytes_at<'a>(
    bytes: &'a [u8],
    offset: usize,
    len: usize,
    field: &'static str,
) -> Result<&'a [u8]> {
    let end = offset
        .checked_add(len)
        .ok_or_else(|| parse_error("byte offset overflow"))?;
    bytes.get(offset..end).ok_or_else(|| {
        parse_error(format!(
            "{field} is truncated at byte {offset}; need {len} bytes"
        ))
    })
}

pub(super) fn usize_from_u32(value: u32, field: &'static str) -> Result<usize> {
    usize::try_from(value).map_err(|_| RSpinError::InvalidSpectrum {
        message: format!("{field} is too large"),
    })
}

pub(super) fn parse_error(message: impl Into<String>) -> RSpinError {
    RSpinError::Parse {
        format: "JEOL",
        message: message.into(),
    }
}

fn read_unit(reader: &mut BinaryReader<'_>, field: &'static str) -> Result<UnitComponent> {
    let prefix_power = reader.u8(field)?;
    let raw_prefix = prefix_power >> 4;
    let prefix = if raw_prefix >= 8 {
        i8::try_from(raw_prefix).map_err(|_| parse_error("unit prefix out of range"))? - 16
    } else {
        i8::try_from(raw_prefix).map_err(|_| parse_error("unit prefix out of range"))?
    };
    let base = reader.u8(field)?;
    Ok(UnitComponent { prefix, base })
}
