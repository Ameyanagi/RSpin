use rspin_core::Spectrum1D;

use super::*;

#[test]
fn parses_jcamp_to_json() -> anyhow::Result<()> {
    let json = parse_jcamp_dx_1d_json(
        "\
##TITLE=demo
##XUNITS=PPM
##FIRSTX=0
##LASTX=2
##XYDATA=(X++(Y..Y))
0 1 2 3
##END=
",
    )?;
    let spectrum: Spectrum1D = from_json(&json)?;
    assert_eq!(spectrum.len(), 3);
    Ok(())
}

#[test]
fn scales_spectrum_json() -> anyhow::Result<()> {
    let spectrum_json = parse_jcamp_dx_1d_json(
        "\
##TITLE=demo
##FIRSTX=0
##LASTX=1
##XYDATA=(X++(Y..Y))
0 2 4
##END=
",
    )?;
    let scaled_json = scale_spectrum_1d_json(&spectrum_json, 0.5)?;
    let scaled: Spectrum1D = from_json(&scaled_json)?;
    assert_eq!(scaled.intensities, vec![1.0, 2.0]);
    Ok(())
}

#[test]
fn picks_peaks_json() -> anyhow::Result<()> {
    let spectrum_json = parse_jcamp_dx_1d_json(
        "\
##TITLE=demo
##FIRSTX=0
##LASTX=4
##XYDATA=(X++(Y..Y))
0 0 2 0 3 0
##END=
",
    )?;
    let peaks_json = pick_peaks_json(
        &spectrum_json,
        r#"{"min_abs_intensity":1.0,"min_prominence":1.0,"polarity":"Positive"}"#,
    )?;
    let peaks: Vec<rspin_analysis::Peak> = from_json(&peaks_json)?;
    assert_eq!(peaks.len(), 2);
    Ok(())
}

#[test]
fn optimizes_peaks_json() -> anyhow::Result<()> {
    let spectrum_json = parse_jcamp_dx_1d_json(
        "\
##TITLE=demo
##FIRSTX=0
##LASTX=4
##XYDATA=(X++(Y..Y))
0 0 0.75 1 0.75 0
##END=
",
    )?;
    let peaks_json = pick_peaks_json(
        &spectrum_json,
        r#"{"min_abs_intensity":0.0,"min_prominence":0.0,"polarity":"Positive"}"#,
    )?;
    let optimized_json = optimize_peaks_json(
        &spectrum_json,
        &peaks_json,
        r#"{"require_vertex_inside":true,"require_matching_curvature":true}"#,
    )?;
    let optimized: Vec<rspin_analysis::OptimizedPeak> = from_json(&optimized_json)?;

    assert_eq!(optimized.len(), 1);
    assert!(optimized[0].optimized);
    Ok(())
}

#[test]
fn detects_multiplets_json() -> anyhow::Result<()> {
    let spectrum_json = parse_jcamp_dx_1d_json(
        "\
##TITLE=demo
##OBSERVE FREQUENCY=400
##FIRSTX=0
##LASTX=8
##XYDATA=(X++(Y..Y))
0 0 1 0 0.8 0 0 0 2 0
##END=
",
    )?;
    let peaks_json = pick_peaks_json(
        &spectrum_json,
        r#"{"min_abs_intensity":0.5,"min_prominence":0.5,"polarity":"Positive"}"#,
    )?;
    let multiplets_json = detect_multiplets_json(
        &spectrum_json,
        &peaks_json,
        r#"{"max_peak_gap_ppm":2.1,"min_peak_count":1,"include_singlets":true,"spectrometer_mhz":400.0}"#,
    )?;
    let multiplets: Vec<rspin_analysis::DetectedMultiplet> = from_json(&multiplets_json)?;

    assert_eq!(multiplets.len(), 2);
    assert_eq!(multiplets[0].kind, rspin_analysis::MultipletKind::Doublet);
    assert_eq!(multiplets[1].kind, rspin_analysis::MultipletKind::Singlet);
    Ok(())
}

#[test]
fn validates_j_coupling_graph_json() -> anyhow::Result<()> {
    let graph_json = validate_j_coupling_graph_json(
        r#"{"nodes":[{"id":"H1","label":"H-1","nucleus":"Hydrogen1"},{"id":"H2","label":null,"nucleus":"Hydrogen1"}],"couplings":[{"id":"j:H1-H2","node_a":"H1","node_b":"H2","j_hz":7.2,"confidence":0.9,"source":"measured"}]}"#,
    )?;
    let graph: rspin_analysis::JCouplingGraph = from_json(&graph_json)?;

    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.couplings.len(), 1);
    Ok(())
}

#[test]
fn integrates_region_json() -> anyhow::Result<()> {
    let spectrum_json = parse_jcamp_dx_1d_json(
        "\
##TITLE=demo
##FIRSTX=0
##LASTX=2
##XYDATA=(X++(Y..Y))
0 0 1 2
##END=
",
    )?;
    let integral_json = integrate_region_json(&spectrum_json, r#"{"from":0.0,"to":2.0}"#)?;
    let integral: rspin_analysis::Integral = from_json(&integral_json)?;
    assert!((integral.area - 2.0).abs() < 1e-12);
    Ok(())
}

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
        r#"{"from_ppm":1.99,"to_ppm":2.01,"points":11,"area":2.0,"line_width_hz":1.0,"line_shape":"Lorentzian","transition_options":{"spectrometer_mhz":400.0,"intensity_threshold":1e-12,"frequency_tolerance_hz":1e-9,"max_spins":10}}"#,
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

#[test]
fn validates_prediction_json() -> anyhow::Result<()> {
    let json = validate_prediction_json(
        r#"{"name":"demo","signals_1d":[{"experiment":"Proton1D","nucleus":"Hydrogen1","delta_ppm":1.0,"intensity":1.0,"confidence":0.9,"assignments":[]}],"correlations_2d":[],"provenance":null}"#,
    )?;
    let prediction: PredictionSet = from_json(&json)?;
    assert_eq!(prediction.signals_1d.len(), 1);
    Ok(())
}
