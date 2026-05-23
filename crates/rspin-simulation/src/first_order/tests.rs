use super::*;

#[test]
fn creates_reference_like_doublet() -> anyhow::Result<()> {
    let multiplet = FirstOrderMultiplet {
        center_ppm: 7.0,
        area: 1.0,
        couplings: vec![CouplingGroup {
            j_hz: 8.0,
            equivalent_spins: 1,
        }],
    };
    let transitions = multiplet_transitions(&multiplet, FirstOrderOptions::default())?;

    assert_eq!(transitions.len(), 2);
    assert_close(transitions[0].center_ppm, 6.99);
    assert_close(transitions[0].frequency_hz, 2796.0);
    assert_close(transitions[0].intensity, 0.5);
    assert_close(transitions[1].center_ppm, 7.01);
    assert_close(transitions[1].frequency_hz, 2804.0);
    assert_close(transitions[1].intensity, 0.5);
    Ok(())
}

#[test]
fn creates_binomial_triplet() -> anyhow::Result<()> {
    let multiplet = FirstOrderMultiplet {
        center_ppm: 1.0,
        area: 2.0,
        couplings: vec![CouplingGroup {
            j_hz: 10.0,
            equivalent_spins: 2,
        }],
    };
    let transitions = multiplet_transitions(&multiplet, FirstOrderOptions::default())?;
    let intensities = transitions
        .iter()
        .map(|transition| transition.intensity)
        .collect::<Vec<_>>();

    assert_eq!(transitions.len(), 3);
    assert_close(intensities[0], 0.5);
    assert_close(intensities[1], 1.0);
    assert_close(intensities[2], 0.5);
    Ok(())
}

#[test]
fn merges_overlapping_transitions() -> anyhow::Result<()> {
    let multiplet = FirstOrderMultiplet {
        center_ppm: 1.0,
        area: 1.0,
        couplings: vec![
            CouplingGroup {
                j_hz: 8.0,
                equivalent_spins: 1,
            },
            CouplingGroup {
                j_hz: 8.0,
                equivalent_spins: 1,
            },
        ],
    };
    let transitions = multiplet_transitions(&multiplet, FirstOrderOptions::default())?;
    assert_eq!(transitions.len(), 3);
    assert_close(transitions[1].intensity, 0.5);
    Ok(())
}

#[test]
fn simulates_dense_spectrum() -> anyhow::Result<()> {
    let multiplet = FirstOrderMultiplet {
        center_ppm: 7.0,
        area: 1.0,
        couplings: vec![CouplingGroup {
            j_hz: 8.0,
            equivalent_spins: 1,
        }],
    };
    let spectrum = simulate_multiplet_1d(
        &multiplet,
        SimulationOptions {
            from_ppm: 6.95,
            to_ppm: 7.05,
            points: 101,
            line_width_hz: 1.0,
            spectrometer_mhz: 400.0,
            line_shape: LineShape::Lorentzian,
        },
    )?;

    assert_eq!(spectrum.len(), 101);
    assert_eq!(spectrum.metadata.frequency_mhz, Some(400.0));
    assert!(spectrum.intensities.iter().all(|value| *value >= 0.0));
    Ok(())
}

#[test]
fn rejects_invalid_line_width() {
    let multiplet = FirstOrderMultiplet {
        center_ppm: 1.0,
        area: 1.0,
        couplings: Vec::new(),
    };
    let error = simulate_multiplet_1d(
        &multiplet,
        SimulationOptions {
            line_width_hz: 0.0,
            ..SimulationOptions::default()
        },
    )
    .expect_err("zero line width should fail");
    assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));
}

fn assert_close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-12, "{actual} != {expected}");
}
