use rspin_core::{Axis, Metadata, Nucleus, Spectrum1D, Spectrum2D, Unit};

use crate::{
    AssignedAtom, Assignment, AssignmentSet, AssignmentTarget, CouplingNode, JCoupling,
    JCouplingGraph, MultipletDetectionOptions, PeakPickOptions, PeakPolarity,
    RangeDetectionOptions, SignalSummary2DOptions, SignalSummaryOptions, SpectrumAnalysis1DOptions,
    SpectrumAnalysis2DOptions, ZoneConnectivity, ZoneDetectionOptions,
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
    assert_eq!(analysis.ranges.len(), 2);
    assert_eq!(analysis.multiplets.len(), 1);
    assert_eq!(analysis.signals.len(), 2);
    assert_eq!(analysis.multiplets[0].estimated_j_hz, Some(800.0));
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
