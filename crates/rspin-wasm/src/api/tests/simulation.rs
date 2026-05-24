use rspin_core::Spectrum1D;

#[cfg(feature = "first-order")]
use super::super::simulate_first_order_multiplet_json;
use super::super::{
    decompose_exact_spin_half_spectrum_json, from_json, simulate_exact_spin_half_spectrum_json,
    simulate_exact_spin_half_transitions_json,
};

#[cfg(feature = "first-order")]
#[test]
fn simulates_first_order_json() -> anyhow::Result<()> {
    let spectrum_json = simulate_first_order_multiplet_json(
        r#"{"center_ppm":7.0,"area":1.0,"couplings":[{"j_hz":8.0,"equivalent_spins":1}]}"#,
        r#"{"from_ppm":6.95,"to_ppm":7.05,"points":16,"line_width_hz":1.0,"spectrometer_mhz":400.0,"line_shape":"Lorentzian"}"#,
    )?;
    let spectrum: Spectrum1D = from_json(&spectrum_json)?;
    assert_eq!(spectrum.len(), 16);
    Ok(())
}

#[test]
fn simulates_exact_transitions_json() -> anyhow::Result<()> {
    let transitions_json = simulate_exact_spin_half_transitions_json(
        r#"{"spins":[{"shift_ppm":7.0},{"shift_ppm":7.04}],"couplings":[{"spin_a":0,"spin_b":1,"j_hz":8.0}]}"#,
        r#"{"spectrometer_mhz":400.0,"intensity_threshold":1e-12,"frequency_tolerance_hz":1e-9,"max_spins":10}"#,
    )?;
    let transitions: Vec<rspin_simulation::ExactTransition> = from_json(&transitions_json)?;

    assert_eq!(transitions.len(), 4);
    assert!((transitions[0].center_ppm - 6.987_639_320_225_002).abs() < 1.0e-10);
    Ok(())
}

#[test]
fn simulates_exact_detected_spin_json() -> anyhow::Result<()> {
    let transitions_json = simulate_exact_spin_half_transitions_json(
        r#"{"spins":[{"shift_ppm":1.0},{"shift_ppm":2.0}],"couplings":[]}"#,
        r#"{"spectrometer_mhz":400.0,"intensity_threshold":1e-12,"frequency_tolerance_hz":1e-9,"max_spins":10,"detected_spins":[1]}"#,
    )?;
    let transitions: Vec<rspin_simulation::ExactTransition> = from_json(&transitions_json)?;

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
    let spectrum: Spectrum1D = from_json(&spectrum_json)?;

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
    let decomposition: rspin_simulation::ExactSpectrumDecomposition1D =
        from_json(&decomposition_json)?;

    assert_eq!(decomposition.spectrum.len(), 32);
    assert_eq!(
        decomposition.contributions.len(),
        decomposition.transitions.len()
    );
    Ok(())
}
