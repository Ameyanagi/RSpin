//! Bruker raw two-dimensional `ser` import.

use std::{collections::BTreeMap, fs, path::Path};

use rspin_core::{Axis, RSpinError, Result, Spectrum2D};

use crate::SpectrumPathReader;
use crate::bruker::{optional_i32, parse_parameter_file_for_reader, read_text, required_usize};

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

impl SpectrumPathReader for BrukerSer2D {
    type Output = Spectrum2D;

    fn read_path(&self, path: &Path) -> Result<Self::Output> {
        read_bruker_ser_2d_dir(path)
    }
}

/// Byte-oriented reader for Bruker raw two-dimensional `ser` data.
///
/// This builder is useful when callers receive uploaded `acqus`, `acqu2s`,
/// and `ser` payloads without a native directory tree.
#[derive(Clone, Copy, Debug)]
pub struct BrukerSer2DBytes<'a> {
    direct_parameters: &'a str,
    indirect_parameters: &'a str,
    ser_bytes: &'a [u8],
}

impl<'a> BrukerSer2DBytes<'a> {
    /// Creates a byte-oriented Bruker raw 2D reader.
    #[must_use]
    pub fn new(
        direct_parameters: &'a str,
        indirect_parameters: &'a str,
        ser_bytes: &'a [u8],
    ) -> Self {
        Self {
            direct_parameters,
            indirect_parameters,
            ser_bytes,
        }
    }

    /// Reads the supplied `acqus`, `acqu2s`, and `ser` bytes.
    ///
    /// # Errors
    ///
    /// Returns an error when metadata or binary data are missing, malformed, or
    /// unsupported.
    pub fn read(self) -> Result<Spectrum2D> {
        let direct_parameters = parse_parameter_file_for_reader(self.direct_parameters)?;
        let indirect_parameters = parse_parameter_file_for_reader(self.indirect_parameters)?;
        build_ser_spectrum(&direct_parameters, &indirect_parameters, self.ser_bytes)
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
        parse_parameter_file_for_reader(&read_text(&dataset_dir.join("acqus"), "Bruker acqus")?)?;
    let indirect_parameters =
        parse_parameter_file_for_reader(&read_text(&dataset_dir.join("acqu2s"), "Bruker acqu2s")?)?;
    let (z, imaginary, x_count, y_count) =
        read_ser_values(&dataset_dir.join("ser"), &direct_parameters)?;
    validate_indirect_count(y_count, &indirect_parameters)?;

    let x = build_raw_axis(x_count, &direct_parameters)?;
    let y = build_indirect_axis(y_count, &indirect_parameters)?;
    let metadata = build_raw_metadata(&direct_parameters)?;
    Spectrum2D::new_complex(x, y, z, Some(imaginary), metadata)
}

/// Reads a raw two-dimensional Bruker `ser` payload from parameter text and
/// bytes.
///
/// # Errors
///
/// Returns an error when metadata or binary data are missing, malformed, or
/// unsupported.
pub fn read_bruker_ser_2d_bytes(
    direct_parameters: &str,
    indirect_parameters: &str,
    ser_bytes: &[u8],
) -> Result<Spectrum2D> {
    BrukerSer2DBytes::new(direct_parameters, indirect_parameters, ser_bytes).read()
}

fn build_ser_spectrum(
    direct_parameters: &BTreeMap<String, String>,
    indirect_parameters: &BTreeMap<String, String>,
    ser_bytes: &[u8],
) -> Result<Spectrum2D> {
    let (z, imaginary, x_count, y_count) = decode_ser_values(ser_bytes, direct_parameters)?;
    validate_indirect_count(y_count, indirect_parameters)?;

    let x = build_raw_axis(x_count, direct_parameters)?;
    let y = build_indirect_axis(y_count, indirect_parameters)?;
    let metadata = build_raw_metadata(direct_parameters)?;
    Spectrum2D::new_complex(x, y, z, Some(imaginary), metadata)
}

fn read_ser_values(
    path: &Path,
    acqus: &BTreeMap<String, String>,
) -> Result<(Vec<f64>, Vec<f64>, usize, usize)> {
    let bytes = fs::read(path).map_err(|error| RSpinError::Parse {
        format: "Bruker",
        message: format!("failed to read {}: {error}", path.display()),
    })?;
    decode_ser_values(&bytes, acqus)
}

fn decode_ser_values(
    bytes: &[u8],
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
    let _ = optional_i32(acqu2s, "FNMODE")?;
    build_raw_axis(point_count, acqu2s)
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
