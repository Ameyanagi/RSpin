//! Spectrum annotation data structures.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{RSpinError, Result};

/// Target referenced by a spectrum annotation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "target", rename_all = "snake_case")]
pub enum AnnotationTarget {
    /// A one-dimensional point by point index and coordinate.
    Point1D {
        /// Point index.
        index: usize,
        /// Axis coordinate.
        x: f64,
    },
    /// A one-dimensional axis range.
    Range1D {
        /// Range start coordinate.
        from: f64,
        /// Range end coordinate.
        to: f64,
    },
    /// A two-dimensional point by indices and coordinates.
    Point2D {
        /// X point index.
        x_index: usize,
        /// Y point index.
        y_index: usize,
        /// X coordinate.
        x: f64,
        /// Y coordinate.
        y: f64,
    },
    /// A two-dimensional rectangular zone.
    Zone2D {
        /// X range start coordinate.
        x_from: f64,
        /// X range end coordinate.
        x_to: f64,
        /// Y range start coordinate.
        y_from: f64,
        /// Y range end coordinate.
        y_to: f64,
    },
    /// A two-dimensional detected zone by stable identifier.
    #[serde(rename = "zone_2d_id")]
    Zone2DId {
        /// Zone identifier.
        id: String,
    },
    /// An atom stored on a metadata molecule.
    MoleculeAtom {
        /// Molecule identifier.
        molecule_id: String,
        /// Atom identifier.
        atom_id: String,
    },
    /// A bond stored on a metadata molecule.
    MoleculeBond {
        /// Molecule identifier.
        molecule_id: String,
        /// First atom identifier.
        from_atom_id: String,
        /// Second atom identifier.
        to_atom_id: String,
    },
}

impl AnnotationTarget {
    /// Creates a one-dimensional point target.
    #[must_use]
    pub fn point_1d(index: usize, x: f64) -> Self {
        Self::Point1D { index, x }
    }

    /// Creates a one-dimensional range target.
    #[must_use]
    pub fn range_1d(from: f64, to: f64) -> Self {
        Self::Range1D { from, to }
    }

    /// Creates a two-dimensional point target.
    #[must_use]
    pub fn point_2d(x_index: usize, y_index: usize, x: f64, y: f64) -> Self {
        Self::Point2D {
            x_index,
            y_index,
            x,
            y,
        }
    }

    /// Creates a two-dimensional rectangular zone target.
    #[must_use]
    pub fn zone_2d(x_from: f64, x_to: f64, y_from: f64, y_to: f64) -> Self {
        Self::Zone2D {
            x_from,
            x_to,
            y_from,
            y_to,
        }
    }

    /// Creates a two-dimensional zone identifier target.
    #[must_use]
    pub fn zone_2d_id(id: impl Into<String>) -> Self {
        Self::Zone2DId { id: id.into() }
    }

    /// Creates a molecule atom target.
    #[must_use]
    pub fn molecule_atom(molecule_id: impl Into<String>, atom_id: impl Into<String>) -> Self {
        Self::MoleculeAtom {
            molecule_id: molecule_id.into(),
            atom_id: atom_id.into(),
        }
    }

    /// Creates a molecule bond target.
    #[must_use]
    pub fn molecule_bond(
        molecule_id: impl Into<String>,
        from_atom_id: impl Into<String>,
        to_atom_id: impl Into<String>,
    ) -> Self {
        Self::MoleculeBond {
            molecule_id: molecule_id.into(),
            from_atom_id: from_atom_id.into(),
            to_atom_id: to_atom_id.into(),
        }
    }

    /// Validates target coordinates and referenced identifiers.
    ///
    /// # Errors
    ///
    /// Returns an error when coordinates are not finite or referenced molecule
    /// or atom identifiers are empty.
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Point1D { x, .. } => ensure_finite("annotation x", *x),
            Self::Range1D { from, to } => {
                ensure_finite("annotation range start", *from)?;
                ensure_finite("annotation range end", *to)
            }
            Self::Point2D { x, y, .. } => {
                ensure_finite("annotation x", *x)?;
                ensure_finite("annotation y", *y)
            }
            Self::Zone2D {
                x_from,
                x_to,
                y_from,
                y_to,
            } => {
                ensure_finite("annotation x range start", *x_from)?;
                ensure_finite("annotation x range end", *x_to)?;
                ensure_finite("annotation y range start", *y_from)?;
                ensure_finite("annotation y range end", *y_to)
            }
            Self::Zone2DId { id } => ensure_non_empty("annotation zone id", id),
            Self::MoleculeAtom {
                molecule_id,
                atom_id,
            } => {
                ensure_non_empty("annotation molecule id", molecule_id)?;
                ensure_non_empty("annotation atom id", atom_id)
            }
            Self::MoleculeBond {
                molecule_id,
                from_atom_id,
                to_atom_id,
            } => {
                ensure_non_empty("annotation molecule id", molecule_id)?;
                ensure_non_empty("annotation bond from atom", from_atom_id)?;
                ensure_non_empty("annotation bond to atom", to_atom_id)?;
                if from_atom_id == to_atom_id {
                    return invalid_metadata("annotation bond endpoints must differ");
                }
                Ok(())
            }
        }
    }
}

/// User or algorithm annotation attached to a spectrum.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpectrumAnnotation {
    /// Stable annotation identifier.
    pub id: String,
    /// Optional human-readable label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Target referenced by the annotation.
    pub target: AnnotationTarget,
}

impl SpectrumAnnotation {
    /// Creates an annotation with a stable identifier and target.
    #[must_use]
    pub fn new(id: impl Into<String>, target: AnnotationTarget) -> Self {
        Self {
            id: id.into(),
            label: None,
            target,
        }
    }

    /// Sets the annotation label.
    #[must_use]
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Clears the annotation label.
    #[must_use]
    pub fn without_label(mut self) -> Self {
        self.label = None;
        self
    }

    /// Validates the annotation identifier and target.
    ///
    /// # Errors
    ///
    /// Returns an error when the annotation identifier is empty or the target is
    /// invalid.
    pub fn validate(&self) -> Result<()> {
        ensure_non_empty("annotation id", &self.id)?;
        self.target.validate()
    }
}

pub(crate) fn validate_annotation_collection(annotations: &[SpectrumAnnotation]) -> Result<()> {
    let mut ids = BTreeSet::new();
    for annotation in annotations {
        annotation.validate()?;
        if !ids.insert(annotation.id.as_str()) {
            return invalid_metadata(format!("duplicate annotation id {}", annotation.id));
        }
    }
    Ok(())
}

fn ensure_finite(field: &'static str, value: f64) -> Result<()> {
    if !value.is_finite() {
        return Err(RSpinError::NonFinite { field });
    }
    Ok(())
}

fn ensure_non_empty(field: &'static str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return invalid_metadata(format!("{field} must not be empty"));
    }
    Ok(())
}

fn invalid_metadata(message: impl Into<String>) -> Result<()> {
    Err(RSpinError::InvalidMetadata {
        message: message.into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_and_validates_annotation_targets() -> Result<()> {
        let point = SpectrumAnnotation::new("peak-1", AnnotationTarget::point_1d(3, 7.25))
            .with_label("peak");
        let atom =
            SpectrumAnnotation::new("atom-1", AnnotationTarget::molecule_atom("sample", "H1"));
        let zone = SpectrumAnnotation::new("zone-1", AnnotationTarget::zone_2d_id("z1"));

        point.validate()?;
        atom.validate()?;
        zone.validate()?;
        assert_eq!(point.label.as_deref(), Some("peak"));
        Ok(())
    }

    #[test]
    fn rejects_invalid_annotation_data() {
        let bad_id = SpectrumAnnotation::new("", AnnotationTarget::point_1d(0, 1.0));
        let bad_coordinate =
            SpectrumAnnotation::new("a", AnnotationTarget::range_1d(0.0, f64::NAN));
        let bad_target = SpectrumAnnotation::new("a", AnnotationTarget::molecule_atom("", "H1"));
        let duplicate = [
            SpectrumAnnotation::new("a", AnnotationTarget::point_1d(0, 1.0)),
            SpectrumAnnotation::new("a", AnnotationTarget::point_1d(1, 2.0)),
        ];

        assert!(matches!(
            bad_id.validate(),
            Err(RSpinError::InvalidMetadata { .. })
        ));
        assert!(matches!(
            bad_coordinate.validate(),
            Err(RSpinError::NonFinite { .. })
        ));
        assert!(matches!(
            bad_target.validate(),
            Err(RSpinError::InvalidMetadata { .. })
        ));
        assert!(matches!(
            validate_annotation_collection(&duplicate),
            Err(RSpinError::InvalidMetadata { .. })
        ));
    }
}
