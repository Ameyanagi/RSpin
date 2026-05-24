use rspin_analysis::{
    AssignedAtom, Assignment, AssignmentSet, AssignmentTarget, CouplingNode, JCoupling,
    JCouplingGraph,
};
use rspin_core::{Nucleus, RSpinError};

use crate::{SpectrumReader, SpectrumWriter};

use super::*;

#[test]
fn round_trips_assignment_set_json() -> anyhow::Result<()> {
    let assignments = assignment_fixture()?;
    let text = write_assignment_set_json(&assignments)?;
    let parsed = read_assignment_set_json(&text)?;

    assert!(text.contains(&format!("\"format\":\"{ASSIGNMENT_SET_JSON_FORMAT}\"")));
    assert!(text.contains(&format!("\"version\":{ASSIGNMENT_JSON_VERSION}")));
    assert!(text.contains("\"assignment_set\""));
    assert_eq!(parsed, assignments);
    Ok(())
}

#[test]
fn reads_legacy_raw_assignment_set_json() -> anyhow::Result<()> {
    let assignments = assignment_fixture()?;
    let text = serde_json::to_string(&assignments)?;
    let parsed = read_assignment_set_json(&text)?;

    assert_eq!(parsed, assignments);
    Ok(())
}

#[test]
fn round_trips_j_coupling_graph_json() -> anyhow::Result<()> {
    let graph = graph_fixture()?;
    let text = write_j_coupling_graph_json(&graph)?;
    let parsed = read_j_coupling_graph_json(&text)?;

    assert!(text.contains(&format!("\"format\":\"{J_COUPLING_GRAPH_JSON_FORMAT}\"")));
    assert!(text.contains(&format!("\"version\":{ASSIGNMENT_JSON_VERSION}")));
    assert!(text.contains("\"graph\""));
    assert_eq!(parsed, graph);
    Ok(())
}

#[test]
fn reads_legacy_raw_j_coupling_graph_json() -> anyhow::Result<()> {
    let graph = graph_fixture()?;
    let text = serde_json::to_string(&graph)?;
    let parsed = read_j_coupling_graph_json(&text)?;

    assert_eq!(parsed, graph);
    Ok(())
}

#[test]
fn rejects_wrong_assignment_json_headers() {
    let wrong_format = read_assignment_set_json(
        r#"{"format":"rspin.j_coupling_graph","version":1,"assignment_set":{"assignments":[]}}"#,
    )
    .expect_err("wrong assignment JSON format should fail");
    assert!(matches!(wrong_format, RSpinError::Parse { .. }));

    let unsupported_version = read_j_coupling_graph_json(
        r#"{"format":"rspin.j_coupling_graph","version":2,"graph":{"nodes":[],"couplings":[]}}"#,
    )
    .expect_err("unsupported assignment JSON version should fail");
    assert!(matches!(
        unsupported_version,
        RSpinError::Unsupported {
            feature: "assignment JSON version"
        }
    ));
}

#[test]
fn assignment_json_codecs_implement_traits() -> anyhow::Result<()> {
    let assignments = assignment_fixture()?;
    let assignments_text = <JsonAssignmentSet as SpectrumWriter<AssignmentSet>>::write_string(
        &JsonAssignmentSet,
        &assignments,
    )?;
    let parsed_assignments: AssignmentSet =
        SpectrumReader::read_str(&JsonAssignmentSet, &assignments_text)?;

    assert_eq!(format!("{JsonAssignmentSet:?}"), "JsonAssignmentSet");
    assert_eq!(parsed_assignments, assignments);

    let graph = graph_fixture()?;
    let graph_text = <JsonJCouplingGraph as SpectrumWriter<JCouplingGraph>>::write_string(
        &JsonJCouplingGraph,
        &graph,
    )?;
    let parsed_graph: JCouplingGraph = SpectrumReader::read_str(&JsonJCouplingGraph, &graph_text)?;

    assert_eq!(format!("{JsonJCouplingGraph:?}"), "JsonJCouplingGraph");
    assert_eq!(parsed_graph, graph);
    Ok(())
}

fn assignment_fixture() -> anyhow::Result<AssignmentSet> {
    Ok(AssignmentSet::new(vec![
        Assignment::deterministic(
            AssignmentTarget::Peak1D { index: 2, x: 7.12 },
            vec![AssignedAtom::new("H2", Nucleus::Hydrogen1)],
        )?
        .with_confidence(0.9)?,
    ])?)
}

fn graph_fixture() -> anyhow::Result<JCouplingGraph> {
    Ok(JCouplingGraph::new(
        vec![
            CouplingNode::new("H1", Nucleus::Hydrogen1),
            CouplingNode::new("H2", Nucleus::Hydrogen1),
        ],
        vec![JCoupling::deterministic("H1", "H2", 7.2)?.with_confidence(0.8)?],
    )?)
}
