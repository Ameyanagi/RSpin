use rspin_core::{Nucleus, RSpinError};

use super::*;

#[test]
fn creates_targets_from_detected_features_and_chains_set_building() -> anyhow::Result<()> {
    let peak = crate::Peak {
        index: 2,
        x: 7.12,
        intensity: 3.0,
        prominence: 1.0,
        polarity: crate::PeakPolarity::Positive,
    };
    let range = crate::DetectedRange {
        start_index: 4,
        end_index: 7,
        from: 3.2,
        to: 2.8,
        active_points: 3,
        max_abs_intensity: 12.0,
        area: 8.0,
    };
    let zone = crate::DetectedZone {
        id: "zone:x1-2:y3-4".to_owned(),
        x_start_index: 1,
        x_end_index: 2,
        y_start_index: 3,
        y_end_index: 4,
        x_from: 1.0,
        x_to: 2.0,
        y_from: 3.0,
        y_to: 4.0,
        centroid_x: 1.5,
        centroid_y: 3.5,
        active_points: 4,
        max_abs_intensity: 10.0,
        sum_intensity: 20.0,
        sum_abs_intensity: 20.0,
    };

    let peak_target = AssignmentTarget::peak_1d(&peak);
    let range_target = AssignmentTarget::range_1d(&range);
    let zone_target = AssignmentTarget::zone_2d(&zone);
    let set = AssignmentSet::default()
        .with_deterministic_assignment(
            peak_target.clone(),
            vec![AssignedAtom::new("H2", Nucleus::Hydrogen1)],
        )?
        .with_deterministic_assignment(
            range_target.clone(),
            vec![AssignedAtom::new("H4", Nucleus::Hydrogen1)],
        )?
        .with_assignment(Assignment::deterministic(
            zone_target.clone(),
            vec![AssignedAtom::new("C1", Nucleus::Carbon13)],
        )?)?;

    assert_eq!(set.len(), 3);
    assert_eq!(set.assignments[0].id, "assign:peak1d:2:H2");
    assert_eq!(set.assignments[1].id, "assign:range1d:4-7:H4");
    assert_eq!(set.assignments[2].id, "assign:zone2d:zone_x1-2_y3-4:C1");
    assert_eq!(set.for_target(&zone_target).len(), 1);
    Ok(())
}

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
