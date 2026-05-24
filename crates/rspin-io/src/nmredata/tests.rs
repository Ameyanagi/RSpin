use rspin_core::{Nucleus, RSpinError};

use super::*;

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
