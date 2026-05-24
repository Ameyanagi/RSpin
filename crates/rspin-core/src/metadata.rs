//! Metadata carried by spectra and derived values.

use serde::{Deserialize, Serialize};

use crate::Nucleus;

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_methods_set_and_clear_metadata_fields() {
        let metadata = Metadata::new()
            .with_name("demo")
            .with_nucleus(Nucleus::Hydrogen1)
            .with_frequency_mhz(400.0)
            .with_solvent("CDCl3")
            .with_temperature_k(298.0)
            .with_origin("fixture");

        assert_eq!(metadata.name.as_deref(), Some("demo"));
        assert_eq!(metadata.nucleus, Some(Nucleus::Hydrogen1));
        assert_eq!(metadata.frequency_mhz, Some(400.0));
        assert_eq!(metadata.solvent.as_deref(), Some("CDCl3"));
        assert_eq!(metadata.temperature_k, Some(298.0));
        assert_eq!(metadata.origin.as_deref(), Some("fixture"));

        let cleared = metadata
            .without_name()
            .without_nucleus()
            .without_frequency_mhz()
            .without_solvent()
            .without_temperature_k()
            .without_origin();

        assert_eq!(cleared, Metadata::default());
    }
}
