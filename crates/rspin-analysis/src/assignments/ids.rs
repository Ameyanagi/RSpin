//! Deterministic assignment identifiers.

use rspin_core::Result;

use super::{AssignedAtom, AssignmentTarget, validate_atoms};

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

impl AssignedAtom {
    pub(super) fn stable_key(&self) -> String {
        sanitize_id_token(&self.id)
    }
}

impl AssignmentTarget {
    pub(super) fn stable_key(&self) -> String {
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
