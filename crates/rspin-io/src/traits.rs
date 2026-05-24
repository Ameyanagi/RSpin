//! Shared IO traits.

use std::fs;
use std::path::Path;

use rspin_core::{RSpinError, Result};

/// Reads a spectrum-like value from a string payload.
pub trait SpectrumReader {
    /// Output produced by the reader.
    type Output;

    /// Reads a value from a string.
    ///
    /// # Errors
    ///
    /// Returns an error when the payload is malformed or unsupported.
    fn read_str(&self, input: &str) -> Result<Self::Output>;
}

/// Reads a spectrum-like value from a filesystem path.
pub trait SpectrumPathReader {
    /// Output produced by the reader.
    type Output;

    /// Reads a value from a path.
    ///
    /// # Errors
    ///
    /// Returns an error when the path is missing, malformed, or unsupported.
    fn read_path(&self, path: &Path) -> Result<Self::Output>;
}

impl<T> SpectrumPathReader for T
where
    T: SpectrumReader,
{
    type Output = T::Output;

    fn read_path(&self, path: &Path) -> Result<Self::Output> {
        let input = fs::read_to_string(path).map_err(|error| RSpinError::Parse {
            format: "spectrum path",
            message: format!("failed to read {}: {error}", path.display()),
        })?;
        self.read_str(&input)
    }
}

/// Writes a spectrum-like value to a string payload.
pub trait SpectrumWriter<S: ?Sized> {
    /// Writes a value to a string.
    ///
    /// # Errors
    ///
    /// Returns an error when the value cannot be represented by the writer.
    fn write_string(&self, spectrum: &S) -> Result<String>;
}

/// Writes a spectrum-like value to a filesystem path.
pub trait SpectrumPathWriter<S: ?Sized>: SpectrumWriter<S> {
    /// Writes a value to a path.
    ///
    /// # Errors
    ///
    /// Returns an error when the value cannot be serialized or the path cannot
    /// be written.
    fn write_path(&self, spectrum: &S, path: &Path) -> Result<()> {
        let payload = self.write_string(spectrum)?;
        fs::write(path, payload).map_err(|error| RSpinError::Parse {
            format: "spectrum path",
            message: format!("failed to write {}: {error}", path.display()),
        })
    }
}

impl<T, S: ?Sized> SpectrumPathWriter<S> for T where T: SpectrumWriter<S> {}
