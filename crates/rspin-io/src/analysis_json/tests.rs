use rspin_analysis::{
    AnalyzeSpectrum1D, AnalyzeSpectrum2D, PeakPickOptions, RangeDetectionOptions,
    ZoneDetectionOptions,
};
use rspin_core::{Axis, Metadata, RSpinError, Spectrum1D, Spectrum2D, Unit};

use crate::{
    JsonAnalysis1D, JsonAnalysis2D, SpectrumReader, SpectrumWriter, read_analysis1d_json,
    read_analysis2d_json, write_analysis1d_json, write_analysis2d_json,
};

#[test]
fn round_trips_one_dimensional_analysis_json() -> anyhow::Result<()> {
    let spectrum = Spectrum1D::new(
        Axis::linear_ppm(0.0, 4.0, 5)?,
        vec![0.0, 2.0, 0.0, 1.5, 0.0],
        Metadata::named("analysis"),
    )?;
    let analysis = spectrum
        .analyze()
        .with_peak_options(PeakPickOptions::new().with_min_abs_intensity(1.0))
        .with_range_options(RangeDetectionOptions::new().with_threshold_abs(1.0))
        .run()?;

    let text = write_analysis1d_json(&analysis)?;
    let parsed = read_analysis1d_json(&text)?;

    assert!(text.contains("peaks"));
    assert_eq!(parsed, analysis);

    let codec = JsonAnalysis1D;
    assert_eq!(codec.read_str(&codec.write_string(&analysis)?)?, analysis);
    Ok(())
}

#[test]
fn round_trips_two_dimensional_analysis_json() -> anyhow::Result<()> {
    let spectrum = Spectrum2D::new(
        Axis::linear("1H", Unit::Ppm, 0.0, 2.0, 3)?,
        Axis::linear("13C", Unit::Ppm, 0.0, 2.0, 3)?,
        vec![2.0, 0.0, 0.0, 1.5, 0.0, -3.0, 0.0, 0.0, -4.0],
        Metadata::named("analysis-2d"),
    )?;
    let analysis = spectrum
        .analyze()
        .with_zone_options(ZoneDetectionOptions::new().with_threshold_abs(1.0))
        .run()?;

    let text = write_analysis2d_json(&analysis)?;
    let parsed = read_analysis2d_json(&text)?;

    assert!(text.contains("zones"));
    assert_eq!(parsed, analysis);

    let codec = JsonAnalysis2D;
    assert_eq!(codec.read_str(&codec.write_string(&analysis)?)?, analysis);
    Ok(())
}

#[test]
fn rejects_invalid_analysis_json() {
    let error = read_analysis1d_json("{").expect_err("invalid JSON should fail");
    assert!(matches!(error, RSpinError::Parse { .. }));
}
