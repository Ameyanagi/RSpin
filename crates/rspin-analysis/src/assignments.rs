//! Assignment storage for analyzed spectra.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use rspin_core::{Nucleus, RSpinError, Result};

use crate::{peaks::Peak, ranges::DetectedRange, zones::DetectedZone};

/// Atom or resonance label assigned to a detected feature.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssignedAtom {
    /// Stable atom or resonance identifier.
    pub id: String,
    /// Optional display label.
    pub label: Option<String>,
    /// Assigned nucleus.
    pub nucleus: Nucleus,
}

impl AssignedAtom {
    /// Creates an assigned atom with no display label.
    #[must_use]
    pub fn new(id: impl Into<String>, nucleus: Nucleus) -> Self {
        Self {
            id: id.into(),
            label: None,
            nucleus,
        }
    }

    /// Returns a copy with a display label.
    #[must_use]
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    fn validate(&self) -> Result<()> {
        validate_non_empty("atom id", &self.id)?;
        if let Some(label) = &self.label {
            validate_non_empty("atom label", label)?;
        }
        if let Nucleus::Other(label) = &self.nucleus {
            validate_non_empty("nucleus label", label)?;
        }
        Ok(())
    }

    fn stable_key(&self) -> String {
        sanitize_id_token(&self.id)
    }
}

/// Feature target for an assignment.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AssignmentTarget {
    /// One-dimensional peak by index and coordinate.
    Peak1D {
        /// Peak index in the source spectrum.
        index: usize,
        /// Peak coordinate.
        x: f64,
    },
    /// One-dimensional detected range.
    Range1D {
        /// First included point index.
        start_index: usize,
        /// Last included point index.
        end_index: usize,
        /// Range start coordinate.
        from: f64,
        /// Range end coordinate.
        to: f64,
    },
    /// Two-dimensional detected zone by stable zone id.
    Zone2D {
        /// Zone id.
        id: String,
    },
}

impl AssignmentTarget {
    /// Creates a one-dimensional peak assignment target from a detected peak.
    #[must_use]
    pub fn peak_1d(peak: &Peak) -> Self {
        Self::Peak1D {
            index: peak.index,
            x: peak.x,
        }
    }

    /// Creates a one-dimensional range assignment target from a detected range.
    #[must_use]
    pub fn range_1d(range: &DetectedRange) -> Self {
        Self::Range1D {
            start_index: range.start_index,
            end_index: range.end_index,
            from: range.from,
            to: range.to,
        }
    }

    /// Creates a two-dimensional zone assignment target from a detected zone.
    #[must_use]
    pub fn zone_2d(zone: &DetectedZone) -> Self {
        Self::Zone2D {
            id: zone.id.clone(),
        }
    }

    fn validate(&self) -> Result<()> {
        match self {
            Self::Peak1D { x, .. } => ensure_finite("peak coordinate", *x),
            Self::Range1D {
                start_index,
                end_index,
                from,
                to,
            } => {
                if start_index > end_index {
                    return Err(RSpinError::InvalidAssignment {
                        message: "range assignment start index must not exceed end index"
                            .to_owned(),
                    });
                }
                ensure_finite("range start", *from)?;
                ensure_finite("range end", *to)
            }
            Self::Zone2D { id } => validate_non_empty("zone id", id),
        }
    }

    fn stable_key(&self) -> String {
        match self {
            Self::Peak1D { index, .. } => format!("peak1d:{index}"),
            Self::Range1D {
                start_index,
                end_index,
                ..
            } => {
                format!("range1d:{start_index}-{end_index}")
            }
            Self::Zone2D { id } => format!("zone2d:{}", sanitize_id_token(id)),
        }
    }
}

/// Assignment between a detected feature and one or more atoms/resonances.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Assignment {
    /// Stable assignment id.
    pub id: String,
    /// Assigned feature.
    pub target: AssignmentTarget,
    /// Atoms or resonances assigned to the feature.
    pub atoms: Vec<AssignedAtom>,
    /// Optional confidence in `[0, 1]`.
    pub confidence: Option<f64>,
    /// Optional human note.
    pub note: Option<String>,
}

impl Assignment {
    /// Creates an assignment with a caller-provided id.
    #[must_use]
    pub fn new(id: impl Into<String>, target: AssignmentTarget, atoms: Vec<AssignedAtom>) -> Self {
        Self {
            id: id.into(),
            target,
            atoms,
            confidence: None,
            note: None,
        }
    }

    /// Creates an assignment with a deterministic id derived from target and atoms.
    ///
    /// # Errors
    ///
    /// Returns an error when target or atom data is invalid.
    pub fn deterministic(target: AssignmentTarget, atoms: Vec<AssignedAtom>) -> Result<Self> {
        let id = deterministic_assignment_id(&target, &atoms)?;
        let assignment = Self::new(id, target, atoms);
        assignment.validate()?;
        Ok(assignment)
    }

    /// Returns a copy with confidence metadata.
    ///
    /// # Errors
    ///
    /// Returns an error when confidence is not finite or outside `[0, 1]`.
    pub fn with_confidence(mut self, confidence: f64) -> Result<Self> {
        validate_confidence(confidence)?;
        self.confidence = Some(confidence);
        Ok(self)
    }

    /// Returns a copy with a note.
    #[must_use]
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    /// Validates assignment consistency.
    ///
    /// # Errors
    ///
    /// Returns an error when ids are empty, target data is invalid, atoms are
    /// missing, atom ids are duplicated, or confidence is invalid.
    pub fn validate(&self) -> Result<()> {
        validate_non_empty("assignment id", &self.id)?;
        self.target.validate()?;
        validate_atoms(&self.atoms)?;
        if let Some(confidence) = self.confidence {
            validate_confidence(confidence)?;
        }
        if let Some(note) = &self.note {
            validate_non_empty("assignment note", note)?;
        }
        Ok(())
    }
}

/// Collection of validated assignments.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AssignmentSet {
    /// Assignments in insertion order.
    pub assignments: Vec<Assignment>,
}

impl AssignmentSet {
    /// Creates a validated assignment set.
    ///
    /// # Errors
    ///
    /// Returns an error when any assignment is invalid or ids are duplicated.
    pub fn new(assignments: Vec<Assignment>) -> Result<Self> {
        validate_assignments(&assignments)?;
        Ok(Self { assignments })
    }

    /// Validates all assignments in the set.
    ///
    /// # Errors
    ///
    /// Returns an error when any assignment is invalid or ids are duplicated.
    pub fn validate(&self) -> Result<()> {
        validate_assignments(&self.assignments)
    }

    /// Adds one assignment after validation.
    ///
    /// # Errors
    ///
    /// Returns an error when the assignment is invalid or duplicates an id.
    pub fn push(&mut self, assignment: Assignment) -> Result<()> {
        assignment.validate()?;
        if self
            .assignments
            .iter()
            .any(|existing| existing.id == assignment.id)
        {
            return Err(RSpinError::InvalidAssignment {
                message: format!("duplicate assignment id '{}'", assignment.id),
            });
        }
        self.assignments.push(assignment);
        Ok(())
    }

    /// Returns a copy with one appended assignment.
    ///
    /// # Errors
    ///
    /// Returns an error when the assignment is invalid or duplicates an id.
    pub fn with_assignment(mut self, assignment: Assignment) -> Result<Self> {
        self.push(assignment)?;
        Ok(self)
    }

    /// Returns a copy with one deterministic assignment appended.
    ///
    /// # Errors
    ///
    /// Returns an error when target or atom data is invalid or the generated id
    /// duplicates an existing assignment id.
    pub fn with_deterministic_assignment(
        mut self,
        target: AssignmentTarget,
        atoms: Vec<AssignedAtom>,
    ) -> Result<Self> {
        self.push(Assignment::deterministic(target, atoms)?)?;
        Ok(self)
    }

    /// Returns assignments targeting the same feature.
    #[must_use]
    pub fn for_target(&self, target: &AssignmentTarget) -> Vec<&Assignment> {
        let target_key = target.stable_key();
        self.assignments
            .iter()
            .filter(|assignment| assignment.target.stable_key() == target_key)
            .collect()
    }

    /// Returns true when the collection has no assignments.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.assignments.is_empty()
    }

    /// Returns the number of assignments.
    #[must_use]
    pub fn len(&self) -> usize {
        self.assignments.len()
    }
}

/// Builds a deterministic assignment id from target and atom ids.
///
/// # Errors
///
/// Returns an error when target or atom data is invalid.
pub fn deterministic_assignment_id(
    target: &AssignmentTarget,
    atoms: &[AssignedAtom],
) -> Result<String> {
    target.validate()?;
    validate_atoms(atoms)?;
    let atom_key = atoms
        .iter()
        .map(AssignedAtom::stable_key)
        .collect::<Vec<_>>()
        .join("+");
    Ok(format!("assign:{}:{atom_key}", target.stable_key()))
}

fn validate_assignments(assignments: &[Assignment]) -> Result<()> {
    let mut ids = BTreeSet::new();
    for assignment in assignments {
        assignment.validate()?;
        if !ids.insert(assignment.id.as_str()) {
            return Err(RSpinError::InvalidAssignment {
                message: format!("duplicate assignment id '{}'", assignment.id),
            });
        }
    }
    Ok(())
}

fn validate_atoms(atoms: &[AssignedAtom]) -> Result<()> {
    if atoms.is_empty() {
        return Err(RSpinError::InvalidAssignment {
            message: "assignment requires at least one atom".to_owned(),
        });
    }

    let mut ids = BTreeSet::new();
    for atom in atoms {
        atom.validate()?;
        if !ids.insert(atom.id.as_str()) {
            return Err(RSpinError::InvalidAssignment {
                message: format!("duplicate assigned atom id '{}'", atom.id),
            });
        }
    }
    Ok(())
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(RSpinError::InvalidAssignment {
            message: format!("{field} must not be empty"),
        });
    }
    Ok(())
}

fn ensure_finite(field: &'static str, value: f64) -> Result<()> {
    if !value.is_finite() {
        return Err(RSpinError::NonFinite { field });
    }
    Ok(())
}

fn validate_confidence(confidence: f64) -> Result<()> {
    if !confidence.is_finite() || !(0.0..=1.0).contains(&confidence) {
        return Err(RSpinError::InvalidAssignment {
            message: "assignment confidence must be finite and between 0 and 1".to_owned(),
        });
    }
    Ok(())
}

fn sanitize_id_token(value: &str) -> String {
    value
        .trim()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.') {
                character
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests;
