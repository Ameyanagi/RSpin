//! Bruker processed two-dimensional spectrum import.

use std::path::{Path, PathBuf};

use rspin_core::{RSpinError, Result, Spectrum2D};

use crate::SpectrumPathReader;

use super::{
    build_axis, build_metadata, parse_parameter_file, read_acqus, read_processed_i32_data,
    read_text, read_title, required_usize,
};

/// Reader for Bruker processed two-dimensional datasets.
///
/// The reader accepts either the dataset root containing `pdata/1` or the
/// processed directory itself. It supports real `2rr` data and optional `2ri`
/// data stored as 32-bit integers with Bruker `procs`/`proc2s` metadata.
#[derive(Clone, Copy, Debug, Default)]
pub struct BrukerProcessed2D;

impl BrukerProcessed2D {
    /// Reads a processed two-dimensional spectrum from a Bruker dataset path.
    ///
    /// # Errors
    ///
    /// Returns an error when required `procs`, `proc2s`, or `2rr` files are
    /// missing, malformed, or use an unsupported binary data type.
    pub fn read_dir(self, path: impl AsRef<Path>) -> Result<Spectrum2D> {
        read_bruker_processed_2d_dir(path)
    }
}

impl SpectrumPathReader for BrukerProcessed2D {
    type Output = Spectrum2D;

    fn read_path(&self, path: &Path) -> Result<Self::Output> {
        read_bruker_processed_2d_dir(path)
    }
}

/// Reads a processed two-dimensional spectrum from a Bruker dataset path.
///
/// The path may point to the dataset root or directly to `pdata/1`.
///
/// # Errors
///
/// Returns an error when required files are missing, malformed, or unsupported.
pub fn read_bruker_processed_2d_dir(path: impl AsRef<Path>) -> Result<Spectrum2D> {
    let input_path = path.as_ref();
    let processed_dir = locate_processed_2d_dir(input_path);
    let direct_parameters =
        parse_parameter_file(&read_text(&processed_dir.join("procs"), "Bruker procs")?);
    let indirect_parameters =
        parse_parameter_file(&read_text(&processed_dir.join("proc2s"), "Bruker proc2s")?);
    let acqus = read_acqus(input_path, &processed_dir)?;
    let title = read_title(&processed_dir)?;

    let x_count = required_usize(&direct_parameters, "SI")?;
    let y_count = required_usize(&indirect_parameters, "SI")?;
    let point_count = x_count
        .checked_mul(y_count)
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "Bruker processed 2D matrix size is too large".to_owned(),
        })?;

    let z = read_processed_i32_data(&processed_dir.join("2rr"), point_count, &direct_parameters)?;
    let imaginary = read_optional_processed_plane(&processed_dir, point_count, &direct_parameters)?;
    let x = build_axis(&direct_parameters, x_count)?;
    let y = build_axis(&indirect_parameters, y_count)?;
    let metadata = build_metadata(&direct_parameters, acqus.as_ref(), title)?;

    Spectrum2D::new_complex(x, y, z, imaginary, metadata)
}

fn locate_processed_2d_dir(path: &Path) -> PathBuf {
    if path.join("procs").is_file() && path.join("proc2s").is_file() && path.join("2rr").is_file() {
        path.to_path_buf()
    } else if path.is_file() {
        path.parent().map_or_else(PathBuf::new, Path::to_path_buf)
    } else {
        path.join("pdata").join("1")
    }
}

fn read_optional_processed_plane(
    processed_dir: &Path,
    point_count: usize,
    procs: &std::collections::BTreeMap<String, String>,
) -> Result<Option<Vec<f64>>> {
    let path = processed_dir.join("2ri");
    if path.is_file() {
        read_processed_i32_data(&path, point_count, procs).map(Some)
    } else {
        Ok(None)
    }
}
