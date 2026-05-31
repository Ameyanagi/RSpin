//! Agilent/Varian raw and processed spectrum import.

use std::{
    collections::BTreeMap,
    fs,
    io::Read,
    path::{Path, PathBuf},
    str::FromStr,
};

use rspin_core::{Axis, Metadata, Nucleus, RSpinError, Result, Spectrum1D, Spectrum2D, Unit};
use serde::{Deserialize, Serialize};

use crate::SpectrumPathReader;

mod procpar;

use procpar::{first_f64, first_text, first_usize, parse_procpar};

const FILE_HEADER_LEN: usize = 32;
const BLOCK_HEADER_LEN: usize = 28;
const STATUS_FLOAT: i16 = 0x0008;
const STATUS_COMPLEX: i16 = 0x0010;
const STATUS_COMPLEX_ALT: i16 = 0x0040;

/// Routing metadata from an Agilent/Varian `procpar` file.
///
/// `procpar` does not use one fixed schema version label across all variants,
/// so `RSpin` preserves common software/provenance fields and exposes acquisition
/// dimensionality for reader routing.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AgilentProcparInfo {
    /// Raw software/revision label, commonly `vnmrrev`, when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub software_revision: Option<String>,
    /// Pulse sequence file, commonly `seqfil`, when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sequence: Option<String>,
    /// Declared acquisition dimension from `acqdim`, when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acquisition_dimension: Option<usize>,
    /// Arrayed parameter name from `array`, when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub array_parameter: Option<String>,
    /// Number of array elements from `arrayelemts`, when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub array_elements: Option<usize>,
    /// Primary observed nucleus from `tn`, when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nucleus: Option<String>,
    /// Direct-dimension spectrometer frequency from `sfrq`, when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frequency_mhz: Option<f64>,
    /// Direct-dimension spectral width from `sw`, when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spectral_width_hz: Option<f64>,
    /// Operator or username provenance, when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operator: Option<String>,
}

impl AgilentProcparInfo {
    /// Returns true when the acquisition dimension is supported by current readers.
    #[must_use]
    pub fn is_supported_by_current_readers(&self) -> bool {
        matches!(self.acquisition_dimension, None | Some(0..=2))
    }

    /// Validates that the acquisition dimension is supported by current readers.
    ///
    /// # Errors
    ///
    /// Returns an unsupported-feature error for three-or-higher-dimensional
    /// acquisitions.
    pub fn validate_supported_by_current_readers(&self) -> Result<()> {
        if self.is_supported_by_current_readers() {
            Ok(())
        } else {
            Err(RSpinError::Unsupported {
                feature: "Agilent three-or-higher-dimensional acquisition",
            })
        }
    }
}

/// Routing metadata from an Agilent/Varian binary `fid` or `phasefile` header.
///
/// The header does not distinguish raw and processed payloads by itself, but it
/// does expose byte order, numeric representation, trace layout, and whether the
/// payload should route to one- or two-dimensional readers.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgilentBinaryFileInfo {
    /// Header endianness label, `big` or `little`.
    pub endian: String,
    /// Number of binary blocks.
    pub blocks: usize,
    /// Number of traces stored in each block.
    pub traces_per_block: usize,
    /// Number of numeric values in each trace.
    pub values_per_trace: usize,
    /// Number of bytes per numeric value.
    pub element_bytes: usize,
    /// Number of bytes in each trace.
    pub trace_bytes: usize,
    /// Number of bytes in each block.
    pub block_bytes: usize,
    /// Number of block headers per block.
    pub block_header_count: usize,
    /// True when the status word marks values as floating point.
    pub is_float: bool,
    /// True when the status word marks traces as complex/interleaved.
    pub is_complex: bool,
    /// Total trace count across all blocks.
    pub trace_count: usize,
    /// Reader dimensionality inferred from the trace layout.
    pub dimension_count: usize,
}

impl AgilentBinaryFileInfo {
    /// Returns true when the binary trace layout routes to a 1D reader.
    #[must_use]
    pub fn is_one_dimensional(&self) -> bool {
        self.dimension_count == 1
    }

    /// Returns true when the binary trace layout routes to a 2D reader.
    #[must_use]
    pub fn is_two_dimensional(&self) -> bool {
        self.dimension_count == 2
    }
}

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

impl SpectrumPathReader for AgilentFid1D {
    type Output = Spectrum1D;

    fn read_path(&self, path: &Path) -> Result<Self::Output> {
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
    let procpar_text = read_text(&dataset_dir.join("procpar"), "Agilent procpar")?;
    let fid_bytes = fs::read(dataset_dir.join("fid")).map_err(|error| RSpinError::Parse {
        format: "Agilent",
        message: format!("failed to read fid: {error}"),
    })?;

    read_agilent_fid_1d_bytes(&procpar_text, &fid_bytes)
}

/// Reads a raw one-dimensional FID from Agilent/Varian `procpar` text and FID bytes.
///
/// This byte-oriented API is useful for callers such as WASM front ends that
/// receive uploaded files without a native directory tree.
///
/// # Errors
///
/// Returns an error when the `procpar` text is malformed, declares an
/// unsupported acquisition, or the FID bytes are malformed or unsupported.
pub fn read_agilent_fid_1d_bytes(procpar: &str, fid_bytes: &[u8]) -> Result<Spectrum1D> {
    let procpar = parse_procpar_for_reader(procpar)?;
    let (real, imaginary) = read_fid_values(fid_bytes)?;
    let axis = build_axis(real.len(), &procpar)?;
    let metadata = build_metadata(&procpar);
    Spectrum1D::new_complex(axis, real, Some(imaginary), metadata)
}

/// Reads arrayed one-dimensional FIDs from an Agilent/Varian dataset directory.
///
/// The path may point to the dataset directory or directly to `fid`. Each
/// binary trace is returned as one `Spectrum1D` with shared acquisition
/// metadata and array index/count metadata properties.
///
/// # Errors
///
/// Returns an error when the dataset is missing, malformed, not a
/// one-dimensional acquisition, or uses an unsupported data representation.
pub fn read_agilent_arrayed_fid_1d_dir(path: impl AsRef<Path>) -> Result<Vec<Spectrum1D>> {
    let dataset_dir = locate_dataset_dir(path.as_ref());
    let procpar_text = read_text(&dataset_dir.join("procpar"), "Agilent procpar")?;
    let fid_bytes = fs::read(dataset_dir.join("fid")).map_err(|error| RSpinError::Parse {
        format: "Agilent",
        message: format!("failed to read fid: {error}"),
    })?;

    read_agilent_arrayed_fid_1d_bytes(&procpar_text, &fid_bytes)
}

/// Reads arrayed one-dimensional FIDs from Agilent/Varian `procpar` text and
/// FID bytes.
///
/// # Errors
///
/// Returns an error when the `procpar` text is malformed, declares a
/// non-one-dimensional acquisition, or the FID bytes are malformed or
/// unsupported.
pub fn read_agilent_arrayed_fid_1d_bytes(
    procpar: &str,
    fid_bytes: &[u8],
) -> Result<Vec<Spectrum1D>> {
    let procpar = parse_procpar_for_reader(procpar)?;
    validate_arrayed_1d_procpar(&procpar)?;
    let (real, imaginary, x_count, spectrum_count) = read_arrayed_fid_values(fid_bytes)?;
    let axis = build_axis(x_count, &procpar)?;
    let metadata = build_metadata(&procpar);
    build_arrayed_1d_spectra(&real, &imaginary, x_count, spectrum_count, &axis, &metadata)
}

/// Reader for Agilent/Varian processed one-dimensional `phasefile` datasets.
///
/// The reader accepts the dataset directory, its `datdir` directory, or the
/// `phasefile` itself. It pairs the binary phasefile with the nearest `procpar`
/// and returns a frequency-domain spectrum.
#[derive(Clone, Copy, Debug, Default)]
pub struct AgilentProcessed1D;

impl AgilentProcessed1D {
    /// Reads a processed one-dimensional spectrum from an Agilent/Varian dataset.
    ///
    /// # Errors
    ///
    /// Returns an error when `phasefile` or `procpar` is missing, malformed, or
    /// the phasefile is multidimensional.
    pub fn read_dir(self, path: impl AsRef<Path>) -> Result<Spectrum1D> {
        read_agilent_processed_1d_dir(path)
    }
}

impl SpectrumPathReader for AgilentProcessed1D {
    type Output = Spectrum1D;

    fn read_path(&self, path: &Path) -> Result<Self::Output> {
        read_agilent_processed_1d_dir(path)
    }
}

/// Reads a processed one-dimensional spectrum from an Agilent/Varian phasefile.
///
/// The path may point to the dataset directory, `datdir`, or directly to the
/// `phasefile`. When `sw`, `sfrq`, `rfl`, and `rfp` are available in `procpar`,
/// the returned axis is chemical shift in ppm. Otherwise the reader falls back
/// to frequency offsets in hertz, then point indices.
///
/// # Errors
///
/// Returns an error when the dataset is missing, malformed, multidimensional,
/// or uses an unsupported data representation.
pub fn read_agilent_processed_1d_dir(path: impl AsRef<Path>) -> Result<Spectrum1D> {
    let path = path.as_ref();
    let phasefile_path = locate_phasefile_path(path)?;
    let procpar_path = locate_procpar_path(path, &phasefile_path)?;
    let procpar_text = read_text(&procpar_path, "Agilent procpar")?;
    let phasefile_bytes = fs::read(&phasefile_path).map_err(|error| RSpinError::Parse {
        format: "Agilent",
        message: format!(
            "failed to read phasefile at {}: {error}",
            phasefile_path.display()
        ),
    })?;

    read_agilent_processed_1d_bytes(&procpar_text, &phasefile_bytes)
}

/// Reads a processed one-dimensional spectrum from Agilent/Varian `procpar`
/// text and `phasefile` bytes.
///
/// # Errors
///
/// Returns an error when the `procpar` text is malformed, declares an
/// unsupported acquisition, or the phasefile bytes are malformed or unsupported.
pub fn read_agilent_processed_1d_bytes(
    procpar: &str,
    phasefile_bytes: &[u8],
) -> Result<Spectrum1D> {
    let procpar = parse_procpar_for_reader(procpar)?;
    let (real, imaginary) = read_phasefile_values(phasefile_bytes)?;
    let axis = build_processed_axis(real.len(), &procpar)?;
    let metadata = build_metadata(&procpar);
    Spectrum1D::new_complex(axis, real, imaginary, metadata)
}

/// Reader for Agilent/Varian raw two-dimensional FID directories.
///
/// The returned `Spectrum2D` preserves the acquired trace matrix. Each direct
/// trace contributes real values to `z` and imaginary values to `imaginary`.
/// Hypercomplex or arrayed indirect reconstruction is left to processing.
#[derive(Clone, Copy, Debug, Default)]
pub struct AgilentFid2D;

impl AgilentFid2D {
    /// Reads a raw two-dimensional FID from an Agilent/Varian dataset directory.
    ///
    /// # Errors
    ///
    /// Returns an error when `fid` or `procpar` is missing, malformed, not a
    /// two-dimensional acquisition, or stored in an unsupported numeric
    /// representation.
    pub fn read_dir(self, path: impl AsRef<Path>) -> Result<Spectrum2D> {
        read_agilent_fid_2d_dir(path)
    }
}

impl SpectrumPathReader for AgilentFid2D {
    type Output = Spectrum2D;

    fn read_path(&self, path: &Path) -> Result<Self::Output> {
        read_agilent_fid_2d_dir(path)
    }
}

/// Reads a raw two-dimensional FID from an Agilent/Varian dataset directory.
///
/// The path may point to the dataset directory or directly to `fid`.
///
/// # Errors
///
/// Returns an error when the dataset is missing, malformed, not a supported
/// two-dimensional acquisition, or uses an unsupported data representation.
pub fn read_agilent_fid_2d_dir(path: impl AsRef<Path>) -> Result<Spectrum2D> {
    let dataset_dir = locate_dataset_dir(path.as_ref());
    let procpar_text = read_text(&dataset_dir.join("procpar"), "Agilent procpar")?;
    let fid_bytes = fs::read(dataset_dir.join("fid")).map_err(|error| RSpinError::Parse {
        format: "Agilent",
        message: format!("failed to read fid: {error}"),
    })?;

    read_agilent_fid_2d_bytes(&procpar_text, &fid_bytes)
}

/// Reads a raw two-dimensional FID from Agilent/Varian `procpar` text and FID bytes.
///
/// # Errors
///
/// Returns an error when the `procpar` text is malformed, does not describe a
/// supported two-dimensional acquisition, or the FID bytes are malformed or
/// unsupported.
pub fn read_agilent_fid_2d_bytes(procpar: &str, fid_bytes: &[u8]) -> Result<Spectrum2D> {
    let procpar = parse_procpar_for_reader(procpar)?;
    validate_2d_procpar(&procpar)?;
    let (z, imaginary, x_count, y_count) = read_fid_matrix_values(fid_bytes)?;
    let x = build_axis(x_count, &procpar)?;
    let y = build_indirect_axis(y_count, &procpar)?;
    let metadata = build_metadata(&procpar);
    Spectrum2D::new_complex(x, y, z, Some(imaginary), metadata)
}

/// Reads arrayed two-dimensional FIDs from an Agilent/Varian dataset directory.
///
/// The path may point to the dataset directory or directly to `fid`. Each
/// array member is returned as one `Spectrum2D` with shared acquisition
/// metadata and array index/count metadata properties.
///
/// # Errors
///
/// Returns an error when the dataset is missing, malformed, not an arrayed
/// two-dimensional acquisition, or uses an unsupported data representation.
pub fn read_agilent_arrayed_fid_2d_dir(path: impl AsRef<Path>) -> Result<Vec<Spectrum2D>> {
    let dataset_dir = locate_dataset_dir(path.as_ref());
    let procpar_text = read_text(&dataset_dir.join("procpar"), "Agilent procpar")?;
    let fid_bytes = fs::read(dataset_dir.join("fid")).map_err(|error| RSpinError::Parse {
        format: "Agilent",
        message: format!("failed to read fid: {error}"),
    })?;

    read_agilent_arrayed_fid_2d_bytes(&procpar_text, &fid_bytes)
}

/// Reads arrayed two-dimensional FIDs from Agilent/Varian `procpar` text and
/// FID bytes.
///
/// # Errors
///
/// Returns an error when the `procpar` text is malformed, does not describe an
/// arrayed two-dimensional acquisition, or the FID bytes are malformed or
/// unsupported.
pub fn read_agilent_arrayed_fid_2d_bytes(
    procpar: &str,
    fid_bytes: &[u8],
) -> Result<Vec<Spectrum2D>> {
    let procpar = parse_procpar_for_reader(procpar)?;
    validate_2d_procpar(&procpar)?;
    let (z, imaginary, x_count, total_y_count) = read_fid_matrix_values(fid_bytes)?;
    let y_count = arrayed_2d_trace_count(&procpar, total_y_count)?;
    let spectrum_count = total_y_count / y_count;
    let x = build_axis(x_count, &procpar)?;
    let y = build_indirect_axis(y_count, &procpar)?;
    let metadata = build_metadata(&procpar);
    build_arrayed_2d_spectra(
        &z,
        &imaginary,
        Arrayed2DLayout {
            x_points: x_count,
            y_traces: y_count,
            series_len: spectrum_count,
        },
        &x,
        &y,
        &metadata,
    )
}

/// Reader for Agilent/Varian processed two-dimensional `phasefile` datasets.
///
/// The reader accepts the dataset directory, its `datdir` directory, or the
/// `phasefile` itself. It pairs the binary phasefile with the nearest `procpar`
/// and returns a frequency-domain matrix.
#[derive(Clone, Copy, Debug, Default)]
pub struct AgilentProcessed2D;

impl AgilentProcessed2D {
    /// Reads a processed two-dimensional spectrum from an Agilent/Varian dataset.
    ///
    /// # Errors
    ///
    /// Returns an error when `phasefile` or `procpar` is missing, malformed, or
    /// the phasefile is not two-dimensional.
    pub fn read_dir(self, path: impl AsRef<Path>) -> Result<Spectrum2D> {
        read_agilent_processed_2d_dir(path)
    }
}

impl SpectrumPathReader for AgilentProcessed2D {
    type Output = Spectrum2D;

    fn read_path(&self, path: &Path) -> Result<Self::Output> {
        read_agilent_processed_2d_dir(path)
    }
}

/// Reads a processed two-dimensional spectrum from an Agilent/Varian phasefile.
///
/// The path may point to the dataset directory, `datdir`, or directly to the
/// `phasefile`. The direct axis uses `sw`/`sfrq`/`rfl`/`rfp`; the indirect axis
/// uses `sw1` with `dfrq` or `sfrq`, plus `rfl1`/`rfp1`, when those values are
/// available. Missing reference metadata falls back to hertz or point axes.
///
/// # Errors
///
/// Returns an error when the dataset is missing, malformed, one-dimensional, or
/// uses an unsupported data representation.
pub fn read_agilent_processed_2d_dir(path: impl AsRef<Path>) -> Result<Spectrum2D> {
    let path = path.as_ref();
    let phasefile_path = locate_phasefile_path(path)?;
    let procpar_path = locate_procpar_path(path, &phasefile_path)?;
    let procpar_text = read_text(&procpar_path, "Agilent procpar")?;
    let phasefile_bytes = fs::read(&phasefile_path).map_err(|error| RSpinError::Parse {
        format: "Agilent",
        message: format!(
            "failed to read phasefile at {}: {error}",
            phasefile_path.display()
        ),
    })?;

    read_agilent_processed_2d_bytes(&procpar_text, &phasefile_bytes)
}

/// Reads a processed two-dimensional spectrum from Agilent/Varian `procpar`
/// text and `phasefile` bytes.
///
/// # Errors
///
/// Returns an error when the `procpar` text is malformed, does not describe a
/// supported two-dimensional acquisition, or the phasefile bytes are malformed
/// or unsupported.
pub fn read_agilent_processed_2d_bytes(
    procpar: &str,
    phasefile_bytes: &[u8],
) -> Result<Spectrum2D> {
    let procpar = parse_procpar_for_reader(procpar)?;
    validate_2d_procpar(&procpar)?;
    let matrix = read_phasefile_matrix_values(phasefile_bytes)?;
    let x = build_processed_axis_with(real_axis_parameters(), matrix.x_count, &procpar)?;
    let y = build_processed_axis_with(indirect_axis_parameters(), matrix.y_count, &procpar)?;
    let metadata = build_metadata(&procpar);
    Spectrum2D::new_complex(x, y, matrix.real, matrix.imaginary, metadata)
}

fn locate_dataset_dir(path: &Path) -> PathBuf {
    if path.is_file() {
        path.parent().map_or_else(PathBuf::new, Path::to_path_buf)
    } else {
        path.to_path_buf()
    }
}

fn locate_phasefile_path(path: &Path) -> Result<PathBuf> {
    if path.is_file() {
        return Ok(path.to_path_buf());
    }

    let datdir_phasefile = path.join("datdir").join("phasefile");
    if datdir_phasefile.is_file() {
        return Ok(datdir_phasefile);
    }

    let direct_phasefile = path.join("phasefile");
    if direct_phasefile.is_file() {
        return Ok(direct_phasefile);
    }

    Err(RSpinError::Parse {
        format: "Agilent",
        message: format!("missing Agilent phasefile below {}", path.display()),
    })
}

fn locate_procpar_path(path: &Path, phasefile_path: &Path) -> Result<PathBuf> {
    let candidates = [
        path.join("procpar"),
        phasefile_path
            .parent()
            .and_then(Path::parent)
            .map_or_else(PathBuf::new, |parent| parent.join("procpar")),
        phasefile_path
            .parent()
            .map_or_else(PathBuf::new, |parent| parent.join("procpar")),
    ];

    for candidate in candidates {
        if candidate.is_file() {
            return Ok(candidate);
        }
    }

    Err(RSpinError::Parse {
        format: "Agilent",
        message: format!("missing Agilent procpar for {}", phasefile_path.display()),
    })
}

/// Inspects routing metadata from Agilent/Varian `procpar` text.
///
/// # Errors
///
/// Returns an error when numeric routing fields are malformed.
pub fn inspect_agilent_procpar(input: &str) -> Result<AgilentProcparInfo> {
    let parameters = parse_procpar(input);
    procpar_info(&parameters)
}

/// Inspects routing metadata from an Agilent/Varian binary header.
///
/// The input may be a complete `fid`/`phasefile` payload or just the first
/// 32-byte file header.
///
/// # Errors
///
/// Returns an error when the binary header is missing or not recognized.
pub fn inspect_agilent_binary_bytes(bytes: &[u8]) -> Result<AgilentBinaryFileInfo> {
    let header = FileHeader::parse(bytes)?;
    agilent_binary_info(header)
}

/// Inspects routing metadata from an Agilent/Varian binary `fid` or `phasefile`.
///
/// This reads only the fixed-size file header, so callers can route large
/// vendor files without loading the whole payload.
///
/// # Errors
///
/// Returns an error when the file is missing, unreadable, or has an invalid
/// binary header.
pub fn inspect_agilent_binary_file(path: impl AsRef<Path>) -> Result<AgilentBinaryFileInfo> {
    let path = path.as_ref();
    let mut file = fs::File::open(path).map_err(|error| RSpinError::Parse {
        format: "Agilent",
        message: format!("failed to open binary file at {}: {error}", path.display()),
    })?;
    let mut header = [0_u8; FILE_HEADER_LEN];
    file.read_exact(&mut header)
        .map_err(|error| RSpinError::Parse {
            format: "Agilent",
            message: format!(
                "failed to read binary file header at {}: {error}",
                path.display()
            ),
        })?;
    inspect_agilent_binary_bytes(&header)
}

fn parse_procpar_for_reader(input: &str) -> Result<BTreeMap<String, Vec<String>>> {
    let parameters = parse_procpar(input);
    procpar_info(&parameters)?.validate_supported_by_current_readers()?;
    Ok(parameters)
}

fn procpar_info(parameters: &BTreeMap<String, Vec<String>>) -> Result<AgilentProcparInfo> {
    Ok(AgilentProcparInfo {
        software_revision: first_non_empty_text(parameters, &["vnmrrev", "version", "revision"]),
        sequence: first_text(parameters, "seqfil"),
        acquisition_dimension: first_usize(parameters, "acqdim")?,
        array_parameter: first_text(parameters, "array"),
        array_elements: first_usize(parameters, "arrayelemts")?,
        nucleus: first_text(parameters, "tn"),
        frequency_mhz: first_f64(parameters, "sfrq")?,
        spectral_width_hz: first_f64(parameters, "sw")?,
        operator: first_text(parameters, "operator").or_else(|| first_text(parameters, "username")),
    })
}

fn agilent_binary_info(header: FileHeader) -> Result<AgilentBinaryFileInfo> {
    let trace_count = header.trace_count()?;
    Ok(AgilentBinaryFileInfo {
        endian: header.endian.as_str().to_owned(),
        blocks: header.nblocks,
        traces_per_block: header.ntraces,
        values_per_trace: header.np_values,
        element_bytes: header.ebytes,
        trace_bytes: header.tbytes,
        block_bytes: header.bbytes,
        block_header_count: header.nbheaders,
        is_float: header.is_float(),
        is_complex: header.is_complex(),
        trace_count,
        dimension_count: if trace_count > 1 { 2 } else { 1 },
    })
}

fn first_non_empty_text(
    parameters: &BTreeMap<String, Vec<String>>,
    keys: &[&str],
) -> Option<String> {
    keys.iter().find_map(|key| {
        first_text(parameters, key).and_then(|value| {
            if value.trim().is_empty() {
                None
            } else {
                Some(value)
            }
        })
    })
}

fn read_fid_values(bytes: &[u8]) -> Result<(Vec<f64>, Vec<f64>)> {
    let header = FileHeader::parse(bytes)?;
    header.validate_1d()?;
    let (real, imaginary, _, _) = read_complex_trace_matrix(bytes, &header)?;
    Ok((real, imaginary))
}

fn read_arrayed_fid_values(bytes: &[u8]) -> Result<(Vec<f64>, Vec<f64>, usize, usize)> {
    let header = FileHeader::parse(bytes)?;
    header.validate_arrayed_1d()?;
    read_complex_trace_matrix(bytes, &header)
}

fn read_fid_matrix_values(bytes: &[u8]) -> Result<(Vec<f64>, Vec<f64>, usize, usize)> {
    let header = FileHeader::parse(bytes)?;
    header.validate_2d()?;
    read_complex_trace_matrix(bytes, &header)
}

fn read_phasefile_values(bytes: &[u8]) -> Result<(Vec<f64>, Option<Vec<f64>>)> {
    let header = FileHeader::parse(bytes)?;
    header.validate_processed_1d()?;
    let trace = first_trace(bytes, &header, "phasefile")?;
    if header.is_complex() {
        let mut real = Vec::with_capacity(header.np_values / 2);
        let mut imaginary = Vec::with_capacity(header.np_values / 2);
        append_complex_trace(trace, &header, &mut real, &mut imaginary)?;
        Ok((real, Some(imaginary)))
    } else {
        let mut real = Vec::with_capacity(header.np_values);
        append_real_trace(trace, &header, &mut real)?;
        Ok((real, None))
    }
}

struct PhasefileMatrixValues {
    real: Vec<f64>,
    imaginary: Option<Vec<f64>>,
    x_count: usize,
    y_count: usize,
}

fn read_phasefile_matrix_values(bytes: &[u8]) -> Result<PhasefileMatrixValues> {
    let header = FileHeader::parse(bytes)?;
    header.validate_processed_2d()?;
    let x_count = if header.is_complex() {
        header.np_values / 2
    } else {
        header.np_values
    };
    let y_count = header.trace_count()?;
    let matrix_len = x_count
        .checked_mul(y_count)
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "Agilent phasefile matrix size overflow".to_owned(),
        })?;
    let mut real = Vec::with_capacity(matrix_len);
    let mut imaginary = if header.is_complex() {
        Some(Vec::with_capacity(matrix_len))
    } else {
        None
    };

    for block_index in 0..header.nblocks {
        for trace_index in 0..header.ntraces {
            let trace = trace_at(bytes, &header, block_index, trace_index, "phasefile")?;
            if let Some(imaginary_values) = imaginary.as_mut() {
                append_complex_trace(trace, &header, &mut real, imaginary_values)?;
            } else {
                append_real_trace(trace, &header, &mut real)?;
            }
        }
    }

    Ok(PhasefileMatrixValues {
        real,
        imaginary,
        x_count,
        y_count,
    })
}

fn read_complex_trace_matrix(
    bytes: &[u8],
    header: &FileHeader,
) -> Result<(Vec<f64>, Vec<f64>, usize, usize)> {
    header.validate_complex()?;
    let x_count = header.np_values / 2;
    let y_count = header.trace_count()?;
    let matrix_len = x_count
        .checked_mul(y_count)
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "Agilent FID matrix size overflow".to_owned(),
        })?;
    let mut real = Vec::with_capacity(matrix_len);
    let mut imaginary = Vec::with_capacity(matrix_len);

    for block_index in 0..header.nblocks {
        for trace_index in 0..header.ntraces {
            let trace = trace_at(bytes, header, block_index, trace_index, "fid")?;
            append_complex_trace(trace, header, &mut real, &mut imaginary)?;
        }
    }
    Ok((real, imaginary, x_count, y_count))
}

fn build_arrayed_1d_spectra(
    real: &[f64],
    imaginary: &[f64],
    x_count: usize,
    spectrum_count: usize,
    axis: &Axis,
    metadata: &Metadata,
) -> Result<Vec<Spectrum1D>> {
    if spectrum_count == 0 {
        return Err(RSpinError::InvalidSpectrum {
            message: "Agilent arrayed FID must contain at least one trace".to_owned(),
        });
    }

    let expected_len =
        x_count
            .checked_mul(spectrum_count)
            .ok_or_else(|| RSpinError::InvalidSpectrum {
                message: "Agilent arrayed FID size overflow".to_owned(),
            })?;
    if real.len() != expected_len || imaginary.len() != expected_len {
        return Err(RSpinError::InvalidSpectrum {
            message: "Agilent arrayed FID trace sizes are inconsistent".to_owned(),
        });
    }

    let mut spectra = Vec::with_capacity(spectrum_count);
    for spectrum_index in 0..spectrum_count {
        let start =
            spectrum_index
                .checked_mul(x_count)
                .ok_or_else(|| RSpinError::InvalidSpectrum {
                    message: "Agilent arrayed FID trace offset overflow".to_owned(),
                })?;
        let end = start
            .checked_add(x_count)
            .ok_or_else(|| RSpinError::InvalidSpectrum {
                message: "Agilent arrayed FID trace end overflow".to_owned(),
            })?;
        let spectrum_metadata = metadata
            .clone()
            .with_property("agilent.array.index", spectrum_index.to_string())
            .with_property("agilent.array.count", spectrum_count.to_string());
        spectra.push(Spectrum1D::new_complex(
            axis.clone(),
            real[start..end].to_vec(),
            Some(imaginary[start..end].to_vec()),
            spectrum_metadata,
        )?);
    }
    Ok(spectra)
}

#[derive(Clone, Copy, Debug)]
struct Arrayed2DLayout {
    x_points: usize,
    y_traces: usize,
    series_len: usize,
}

fn build_arrayed_2d_spectra(
    z: &[f64],
    imaginary: &[f64],
    layout: Arrayed2DLayout,
    x: &Axis,
    y: &Axis,
    metadata: &Metadata,
) -> Result<Vec<Spectrum2D>> {
    let Arrayed2DLayout {
        x_points,
        y_traces,
        series_len,
    } = layout;

    if series_len == 0 {
        return Err(RSpinError::InvalidSpectrum {
            message: "Agilent arrayed 2D FID must contain at least one spectrum".to_owned(),
        });
    }

    let matrix_len = x_points
        .checked_mul(y_traces)
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "Agilent arrayed 2D FID matrix size overflow".to_owned(),
        })?;
    let expected_len =
        matrix_len
            .checked_mul(series_len)
            .ok_or_else(|| RSpinError::InvalidSpectrum {
                message: "Agilent arrayed 2D FID size overflow".to_owned(),
            })?;
    if z.len() != expected_len || imaginary.len() != expected_len {
        return Err(RSpinError::InvalidSpectrum {
            message: "Agilent arrayed 2D FID trace sizes are inconsistent".to_owned(),
        });
    }

    let mut spectra = Vec::with_capacity(series_len);
    for spectrum_index in 0..series_len {
        let start =
            spectrum_index
                .checked_mul(matrix_len)
                .ok_or_else(|| RSpinError::InvalidSpectrum {
                    message: "Agilent arrayed 2D FID matrix offset overflow".to_owned(),
                })?;
        let end = start
            .checked_add(matrix_len)
            .ok_or_else(|| RSpinError::InvalidSpectrum {
                message: "Agilent arrayed 2D FID matrix end overflow".to_owned(),
            })?;
        let spectrum_metadata = metadata
            .clone()
            .with_property("agilent.array.index", spectrum_index.to_string())
            .with_property("agilent.array.count", series_len.to_string())
            .with_property("agilent.array.traces_per_spectrum", y_traces.to_string());
        spectra.push(Spectrum2D::new_complex(
            x.clone(),
            y.clone(),
            z[start..end].to_vec(),
            Some(imaginary[start..end].to_vec()),
            spectrum_metadata,
        )?);
    }
    Ok(spectra)
}

pub(crate) fn is_agilent_arrayed_2d_series_array(value: &str) -> bool {
    let mut has_series_parameter = false;
    for token in value
        .split(|character: char| {
            character == ',' || character == '(' || character == ')' || character.is_whitespace()
        })
        .map(str::trim)
        .filter(|token| !token.is_empty())
    {
        if !token.eq_ignore_ascii_case("phase") {
            has_series_parameter = true;
        }
    }
    has_series_parameter
}

fn first_trace<'a>(
    bytes: &'a [u8],
    header: &FileHeader,
    file_label: &'static str,
) -> Result<&'a [u8]> {
    trace_at(bytes, header, 0, 0, file_label)
}

fn trace_at<'a>(
    bytes: &'a [u8],
    header: &FileHeader,
    block_index: usize,
    trace_index: usize,
    file_label: &'static str,
) -> Result<&'a [u8]> {
    let block_header_len = header
        .nbheaders
        .checked_mul(BLOCK_HEADER_LEN)
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "Agilent block header size overflow".to_owned(),
        })?;
    let block_start = FILE_HEADER_LEN
        .checked_add(block_index.checked_mul(header.bbytes).ok_or_else(|| {
            RSpinError::InvalidSpectrum {
                message: "Agilent block offset overflow".to_owned(),
            }
        })?)
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "Agilent block offset overflow".to_owned(),
        })?;
    let data_start =
        block_start
            .checked_add(block_header_len)
            .ok_or_else(|| RSpinError::InvalidSpectrum {
                message: "Agilent block data offset overflow".to_owned(),
            })?;
    let trace_start = data_start
        .checked_add(trace_index.checked_mul(header.tbytes).ok_or_else(|| {
            RSpinError::InvalidSpectrum {
                message: "Agilent trace offset overflow".to_owned(),
            }
        })?)
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "Agilent trace offset overflow".to_owned(),
        })?;
    let trace_end =
        trace_start
            .checked_add(header.tbytes)
            .ok_or_else(|| RSpinError::InvalidSpectrum {
                message: "Agilent trace end overflow".to_owned(),
            })?;
    bytes
        .get(trace_start..trace_end)
        .ok_or_else(|| RSpinError::Parse {
            format: "Agilent",
            message: format!(
                "{file_label} has {} bytes but trace ending at {trace_end} is required",
                bytes.len()
            ),
        })
}

fn append_real_trace(bytes: &[u8], header: &FileHeader, real: &mut Vec<f64>) -> Result<()> {
    for point_index in 0..header.np_values {
        let offset =
            point_index
                .checked_mul(header.ebytes)
                .ok_or_else(|| RSpinError::InvalidSpectrum {
                    message: "Agilent data offset overflow".to_owned(),
                })?;
        real.push(decode_value(bytes, header, offset)?);
    }
    Ok(())
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
                message: "binary file is shorter than the file header".to_owned(),
            });
        }
        [Endian::Big, Endian::Little]
            .into_iter()
            .find_map(|endian| {
                Self::parse_with_endian(bytes, endian).filter(|header| header.is_plausible())
            })
            .ok_or_else(|| RSpinError::Parse {
                format: "Agilent",
                message: "binary file header is not recognized".to_owned(),
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
        let block_header_len = self.nbheaders.saturating_mul(BLOCK_HEADER_LEN);
        let trace_bytes = self.ntraces.saturating_mul(self.tbytes);
        let minimum_block_bytes = block_header_len.saturating_add(trace_bytes);
        self.nblocks > 0
            && self.ntraces > 0
            && self.np_values > 0
            && (!self.is_complex() || self.np_values.is_multiple_of(2))
            && matches!(self.ebytes, 2 | 4 | 8)
            && self.tbytes == self.np_values.saturating_mul(self.ebytes)
            && self.nbheaders > 0
            && self.bbytes >= minimum_block_bytes
    }

    fn validate_1d(self) -> Result<()> {
        if self.nblocks != 1 || self.ntraces != 1 {
            return Err(RSpinError::Unsupported {
                feature: "Agilent arrayed or multidimensional FID",
            });
        }
        self.validate_complex()?;
        Ok(())
    }

    fn validate_arrayed_1d(self) -> Result<()> {
        self.validate_complex()?;
        if self.trace_count()? <= 1 {
            return Err(RSpinError::Unsupported {
                feature: "Agilent non-arrayed FID in arrayed 1D reader",
            });
        }
        Ok(())
    }

    fn validate_2d(self) -> Result<()> {
        if self.trace_count()? <= 1 {
            return Err(RSpinError::Unsupported {
                feature: "Agilent one-dimensional FID in 2D reader",
            });
        }
        self.validate_complex()?;
        Ok(())
    }

    fn validate_processed_1d(self) -> Result<()> {
        if self.nblocks != 1 || self.ntraces != 1 {
            return Err(RSpinError::Unsupported {
                feature: "Agilent multidimensional phasefile",
            });
        }
        if self.is_complex() && !self.np_values.is_multiple_of(2) {
            return Err(RSpinError::InvalidSpectrum {
                message: "Agilent complex phasefile trace must contain an even value count"
                    .to_owned(),
            });
        }
        Ok(())
    }

    fn validate_processed_2d(self) -> Result<()> {
        if self.trace_count()? <= 1 {
            return Err(RSpinError::Unsupported {
                feature: "Agilent one-dimensional phasefile in 2D reader",
            });
        }
        if self.is_complex() && !self.np_values.is_multiple_of(2) {
            return Err(RSpinError::InvalidSpectrum {
                message: "Agilent complex phasefile trace must contain an even value count"
                    .to_owned(),
            });
        }
        Ok(())
    }

    fn validate_complex(self) -> Result<()> {
        if !self.is_complex() {
            return Err(RSpinError::Unsupported {
                feature: "Agilent real-only FID",
            });
        }
        if !self.np_values.is_multiple_of(2) {
            return Err(RSpinError::InvalidSpectrum {
                message: "Agilent complex FID trace must contain an even value count".to_owned(),
            });
        }
        Ok(())
    }

    fn is_float(self) -> bool {
        self.status & STATUS_FLOAT != 0
    }

    fn is_complex(self) -> bool {
        self.status & (STATUS_COMPLEX | STATUS_COMPLEX_ALT) != 0
    }

    fn trace_count(self) -> Result<usize> {
        self.nblocks
            .checked_mul(self.ntraces)
            .ok_or_else(|| RSpinError::InvalidSpectrum {
                message: "Agilent FID trace count overflow".to_owned(),
            })
    }
}

#[derive(Clone, Copy)]
enum Endian {
    Big,
    Little,
}

impl Endian {
    fn as_str(self) -> &'static str {
        match self {
            Self::Big => "big",
            Self::Little => "little",
        }
    }

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

fn append_complex_trace(
    bytes: &[u8],
    header: &FileHeader,
    real: &mut Vec<f64>,
    imaginary: &mut Vec<f64>,
) -> Result<()> {
    for pair_index in 0..(header.np_values / 2) {
        let real_offset = pair_index
            .checked_mul(2)
            .and_then(|index| index.checked_mul(header.ebytes))
            .ok_or_else(|| RSpinError::InvalidSpectrum {
                message: "Agilent data offset overflow".to_owned(),
            })?;
        let imaginary_offset =
            real_offset
                .checked_add(header.ebytes)
                .ok_or_else(|| RSpinError::InvalidSpectrum {
                    message: "Agilent data offset overflow".to_owned(),
                })?;
        real.push(decode_value(bytes, header, real_offset)?);
        imaginary.push(decode_value(bytes, header, imaginary_offset)?);
    }
    Ok(())
}

fn decode_value(bytes: &[u8], header: &FileHeader, offset: usize) -> Result<f64> {
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
                feature: "Agilent binary numeric representation",
            });
        }
    };
    if !value.is_finite() {
        return Err(RSpinError::NonFinite {
            field: "Agilent binary data",
        });
    }
    Ok(value)
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

fn build_processed_axis(
    point_count: usize,
    parameters: &BTreeMap<String, Vec<String>>,
) -> Result<Axis> {
    build_processed_axis_with(real_axis_parameters(), point_count, parameters)
}

#[derive(Clone, Copy)]
struct ProcessedAxisParameters {
    sw_key: &'static str,
    frequency_key: &'static str,
    fallback_frequency_key: Option<&'static str>,
    rfl_key: &'static str,
    rfp_key: &'static str,
    hz_label: &'static str,
    point_label: &'static str,
}

fn real_axis_parameters() -> ProcessedAxisParameters {
    ProcessedAxisParameters {
        sw_key: "sw",
        frequency_key: "sfrq",
        fallback_frequency_key: None,
        rfl_key: "rfl",
        rfp_key: "rfp",
        hz_label: "frequency offset",
        point_label: "point",
    }
}

fn indirect_axis_parameters() -> ProcessedAxisParameters {
    ProcessedAxisParameters {
        sw_key: "sw1",
        frequency_key: "dfrq",
        fallback_frequency_key: Some("sfrq"),
        rfl_key: "rfl1",
        rfp_key: "rfp1",
        hz_label: "indirect frequency offset",
        point_label: "indirect point",
    }
}

fn build_processed_axis_with(
    axis_parameters: ProcessedAxisParameters,
    point_count: usize,
    parameters: &BTreeMap<String, Vec<String>>,
) -> Result<Axis> {
    let sw = first_f64(parameters, axis_parameters.sw_key)?;
    let frequency = match first_f64(parameters, axis_parameters.frequency_key)? {
        Some(value) => Some(value),
        None => match axis_parameters.fallback_frequency_key {
            Some(key) => first_f64(parameters, key)?,
            None => None,
        },
    };
    let rfl = first_f64(parameters, axis_parameters.rfl_key)?;
    let rfp = first_f64(parameters, axis_parameters.rfp_key)?;

    match (sw, frequency, rfl, rfp) {
        (Some(sw), Some(sfrq), Some(rfl), Some(rfp)) if sw > 0.0 && sfrq > 0.0 => {
            let start_ppm = (sw - rfl + rfp) / sfrq;
            let end_ppm = (-rfl + rfp) / sfrq;
            Axis::linear_ppm(start_ppm, end_ppm, point_count)
        }
        (Some(sw), _, _, _) if sw > 0.0 => Axis::linear(
            axis_parameters.hz_label,
            Unit::Hertz,
            sw / 2.0,
            -sw / 2.0,
            point_count,
        ),
        _ => {
            let end = u32::try_from(point_count.saturating_sub(1)).map_or(0.0, f64::from);
            Axis::linear(
                axis_parameters.point_label,
                Unit::Points,
                0.0,
                end,
                point_count,
            )
        }
    }
}

fn build_indirect_axis(
    point_count: usize,
    parameters: &BTreeMap<String, Vec<String>>,
) -> Result<Axis> {
    let is_arrayed = first_usize(parameters, "arrayelemts")?
        .is_some_and(|array_elements| array_elements > 1)
        || first_text(parameters, "array").is_some_and(|array| !array.is_empty());
    match (first_f64(parameters, "sw1")?, is_arrayed) {
        (Some(sw1), false) if sw1 > 0.0 => {
            let end = if point_count <= 1 {
                0.0
            } else {
                let segments =
                    u32::try_from(point_count - 1).map_err(|_| RSpinError::InvalidAxis {
                        message: "Agilent indirect point count is too large".to_owned(),
                    })?;
                f64::from(segments) / sw1
            };
            Axis::linear("indirect time", Unit::Seconds, 0.0, end, point_count)
        }
        _ => {
            let end = u32::try_from(point_count.saturating_sub(1)).map_or(0.0, f64::from);
            Axis::linear("indirect trace", Unit::Points, 0.0, end, point_count)
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
        properties: procpar_metadata_properties(parameters),
        ..Metadata::default()
    }
}

fn procpar_metadata_properties(
    parameters: &BTreeMap<String, Vec<String>>,
) -> BTreeMap<String, String> {
    parameters
        .iter()
        .map(|(key, values)| (format!("agilent.procpar.{key}"), values.join(" ")))
        .collect()
}

fn validate_2d_procpar(parameters: &BTreeMap<String, Vec<String>>) -> Result<()> {
    match first_usize(parameters, "acqdim")? {
        Some(2) | None => Ok(()),
        Some(0 | 1) => Err(RSpinError::Unsupported {
            feature: "Agilent one-dimensional FID in 2D reader",
        }),
        Some(_) => Err(RSpinError::Unsupported {
            feature: "Agilent three-or-higher-dimensional FID",
        }),
    }
}

fn validate_arrayed_1d_procpar(parameters: &BTreeMap<String, Vec<String>>) -> Result<()> {
    match first_usize(parameters, "acqdim")? {
        Some(0 | 1) | None => Ok(()),
        Some(2) => Err(RSpinError::Unsupported {
            feature: "Agilent two-dimensional FID in arrayed 1D reader",
        }),
        Some(_) => Err(RSpinError::Unsupported {
            feature: "Agilent three-or-higher-dimensional FID",
        }),
    }
}

fn arrayed_2d_trace_count(
    parameters: &BTreeMap<String, Vec<String>>,
    total_y_count: usize,
) -> Result<usize> {
    let Some(array_parameter) = first_text(parameters, "array") else {
        return Err(RSpinError::Unsupported {
            feature: "Agilent non-arrayed FID in arrayed 2D reader",
        });
    };
    if !is_agilent_arrayed_2d_series_array(&array_parameter) {
        return Err(RSpinError::Unsupported {
            feature: "Agilent non-arrayed FID in arrayed 2D reader",
        });
    }

    let y_count = first_usize(parameters, "ni")?.ok_or(RSpinError::Unsupported {
        feature: "Agilent arrayed 2D FID without ni trace count",
    })?;
    if y_count == 0 {
        return Err(RSpinError::InvalidSpectrum {
            message: "Agilent arrayed 2D FID ni must be positive".to_owned(),
        });
    }
    if total_y_count <= y_count {
        return Err(RSpinError::Unsupported {
            feature: "Agilent non-arrayed FID in arrayed 2D reader",
        });
    }
    if !total_y_count.is_multiple_of(y_count) {
        return Err(RSpinError::InvalidSpectrum {
            message: format!(
                "Agilent arrayed 2D FID has {total_y_count} traces but ni is {y_count}"
            ),
        });
    }
    Ok(y_count)
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
