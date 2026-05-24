use super::*;
use crate::Simulator;

#[test]
fn single_spin_transition_matches_shift() -> anyhow::Result<()> {
    let system = SpinHalfSystem {
        spins: vec![SpinHalf { shift_ppm: 2.0 }],
        couplings: Vec::new(),
    };
    let transitions = exact_spin_half_transitions(&system, &ExactSpinOptions::default())?;

    assert_eq!(transitions.len(), 1);
    assert_close(transitions[0].frequency_hz, 800.0, 1.0e-10);
    assert_close(transitions[0].offset_hz, 800.0, 1.0e-10);
    assert_close(transitions[0].center_ppm, 2.0, 1.0e-12);
    assert_close(transitions[0].intensity, 0.25, 1.0e-12);
    assert_eq!(transitions[0].contribution_count, 1);
    Ok(())
}

#[test]
fn single_spin_transition_preserves_negative_shift() -> anyhow::Result<()> {
    let system = SpinHalfSystem {
        spins: vec![SpinHalf { shift_ppm: -0.5 }],
        couplings: Vec::new(),
    };
    let transitions = exact_spin_half_transitions(&system, &ExactSpinOptions::default())?;

    assert_eq!(transitions.len(), 1);
    assert_close(transitions[0].frequency_hz, 200.0, 1.0e-10);
    assert_close(transitions[0].offset_hz, -200.0, 1.0e-10);
    assert_close(transitions[0].center_ppm, -0.5, 1.0e-12);
    Ok(())
}

#[test]
fn uncoupled_spins_merge_degenerate_state_transitions() -> anyhow::Result<()> {
    let system = SpinHalfSystem {
        spins: vec![SpinHalf { shift_ppm: 1.0 }, SpinHalf { shift_ppm: 2.0 }],
        couplings: Vec::new(),
    };
    let transitions = exact_spin_half_transitions(&system, &ExactSpinOptions::default())?;

    assert_eq!(transitions.len(), 2);
    assert_close(transitions[0].frequency_hz, 400.0, 1.0e-10);
    assert_close(transitions[0].offset_hz, 400.0, 1.0e-10);
    assert_close(transitions[0].center_ppm, 1.0, 1.0e-12);
    assert_close(transitions[0].intensity, 0.5, 1.0e-12);
    assert_eq!(transitions[0].contribution_count, 2);
    assert_close(transitions[1].frequency_hz, 800.0, 1.0e-10);
    assert_close(transitions[1].offset_hz, 800.0, 1.0e-10);
    assert_close(transitions[1].center_ppm, 2.0, 1.0e-12);
    assert_close(transitions[1].intensity, 0.5, 1.0e-12);
    assert_eq!(transitions[1].contribution_count, 2);
    Ok(())
}

#[test]
fn detects_selected_spin_only() -> anyhow::Result<()> {
    let system = SpinHalfSystem {
        spins: vec![SpinHalf { shift_ppm: 1.0 }, SpinHalf { shift_ppm: 2.0 }],
        couplings: Vec::new(),
    };
    let transitions = exact_spin_half_transitions(
        &system,
        &ExactSpinOptions {
            detected_spins: vec![1],
            ..ExactSpinOptions::default()
        },
    )?;

    assert_eq!(transitions.len(), 1);
    assert_close(transitions[0].frequency_hz, 800.0, 1.0e-10);
    assert_close(transitions[0].center_ppm, 2.0, 1.0e-12);
    assert_close(transitions[0].intensity, 0.5, 1.0e-12);
    assert_eq!(transitions[0].contribution_count, 2);
    Ok(())
}

#[test]
fn scalar_coupling_splits_two_spin_transitions() -> anyhow::Result<()> {
    let system = SpinHalfSystem {
        spins: vec![SpinHalf { shift_ppm: 1.0 }, SpinHalf { shift_ppm: 1.05 }],
        couplings: vec![ScalarCoupling {
            spin_a: 0,
            spin_b: 1,
            j_hz: 12.0,
        }],
    };
    let transitions = exact_spin_half_transitions(&system, &ExactSpinOptions::default())?;

    assert_eq!(transitions.len(), 4);
    assert!(
        transitions
            .windows(2)
            .all(|pair| pair[0].frequency_hz <= pair[1].frequency_hz)
    );
    assert!(
        transitions
            .iter()
            .all(|transition| transition.intensity > 0.0)
    );
    Ok(())
}

#[test]
fn matches_ab_reference_fixture() -> anyhow::Result<()> {
    let system = SpinHalfSystem {
        spins: vec![SpinHalf { shift_ppm: 7.0 }, SpinHalf { shift_ppm: 7.04 }],
        couplings: vec![ScalarCoupling {
            spin_a: 0,
            spin_b: 1,
            j_hz: 8.0,
        }],
    };
    let transitions = exact_spin_half_transitions(&system, &ExactSpinOptions::default())?;
    let expected = [
        (
            2_795.055_728_090_000_7,
            6.987_639_320_225_002,
            0.138_196_601_125_010_56,
        ),
        (
            2_803.055_728_090_000_7,
            7.007_639_320_225_001,
            0.361_803_398_874_989_47,
        ),
        (
            2_812.944_271_909_999_3,
            7.032_360_679_774_999,
            0.361_803_398_874_989_47,
        ),
        (
            2_820.944_271_909_999_3,
            7.052_360_679_774_998,
            0.138_196_601_125_010_56,
        ),
    ];

    assert_eq!(transitions.len(), expected.len());
    for (transition, (frequency_hz, center_ppm, intensity)) in transitions.iter().zip(expected) {
        assert_close(transition.frequency_hz, frequency_hz, 1.0e-8);
        assert_close(transition.offset_hz, frequency_hz, 1.0e-8);
        assert_close(transition.center_ppm, center_ppm, 1.0e-10);
        assert_close(transition.intensity, intensity, 1.0e-10);
    }
    Ok(())
}

#[test]
fn simulator_trait_runs_exact_transition_simulation() -> anyhow::Result<()> {
    let system = SpinHalfSystem {
        spins: vec![SpinHalf { shift_ppm: 1.5 }],
        couplings: Vec::new(),
    };
    let transitions = ExactSpinOptions::default().simulate(&system)?;

    assert_eq!(transitions.len(), 1);
    assert_close(transitions[0].center_ppm, 1.5, 1.0e-12);
    Ok(())
}

#[test]
fn builders_create_chainable_exact_system_and_options() -> anyhow::Result<()> {
    let system = SpinHalfSystem::new()
        .with_spin(1.2)
        .with_spin(1.25)
        .with_coupling(0, 1, 7.0);

    assert_eq!(system.spins, vec![SpinHalf::new(1.2), SpinHalf::new(1.25)]);
    assert_eq!(system.couplings, vec![ScalarCoupling::new(0, 1, 7.0)]);

    let options = ExactSpinOptions::new()
        .with_spectrometer_mhz(500.0)
        .with_intensity_threshold(1.0e-14)
        .with_frequency_tolerance_hz(1.0e-8)
        .with_max_spins(4)
        .with_detected_spins([0, 1]);
    let transitions = options.simulate(&system)?;

    assert_eq!(transitions.len(), 4);
    assert!(
        transitions
            .iter()
            .all(|transition| transition.frequency_hz > 0.0)
    );
    Ok(())
}

#[test]
fn from_shifts_builds_uncoupled_spin_system() {
    let system = SpinHalfSystem::from_shifts([1.0, 2.0, 3.0]);

    assert_eq!(
        system.spins,
        vec![SpinHalf::new(1.0), SpinHalf::new(2.0), SpinHalf::new(3.0)]
    );
    assert!(system.couplings.is_empty());
}

#[test]
fn rejects_invalid_couplings_and_options() {
    let invalid_coupling = SpinHalfSystem {
        spins: vec![SpinHalf { shift_ppm: 1.0 }],
        couplings: vec![ScalarCoupling {
            spin_a: 0,
            spin_b: 1,
            j_hz: 8.0,
        }],
    };
    let error = exact_spin_half_transitions(&invalid_coupling, &ExactSpinOptions::default())
        .expect_err("out-of-range coupling should fail");
    assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));

    let valid_system = SpinHalfSystem {
        spins: vec![SpinHalf { shift_ppm: 1.0 }],
        couplings: Vec::new(),
    };
    let error = exact_spin_half_transitions(
        &valid_system,
        &ExactSpinOptions {
            intensity_threshold: -1.0,
            ..ExactSpinOptions::default()
        },
    )
    .expect_err("negative intensity threshold should fail");
    assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));
}

#[test]
fn rejects_invalid_detected_spin_indices() {
    let system = SpinHalfSystem {
        spins: vec![SpinHalf { shift_ppm: 1.0 }, SpinHalf { shift_ppm: 2.0 }],
        couplings: Vec::new(),
    };
    let error = exact_spin_half_transitions(
        &system,
        &ExactSpinOptions {
            detected_spins: vec![2],
            ..ExactSpinOptions::default()
        },
    )
    .expect_err("out-of-range detected spin should fail");
    assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));

    let error = exact_spin_half_transitions(
        &system,
        &ExactSpinOptions {
            detected_spins: vec![0, 0],
            ..ExactSpinOptions::default()
        },
    )
    .expect_err("duplicate detected spin should fail");
    assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));
}

#[test]
fn rejects_duplicate_couplings() {
    let system = SpinHalfSystem {
        spins: vec![SpinHalf { shift_ppm: 1.0 }, SpinHalf { shift_ppm: 2.0 }],
        couplings: vec![
            ScalarCoupling {
                spin_a: 0,
                spin_b: 1,
                j_hz: 8.0,
            },
            ScalarCoupling {
                spin_a: 1,
                spin_b: 0,
                j_hz: 8.0,
            },
        ],
    };
    let error = exact_spin_half_transitions(&system, &ExactSpinOptions::default())
        .expect_err("duplicate coupling should fail");
    assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));
}

fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual} != {expected}"
    );
}
