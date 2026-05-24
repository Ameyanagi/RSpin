//! Bruker processed two-dimensional spectrum import.

use std::path::{Path, PathBuf};

use rspin_core::{RSpinError, Result, Spectrum2D};

use crate::SpectrumPathReader;

use super::{
    build_axis, build_metadata, decode_processed_i32_data, first_non_empty_line,
    parse_parameter_file_for_reader, read_acqus, read_processed_i32_data, read_text, read_title,
    required_usize,
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

/// Byte-oriented reader for Bruker processed two-dimensional spectra.
///
/// The required inputs are direct `procs`, indirect `proc2s`, and real `2rr`
/// bytes. Optional `acqus`, imaginary `2ri`, and title text can be attached
/// with chainable builder methods.
#[derive(Clone, Copy, Debug)]
pub struct BrukerProcessed2DBytes<'a> {
    direct_parameters: &'a str,
    indirect_parameters: &'a str,
    real_bytes: &'a [u8],
    acqus: Option<&'a str>,
    imaginary_bytes: Option<&'a [u8]>,
    title: Option<&'a str>,
}

impl<'a> BrukerProcessed2DBytes<'a> {
    /// Creates a byte-oriented Bruker processed 2D reader.
    #[must_use]
    pub fn new(
        direct_parameters: &'a str,
        indirect_parameters: &'a str,
        real_bytes: &'a [u8],
    ) -> Self {
        Self {
            direct_parameters,
            indirect_parameters,
            real_bytes,
            acqus: None,
            imaginary_bytes: None,
            title: None,
        }
    }

    /// Attaches optional direct-dimension `acqus` metadata text.
    #[must_use]
    pub fn with_acqus(mut self, acqus: &'a str) -> Self {
        self.acqus = Some(acqus);
        self
    }

    /// Attaches optional imaginary `2ri` bytes.
    #[must_use]
    pub fn with_imaginary(mut self, imaginary_bytes: &'a [u8]) -> Self {
        self.imaginary_bytes = Some(imaginary_bytes);
        self
    }

    /// Attaches optional Bruker title text.
    #[must_use]
    pub fn with_title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    /// Reads the supplied bytes into a spectrum.
    ///
    /// # Errors
    ///
    /// Returns an error when metadata or binary data are missing, malformed, or
    /// unsupported.
    pub fn read(self) -> Result<Spectrum2D> {
        let direct_parameters = parse_parameter_file_for_reader(self.direct_parameters)?;
        let indirect_parameters = parse_parameter_file_for_reader(self.indirect_parameters)?;
        let acqus = match self.acqus {
            Some(input) => Some(parse_parameter_file_for_reader(input)?),
            None => None,
        };
        let title = self.title.and_then(first_non_empty_line);
        build_processed_2d_spectrum(
            &direct_parameters,
            &indirect_parameters,
            self.real_bytes,
            self.imaginary_bytes,
            acqus.as_ref(),
            title,
        )
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
        parse_parameter_file_for_reader(&read_text(&processed_dir.join("procs"), "Bruker procs")?)?;
    let indirect_parameters = parse_parameter_file_for_reader(&read_text(
        &processed_dir.join("proc2s"),
        "Bruker proc2s",
    )?)?;
    let acqus = read_acqus(input_path, &processed_dir)?;
    let title = read_title(&processed_dir)?;

    let x_count = required_usize(&direct_parameters, "SI")?;
    let y_count = required_usize(&indirect_parameters, "SI")?;
    let point_count = processed_2d_point_count(x_count, y_count)?;

    let z = read_processed_i32_data(&processed_dir.join("2rr"), point_count, &direct_parameters)?;
    let imaginary = read_optional_processed_plane(&processed_dir, point_count, &direct_parameters)?;
    let x = build_axis(&direct_parameters, x_count)?;
    let y = build_axis(&indirect_parameters, y_count)?;
    let metadata = build_metadata(&direct_parameters, acqus.as_ref(), title)?;

    Spectrum2D::new_complex(x, y, z, imaginary, metadata)
}

/// Reads processed two-dimensional Bruker `2rr` bytes with `procs`/`proc2s`
/// metadata.
///
/// For optional `acqus`, `2ri`, or title metadata, use
/// [`BrukerProcessed2DBytes`].
///
/// # Errors
///
/// Returns an error when metadata or binary data are malformed or unsupported.
pub fn read_bruker_processed_2d_bytes(
    direct_parameters: &str,
    indirect_parameters: &str,
    real_bytes: &[u8],
) -> Result<Spectrum2D> {
    BrukerProcessed2DBytes::new(direct_parameters, indirect_parameters, real_bytes).read()
}

fn build_processed_2d_spectrum(
    direct_parameters: &std::collections::BTreeMap<String, String>,
    indirect_parameters: &std::collections::BTreeMap<String, String>,
    real_bytes: &[u8],
    imaginary_bytes: Option<&[u8]>,
    acqus: Option<&std::collections::BTreeMap<String, String>>,
    title: Option<String>,
) -> Result<Spectrum2D> {
    let x_count = required_usize(direct_parameters, "SI")?;
    let y_count = required_usize(indirect_parameters, "SI")?;
    let point_count = processed_2d_point_count(x_count, y_count)?;
    let z = decode_processed_i32_data(real_bytes, point_count, direct_parameters, "2rr")?;
    let imaginary = match imaginary_bytes {
        Some(bytes) => Some(decode_processed_i32_data(
            bytes,
            point_count,
            direct_parameters,
            "2ri",
        )?),
        None => None,
    };
    let x = build_axis(direct_parameters, x_count)?;
    let y = build_axis(indirect_parameters, y_count)?;
    let metadata = build_metadata(direct_parameters, acqus, title)?;

    Spectrum2D::new_complex(x, y, z, imaginary, metadata)
}

fn processed_2d_point_count(x_count: usize, y_count: usize) -> Result<usize> {
    x_count
        .checked_mul(y_count)
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "Bruker processed 2D matrix size is too large".to_owned(),
        })
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
