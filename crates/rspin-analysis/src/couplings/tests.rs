use super::*;

#[test]
fn builds_valid_coupling_graph() -> anyhow::Result<()> {
    let nodes = vec![
        CouplingNode::new("H1", Nucleus::Hydrogen1).with_label("H-1"),
        CouplingNode::new("H2", Nucleus::Hydrogen1).with_label("H-2"),
        CouplingNode::new("C1", Nucleus::Carbon13),
    ];
    let couplings = vec![
        JCoupling::deterministic("H1", "H2", 7.2)?.with_confidence(0.95)?,
        JCoupling::new("j:H1-C1", "H1", "C1", 142.0).with_source("measured"),
    ];

    let graph = JCouplingGraph::new(nodes, couplings)?;

    assert!(!graph.is_empty());
    assert_eq!(graph.couplings[0].id, "j:H1-H2");
    assert_eq!(graph.couplings_for_node("H1").len(), 2);
    assert_eq!(graph.couplings_for_node("H2").len(), 1);
    Ok(())
}

#[test]
fn deterministic_ids_order_endpoint_tokens() -> anyhow::Result<()> {
    let forward = deterministic_j_coupling_id("H 2", "H1")?;
    let reverse = deterministic_j_coupling_id("H1", "H 2")?;

    assert_eq!(forward, "j:H_2-H1");
    assert_eq!(forward, reverse);
    Ok(())
}

#[test]
fn rejects_invalid_couplings() {
    let nodes = vec![
        CouplingNode::new("H1", Nucleus::Hydrogen1),
        CouplingNode::new("H2", Nucleus::Hydrogen1),
    ];

    let error = JCouplingGraph::new(
        nodes.clone(),
        vec![JCoupling::new("j:H1-H3", "H1", "H3", 7.0)],
    )
    .expect_err("missing endpoint should fail");
    assert!(matches!(error, RSpinError::InvalidAssignment { .. }));

    let error = JCouplingGraph::new(
        nodes.clone(),
        vec![
            JCoupling::new("j:H1-H2", "H1", "H2", 7.0),
            JCoupling::new("j:H2-H1", "H2", "H1", 7.1),
        ],
    )
    .expect_err("duplicate endpoint pair should fail");
    assert!(matches!(error, RSpinError::InvalidAssignment { .. }));

    let error = JCouplingGraph::new(nodes, vec![JCoupling::new("j:H1-H2", "H1", "H2", f64::NAN)])
        .expect_err("non-finite coupling should fail");
    assert!(matches!(error, RSpinError::NonFinite { .. }));
}

#[test]
fn rejects_duplicate_node_ids_and_invalid_confidence() {
    let nodes = vec![
        CouplingNode::new("H1", Nucleus::Hydrogen1),
        CouplingNode::new("H1", Nucleus::Carbon13),
    ];
    let error = JCouplingGraph::new(nodes, Vec::new()).expect_err("duplicate node id should fail");
    assert!(matches!(error, RSpinError::InvalidAssignment { .. }));

    let error = JCoupling::new("j:H1-H2", "H1", "H2", 7.0)
        .with_confidence(1.5)
        .expect_err("confidence outside range should fail");
    assert!(matches!(error, RSpinError::InvalidAssignment { .. }));
}
