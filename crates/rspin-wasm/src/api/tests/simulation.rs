use super::super::{
    decompose_exact_spin_half_spectrum_2d_json, decompose_exact_spin_half_spectrum_json,
    simulate_exact_spin_half_spectrum_2d_json, simulate_exact_spin_half_spectrum_json,
    simulate_exact_spin_half_transitions_json, spectrum1d_from_json, spectrum2d_from_json,
    validate_exact_spectrum_2d_options_json, validate_exact_spectrum_options_json,
    validate_exact_spin_half_system_json, validate_exact_spin_options_json,
    write_exact_transitions_csv_json,
};

#[test]
fn validates_exact_simulation_json() -> anyhow::Result<()> {
    let system_json = validate_exact_spin_half_system_json(
        r#"{"spins":[{"shift_ppm":1.0},{"shift_ppm":2.0}],"couplings":[{"spin_a":0,"spin_b":1,"j_hz":8.0}]}"#,
    )?;
    assert!(system_json.contains(rspin_io::SPIN_HALF_SYSTEM_JSON_FORMAT));
    assert_eq!(
        rspin_io::read_spin_half_system_json(&system_json)?
            .spins
            .len(),
        2
    );

    let options_json = validate_exact_spin_options_json(
        r#"{"spectrometer_mhz":400.0,"intensity_threshold":1e-12,"frequency_tolerance_hz":1e-9,"max_spins":10}"#,
    )?;
    assert!(options_json.contains(rspin_io::EXACT_SPIN_OPTIONS_JSON_FORMAT));
    let parsed_options = rspin_io::read_exact_spin_options_json(&options_json)?;
    assert!((parsed_options.spectrometer_mhz - 400.0).abs() < 1.0e-12);

    let options_1d_json = validate_exact_spectrum_options_json(
        r#"{"from_ppm":0.0,"to_ppm":3.0,"points":16,"area":1.0,"line_width_hz":1.0,"line_shape":"Lorentzian","transition_options":{"spectrometer_mhz":400.0,"intensity_threshold":1e-12,"frequency_tolerance_hz":1e-9,"max_spins":10}}"#,
    )?;
    assert!(options_1d_json.contains(rspin_io::EXACT_SPECTRUM_1D_OPTIONS_JSON_FORMAT));

    let options_2d_json = validate_exact_spectrum_2d_options_json(
        r#"{"x_from_ppm":0.0,"x_to_ppm":3.0,"x_points":8,"y_from_ppm":0.0,"y_to_ppm":3.0,"y_points":8,"volume":1.0,"x_line_width_hz":1.0,"y_line_width_hz":1.0,"line_shape":"Lorentzian","transition_options":{"spectrometer_mhz":400.0,"intensity_threshold":1e-12,"frequency_tolerance_hz":1e-9,"max_spins":10},"spin_pairs":[{"x_spin":0,"y_spin":1}]}"#,
    )?;
    assert!(options_2d_json.contains(rspin_io::EXACT_SPECTRUM_2D_OPTIONS_JSON_FORMAT));
    Ok(())
}

#[test]
fn rejects_invalid_exact_simulation_json_validation() {
    let error = validate_exact_spin_half_system_json(r#"{"spins":[],"couplings":[]}"#)
        .expect_err("empty system should fail validation");
    assert!(matches!(
        error,
        rspin_core::RSpinError::InvalidSpectrum { .. }
    ));

    let error = validate_exact_spectrum_options_json(
        r#"{"from_ppm":0.0,"to_ppm":3.0,"points":0,"area":1.0,"line_width_hz":1.0,"line_shape":"Lorentzian","transition_options":{"spectrometer_mhz":400.0,"intensity_threshold":1e-12,"frequency_tolerance_hz":1e-9,"max_spins":10}}"#,
    )
    .expect_err("zero render point count should fail validation");
    assert!(matches!(
        error,
        rspin_core::RSpinError::InvalidSpectrum { .. }
    ));
}

#[test]
fn simulates_exact_transitions_json() -> anyhow::Result<()> {
    let system_json = rspin_io::write_spin_half_system_json(
        &rspin_simulation::SpinHalfSystem::new()
            .with_spin(7.0)
            .with_spin(7.04)
            .with_coupling(0, 1, 8.0),
    )?;
    let options_json = rspin_io::write_exact_spin_options_json(
        &rspin_simulation::ExactSpinOptions::new().with_spectrometer_mhz(400.0),
    )?;
    let transitions_json = simulate_exact_spin_half_transitions_json(&system_json, &options_json)?;
    assert!(transitions_json.contains(rspin_io::EXACT_TRANSITIONS_JSON_FORMAT));
    let transitions = rspin_io::read_exact_transitions_json(&transitions_json)?;

    assert_eq!(transitions.len(), 4);
    assert!((transitions[0].center_ppm - 6.987_639_320_225_002).abs() < 1.0e-10);
    Ok(())
}

#[test]
fn writes_exact_transitions_csv_json() -> anyhow::Result<()> {
    let transitions_json = simulate_exact_spin_half_transitions_json(
        r#"{"spins":[{"shift_ppm":1.0},{"shift_ppm":1.02}],"couplings":[{"spin_a":0,"spin_b":1,"j_hz":8.0}]}"#,
        r#"{"spectrometer_mhz":400.0,"intensity_threshold":1e-12,"frequency_tolerance_hz":1e-9,"max_spins":10}"#,
    )?;
    let csv = write_exact_transitions_csv_json(&transitions_json)?;

    assert!(csv.starts_with("# format=RSpin Exact Transitions CSV\n"));
    assert!(csv.contains("frequency_hz,offset_hz,center_ppm,intensity,contribution_count\n"));
    assert_eq!(csv.lines().count(), 6);
    Ok(())
}

#[test]
fn simulates_exact_detected_spin_json() -> anyhow::Result<()> {
    let transitions_json = simulate_exact_spin_half_transitions_json(
        r#"{"spins":[{"shift_ppm":1.0},{"shift_ppm":2.0}],"couplings":[]}"#,
        r#"{"spectrometer_mhz":400.0,"intensity_threshold":1e-12,"frequency_tolerance_hz":1e-9,"max_spins":10,"detected_spins":[1]}"#,
    )?;
    let transitions = rspin_io::read_exact_transitions_json(&transitions_json)?;

    assert_eq!(transitions.len(), 1);
    assert!((transitions[0].center_ppm - 2.0).abs() < 1.0e-12);
    Ok(())
}

#[test]
fn simulates_exact_spectrum_json() -> anyhow::Result<()> {
    let spectrum_json = simulate_exact_spin_half_spectrum_json(
        r#"{"spins":[{"shift_ppm":2.0}],"couplings":[]}"#,
        r#"{"from_ppm":1.99,"to_ppm":2.01,"points":11,"area":2.0,"line_width_hz":1.0,"line_shape":"PseudoVoigt","transition_options":{"spectrometer_mhz":400.0,"intensity_threshold":1e-12,"frequency_tolerance_hz":1e-9,"max_spins":10}}"#,
    )?;
    let spectrum = spectrum1d_from_json(&spectrum_json)?;

    assert_eq!(spectrum.len(), 11);
    assert_eq!(spectrum.metadata.frequency_mhz, Some(400.0));
    assert!(spectrum.intensities.iter().any(|value| *value > 0.0));
    Ok(())
}

#[test]
fn decomposes_exact_spectrum_json() -> anyhow::Result<()> {
    let decomposition_json = decompose_exact_spin_half_spectrum_json(
        r#"{"spins":[{"shift_ppm":7.0},{"shift_ppm":7.04}],"couplings":[{"spin_a":0,"spin_b":1,"j_hz":8.0}]}"#,
        r#"{"from_ppm":6.95,"to_ppm":7.08,"points":32,"area":1.0,"line_width_hz":1.0,"line_shape":"Lorentzian","transition_options":{"spectrometer_mhz":400.0,"intensity_threshold":1e-12,"frequency_tolerance_hz":1e-9,"max_spins":10}}"#,
    )?;
    assert!(decomposition_json.contains(rspin_io::EXACT_DECOMPOSITION_1D_JSON_FORMAT));
    let decomposition = rspin_io::read_exact_decomposition_1d_json(&decomposition_json)?;

    assert_eq!(decomposition.spectrum.len(), 32);
    assert_eq!(
        decomposition.contributions.len(),
        decomposition.transitions.len()
    );
    Ok(())
}

#[test]
fn simulates_exact_2d_spectrum_json() -> anyhow::Result<()> {
    let spectrum_json = simulate_exact_spin_half_spectrum_2d_json(
        r#"{"spins":[{"shift_ppm":1.0},{"shift_ppm":2.0}],"couplings":[]}"#,
        r#"{"x_from_ppm":0.95,"x_to_ppm":1.05,"x_points":5,"y_from_ppm":1.95,"y_to_ppm":2.05,"y_points":5,"volume":1.0,"x_line_width_hz":1.0,"y_line_width_hz":1.0,"line_shape":"Lorentzian","transition_options":{"spectrometer_mhz":400.0,"intensity_threshold":1e-12,"frequency_tolerance_hz":1e-9,"max_spins":10},"spin_pairs":[{"x_spin":0,"y_spin":1}]}"#,
    )?;
    let spectrum = spectrum2d_from_json(&spectrum_json)?;

    assert_eq!(spectrum.shape(), (5, 5));
    assert!(spectrum.z[12] > spectrum.z[0]);
    Ok(())
}

#[test]
fn decomposes_exact_2d_spectrum_json() -> anyhow::Result<()> {
    let decomposition_json = decompose_exact_spin_half_spectrum_2d_json(
        r#"{"spins":[{"shift_ppm":7.0},{"shift_ppm":7.04}],"couplings":[{"spin_a":0,"spin_b":1,"j_hz":8.0}]}"#,
        r#"{"x_from_ppm":6.95,"x_to_ppm":7.08,"x_points":16,"y_from_ppm":6.95,"y_to_ppm":7.08,"y_points":16,"volume":1.0,"x_line_width_hz":1.0,"y_line_width_hz":1.0,"line_shape":"Gaussian","transition_options":{"spectrometer_mhz":400.0,"intensity_threshold":1e-12,"frequency_tolerance_hz":1e-9,"max_spins":10}}"#,
    )?;
    assert!(decomposition_json.contains(rspin_io::EXACT_DECOMPOSITION_2D_JSON_FORMAT));
    let decomposition = rspin_io::read_exact_decomposition_2d_json(&decomposition_json)?;

    assert_eq!(decomposition.spectrum.shape(), (16, 16));
    assert_eq!(decomposition.contributions.len(), 16);
    Ok(())
}
