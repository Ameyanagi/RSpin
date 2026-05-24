//! JEOL Delta `.jdf` import.

use std::{fs, path::Path, str::FromStr};

use rspin_core::{Axis, Metadata, Nucleus, RSpinError, Result, Spectrum1D, Spectrum2D, Unit};
use serde::{Deserialize, Serialize};

use crate::SpectrumPathReader;

mod binary;
mod data;
mod header;
mod parameters;

use data::{read_data_matrix_sections, read_data_sections};
use header::Header;
use parameters::Parameters;

const VALUE_TYPE_STRING: i32 = 0;
const VALUE_TYPE_INTEGER: i32 = 1;
const VALUE_TYPE_FLOAT: i32 = 2;
const DATA_TYPE_FLOAT_64: u8 = 0;
const DATA_TYPE_FLOAT_32: u8 = 1;
const DATA_FORMAT_ONE_D: u8 = 1;
const DATA_FORMAT_TWO_D: u8 = 2;
const AXIS_TYPE_COMPLEX: u8 = 3;
const AXIS_TYPE_REAL_COMPLEX: u8 = 4;
const PARAMETER_RECORD_LEN: usize = 64;
const PARAMETER_HEADER_LEN: usize = 16;

/// Parsed JEOL Delta `.jdf` file format version.
///
/// The binary JDF header stores version components separately. `RSpin` exposes
/// them so callers can inspect format changes before choosing a reader.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JeolJdfVersion {
    /// Human-readable `major.minor` label.
    pub raw: String,
    /// JDF major version.
    pub major: u8,
    /// JDF minor version.
    pub minor: u16,
}

impl JeolJdfVersion {
    /// Creates a JEOL JDF version value.
    #[must_use]
    pub fn new(major: u8, minor: u16) -> Self {
        Self {
            raw: format!("{major}.{minor}"),
            major,
            minor,
        }
    }

    /// Returns true when `RSpin`'s current JDF readers support this version.
    #[must_use]
    pub fn is_supported_by_current_reader(&self) -> bool {
        self.major == 1
    }

    /// Validates that the version is supported by `RSpin`'s current readers.
    ///
    /// # Errors
    ///
    /// Returns an unsupported-feature error for future or otherwise unsupported
    /// JEOL JDF major versions.
    pub fn validate_supported_by_current_reader(&self) -> Result<()> {
        if self.is_supported_by_current_reader() {
            Ok(())
        } else {
            Err(RSpinError::Unsupported {
                feature: "JEOL JDF version",
            })
        }
    }
}

/// Routing metadata from a JEOL Delta `.jdf` header.
///
/// This type is intentionally small and serializable for native and WASM
/// callers that need to inspect a file before reading full spectral data.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JeolJdfInfo {
    /// Parsed JDF header version.
    pub version: JeolJdfVersion,
    /// Header endianness label, `big` or `little`.
    pub endian: String,
    /// Declared data dimensionality.
    pub dimension_count: usize,
    /// Raw JDF data format code.
    pub data_format_code: u8,
    /// Raw JDF numeric data type code.
    pub data_type_code: u8,
    /// Point counts for declared dimensions, up to the eight counts stored in
    /// the JDF header.
    pub point_counts: Vec<usize>,
    /// Optional title stored in the JDF header.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

impl JeolJdfInfo {
    /// Returns a descriptive name for the raw JDF data format code.
    #[must_use]
    pub fn data_format_name(&self) -> &'static str {
        match self.data_format_code {
            DATA_FORMAT_ONE_D => "one_d",
            DATA_FORMAT_TWO_D => "two_d",
            _ => "unknown",
        }
    }

    /// Returns a descriptive name for the raw JDF numeric type code.
    #[must_use]
    pub fn data_type_name(&self) -> &'static str {
        match self.data_type_code {
            DATA_TYPE_FLOAT_64 => "float64",
            DATA_TYPE_FLOAT_32 => "float32",
            _ => "unknown",
        }
    }

    /// Returns true when the header can be routed to current `RSpin` readers.
    #[must_use]
    pub fn is_supported_by_current_readers(&self) -> bool {
        self.version.is_supported_by_current_reader()
            && matches!(self.data_type_code, DATA_TYPE_FLOAT_64 | DATA_TYPE_FLOAT_32)
            && matches!(
                (self.dimension_count, self.data_format_code),
                (1, DATA_FORMAT_ONE_D) | (2, DATA_FORMAT_TWO_D)
            )
    }

    /// Validates that the header can be routed to current `RSpin` readers.
    ///
    /// # Errors
    ///
    /// Returns an unsupported-feature error for unsupported JDF versions,
    /// dimensionality, data format, or numeric representation.
    pub fn validate_supported_by_current_readers(&self) -> Result<()> {
        self.version.validate_supported_by_current_reader()?;
        if !matches!(self.data_type_code, DATA_TYPE_FLOAT_64 | DATA_TYPE_FLOAT_32) {
            return Err(RSpinError::Unsupported {
                feature: "JEOL JDF numeric representation",
            });
        }
        if !matches!(
            (self.dimension_count, self.data_format_code),
            (1, DATA_FORMAT_ONE_D) | (2, DATA_FORMAT_TWO_D)
        ) {
            return Err(RSpinError::Unsupported {
                feature: "JEOL JDF dimensionality",
            });
        }
        Ok(())
    }
}

/// Reader for JEOL Delta `.jdf` one-dimensional spectra or FIDs.
#[derive(Clone, Copy, Debug, Default)]
pub struct JeolJdf1D;

impl JeolJdf1D {
    /// Reads a JEOL Delta `.jdf` one-dimensional file.
    ///
    /// # Errors
    ///
    /// Returns an error when the file is missing, malformed, multidimensional,
    /// or uses an unsupported numeric representation.
    pub fn read_file(self, path: impl AsRef<Path>) -> Result<Spectrum1D> {
        read_jeol_jdf_1d_file(path)
    }

    /// Reads a JEOL Delta `.jdf` one-dimensional payload.
    ///
    /// # Errors
    ///
    /// Returns an error when the payload is malformed, multidimensional, or
    /// uses an unsupported numeric representation.
    pub fn read_bytes(self, bytes: &[u8]) -> Result<Spectrum1D> {
        read_jeol_jdf_1d_bytes(bytes)
    }
}

impl SpectrumPathReader for JeolJdf1D {
    type Output = Spectrum1D;

    fn read_path(&self, path: &Path) -> Result<Self::Output> {
        read_jeol_jdf_1d_file(path)
    }
}

/// Reads a JEOL Delta `.jdf` one-dimensional file.
///
/// # Errors
///
/// Returns an error when the file is missing, malformed, multidimensional, or
/// uses an unsupported numeric representation.
pub fn read_jeol_jdf_1d_file(path: impl AsRef<Path>) -> Result<Spectrum1D> {
    let path = path.as_ref();
    let bytes = fs::read(path).map_err(|error| RSpinError::Parse {
        format: "JEOL",
        message: format!("failed to read {}: {error}", path.display()),
    })?;
    read_jeol_jdf_1d_bytes(&bytes)
}

/// Reads a JEOL Delta `.jdf` one-dimensional payload.
///
/// # Errors
///
/// Returns an error when the payload is malformed, multidimensional, or uses an
/// unsupported numeric representation.
pub fn read_jeol_jdf_1d_bytes(bytes: &[u8]) -> Result<Spectrum1D> {
    let header = Header::parse(bytes)?;
    header.validate_1d()?;

    let parameters = Parameters::parse(bytes, &header)?;
    let (real, imaginary) = read_data_sections(bytes, &header)?;
    let axis = build_axis(&header, 0, header.point_count()?)?;
    let metadata = build_metadata(&header, &parameters);

    Spectrum1D::new_complex(axis, real, imaginary, metadata)
}

/// Reader for JEOL Delta `.jdf` two-dimensional spectra or FIDs.
#[derive(Clone, Copy, Debug, Default)]
pub struct JeolJdf2D;

impl JeolJdf2D {
    /// Reads a JEOL Delta `.jdf` two-dimensional file.
    ///
    /// # Errors
    ///
    /// Returns an error when the file is missing, malformed,
    /// non-two-dimensional, or uses an unsupported numeric representation.
    pub fn read_file(self, path: impl AsRef<Path>) -> Result<Spectrum2D> {
        read_jeol_jdf_2d_file(path)
    }

    /// Reads a JEOL Delta `.jdf` two-dimensional payload.
    ///
    /// # Errors
    ///
    /// Returns an error when the payload is malformed, non-two-dimensional, or
    /// uses an unsupported numeric representation.
    pub fn read_bytes(self, bytes: &[u8]) -> Result<Spectrum2D> {
        read_jeol_jdf_2d_bytes(bytes)
    }
}

impl SpectrumPathReader for JeolJdf2D {
    type Output = Spectrum2D;

    fn read_path(&self, path: &Path) -> Result<Self::Output> {
        read_jeol_jdf_2d_file(path)
    }
}

/// Reads a JEOL Delta `.jdf` two-dimensional file.
///
/// # Errors
///
/// Returns an error when the file is missing, malformed, non-two-dimensional,
/// or uses an unsupported numeric representation.
pub fn read_jeol_jdf_2d_file(path: impl AsRef<Path>) -> Result<Spectrum2D> {
    let path = path.as_ref();
    let bytes = fs::read(path).map_err(|error| RSpinError::Parse {
        format: "JEOL",
        message: format!("failed to read {}: {error}", path.display()),
    })?;
    read_jeol_jdf_2d_bytes(&bytes)
}

/// Reads a JEOL Delta `.jdf` two-dimensional payload.
///
/// JEOL hypercomplex files may contain more than two planes. `RSpin`'s current
/// `Spectrum2D` model stores one optional companion plane, so this reader keeps
/// the first real plane and the first following companion plane.
///
/// # Errors
///
/// Returns an error when the payload is malformed, non-two-dimensional, or uses
/// an unsupported numeric representation.
pub fn read_jeol_jdf_2d_bytes(bytes: &[u8]) -> Result<Spectrum2D> {
    let header = Header::parse(bytes)?;
    header.validate_2d()?;

    let parameters = Parameters::parse(bytes, &header)?;
    let (z, imaginary, x_count, y_count) = read_data_matrix_sections(bytes, &header)?;
    let x = build_axis(&header, 0, x_count)?;
    let y = build_axis(&header, 1, y_count)?;
    let metadata = build_metadata(&header, &parameters);

    Spectrum2D::new_complex(x, y, z, imaginary, metadata)
}

/// Inspects routing metadata from a JEOL Delta `.jdf` file.
///
/// # Errors
///
/// Returns an error when the file is missing or the JDF header is malformed.
pub fn inspect_jeol_jdf_file(path: impl AsRef<Path>) -> Result<JeolJdfInfo> {
    let path = path.as_ref();
    let bytes = fs::read(path).map_err(|error| RSpinError::Parse {
        format: "JEOL",
        message: format!("failed to read {}: {error}", path.display()),
    })?;
    inspect_jeol_jdf_bytes(&bytes)
}

/// Inspects routing metadata from a JEOL Delta `.jdf` payload.
///
/// # Errors
///
/// Returns an error when the JDF header is malformed.
pub fn inspect_jeol_jdf_bytes(bytes: &[u8]) -> Result<JeolJdfInfo> {
    let header = Header::parse(bytes)?;
    info_from_header(&header)
}

fn info_from_header(header: &Header) -> Result<JeolJdfInfo> {
    Ok(JeolJdfInfo {
        version: JeolJdfVersion::new(header.major_version(), header.minor_version()),
        endian: match header.endian {
            binary::Endian::Big => "big",
            binary::Endian::Little => "little",
        }
        .to_owned(),
        dimension_count: header.dimension_count(),
        data_format_code: header.data_format_code(),
        data_type_code: header.data_type_code(),
        point_counts: header.point_counts()?,
        title: header.title.clone(),
    })
}

fn build_axis(header: &Header, axis_index: usize, point_count: usize) -> Result<Axis> {
    let unit = header.data_units[axis_index].axis_unit();
    let label = match unit {
        Unit::Ppm => "chemical shift",
        Unit::Hertz => "frequency",
        Unit::Seconds => "time",
        Unit::Points => "point",
        _ => "axis",
    };
    Axis::linear(
        label,
        unit,
        header.data_axis_start[axis_index],
        header.data_axis_stop[axis_index],
        point_count,
    )
}

fn build_metadata(header: &Header, parameters: &Parameters) -> Metadata {
    let nucleus = parameters
        .string("x_domain")
        .and_then(|value| parse_jeol_nucleus(value).ok());
    let frequency_mhz = parameters
        .magnitude("x_freq")
        .map(|(value, _unit)| value / 1_000_000.0);
    let solvent = parameters.string("solvent").map(ToOwned::to_owned);
    let temperature_k = parameters
        .magnitude("temp_get")
        .map(|(value, unit)| temperature_to_kelvin(value, unit.base));
    let name = parameters
        .string("sample_id")
        .map(ToOwned::to_owned)
        .or_else(|| header.title.clone());

    Metadata {
        name,
        nucleus,
        frequency_mhz,
        solvent,
        temperature_k,
        origin: Some("JEOL".to_owned()),
        properties: parameters.properties(),
        ..Metadata::default()
    }
}

fn parse_jeol_nucleus(value: &str) -> Result<Nucleus> {
    match value.trim() {
        "Proton" => Ok(Nucleus::Hydrogen1),
        "Carbon13" => Ok(Nucleus::Carbon13),
        "Fluorine19" => Ok(Nucleus::Fluorine19),
        "Phosphorus31" => Ok(Nucleus::Phosphorus31),
        other => Nucleus::from_str(other),
    }
}

fn temperature_to_kelvin(value: f64, unit_base: u8) -> f64 {
    match unit_base {
        4 => value + 273.15,
        _ => value,
    }
}

#[cfg(test)]
mod tests;
