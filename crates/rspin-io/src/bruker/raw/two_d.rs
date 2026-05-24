//! Bruker raw two-dimensional `ser` import.

use std::{collections::BTreeMap, fs, path::Path};

use rspin_core::{Axis, RSpinError, Result, Spectrum2D, Unit};

use crate::bruker::{optional_i32, parse_parameter_file, read_text, required_usize};

use super::{
    build_raw_axis, build_raw_metadata, byte_order, decode_i32_chunk, ensure_i32_data,
    locate_dataset_dir, scale_factor,
};

const BRUKER_SER_ROW_ALIGNMENT_WORDS: usize = 256;

/// Reader for Bruker raw two-dimensional `ser` datasets.
///
/// The returned `Spectrum2D` is a raw trace matrix: each indirect row stores
/// direct-dimension real values in `z` and direct-dimension imaginary values in
/// `imaginary`. Hypercomplex indirect reconstruction is left to processing.
#[derive(Clone, Copy, Debug, Default)]
pub struct BrukerSer2D;

impl BrukerSer2D {
    /// Reads a raw two-dimensional Bruker `ser` dataset.
    ///
    /// # Errors
    ///
    /// Returns an error when `ser`, `acqus`, or `acqu2s` are missing or when
    /// the binary layout is malformed or unsupported.
    pub fn read_dir(self, path: impl AsRef<Path>) -> Result<Spectrum2D> {
        read_bruker_ser_2d_dir(path)
    }
}

/// Reads a raw two-dimensional Bruker `ser` dataset.
///
/// The path may point to the dataset directory or directly to `ser`.
///
/// # Errors
///
/// Returns an error when the dataset is missing, malformed, or unsupported.
pub fn read_bruker_ser_2d_dir(path: impl AsRef<Path>) -> Result<Spectrum2D> {
    let dataset_dir = locate_dataset_dir(path.as_ref());
    let direct_parameters =
        parse_parameter_file(&read_text(&dataset_dir.join("acqus"), "Bruker acqus")?);
    let indirect_parameters =
        parse_parameter_file(&read_text(&dataset_dir.join("acqu2s"), "Bruker acqu2s")?);
    let (z, imaginary, x_count, y_count) =
        read_ser_values(&dataset_dir.join("ser"), &direct_parameters)?;
    validate_indirect_count(y_count, &indirect_parameters)?;

    let x = build_raw_axis(x_count, &direct_parameters)?;
    let y = build_indirect_axis(y_count, &indirect_parameters)?;
    let metadata = build_raw_metadata(&direct_parameters)?;
    Spectrum2D::new_complex(x, y, z, Some(imaginary), metadata)
}

fn read_ser_values(
    path: &Path,
    acqus: &BTreeMap<String, String>,
) -> Result<(Vec<f64>, Vec<f64>, usize, usize)> {
    ensure_i32_data(acqus)?;
    let direct_words = required_usize(acqus, "TD")?;
    if direct_words == 0 || direct_words % 2 != 0 {
        return Err(RSpinError::InvalidSpectrum {
            message: "Bruker raw 2D direct TD must be a positive even value".to_owned(),
        });
    }
    let row_words = padded_row_words(direct_words)?;
    let row_bytes = row_words
        .checked_mul(4)
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "Bruker raw 2D row length is too large".to_owned(),
        })?;
    let bytes = fs::read(path).map_err(|error| RSpinError::Parse {
        format: "Bruker",
        message: format!("failed to read {}: {error}", path.display()),
    })?;
    if bytes.len() % row_bytes != 0 {
        return Err(RSpinError::Parse {
            format: "Bruker",
            message: format!(
                "raw ser has {} bytes but row length is {row_bytes} bytes",
                bytes.len()
            ),
        });
    }

    let y_count = bytes.len() / row_bytes;
    if y_count == 0 {
        return Err(RSpinError::InvalidSpectrum {
            message: "Bruker raw 2D ser must contain at least one row".to_owned(),
        });
    }

    let x_count = direct_words / 2;
    let expected = x_count
        .checked_mul(y_count)
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "Bruker raw 2D matrix size is too large".to_owned(),
        })?;
    let mut z = Vec::with_capacity(expected);
    let mut imaginary = Vec::with_capacity(expected);
    let byte_order = byte_order(acqus)?;
    let scale = scale_factor(acqus)?;

    for row in bytes.chunks_exact(row_bytes) {
        let acquired = &row[..direct_words * 4];
        for pair in acquired.chunks_exact(8) {
            z.push(decode_i32_chunk(&pair[..4], byte_order) * scale);
            imaginary.push(decode_i32_chunk(&pair[4..8], byte_order) * scale);
        }
    }
    Ok((z, imaginary, x_count, y_count))
}

fn build_indirect_axis(point_count: usize, acqu2s: &BTreeMap<String, String>) -> Result<Axis> {
    match optional_i32(acqu2s, "FNMODE")? {
        Some(0) | None => build_raw_axis(point_count, acqu2s),
        Some(_) => {
            let end = u32::try_from(point_count.saturating_sub(1)).map_or(0.0, f64::from);
            Axis::linear("indirect trace", Unit::Points, 0.0, end, point_count)
        }
    }
}

fn validate_indirect_count(y_count: usize, acqu2s: &BTreeMap<String, String>) -> Result<()> {
    let expected = required_usize(acqu2s, "TD")?;
    if expected != y_count {
        return Err(RSpinError::InvalidSpectrum {
            message: format!("Bruker raw 2D has {y_count} rows but acqu2s TD is {expected}"),
        });
    }
    Ok(())
}

fn padded_row_words(direct_words: usize) -> Result<usize> {
    let adjusted = direct_words
        .checked_add(BRUKER_SER_ROW_ALIGNMENT_WORDS - 1)
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "Bruker raw 2D row length is too large".to_owned(),
        })?;
    Ok((adjusted / BRUKER_SER_ROW_ALIGNMENT_WORDS) * BRUKER_SER_ROW_ALIGNMENT_WORDS)
}
