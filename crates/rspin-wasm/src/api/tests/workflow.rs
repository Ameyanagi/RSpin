use rspin_core::{Axis, Metadata, Spectrum1D, Spectrum2D, Unit};
use rspin_io::{ANALYSIS_1D_JSON_FORMAT, ANALYSIS_2D_JSON_FORMAT};

use super::*;

#[test]
fn analyzes_spectrum_1d_json() -> anyhow::Result<()> {
    let spectrum = Spectrum1D::new(
        Axis::linear("shift", Unit::Ppm, 0.0, 4.0, 5)?,
        vec![0.0, 2.0, 0.0, 1.5, 0.0],
        Metadata::named("analysis-json"),
    )?;
    let analysis_json = analyze_spectrum_1d_json(
        &to_json(&spectrum)?,
        r#"{"peak_options":{"min_abs_intensity":1.0,"min_prominence":0.0,"polarity":"Positive"},"peak_optimization_options":{"require_vertex_inside":true,"require_matching_curvature":true},"range_options":{"threshold_abs":1.0,"min_active_points":1,"merge_gap_points":0},"multiplet_options":{"max_peak_gap_ppm":1.1,"min_peak_count":1,"include_singlets":true,"spectrometer_mhz":400.0},"signal_options":{"include_empty_ranges":true,"include_orphan_multiplets":true}}"#,
    )?;
    assert!(analysis_json.contains(ANALYSIS_1D_JSON_FORMAT));
    let analysis = rspin_io::read_analysis1d_json(&analysis_json)?;

    assert_eq!(analysis.peaks.len(), 2);
    assert_eq!(analysis.optimized_peaks.len(), 2);
    assert_eq!(analysis.ranges.len(), 2);
    assert_eq!(analysis.integrals.len(), 2);
    assert_eq!(analysis.signals.len(), 2);
    Ok(())
}

#[test]
fn analyzes_spectrum_2d_json() -> anyhow::Result<()> {
    let spectrum = Spectrum2D::new(
        Axis::linear("1H", Unit::Ppm, 0.0, 2.0, 3)?,
        Axis::linear("13C", Unit::Ppm, 0.0, 2.0, 3)?,
        vec![2.0, 0.0, 0.0, 1.5, 0.0, -3.0, 0.0, 0.0, -4.0],
        Metadata::named("analysis-json-2d"),
    )?;
    let analysis_json = analyze_spectrum_2d_json(
        &to_json(&spectrum)?,
        r#"{"zone_options":{"threshold_abs":1.0,"min_active_points":1,"connectivity":"Four"},"signal_options":{"include_unassigned_zones":true}}"#,
    )?;
    assert!(analysis_json.contains(ANALYSIS_2D_JSON_FORMAT));
    let analysis = rspin_io::read_analysis2d_json(&analysis_json)?;

    assert_eq!(analysis.zones.len(), 2);
    assert_eq!(analysis.integrals.len(), 2);
    assert_eq!(analysis.signals.len(), 2);
    Ok(())
}
