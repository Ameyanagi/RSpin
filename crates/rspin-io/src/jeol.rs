//! JEOL Delta `.jdf` import.

use std::{fs, path::Path, str::FromStr};

use rspin_core::{Axis, Metadata, Nucleus, RSpinError, Result, Spectrum1D, Spectrum2D, Unit};

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
        molecules: Vec::new(),
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
