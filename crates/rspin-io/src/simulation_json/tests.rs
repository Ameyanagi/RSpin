use rspin_core::RSpinError;
use rspin_simulation::{
    ExactSpectrum2DOptions, ExactSpectrumDecomposition1D, ExactSpectrumDecomposition2D,
    ExactSpectrumOptions, ExactSpinOptions, ExactSpinPair, ExactTransition, SpinHalfSystem,
    decompose_exact_spin_half_1d, decompose_exact_spin_half_2d, exact_spin_half_transitions,
};

use crate::{SpectrumReader, SpectrumWriter};

use super::*;

#[test]
fn round_trips_exact_simulation_input_json() -> anyhow::Result<()> {
    let system = system_fixture();
    let system_text = write_spin_half_system_json(&system)?;
    assert!(system_text.contains(&format!("\"format\":\"{SPIN_HALF_SYSTEM_JSON_FORMAT}\"")));
    assert!(system_text.contains(&format!("\"version\":{SIMULATION_JSON_VERSION}")));
    assert_eq!(read_spin_half_system_json(&system_text)?, system);

    let spin_options = spin_options_fixture();
    let spin_options_text = write_exact_spin_options_json(&spin_options)?;
    assert!(spin_options_text.contains(EXACT_SPIN_OPTIONS_JSON_FORMAT));
    assert_eq!(
        read_exact_spin_options_json(&spin_options_text)?,
        spin_options
    );

    let options_1d = spectrum_options_fixture(spin_options.clone());
    let options_1d_text = write_exact_spectrum_options_json(&options_1d)?;
    assert!(options_1d_text.contains(EXACT_SPECTRUM_1D_OPTIONS_JSON_FORMAT));
    assert_eq!(
        read_exact_spectrum_options_json(&options_1d_text)?,
        options_1d
    );

    let options_2d = spectrum_2d_options_fixture(spin_options);
    let options_2d_text = write_exact_spectrum_2d_options_json(&options_2d)?;
    assert!(options_2d_text.contains(EXACT_SPECTRUM_2D_OPTIONS_JSON_FORMAT));
    assert_eq!(
        read_exact_spectrum_2d_options_json(&options_2d_text)?,
        options_2d
    );
    Ok(())
}

#[test]
fn round_trips_exact_simulation_output_json() -> anyhow::Result<()> {
    let system = system_fixture();
    let spin_options = spin_options_fixture();
    let transitions = exact_spin_half_transitions(&system, &spin_options)?;
    let transitions_text = write_exact_transitions_json(&transitions)?;

    assert!(transitions_text.contains(EXACT_TRANSITIONS_JSON_FORMAT));
    assert_transitions_close(
        &read_exact_transitions_json(&transitions_text)?,
        &transitions,
    );

    let decomposition_1d = decompose_exact_spin_half_1d(
        &system,
        &spectrum_options_fixture(ExactSpinOptions::new().with_spectrometer_mhz(400.0)),
    )?;
    let decomposition_1d_text = write_exact_decomposition_1d_json(&decomposition_1d)?;
    assert!(decomposition_1d_text.contains(EXACT_DECOMPOSITION_1D_JSON_FORMAT));
    let parsed_decomposition_1d = read_exact_decomposition_1d_json(&decomposition_1d_text)?;
    assert_decomposition_1d_close(&parsed_decomposition_1d, &decomposition_1d);

    let decomposition_2d = decompose_exact_spin_half_2d(
        &system,
        &spectrum_2d_options_fixture(ExactSpinOptions::new().with_spectrometer_mhz(400.0)),
    )?;
    let decomposition_2d_text = write_exact_decomposition_2d_json(&decomposition_2d)?;
    assert!(decomposition_2d_text.contains(EXACT_DECOMPOSITION_2D_JSON_FORMAT));
    let parsed_decomposition_2d = read_exact_decomposition_2d_json(&decomposition_2d_text)?;
    assert_decomposition_2d_close(&parsed_decomposition_2d, &decomposition_2d);
    Ok(())
}

#[test]
fn reads_legacy_raw_exact_simulation_json() -> anyhow::Result<()> {
    let system = system_fixture();
    assert_eq!(
        read_spin_half_system_json(&serde_json::to_string(&system)?)?,
        system
    );

    let spin_options = spin_options_fixture();
    assert_eq!(
        read_exact_spin_options_json(&serde_json::to_string(&spin_options)?)?,
        spin_options
    );

    let options_1d = spectrum_options_fixture(spin_options.clone());
    assert_eq!(
        read_exact_spectrum_options_json(&serde_json::to_string(&options_1d)?)?,
        options_1d
    );

    let transitions = exact_spin_half_transitions(&system, &spin_options)?;
    let parsed_transitions = read_exact_transitions_json(&serde_json::to_string(&transitions)?)?;
    assert_transitions_close(&parsed_transitions, &transitions);
    Ok(())
}

#[test]
fn rejects_wrong_simulation_json_headers() {
    let wrong_format = read_spin_half_system_json(
        r#"{"format":"rspin.exact_spin_options","version":1,"system":{"spins":[],"couplings":[]}}"#,
    )
    .expect_err("wrong simulation JSON format should fail");
    assert!(matches!(wrong_format, RSpinError::Parse { .. }));

    let unsupported_version = read_exact_transitions_json(
        r#"{"format":"rspin.exact_transitions","version":2,"transitions":[]}"#,
    )
    .expect_err("unsupported simulation JSON version should fail");
    assert!(matches!(
        unsupported_version,
        RSpinError::Unsupported {
            feature: "simulation JSON version"
        }
    ));
}

#[test]
fn exact_simulation_json_codecs_implement_traits() -> anyhow::Result<()> {
    let system = system_fixture();
    let system_text = <JsonSpinHalfSystem as SpectrumWriter<SpinHalfSystem>>::write_string(
        &JsonSpinHalfSystem,
        &system,
    )?;
    let parsed_system: SpinHalfSystem =
        SpectrumReader::read_str(&JsonSpinHalfSystem, &system_text)?;

    assert_eq!(format!("{JsonSpinHalfSystem:?}"), "JsonSpinHalfSystem");
    assert_eq!(parsed_system, system);

    let transitions = exact_spin_half_transitions(&system, &spin_options_fixture())?;
    let transitions_text =
        <JsonExactTransitions as SpectrumWriter<[ExactTransition]>>::write_string(
            &JsonExactTransitions,
            &transitions,
        )?;
    let parsed_transitions: Vec<ExactTransition> =
        SpectrumReader::read_str(&JsonExactTransitions, &transitions_text)?;

    assert_eq!(format!("{JsonExactTransitions:?}"), "JsonExactTransitions");
    assert_transitions_close(&parsed_transitions, &transitions);
    assert_option_codecs_round_trip()?;
    assert_decomposition_codecs_round_trip()?;
    Ok(())
}

fn assert_option_codecs_round_trip() -> anyhow::Result<()> {
    let spin_options = spin_options_fixture();
    let spin_options_text =
        <JsonExactSpinOptions as SpectrumWriter<ExactSpinOptions>>::write_string(
            &JsonExactSpinOptions,
            &spin_options,
        )?;
    assert_eq!(
        SpectrumReader::read_str(&JsonExactSpinOptions, &spin_options_text)?,
        spin_options
    );

    let options_1d = spectrum_options_fixture(spin_options.clone());
    let options_1d_text =
        <JsonExactSpectrumOptions as SpectrumWriter<ExactSpectrumOptions>>::write_string(
            &JsonExactSpectrumOptions,
            &options_1d,
        )?;
    assert_eq!(
        SpectrumReader::read_str(&JsonExactSpectrumOptions, &options_1d_text)?,
        options_1d
    );

    let options_2d = spectrum_2d_options_fixture(spin_options);
    let options_2d_text =
        <JsonExactSpectrum2DOptions as SpectrumWriter<ExactSpectrum2DOptions>>::write_string(
            &JsonExactSpectrum2DOptions,
            &options_2d,
        )?;
    assert_eq!(
        SpectrumReader::read_str(&JsonExactSpectrum2DOptions, &options_2d_text)?,
        options_2d
    );
    Ok(())
}

fn assert_decomposition_codecs_round_trip() -> anyhow::Result<()> {
    let system = system_fixture();
    let options = ExactSpinOptions::new().with_spectrometer_mhz(400.0);
    let decomposition_1d =
        decompose_exact_spin_half_1d(&system, &spectrum_options_fixture(options.clone()))?;
    let decomposition_1d_text = <JsonExactDecomposition1D as SpectrumWriter<
        ExactSpectrumDecomposition1D,
    >>::write_string(&JsonExactDecomposition1D, &decomposition_1d)?;
    let parsed_decomposition_1d =
        SpectrumReader::read_str(&JsonExactDecomposition1D, &decomposition_1d_text)?;
    assert_decomposition_1d_close(&parsed_decomposition_1d, &decomposition_1d);

    let decomposition_2d =
        decompose_exact_spin_half_2d(&system, &spectrum_2d_options_fixture(options))?;
    let decomposition_2d_text = <JsonExactDecomposition2D as SpectrumWriter<
        ExactSpectrumDecomposition2D,
    >>::write_string(&JsonExactDecomposition2D, &decomposition_2d)?;
    let parsed_decomposition_2d =
        SpectrumReader::read_str(&JsonExactDecomposition2D, &decomposition_2d_text)?;
    assert_decomposition_2d_close(&parsed_decomposition_2d, &decomposition_2d);
    Ok(())
}

fn system_fixture() -> SpinHalfSystem {
    SpinHalfSystem::new()
        .with_spin(7.0)
        .with_spin(7.04)
        .with_coupling(0, 1, 8.0)
}

fn spin_options_fixture() -> ExactSpinOptions {
    ExactSpinOptions::new()
        .with_spectrometer_mhz(400.0)
        .with_frequency_tolerance_hz(1.0e-9)
        .with_detected_spin(0)
}

fn spectrum_options_fixture(transition_options: ExactSpinOptions) -> ExactSpectrumOptions {
    ExactSpectrumOptions::new()
        .with_ppm_range(6.95, 7.08)
        .with_points(32)
        .with_transition_options(transition_options)
}

fn spectrum_2d_options_fixture(transition_options: ExactSpinOptions) -> ExactSpectrum2DOptions {
    ExactSpectrum2DOptions::new()
        .with_x_ppm_range(6.95, 7.08)
        .with_y_ppm_range(6.95, 7.08)
        .with_points(16, 16)
        .with_spin_pairs([ExactSpinPair::new(0, 1)])
        .with_transition_options(transition_options)
}

fn assert_decomposition_1d_close(
    parsed: &ExactSpectrumDecomposition1D,
    expected: &ExactSpectrumDecomposition1D,
) {
    assert_eq!(parsed.spectrum.x, expected.spectrum.x);
    assert_values_close(&parsed.spectrum.intensities, &expected.spectrum.intensities);
    assert_transitions_close(&parsed.transitions, &expected.transitions);
    assert_eq!(parsed.contributions.len(), expected.contributions.len());
    for (parsed_contribution, expected_contribution) in
        parsed.contributions.iter().zip(&expected.contributions)
    {
        assert_transition_close(
            &parsed_contribution.transition,
            &expected_contribution.transition,
        );
        assert_values_close(
            &parsed_contribution.intensities,
            &expected_contribution.intensities,
        );
    }
}

fn assert_decomposition_2d_close(
    parsed: &ExactSpectrumDecomposition2D,
    expected: &ExactSpectrumDecomposition2D,
) {
    assert_eq!(parsed.spectrum.x, expected.spectrum.x);
    assert_eq!(parsed.spectrum.y, expected.spectrum.y);
    assert_values_close(&parsed.spectrum.z, &expected.spectrum.z);
    assert_eq!(parsed.contributions.len(), expected.contributions.len());
    for (parsed_contribution, expected_contribution) in
        parsed.contributions.iter().zip(&expected.contributions)
    {
        assert_eq!(parsed_contribution.x_spin, expected_contribution.x_spin);
        assert_eq!(parsed_contribution.y_spin, expected_contribution.y_spin);
        assert_transition_close(
            &parsed_contribution.x_transition,
            &expected_contribution.x_transition,
        );
        assert_transition_close(
            &parsed_contribution.y_transition,
            &expected_contribution.y_transition,
        );
        assert!((parsed_contribution.volume - expected_contribution.volume).abs() < 1.0e-12);
        assert_values_close(&parsed_contribution.z, &expected_contribution.z);
    }
}

fn assert_transitions_close(parsed: &[ExactTransition], expected: &[ExactTransition]) {
    assert_eq!(parsed.len(), expected.len());
    for (parsed_transition, expected_transition) in parsed.iter().zip(expected) {
        assert_transition_close(parsed_transition, expected_transition);
    }
}

fn assert_transition_close(parsed: &ExactTransition, expected: &ExactTransition) {
    assert!((parsed.frequency_hz - expected.frequency_hz).abs() < 1.0e-12);
    assert!((parsed.offset_hz - expected.offset_hz).abs() < 1.0e-12);
    assert!((parsed.center_ppm - expected.center_ppm).abs() < 1.0e-12);
    assert!((parsed.intensity - expected.intensity).abs() < 1.0e-12);
    assert_eq!(parsed.contribution_count, expected.contribution_count);
}

fn assert_values_close(parsed: &[f64], expected: &[f64]) {
    assert_eq!(parsed.len(), expected.len());
    for (parsed_value, expected_value) in parsed.iter().zip(expected) {
        assert!((parsed_value - expected_value).abs() < 1.0e-12);
    }
}
