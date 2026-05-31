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

/// Edge length of a JEOL Delta 2D data submatrix tile.
///
/// JEOL stores nD data in square submatrix blocks rather than row-major order.
/// The edge is 32 points per dimension (clean-room: confirmed from the
/// documented `jeolconverter` two-dimensional read behavior).
pub(super) const JEOL_2D_SUBMATRIX_EDGE: usize = 32;

pub(super) fn read_data_matrix_sections(
    bytes: &[u8],
    header: &Header,
) -> Result<DataMatrixSections> {
    let planes = read_data_matrix_planes(bytes, header, 2)?;
    let mut iter = planes.planes.into_iter();
    let real = iter
        .next()
        .ok_or_else(|| parse_error("2D data has no real section"))?;
    let imaginary = iter.next();
    Ok((real, imaginary, planes.x_count, planes.y_count))
}

/// All de-tiled section planes of a JEOL 2D dataset, in file order.
pub(super) struct DataMatrixPlanes {
    pub(super) planes: Vec<Vec<f64>>,
    pub(super) x_count: usize,
    pub(super) y_count: usize,
}

/// Reads up to `max_sections` JEOL 2D section planes, de-tiling each submatrix
/// block layout into row-major (`index = y * x_count + x`) order.
pub(super) fn read_data_matrix_planes(
    bytes: &[u8],
    header: &Header,
    max_sections: usize,
) -> Result<DataMatrixPlanes> {
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

    // Real JEOL Delta 2D data is stored in 32-point submatrix tiles, so both
    // dimensions are padded to a multiple of the edge. Sub-tile dimensions only
    // occur in synthetic fixtures, which are laid out row-major; read those
    // linearly.
    let edge = JEOL_2D_SUBMATRIX_EDGE;
    let tiled_layout = x_count.is_multiple_of(edge) && y_count.is_multiple_of(edge);

    let wanted = section_count.min(max_sections);
    let mut planes = Vec::with_capacity(wanted);
    for section in 0..wanted {
        let start = section * section_len;
        let linear = decode_section(&data[start..start + section_len], header, point_count)?;
        planes.push(if tiled_layout {
            detile_submatrix(&linear, x_count, y_count)
        } else {
            linear
        });
    }
    Ok(DataMatrixPlanes {
        planes,
        x_count,
        y_count,
    })
}

/// Reorders one JEOL 2D section from submatrix-tile order into row-major order
/// (`index = y * x_count + x`).
///
/// File order iterates `y_block`, then `x_block`, then the row and column
/// within each [`JEOL_2D_SUBMATRIX_EDGE`]-square tile. Callers guarantee both
/// dimensions are multiples of the edge.
fn detile_submatrix(tiled: &[f64], x_count: usize, y_count: usize) -> Vec<f64> {
    let edge = JEOL_2D_SUBMATRIX_EDGE;
    let x_blocks = x_count / edge;
    let y_blocks = y_count / edge;
    let mut out = vec![0.0_f64; x_count * y_count];
    let mut index = 0usize;
    for y_block in 0..y_blocks {
        for x_block in 0..x_blocks {
            for row in 0..edge {
                let y = y_block * edge + row;
                let base = y * x_count + x_block * edge;
                for column in 0..edge {
                    out[base + column] = tiled[index];
                    index += 1;
                }
            }
        }
    }
    out
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

#[cfg(test)]
mod tests {
    use super::{JEOL_2D_SUBMATRIX_EDGE, detile_submatrix};

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn detiles_submatrix_blocks_into_row_major() {
        // 64 x 32: two x-blocks, one y-block. File order is x-block 0 (32 rows
        // x 32 cols) then x-block 1, each row-major within the tile.
        let edge = JEOL_2D_SUBMATRIX_EDGE;
        let x_count = 2 * edge;
        let y_count = edge;
        let tiled: Vec<f64> = (0..(x_count * y_count)).map(|v| v as f64).collect();

        let out = detile_submatrix(&tiled, x_count, y_count);
        assert_eq!(out.len(), x_count * y_count);

        // Reconstruct the expected mapping: linear index i belongs to
        // x_block = i / (edge*edge), row = (i % (edge*edge)) / edge,
        // col = i % edge, landing at out[row][x_block*edge + col].
        for (i, value) in tiled.iter().enumerate() {
            let x_block = i / (edge * edge);
            let within = i % (edge * edge);
            let row = within / edge;
            let col = within % edge;
            let x = x_block * edge + col;
            assert!((out[row * x_count + x] - *value).abs() < f64::EPSILON);
        }

        // A corner spot-check: the first value of x-block 1 (index edge*edge)
        // must land at row 0, column `edge`.
        assert!((out[edge] - (edge * edge) as f64).abs() < f64::EPSILON);
    }
}
