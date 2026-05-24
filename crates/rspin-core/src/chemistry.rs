//! Molecule and sample chemistry data structures.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{RSpinError, Result};

/// Atom data stored with sample metadata.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Atom {
    /// Stable atom identifier within its molecule.
    pub id: String,
    /// Element symbol, such as `C`, `H`, or `Cl`.
    pub element: String,
    /// Optional isotope mass number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub isotope: Option<u16>,
    /// Optional human-readable atom label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Optional x coordinate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x: Option<f64>,
    /// Optional y coordinate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub y: Option<f64>,
    /// Optional z coordinate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub z: Option<f64>,
}

impl Atom {
    /// Creates an atom with an identifier and element symbol.
    #[must_use]
    pub fn new(id: impl Into<String>, element: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            element: element.into(),
            isotope: None,
            label: None,
            x: None,
            y: None,
            z: None,
        }
    }

    /// Sets the isotope mass number.
    #[must_use]
    pub fn with_isotope(mut self, isotope: u16) -> Self {
        self.isotope = Some(isotope);
        self
    }

    /// Clears the isotope mass number.
    #[must_use]
    pub fn without_isotope(mut self) -> Self {
        self.isotope = None;
        self
    }

    /// Sets a human-readable atom label.
    #[must_use]
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Clears the atom label.
    #[must_use]
    pub fn without_label(mut self) -> Self {
        self.label = None;
        self
    }

    /// Sets a two-dimensional atom position.
    #[must_use]
    pub fn with_position_2d(mut self, x: f64, y: f64) -> Self {
        self.x = Some(x);
        self.y = Some(y);
        self.z = None;
        self
    }

    /// Sets a three-dimensional atom position.
    #[must_use]
    pub fn with_position_3d(mut self, x: f64, y: f64, z: f64) -> Self {
        self.x = Some(x);
        self.y = Some(y);
        self.z = Some(z);
        self
    }

    /// Clears all atom coordinates.
    #[must_use]
    pub fn without_position(mut self) -> Self {
        self.x = None;
        self.y = None;
        self.z = None;
        self
    }

    /// Validates atom identifiers, element data, isotope data, and coordinates.
    ///
    /// # Errors
    ///
    /// Returns an error when identifiers or symbols are empty, isotope zero is
    /// used, or coordinates are not finite.
    pub fn validate(&self) -> Result<()> {
        ensure_non_empty("atom id", &self.id)?;
        ensure_non_empty("atom element", &self.element)?;
        if self.isotope == Some(0) {
            return invalid_metadata("atom isotope must be positive");
        }
        validate_optional_coordinate("atom x", self.x)?;
        validate_optional_coordinate("atom y", self.y)?;
        validate_optional_coordinate("atom z", self.z)
    }
}

/// Chemical bond order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BondOrder {
    /// Single covalent bond.
    Single,
    /// Double covalent bond.
    Double,
    /// Triple covalent bond.
    Triple,
    /// Aromatic bond.
    Aromatic,
    /// Caller-defined bond order label.
    Other(String),
}

impl BondOrder {
    fn validate(&self) -> Result<()> {
        match self {
            Self::Other(label) => ensure_non_empty("bond order", label),
            Self::Single | Self::Double | Self::Triple | Self::Aromatic => Ok(()),
        }
    }
}

/// Bond between two atoms in the same molecule.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bond {
    /// Stable identifier of the first atom.
    pub from_atom_id: String,
    /// Stable identifier of the second atom.
    pub to_atom_id: String,
    /// Bond order.
    pub order: BondOrder,
}

impl Bond {
    /// Creates a single bond between two atom identifiers.
    #[must_use]
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from_atom_id: from.into(),
            to_atom_id: to.into(),
            order: BondOrder::Single,
        }
    }

    /// Sets the bond order.
    #[must_use]
    pub fn with_order(mut self, order: BondOrder) -> Self {
        self.order = order;
        self
    }

    /// Validates endpoints and bond order.
    ///
    /// # Errors
    ///
    /// Returns an error when either endpoint is empty, both endpoints are the
    /// same atom, or the custom bond order label is empty.
    pub fn validate(&self) -> Result<()> {
        ensure_non_empty("bond from atom", &self.from_atom_id)?;
        ensure_non_empty("bond to atom", &self.to_atom_id)?;
        if self.from_atom_id == self.to_atom_id {
            return invalid_metadata("bond endpoints must reference different atoms");
        }
        self.order.validate()
    }
}

/// Molecule stored with sample metadata.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Molecule {
    /// Stable molecule identifier.
    pub id: String,
    /// Optional human-readable molecule name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Optional molecular formula.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub formula: Option<String>,
    /// Atoms in stable caller-provided order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub atoms: Vec<Atom>,
    /// Bonds in stable caller-provided order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bonds: Vec<Bond>,
}

impl Molecule {
    /// Creates an empty molecule with a stable identifier.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: None,
            formula: None,
            atoms: Vec::new(),
            bonds: Vec::new(),
        }
    }

    /// Sets a human-readable molecule name.
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Clears the molecule name.
    #[must_use]
    pub fn without_name(mut self) -> Self {
        self.name = None;
        self
    }

    /// Sets the molecular formula.
    #[must_use]
    pub fn with_formula(mut self, formula: impl Into<String>) -> Self {
        self.formula = Some(formula.into());
        self
    }

    /// Clears the molecular formula.
    #[must_use]
    pub fn without_formula(mut self) -> Self {
        self.formula = None;
        self
    }

    /// Appends an atom.
    #[must_use]
    pub fn with_atom(mut self, atom: Atom) -> Self {
        self.atoms.push(atom);
        self
    }

    /// Replaces all atoms.
    #[must_use]
    pub fn with_atoms(mut self, atoms: Vec<Atom>) -> Self {
        self.atoms = atoms;
        self
    }

    /// Clears all atoms and bonds.
    #[must_use]
    pub fn without_atoms(mut self) -> Self {
        self.atoms.clear();
        self.bonds.clear();
        self
    }

    /// Appends a bond.
    #[must_use]
    pub fn with_bond(mut self, bond: Bond) -> Self {
        self.bonds.push(bond);
        self
    }

    /// Replaces all bonds.
    #[must_use]
    pub fn with_bonds(mut self, bonds: Vec<Bond>) -> Self {
        self.bonds = bonds;
        self
    }

    /// Clears all bonds.
    #[must_use]
    pub fn without_bonds(mut self) -> Self {
        self.bonds.clear();
        self
    }

    /// Finds an atom by stable identifier.
    #[must_use]
    pub fn atom(&self, id: &str) -> Option<&Atom> {
        self.atoms.iter().find(|atom| atom.id == id)
    }

    /// Validates molecule identifiers, atoms, duplicate IDs, and bond endpoints.
    ///
    /// # Errors
    ///
    /// Returns an error when molecule data is incomplete or internally
    /// inconsistent.
    pub fn validate(&self) -> Result<()> {
        ensure_non_empty("molecule id", &self.id)?;
        let mut atom_ids = BTreeSet::new();
        for atom in &self.atoms {
            atom.validate()?;
            if !atom_ids.insert(atom.id.as_str()) {
                return invalid_metadata(format!("duplicate atom id {}", atom.id));
            }
        }
        for bond in &self.bonds {
            bond.validate()?;
            if !atom_ids.contains(bond.from_atom_id.as_str()) {
                return invalid_metadata(format!(
                    "bond references unknown atom {}",
                    bond.from_atom_id
                ));
            }
            if !atom_ids.contains(bond.to_atom_id.as_str()) {
                return invalid_metadata(format!(
                    "bond references unknown atom {}",
                    bond.to_atom_id
                ));
            }
        }
        Ok(())
    }
}

fn validate_optional_coordinate(field: &'static str, value: Option<f64>) -> Result<()> {
    if value.is_none_or(f64::is_finite) {
        return Ok(());
    }
    Err(RSpinError::NonFinite { field })
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
mod tests;
