use rspin_core::{Axis, Metadata, Nucleus, Unit};

use super::*;
use crate::{
    CouplingNode, JCoupling, MultipletKind, Peak, PeakPolarity, deterministic_assignment_id,
};

#[test]
fn summarizes_range_multiplet_assignments_and_couplings() -> anyhow::Result<()> {
    let spectrum = spectrum(&[0.0, 2.0, 0.0])?;
    let range = DetectedRange {
        start_index: 0,
        end_index: 2,
        from: 0.0,
        to: 2.0,
        active_points: 1,
        max_abs_intensity: 2.0,
        area: 2.0,
    };
    let multiplet = singlet(1, 1.0, 2.0);
    let range_assignment = Assignment::deterministic(
        AssignmentTarget::Range1D {
            start_index: 0,
            end_index: 2,
            from: 0.0,
            to: 2.0,
        },
        vec![AssignedAtom::new("H1", Nucleus::Hydrogen1)],
    )?;
    let peak_target = AssignmentTarget::Peak1D { index: 1, x: 1.0 };
    let peak_assignment = Assignment::new(
        deterministic_assignment_id(&peak_target, &[AssignedAtom::new("H2", Nucleus::Hydrogen1)])?,
        peak_target,
        vec![AssignedAtom::new("H2", Nucleus::Hydrogen1)],
    );
    let assignments = AssignmentSet::new(vec![range_assignment, peak_assignment])?;
    let coupling_graph = JCouplingGraph::new(
        vec![
            CouplingNode::new("H1", Nucleus::Hydrogen1),
            CouplingNode::new("H2", Nucleus::Hydrogen1),
        ],
        vec![JCoupling::deterministic("H1", "H2", 7.2)?],
    )?;

    let signals = summarize_signals_1d(
        &spectrum,
        &[range],
        &[multiplet],
        &assignments,
        &coupling_graph,
        SignalSummaryOptions::default(),
    )?;

    assert_eq!(signals.len(), 1);
    assert_eq!(signals[0].id, "signal1d:range:0-2");
    assert_eq!(signals[0].multiplet_kinds, vec![MultipletKind::Singlet]);
    assert_eq!(signals[0].peak_count, 1);
    assert_eq!(signals[0].assignments.len(), 2);
    assert_eq!(signals[0].atoms.len(), 2);
    assert_eq!(signals[0].couplings.len(), 1);
    assert_close(signals[0].center_ppm, 1.0);
    Ok(())
}

#[test]
fn emits_orphan_multiplets_and_can_suppress_empty_ranges() -> anyhow::Result<()> {
    let spectrum = spectrum(&[0.0, 0.0, 1.0, 0.0])?;
    let empty_range = DetectedRange {
        start_index: 0,
        end_index: 1,
        from: 0.0,
        to: 1.0,
        active_points: 1,
        max_abs_intensity: 0.0,
        area: 0.0,
    };
    let orphan = singlet(2, 2.0, 1.0);

    let signals = summarize_signals_1d(
        &spectrum,
        &[empty_range],
        &[orphan],
        &AssignmentSet::default(),
        &JCouplingGraph::default(),
        SignalSummaryOptions {
            include_empty_ranges: false,
            include_orphan_multiplets: true,
        },
    )?;

    assert_eq!(signals.len(), 1);
    assert_eq!(signals[0].id, "signal1d:multiplet1d:2-2");
    assert!(signals[0].range.is_none());
    assert_close(signals[0].from_ppm, 2.0);
    Ok(())
}

#[test]
fn can_suppress_orphan_multiplets() -> anyhow::Result<()> {
    let spectrum = spectrum(&[0.0, 1.0, 0.0])?;
    let signals = summarize_signals_1d(
        &spectrum,
        &[],
        &[singlet(1, 1.0, 1.0)],
        &AssignmentSet::default(),
        &JCouplingGraph::default(),
        SignalSummaryOptions {
            include_empty_ranges: true,
            include_orphan_multiplets: false,
        },
    )?;

    assert!(signals.is_empty());
    Ok(())
}

#[test]
fn rejects_invalid_range_and_multiplet_data() -> anyhow::Result<()> {
    let spectrum = spectrum(&[0.0, 1.0, 0.0])?;
    let invalid_range = DetectedRange {
        start_index: 0,
        end_index: 3,
        from: 0.0,
        to: 3.0,
        active_points: 1,
        max_abs_intensity: 1.0,
        area: 1.0,
    };
    let error = summarize_signals_1d(
        &spectrum,
        &[invalid_range],
        &[],
        &AssignmentSet::default(),
        &JCouplingGraph::default(),
        SignalSummaryOptions::default(),
    )
    .expect_err("out-of-range range should fail");
    assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));

    let mut invalid_multiplet = singlet(1, 1.0, 1.0);
    invalid_multiplet.id.clear();
    let error = summarize_signals_1d(
        &spectrum,
        &[],
        &[invalid_multiplet],
        &AssignmentSet::default(),
        &JCouplingGraph::default(),
        SignalSummaryOptions::default(),
    )
    .expect_err("empty multiplet id should fail");
    assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));
    Ok(())
}

fn spectrum(intensities: &[f64]) -> anyhow::Result<Spectrum1D> {
    let end = f64::from(u32::try_from(intensities.len() - 1)?);
    Ok(Spectrum1D::new(
        Axis::linear("x", Unit::Ppm, 0.0, end, intensities.len())?,
        intensities.to_vec(),
        Metadata::default(),
    )?)
}

fn singlet(index: usize, x: f64, intensity: f64) -> DetectedMultiplet {
    DetectedMultiplet {
        id: format!("multiplet1d:{index}-{index}"),
        kind: MultipletKind::Singlet,
        peaks: vec![Peak {
            index,
            x,
            intensity,
            prominence: intensity.abs(),
            polarity: PeakPolarity::Positive,
        }],
        center_ppm: x,
        from_ppm: x,
        to_ppm: x,
        total_abs_intensity: intensity.abs(),
        spacings_ppm: Vec::new(),
        estimated_j_hz: None,
    }
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1.0e-12,
        "{actual} != {expected}"
    );
}
