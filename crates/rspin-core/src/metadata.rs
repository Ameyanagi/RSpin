//! Metadata carried by spectra and derived values.

use serde::{Deserialize, Serialize};

use crate::Nucleus;
use crate::{Molecule, RSpinError, Result};

/// Descriptive metadata for a spectrum.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Metadata {
    /// Human-readable experiment or spectrum name.
    pub name: Option<String>,
    /// Primary observed nucleus.
    pub nucleus: Option<Nucleus>,
    /// Spectrometer frequency in MHz for the primary nucleus.
    pub frequency_mhz: Option<f64>,
    /// Solvent name or code.
    pub solvent: Option<String>,
    /// Sample temperature in kelvin.
    pub temperature_k: Option<f64>,
    /// Free-form source label, path, or provenance identifier.
    pub origin: Option<String>,
    /// Sample molecules associated with this spectrum or dataset.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub molecules: Vec<Molecule>,
}

impl Metadata {
    /// Creates empty metadata.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates metadata with only a name.
    #[must_use]
    pub fn named(name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            ..Self::default()
        }
    }

    /// Sets the human-readable experiment or spectrum name.
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Clears the human-readable name.
    #[must_use]
    pub fn without_name(mut self) -> Self {
        self.name = None;
        self
    }

    /// Sets the primary observed nucleus.
    #[must_use]
    pub fn with_nucleus(mut self, nucleus: Nucleus) -> Self {
        self.nucleus = Some(nucleus);
        self
    }

    /// Clears the primary observed nucleus.
    #[must_use]
    pub fn without_nucleus(mut self) -> Self {
        self.nucleus = None;
        self
    }

    /// Sets the spectrometer frequency in MHz.
    #[must_use]
    pub fn with_frequency_mhz(mut self, frequency_mhz: f64) -> Self {
        self.frequency_mhz = Some(frequency_mhz);
        self
    }

    /// Clears the spectrometer frequency.
    #[must_use]
    pub fn without_frequency_mhz(mut self) -> Self {
        self.frequency_mhz = None;
        self
    }

    /// Sets the solvent name or code.
    #[must_use]
    pub fn with_solvent(mut self, solvent: impl Into<String>) -> Self {
        self.solvent = Some(solvent.into());
        self
    }

    /// Clears the solvent name or code.
    #[must_use]
    pub fn without_solvent(mut self) -> Self {
        self.solvent = None;
        self
    }

    /// Sets the sample temperature in kelvin.
    #[must_use]
    pub fn with_temperature_k(mut self, temperature_k: f64) -> Self {
        self.temperature_k = Some(temperature_k);
        self
    }

    /// Clears the sample temperature.
    #[must_use]
    pub fn without_temperature_k(mut self) -> Self {
        self.temperature_k = None;
        self
    }

    /// Sets the free-form source label, path, or provenance identifier.
    #[must_use]
    pub fn with_origin(mut self, origin: impl Into<String>) -> Self {
        self.origin = Some(origin.into());
        self
    }

    /// Clears the source label, path, or provenance identifier.
    #[must_use]
    pub fn without_origin(mut self) -> Self {
        self.origin = None;
        self
    }

    /// Appends a sample molecule.
    #[must_use]
    pub fn with_molecule(mut self, molecule: Molecule) -> Self {
        self.molecules.push(molecule);
        self
    }

    /// Replaces all sample molecules.
    #[must_use]
    pub fn with_molecules(mut self, molecules: Vec<Molecule>) -> Self {
        self.molecules = molecules;
        self
    }

    /// Clears all sample molecules.
    #[must_use]
    pub fn without_molecules(mut self) -> Self {
        self.molecules.clear();
        self
    }

    /// Finds a sample molecule by stable identifier.
    #[must_use]
    pub fn molecule(&self, id: &str) -> Option<&Molecule> {
        self.molecules.iter().find(|molecule| molecule.id == id)
    }

    /// Validates all sample molecules and checks for duplicate molecule IDs.
    ///
    /// # Errors
    ///
    /// Returns an error when any molecule is invalid or duplicate molecule IDs
    /// are present.
    pub fn validate_molecules(&self) -> Result<()> {
        let mut ids = std::collections::BTreeSet::new();
        for molecule in &self.molecules {
            molecule.validate()?;
            if !ids.insert(molecule.id.as_str()) {
                return Err(RSpinError::InvalidMetadata {
                    message: format!("duplicate molecule id {}", molecule.id),
                });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_methods_set_and_clear_metadata_fields() -> Result<()> {
        let metadata = Metadata::new()
            .with_name("demo")
            .with_nucleus(Nucleus::Hydrogen1)
            .with_frequency_mhz(400.0)
            .with_solvent("CDCl3")
            .with_temperature_k(298.0)
            .with_origin("fixture")
            .with_molecule(Molecule::new("sample").with_name("Sample"));

        assert_eq!(metadata.name.as_deref(), Some("demo"));
        assert_eq!(metadata.nucleus, Some(Nucleus::Hydrogen1));
        assert_eq!(metadata.frequency_mhz, Some(400.0));
        assert_eq!(metadata.solvent.as_deref(), Some("CDCl3"));
        assert_eq!(metadata.temperature_k, Some(298.0));
        assert_eq!(metadata.origin.as_deref(), Some("fixture"));
        assert_eq!(
            metadata
                .molecule("sample")
                .and_then(|molecule| molecule.name.as_deref()),
            Some("Sample")
        );
        metadata.validate_molecules()?;

        let cleared = metadata
            .without_name()
            .without_nucleus()
            .without_frequency_mhz()
            .without_solvent()
            .without_temperature_k()
            .without_origin()
            .without_molecules();

        assert_eq!(cleared, Metadata::default());
        Ok(())
    }

    #[test]
    fn rejects_duplicate_metadata_molecules() {
        let metadata = Metadata::new()
            .with_molecule(Molecule::new("sample"))
            .with_molecule(Molecule::new("sample"));

        assert!(matches!(
            metadata.validate_molecules(),
            Err(RSpinError::InvalidMetadata { .. })
        ));
    }
}
