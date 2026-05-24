use super::prelude::*;

#[test]
fn prelude_supports_common_processing_workflow() -> Result<()> {
    let spectrum = Spectrum1D::new(
        Axis::linear_ppm(0.0, 2.0, 3)?,
        vec![1.0, -2.0, 4.0],
        Metadata::new()
            .with_name("demo")
            .with_nucleus(Nucleus::Hydrogen1)
            .with_frequency_mhz(400.0),
    )?;

    let processed = spectrum
        .process()
        .crop(0.0, 1.0)
        .resample(Axis::linear_ppm(0.0, 1.0, 3)?)
        .scale(2.0)
        .absolute_value()
        .normalize_max_abs()
        .finish()?;

    assert_eq!(processed.intensities, vec![0.5, 0.25, 1.0]);
    assert_eq!(processed.processing.len(), 5);

    let recipe = ProcessingRecipe1D::new()
        .scale(2.0)
        .offset(-2.0)
        .absolute_value()
        .normalize_max_abs();
    let recipe_json = write_processing_recipe_1d_json(&recipe)?;
    assert_eq!(read_processing_recipe_1d_json(&recipe_json)?, recipe);

    let recipe_2d = ProcessingRecipe2D::new()
        .scale(2.0)
        .zero_fill(4, 4)
        .normalize_max_abs();
    let recipe_2d_json = write_processing_recipe_2d_json(&recipe_2d)?;
    assert_eq!(read_processing_recipe_2d_json(&recipe_2d_json)?, recipe_2d);
    Ok(())
}

#[test]
fn prelude_supports_common_io_and_exact_simulation() -> Result<()> {
    let spectrum = read_spectrum1d_csv("x,intensity\n1,2\n2,4\n")?;
    assert_eq!(spectrum.len(), 2);

    let aligned = align_spectra_by_peak_to_matrix(
        &[
            Spectrum1D::new(
                Axis::linear_ppm(0.0, 2.0, 3)?,
                vec![0.0, 5.0, 0.0],
                Metadata::named("ref"),
            )?,
            Spectrum1D::new(
                Axis::linear_ppm(0.5, 2.5, 3)?,
                vec![0.0, 7.0, 0.0],
                Metadata::named("shifted"),
            )?,
        ],
        PeakAlignmentOptions::new(),
        MatrixGenerationOptions::new(),
    )?;
    assert_eq!(aligned.matrix.shape(), (2, 3));

    let buckets = bucket_spectrum_1d(
        &Spectrum1D::new(
            Axis::linear_ppm(0.0, 2.0, 3)?,
            vec![1.0, 1.0, 1.0],
            Metadata::named("bucketed"),
        )?,
        BucketOptions1D::new(0.0, 2.0, 2),
    )?;
    assert_eq!(buckets.len(), 2);

    let buckets_2d = bucket_spectrum_2d(
        &Spectrum2D::new(
            Axis::linear_ppm(0.0, 2.0, 3)?,
            Axis::linear_ppm(0.0, 2.0, 3)?,
            vec![1.0; 9],
            Metadata::named("bucketed-2d"),
        )?,
        BucketOptions2D::new(0.0, 2.0, 0.0, 2.0, 2, 2),
    )?;
    assert_eq!(buckets_2d.len(), 4);

    let pca = pca_matrix(
        &["a".to_owned(), "b".to_owned(), "c".to_owned()],
        &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
        3,
        2,
        MatrixPcaOptions::new()
            .with_component_count(1)
            .with_scaling(MatrixScaling::None),
    )?;
    assert_eq!(pca.score_shape(), (3, 1));

    let pairwise = pairwise_matrix(
        &["a".to_owned(), "b".to_owned()],
        &[3.0, 4.0, 0.0, 0.0],
        2,
        2,
        MatrixPairwiseOptions::new().with_metric(MatrixPairwiseMetric::EuclideanDistance),
    )?;
    let pairwise_value = pairwise
        .value_at(0, 1)
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "missing pairwise value".to_owned(),
        })?;
    assert!((pairwise_value - 5.0).abs() < 1.0e-12);

    let clusters = cluster_matrix(
        &["a".to_owned(), "b".to_owned(), "c".to_owned()],
        &[0.0, 2.0, 5.0],
        3,
        1,
        MatrixClusteringOptions::new().with_linkage(MatrixLinkage::Single),
    )?;
    assert_eq!(clusters.merges.len(), 2);
    let cluster_cut = clusters.cut_to_cluster_count(2)?;
    assert_eq!(cluster_cut.cluster_ids, vec![0, 0, 1]);

    let system = SpinHalfSystem::new().with_spin(1.0);
    let transitions = exact_spin_half_transitions(
        &system,
        &ExactSpinOptions {
            spectrometer_mhz: 400.0,
            ..ExactSpinOptions::default()
        },
    )?;

    assert_eq!(transitions.len(), 1);
    assert!((transitions[0].center_ppm - 1.0).abs() < 1.0e-12);
    Ok(())
}

#[test]
fn prelude_supports_prediction_bond_correlations() -> Result<()> {
    let molecule = Molecule::new("methanol")
        .with_atom(Atom::new("H1", "H"))
        .with_atom(Atom::new("C1", "C"))
        .with_bond(Bond::new("C1", "H1"));
    let prediction = predict_molecule_with_rules(
        &molecule,
        &ElementShiftPredictor::new()
            .with_rule(ElementShiftRule::new(
                "H",
                Experiment::Proton1D,
                Nucleus::Hydrogen1,
                0.9,
            ))
            .with_rule(ElementShiftRule::new(
                "C",
                Experiment::Carbon13_1D,
                Nucleus::Carbon13,
                50.0,
            ))
            .with_correlation_rule(BondCorrelationRule::new(
                Experiment::Hsqc,
                Nucleus::Hydrogen1,
                Nucleus::Carbon13,
            )),
    )?;

    assert_eq!(prediction.signals_1d.len(), 2);
    assert_eq!(prediction.correlations_2d.len(), 1);
    Ok(())
}

#[test]
fn prelude_supports_exact_2d_simulation() -> Result<()> {
    let system = SpinHalfSystem::new().with_spin(1.0).with_spin(2.0);
    let spectrum = simulate_exact_spin_half_2d(
        &system,
        &ExactSpectrum2DOptions::new()
            .with_x_ppm_range(0.95, 1.05)
            .with_y_ppm_range(1.95, 2.05)
            .with_points(5, 5)
            .with_spin_pair(0, 1),
    )?;

    assert_eq!(spectrum.shape(), (5, 5));
    assert!(spectrum.z[12] > spectrum.z[0]);
    Ok(())
}

#[test]
fn prelude_supports_consensus_workflows() -> Result<()> {
    let consensus = detect_consensus_peaks_1d(
        &[
            Spectrum1D::new(
                Axis::linear_ppm(0.0, 2.0, 3)?,
                vec![0.0, 5.0, 0.0],
                Metadata::named("a"),
            )?,
            Spectrum1D::new(
                Axis::linear_ppm(0.02, 2.02, 3)?,
                vec![0.0, 4.0, 0.0],
                Metadata::named("b"),
            )?,
        ],
        ConsensusPeakOptions::new()
            .with_max_shift(0.05)
            .with_min_spectrum_count(2),
    )?;

    assert_eq!(consensus.len(), 1);
    assert_eq!(consensus[0].spectrum_count, 2);

    let consensus_ranges = detect_consensus_ranges_1d(
        &[
            Spectrum1D::new(
                Axis::linear_ppm(0.0, 3.0, 4)?,
                vec![0.0, 2.0, 3.0, 0.0],
                Metadata::named("a"),
            )?,
            Spectrum1D::new(
                Axis::linear_ppm(0.02, 3.02, 4)?,
                vec![0.0, 4.0, 5.0, 0.0],
                Metadata::named("b"),
            )?,
        ],
        ConsensusRangeOptions::new()
            .with_max_gap(0.05)
            .with_min_spectrum_count(2)
            .with_range_options(RangeDetectionOptions::new().with_threshold_abs(1.0)),
    )?;

    assert_eq!(consensus_ranges.len(), 1);
    assert_eq!(consensus_ranges[0].spectrum_count, 2);
    Ok(())
}

#[test]
fn prelude_supports_simple_analysis_workflows() -> Result<()> {
    let analysis = analyze_spectrum_1d(
        &Spectrum1D::new(
            Axis::linear_ppm(0.0, 4.0, 5)?,
            vec![0.0, 2.0, 0.0, 1.5, 0.0],
            Metadata::named("analysis-1d"),
        )?,
        SpectrumAnalysis1DOptions::new()
            .with_peak_options(PeakPickOptions::new().with_min_abs_intensity(1.0))
            .with_range_options(RangeDetectionOptions::new().with_threshold_abs(1.0)),
    )?;

    assert_eq!(analysis.peaks.len(), 2);
    assert_eq!(analysis.ranges.len(), 2);
    assert_eq!(analysis.signals.len(), 2);
    let analysis_csv = write_analysis1d_csv(&analysis)?;
    assert!(analysis_csv.contains("# section=peaks"));
    assert!(analysis_csv.contains("# section=signals"));
    let analysis_json = write_analysis1d_json(&analysis)?;
    assert_eq!(read_analysis1d_json(&analysis_json)?, analysis);

    let spectrum_2d = Spectrum2D::new(
        Axis::linear_ppm(0.0, 2.0, 3)?,
        Axis::linear_ppm(0.0, 2.0, 3)?,
        vec![2.0, 0.0, 0.0, 1.5, 0.0, -3.0, 0.0, 0.0, -4.0],
        Metadata::named("analysis-2d"),
    )?;
    let analysis_2d = spectrum_2d
        .analyze()
        .with_zone_options(ZoneDetectionOptions::new().with_threshold_abs(1.0))
        .run()?;

    assert_eq!(analysis_2d.zones.len(), 2);
    assert_eq!(analysis_2d.signals.len(), 2);
    let analysis_2d_csv = write_analysis2d_csv(&analysis_2d)?;
    assert!(analysis_2d_csv.contains("# section=zones"));
    let analysis_2d_json = write_analysis2d_json(&analysis_2d)?;
    assert_eq!(read_analysis2d_json(&analysis_2d_json)?, analysis_2d);
    Ok(())
}

#[test]
fn prelude_supports_consensus_zone_workflows() -> Result<()> {
    let consensus_zones = detect_consensus_zones_2d(
        &[
            Spectrum2D::new(
                Axis::linear_ppm(0.0, 2.0, 3)?,
                Axis::linear_ppm(0.0, 2.0, 3)?,
                vec![0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0],
                Metadata::named("a"),
            )?,
            Spectrum2D::new(
                Axis::linear_ppm(0.02, 2.02, 3)?,
                Axis::linear_ppm(0.01, 2.01, 3)?,
                vec![0.0, 0.0, 0.0, 0.0, 4.0, 0.0, 0.0, 0.0, 0.0],
                Metadata::named("b"),
            )?,
        ],
        ConsensusZoneOptions::new()
            .with_max_gap(0.05)
            .with_min_spectrum_count(2)
            .with_zone_options(ZoneDetectionOptions::new().with_threshold_abs(1.0)),
    )?;

    assert_eq!(consensus_zones.len(), 1);
    assert_eq!(consensus_zones[0].spectrum_count, 2);
    Ok(())
}

#[test]
fn prelude_supports_zone_alignment_workflows() -> Result<()> {
    let result = align_spectra_by_zone_to_matrix(
        &[
            Spectrum2D::new(
                Axis::linear_ppm(0.0, 2.0, 3)?,
                Axis::linear_ppm(0.0, 2.0, 3)?,
                vec![0.0, 0.0, 0.0, 0.0, 5.0, 0.0, 0.0, 0.0, 0.0],
                Metadata::named("ref"),
            )?,
            Spectrum2D::new(
                Axis::linear_ppm(0.5, 2.5, 3)?,
                Axis::linear_ppm(-0.25, 1.75, 3)?,
                vec![0.0, 0.0, 0.0, 0.0, 7.0, 0.0, 0.0, 0.0, 0.0],
                Metadata::named("shifted"),
            )?,
        ],
        ZoneAlignmentOptions::new(),
        MatrixGeneration2DOptions::new(),
    )?;

    assert_eq!(result.matrix.shape(), (2, 3, 3));
    assert!((result.shifts[1].delta_x + 0.5).abs() < 1.0e-12);
    assert!((result.shifts[1].delta_y - 0.25).abs() < 1.0e-12);
    Ok(())
}
