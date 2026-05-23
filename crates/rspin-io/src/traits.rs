//! Shared IO traits.

use rspin_core::Result;

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

/// Writes a spectrum-like value to a string payload.
pub trait SpectrumWriter<S> {
    /// Writes a value to a string.
    ///
    /// # Errors
    ///
    /// Returns an error when the value cannot be represented by the writer.
    fn write_string(&self, spectrum: &S) -> Result<String>;
}
