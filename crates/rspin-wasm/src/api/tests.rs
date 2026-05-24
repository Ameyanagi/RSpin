use rspin_core::{Axis, Metadata, Spectrum1D, Spectrum2D, Unit};

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
fn auto_phases_spectrum_json() -> anyhow::Result<()> {
    let spectrum = Spectrum1D::new_complex(
        Axis::linear("shift", Unit::Ppm, 0.0, 2.0, 3)?,
        vec![
            std::f64::consts::FRAC_1_SQRT_2,
            2.0 * std::f64::consts::FRAC_1_SQRT_2,
            std::f64::consts::FRAC_1_SQRT_2,
        ],
        Some(vec![
            std::f64::consts::FRAC_1_SQRT_2,
            2.0 * std::f64::consts::FRAC_1_SQRT_2,
            std::f64::consts::FRAC_1_SQRT_2,
        ]),
        Metadata::default(),
    )?;
    let spectrum_json = to_json(&spectrum)?;
    let result_json = auto_phase_spectrum_1d_json(
        &spectrum_json,
        r#"{"zero_order_min_deg":-90.0,"zero_order_max_deg":90.0,"zero_order_step_deg":5.0,"first_order_min_deg":0.0,"first_order_max_deg":0.0,"first_order_step_deg":1.0,"pivot_fraction":0.5,"imaginary_weight":1.0,"negative_weight":4.0}"#,
    )?;
    let result: AutoPhaseResponseJson = from_json(&result_json)?;

    assert!((result.zero_order_deg + 45.0).abs() < 1.0e-12);
    assert!(result.spectrum.intensities[1] > 1.99);
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
fn detects_ranges_json() -> anyhow::Result<()> {
    let spectrum_json = parse_jcamp_dx_1d_json(
        "\
##TITLE=demo
##FIRSTX=0
##LASTX=6
##XYDATA=(X++(Y..Y))
0 0 2 3 0 -4 -5 0
##END=
",
    )?;
    let ranges_json = detect_ranges_json(
        &spectrum_json,
        r#"{"threshold_abs":1.0,"min_active_points":1,"merge_gap_points":0}"#,
    )?;
    let ranges: Vec<rspin_analysis::DetectedRange> = from_json(&ranges_json)?;

    assert_eq!(ranges.len(), 2);
    assert_eq!(ranges[0].start_index, 1);
    assert_eq!(ranges[0].end_index, 2);
    assert_eq!(ranges[1].start_index, 4);
    assert_eq!(ranges[1].end_index, 5);
    Ok(())
}

#[test]
fn detects_zones_json() -> anyhow::Result<()> {
    let spectrum = Spectrum2D::new(
        Axis::linear("x", Unit::Ppm, 0.0, 2.0, 3)?,
        Axis::linear("y", Unit::Ppm, 0.0, 2.0, 3)?,
        vec![2.0, 0.0, 0.0, 3.0, 0.0, -4.0, 0.0, 0.0, -5.0],
        Metadata::default(),
    )?;
    let zones_json = detect_zones_json(
        &to_json(&spectrum)?,
        r#"{"threshold_abs":1.0,"min_active_points":1,"connectivity":"Four"}"#,
    )?;
    let zones: Vec<rspin_analysis::DetectedZone> = from_json(&zones_json)?;

    assert_eq!(zones.len(), 2);
    assert_eq!(zones[0].id, "zone:x0-0:y0-1");
    assert_eq!(zones[0].active_points, 2);
    assert_eq!(zones[1].id, "zone:x2-2:y1-2");
    assert!((zones[1].max_abs_intensity - 5.0).abs() < 1.0e-12);
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
fn summarizes_signals_json() -> anyhow::Result<()> {
    let spectrum_json = parse_jcamp_dx_1d_json(
        "\
##TITLE=demo
##FIRSTX=0
##LASTX=2
##XYDATA=(X++(Y..Y))
0 0 2 0
##END=
",
    )?;
    let peaks_json = pick_peaks_json(
        &spectrum_json,
        r#"{"min_abs_intensity":1.0,"min_prominence":1.0,"polarity":"Positive"}"#,
    )?;
    let multiplets_json = detect_multiplets_json(
        &spectrum_json,
        &peaks_json,
        r#"{"max_peak_gap_ppm":0.1,"min_peak_count":1,"include_singlets":true,"spectrometer_mhz":400.0}"#,
    )?;
    let signals_json = summarize_signals_1d_json(
        &spectrum_json,
        r#"[{"start_index":0,"end_index":2,"from":0.0,"to":2.0,"active_points":1,"max_abs_intensity":2.0,"area":2.0}]"#,
        &multiplets_json,
        r#"{"assignments":[{"id":"assign:range1d:0-2:H1","target":{"Range1D":{"start_index":0,"end_index":2,"from":0.0,"to":2.0}},"atoms":[{"id":"H1","label":null,"nucleus":"Hydrogen1"}],"confidence":null,"note":null}]}"#,
        r#"{"nodes":[{"id":"H1","label":null,"nucleus":"Hydrogen1"},{"id":"H2","label":null,"nucleus":"Hydrogen1"}],"couplings":[{"id":"j:H1-H2","node_a":"H1","node_b":"H2","j_hz":7.2,"confidence":null,"source":null}]}"#,
        r#"{"include_empty_ranges":true,"include_orphan_multiplets":true}"#,
    )?;
    let signals: Vec<rspin_analysis::SignalSummary1D> = from_json(&signals_json)?;

    assert_eq!(signals.len(), 1);
    assert_eq!(signals[0].assignments.len(), 1);
    assert_eq!(signals[0].couplings.len(), 1);
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
fn integrates_2d_region_json() -> anyhow::Result<()> {
    let spectrum_json = to_json(&Spectrum2D::new(
        Axis::linear("x", Unit::Ppm, 0.0, 2.0, 3)?,
        Axis::linear("y", Unit::Ppm, 0.0, 2.0, 3)?,
        vec![0.0, 1.0, 2.0, 1.0, 2.0, 3.0, 2.0, 3.0, 4.0],
        Metadata::default(),
    )?)?;
    let integral_json = integrate_region_2d_json(
        &spectrum_json,
        r#"{"x_from":0.5,"x_to":1.5,"y_from":0.5,"y_to":1.5}"#,
    )?;
    let integral: rspin_analysis::Integral2D = from_json(&integral_json)?;
    assert!((integral.volume - 2.0).abs() < 1e-12);
    assert_eq!(integral.cells, 4);
    Ok(())
}

#[test]
fn generates_spectrum_matrix_1d_json() -> anyhow::Result<()> {
    let spectra_json = to_json(&vec![
        Spectrum1D::new(
            Axis::linear("x", Unit::Ppm, 0.0, 2.0, 3)?,
            vec![1.0, 2.0, 3.0],
            Metadata::named("a"),
        )?,
        Spectrum1D::new(
            Axis::linear("x", Unit::Ppm, 0.0, 2.0, 2)?,
            vec![10.0, 14.0],
            Metadata::named("b b"),
        )?,
    ])?;
    let matrix_json = generate_spectrum_matrix_1d_json(
        &spectra_json,
        r#"{"target_axis":null,"outside_value":0.0}"#,
    )?;
    let matrix: rspin_analysis::SpectrumMatrix1D = from_json(&matrix_json)?;

    assert_eq!(matrix.shape(), (2, 3));
    assert_eq!(matrix.row_ids, vec!["0:a", "1:b_b"]);
    assert_eq!(matrix.values, vec![1.0, 2.0, 3.0, 10.0, 12.0, 14.0]);
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

#[test]
fn validates_prediction_json() -> anyhow::Result<()> {
    let json = validate_prediction_json(
        r#"{"name":"demo","signals_1d":[{"experiment":"Proton1D","nucleus":"Hydrogen1","delta_ppm":1.0,"intensity":1.0,"confidence":0.9,"assignments":[]}],"correlations_2d":[],"provenance":null}"#,
    )?;
    let prediction: PredictionSet = from_json(&json)?;
    assert_eq!(prediction.signals_1d.len(), 1);
    Ok(())
}

#[test]
fn renders_prediction_json() -> anyhow::Result<()> {
    let spectrum_json = render_prediction_1d_json(
        r#"{"name":"demo","signals_1d":[{"experiment":"Proton1D","nucleus":"Hydrogen1","delta_ppm":1.0,"intensity":1.0,"confidence":0.9,"assignments":["H1"]}],"correlations_2d":[],"provenance":{"source":"fixture","version":null}}"#,
        r#"{"experiment":"Proton1D","nucleus":"Hydrogen1","from_ppm":0.99,"to_ppm":1.01,"points":3,"spectrometer_mhz":400.0,"line_width_hz":1.0,"line_shape":"PseudoVoigt","area_scale":1.0}"#,
    )?;
    let spectrum: Spectrum1D = from_json(&spectrum_json)?;

    assert_eq!(spectrum.len(), 3);
    assert_eq!(spectrum.metadata.frequency_mhz, Some(400.0));
    assert!(spectrum.intensities[1] > spectrum.intensities[0]);
    Ok(())
}

#[test]
fn renders_prediction_2d_json() -> anyhow::Result<()> {
    let spectrum_json = render_prediction_2d_json(
        r#"{"name":"demo","signals_1d":[],"correlations_2d":[{"experiment":"Hsqc","x_nucleus":"Hydrogen1","y_nucleus":"Carbon13","x_ppm":1.0,"y_ppm":20.0,"intensity":1.0,"confidence":0.9,"assignments":["H1-C1"]}],"provenance":{"source":"fixture","version":null}}"#,
        r#"{"experiment":"Hsqc","x_nucleus":"Hydrogen1","y_nucleus":"Carbon13","x_from_ppm":0.99,"x_to_ppm":1.01,"x_points":3,"y_from_ppm":19.9,"y_to_ppm":20.1,"y_points":3,"x_spectrometer_mhz":400.0,"y_spectrometer_mhz":100.0,"x_line_width_hz":1.0,"y_line_width_hz":4.0,"line_shape":"PseudoVoigt","volume_scale":1.0}"#,
    )?;
    let spectrum: Spectrum2D = from_json(&spectrum_json)?;

    assert_eq!(spectrum.shape(), (3, 3));
    assert_eq!(spectrum.metadata.origin, Some("fixture".to_owned()));
    let Some(center) = spectrum.value_at(1, 1) else {
        panic!("center point should exist");
    };
    let Some(edge) = spectrum.value_at(0, 0) else {
        panic!("edge point should exist");
    };
    assert!(center > edge);
    Ok(())
}
