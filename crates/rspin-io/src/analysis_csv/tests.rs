use rspin_analysis::{
    AnalyzeSpectrum1D, AnalyzeSpectrum2D, AssignedAtom, Assignment, AssignmentSet,
    AssignmentTarget, CouplingNode, JCoupling, JCouplingGraph, PeakPickOptions,
    RangeDetectionOptions, ZoneDetectionOptions,
};
use rspin_core::{Axis, Metadata, Nucleus, RSpinError, Spectrum1D, Spectrum2D, Unit};

use crate::{CsvAnalysis1D, CsvAnalysis2D, SpectrumWriter, write_analysis1d_csv};

use super::write_analysis2d_csv;

#[test]
fn writes_one_dimensional_analysis_csv() -> anyhow::Result<()> {
    let spectrum = Spectrum1D::new(
        Axis::linear_ppm(0.0, 4.0, 5)?,
        vec![0.0, 2.0, 0.0, 1.5, 0.0],
        Metadata::named("analysis"),
    )?;
    let assignments = AssignmentSet::new(vec![Assignment::deterministic(
        AssignmentTarget::Range1D {
            start_index: 1,
            end_index: 1,
            from: 1.0,
            to: 1.0,
        },
        vec![AssignedAtom::new("H,1", Nucleus::Hydrogen1)],
    )?])?;
    let graph = JCouplingGraph::new(
        vec![
            CouplingNode::new("H,1", Nucleus::Hydrogen1),
            CouplingNode::new("H2", Nucleus::Hydrogen1),
        ],
        vec![JCoupling::deterministic("H,1", "H2", 7.2)?],
    )?;
    let analysis = spectrum
        .analyze()
        .with_peak_options(PeakPickOptions::new().with_min_abs_intensity(1.0))
        .with_range_options(RangeDetectionOptions::new().with_threshold_abs(1.0))
        .with_assignments(&assignments)
        .with_coupling_graph(&graph)
        .run()?;

    let csv = write_analysis1d_csv(&analysis)?;

    assert!(csv.starts_with("# format=RSpin Analysis 1D CSV\n"));
    assert!(csv.contains("# section=peaks\n"));
    assert!(csv.contains("index,x,intensity,prominence,polarity\n"));
    assert!(csv.contains("# section=ranges\n"));
    assert!(csv.contains("# section=multiplets\n"));
    assert!(csv.contains("# section=signals\n"));
    assert!(csv.contains("\"H,1\""));
    assert!(csv.contains("assignment_count,atom_ids,coupling_count"));

    let codec = CsvAnalysis1D;
    assert_eq!(codec.write_string(&analysis)?, csv);
    Ok(())
}

#[test]
fn writes_two_dimensional_analysis_csv() -> anyhow::Result<()> {
    let spectrum = Spectrum2D::new(
        Axis::linear("1H", Unit::Ppm, 0.0, 2.0, 3)?,
        Axis::linear("13C", Unit::Ppm, 0.0, 2.0, 3)?,
        vec![2.0, 0.0, 0.0, 1.5, 0.0, -3.0, 0.0, 0.0, -4.0],
        Metadata::named("analysis-2d"),
    )?;
    let assignments = AssignmentSet::new(vec![Assignment::deterministic(
        AssignmentTarget::Zone2D {
            id: "zone:x0-0:y0-1".to_owned(),
        },
        vec![AssignedAtom::new("C1H1", Nucleus::Carbon13)],
    )?])?;
    let analysis = spectrum
        .analyze()
        .with_zone_options(ZoneDetectionOptions::new().with_threshold_abs(1.0))
        .with_assignments(&assignments)
        .run()?;

    let csv = write_analysis2d_csv(&analysis)?;

    assert!(csv.starts_with("# format=RSpin Analysis 2D CSV\n"));
    assert!(csv.contains("# section=zones\n"));
    assert!(csv.contains("id,x_start_index,x_end_index,y_start_index,y_end_index"));
    assert!(csv.contains("# section=signals\n"));
    assert!(csv.contains("signal2d:zone:x0-0:y0-1"));
    assert!(csv.contains("C1H1"));

    let codec = CsvAnalysis2D;
    assert_eq!(codec.write_string(&analysis)?, csv);
    Ok(())
}

#[test]
fn rejects_non_finite_analysis_values() {
    let analysis = rspin_analysis::SpectrumAnalysis1D {
        peaks: vec![rspin_analysis::Peak {
            index: 0,
            x: f64::NAN,
            intensity: 1.0,
            prominence: 1.0,
            polarity: rspin_analysis::PeakPolarity::Positive,
        }],
        ranges: Vec::new(),
        multiplets: Vec::new(),
        signals: Vec::new(),
    };

    let error = write_analysis1d_csv(&analysis).expect_err("non-finite export should fail");
    assert_eq!(error, RSpinError::NonFinite { field: "peak x" });
}
