//! Metadata carried by spectra and derived values.

use std::collections::BTreeMap;

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
    /// Additional stable metadata properties.
    ///
    /// Readers use these key/value pairs to preserve format-specific metadata
    /// that does not have a dedicated field yet. Keys should be namespaced by
    /// source, for example `bruker.acqus.SFO1` or `agilent.procpar.sfrq`.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub properties: BTreeMap<String, String>,
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

    /// Sets an additional metadata property.
    #[must_use]
    pub fn with_property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.properties.insert(key.into(), value.into());
        self
    }

    /// Replaces all additional metadata properties.
    #[must_use]
    pub fn with_properties(mut self, properties: BTreeMap<String, String>) -> Self {
        self.properties = properties;
        self
    }

    /// Removes an additional metadata property.
    #[must_use]
    pub fn without_property(mut self, key: &str) -> Self {
        self.properties.remove(key);
        self
    }

    /// Clears all additional metadata properties.
    #[must_use]
    pub fn without_properties(mut self) -> Self {
        self.properties.clear();
        self
    }

    /// Returns an additional metadata property by key.
    #[must_use]
    pub fn property(&self, key: &str) -> Option<&str> {
        self.properties.get(key).map(String::as_str)
    }

    /// Finds a sample molecule by stable identifier.
    #[must_use]
    pub fn molecule(&self, id: &str) -> Option<&Molecule> {
        self.molecules.iter().find(|molecule| molecule.id == id)
    }

    /// Validates molecules and additional metadata properties.
    ///
    /// # Errors
    ///
    /// Returns an error when a molecule is invalid, molecule IDs are duplicate,
    /// or a property key is empty.
    pub fn validate(&self) -> Result<()> {
        self.validate_molecules()?;
        for key in self.properties.keys() {
            if key.trim().is_empty() {
                return Err(RSpinError::InvalidMetadata {
                    message: "metadata property key must not be empty".to_owned(),
                });
            }
        }
        Ok(())
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
            .with_molecule(Molecule::new("sample").with_name("Sample"))
            .with_property("vendor.field", "value");

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
        assert_eq!(metadata.property("vendor.field"), Some("value"));
        metadata.validate()?;

        let cleared = metadata
            .without_name()
            .without_nucleus()
            .without_frequency_mhz()
            .without_solvent()
            .without_temperature_k()
            .without_origin()
            .without_molecules()
            .without_properties();

        assert_eq!(cleared, Metadata::default());
        Ok(())
    }

    #[test]
    fn rejects_duplicate_metadata_molecules() {
        let metadata = Metadata::new()
            .with_molecule(Molecule::new("sample"))
            .with_molecule(Molecule::new("sample"));

        assert!(matches!(
            metadata.validate(),
            Err(RSpinError::InvalidMetadata { .. })
        ));
    }

    #[test]
    fn rejects_empty_metadata_property_keys() {
        let metadata = Metadata::new().with_property(" ", "value");

        assert!(matches!(
            metadata.validate(),
            Err(RSpinError::InvalidMetadata { .. })
        ));
    }
}
