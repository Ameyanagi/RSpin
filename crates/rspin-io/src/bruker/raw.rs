//! Bruker raw FID and `ser` import.

mod two_d;

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use rspin_core::{Axis, Metadata, Nucleus, RSpinError, Result, Spectrum1D, Unit};

use crate::SpectrumPathReader;

use super::{
    optional_f64, optional_i32, parse_parameter_file_for_reader, prefixed_parameter_properties,
    read_text, required_usize, text_parameter,
};

pub use two_d::{BrukerSer2D, BrukerSer2DBytes, read_bruker_ser_2d_bytes, read_bruker_ser_2d_dir};

/// Byte-oriented reader for Bruker raw one-dimensional FID data.
///
/// This builder is useful when callers receive uploaded `acqus` and `fid`
/// payloads without a native directory tree.
#[derive(Clone, Copy, Debug)]
pub struct BrukerFid1DBytes<'a> {
    acqus: &'a str,
    fid_bytes: &'a [u8],
}

impl<'a> BrukerFid1DBytes<'a> {
    /// Creates a byte-oriented Bruker raw FID reader.
    #[must_use]
    pub fn new(acqus: &'a str, fid_bytes: &'a [u8]) -> Self {
        Self { acqus, fid_bytes }
    }

    /// Reads the supplied `acqus` text and `fid` bytes.
    ///
    /// # Errors
    ///
    /// Returns an error when `acqus` or `fid` is malformed, real-only,
    /// odd-length, or uses an unsupported binary data type.
    pub fn read(self) -> Result<Spectrum1D> {
        let acqus = parse_parameter_file_for_reader(self.acqus)?;
        let values = decode_raw_i32_values(self.fid_bytes, &acqus)?;
        build_raw_spectrum(&acqus, &values)
    }
}

/// Reader for Bruker raw one-dimensional FID datasets.
///
/// The reader accepts either a dataset directory containing `fid` and `acqus`,
/// or the `fid` file itself. The first implementation supports complex 32-bit
/// integer FID data with Bruker acquisition metadata.
#[derive(Clone, Copy, Debug, Default)]
pub struct BrukerFid1D;

impl BrukerFid1D {
    /// Reads a raw one-dimensional FID from a Bruker dataset path.
    ///
    /// # Errors
    ///
    /// Returns an error when required `fid` or `acqus` files are missing,
    /// malformed, real-only, odd-length, or use an unsupported binary data type.
    pub fn read_dir(self, path: impl AsRef<Path>) -> Result<Spectrum1D> {
        read_bruker_fid_1d_dir(path)
    }
}

impl SpectrumPathReader for BrukerFid1D {
    type Output = Spectrum1D;

    fn read_path(&self, path: &Path) -> Result<Self::Output> {
        read_bruker_fid_1d_dir(path)
    }
}

/// Reads a raw one-dimensional FID from a Bruker dataset path.
///
/// The path may point to the dataset directory or directly to `fid`.
///
/// # Errors
///
/// Returns an error when required files are missing, malformed, real-only,
/// odd-length, or unsupported.
pub fn read_bruker_fid_1d_dir(path: impl AsRef<Path>) -> Result<Spectrum1D> {
    let dataset_dir = locate_dataset_dir(path.as_ref());
    let acqus =
        parse_parameter_file_for_reader(&read_text(&dataset_dir.join("acqus"), "Bruker acqus")?)?;
    let values = read_raw_i32_values(&dataset_dir.join("fid"), &acqus)?;
    build_raw_spectrum(&acqus, &values)
}

/// Reads a raw one-dimensional FID from Bruker `acqus` text and `fid` bytes.
///
/// # Errors
///
/// Returns an error when `acqus` or `fid` is malformed, real-only,
/// odd-length, or uses an unsupported binary data type.
pub fn read_bruker_fid_1d_bytes(acqus: &str, fid_bytes: &[u8]) -> Result<Spectrum1D> {
    BrukerFid1DBytes::new(acqus, fid_bytes).read()
}

fn build_raw_spectrum(acqus: &BTreeMap<String, String>, values: &[f64]) -> Result<Spectrum1D> {
    if !values.len().is_multiple_of(2) {
        return Err(RSpinError::InvalidSpectrum {
            message: "Bruker raw FID must contain interleaved real/imaginary pairs".to_owned(),
        });
    }

    let mut real = Vec::with_capacity(values.len() / 2);
    let mut imaginary = Vec::with_capacity(values.len() / 2);
    for pair in values.chunks_exact(2) {
        real.push(pair[0]);
        imaginary.push(pair[1]);
    }

    let axis = build_raw_axis(real.len(), acqus)?;
    let metadata = build_raw_metadata(acqus)?;
    Spectrum1D::new_complex(axis, real, Some(imaginary), metadata)
}

pub(super) fn locate_dataset_dir(path: &Path) -> PathBuf {
    if path.is_file() {
        path.parent().map_or_else(PathBuf::new, Path::to_path_buf)
    } else {
        path.to_path_buf()
    }
}

fn read_raw_i32_values(path: &Path, acqus: &BTreeMap<String, String>) -> Result<Vec<f64>> {
    let bytes = fs::read(path).map_err(|error| RSpinError::Parse {
        format: "Bruker",
        message: format!("failed to read {}: {error}", path.display()),
    })?;
    decode_raw_i32_values(&bytes, acqus)
}

fn decode_raw_i32_values(bytes: &[u8], acqus: &BTreeMap<String, String>) -> Result<Vec<f64>> {
    ensure_i32_data(acqus)?;

    let raw_count = required_usize(acqus, "TD")?;
    let required_len = raw_count
        .checked_mul(4)
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "Bruker raw FID point count is too large".to_owned(),
        })?;
    if bytes.len() < required_len {
        return Err(RSpinError::Parse {
            format: "Bruker",
            message: format!(
                "raw fid has {} bytes but {required_len} are required",
                bytes.len()
            ),
        });
    }

    let byte_order = byte_order(acqus)?;
    let scale = scale_factor(acqus)?;
    let mut values = Vec::with_capacity(raw_count);
    for chunk in bytes[..required_len].chunks_exact(4) {
        values.push(decode_i32_chunk(chunk, byte_order) * scale);
    }
    Ok(values)
}

pub(super) fn build_raw_axis(point_count: usize, acqus: &BTreeMap<String, String>) -> Result<Axis> {
    match optional_f64(acqus, "SWH")? {
        Some(sweep_hz) if sweep_hz > 0.0 => {
            let end = if point_count <= 1 {
                0.0
            } else {
                let segments =
                    u32::try_from(point_count - 1).map_err(|_| RSpinError::InvalidAxis {
                        message: "Bruker raw FID point count is too large".to_owned(),
                    })?;
                f64::from(segments) / sweep_hz
            };
            Axis::linear("time", Unit::Seconds, 0.0, end, point_count)
        }
        _ => {
            let end = u32::try_from(point_count.saturating_sub(1)).map_or(0.0, f64::from);
            Axis::linear("point", Unit::Points, 0.0, end, point_count)
        }
    }
}

pub(super) fn build_raw_metadata(acqus: &BTreeMap<String, String>) -> Result<Metadata> {
    let nucleus = text_parameter(acqus, "NUC1").and_then(|value| Nucleus::from_str(&value).ok());
    let frequency_mhz = optional_f64(acqus, "SFO1")?;
    let solvent = text_parameter(acqus, "SOLVENT");
    let temperature_k = optional_f64(acqus, "TE")?;
    let origin = text_parameter(acqus, "ORIGIN").or_else(|| text_parameter(acqus, "OWNER"));

    Ok(Metadata {
        name: text_parameter(acqus, "EXP").or_else(|| text_parameter(acqus, "PULPROG")),
        nucleus,
        frequency_mhz,
        solvent,
        temperature_k,
        origin,
        properties: prefixed_parameter_properties("bruker.acqus", acqus),
        ..Metadata::default()
    })
}

pub(super) fn ensure_i32_data(acqus: &BTreeMap<String, String>) -> Result<()> {
    let data_type = optional_i32(acqus, "DTYPA")?;
    if matches!(data_type, Some(value) if value != 0) {
        return Err(RSpinError::Unsupported {
            feature: "Bruker raw non-i32 data",
        });
    }
    Ok(())
}

pub(super) fn scale_factor(acqus: &BTreeMap<String, String>) -> Result<f64> {
    Ok(optional_i32(acqus, "NC")?.map_or(1.0, |value| 2_f64.powi(-value)))
}

pub(super) fn byte_order(acqus: &BTreeMap<String, String>) -> Result<Option<i32>> {
    optional_i32(acqus, "BYTORDA")
}

pub(super) fn decode_i32_chunk(chunk: &[u8], byte_order: Option<i32>) -> f64 {
    let raw = match byte_order {
        Some(1) => i32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]),
        _ => i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]),
    };
    f64::from(raw)
}
