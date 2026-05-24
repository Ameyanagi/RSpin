use rspin_core::{Spectrum1D, Spectrum2D};

use super::super::{from_json, to_json};
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

#[test]
fn writes_analysis_csv_json() -> anyhow::Result<()> {
    let analysis_1d = rspin_analysis::SpectrumAnalysis1D {
        peaks: vec![rspin_analysis::Peak {
            index: 1,
            x: 1.0,
            intensity: 2.0,
            prominence: 1.5,
            polarity: rspin_analysis::PeakPolarity::Positive,
        }],
        ranges: Vec::new(),
        multiplets: Vec::new(),
        signals: Vec::new(),
    };
    let csv_1d = write_analysis_1d_csv_json(&to_json(&analysis_1d)?)?;
    assert!(csv_1d.contains("# format=RSpin Analysis 1D CSV"));
    assert!(csv_1d.contains("index,x,intensity,prominence,polarity"));

    let analysis_2d = rspin_analysis::SpectrumAnalysis2D {
        zones: vec![rspin_analysis::DetectedZone {
            id: "zone:x0-0:y0-0".to_owned(),
            x_start_index: 0,
            x_end_index: 0,
            y_start_index: 0,
            y_end_index: 0,
            x_from: 1.0,
            x_to: 1.0,
            y_from: 10.0,
            y_to: 10.0,
            centroid_x: 1.0,
            centroid_y: 10.0,
            active_points: 1,
            max_abs_intensity: 3.0,
            sum_intensity: 3.0,
            sum_abs_intensity: 3.0,
        }],
        signals: Vec::new(),
    };
    let csv_2d = write_analysis_2d_csv_json(&to_json(&analysis_2d)?)?;
    assert!(csv_2d.contains("# format=RSpin Analysis 2D CSV"));
    assert!(csv_2d.contains("zone:x0-0:y0-0"));
    Ok(())
}
