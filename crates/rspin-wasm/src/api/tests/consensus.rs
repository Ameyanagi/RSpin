use rspin_core::{Axis, Metadata, Spectrum1D, Unit};

use super::super::{
    detect_consensus_peaks_1d_json, detect_consensus_ranges_1d_json, from_json, to_json,
};

#[test]
fn detects_consensus_peaks_json() -> anyhow::Result<()> {
    let spectra = vec![
        spectrum("a", 0.0, &[0.0, 5.0, 0.0])?,
        spectrum("b", 0.02, &[0.0, 4.0, 0.0])?,
    ];

    let result_json = detect_consensus_peaks_1d_json(
        &to_json(&spectra)?,
        r#"{"max_shift":0.05,"min_spectrum_count":2,"peak_options":{"min_abs_intensity":1.0,"min_prominence":0.0,"polarity":"Positive"}}"#,
    )?;
    let result: Vec<rspin_analysis::ConsensusPeak1D> = from_json(&result_json)?;

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].id, "consensus1d:0");
    assert_eq!(result[0].spectrum_count, 2);
    assert_eq!(result[0].members[0].row_id, "0:a");
    assert_eq!(result[0].members[1].row_id, "1:b");
    Ok(())
}

#[test]
fn detects_consensus_ranges_json() -> anyhow::Result<()> {
    let spectra = vec![
        spectrum("a", 0.0, &[0.0, 2.0, 3.0, 0.0])?,
        spectrum("b", 0.02, &[0.0, 4.0, 5.0, 0.0])?,
    ];

    let result_json = detect_consensus_ranges_1d_json(
        &to_json(&spectra)?,
        r#"{"max_gap":0.05,"min_spectrum_count":2,"range_options":{"threshold_abs":1.0,"min_active_points":1,"merge_gap_points":0}}"#,
    )?;
    let result: Vec<rspin_analysis::ConsensusRange1D> = from_json(&result_json)?;

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].id, "consensus-range1d:0");
    assert_eq!(result[0].spectrum_count, 2);
    assert_eq!(result[0].members[0].row_id, "0:a");
    assert_eq!(result[0].members[1].row_id, "1:b");
    Ok(())
}

fn spectrum(name: &str, offset: f64, intensities: &[f64]) -> anyhow::Result<Spectrum1D> {
    let end = offset + f64::from(u32::try_from(intensities.len() - 1)?);
    Ok(Spectrum1D::new(
        Axis::linear("x", Unit::Ppm, offset, end, intensities.len())?,
        intensities.to_vec(),
        Metadata::named(name),
    )?)
}
