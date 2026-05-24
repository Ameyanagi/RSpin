use super::*;
use crate::{ScalarCoupling, Simulator, SpinHalf};

#[test]
fn renders_single_spin_with_requested_area() -> anyhow::Result<()> {
    let system = SpinHalfSystem {
        spins: vec![SpinHalf { shift_ppm: 2.0 }],
        couplings: Vec::new(),
    };
    let options = ExactSpectrumOptions {
        from_ppm: 1.99,
        to_ppm: 2.01,
        points: 11,
        area: 3.0,
        line_width_hz: 2.0,
        line_shape: LineShape::Lorentzian,
        transition_options: ExactSpinOptions::default(),
    };
    let spectrum = simulate_exact_spin_half_1d(&system, &options)?;
    let expected_center = LineShape::Lorentzian.value(2.0, 2.0, 2.0, 400.0, 3.0);

    assert_eq!(spectrum.len(), 11);
    assert_eq!(spectrum.metadata.frequency_mhz, Some(400.0));
    assert_eq!(spectrum.processing.len(), 1);
    assert_close(spectrum.intensities[5], expected_center, 1.0e-10);
    Ok(())
}

#[test]
fn preserves_negative_shift_position() -> anyhow::Result<()> {
    let system = SpinHalfSystem {
        spins: vec![SpinHalf { shift_ppm: -0.5 }],
        couplings: Vec::new(),
    };
    let spectrum = simulate_exact_spin_half_1d(
        &system,
        &ExactSpectrumOptions {
            from_ppm: -1.0,
            to_ppm: 1.0,
            points: 9,
            line_width_hz: 1.0,
            ..ExactSpectrumOptions::default()
        },
    )?;
    let max_point = spectrum
        .points()
        .max_by(|left, right| left.1.total_cmp(&right.1))
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "spectrum should contain points".to_owned(),
        })?;

    assert_close(max_point.0, -0.5, 1.0e-12);
    Ok(())
}

#[test]
fn renders_coupled_ab_system() -> anyhow::Result<()> {
    let system = SpinHalfSystem {
        spins: vec![SpinHalf { shift_ppm: 7.0 }, SpinHalf { shift_ppm: 7.04 }],
        couplings: vec![ScalarCoupling {
            spin_a: 0,
            spin_b: 1,
            j_hz: 8.0,
        }],
    };
    let spectrum = ExactSpectrumOptions {
        from_ppm: 6.95,
        to_ppm: 7.08,
        points: 128,
        area: 2.0,
        line_width_hz: 1.0,
        line_shape: LineShape::Gaussian,
        transition_options: ExactSpinOptions::default(),
    }
    .simulate(&system)?;

    assert_eq!(spectrum.len(), 128);
    assert!(spectrum.intensities.iter().all(|value| *value >= 0.0));
    assert!(spectrum.intensities.iter().any(|value| *value > 0.0));
    Ok(())
}

#[test]
fn builder_options_render_exact_spectrum() -> anyhow::Result<()> {
    let system = SpinHalfSystem::new().with_spin(2.0);
    let options = ExactSpectrumOptions::new()
        .with_ppm_range(1.99, 2.01)
        .with_points(11)
        .with_area(3.0)
        .with_line_width_hz(2.0)
        .with_line_shape(LineShape::Gaussian)
        .with_transition_options(ExactSpinOptions::new().with_spectrometer_mhz(400.0));
    let spectrum = options.simulate(&system)?;

    assert_eq!(spectrum.len(), 11);
    assert_eq!(spectrum.metadata.frequency_mhz, Some(400.0));
    assert!(spectrum.intensities[5] > spectrum.intensities[0]);
    Ok(())
}

#[test]
fn decomposes_spectrum_into_transition_contributions() -> anyhow::Result<()> {
    let system = SpinHalfSystem {
        spins: vec![SpinHalf { shift_ppm: 7.0 }, SpinHalf { shift_ppm: 7.04 }],
        couplings: vec![ScalarCoupling {
            spin_a: 0,
            spin_b: 1,
            j_hz: 8.0,
        }],
    };
    let result = decompose_exact_spin_half_1d(
        &system,
        &ExactSpectrumOptions {
            from_ppm: 6.95,
            to_ppm: 7.08,
            points: 64,
            area: 2.0,
            line_width_hz: 1.0,
            line_shape: LineShape::Lorentzian,
            transition_options: ExactSpinOptions::default(),
        },
    )?;

    assert_eq!(result.transitions.len(), 4);
    assert_eq!(result.contributions.len(), result.transitions.len());
    assert!(
        result
            .contributions
            .iter()
            .all(|contribution| contribution.intensities.len() == result.spectrum.len())
    );

    for point in 0..result.spectrum.len() {
        let sum = result
            .contributions
            .iter()
            .map(|contribution| contribution.intensities[point])
            .sum::<f64>();
        assert_close(result.spectrum.intensities[point], sum, 1.0e-10);
    }

    Ok(())
}

#[test]
fn rejects_invalid_rendering_options() {
    let system = SpinHalfSystem {
        spins: vec![SpinHalf { shift_ppm: 1.0 }],
        couplings: Vec::new(),
    };
    let error = simulate_exact_spin_half_1d(
        &system,
        &ExactSpectrumOptions {
            points: 0,
            ..ExactSpectrumOptions::default()
        },
    )
    .expect_err("zero points should fail");
    assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));

    let error = simulate_exact_spin_half_1d(
        &system,
        &ExactSpectrumOptions {
            area: 0.0,
            ..ExactSpectrumOptions::default()
        },
    )
    .expect_err("zero area should fail");
    assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));
}

fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual} != {expected}"
    );
}
