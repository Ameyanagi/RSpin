use rspin_core::{Spectrum1D, Spectrum2D};

use super::super::from_json;
use super::*;

#[test]
fn parses_and_writes_1d_csv_json() -> anyhow::Result<()> {
    let csv = "\
# name=one
# x_unit=PPM
x,intensity,imaginary
1,2,0.5
2,3,-0.25
";
    let spectrum_json = parse_spectrum_1d_csv_json(csv)?;
    let spectrum: Spectrum1D = from_json(&spectrum_json)?;

    assert_eq!(spectrum.metadata.name.as_deref(), Some("one"));
    assert_eq!(spectrum.x.values, vec![1.0, 2.0]);
    assert_eq!(spectrum.intensities, vec![2.0, 3.0]);
    assert_eq!(spectrum.imaginary, Some(vec![0.5, -0.25]));

    let written = write_spectrum_1d_csv_json(&spectrum_json)?;
    let reparsed_json = parse_spectrum_1d_csv_json(&written)?;
    let reparsed: Spectrum1D = from_json(&reparsed_json)?;
    assert_eq!(reparsed, spectrum);
    Ok(())
}

#[test]
fn parses_and_writes_2d_csv_json() -> anyhow::Result<()> {
    let csv = "\
# name=two
# x_unit=PPM
# y_unit=HZ
x,y,intensity,imaginary
1,10,2,0.5
2,10,3,-0.25
1,20,4,1.5
2,20,5,-1
";
    let spectrum_json = parse_spectrum_2d_csv_json(csv)?;
    let spectrum: Spectrum2D = from_json(&spectrum_json)?;

    assert_eq!(spectrum.metadata.name.as_deref(), Some("two"));
    assert_eq!(spectrum.shape(), (2, 2));
    assert_eq!(spectrum.x.values, vec![1.0, 2.0]);
    assert_eq!(spectrum.y.values, vec![10.0, 20.0]);
    assert_eq!(spectrum.z, vec![2.0, 3.0, 4.0, 5.0]);
    assert_eq!(spectrum.imaginary, Some(vec![0.5, -0.25, 1.5, -1.0]));

    let written = write_spectrum_2d_csv_json(&spectrum_json)?;
    let reparsed_json = parse_spectrum_2d_csv_json(&written)?;
    let reparsed: Spectrum2D = from_json(&reparsed_json)?;
    assert_eq!(reparsed, spectrum);
    Ok(())
}

#[test]
fn rejects_invalid_csv_and_json() {
    let error = parse_spectrum_2d_csv_json("x,y,intensity\n1,10,2\n2,10,3\n1,20,4\n")
        .expect_err("incomplete 2D CSV should fail");
    assert!(error.to_string().contains("expected"));

    let error = write_spectrum_1d_csv_json("{").expect_err("invalid JSON should fail");
    assert!(error.to_string().contains("JSON"));
}
