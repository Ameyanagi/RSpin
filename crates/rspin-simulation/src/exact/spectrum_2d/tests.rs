use rspin_core::RSpinError;

use super::*;
use crate::{LineShape, ScalarCoupling, Simulator, SpinHalf};

#[test]
fn renders_default_coupling_pairs() -> anyhow::Result<()> {
    let system = coupled_ab_system();
    let result = decompose_exact_spin_half_2d(
        &system,
        &ExactSpectrum2DOptions::new()
            .with_x_ppm_range(6.95, 7.08)
            .with_y_ppm_range(6.95, 7.08)
            .with_points(32, 24)
            .with_volume(2.0)
            .with_x_line_width_hz(1.0)
            .with_y_line_width_hz(1.5)
            .with_line_shape(LineShape::Gaussian),
    )?;

    assert_eq!(result.spectrum.shape(), (32, 24));
    assert_eq!(result.spectrum.metadata.frequency_mhz, Some(400.0));
    assert_eq!(result.spectrum.processing.len(), 1);
    assert_eq!(
        result.spectrum.processing[0].operation,
        "simulate_exact_spin_half_2d"
    );
    assert_eq!(result.contributions.len(), 16);
    assert!(result.spectrum.z.iter().all(|value| *value >= 0.0));
    assert!(result.spectrum.z.iter().any(|value| *value > 0.0));
    Ok(())
}

#[test]
fn supports_chainable_options_and_trait_api() -> anyhow::Result<()> {
    let system = SpinHalfSystem::new().with_spin(1.0).with_spin(2.0);
    let options = ExactSpectrum2DOptions::new()
        .with_x_ppm_range(0.95, 1.05)
        .with_y_ppm_range(1.95, 2.05)
        .with_points(5, 7)
        .with_volume(3.0)
        .with_x_line_width_hz(2.0)
        .with_y_line_width_hz(4.0)
        .with_line_shape(LineShape::PseudoVoigt)
        .with_transition_options(ExactSpinOptions::new().with_spectrometer_mhz(400.0))
        .with_spin_pair(0, 1);
    let spectrum = options.simulate(&system)?;

    assert_eq!(spectrum.shape(), (5, 7));
    assert!(spectrum.z[17] > spectrum.z[0]);
    Ok(())
}

#[test]
fn decomposition_contributions_sum_to_spectrum() -> anyhow::Result<()> {
    let system = SpinHalfSystem::new().with_spin(1.0).with_spin(2.0);
    let result = decompose_exact_spin_half_2d(
        &system,
        &ExactSpectrum2DOptions::new()
            .with_x_ppm_range(0.95, 1.05)
            .with_y_ppm_range(1.95, 2.05)
            .with_points(5, 5)
            .with_spin_pairs([ExactSpinPair::new(0, 1)]),
    )?;

    assert_eq!(result.contributions.len(), 1);
    for point in 0..result.spectrum.z.len() {
        let sum = result
            .contributions
            .iter()
            .map(|contribution| contribution.z[point])
            .sum::<f64>();
        assert_close(result.spectrum.z[point], sum, 1.0e-10);
    }
    Ok(())
}

#[test]
fn empty_default_pairs_render_zero_spectrum() -> anyhow::Result<()> {
    let system = SpinHalfSystem::new().with_spin(1.0).with_spin(2.0);
    let result = decompose_exact_spin_half_2d(
        &system,
        &ExactSpectrum2DOptions::new()
            .with_x_ppm_range(0.0, 3.0)
            .with_y_ppm_range(0.0, 3.0)
            .with_points(3, 3)
            .without_spin_pairs(),
    )?;

    assert!(result.contributions.is_empty());
    assert_eq!(result.spectrum.z, vec![0.0; 9]);
    Ok(())
}

#[test]
fn rejects_invalid_2d_options_and_pairs() {
    let system = SpinHalfSystem::new().with_spin(1.0).with_spin(2.0);
    let zero_points = simulate_exact_spin_half_2d(
        &system,
        &ExactSpectrum2DOptions::new()
            .with_points(0, 8)
            .with_spin_pair(0, 1),
    )
    .expect_err("zero point count should fail");
    assert!(matches!(zero_points, RSpinError::InvalidSpectrum { .. }));

    let same_spin =
        simulate_exact_spin_half_2d(&system, &ExactSpectrum2DOptions::new().with_spin_pair(0, 0))
            .expect_err("same-spin pair should fail");
    assert!(matches!(same_spin, RSpinError::InvalidSpectrum { .. }));

    let duplicate = simulate_exact_spin_half_2d(
        &system,
        &ExactSpectrum2DOptions::new()
            .with_spin_pair(0, 1)
            .with_spin_pair(0, 1),
    )
    .expect_err("duplicate spin pair should fail");
    assert!(matches!(duplicate, RSpinError::InvalidSpectrum { .. }));

    let out_of_range =
        simulate_exact_spin_half_2d(&system, &ExactSpectrum2DOptions::new().with_spin_pair(0, 2))
            .expect_err("out-of-range spin pair should fail");
    assert!(matches!(out_of_range, RSpinError::InvalidSpectrum { .. }));
}

fn coupled_ab_system() -> SpinHalfSystem {
    SpinHalfSystem {
        spins: vec![SpinHalf { shift_ppm: 7.0 }, SpinHalf { shift_ppm: 7.04 }],
        couplings: vec![ScalarCoupling {
            spin_a: 0,
            spin_b: 1,
            j_hz: 8.0,
        }],
    }
}

fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual} != {expected}"
    );
}
