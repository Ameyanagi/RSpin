//! Metadata carried by spectra and derived values.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::Nucleus;
use crate::{Molecule, RSpinError, Result};

/// High-level NMR experiment classification.
///
/// Derived heuristically from vendor pulse-program names when available. The
/// [`Self::Other`] variant carries the raw source token (for example the
/// Bruker `PULPROG` string) so an unrecognized experiment still round-trips.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ExperimentKind {
    /// A generic, non-edited 1D experiment (for example a simple `zg`).
    Generic1D,
    /// Heteronuclear single-quantum coherence.
    Hsqc,
    /// Heteronuclear multiple-bond correlation.
    Hmbc,
    /// Correlation spectroscopy.
    Cosy,
    /// Total correlation spectroscopy.
    Tocsy,
    /// Nuclear Overhauser effect spectroscopy.
    Noesy,
    /// Distortionless enhancement by polarization transfer.
    Dept,
    /// Attached proton test.
    Apt,
    /// Any experiment not covered by a dedicated variant; carries the raw
    /// source token (for example the pulse-program name).
    Other(String),
}

/// Indirect-dimension quadrature detection mode.
///
/// Mirrors the Bruker `FNMODE` acquisition parameter and determines how raw
/// hypercomplex 2D data is assembled and transformed along the indirect
/// dimension.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuadMode {
    /// Undefined / no quadrature detection in the indirect dimension.
    None,
    /// Single-channel (magnitude) quadrature, Bruker `FNMODE = 1`.
    Qf,
    /// Sequential (Redfield) quadrature, Bruker `FNMODE = 2`.
    Qseq,
    /// Time-proportional phase incrementation, Bruker `FNMODE = 3`.
    Tppi,
    /// States (hypercomplex) quadrature, Bruker `FNMODE = 4`.
    States,
    /// States-TPPI quadrature, Bruker `FNMODE = 5`.
    StatesTppi,
    /// Echo / anti-echo (sensitivity-enhanced) quadrature, Bruker `FNMODE = 6`.
    EchoAntiecho,
}

impl QuadMode {
    /// Maps a Bruker `FNMODE` integer to a quadrature mode.
    ///
    /// Returns `None` for values outside the documented `0..=6` range.
    #[must_use]
    pub fn from_fnmode(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::Qf),
            2 => Some(Self::Qseq),
            3 => Some(Self::Tppi),
            4 => Some(Self::States),
            5 => Some(Self::StatesTppi),
            6 => Some(Self::EchoAntiecho),
            _ => None,
        }
    }
}

/// Descriptive metadata for a spectrum.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Metadata {
    /// Human-readable experiment or spectrum name.
    pub name: Option<String>,
    /// Primary observed nucleus.
    pub nucleus: Option<Nucleus>,
    /// Spectrometer frequency in MHz for the primary nucleus.
    pub frequency_mhz: Option<f64>,
    /// Spectrometer frequency in MHz for the indirect dimension of a 2D
    /// experiment. `None` means the indirect dimension shares
    /// [`Self::frequency_mhz`] (homonuclear experiments such as COSY or
    /// TOCSY). Heteronuclear datasets (HSQC, HMBC, etc.) set this so the
    /// indirect axis can be relabeled to ppm with the right carrier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub indirect_frequency_mhz: Option<f64>,
    /// High-level experiment classification, when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub experiment: Option<ExperimentKind>,
    /// Indirect-dimension quadrature detection mode, when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quad_mode: Option<QuadMode>,
    /// Editing pulse angle in degrees for multiplicity-edited experiments such
    /// as DEPT (for example `135.0`), when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dept_angle_deg: Option<f64>,
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

    /// Sets the indirect-dimension spectrometer frequency in MHz for
    /// heteronuclear 2D experiments.
    #[must_use]
    pub fn with_indirect_frequency_mhz(mut self, frequency_mhz: f64) -> Self {
        self.indirect_frequency_mhz = Some(frequency_mhz);
        self
    }

    /// Clears the indirect-dimension spectrometer frequency.
    #[must_use]
    pub fn without_indirect_frequency_mhz(mut self) -> Self {
        self.indirect_frequency_mhz = None;
        self
    }

    /// Sets the high-level experiment classification.
    #[must_use]
    pub fn with_experiment(mut self, experiment: ExperimentKind) -> Self {
        self.experiment = Some(experiment);
        self
    }

    /// Clears the high-level experiment classification.
    #[must_use]
    pub fn without_experiment(mut self) -> Self {
        self.experiment = None;
        self
    }

    /// Sets the indirect-dimension quadrature detection mode.
    #[must_use]
    pub fn with_quad_mode(mut self, quad_mode: QuadMode) -> Self {
        self.quad_mode = Some(quad_mode);
        self
    }

    /// Clears the indirect-dimension quadrature detection mode.
    #[must_use]
    pub fn without_quad_mode(mut self) -> Self {
        self.quad_mode = None;
        self
    }

    /// Sets the multiplicity-editing pulse angle in degrees.
    #[must_use]
    pub fn with_dept_angle_deg(mut self, dept_angle_deg: f64) -> Self {
        self.dept_angle_deg = Some(dept_angle_deg);
        self
    }

    /// Clears the multiplicity-editing pulse angle.
    #[must_use]
    pub fn without_dept_angle_deg(mut self) -> Self {
        self.dept_angle_deg = None;
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
        if self.dept_angle_deg.is_some_and(|angle| !angle.is_finite()) {
            return Err(RSpinError::NonFinite {
                field: "metadata.dept_angle_deg",
            });
        }
        if matches!(&self.experiment, Some(ExperimentKind::Other(token)) if token.trim().is_empty())
        {
            return Err(RSpinError::InvalidMetadata {
                message: "experiment kind token must not be empty".to_owned(),
            });
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
            .with_indirect_frequency_mhz(100.0)
            .with_experiment(ExperimentKind::Hsqc)
            .with_quad_mode(QuadMode::States)
            .with_dept_angle_deg(135.0)
            .with_solvent("CDCl3")
            .with_temperature_k(298.0)
            .with_origin("fixture")
            .with_molecule(Molecule::new("sample").with_name("Sample"))
            .with_property("vendor.field", "value");

        assert_eq!(metadata.name.as_deref(), Some("demo"));
        assert_eq!(metadata.nucleus, Some(Nucleus::Hydrogen1));
        assert_eq!(metadata.frequency_mhz, Some(400.0));
        assert_eq!(metadata.indirect_frequency_mhz, Some(100.0));
        assert_eq!(metadata.experiment, Some(ExperimentKind::Hsqc));
        assert_eq!(metadata.quad_mode, Some(QuadMode::States));
        assert_eq!(metadata.dept_angle_deg, Some(135.0));
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
            .without_indirect_frequency_mhz()
            .without_experiment()
            .without_quad_mode()
            .without_dept_angle_deg()
            .without_solvent()
            .without_temperature_k()
            .without_origin()
            .without_molecules()
            .without_properties();

        assert_eq!(cleared, Metadata::default());
        Ok(())
    }

    #[test]
    fn quad_mode_maps_bruker_fnmode_values() {
        assert_eq!(QuadMode::from_fnmode(0), Some(QuadMode::None));
        assert_eq!(QuadMode::from_fnmode(4), Some(QuadMode::States));
        assert_eq!(QuadMode::from_fnmode(5), Some(QuadMode::StatesTppi));
        assert_eq!(QuadMode::from_fnmode(6), Some(QuadMode::EchoAntiecho));
        assert_eq!(QuadMode::from_fnmode(7), None);
        assert_eq!(QuadMode::from_fnmode(-1), None);
    }

    #[test]
    fn experiment_and_quad_mode_round_trip_through_json() -> Result<()> {
        let metadata = Metadata::new()
            .with_experiment(ExperimentKind::Dept)
            .with_quad_mode(QuadMode::EchoAntiecho)
            .with_dept_angle_deg(90.0);
        let json = serde_json::to_string(&metadata).map_err(|error| RSpinError::Parse {
            format: "metadata-json",
            message: error.to_string(),
        })?;
        let restored: Metadata =
            serde_json::from_str(&json).map_err(|error| RSpinError::Parse {
                format: "metadata-json",
                message: error.to_string(),
            })?;
        assert_eq!(restored, metadata);
        // Snake-case tags are stable for downstream consumers.
        assert!(json.contains("\"dept\""));
        assert!(json.contains("\"echo_antiecho\""));
        Ok(())
    }

    #[test]
    fn default_metadata_omits_new_optional_fields_in_json() -> Result<()> {
        let json =
            serde_json::to_string(&Metadata::default()).map_err(|error| RSpinError::Parse {
                format: "metadata-json",
                message: error.to_string(),
            })?;
        assert!(!json.contains("experiment"));
        assert!(!json.contains("quad_mode"));
        assert!(!json.contains("dept_angle_deg"));
        Ok(())
    }

    #[test]
    fn rejects_non_finite_dept_angle() {
        let metadata = Metadata::new().with_dept_angle_deg(f64::NAN);
        assert!(matches!(
            metadata.validate(),
            Err(RSpinError::NonFinite { .. })
        ));
    }

    #[test]
    fn rejects_empty_experiment_other_token() {
        let metadata = Metadata::new().with_experiment(ExperimentKind::Other(" ".to_owned()));
        assert!(matches!(
            metadata.validate(),
            Err(RSpinError::InvalidMetadata { .. })
        ));
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
