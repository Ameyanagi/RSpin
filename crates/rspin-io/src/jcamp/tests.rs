use rspin_core::{Axis, Metadata, Nucleus, RSpinError, Spectrum1D, Unit};

use super::*;

#[test]
fn reads_xydata_spectrum() -> anyhow::Result<()> {
    let input = "\
##TITLE=ethyl sample
##JCAMP-DX=5.00
##DATA TYPE=NMR SPECTRUM
##OBSERVE NUCLEUS=1H
##OBSERVE FREQUENCY=400
##XUNITS=PPM
##FIRSTX=10
##LASTX=8
##NPOINTS=4
##XYDATA=(X++(Y..Y))
10 1 2
9 3 4
##END=
";
    let spectrum = read_jcamp_dx_1d(input)?;

    assert_eq!(spectrum.metadata.name.as_deref(), Some("ethyl sample"));
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(spectrum.metadata.frequency_mhz, Some(400.0));
    assert_eq!(
        spectrum
            .metadata
            .properties
            .get("jcamp_dx.version")
            .map(String::as_str),
        Some("5.00")
    );
    assert_axis_close(
        &spectrum.x.values,
        &[10.0, 9.333_333_333_333_334, 8.666_666_666_666_666, 8.0],
    );
    assert_eq!(spectrum.intensities, vec![1.0, 2.0, 3.0, 4.0]);
    Ok(())
}

#[test]
fn parses_jcamp_dx_version_labels() -> anyhow::Result<()> {
    let version = parse_jcamp_dx_version("5.00 $$ comment")?;
    assert_eq!(
        version,
        JcampDxVersion {
            raw: "5.00".to_owned(),
            major: 5,
            minor: 0,
            patch: None,
        }
    );
    assert!(version.is_supported_by_current_reader());

    let patch = parse_jcamp_dx_version("4.24.1")?;
    assert_eq!(patch.major, 4);
    assert_eq!(patch.minor, 24);
    assert_eq!(patch.patch, Some(1));
    assert!(patch.is_supported_by_current_reader());

    let future = parse_jcamp_dx_version("6.0")?;
    assert!(!future.is_supported_by_current_reader());
    let error = future
        .validate_supported_by_current_reader()
        .expect_err("future JCAMP-DX versions should be rejected");
    assert!(matches!(error, RSpinError::Unsupported { .. }));
    Ok(())
}

#[test]
fn rejects_malformed_jcamp_dx_version_labels() {
    let error =
        parse_jcamp_dx_version("5.beta").expect_err("malformed JCAMP-DX version should fail");
    assert!(matches!(error, RSpinError::Parse { .. }));
}

#[test]
fn rejects_unsupported_jcamp_dx_version_label() {
    let input = "\
##TITLE=future
##JCAMP-DX=6.00
##XUNITS=PPM
##FIRSTX=0
##LASTX=1
##NPOINTS=2
##XYDATA=(X++(Y..Y))
0 1 2
##END=
";
    let error = read_jcamp_dx_1d(input).expect_err("unsupported JCAMP-DX version should fail");
    assert!(matches!(error, RSpinError::Unsupported { .. }));
}

#[test]
fn applies_yfactor_to_xydata_values() -> anyhow::Result<()> {
    let input = "\
##TITLE=scaled xydata
##XUNITS=PPM
##FIRSTX=3
##LASTX=1
##NPOINTS=3
##YFACTOR=0.5
##XYDATA=(X++(Y..Y))
3 2 4
2 6
##END=
";
    let spectrum = read_jcamp_dx_1d(input)?;

    assert_axis_close(&spectrum.x.values, &[3.0, 2.0, 1.0]);
    assert_eq!(spectrum.intensities, vec![1.0, 2.0, 3.0]);
    Ok(())
}

#[test]
fn accepts_integer_decimal_npoints_label() -> anyhow::Result<()> {
    let input = "\
##TITLE=decimal count
##XUNITS=PPM
##FIRSTX=4
##LASTX=1
##NPOINTS=4.000000000
##XYDATA=(X++(Y..Y))
0 1 2
2 3 4
##END=
";
    let spectrum = read_jcamp_dx_1d(input)?;

    assert_eq!(spectrum.metadata.name.as_deref(), Some("decimal count"));
    assert_eq!(spectrum.len(), 4);
    assert_axis_close(&spectrum.x.values, &[4.0, 3.0, 2.0, 1.0]);
    assert_eq!(spectrum.intensities, vec![1.0, 2.0, 3.0, 4.0]);
    Ok(())
}

#[test]
fn reads_common_metadata_labels_and_comment_assignments() -> anyhow::Result<()> {
    let input = "\
##TITLE=metadata sample
##ORIGIN=benchtop source $$ free comment
##.SOLVENT NAME=CDCl3
##TEMPERATURE=25.0 $$ Celsius
##.OBSERVE NUCLEUS=^13C
##.OBSERVE FREQUENCY=100.5 $$ MHz
$$solvent=CD2Cl2
##XUNITS=PPM
##FIRSTX=2
##LASTX=1
##NPOINTS=2
##XYDATA=(X++(Y..Y))
0 5 6
##END=
";
    let spectrum = read_jcamp_dx_1d(input)?;

    assert_eq!(spectrum.metadata.name.as_deref(), Some("metadata sample"));
    assert_eq!(spectrum.metadata.origin.as_deref(), Some("benchtop source"));
    assert_eq!(spectrum.metadata.solvent.as_deref(), Some("CD2Cl2"));
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Carbon13));
    assert_eq!(spectrum.metadata.frequency_mhz, Some(100.5));
    assert_close(spectrum.metadata.temperature_k, Some(298.15));
    Ok(())
}

#[test]
fn reads_scaled_xypoints_spectrum() -> anyhow::Result<()> {
    let input = "\
##TITLE=scaled points
##XUNITS=HZ
##NPOINTS=3
##XFACTOR=0.1
##YFACTOR=2
##XYPOINTS=(XY..XY)
10 1 20 2
30 3
##END=
";
    let spectrum = read_jcamp_dx_1d(input)?;

    assert_eq!(spectrum.metadata.name.as_deref(), Some("scaled points"));
    assert_eq!(spectrum.x.unit, Unit::Hertz);
    assert_axis_close(&spectrum.x.values, &[1.0, 2.0, 3.0]);
    assert_eq!(spectrum.intensities, vec![2.0, 4.0, 6.0]);
    Ok(())
}

#[test]
fn reads_peak_table_as_explicit_points() -> anyhow::Result<()> {
    let input = "\
##TITLE=peak table
##XUNITS=(ppm)
##NPOINTS=3
##PEAK TABLE=(XY..XY)
0.5 2
1.0 4
1.5 3
##END
";
    let spectrum = read_jcamp_dx_1d(input)?;

    assert_eq!(spectrum.metadata.name.as_deref(), Some("peak table"));
    assert_eq!(spectrum.x.unit, Unit::Ppm);
    assert_axis_close(&spectrum.x.values, &[0.5, 1.0, 1.5]);
    assert_eq!(spectrum.intensities, vec![2.0, 4.0, 3.0]);
    Ok(())
}

#[test]
fn reads_numeric_data_table_real_and_imaginary_pages() -> anyhow::Result<()> {
    let input = "\
##TITLE=ntuple fid
##NPOINTS=4
##UNITS=SECONDS,ARBITRARY UNITS,ARBITRARY UNITS
##FACTOR=0.1,2,3
##FIRST=0,1,5,1
##LAST=0.3,4,8,2
##PAGE=N=1
##DATA TABLE=(X++(R..R)), XYDATA
0 1 2
2 3 4
##PAGE=N=2
##DATATABLE=(X++(I..I)), XYDATA
0 5 6
2 7 8
##END
";
    let spectrum = read_jcamp_dx_1d(input)?;

    assert_eq!(spectrum.metadata.name.as_deref(), Some("ntuple fid"));
    assert_eq!(spectrum.x.unit, Unit::Seconds);
    assert_axis_close(&spectrum.x.values, &[0.0, 0.1, 0.2, 0.3]);
    assert_eq!(spectrum.intensities, vec![2.0, 4.0, 6.0, 8.0]);
    assert_eq!(spectrum.imaginary, Some(vec![15.0, 18.0, 21.0, 24.0]));
    Ok(())
}

#[test]
fn reads_asdf_sqz_xydata_values() -> anyhow::Result<()> {
    let input = "\
##TITLE=sqz compressed
##XUNITS=PPM
##FIRSTX=0
##LASTX=1
##NPOINTS=2
##XYDATA=(X++(Y..Y))
0E3F4
##END=
";
    let spectrum = read_jcamp_dx_1d(input)?;

    assert_eq!(spectrum.metadata.name.as_deref(), Some("sqz compressed"));
    assert_eq!(spectrum.intensities, vec![53.0, 64.0]);
    Ok(())
}

#[test]
fn reads_asdf_difdup_xydata_values() -> anyhow::Result<()> {
    let input = "\
##TITLE=difdup compressed
##XUNITS=PPM
##FIRSTX=0
##LASTX=9
##NPOINTS=10
##XYDATA=(X++(Y..Y))
0 1JT%jX
##END=
";
    let spectrum = read_jcamp_dx_1d(input)?;

    assert_eq!(spectrum.metadata.name.as_deref(), Some("difdup compressed"));
    assert_eq!(
        spectrum.intensities,
        vec![1.0, 2.0, 3.0, 3.0, 2.0, 1.0, 0.0, -1.0, -2.0, -3.0]
    );
    Ok(())
}

#[test]
fn rejects_odd_xypoints_values() {
    let input = "\
##XYPOINTS=(XY..XY)
1 2 3
##END=
";
    let error = read_jcamp_dx_1d(input).expect_err("odd XYPOINTS should fail");
    assert!(matches!(error, RSpinError::Parse { .. }));
}

#[test]
fn rejects_missing_xydata() {
    let error = read_jcamp_dx_1d("##TITLE=empty\n").expect_err("missing data should fail");
    assert!(matches!(error, RSpinError::Parse { .. }));
}

#[test]
fn writes_readable_xydata_spectrum() -> anyhow::Result<()> {
    let x = Axis::linear("shift", Unit::Ppm, 1.0, 3.0, 3)?;
    let mut metadata = Metadata::named("demo");
    metadata.nucleus = Some(Nucleus::Hydrogen1);
    metadata.frequency_mhz = Some(400.0);
    let spectrum = Spectrum1D::new(x, vec![2.0, 4.0, 8.0], metadata)?;

    let text = write_jcamp_dx_1d(&spectrum)?;
    assert!(text.contains("##TITLE=demo"));
    assert!(text.contains("##OBSERVE NUCLEUS=1H"));
    assert!(text.contains("##XYDATA=(X++(Y..Y))"));
    assert!(!text.contains("##XYPOINTS=(XY..XY)"));

    let parsed = read_jcamp_dx_1d(&text)?;
    assert_eq!(parsed.metadata.name.as_deref(), Some("demo"));
    assert_eq!(parsed.intensities, spectrum.intensities);
    assert_eq!(parsed.x.values, spectrum.x.values);
    Ok(())
}

#[test]
fn writes_sample_metadata_labels() -> anyhow::Result<()> {
    let x = Axis::linear("shift", Unit::Ppm, 1.0, 2.0, 2)?;
    let metadata = Metadata::named("with metadata")
        .with_origin("local export")
        .with_solvent("DMSO-D6")
        .with_temperature_k(298.15)
        .with_nucleus(Nucleus::Carbon13)
        .with_frequency_mhz(100.5);
    let spectrum = Spectrum1D::new(x, vec![2.0, 4.0], metadata)?;

    let text = write_jcamp_dx_1d(&spectrum)?;

    assert!(text.contains("##ORIGIN=local export"));
    assert!(text.contains("##.SOLVENT NAME=DMSO-D6"));
    assert!(text.contains("##TEMPERATURE=298.15 K"));
    let parsed = read_jcamp_dx_1d(&text)?;
    assert_eq!(parsed.metadata.origin.as_deref(), Some("local export"));
    assert_eq!(parsed.metadata.solvent.as_deref(), Some("DMSO-D6"));
    assert_eq!(parsed.metadata.nucleus, Some(Nucleus::Carbon13));
    assert_eq!(parsed.metadata.frequency_mhz, Some(100.5));
    assert_close(parsed.metadata.temperature_k, Some(298.15));
    Ok(())
}

#[test]
fn writes_complex_data_tables_for_uniform_axis() -> anyhow::Result<()> {
    let x = Axis::linear("time", Unit::Seconds, 0.0, 0.3, 4)?;
    let metadata = Metadata::named("complex fid").with_nucleus(Nucleus::Hydrogen1);
    let spectrum = Spectrum1D::new_complex(
        x,
        vec![2.0, 4.0, 6.0, 8.0],
        Some(vec![1.0, 3.0, 5.0, 7.0]),
        metadata,
    )?;

    let text = write_jcamp_dx_1d(&spectrum)?;

    assert!(text.contains("##DATA CLASS=NTUPLES"));
    assert!(text.contains("##DATA TABLE=(X++(R..R)), XYDATA"));
    assert!(text.contains("##DATA TABLE=(X++(I..I)), XYDATA"));
    let parsed = read_jcamp_dx_1d(&text)?;
    assert_eq!(parsed.metadata.name.as_deref(), Some("complex fid"));
    assert_eq!(parsed.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(parsed.x.unit, Unit::Seconds);
    assert_axis_close(&parsed.x.values, &spectrum.x.values);
    assert_eq!(parsed.intensities, spectrum.intensities);
    assert_eq!(parsed.imaginary, spectrum.imaginary);
    Ok(())
}

#[test]
fn rejects_complex_non_uniform_jcamp_export() -> anyhow::Result<()> {
    let x = Axis::new("time", Unit::Seconds, vec![0.0, 0.1, 0.25])?;
    let spectrum = Spectrum1D::new_complex(
        x,
        vec![2.0, 4.0, 6.0],
        Some(vec![1.0, 3.0, 5.0]),
        Metadata::named("complex nonlinear"),
    )?;

    let error = write_jcamp_dx_1d(&spectrum).expect_err("non-uniform complex export should fail");
    assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));
    Ok(())
}

#[test]
fn rejects_non_finite_writer_metadata() -> anyhow::Result<()> {
    let x = Axis::linear("shift", Unit::Ppm, 1.0, 2.0, 2)?;
    let spectrum = Spectrum1D::new(
        x,
        vec![2.0, 4.0],
        Metadata::named("bad").with_temperature_k(f64::NAN),
    )?;

    let error = write_jcamp_dx_1d(&spectrum).expect_err("non-finite metadata should fail");
    assert!(matches!(error, RSpinError::NonFinite { .. }));
    Ok(())
}

#[test]
fn writes_xypoints_for_non_uniform_axis() -> anyhow::Result<()> {
    let x = Axis::new("shift", Unit::Ppm, vec![0.0, 0.4, 1.5, 1.9])?;
    let spectrum = Spectrum1D::new(x, vec![2.0, 4.0, 8.0, 16.0], Metadata::named("nonlinear"))?;

    let text = write_jcamp_dx_1d(&spectrum)?;
    assert!(text.contains("##XYPOINTS=(XY..XY)"));
    assert!(!text.contains("##XYDATA=(X++(Y..Y))"));

    let parsed = read_jcamp_dx_1d(&text)?;
    assert_eq!(parsed.metadata.name.as_deref(), Some("nonlinear"));
    assert_axis_close(&parsed.x.values, &spectrum.x.values);
    assert_eq!(parsed.intensities, spectrum.intensities);
    Ok(())
}

#[test]
fn supports_trait_api() -> anyhow::Result<()> {
    let codec = JcampDx;
    let x = Axis::linear("shift", Unit::Ppm, 0.0, 1.0, 2)?;
    let spectrum = Spectrum1D::new(x, vec![5.0, 6.0], Metadata::default())?;
    let text = codec.write_string(&spectrum)?;
    let parsed = codec.read_str(&text)?;
    assert_eq!(parsed.intensities, vec![5.0, 6.0]);
    Ok(())
}

fn assert_axis_close(actual: &[f64], expected: &[f64]) {
    assert_eq!(actual.len(), expected.len());
    for (left, right) in actual.iter().zip(expected) {
        assert!((left - right).abs() < 1e-12, "{left} != {right}");
    }
}

fn assert_close(actual: Option<f64>, expected: Option<f64>) {
    match (actual, expected) {
        (Some(left), Some(right)) => assert!((left - right).abs() < 1e-12, "{left} != {right}"),
        (left, right) => assert_eq!(left, right),
    }
}
