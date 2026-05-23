//! JSON spectrum serialization.

use rspin_core::{RSpinError, Result, Spectrum1D, Spectrum2D};

use crate::{SpectrumReader, SpectrumWriter};

/// JSON reader/writer for one-dimensional spectra.
#[derive(Clone, Copy, Debug, Default)]
pub struct JsonSpectrum1D;

impl SpectrumReader for JsonSpectrum1D {
    type Output = Spectrum1D;

    fn read_str(&self, input: &str) -> Result<Self::Output> {
        read_spectrum1d_json(input)
    }
}

impl SpectrumWriter<Spectrum1D> for JsonSpectrum1D {
    fn write_string(&self, spectrum: &Spectrum1D) -> Result<String> {
        write_spectrum1d_json(spectrum)
    }
}

/// JSON reader/writer for two-dimensional spectra.
#[derive(Clone, Copy, Debug, Default)]
pub struct JsonSpectrum2D;

impl SpectrumReader for JsonSpectrum2D {
    type Output = Spectrum2D;

    fn read_str(&self, input: &str) -> Result<Self::Output> {
        read_spectrum2d_json(input)
    }
}

impl SpectrumWriter<Spectrum2D> for JsonSpectrum2D {
    fn write_string(&self, spectrum: &Spectrum2D) -> Result<String> {
        write_spectrum2d_json(spectrum)
    }
}

/// Reads a one-dimensional spectrum from JSON.
///
/// # Errors
///
/// Returns an error when JSON deserialization fails.
pub fn read_spectrum1d_json(input: &str) -> Result<Spectrum1D> {
    serde_json::from_str(input).map_err(|error| json_error(&error))
}

/// Writes a one-dimensional spectrum to compact JSON.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn write_spectrum1d_json(spectrum: &Spectrum1D) -> Result<String> {
    serde_json::to_string(spectrum).map_err(|error| json_error(&error))
}

/// Reads a two-dimensional spectrum from JSON.
///
/// # Errors
///
/// Returns an error when JSON deserialization fails.
pub fn read_spectrum2d_json(input: &str) -> Result<Spectrum2D> {
    serde_json::from_str(input).map_err(|error| json_error(&error))
}

/// Writes a two-dimensional spectrum to compact JSON.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn write_spectrum2d_json(spectrum: &Spectrum2D) -> Result<String> {
    serde_json::to_string(spectrum).map_err(|error| json_error(&error))
}

fn json_error(error: &serde_json::Error) -> RSpinError {
    RSpinError::Parse {
        format: "JSON",
        message: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use rspin_core::{Axis, Metadata, Unit};

    use super::*;

    #[test]
    fn round_trips_1d_spectrum() -> anyhow::Result<()> {
        let spectrum = Spectrum1D::new(
            Axis::linear("shift", Unit::Ppm, 0.0, 2.0, 3)?,
            vec![1.0, 2.0, 3.0],
            Metadata::named("one"),
        )?;
        let text = write_spectrum1d_json(&spectrum)?;
        let parsed = read_spectrum1d_json(&text)?;
        assert_eq!(parsed, spectrum);
        Ok(())
    }

    #[test]
    fn round_trips_2d_spectrum_with_trait_api() -> anyhow::Result<()> {
        let spectrum = Spectrum2D::new(
            Axis::linear("x", Unit::Ppm, 0.0, 1.0, 2)?,
            Axis::linear("y", Unit::Ppm, 10.0, 11.0, 2)?,
            vec![1.0, 2.0, 3.0, 4.0],
            Metadata::named("two"),
        )?;
        let codec = JsonSpectrum2D;
        let text = codec.write_string(&spectrum)?;
        let parsed = codec.read_str(&text)?;
        assert_eq!(parsed, spectrum);
        Ok(())
    }

    #[test]
    fn rejects_invalid_json() {
        let error = read_spectrum1d_json("{").expect_err("invalid JSON should fail");
        assert!(matches!(error, RSpinError::Parse { .. }));
    }
}
