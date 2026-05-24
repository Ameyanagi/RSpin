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
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use rspin_core::{
        AnnotationTarget, Atom, Axis, Bond, Metadata, Molecule, SpectrumAnnotation, Unit,
    };

    use crate::{SpectrumPathReader, SpectrumPathWriter};

    use super::*;

    #[test]
    fn round_trips_1d_spectrum() -> anyhow::Result<()> {
        let spectrum = Spectrum1D::new(
            Axis::linear("shift", Unit::Ppm, 0.0, 2.0, 3)?,
            vec![1.0, 2.0, 3.0],
            Metadata::named("one").with_property("vendor.field", "value"),
        )?;
        let text = write_spectrum1d_json(&spectrum)?;
        let parsed = read_spectrum1d_json(&text)?;
        assert!(text.contains("properties"));
        assert_eq!(parsed, spectrum);
        Ok(())
    }

    #[test]
    fn round_trips_molecules_and_annotations() -> anyhow::Result<()> {
        let molecule = Molecule::new("sample")
            .with_atom(Atom::new("H1", "H"))
            .with_atom(Atom::new("C1", "C"))
            .with_bond(Bond::new("H1", "C1"));
        let spectrum = Spectrum1D::new(
            Axis::linear("shift", Unit::Ppm, 0.0, 2.0, 3)?,
            vec![1.0, 2.0, 3.0],
            Metadata::named("annotated").with_molecule(molecule),
        )?
        .with_annotation(SpectrumAnnotation::new(
            "peak-1",
            AnnotationTarget::molecule_atom("sample", "H1"),
        ));

        let text = write_spectrum1d_json(&spectrum)?;
        let parsed = read_spectrum1d_json(&text)?;

        assert!(text.contains("molecules"));
        assert!(text.contains("annotations"));
        parsed.metadata.validate_molecules()?;
        parsed.validate_annotations()?;
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
    fn writes_spectrum_path_with_trait_api() -> anyhow::Result<()> {
        let root = temp_dir("json-path-writer")?;
        let path = root.join("spectrum.json");
        let spectrum = Spectrum1D::new(
            Axis::linear("shift", Unit::Ppm, 0.0, 2.0, 3)?,
            vec![1.0, 2.0, 3.0],
            Metadata::named("path one"),
        )?;

        JsonSpectrum1D.write_path(&spectrum, &path)?;
        let parsed = JsonSpectrum1D.read_path(&path)?;

        assert_eq!(parsed, spectrum);
        remove_dir(root)?;
        Ok(())
    }

    #[test]
    fn round_trips_complex_2d_spectrum() -> anyhow::Result<()> {
        let spectrum = Spectrum2D::new_complex(
            Axis::linear("x", Unit::Ppm, 0.0, 1.0, 2)?,
            Axis::linear("y", Unit::Ppm, 10.0, 11.0, 2)?,
            vec![1.0, 2.0, 3.0, 4.0],
            Some(vec![0.1, 0.2, 0.3, 0.4]),
            Metadata::named("complex two"),
        )?;
        let text = write_spectrum2d_json(&spectrum)?;
        let parsed = read_spectrum2d_json(&text)?;
        assert_eq!(parsed, spectrum);
        Ok(())
    }

    #[test]
    fn rejects_invalid_json() {
        let error = read_spectrum1d_json("{").expect_err("invalid JSON should fail");
        assert!(matches!(error, RSpinError::Parse { .. }));
    }

    fn temp_dir(name: &str) -> anyhow::Result<PathBuf> {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let mut path = std::env::temp_dir();
        path.push(format!("rspin-json-{name}-{}-{nanos}", std::process::id()));
        fs::create_dir_all(&path)?;
        Ok(path)
    }

    fn remove_dir(path: PathBuf) -> anyhow::Result<()> {
        fs::remove_dir_all(path)?;
        Ok(())
    }
}
