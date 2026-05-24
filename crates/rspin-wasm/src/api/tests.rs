use rspin_core::{Axis, Metadata, Spectrum1D, Spectrum2D, Unit};

use super::*;

mod assignments;
mod buckets;
mod clustering;
mod consensus;
mod matrix;
mod pairwise;
mod pca;
mod prediction;
mod simulation;
mod workflow;

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
fn parses_nmrml_to_json() -> anyhow::Result<()> {
    let json = parse_nmrml_1d_json(
        r#"
        <nmrML version="v1.0.rc1" xmlns="http://nmrml.org/schema">
          <acquisition>
            <acquisition1D>
              <acquisitionParameterSet numberOfScans="1" numberOfSteadyStateScans="0">
                <sampleAcquisitionTemperature value="298.15" unitName="kelvin"/>
                <DirectDimensionParameterSet decoupled="false" numberOfDataPoints="3">
                  <acquisitionNucleus cvRef="CHEBI" accession="CHEBI:49637" name="hydrogen atom"/>
                  <irradiationFrequency value="600.0" unitName="megaHertz"/>
                </DirectDimensionParameterSet>
              </acquisitionParameterSet>
            </acquisition1D>
          </acquisition>
          <spectrumList>
            <spectrum1D id="s1" numberOfDataPoints="3">
              <spectrumDataArray compressed="true" encodedLength="28" byteFormat="float64">eJxjYACBD/YMEHAAQvE4AAAcPwI8</spectrumDataArray>
              <xAxis unitName="parts per million" startValue="10.0" endValue="8.0"/>
            </spectrum1D>
          </spectrumList>
        </nmrML>
        "#,
    )?;
    let spectrum: Spectrum1D = from_json(&json)?;

    assert_eq!(spectrum.x.unit, Unit::Ppm);
    assert_eq!(spectrum.x.values, vec![10.0, 9.0, 8.0]);
    assert_eq!(spectrum.intensities, vec![1.0, -2.0, 3.5]);
    assert_eq!(
        spectrum.metadata.nucleus,
        Some(rspin_core::Nucleus::Hydrogen1)
    );
    Ok(())
}

#[test]
fn parses_nmrml_2d_to_json() -> anyhow::Result<()> {
    let json = parse_nmrml_2d_json(
        r#"
        <nmrML version="v1.0.rc1" id="two-d" xmlns="http://nmrml.org/schema">
          <acquisition>
            <acquisitionMultiD>
              <acquisitionParameterSet numberOfScans="1" numberOfSteadyStateScans="0">
                <sampleAcquisitionTemperature value="25.0" unitName="degree celsius"/>
                <directDimensionParameterSet decoupled="false" numberOfDataPoints="2">
                  <acquisitionNucleus cvRef="NMR" accession="NMR:1400151" name="1H"/>
                  <irradiationFrequency value="600.0" unitName="megaHertz"/>
                  <sweepWidth value="2.0" unitName="hertz"/>
                </directDimensionParameterSet>
                <indirectDimensionParameterSet decoupled="false" numberOfDataPoints="2">
                  <acquisitionNucleus cvRef="NMR" accession="NMR:1400154" name="13C"/>
                  <irradiationFrequency value="150.0" unitName="megaHertz"/>
                  <sweepWidth value="4.0" unitName="hertz"/>
                </indirectDimensionParameterSet>
              </acquisitionParameterSet>
              <fidData compressed="true" encodedLength="44" byteFormat="complex64">
                eJxjYGiwZ2Bo2M/AwOAAxAeAFJB2ANINQLrhAABd6gZ/
              </fidData>
            </acquisitionMultiD>
          </acquisition>
        </nmrML>
        "#,
    )?;
    let spectrum: Spectrum2D = from_json(&json)?;

    assert_eq!(spectrum.shape(), (2, 2));
    assert_eq!(spectrum.x.unit, Unit::Seconds);
    assert_eq!(spectrum.y.unit, Unit::Seconds);
    assert_eq!(spectrum.x.values, vec![0.0, 0.5]);
    assert_eq!(spectrum.y.values, vec![0.0, 0.25]);
    assert_eq!(spectrum.z, vec![1.0, 2.0, 3.0, 4.0]);
    assert_eq!(spectrum.imaginary, Some(vec![-1.0, -2.0, -3.0, -4.0]));
    assert_eq!(
        spectrum.metadata.nucleus,
        Some(rspin_core::Nucleus::Hydrogen1)
    );
    Ok(())
}

#[test]
fn parses_auto_detected_1d_text_to_json() -> anyhow::Result<()> {
    let json = parse_spectrum_1d_text_json(
        "\
# name=auto one
# x_unit=PPM
x,intensity
0.0,1.0
1.0,2.0
",
    )?;
    let spectrum: Spectrum1D = from_json(&json)?;

    assert_eq!(spectrum.metadata.name.as_deref(), Some("auto one"));
    assert_eq!(spectrum.x.unit, Unit::Ppm);
    assert_eq!(spectrum.x.values, vec![0.0, 1.0]);
    assert_eq!(spectrum.intensities, vec![1.0, 2.0]);
    Ok(())
}

#[test]
fn parses_auto_detected_2d_text_to_json() -> anyhow::Result<()> {
    let spectrum = Spectrum2D::new(
        Axis::linear("x", Unit::Ppm, 0.0, 1.0, 2)?,
        Axis::linear("y", Unit::Ppm, 10.0, 10.0, 1)?,
        vec![1.0, 2.0],
        Metadata::named("auto two"),
    )?;
    let input = to_json(&spectrum)?;
    let json = parse_spectrum_2d_text_json(&input)?;
    let parsed: Spectrum2D = from_json(&json)?;

    assert_eq!(parsed, spectrum);
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
fn validates_assignment_set_json() -> anyhow::Result<()> {
    let assignments_json = validate_assignment_set_json(
        r#"{"assignments":[{"id":"assign:peak1d:2:H2","target":{"Peak1D":{"index":2,"x":7.12}},"atoms":[{"id":"H2","label":null,"nucleus":"Hydrogen1"}],"confidence":0.9,"note":null}]}"#,
    )?;
    let assignments: rspin_analysis::AssignmentSet = from_json(&assignments_json)?;

    assert_eq!(assignments.len(), 1);
    assert_eq!(assignments.assignments[0].id, "assign:peak1d:2:H2");
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
fn summarizes_2d_signals_json() -> anyhow::Result<()> {
    let spectrum_json = to_json(&Spectrum2D::new(
        Axis::linear("x", Unit::Ppm, 0.0, 2.0, 3)?,
        Axis::linear("y", Unit::Ppm, 0.0, 2.0, 3)?,
        vec![0.0, 0.0, 0.0, 0.0, 5.0, 0.0, 0.0, 0.0, 0.0],
        Metadata::default(),
    )?)?;
    let zones_json = detect_zones_json(
        &spectrum_json,
        r#"{"threshold_abs":1.0,"min_active_points":1,"connectivity":"Four"}"#,
    )?;
    let signals_json = summarize_signals_2d_json(
        &spectrum_json,
        &zones_json,
        r#"{"assignments":[{"id":"assign:zone2d:center:H1","target":{"Zone2D":{"id":"zone:x1-1:y1-1"}},"atoms":[{"id":"H1","label":null,"nucleus":"Hydrogen1"}],"confidence":null,"note":null}]}"#,
        r#"{"include_unassigned_zones":true}"#,
    )?;
    let signals: Vec<rspin_analysis::SignalSummary2D> = from_json(&signals_json)?;

    assert_eq!(signals.len(), 1);
    assert_eq!(signals[0].id, "signal2d:zone:x1-1:y1-1");
    assert_eq!(signals[0].assignments.len(), 1);
    assert_eq!(signals[0].atoms.len(), 1);
    assert!((signals[0].center_x - 1.0).abs() < 1e-12);
    assert!((signals[0].center_y - 1.0).abs() < 1e-12);
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
