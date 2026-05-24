//! Agilent/Varian raw one-dimensional FID import.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use rspin_core::{Axis, Metadata, Nucleus, RSpinError, Result, Spectrum1D, Unit};

mod procpar;

use procpar::{first_f64, first_text, parse_procpar};

const FILE_HEADER_LEN: usize = 32;
const BLOCK_HEADER_LEN: usize = 28;
const STATUS_FLOAT: i16 = 0x0008;
const STATUS_COMPLEX: i16 = 0x0010;
const STATUS_COMPLEX_ALT: i16 = 0x0040;

/// Reader for Agilent/Varian raw one-dimensional FID directories.
#[derive(Clone, Copy, Debug, Default)]
pub struct AgilentFid1D;

impl AgilentFid1D {
    /// Reads a raw one-dimensional FID from an Agilent/Varian dataset directory.
    ///
    /// # Errors
    ///
    /// Returns an error when `fid` or `procpar` is missing, malformed, arrayed,
    /// or stored in an unsupported numeric representation.
    pub fn read_dir(self, path: impl AsRef<Path>) -> Result<Spectrum1D> {
        read_agilent_fid_1d_dir(path)
    }
}

/// Reads a raw one-dimensional FID from an Agilent/Varian dataset directory.
///
/// The returned spectrum uses a seconds axis when `sw` is available and stores
/// the FID real and imaginary channels separately.
///
/// # Errors
///
/// Returns an error when the dataset is missing, malformed, arrayed, or uses an
/// unsupported data representation.
pub fn read_agilent_fid_1d_dir(path: impl AsRef<Path>) -> Result<Spectrum1D> {
    let dataset_dir = locate_dataset_dir(path.as_ref());
    let procpar = parse_procpar(&read_text(&dataset_dir.join("procpar"), "Agilent procpar")?);
    let fid_bytes = fs::read(dataset_dir.join("fid")).map_err(|error| RSpinError::Parse {
        format: "Agilent",
        message: format!("failed to read fid: {error}"),
    })?;

    let (real, imaginary) = read_fid_values(&fid_bytes)?;
    let axis = build_axis(real.len(), &procpar)?;
    let metadata = build_metadata(&procpar);
    Spectrum1D::new_complex(axis, real, Some(imaginary), metadata)
}

fn locate_dataset_dir(path: &Path) -> PathBuf {
    if path.is_file() {
        path.parent().map_or_else(PathBuf::new, Path::to_path_buf)
    } else {
        path.to_path_buf()
    }
}

fn read_fid_values(bytes: &[u8]) -> Result<(Vec<f64>, Vec<f64>)> {
    let header = FileHeader::parse(bytes)?;
    header.validate_1d()?;

    let block_header_len = header
        .nbheaders
        .checked_mul(BLOCK_HEADER_LEN)
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "Agilent block header size overflow".to_owned(),
        })?;
    let data_start = FILE_HEADER_LEN
        .checked_add(block_header_len)
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "Agilent data offset overflow".to_owned(),
        })?;
    let data_len =
        header
            .np_values
            .checked_mul(header.ebytes)
            .ok_or_else(|| RSpinError::InvalidSpectrum {
                message: "Agilent trace size overflow".to_owned(),
            })?;
    let data_end = data_start
        .checked_add(data_len)
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "Agilent data end overflow".to_owned(),
        })?;
    if bytes.len() < data_end {
        return Err(RSpinError::Parse {
            format: "Agilent",
            message: format!("fid has {} bytes but {data_end} are required", bytes.len()),
        });
    }

    let values = decode_values(&bytes[data_start..data_end], &header)?;
    let mut real = Vec::with_capacity(values.len() / 2);
    let mut imaginary = Vec::with_capacity(values.len() / 2);
    for pair in values.chunks_exact(2) {
        real.push(pair[0]);
        imaginary.push(pair[1]);
    }
    Ok((real, imaginary))
}

#[derive(Clone, Copy)]
struct FileHeader {
    endian: Endian,
    nblocks: usize,
    ntraces: usize,
    np_values: usize,
    ebytes: usize,
    tbytes: usize,
    bbytes: usize,
    status: i16,
    nbheaders: usize,
}

impl FileHeader {
    fn parse(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < FILE_HEADER_LEN {
            return Err(RSpinError::Parse {
                format: "Agilent",
                message: "fid is shorter than the file header".to_owned(),
            });
        }
        [Endian::Big, Endian::Little]
            .into_iter()
            .find_map(|endian| {
                Self::parse_with_endian(bytes, endian).filter(|header| header.is_plausible())
            })
            .ok_or_else(|| RSpinError::Parse {
                format: "Agilent",
                message: "fid file header is not recognized".to_owned(),
            })
    }

    fn parse_with_endian(bytes: &[u8], endian: Endian) -> Option<Self> {
        Some(Self {
            endian,
            nblocks: usize::try_from(endian.i32_at(bytes, 0)?).ok()?,
            ntraces: usize::try_from(endian.i32_at(bytes, 4)?).ok()?,
            np_values: usize::try_from(endian.i32_at(bytes, 8)?).ok()?,
            ebytes: usize::try_from(endian.i32_at(bytes, 12)?).ok()?,
            tbytes: usize::try_from(endian.i32_at(bytes, 16)?).ok()?,
            bbytes: usize::try_from(endian.i32_at(bytes, 20)?).ok()?,
            status: endian.i16_at(bytes, 26)?,
            nbheaders: usize::try_from(endian.i32_at(bytes, 28)?).ok()?,
        })
    }

    fn is_plausible(self) -> bool {
        self.nblocks > 0
            && self.ntraces > 0
            && self.np_values > 0
            && self.np_values % 2 == 0
            && matches!(self.ebytes, 2 | 4 | 8)
            && self.tbytes == self.np_values.saturating_mul(self.ebytes)
            && self.nbheaders > 0
            && self.bbytes >= self.tbytes
    }

    fn validate_1d(self) -> Result<()> {
        if self.nblocks != 1 || self.ntraces != 1 {
            return Err(RSpinError::Unsupported {
                feature: "Agilent arrayed or multidimensional FID",
            });
        }
        if self.status & (STATUS_COMPLEX | STATUS_COMPLEX_ALT) == 0 {
            return Err(RSpinError::Unsupported {
                feature: "Agilent real-only FID",
            });
        }
        Ok(())
    }

    fn is_float(self) -> bool {
        self.status & STATUS_FLOAT != 0
    }
}

#[derive(Clone, Copy)]
enum Endian {
    Big,
    Little,
}

impl Endian {
    fn i16_at(self, bytes: &[u8], offset: usize) -> Option<i16> {
        let bytes = bytes.get(offset..offset.checked_add(2)?)?;
        Some(match self {
            Self::Big => i16::from_be_bytes([bytes[0], bytes[1]]),
            Self::Little => i16::from_le_bytes([bytes[0], bytes[1]]),
        })
    }

    fn i32_at(self, bytes: &[u8], offset: usize) -> Option<i32> {
        let bytes = bytes.get(offset..offset.checked_add(4)?)?;
        Some(match self {
            Self::Big => i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            Self::Little => i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        })
    }

    fn f32_at(self, bytes: &[u8], offset: usize) -> Option<f32> {
        let bytes = bytes.get(offset..offset.checked_add(4)?)?;
        Some(match self {
            Self::Big => f32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            Self::Little => f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        })
    }

    fn f64_at(self, bytes: &[u8], offset: usize) -> Option<f64> {
        let bytes = bytes.get(offset..offset.checked_add(8)?)?;
        Some(match self {
            Self::Big => f64::from_be_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]),
            Self::Little => f64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]),
        })
    }
}

fn decode_values(bytes: &[u8], header: &FileHeader) -> Result<Vec<f64>> {
    let mut values = Vec::with_capacity(header.np_values);
    for index in 0..header.np_values {
        let offset =
            index
                .checked_mul(header.ebytes)
                .ok_or_else(|| RSpinError::InvalidSpectrum {
                    message: "Agilent data offset overflow".to_owned(),
                })?;
        let value = match (header.is_float(), header.ebytes) {
            (false, 2) => f64::from(
                header
                    .endian
                    .i16_at(bytes, offset)
                    .ok_or_else(|| parse_error("truncated 16-bit integer data"))?,
            ),
            (false, 4) => f64::from(
                header
                    .endian
                    .i32_at(bytes, offset)
                    .ok_or_else(|| parse_error("truncated 32-bit integer data"))?,
            ),
            (true, 4) => f64::from(
                header
                    .endian
                    .f32_at(bytes, offset)
                    .ok_or_else(|| parse_error("truncated 32-bit float data"))?,
            ),
            (true, 8) => header
                .endian
                .f64_at(bytes, offset)
                .ok_or_else(|| parse_error("truncated 64-bit float data"))?,
            _ => {
                return Err(RSpinError::Unsupported {
                    feature: "Agilent FID numeric representation",
                });
            }
        };
        if !value.is_finite() {
            return Err(RSpinError::NonFinite {
                field: "Agilent FID data",
            });
        }
        values.push(value);
    }
    Ok(values)
}

fn build_axis(point_count: usize, parameters: &BTreeMap<String, Vec<String>>) -> Result<Axis> {
    match first_f64(parameters, "sw")? {
        Some(sw) if sw > 0.0 => {
            let end = if point_count <= 1 {
                0.0
            } else {
                let segments =
                    u32::try_from(point_count - 1).map_err(|_| RSpinError::InvalidAxis {
                        message: "Agilent FID point count is too large".to_owned(),
                    })?;
                f64::from(segments) / sw
            };
            Axis::linear("time", Unit::Seconds, 0.0, end, point_count)
        }
        _ => {
            let end = u32::try_from(point_count.saturating_sub(1)).map_or(0.0, f64::from);
            Axis::linear("point", Unit::Points, 0.0, end, point_count)
        }
    }
}

fn build_metadata(parameters: &BTreeMap<String, Vec<String>>) -> Metadata {
    let nucleus = first_text(parameters, "tn").and_then(|value| Nucleus::from_str(&value).ok());
    let frequency_mhz = first_f64(parameters, "sfrq").ok().flatten();
    let solvent = first_text(parameters, "solvent");
    let temperature_k = first_f64(parameters, "temp")
        .ok()
        .flatten()
        .map(varian_temperature_to_kelvin);

    Metadata {
        name: first_text(parameters, "comment").or_else(|| first_text(parameters, "seqfil")),
        nucleus,
        frequency_mhz,
        solvent,
        temperature_k,
        origin: first_text(parameters, "operator").or_else(|| first_text(parameters, "username")),
        molecules: Vec::new(),
    }
}

fn varian_temperature_to_kelvin(value: f64) -> f64 {
    if value > 150.0 { value } else { value + 273.15 }
}

fn read_text(path: &Path, description: &'static str) -> Result<String> {
    fs::read_to_string(path).map_err(|error| RSpinError::Parse {
        format: "Agilent",
        message: format!(
            "failed to read {description} at {}: {error}",
            path.display()
        ),
    })
}

fn parse_error(message: impl Into<String>) -> RSpinError {
    RSpinError::Parse {
        format: "Agilent",
        message: message.into(),
    }
}

#[cfg(test)]
mod tests;
