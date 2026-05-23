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
    /// Creates metadata with only a name.
    #[must_use]
    pub fn named(name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            ..Self::default()
        }
    }
}
