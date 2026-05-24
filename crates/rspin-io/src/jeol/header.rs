use rspin_core::{RSpinError, Result};

use super::{
    AXIS_TYPE_COMPLEX, AXIS_TYPE_REAL_COMPLEX, DATA_FORMAT_ONE_D, DATA_TYPE_FLOAT_32,
    DATA_TYPE_FLOAT_64,
    binary::{BinaryReader, Endian, UnitComponent, parse_error, read_unit_array, usize_from_u32},
};

const MAGIC: &[u8; 8] = b"JEOL.NMR";

#[derive(Clone, Debug)]
pub(super) struct Header {
    pub(super) endian: Endian,
    pub(super) data_type: u8,
    pub(super) axis_types: [u8; 8],
    pub(super) data_units: [UnitComponent; 8],
    pub(super) title: Option<String>,
    pub(super) data_axis_start: [f64; 8],
    pub(super) data_axis_stop: [f64; 8],
    pub(super) param_start: usize,
    pub(super) param_length: usize,
    pub(super) data_start: usize,
    data_dimension_number: u8,
    data_format: u8,
    data_points: [u32; 8],
}

impl Header {
    pub(super) fn parse(bytes: &[u8]) -> Result<Self> {
        let mut reader = BinaryReader::new(bytes, Endian::Big);
        let signature = reader.bytes(8, "file identifier")?;
        if signature != MAGIC.as_slice() {
            return Err(parse_error("file identifier is not JEOL.NMR"));
        }

        let endian = match reader.u8("endianness")? {
            0 => Endian::Big,
            1 => Endian::Little,
            value => return Err(parse_error(format!("unknown endian marker {value}"))),
        };
        let _major_version = reader.u8("major version")?;
        let _minor_version = reader.u16("minor version")?;
        let data_dimension_number = reader.u8("dimension count")?;
        let _dimension_exist = reader.u8("dimension presence")?;
        let data_type_and_format = reader.u8("data type and format")?;
        let data_type = data_type_and_format >> 6;
        let data_format = data_type_and_format & 0b0011_1111;
        let _instrument = reader.u8("instrument")?;
        reader.skip(8, "translate table")?;

        let axis_types = reader.u8_array::<8>("axis types")?;
        let data_units = read_unit_array(&mut reader, "data units")?;
        let title = super::binary::non_empty_string(reader.bytes(124, "title")?);
        reader.skip(4, "axis range table")?;
        let data_points = reader.u32_array::<8>("data point counts")?;
        let _data_offset_start = reader.u32_array::<8>("data offset starts")?;
        let _data_offset_stop = reader.u32_array::<8>("data offset stops")?;
        let data_axis_start = reader.f64_array::<8>("axis starts")?;
        let data_axis_stop = reader.f64_array::<8>("axis stops")?;

        reader.skip(8, "timestamps")?;
        reader.skip(16, "node name")?;
        reader.skip(128, "site")?;
        reader.skip(128, "author")?;
        reader.skip(128, "comment")?;
        reader.skip(8 * 32, "axis titles")?;
        reader.skip(8 * 8, "base frequencies")?;
        reader.skip(8 * 8, "zero points")?;
        reader.skip(8, "axis reversal flags")?;
        reader.skip(4, "annotation flags")?;
        reader.skip(8, "history metadata")?;

        let param_start = usize_from_u32(reader.u32("parameter start")?, "parameter start")?;
        let param_length = usize_from_u32(reader.u32("parameter length")?, "parameter length")?;
        reader.skip(8 * 4, "list starts")?;
        reader.skip(8 * 4, "list lengths")?;
        let data_start = usize_from_u32(reader.u32("data start")?, "data start")?;

        Ok(Self {
            endian,
            data_type,
            axis_types,
            data_units,
            title,
            data_axis_start,
            data_axis_stop,
            param_start,
            param_length,
            data_start,
            data_dimension_number,
            data_format,
            data_points,
        })
    }

    pub(super) fn validate_1d(&self) -> Result<()> {
        if self.data_dimension_number != 1 || self.data_format != DATA_FORMAT_ONE_D {
            return Err(RSpinError::Unsupported {
                feature: "JEOL multidimensional JDF data",
            });
        }
        if !matches!(self.data_type, DATA_TYPE_FLOAT_64 | DATA_TYPE_FLOAT_32) {
            return Err(RSpinError::Unsupported {
                feature: "JEOL JDF numeric representation",
            });
        }
        if self.point_count()? == 0 {
            return Err(RSpinError::InvalidSpectrum {
                message: "JEOL JDF point count must be positive".to_owned(),
            });
        }
        Ok(())
    }

    pub(super) fn point_count(&self) -> Result<usize> {
        usize_from_u32(self.data_points[0], "JEOL JDF point count")
    }

    pub(super) fn data_section_count(&self) -> Result<usize> {
        let mut count = 1usize;
        let mut saw_real_complex = false;
        for axis_type in self.axis_types {
            if axis_type == AXIS_TYPE_REAL_COMPLEX && !saw_real_complex {
                count = count
                    .checked_add(1)
                    .ok_or_else(|| RSpinError::InvalidSpectrum {
                        message: "JEOL JDF section count overflow".to_owned(),
                    })?;
                saw_real_complex = true;
            }
            if axis_type == AXIS_TYPE_COMPLEX {
                count = count
                    .checked_mul(2)
                    .ok_or_else(|| RSpinError::InvalidSpectrum {
                        message: "JEOL JDF section count overflow".to_owned(),
                    })?;
            }
        }
        Ok(count)
    }

    pub(super) fn element_width(&self) -> usize {
        match self.data_type {
            DATA_TYPE_FLOAT_32 => 4,
            _ => 8,
        }
    }
}
