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
    assert_axis_close(
        &spectrum.x.values,
        &[10.0, 9.333_333_333_333_334, 8.666_666_666_666_666, 8.0],
    );
    assert_eq!(spectrum.intensities, vec![1.0, 2.0, 3.0, 4.0]);
    Ok(())
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
