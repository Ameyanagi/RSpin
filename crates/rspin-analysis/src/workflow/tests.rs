use rspin_core::{Axis, Metadata, Nucleus, RSpinError, Spectrum1D, Spectrum2D, Unit};

use crate::{
    AnalyzeSpectrum1D, AnalyzeSpectrum1DResult, AnalyzeSpectrum2D, AnalyzeSpectrum2DResult,
    AssignedAtom, Assignment, AssignmentSet, AssignmentTarget, CouplingNode, JCoupling,
    JCouplingGraph, MultipletDetectionOptions, PeakOptimizationOptions, PeakPickOptions,
    PeakPolarity, RangeDetectionOptions, SignalSummary2DOptions, SignalSummaryOptions,
    SpectrumAnalysis1DOptions, SpectrumAnalysis2DOptions, ZoneConnectivity, ZoneDetectionOptions,
    analyze_assigned_spectrum_1d, analyze_assigned_spectrum_2d, analyze_spectrum_1d,
    analyze_spectrum_2d,
};

#[test]
fn analyzes_one_dimensional_spectrum_with_defaults() -> anyhow::Result<()> {
    let spectrum = spectrum_1d()?;
    let options = SpectrumAnalysis1DOptions::new()
        .with_peak_options(
            PeakPickOptions::new()
                .with_min_abs_intensity(1.0)
                .with_min_prominence(1.0)
                .with_polarity(PeakPolarity::Positive),
        )
        .with_range_options(RangeDetectionOptions::new().with_threshold_abs(1.0))
        .with_multiplet_options(
            MultipletDetectionOptions::new()
                .with_max_peak_gap_ppm(2.1)
                .with_spectrometer_mhz(400.0),
        );

    let analysis = analyze_spectrum_1d(&spectrum, options)?;

    assert_eq!(analysis.peaks.len(), 2);
    assert!(analysis.optimized_peaks.is_empty());
    assert_eq!(analysis.ranges.len(), 2);
    assert_eq!(analysis.multiplets.len(), 1);
    assert_eq!(analysis.signals.len(), 2);
    assert_eq!(analysis.multiplets[0].estimated_j_hz, Some(800.0));
    Ok(())
}

#[test]
fn analyzes_one_dimensional_spectrum_with_peak_optimization() -> anyhow::Result<()> {
    let spectrum = Spectrum1D::new(
        Axis::new("shift", Unit::Ppm, vec![0.0, 0.4, 1.0])?,
        vec![0.75, 1.0, 0.75],
        Metadata::named("optimized"),
    )?;
    let options = SpectrumAnalysis1DOptions::new()
        .with_peak_options(
            PeakPickOptions::new()
                .with_min_abs_intensity(0.5)
                .with_min_prominence(0.1)
                .with_polarity(PeakPolarity::Positive),
        )
        .with_peak_optimization_options(PeakOptimizationOptions::new());

    let analysis = analyze_spectrum_1d(&spectrum, options)?;

    assert_eq!(analysis.peaks.len(), 1);
    assert_eq!(analysis.optimized_peaks.len(), 1);
    assert!(analysis.optimized_peaks[0].optimized);
    assert!((analysis.optimized_peaks[0].x - 0.5).abs() < 1.0e-12);
    assert!((analysis.optimized_peaks[0].delta_x - 0.1).abs() < 1.0e-12);
    Ok(())
}

#[test]
fn analyzes_one_dimensional_spectrum_with_assignments() -> anyhow::Result<()> {
    let spectrum = spectrum_1d()?;
    let options = SpectrumAnalysis1DOptions::new()
        .with_peak_options(PeakPickOptions::new().with_min_abs_intensity(1.0))
        .with_range_options(RangeDetectionOptions::new().with_threshold_abs(1.0))
        .with_signal_options(
            SignalSummaryOptions::new()
                .with_include_empty_ranges(true)
                .with_include_orphan_multiplets(false),
        );
    let assignments = AssignmentSet::new(vec![Assignment::deterministic(
        AssignmentTarget::Range1D {
            start_index: 2,
            end_index: 2,
            from: 2.0,
            to: 2.0,
        },
        vec![AssignedAtom::new("H2", Nucleus::Hydrogen1)],
    )?])?;
    let graph = JCouplingGraph::new(
        vec![
            CouplingNode::new("H2", Nucleus::Hydrogen1),
            CouplingNode::new("H3", Nucleus::Hydrogen1),
        ],
        vec![JCoupling::deterministic("H2", "H3", 7.2)?],
    )?;

    let analysis = analyze_assigned_spectrum_1d(&spectrum, &assignments, &graph, options)?;

    assert_eq!(analysis.signals.len(), 2);
    assert_eq!(analysis.signals[0].assignments.len(), 1);
    assert_eq!(analysis.signals[0].atoms[0].id, "H2");
    assert_eq!(analysis.signals[0].couplings.len(), 1);
    Ok(())
}

#[test]
fn rejects_invalid_one_dimensional_options() -> anyhow::Result<()> {
    let spectrum = spectrum_1d()?;
    let result = analyze_spectrum_1d(
        &spectrum,
        SpectrumAnalysis1DOptions::new()
            .with_range_options(RangeDetectionOptions::new().with_threshold_abs(-1.0)),
    );

    assert!(result.is_err());
    Ok(())
}

#[test]
fn runs_chainable_one_dimensional_workflow() -> anyhow::Result<()> {
    let spectrum = spectrum_1d()?;
    let assignments = AssignmentSet::new(vec![Assignment::deterministic(
        AssignmentTarget::Range1D {
            start_index: 2,
            end_index: 2,
            from: 2.0,
            to: 2.0,
        },
        vec![AssignedAtom::new("H2", Nucleus::Hydrogen1)],
    )?])?;
    let graph = JCouplingGraph::new(
        vec![
            CouplingNode::new("H2", Nucleus::Hydrogen1),
            CouplingNode::new("H3", Nucleus::Hydrogen1),
        ],
        vec![JCoupling::deterministic("H2", "H3", 7.2)?],
    )?;

    let analysis = spectrum
        .analyze()
        .with_peak_options(PeakPickOptions::new().with_min_abs_intensity(1.0))
        .with_range_options(RangeDetectionOptions::new().with_threshold_abs(1.0))
        .with_peak_optimization_options(PeakOptimizationOptions::new())
        .with_multiplet_options(MultipletDetectionOptions::new().with_max_peak_gap_ppm(2.1))
        .with_assignments(&assignments)
        .with_coupling_graph(&graph)
        .run()?;

    assert_eq!(analysis.peaks.len(), 2);
    assert_eq!(analysis.optimized_peaks.len(), 2);
    assert_eq!(analysis.signals[0].atoms[0].id, "H2");
    assert_eq!(analysis.signals[0].couplings.len(), 1);

    let analysis_without_couplings = spectrum
        .analyze()
        .with_peak_options(PeakPickOptions::new().with_min_abs_intensity(1.0))
        .with_range_options(RangeDetectionOptions::new().with_threshold_abs(1.0))
        .with_peak_optimization_options(PeakOptimizationOptions::new())
        .with_assignments(&assignments)
        .with_coupling_graph(&graph)
        .without_peak_optimization()
        .without_coupling_graph()
        .run()?;
    assert!(analysis_without_couplings.signals[0].couplings.is_empty());
    assert!(analysis_without_couplings.optimized_peaks.is_empty());

    let analysis_without_assignments = spectrum
        .analyze()
        .with_options(SpectrumAnalysis1DOptions::new())
        .with_assignments(&assignments)
        .without_assignments()
        .run()?;
    assert!(analysis_without_assignments.signals[0].atoms.is_empty());
    Ok(())
}

#[test]
fn runs_chainable_one_dimensional_workflow_from_result() -> anyhow::Result<()> {
    let spectrum = Ok(spectrum_1d()?);
    let assignments = AssignmentSet::new(vec![Assignment::deterministic(
        AssignmentTarget::Range1D {
            start_index: 2,
            end_index: 2,
            from: 2.0,
            to: 2.0,
        },
        vec![AssignedAtom::new("H2", Nucleus::Hydrogen1)],
    )?])?;
    let graph = JCouplingGraph::new(
        vec![
            CouplingNode::new("H2", Nucleus::Hydrogen1),
            CouplingNode::new("H3", Nucleus::Hydrogen1),
        ],
        vec![JCoupling::deterministic("H2", "H3", 7.2)?],
    )?;

    let analysis = spectrum
        .analyze()
        .with_peak_options(PeakPickOptions::new().with_min_abs_intensity(1.0))
        .with_range_options(RangeDetectionOptions::new().with_threshold_abs(1.0))
        .with_assignments(&assignments)
        .with_coupling_graph(&graph)
        .run()?;

    assert_eq!(analysis.peaks.len(), 2);
    assert_eq!(analysis.signals[0].atoms[0].id, "H2");
    assert_eq!(analysis.signals[0].couplings.len(), 1);
    Ok(())
}

#[test]
fn one_dimensional_result_workflow_preserves_initial_error() {
    let spectrum: rspin_core::Result<Spectrum1D> = Err(RSpinError::InvalidSpectrum {
        message: "load failed".to_owned(),
    });
    let error = spectrum
        .analyze()
        .with_range_options(RangeDetectionOptions::new().with_threshold_abs(1.0))
        .run()
        .expect_err("initial spectrum error should be preserved");

    assert_eq!(
        error,
        RSpinError::InvalidSpectrum {
            message: "load failed".to_owned()
        }
    );
}

#[test]
fn analyzes_two_dimensional_spectrum_with_defaults() -> anyhow::Result<()> {
    let spectrum = spectrum_2d()?;
    let options = SpectrumAnalysis2DOptions::new().with_zone_options(
        ZoneDetectionOptions::new()
            .with_threshold_abs(1.0)
            .with_connectivity(ZoneConnectivity::Eight),
    );

    let analysis = analyze_spectrum_2d(&spectrum, options)?;

    assert_eq!(analysis.zones.len(), 2);
    assert_eq!(analysis.signals.len(), 2);
    assert_eq!(analysis.signals[0].zone.id, "zone:x0-0:y0-1");
    Ok(())
}

#[test]
fn analyzes_two_dimensional_spectrum_with_assignments() -> anyhow::Result<()> {
    let spectrum = spectrum_2d()?;
    let options = SpectrumAnalysis2DOptions::new()
        .with_zone_options(ZoneDetectionOptions::new().with_threshold_abs(1.0))
        .with_signal_options(SignalSummary2DOptions::new().with_include_unassigned_zones(true));
    let assignments = AssignmentSet::new(vec![Assignment::deterministic(
        AssignmentTarget::Zone2D {
            id: "zone:x0-0:y0-1".to_owned(),
        },
        vec![AssignedAtom::new("C1H1", Nucleus::Carbon13)],
    )?])?;

    let analysis = analyze_assigned_spectrum_2d(&spectrum, &assignments, options)?;

    assert_eq!(analysis.zones.len(), 2);
    assert_eq!(analysis.signals.len(), 2);
    assert_eq!(analysis.signals[0].assignments.len(), 1);
    assert_eq!(analysis.signals[0].atoms[0].id, "C1H1");
    Ok(())
}

#[test]
fn rejects_invalid_two_dimensional_options() -> anyhow::Result<()> {
    let spectrum = spectrum_2d()?;
    let result = analyze_spectrum_2d(
        &spectrum,
        SpectrumAnalysis2DOptions::new()
            .with_zone_options(ZoneDetectionOptions::new().with_threshold_abs(-1.0)),
    );

    assert!(result.is_err());
    Ok(())
}

#[test]
fn runs_chainable_two_dimensional_workflow() -> anyhow::Result<()> {
    let spectrum = spectrum_2d()?;
    let assignments = AssignmentSet::new(vec![Assignment::deterministic(
        AssignmentTarget::Zone2D {
            id: "zone:x0-0:y0-1".to_owned(),
        },
        vec![AssignedAtom::new("C1", Nucleus::Carbon13)],
    )?])?;

    let analysis = spectrum
        .analyze()
        .with_zone_options(ZoneDetectionOptions::new().with_threshold_abs(1.0))
        .with_signal_options(SignalSummary2DOptions::new().with_include_unassigned_zones(true))
        .with_assignments(&assignments)
        .run()?;

    assert_eq!(analysis.zones.len(), 2);
    assert_eq!(analysis.signals[0].atoms[0].id, "C1");

    let analysis_without_assignments = spectrum
        .analyze()
        .with_options(SpectrumAnalysis2DOptions::new())
        .with_assignments(&assignments)
        .without_assignments()
        .run()?;
    assert!(analysis_without_assignments.signals[0].atoms.is_empty());
    Ok(())
}

#[test]
fn runs_chainable_two_dimensional_workflow_from_result() -> anyhow::Result<()> {
    let spectrum = Ok(spectrum_2d()?);
    let assignments = AssignmentSet::new(vec![Assignment::deterministic(
        AssignmentTarget::Zone2D {
            id: "zone:x0-0:y0-1".to_owned(),
        },
        vec![AssignedAtom::new("C1", Nucleus::Carbon13)],
    )?])?;

    let analysis = spectrum
        .analyze()
        .with_zone_options(ZoneDetectionOptions::new().with_threshold_abs(1.0))
        .with_assignments(&assignments)
        .run()?;

    assert_eq!(analysis.zones.len(), 2);
    assert_eq!(analysis.signals[0].atoms[0].id, "C1");
    Ok(())
}

#[test]
fn two_dimensional_result_workflow_preserves_initial_error() {
    let spectrum: rspin_core::Result<Spectrum2D> = Err(RSpinError::InvalidSpectrum {
        message: "2d load failed".to_owned(),
    });
    let error = spectrum
        .analyze()
        .with_zone_options(ZoneDetectionOptions::new().with_threshold_abs(1.0))
        .run()
        .expect_err("initial spectrum error should be preserved");

    assert_eq!(
        error,
        RSpinError::InvalidSpectrum {
            message: "2d load failed".to_owned()
        }
    );
}

fn spectrum_1d() -> anyhow::Result<Spectrum1D> {
    Ok(Spectrum1D::new(
        Axis::linear("shift", Unit::Ppm, 0.0, 6.0, 7)?,
        vec![0.0, 0.0, 2.0, 0.0, 1.5, 0.0, 0.0],
        Metadata::new()
            .with_name("analysis-demo")
            .with_frequency_mhz(400.0),
    )?)
}

fn spectrum_2d() -> anyhow::Result<Spectrum2D> {
    Ok(Spectrum2D::new(
        Axis::linear("1H", Unit::Ppm, 0.0, 2.0, 3)?,
        Axis::linear("13C", Unit::Ppm, 0.0, 2.0, 3)?,
        vec![2.0, 0.0, 0.0, 1.5, 0.0, -3.0, 0.0, 0.0, -4.0],
        Metadata::new().with_name("analysis-2d-demo"),
    )?)
}
