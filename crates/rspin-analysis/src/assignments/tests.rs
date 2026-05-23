use rspin_core::{Nucleus, RSpinError};

use super::*;

#[test]
fn creates_deterministic_assignment_ids() -> anyhow::Result<()> {
    let target = AssignmentTarget::Peak1D { index: 4, x: 7.12 };
    let atoms = vec![
        AssignedAtom::new("H-4", Nucleus::Hydrogen1).with_label("H4"),
        AssignedAtom::new("H 5", Nucleus::Hydrogen1),
    ];

    let assignment = Assignment::deterministic(target, atoms)?.with_confidence(0.8)?;

    assert_eq!(assignment.id, "assign:peak1d:4:H-4+H_5");
    assert_eq!(assignment.confidence, Some(0.8));
    assignment.validate()?;
    Ok(())
}

#[test]
fn validates_assignment_sets_and_target_lookup() -> anyhow::Result<()> {
    let target = AssignmentTarget::Range1D {
        start_index: 2,
        end_index: 5,
        from: 7.4,
        to: 7.1,
    };
    let assignment = Assignment::deterministic(
        target.clone(),
        vec![AssignedAtom::new("H7", Nucleus::Hydrogen1)],
    )?;
    let mut set = AssignmentSet::new(vec![assignment])?;
    set.push(Assignment::deterministic(
        AssignmentTarget::Zone2D {
            id: "zone:x1-2:y0-1".to_owned(),
        },
        vec![AssignedAtom::new("C2", Nucleus::Carbon13)],
    )?)?;

    assert_eq!(set.len(), 2);
    assert_eq!(set.for_target(&target).len(), 1);
    assert!(!set.is_empty());
    Ok(())
}

#[test]
fn rejects_duplicate_assignment_ids() {
    let first = Assignment::new(
        "a1",
        AssignmentTarget::Peak1D { index: 1, x: 1.0 },
        vec![AssignedAtom::new("H1", Nucleus::Hydrogen1)],
    );
    let second = Assignment::new(
        "a1",
        AssignmentTarget::Peak1D { index: 2, x: 2.0 },
        vec![AssignedAtom::new("H2", Nucleus::Hydrogen1)],
    );

    let error =
        AssignmentSet::new(vec![first, second]).expect_err("duplicate assignment id should fail");

    assert!(matches!(error, RSpinError::InvalidAssignment { .. }));
}

#[test]
fn rejects_invalid_assignment_payloads() {
    let no_atoms = Assignment::new(
        "a1",
        AssignmentTarget::Peak1D { index: 1, x: 1.0 },
        Vec::new(),
    );
    assert!(matches!(
        no_atoms.validate(),
        Err(RSpinError::InvalidAssignment { .. })
    ));

    let invalid_range = Assignment::new(
        "a2",
        AssignmentTarget::Range1D {
            start_index: 5,
            end_index: 2,
            from: 1.0,
            to: 2.0,
        },
        vec![AssignedAtom::new("H2", Nucleus::Hydrogen1)],
    );
    assert!(matches!(
        invalid_range.validate(),
        Err(RSpinError::InvalidAssignment { .. })
    ));

    let invalid_confidence = Assignment::new(
        "a3",
        AssignmentTarget::Peak1D { index: 1, x: 1.0 },
        vec![AssignedAtom::new("H3", Nucleus::Hydrogen1)],
    )
    .with_confidence(1.5);
    assert!(matches!(
        invalid_confidence,
        Err(RSpinError::InvalidAssignment { .. })
    ));
}
