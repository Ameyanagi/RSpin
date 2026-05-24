//! Conversion helpers from `NMReDATA` records into analysis data models.

use std::collections::BTreeSet;

use rspin_analysis::{
    AssignedAtom, Assignment, AssignmentSet, AssignmentTarget, CouplingNode, JCoupling,
    JCouplingGraph,
};
use rspin_core::{Nucleus, Result};
use serde::{Deserialize, Serialize};

use super::NmreDataRecord;

/// Analysis models derived from one parsed `NMReDATA` record.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NmreDataAnalysis {
    /// Converted one-dimensional assignment set.
    pub assignment_set: AssignmentSet,
    /// Converted J-coupling graph.
    pub j_coupling_graph: JCouplingGraph,
}

impl NmreDataRecord {
    /// Converts parsed assignments and scalar couplings into analysis models.
    ///
    /// # Errors
    ///
    /// Returns an error when either generated analysis payload is invalid.
    pub fn to_analysis(&self, nucleus: impl Into<Nucleus>) -> Result<NmreDataAnalysis> {
        nmredata_to_analysis(self, nucleus)
    }

    /// Converts parsed `NMReDATA` chemical-shift assignments to an [`AssignmentSet`].
    ///
    /// Each `NMReDATA` assignment becomes a one-dimensional peak target at the
    /// assignment shift. Atom references are used as assigned atom ids; when a
    /// source assignment has no atom references, its assignment label is used as
    /// a resonance id.
    ///
    /// # Errors
    ///
    /// Returns an error when the generated assignment payload is invalid.
    pub fn to_assignment_set(&self, nucleus: impl Into<Nucleus>) -> Result<AssignmentSet> {
        nmredata_assignments_to_assignment_set(self, nucleus)
    }

    /// Converts parsed one-dimensional signal labels to an [`AssignmentSet`].
    ///
    /// Signal `L=` attributes are used as assigned atom ids. If a signal has no
    /// `L=` attribute, positional items on that signal are used instead. Scalar
    /// signal positions become peak targets; ranged signals become range targets
    /// using stable synthetic signal indices.
    ///
    /// # Errors
    ///
    /// Returns an error when the generated assignment payload is invalid.
    pub fn to_signal_assignment_set(&self, nucleus: impl Into<Nucleus>) -> Result<AssignmentSet> {
        nmredata_1d_signals_to_assignment_set(self, nucleus)
    }

    /// Converts parsed two-dimensional signal labels to an [`AssignmentSet`].
    ///
    /// Each 2D signal becomes a synthetic [`AssignmentTarget::Zone2D`] target
    /// whose id is derived from the source signal order and labels. For standard
    /// 2D tags, the left side of the signal pair is assigned the direct-dimension
    /// nucleus and the right side is assigned the indirect-dimension nucleus.
    ///
    /// # Errors
    ///
    /// Returns an error when the generated assignment payload is invalid.
    pub fn to_2d_signal_assignment_set(&self) -> Result<AssignmentSet> {
        nmredata_2d_signals_to_assignment_set(self)
    }

    /// Converts parsed `NMReDATA` scalar couplings to a [`JCouplingGraph`].
    ///
    /// Assignment labels become graph nodes. Coupling endpoints that are not
    /// present in the assignment list are still emitted as nodes so partial
    /// records can be represented.
    ///
    /// # Errors
    ///
    /// Returns an error when the generated coupling graph is invalid.
    pub fn to_j_coupling_graph(&self, nucleus: impl Into<Nucleus>) -> Result<JCouplingGraph> {
        nmredata_couplings_to_j_coupling_graph(self, nucleus)
    }
}

/// Converts parsed assignments and scalar couplings into analysis models.
///
/// # Errors
///
/// Returns an error when either generated analysis payload is invalid.
pub fn nmredata_to_analysis(
    record: &NmreDataRecord,
    nucleus: impl Into<Nucleus>,
) -> Result<NmreDataAnalysis> {
    let nucleus = nucleus.into();
    Ok(NmreDataAnalysis {
        assignment_set: nmredata_assignments_to_assignment_set(record, nucleus.clone())?,
        j_coupling_graph: nmredata_couplings_to_j_coupling_graph(record, nucleus)?,
    })
}

/// Converts parsed `NMReDATA` chemical-shift assignments to an [`AssignmentSet`].
///
/// # Errors
///
/// Returns an error when the generated assignment payload is invalid.
pub fn nmredata_assignments_to_assignment_set(
    record: &NmreDataRecord,
    nucleus: impl Into<Nucleus>,
) -> Result<AssignmentSet> {
    let nucleus = nucleus.into();
    let mut assignments = Vec::with_capacity(record.assignments.len());
    for (index, source) in record.assignments.iter().enumerate() {
        let atoms = assigned_atoms_for_source(source, nucleus.clone());
        let assignment = Assignment::deterministic(
            AssignmentTarget::Peak1D {
                index,
                x: source.shift_ppm,
            },
            atoms,
        )?;
        assignments.push(assignment);
    }
    AssignmentSet::new(assignments)
}

/// Converts parsed one-dimensional signal labels to an [`AssignmentSet`].
///
/// # Errors
///
/// Returns an error when the generated assignment payload is invalid.
pub fn nmredata_1d_signals_to_assignment_set(
    record: &NmreDataRecord,
    nucleus: impl Into<Nucleus>,
) -> Result<AssignmentSet> {
    let nucleus = nucleus.into();
    let mut assignments = Vec::new();
    let mut signal_index = 0;

    for spectrum in &record.spectra {
        if !matches!(spectrum.kind, super::NmreDataSpectrumKind::OneD { .. }) {
            continue;
        }
        for signal in &spectrum.signals_1d {
            let atoms = assigned_atoms_for_signal(signal, &nucleus);
            if !atoms.is_empty() {
                let target = match signal.to_ppm {
                    Some(to) => AssignmentTarget::Range1D {
                        start_index: signal_index,
                        end_index: signal_index,
                        from: signal.from_ppm,
                        to,
                    },
                    None => AssignmentTarget::Peak1D {
                        index: signal_index,
                        x: signal.from_ppm,
                    },
                };
                assignments.push(Assignment::deterministic(target, atoms)?);
            }
            signal_index += 1;
        }
    }

    AssignmentSet::new(assignments)
}

/// Converts parsed two-dimensional signal labels to an [`AssignmentSet`].
///
/// # Errors
///
/// Returns an error when the generated assignment payload is invalid.
pub fn nmredata_2d_signals_to_assignment_set(record: &NmreDataRecord) -> Result<AssignmentSet> {
    let mut assignments = Vec::new();
    let mut signal_index = 0;

    for spectrum in &record.spectra {
        let super::NmreDataSpectrumKind::TwoD {
            indirect_label,
            indirect_nucleus,
            direct_label,
            direct_nucleus,
            ..
        } = &spectrum.kind
        else {
            continue;
        };
        let left_nucleus = resolved_nucleus(direct_nucleus.as_ref(), direct_label);
        let right_nucleus = resolved_nucleus(indirect_nucleus.as_ref(), indirect_label);
        for signal in &spectrum.signals_2d {
            let atoms = assigned_atoms_for_signal_2d(signal, &left_nucleus, &right_nucleus);
            let target = AssignmentTarget::Zone2D {
                id: nmredata_2d_signal_zone_id(signal_index, signal),
            };
            assignments.push(Assignment::deterministic(target, atoms)?);
            signal_index += 1;
        }
    }

    AssignmentSet::new(assignments)
}

/// Returns the stable synthetic zone id used for a parsed `NMReDATA` 2D signal.
#[must_use]
pub fn nmredata_2d_signal_zone_id(signal_index: usize, signal: &super::NmreDataSignal2D) -> String {
    format!(
        "nmredata:2d-signal:{signal_index}:{}:{}",
        zone_id_token(&signal.left),
        zone_id_token(&signal.right)
    )
}

/// Converts parsed `NMReDATA` scalar couplings to a [`JCouplingGraph`].
///
/// # Errors
///
/// Returns an error when the generated coupling graph is invalid.
pub fn nmredata_couplings_to_j_coupling_graph(
    record: &NmreDataRecord,
    nucleus: impl Into<Nucleus>,
) -> Result<JCouplingGraph> {
    let nucleus = nucleus.into();
    let mut node_ids = BTreeSet::new();
    let mut nodes = Vec::new();

    for assignment in &record.assignments {
        push_node(
            &mut nodes,
            &mut node_ids,
            &assignment.label,
            nucleus.clone(),
        );
    }
    for coupling in &record.couplings {
        push_node(
            &mut nodes,
            &mut node_ids,
            &coupling.from_label,
            nucleus.clone(),
        );
        push_node(
            &mut nodes,
            &mut node_ids,
            &coupling.to_label,
            nucleus.clone(),
        );
    }

    let mut couplings = Vec::with_capacity(record.couplings.len());
    for coupling in &record.couplings {
        couplings.push(
            JCoupling::deterministic(
                coupling.from_label.clone(),
                coupling.to_label.clone(),
                coupling.j_hz,
            )?
            .with_source("NMReDATA"),
        );
    }

    JCouplingGraph::new(nodes, couplings)
}

fn assigned_atoms_for_source(
    source: &super::NmreDataAssignment,
    nucleus: Nucleus,
) -> Vec<AssignedAtom> {
    if source.atom_refs.is_empty() {
        return vec![AssignedAtom::new(source.label.clone(), nucleus).with_label(&source.label)];
    }

    source
        .atom_refs
        .iter()
        .map(|atom_ref| AssignedAtom::new(atom_ref.clone(), nucleus.clone()).with_label(atom_ref))
        .collect()
}

fn assigned_atoms_for_signal(
    signal: &super::NmreDataSignal1D,
    nucleus: &Nucleus,
) -> Vec<AssignedAtom> {
    let labels = match signal
        .attributes
        .get("L")
        .filter(|labels| !labels.is_empty())
    {
        Some(labels) => labels.as_slice(),
        None => signal.items.as_slice(),
    };

    labels
        .iter()
        .filter(|label| !label.trim().is_empty())
        .map(|label| AssignedAtom::new(label.clone(), nucleus.clone()).with_label(label))
        .collect()
}

fn assigned_atoms_for_signal_2d(
    signal: &super::NmreDataSignal2D,
    left_nucleus: &Nucleus,
    right_nucleus: &Nucleus,
) -> Vec<AssignedAtom> {
    let mut seen = BTreeSet::new();
    let mut atoms = Vec::with_capacity(2);
    push_signal_atom(&mut atoms, &mut seen, &signal.left, left_nucleus);
    push_signal_atom(&mut atoms, &mut seen, &signal.right, right_nucleus);
    atoms
}

fn push_signal_atom(
    atoms: &mut Vec<AssignedAtom>,
    seen: &mut BTreeSet<String>,
    label: &str,
    nucleus: &Nucleus,
) {
    if seen.insert(label.to_owned()) {
        atoms.push(AssignedAtom::new(label.to_owned(), nucleus.clone()).with_label(label));
    }
}

fn resolved_nucleus(parsed: Option<&Nucleus>, fallback_label: &str) -> Nucleus {
    match parsed {
        Some(nucleus) => nucleus.clone(),
        None => Nucleus::Other(fallback_label.to_owned()),
    }
}

fn zone_id_token(value: &str) -> String {
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

fn push_node(
    nodes: &mut Vec<CouplingNode>,
    node_ids: &mut BTreeSet<String>,
    id: &str,
    nucleus: Nucleus,
) {
    if node_ids.insert(id.to_owned()) {
        nodes.push(CouplingNode::new(id.to_owned(), nucleus).with_label(id));
    }
}
