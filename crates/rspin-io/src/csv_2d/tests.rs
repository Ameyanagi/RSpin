use rspin_core::{Axis, Metadata, Nucleus, Unit};

use super::*;

#[test]
fn round_trips_real_2d_spectrum_with_trait_api() -> anyhow::Result<()> {
    let mut metadata = Metadata::named("map");
    metadata.nucleus = Some(Nucleus::Carbon13);
    metadata.frequency_mhz = Some(100.0);
    let spectrum = Spectrum2D::new(
        Axis::linear("direct", Unit::Ppm, 10.0, 8.0, 3)?,
        Axis::linear("indirect", Unit::Hertz, 20.0, 21.0, 2)?,
        vec![1.0, 2.0, 3.0, -4.0, -5.0, -6.0],
        metadata,
    )?;

    let codec = CsvSpectrum2D;
    let text = codec.write_string(&spectrum)?;
    let parsed = codec.read_str(&text)?;

    assert_eq!(parsed.metadata.name.as_deref(), Some("map"));
    assert_eq!(parsed.metadata.nucleus, Some(Nucleus::Carbon13));
    assert_eq!(parsed.metadata.frequency_mhz, Some(100.0));
    assert_eq!(parsed.x.unit, Unit::Ppm);
    assert_eq!(parsed.y.unit, Unit::Hertz);
    assert_eq!(parsed.x.values, spectrum.x.values);
    assert_eq!(parsed.y.values, spectrum.y.values);
    assert_eq!(parsed.z, spectrum.z);
    assert_eq!(parsed.imaginary, None);
    Ok(())
}

#[test]
fn reads_complex_long_table() -> anyhow::Result<()> {
    let input = "\
# name=complex map
# x_unit=PPM
# y_unit=HZ
x,y,intensity,imaginary
1,10,2,0.5
2,10,3,-0.25
1,20,4,1.5
2,20,5,-1
";
    let spectrum = read_spectrum2d_csv(input)?;

    assert_eq!(spectrum.metadata.name.as_deref(), Some("complex map"));
    assert_eq!(spectrum.x.unit, Unit::Ppm);
    assert_eq!(spectrum.y.unit, Unit::Hertz);
    assert_eq!(spectrum.x.values, vec![1.0, 2.0]);
    assert_eq!(spectrum.y.values, vec![10.0, 20.0]);
    assert_eq!(spectrum.z, vec![2.0, 3.0, 4.0, 5.0]);
    assert_eq!(spectrum.imaginary, Some(vec![0.5, -0.25, 1.5, -1.0]));
    Ok(())
}

#[test]
fn reads_data_without_header() -> anyhow::Result<()> {
    let spectrum = read_spectrum2d_csv("1,10,2\n2,10,3\n1,20,4\n2,20,5\n")?;

    assert_eq!(spectrum.shape(), (2, 2));
    assert_eq!(spectrum.x.values, vec![1.0, 2.0]);
    assert_eq!(spectrum.y.values, vec![10.0, 20.0]);
    assert_eq!(spectrum.z, vec![2.0, 3.0, 4.0, 5.0]);
    Ok(())
}

#[test]
fn rejects_incomplete_rows() {
    let error = read_spectrum2d_csv("x,y,intensity\n1,10,2\n2,10,3\n1,20,4\n")
        .expect_err("incomplete 2D row should fail");
    assert!(matches!(error, RSpinError::Parse { .. }));
}

#[test]
fn rejects_inconsistent_x_order() {
    let error = read_spectrum2d_csv("x,y,intensity\n1,10,2\n2,10,3\n2,20,4\n1,20,5\n")
        .expect_err("inconsistent x order should fail");
    assert!(matches!(error, RSpinError::Parse { .. }));
}

#[test]
fn rejects_inconsistent_imaginary_columns() {
    let error = read_spectrum2d_csv("x,y,intensity\n1,10,2\n2,10,3,0.5\n")
        .expect_err("mixed imaginary columns should fail");
    assert!(matches!(error, RSpinError::Parse { .. }));
}
