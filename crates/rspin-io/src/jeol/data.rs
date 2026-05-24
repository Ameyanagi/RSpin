use rspin_core::{RSpinError, Result};

use super::{DATA_TYPE_FLOAT_32, DATA_TYPE_FLOAT_64, binary::parse_error, header::Header};

pub(super) type DataMatrixSections = (Vec<f64>, Option<Vec<f64>>, usize, usize);

pub(super) fn read_data_sections(
    bytes: &[u8],
    header: &Header,
) -> Result<(Vec<f64>, Option<Vec<f64>>)> {
    let point_count = header.point_count()?;
    let section_count = header.data_section_count()?;
    if section_count > 2 {
        return Err(RSpinError::Unsupported {
            feature: "JEOL JDF multidimensional complex layout",
        });
    }

    let element_width = header.element_width();
    let section_len =
        point_count
            .checked_mul(element_width)
            .ok_or_else(|| RSpinError::InvalidSpectrum {
                message: "JEOL JDF section length overflow".to_owned(),
            })?;
    let all_sections_len =
        section_len
            .checked_mul(section_count)
            .ok_or_else(|| RSpinError::InvalidSpectrum {
                message: "JEOL JDF data length overflow".to_owned(),
            })?;
    let data_end = header
        .data_start
        .checked_add(all_sections_len)
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "JEOL JDF data offset overflow".to_owned(),
        })?;
    let data = bytes
        .get(header.data_start..data_end)
        .ok_or_else(|| parse_error("data section is truncated"))?;

    let real = decode_section(&data[..section_len], header, point_count)?;
    let imaginary = if section_count == 2 {
        Some(decode_section(
            &data[section_len..all_sections_len],
            header,
            point_count,
        )?)
    } else {
        None
    };
    Ok((real, imaginary))
}

pub(super) fn read_data_matrix_sections(
    bytes: &[u8],
    header: &Header,
) -> Result<DataMatrixSections> {
    let (x_count, y_count) = header.matrix_shape()?;
    let point_count = x_count
        .checked_mul(y_count)
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "JEOL JDF 2D point count overflow".to_owned(),
        })?;
    let section_count = header.data_section_count()?;
    let element_width = header.element_width();
    let section_len =
        point_count
            .checked_mul(element_width)
            .ok_or_else(|| RSpinError::InvalidSpectrum {
                message: "JEOL JDF 2D section length overflow".to_owned(),
            })?;
    let all_sections_len =
        section_len
            .checked_mul(section_count)
            .ok_or_else(|| RSpinError::InvalidSpectrum {
                message: "JEOL JDF 2D data length overflow".to_owned(),
            })?;
    let data_end = header
        .data_start
        .checked_add(all_sections_len)
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "JEOL JDF 2D data offset overflow".to_owned(),
        })?;
    let data = bytes
        .get(header.data_start..data_end)
        .ok_or_else(|| parse_error("2D data section is truncated"))?;

    let real = decode_section(&data[..section_len], header, point_count)?;
    let imaginary = if section_count >= 2 {
        let second_section_end =
            section_len
                .checked_mul(2)
                .ok_or_else(|| RSpinError::InvalidSpectrum {
                    message: "JEOL JDF 2D companion section overflow".to_owned(),
                })?;
        Some(decode_section(
            &data[section_len..second_section_end],
            header,
            point_count,
        )?)
    } else {
        None
    };
    Ok((real, imaginary, x_count, y_count))
}

fn decode_section(bytes: &[u8], header: &Header, point_count: usize) -> Result<Vec<f64>> {
    let mut values = Vec::with_capacity(point_count);
    for index in 0..point_count {
        let offset = index.checked_mul(header.element_width()).ok_or_else(|| {
            RSpinError::InvalidSpectrum {
                message: "JEOL JDF data offset overflow".to_owned(),
            }
        })?;
        let value = match header.data_type {
            DATA_TYPE_FLOAT_32 => f64::from(header.endian.f32_at(bytes, offset, "data value")?),
            DATA_TYPE_FLOAT_64 => header.endian.f64_at(bytes, offset, "data value")?,
            _ => {
                return Err(RSpinError::Unsupported {
                    feature: "JEOL JDF numeric representation",
                });
            }
        };
        if !value.is_finite() {
            return Err(RSpinError::NonFinite {
                field: "JEOL JDF data",
            });
        }
        values.push(value);
    }
    Ok(values)
}
