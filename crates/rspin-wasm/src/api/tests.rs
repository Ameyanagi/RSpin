use rspin_core::{Axis, Metadata, RSpinError, Spectrum1D, Spectrum2D, Unit};
use rspin_io::{NMREDATA_RECORD_JSON_FORMAT, NMREDATA_RECORDS_JSON_FORMAT};

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
    assert!(json.contains("\"format\":\"rspin.spectrum_1d\""));
    let spectrum = spectrum1d_from_json(&json)?;
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
    let spectrum = spectrum1d_from_json(&json)?;

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
fn writes_nmrml_1d_from_json() -> anyhow::Result<()> {
    let spectrum = Spectrum1D::new(
        Axis::linear_ppm(10.0, 8.0, 3)?,
        vec![1.0, -2.0, 3.5],
        Metadata::named("wasm nmrML"),
    )?;
    let text = write_nmrml_1d_json(&to_json(&spectrum)?)?;
    let parsed_json = parse_nmrml_1d_json(&text)?;
    let parsed = spectrum1d_from_json(&parsed_json)?;

    assert!(text.contains("byteFormat=\"float64\""));
    assert_eq!(parsed.x, spectrum.x);
    assert_eq!(parsed.intensities, spectrum.intensities);
    Ok(())
}

#[test]
fn writes_nmrml_2d_from_json() -> anyhow::Result<()> {
    let spectrum = Spectrum2D::new(
        Axis::linear_ppm(10.0, 8.0, 3)?,
        Axis::linear_ppm(120.0, 100.0, 2)?,
        vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
        Metadata::named("wasm 2d"),
    )?;
    let text = write_nmrml_2d_json(&to_json(&spectrum)?)?;
    let parsed_json = parse_nmrml_2d_json(&text)?;
    let parsed = spectrum2d_from_json(&parsed_json)?;

    assert!(text.contains("<spectrumMultiD"));
    assert_eq!(parsed.x, spectrum.x);
    assert_eq!(parsed.y, spectrum.y);
    assert_eq!(parsed.z, spectrum.z);
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
    assert!(json.contains("\"format\":\"rspin.spectrum_2d\""));
    let spectrum = spectrum2d_from_json(&json)?;

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
fn inspects_nmrml_document_info_json() -> anyhow::Result<()> {
    let json = inspect_nmrml_document_json(
        r#"
        <nmrML
            version="v1.0.rc1"
            xmlns="http://nmrml.org/schema"
            xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
            xsi:schemaLocation="http://nmrml.org/schema nmrML.xsd"/>
        "#,
    )?;
    let value: serde_json::Value = from_json(&json)?;

    assert_eq!(value["version"], "v1.0.rc1");
    assert_eq!(value["normalized_version"], "1.0.rc1");
    assert_eq!(value["parsed_version"]["major"], 1);
    assert_eq!(value["parsed_version"]["minor"], 0);
    assert_eq!(value["parsed_version"]["build"], "rc1");
    assert_eq!(value["namespace"], "http://nmrml.org/schema");
    assert_eq!(value["schema_locations"][0]["location"], "nmrML.xsd");
    Ok(())
}

#[test]
fn parses_nmredata_to_json() -> anyhow::Result<()> {
    let json = parse_nmredata_json(
        r"
>  <NMREDATA_VERSION>
1.1

>  <NMREDATA_ASSIGNMENT>
H1, 4.200, H1

>  <NMREDATA_J>
H1, H2, 7.0

>  <NMREDATA_1D_1H>
Larmor=500.0
Spectrum_Location=file:./nmr/10
4.200, L=H1, J=7.0
",
    )?;
    assert!(json.contains(NMREDATA_RECORD_JSON_FORMAT));
    let record = rspin_io::read_nmredata_record_json(&json)?;

    assert_eq!(
        record.version.as_ref().map(|version| version.major),
        Some(1)
    );
    assert_eq!(
        record.version.as_ref().and_then(|version| version.minor),
        Some(1)
    );
    assert_eq!(record.assignments[0].label, "H1");
    assert!((record.assignments[0].shift_ppm - 4.2).abs() < 1.0e-12);
    assert!((record.couplings[0].j_hz - 7.0).abs() < 1.0e-12);
    assert_eq!(
        record.spectra[0].kind,
        rspin_io::NmreDataSpectrumKind::OneD {
            observed_label: "1H".to_owned(),
            observed_nucleus: Some(rspin_core::Nucleus::Hydrogen1),
        }
    );
    assert_eq!(record.spectra[0].larmor_mhz, Some(500.0));
    assert_eq!(
        record.spectra[0].spectrum_locations,
        vec!["file:./nmr/10".to_owned()]
    );
    assert!((record.spectra[0].signals_1d[0].from_ppm - 4.2).abs() < 1.0e-12);
    Ok(())
}

#[test]
fn parses_nmredata_records_to_json() -> anyhow::Result<()> {
    let json = parse_nmredata_records_json(
        r"
>  <NMREDATA_VERSION>
1.0
$$$$
>  <NMREDATA_VERSION>
1.1

>  <NMREDATA_FORMULA>
C2H6O
$$$$
",
    )?;
    assert!(json.contains(NMREDATA_RECORDS_JSON_FORMAT));
    let records = rspin_io::read_nmredata_records_json(&json)?;

    assert_eq!(records.len(), 2);
    assert_eq!(
        records[0]
            .version
            .as_ref()
            .map(|version| version.raw.as_str()),
        Some("1.0")
    );
    assert_eq!(
        records[1]
            .version
            .as_ref()
            .map(|version| version.raw.as_str()),
        Some("1.1")
    );
    assert_eq!(records[1].formula.as_deref(), Some("C2H6O"));
    Ok(())
}

#[test]
fn writes_nmredata_from_json() -> anyhow::Result<()> {
    let json = parse_nmredata_json(
        r"
>  <NMREDATA_VERSION>
1.1

>  <NMREDATA_ASSIGNMENT>
H1, 4.200, H1
",
    )?;

    let text = write_nmredata_json(&json)?;
    assert!(text.contains(">  <NMREDATA_VERSION>"));
    assert!(text.contains("H1, 4.200, H1"));

    let reparsed_json = parse_nmredata_json(&text)?;
    let reparsed = rspin_io::read_nmredata_record_json(&reparsed_json)?;
    assert_eq!(
        reparsed
            .version
            .as_ref()
            .map(|version| version.raw.as_str()),
        Some("1.1")
    );
    assert_eq!(reparsed.assignments[0].label, "H1");

    let legacy_json = serde_json::to_string(&reparsed)?;
    let legacy_text = write_nmredata_json(&legacy_json)?;
    assert!(legacy_text.contains("H1, 4.200, H1"));
    Ok(())
}

#[test]
fn writes_nmredata_records_from_json() -> anyhow::Result<()> {
    let records_json = parse_nmredata_records_json(
        r"
>  <NMREDATA_VERSION>
1.0
$$$$
>  <NMREDATA_VERSION>
1.1
$$$$
",
    )?;

    let text = write_nmredata_records_json(&records_json)?;
    assert_eq!(text.matches("$$$$").count(), 2);
    let reparsed = rspin_io::read_nmredata_records_str(&text)?;
    assert_eq!(
        reparsed[0]
            .version
            .as_ref()
            .map(|version| version.raw.as_str()),
        Some("1.0")
    );
    assert_eq!(
        reparsed[1]
            .version
            .as_ref()
            .map(|version| version.raw.as_str()),
        Some("1.1")
    );

    let legacy_json = serde_json::to_string(&reparsed)?;
    let legacy_text = write_nmredata_records_json(&legacy_json)?;
    assert_eq!(legacy_text.matches("$$$$").count(), 2);
    Ok(())
}

fn nmredata_analysis_fixture_json() -> anyhow::Result<String> {
    Ok(parse_nmredata_json(
        r"
>  <NMREDATA_ASSIGNMENT>
H1, 4.200, H1
Hcombo, 3.900, H2, H3

>  <NMREDATA_J>
H1, Hcombo, 7.0

>  <NMREDATA_1D_1H>
4.200, L=H1
3.900-3.800, L=H2, H3
2.000, orphan

>  <NMREDATA_2D_13C_1J_1H>
H1/C1, I=1.2
Hcombo/C2, I=2.4
",
    )?)
}

#[test]
fn converts_nmredata_analysis_to_json() -> anyhow::Result<()> {
    let record_json = nmredata_analysis_fixture_json()?;
    let assignments_json = nmredata_assignments_to_assignment_set_json(&record_json, "1H")?;
    let assignments: serde_json::Value = from_json(&assignments_json)?;
    assert_eq!(assignments["assignments"].as_array().map(Vec::len), Some(2));
    assert_eq!(assignments["assignments"][0]["atoms"][0]["id"], "H1");
    assert_eq!(
        assignments["assignments"][1]["atoms"][1]["nucleus"],
        "Hydrogen1"
    );
    assert_eq!(
        assignments["assignments"][0]["target"]["Peak1D"]["index"],
        0
    );
    assert_eq!(assignments["assignments"][0]["target"]["Peak1D"]["x"], 4.2);

    let graph_json = nmredata_couplings_to_j_coupling_graph_json(&record_json, "1H")?;
    let graph: serde_json::Value = from_json(&graph_json)?;
    assert_eq!(graph["nodes"].as_array().map(Vec::len), Some(2));
    assert_eq!(graph["nodes"][0]["id"], "H1");
    assert_eq!(graph["couplings"].as_array().map(Vec::len), Some(1));
    assert_eq!(graph["couplings"][0]["node_a"], "H1");
    assert_eq!(graph["couplings"][0]["node_b"], "Hcombo");
    assert_eq!(graph["couplings"][0]["j_hz"], 7.0);

    let analysis_json = nmredata_to_analysis_json(&record_json, "1H")?;
    let analysis: serde_json::Value = from_json(&analysis_json)?;
    assert_eq!(
        analysis["assignment_set"]["assignments"]
            .as_array()
            .map(Vec::len),
        Some(2)
    );
    assert_eq!(
        analysis["assignment_set"]["assignments"][1]["atoms"][1]["nucleus"],
        "Hydrogen1"
    );
    assert_eq!(
        analysis["j_coupling_graph"]["couplings"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );
    assert_eq!(
        analysis["j_coupling_graph"]["couplings"][0]["node_b"],
        "Hcombo"
    );
    assert_eq!(
        analysis["signal_assignment_set"]["assignments"]
            .as_array()
            .map(Vec::len),
        Some(3)
    );
    assert_eq!(
        analysis["signal_assignment_set_2d"]["assignments"]
            .as_array()
            .map(Vec::len),
        Some(2)
    );

    let signal_assignments_json = nmredata_1d_signals_to_assignment_set_json(&record_json, "1H")?;
    let signal_assignments: serde_json::Value = from_json(&signal_assignments_json)?;
    assert_eq!(
        signal_assignments["assignments"].as_array().map(Vec::len),
        Some(3)
    );
    assert_eq!(
        signal_assignments["assignments"][0]["target"]["Peak1D"]["index"],
        0
    );
    assert_eq!(
        signal_assignments["assignments"][1]["target"]["Range1D"]["start_index"],
        1
    );
    assert_eq!(signal_assignments["assignments"][1]["atoms"][1]["id"], "H3");
    assert_eq!(
        signal_assignments["assignments"][2]["atoms"][0]["id"],
        "orphan"
    );

    let signal_assignments_2d_json = nmredata_2d_signals_to_assignment_set_json(&record_json)?;
    let signal_assignments_2d: serde_json::Value = from_json(&signal_assignments_2d_json)?;
    assert_eq!(
        signal_assignments_2d["assignments"]
            .as_array()
            .map(Vec::len),
        Some(2)
    );
    assert_eq!(
        signal_assignments_2d["assignments"][0]["target"]["Zone2D"]["id"],
        "nmredata:2d-signal:0:H1:C1"
    );
    assert_eq!(
        signal_assignments_2d["assignments"][0]["atoms"][0]["nucleus"],
        "Hydrogen1"
    );
    assert_eq!(
        signal_assignments_2d["assignments"][0]["atoms"][1]["nucleus"],
        "Carbon13"
    );
    Ok(())
}

#[test]
fn rejects_invalid_nmredata_analysis_conversion_json() -> anyhow::Result<()> {
    let record_json = parse_nmredata_json(
        r"
>  <NMREDATA_J>
H1, H2, 7.0
H2, H1, 7.0
",
    )?;

    let duplicate_error = nmredata_couplings_to_j_coupling_graph_json(&record_json, "1H")
        .expect_err("duplicate coupling pairs should fail");
    assert!(matches!(
        duplicate_error,
        RSpinError::InvalidAssignment { .. }
    ));
    let analysis_error = nmredata_to_analysis_json(&record_json, "1H")
        .expect_err("duplicate coupling pairs should fail in combined analysis");
    assert!(matches!(
        analysis_error,
        RSpinError::InvalidAssignment { .. }
    ));

    let duplicate_signal_record_json = parse_nmredata_json(
        r"
>  <NMREDATA_1D_1H>
4.200, L=H1, H1
",
    )?;
    let signal_error =
        nmredata_1d_signals_to_assignment_set_json(&duplicate_signal_record_json, "1H")
            .expect_err("duplicate signal labels should fail");
    assert!(matches!(signal_error, RSpinError::InvalidAssignment { .. }));
    let analysis_error = nmredata_to_analysis_json(&duplicate_signal_record_json, "1H")
        .expect_err("duplicate signal labels should fail in combined analysis");
    assert!(matches!(
        analysis_error,
        RSpinError::InvalidAssignment { .. }
    ));

    let nucleus_error = nmredata_assignments_to_assignment_set_json(&record_json, " ")
        .expect_err("empty nucleus labels should fail");
    assert!(matches!(nucleus_error, RSpinError::Parse { .. }));
    Ok(())
}

#[test]
fn rejects_invalid_nmredata_json_write() {
    let error = write_nmredata_json(r#"{"tags":[{"name":"","values":["value"]}]}"#)
        .expect_err("empty SDF tag name should fail");

    assert!(matches!(error, RSpinError::Parse { .. }));
}

#[test]
fn rejects_invalid_nmredata_json_parse() {
    let error = parse_nmredata_json(
        r"
>  <NMREDATA_ASSIGNMENT>
H1, not-a-shift, H1
",
    )
    .expect_err("invalid NMReDATA assignment should fail");

    assert!(matches!(error, RSpinError::Parse { .. }));
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
    let spectrum = spectrum1d_from_json(&json)?;

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
    let parsed = spectrum2d_from_json(&json)?;

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
    assert!(spectrum_json.contains("\"format\":\"rspin.spectrum_1d\""));
    let scaled_json = scale_spectrum_1d_json(&spectrum_json, 0.5)?;
    let scaled = spectrum1d_from_json(&scaled_json)?;
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
