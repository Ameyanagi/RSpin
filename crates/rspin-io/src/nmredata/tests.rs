use std::collections::BTreeMap;

use rspin_analysis::AssignmentTarget;
use rspin_core::{Nucleus, RSpinError};

use super::*;
use crate::{SpectrumPathWriter, SpectrumReader, SpectrumWriter};

#[test]
fn parses_header_assignments_couplings_and_spectra() -> anyhow::Result<()> {
    let record = read_nmredata_str(
        r#"
Demo
  RSpin

>  <NMREDATA_VERSION>
1.1\

>  <NMREDATA_LEVEL>
0

>  <NMREDATA_ID>
Doi=10.5281/example\
Spectrum_Location=https://example.test/sample.zip\

>  <NMREDATA_FORMULA>
C2H6O

>  <NMREDATA_SMILES>
CCO

>  <NMREDATA_SOLVENT>
CDCl3

>  <NMREDATA_TEMPERATURE>
298.0 K

>  <NMREDATA_ASSIGNMENT>
A, 48.301, 1 ; carbon assignment\
B, 20.322, 2\
<"Ha,Hb">, 4.802, H1, H2\
Equivalent=A, B\

>  <NMREDATA_J>
Ha, Hb, 7.00\
Equivalent=Ha/Hb, Hc/Hd\

>  <NMREDATA_1D_1H>
Larmor=500.13\
Pulseprogram=zg30\
Spectrum_Location=file:./nmr/10\
4.8000, S=q, N=2, L=Ha, Hb, J=7.00\
7.600-7.200, N=5, L=ArH\

>  <NMREDATA_2D_13C_1J_1H#2>
Larmor=125.75\
CorType=HSQC\
a/C1, I=1.2\
(b,c)/<"C,2">, I=2.4\
$$$$
"#,
    )?;

    let Some(version) = record.version.as_ref() else {
        panic!("version should parse");
    };
    assert_eq!(version.major, 1);
    assert_eq!(version.minor, Some(1));
    assert_eq!(record.level, Some(0));
    assert_eq!(record.formula.as_deref(), Some("C2H6O"));
    assert_eq!(record.smiles.as_deref(), Some("CCO"));
    assert_eq!(record.solvent.as_deref(), Some("CDCl3"));
    assert_some_close(record.temperature_k, 298.0);
    assert_eq!(
        record.id.get("Doi").map(String::as_str),
        Some("10.5281/example")
    );

    assert_eq!(record.assignments.len(), 3);
    assert_eq!(record.assignments[0].label, "A");
    assert_close(record.assignments[0].shift_ppm, 48.301);
    assert_eq!(record.assignments[2].label, "Ha,Hb");
    assert_eq!(record.assignments[2].atom_refs, vec!["H1", "H2"]);
    assert_eq!(record.assignment_equivalences, vec!["Equivalent=A, B"]);

    assert_eq!(record.couplings.len(), 1);
    assert_eq!(record.couplings[0].from_label, "Ha");
    assert_eq!(record.couplings[0].to_label, "Hb");
    assert_close(record.couplings[0].j_hz, 7.0);
    assert_eq!(record.coupling_equivalences.len(), 1);

    assert_eq!(record.spectra.len(), 2);
    let one_d = &record.spectra[0];
    assert_some_close(one_d.larmor_mhz, 500.13);
    assert_eq!(one_d.spectrum_locations, vec!["file:./nmr/10"]);
    assert_eq!(one_d.signals_1d.len(), 2);
    assert_close(one_d.signals_1d[0].from_ppm, 4.8);
    assert_eq!(
        one_d.signals_1d[0].attributes.get("L"),
        Some(&vec!["Ha".to_owned(), "Hb".to_owned()])
    );
    assert_close(one_d.signals_1d[1].from_ppm, 7.6);
    assert_some_close(one_d.signals_1d[1].to_ppm, 7.2);

    let two_d = &record.spectra[1];
    assert_eq!(two_d.tag, "NMREDATA_2D_13C_1J_1H#2");
    assert_eq!(
        two_d.kind,
        NmreDataSpectrumKind::TwoD {
            indirect_label: "13C".to_owned(),
            indirect_nucleus: Some(Nucleus::Carbon13),
            mixing: "1J".to_owned(),
            direct_label: "1H".to_owned(),
            direct_nucleus: Some(Nucleus::Hydrogen1),
        }
    );
    assert_eq!(two_d.signals_2d.len(), 2);
    assert_eq!(two_d.signals_2d[1].left, "(b,c)");
    assert_eq!(two_d.signals_2d[1].right, "C,2");

    assert!(record.tag("NMREDATA_VERSION").is_some());
    assert_eq!(record.tags_named("NMREDATA_2D_13C_1J_1H").len(), 1);
    Ok(())
}

#[test]
fn parses_multiple_records() -> anyhow::Result<()> {
    let records = read_nmredata_records_str(
        r"
>  <NMREDATA_VERSION>
1.0
$$$$
>  <NMREDATA_VERSION>
1.1
$$$$
",
    )?;

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
    Ok(())
}

#[test]
fn writes_and_reads_nmredata_records() -> anyhow::Result<()> {
    let records = read_nmredata_records_str(
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

    let output = write_nmredata_records(&records)?;
    assert_eq!(output.matches("$$$$").count(), 2);

    let reparsed = read_nmredata_records_str(&output)?;
    assert_eq!(reparsed.len(), 2);
    assert_eq!(
        reparsed[0]
            .version
            .as_ref()
            .map(|version| version.raw.as_str()),
        Some("1.0")
    );
    assert_eq!(reparsed[1].formula.as_deref(), Some("C2H6O"));
    Ok(())
}

#[test]
fn converts_nmredata_to_analysis_models() -> anyhow::Result<()> {
    let record = read_nmredata_str(
        r"
>  <NMREDATA_ASSIGNMENT>
H1, 4.200, H1
Hcombo, 3.900, H2, H3

>  <NMREDATA_J>
H1, Hcombo, 7.0
",
    )?;

    let assignments = record.to_assignment_set(Nucleus::Hydrogen1)?;
    assert_eq!(assignments.len(), 2);
    assert!(matches!(
        assignments.assignments[0].target,
        AssignmentTarget::Peak1D { index: 0, x } if (x - 4.2).abs() < 1.0e-12
    ));
    assert_eq!(assignments.assignments[0].atoms[0].id, "H1");
    assert_eq!(assignments.assignments[1].atoms.len(), 2);
    assert_eq!(assignments.assignments[1].atoms[1].id, "H3");

    let graph = nmredata_couplings_to_j_coupling_graph(&record, Nucleus::Hydrogen1)?;
    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.nodes[0].id, "H1");
    assert_eq!(graph.nodes[1].id, "Hcombo");
    assert_eq!(graph.couplings.len(), 1);
    assert_eq!(graph.couplings[0].node_a, "H1");
    assert_eq!(graph.couplings[0].node_b, "Hcombo");
    assert_close(graph.couplings[0].j_hz, 7.0);
    assert_eq!(graph.couplings[0].source.as_deref(), Some("NMReDATA"));

    let analysis = nmredata_to_analysis(&record, Nucleus::Hydrogen1)?;
    assert_eq!(analysis.assignment_set, assignments);
    assert_eq!(analysis.j_coupling_graph, graph);
    let inherent_analysis = record.to_analysis(Nucleus::Hydrogen1)?;
    assert_eq!(inherent_analysis, analysis);
    Ok(())
}

#[test]
fn rejects_invalid_nmredata_analysis_conversions() -> anyhow::Result<()> {
    let duplicate_coupling = read_nmredata_str(
        r"
>  <NMREDATA_J>
H1, H2, 7.0
H2, H1, 7.0
",
    )?;

    let error = duplicate_coupling
        .to_j_coupling_graph(Nucleus::Hydrogen1)
        .expect_err("duplicate coupling pairs should fail");
    assert!(matches!(error, RSpinError::InvalidAssignment { .. }));
    Ok(())
}

#[test]
fn supports_shared_io_traits() -> anyhow::Result<()> {
    let input = r"
>  <NMREDATA_VERSION>
1.1

>  <NMREDATA_ASSIGNMENT>
H1, 4.200, H1
";
    let record = SpectrumReader::read_str(&NmreData, input)?;
    let text = SpectrumWriter::write_string(&NmreData, &record)?;
    let reparsed = read_nmredata_str(&text)?;
    assert_eq!(
        reparsed
            .version
            .as_ref()
            .map(|version| version.raw.as_str()),
        Some("1.1")
    );

    let records = vec![record.clone(), record];
    let records_text =
        <NmreData as SpectrumWriter<[NmreDataRecord]>>::write_string(&NmreData, &records)?;
    assert_eq!(records_text.matches("$$$$").count(), 2);

    let path = std::env::temp_dir().join(format!(
        "rspin-nmredata-trait-{}-{}.sdf",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    <NmreData as SpectrumPathWriter<[NmreDataRecord]>>::write_path(&NmreData, &records, &path)?;
    let from_file = read_nmredata_records_str(&std::fs::read_to_string(&path)?)?;
    assert_eq!(from_file.len(), 2);
    std::fs::remove_file(path)?;
    Ok(())
}

#[test]
fn writes_constructed_record_from_typed_fields() -> anyhow::Result<()> {
    let mut id = BTreeMap::new();
    id.insert("Name".to_owned(), "demo".to_owned());

    let mut spectrum_attributes = BTreeMap::new();
    spectrum_attributes.insert("Larmor".to_owned(), vec!["500.0".to_owned()]);

    let record = NmreDataRecord {
        version: Some(NmreDataVersion {
            raw: "1.1".to_owned(),
            major: 1,
            minor: Some(1),
            qualifier: None,
        }),
        level: Some(0),
        id,
        formula: Some("CH4".to_owned()),
        assignments: vec![NmreDataAssignment {
            label: "H,1".to_owned(),
            shift_ppm: 4.2,
            atom_refs: vec!["H1".to_owned()],
            raw_line: String::new(),
        }],
        couplings: vec![NmreDataCoupling {
            from_label: "H1".to_owned(),
            to_label: "H2".to_owned(),
            j_hz: 7.0,
            raw_line: String::new(),
        }],
        spectra: vec![NmreDataSpectrum {
            tag: "NMREDATA_1D_1H".to_owned(),
            kind: NmreDataSpectrumKind::OneD {
                observed_label: "1H".to_owned(),
                observed_nucleus: Some(Nucleus::Hydrogen1),
            },
            attributes: spectrum_attributes,
            larmor_mhz: Some(500.0),
            spectrum_locations: Vec::new(),
            signals_1d: vec![NmreDataSignal1D {
                from_ppm: 4.2,
                to_ppm: None,
                attributes: BTreeMap::new(),
                items: vec!["H1".to_owned()],
                raw_line: String::new(),
            }],
            signals_2d: Vec::new(),
        }],
        ..NmreDataRecord::default()
    };

    let output = NmreData.write_string(&record)?;
    assert!(output.contains(r#"<"H,1">, 4.2, H1"#));
    let reparsed = read_nmredata_str(&output)?;
    assert_eq!(
        reparsed
            .version
            .as_ref()
            .map(|version| version.raw.as_str()),
        Some("1.1")
    );
    assert_eq!(reparsed.id.get("Name").map(String::as_str), Some("demo"));
    assert_eq!(reparsed.assignments[0].label, "H,1");
    assert_close(reparsed.couplings[0].j_hz, 7.0);
    assert_some_close(reparsed.spectra[0].larmor_mhz, 500.0);
    Ok(())
}

#[test]
fn rejects_invalid_nmredata_writes() {
    let empty_error =
        write_nmredata_records(&[]).expect_err("empty NMReDATA record list should fail");
    assert!(matches!(empty_error, RSpinError::Parse { .. }));

    let invalid_tag = NmreDataRecord {
        tags: vec![NmreDataTag {
            name: String::new(),
            values: vec!["value".to_owned()],
        }],
        ..NmreDataRecord::default()
    };
    let invalid_tag_error =
        write_nmredata_record(&invalid_tag).expect_err("empty SDF tag name should fail");
    assert!(matches!(invalid_tag_error, RSpinError::Parse { .. }));

    let invalid_value = NmreDataRecord {
        tags: vec![NmreDataTag {
            name: "NMREDATA_VERSION".to_owned(),
            values: vec!["1.1\n2.0".to_owned()],
        }],
        ..NmreDataRecord::default()
    };
    let invalid_value_error =
        write_nmredata_record(&invalid_value).expect_err("embedded newlines should fail");
    assert!(matches!(invalid_value_error, RSpinError::Parse { .. }));
}

#[test]
fn parses_version_qualifier() -> anyhow::Result<()> {
    let version = parse_nmredata_version("1.1.rc1")?;

    assert_eq!(version.major, 1);
    assert_eq!(version.minor, Some(1));
    assert_eq!(version.qualifier.as_deref(), Some("rc1"));
    Ok(())
}

#[test]
fn rejects_malformed_version() {
    let error = parse_nmredata_version("release").expect_err("malformed version should fail");
    assert!(matches!(error, RSpinError::Parse { .. }));
}

#[test]
fn rejects_malformed_assignments() {
    let error = read_nmredata_str(
        r"
>  <NMREDATA_ASSIGNMENT>
a, not-a-number, 1
",
    )
    .expect_err("malformed assignment shift should fail");

    assert!(matches!(error, RSpinError::Parse { .. }));
}

#[test]
fn reads_bytes_and_file() -> anyhow::Result<()> {
    let payload = b">  <NMREDATA_VERSION>\n1.1\n";
    let parsed = NmreData.read_bytes(payload)?;
    assert_eq!(
        parsed.version.as_ref().map(|version| version.major),
        Some(1)
    );

    let path = std::env::temp_dir().join(format!(
        "rspin-nmredata-{}-{}.sdf",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    std::fs::write(&path, payload)?;
    let from_file = read_nmredata_file(&path)?;
    assert_eq!(
        from_file.version.as_ref().map(|version| version.minor),
        Some(Some(1))
    );
    std::fs::remove_file(path)?;
    Ok(())
}

#[test]
fn writes_nmredata_file() -> anyhow::Result<()> {
    let record = read_nmredata_str(
        r"
>  <NMREDATA_VERSION>
1.1
",
    )?;
    let path = std::env::temp_dir().join(format!(
        "rspin-nmredata-write-{}-{}.sdf",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));

    write_nmredata_file(&record, &path)?;
    let from_file = read_nmredata_file(&path)?;
    assert_eq!(
        from_file
            .version
            .as_ref()
            .map(|version| version.raw.as_str()),
        Some("1.1")
    );
    std::fs::remove_file(path)?;
    Ok(())
}

fn assert_some_close(actual: Option<f64>, expected: f64) {
    let Some(actual) = actual else {
        panic!("expected Some({expected})");
    };
    assert_close(actual, expected);
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 1.0e-12,
        "{actual} != {expected}"
    );
}
