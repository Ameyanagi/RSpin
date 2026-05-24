use rspin_core::{RSpinError, Result};

use super::*;

#[test]
fn simulates_transitions_from_system_chain() -> anyhow::Result<()> {
    let transitions = SpinHalfSystem::new()
        .with_spin(1.0)
        .with_spin(2.0)
        .simulate_exact()
        .with_spectrometer_mhz(500.0)
        .with_detected_spin(1)
        .transitions()?;

    assert_eq!(transitions.len(), 1);
    assert_close(transitions[0].frequency_hz, 1_000.0, 1.0e-10);
    assert_close(transitions[0].center_ppm, 2.0, 1.0e-12);
    Ok(())
}

#[test]
fn renders_one_dimensional_spectrum_from_system_chain() -> anyhow::Result<()> {
    let system = SpinHalfSystem::new().with_spin(2.0);
    let spectrum = system
        .simulate_exact()
        .with_spectrometer_mhz(500.0)
        .render_1d()
        .with_ppm_range(1.99, 2.01)
        .with_points(11)
        .with_area(2.0)
        .with_line_width_hz(2.0)
        .with_line_shape(LineShape::PseudoVoigt)
        .run()?;

    assert_eq!(spectrum.len(), 11);
    assert_eq!(spectrum.metadata.frequency_mhz, Some(500.0));
    assert!(spectrum.intensities[5] > spectrum.intensities[0]);
    Ok(())
}

#[test]
fn decomposes_one_dimensional_spectrum_from_system_chain() -> anyhow::Result<()> {
    let system = SpinHalfSystem::new()
        .with_spin(7.0)
        .with_spin(7.04)
        .with_coupling(0, 1, 8.0);
    let decomposition = system
        .simulate_exact()
        .render_1d()
        .with_ppm_range(6.95, 7.08)
        .with_points(64)
        .decompose()?;

    assert_eq!(decomposition.transitions.len(), 4);
    assert_eq!(
        decomposition.contributions.len(),
        decomposition.transitions.len()
    );
    Ok(())
}

#[test]
fn renders_two_dimensional_spectrum_from_system_chain() -> anyhow::Result<()> {
    let system = SpinHalfSystem::new().with_spin(1.0).with_spin(2.0);
    let spectrum = system
        .simulate_exact()
        .with_spectrometer_mhz(400.0)
        .render_2d()
        .with_x_ppm_range(0.95, 1.05)
        .with_y_ppm_range(1.95, 2.05)
        .with_points(5, 7)
        .with_volume(3.0)
        .with_x_line_width_hz(2.0)
        .with_y_line_width_hz(4.0)
        .with_line_shape(LineShape::PseudoVoigt)
        .with_spin_pair(0, 1)
        .run()?;

    assert_eq!(spectrum.shape(), (5, 7));
    assert_eq!(spectrum.metadata.frequency_mhz, Some(400.0));
    assert!(spectrum.z[17] > spectrum.z[0]);
    Ok(())
}

#[test]
fn fallible_workflow_preserves_initial_error() {
    let system: Result<SpinHalfSystem> = Err(parse_error());
    let error = system
        .simulate_exact()
        .with_spectrometer_mhz(500.0)
        .render_1d()
        .with_points(0)
        .run()
        .expect_err("initial system error should be returned first");

    assert!(matches!(
        error,
        RSpinError::Parse {
            format: "spin-system",
            ..
        }
    ));
}

#[test]
fn fallible_two_dimensional_workflow_runs() -> anyhow::Result<()> {
    let system: Result<SpinHalfSystem> = Ok(SpinHalfSystem::new().with_spin(1.0).with_spin(2.0));
    let spectrum = system
        .simulate_exact()
        .render_2d()
        .with_x_ppm_range(0.95, 1.05)
        .with_y_ppm_range(1.95, 2.05)
        .with_points(3, 3)
        .with_spin_pair(0, 1)
        .run()?;

    assert_eq!(spectrum.shape(), (3, 3));
    assert!(spectrum.z[4] > 0.0);
    Ok(())
}

fn parse_error() -> RSpinError {
    RSpinError::Parse {
        format: "spin-system",
        message: "synthetic failure".to_owned(),
    }
}

fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual} != {expected}"
    );
}
