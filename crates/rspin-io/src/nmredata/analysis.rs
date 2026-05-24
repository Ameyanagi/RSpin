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
