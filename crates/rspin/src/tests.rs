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
    assert!(recipe_json.contains(PROCESSING_RECIPE_1D_FORMAT));
    assert!(recipe_json.contains(&format!("\"version\":{PROCESSING_RECIPE_JSON_VERSION}")));
    assert_eq!(read_processing_recipe_1d_json(&recipe_json)?, recipe);

    let recipe_2d = ProcessingRecipe2D::new()
        .scale(2.0)
        .zero_fill(4, 4)
        .normalize_max_abs();
    let recipe_2d_json = write_processing_recipe_2d_json(&recipe_2d)?;
    assert!(recipe_2d_json.contains(PROCESSING_RECIPE_2D_FORMAT));
    assert_eq!(read_processing_recipe_2d_json(&recipe_2d_json)?, recipe_2d);

    let baseline_corrected = Spectrum1D::new(
        Axis::linear_ppm(0.0, 3.0, 4)?,
        vec![1.0, 3.0, 5.0, 7.0],
        Metadata::named("sloped baseline"),
    )?
    .process()
    .subtract_baseline_with(BaselineMethod::Polynomial { degree: 1 })
    .finish()?;
    for value in baseline_corrected.intensities {
        assert!(value.abs() < 1.0e-12);
    }

    Ok(())
}

#[test]
fn prelude_supports_processed_analysis_bridge() -> Result<()> {
    let analysis = read_spectrum1d_csv("x,intensity\n0,0\n1,4\n2,0\n")?
        .process()
        .scale(0.5)
        .analyze()
        .with_peak_options(
            PeakPickOptions::new()
                .with_min_abs_intensity(1.0)
                .with_min_prominence(1.0),
        )
        .with_range_options(RangeDetectionOptions::new().with_threshold_abs(1.0))
        .run()?;

    assert_eq!(analysis.peaks.len(), 1);
    assert_eq!(analysis.ranges.len(), 1);

    let spectrum_2d = Spectrum2D::new(
        Axis::linear_ppm(0.0, 1.0, 2)?,
        Axis::linear_ppm(10.0, 11.0, 2)?,
        vec![0.0, 4.0, 0.0, 0.0],
        Metadata::named("processed zones"),
    )?;
    let analysis_2d = spectrum_2d
        .process()
        .scale(0.5)
        .analyze()
        .with_zone_options(ZoneDetectionOptions::new().with_threshold_abs(1.0))
        .run()?;

    assert_eq!(analysis_2d.zones.len(), 1);
    assert_eq!(analysis_2d.signals.len(), 1);
    Ok(())
}

#[test]
fn prelude_supports_common_io_and_exact_simulation() -> Result<()> {
    let agilent_2d_reader = AgilentFid2D;
    assert_eq!(format!("{agilent_2d_reader:?}"), "AgilentFid2D");
    let spectrum = read_spectrum1d_csv("x,intensity\n1,2\n2,4\n")?;
    assert_eq!(spectrum.len(), 2);
    let nmrml_version = parse_nmrml_version("v1.0.rc1")?;
    assert_eq!(nmrml_version.build.as_deref(), Some("rc1"));
    assert!(nmrml_version.is_supported_by_current_readers());
    let nmrml_text = write_nmrml_1d(&spectrum)?;
    assert_eq!(
        read_nmrml_1d_str(&nmrml_text)?.intensities,
        spectrum.intensities
    );
    let spectrum_2d = Spectrum2D::new(
        Axis::linear_ppm(0.0, 1.0, 2)?,
        Axis::linear_ppm(10.0, 11.0, 2)?,
        vec![1.0, 2.0, 3.0, 4.0],
        Metadata::named("io 2d"),
    )?;
    let nmrml_2d_text = write_nmrml_2d(&spectrum_2d)?;
    assert_eq!(read_nmrml_2d_str(&nmrml_2d_text)?.z, spectrum_2d.z);

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
    let transitions = system
        .simulate_exact()
        .with_spectrometer_mhz(400.0)
        .transitions()?;

    assert_eq!(transitions.len(), 1);
    assert!((transitions[0].center_ppm - 1.0).abs() < 1.0e-12);
    Ok(())
}

#[test]
fn prelude_supports_exact_simulation_json() -> Result<()> {
    let system = SpinHalfSystem::new().with_spin(1.0);
    let system_json = write_spin_half_system_json(&system)?;
    assert!(system_json.contains(SPIN_HALF_SYSTEM_JSON_FORMAT));
    assert!(system_json.contains(&format!("\"version\":{SIMULATION_JSON_VERSION}")));
    assert_eq!(read_spin_half_system_json(&system_json)?, system);

    let options = ExactSpinOptions::new().with_spectrometer_mhz(400.0);
    let options_json = write_exact_spin_options_json(&options)?;
    assert!(options_json.contains(EXACT_SPIN_OPTIONS_JSON_FORMAT));
    assert_eq!(read_exact_spin_options_json(&options_json)?, options);

    let transitions = exact_spin_half_transitions(&system, &options)?;
    let transitions_json =
        <JsonExactTransitions as SpectrumWriter<[ExactTransition]>>::write_string(
            &JsonExactTransitions,
            &transitions,
        )?;
    assert!(transitions_json.contains(EXACT_TRANSITIONS_JSON_FORMAT));
    let parsed: Vec<ExactTransition> =
        SpectrumReader::read_str(&JsonExactTransitions, &transitions_json)?;
    assert_eq!(parsed, transitions);
    Ok(())
}

#[test]
fn prelude_supports_nmredata_import() -> Result<()> {
    let record = nmredata_prelude_fixture()?;

    assert_eq!(
        record.version.as_ref().map(|version| version.major),
        Some(1)
    );
    assert_eq!(record.assignments[0].label, "H1");
    assert_eq!(
        record.spectra[0].kind,
        NmreDataSpectrumKind::OneD {
            observed_label: "1H".to_owned(),
            observed_nucleus: Some(Nucleus::Hydrogen1),
        }
    );
    let larmor = record.spectra[0]
        .larmor_mhz
        .ok_or_else(|| RSpinError::InvalidMetadata {
            message: "missing NMReDATA larmor".to_owned(),
        })?;
    assert!((larmor - 500.0).abs() < 1.0e-12);

    let parsed_version = parse_nmredata_version("1.1")?;
    assert_eq!(parsed_version.minor, Some(1));
    let nmredata_text = write_nmredata_record(&record)?;
    let reparsed = read_nmredata_str(&nmredata_text)?;
    assert!((reparsed.assignments[0].shift_ppm - 4.2).abs() < 1.0e-12);
    let trait_record = SpectrumReader::read_str(&NmreData, &nmredata_text)?;
    let trait_text = SpectrumWriter::write_string(&NmreData, &trait_record)?;
    assert!(trait_text.contains(">  <NMREDATA_VERSION>"));
    let record_payload = write_nmredata_record_json(&trait_record)?;
    assert!(record_payload.contains(NMREDATA_RECORD_JSON_FORMAT));
    assert!(record_payload.contains(&format!("\"version\":{NMREDATA_JSON_VERSION}")));
    assert_eq!(read_nmredata_record_json(&record_payload)?, trait_record);
    let assignment_set = trait_record.to_assignment_set(Nucleus::Hydrogen1)?;
    assert_eq!(assignment_set.len(), 1);
    assert_assignment_json_round_trip(&assignment_set)?;
    let coupling_graph = nmredata_couplings_to_j_coupling_graph(&trait_record, Nucleus::Hydrogen1)?;
    assert_eq!(coupling_graph.couplings.len(), 1);
    assert_j_coupling_json_round_trip(&coupling_graph)?;
    let analysis: NmreDataAnalysis = nmredata_to_analysis(&trait_record, Nucleus::Hydrogen1)?;
    assert_eq!(analysis.assignment_set.len(), 1);
    assert_eq!(analysis.j_coupling_graph.couplings.len(), 1);
    let signal_assignments =
        nmredata_1d_signals_to_assignment_set(&trait_record, Nucleus::Hydrogen1)?;
    assert_eq!(signal_assignments.len(), 1);
    assert_eq!(analysis.signal_assignment_set, signal_assignments);
    assert!(matches!(
        signal_assignments.assignments[0].target,
        AssignmentTarget::Peak1D { index: 0, x } if (x - 4.2).abs() < 1.0e-12
    ));
    let signal_assignments_2d = nmredata_2d_signals_to_assignment_set(&trait_record)?;
    assert_eq!(signal_assignments_2d.len(), 1);
    assert_eq!(analysis.signal_assignment_set_2d, signal_assignments_2d);
    assert_eq!(
        signal_assignments_2d.assignments[0].target,
        AssignmentTarget::Zone2D {
            id: nmredata_2d_signal_zone_id(0, &trait_record.spectra[1].signals_2d[0]),
        }
    );
    assert_eq!(
        signal_assignments_2d.assignments[0].atoms[0].nucleus,
        Nucleus::Hydrogen1
    );
    assert_eq!(
        signal_assignments_2d.assignments[0].atoms[1].nucleus,
        Nucleus::Carbon13
    );
    let records = vec![trait_record];
    let records_text =
        <NmreData as SpectrumWriter<[NmreDataRecord]>>::write_string(&NmreData, &records)?;
    assert_eq!(records_text.matches("$$$$").count(), 1);
    let trait_records: Vec<NmreDataRecord> =
        SpectrumReader::read_str(&NmreDataRecords, &records_text)?;
    assert_eq!(trait_records.len(), 1);
    let bytes_records = read_nmredata_records_bytes(records_text.as_bytes())?;
    assert_eq!(bytes_records, trait_records);
    let records_codec_text = <NmreDataRecords as SpectrumWriter<[NmreDataRecord]>>::write_string(
        &NmreDataRecords,
        &trait_records,
    )?;
    assert_eq!(records_codec_text.matches("$$$$").count(), 1);
    let record_list_payload = write_nmredata_records_json(&trait_records)?;
    assert!(record_list_payload.contains(NMREDATA_RECORDS_JSON_FORMAT));
    assert_eq!(
        read_nmredata_records_json(&record_list_payload)?,
        trait_records
    );
    assert_eq!(format!("{NmreData:?}"), "NmreData");
    assert_eq!(format!("{NmreDataRecords:?}"), "NmreDataRecords");
    Ok(())
}

fn nmredata_prelude_fixture() -> Result<NmreDataRecord> {
    read_nmredata_str(
        r"
>  <NMREDATA_VERSION>
1.1

>  <NMREDATA_ASSIGNMENT>
H1, 4.200, H1

>  <NMREDATA_J>
H1, H2, 7.0

>  <NMREDATA_1D_1H>
Larmor=500.0
4.200, L=H1

>  <NMREDATA_2D_13C_1J_1H>
H1/C1, I=1.0
",
    )
}

fn assert_assignment_json_round_trip(assignment_set: &AssignmentSet) -> Result<()> {
    let assignment_set_json = write_assignment_set_json(assignment_set)?;
    assert!(assignment_set_json.contains(ASSIGNMENT_SET_JSON_FORMAT));
    assert!(assignment_set_json.contains(&format!("\"version\":{ASSIGNMENT_JSON_VERSION}")));
    assert_eq!(
        read_assignment_set_json(&assignment_set_json)?,
        *assignment_set
    );
    Ok(())
}

fn assert_j_coupling_json_round_trip(coupling_graph: &JCouplingGraph) -> Result<()> {
    let graph_json = write_j_coupling_graph_json(coupling_graph)?;
    assert!(graph_json.contains(J_COUPLING_GRAPH_JSON_FORMAT));
    assert_eq!(read_j_coupling_graph_json(&graph_json)?, *coupling_graph);
    Ok(())
}

#[test]
fn prelude_supports_path_writer_exports() -> Result<()> {
    let path_writer_1d = AutoSpectrum1DPathWriter;
    let path_writer_2d = AutoSpectrum2DPathWriter;
    assert_eq!(format!("{path_writer_1d:?}"), "AutoSpectrum1DPathWriter");
    assert_eq!(format!("{path_writer_2d:?}"), "AutoSpectrum2DPathWriter");
    assert_eq!(
        detect_spectrum1d_write_path_format("one.csv")?,
        Spectrum1DWritePathFormat::Csv
    );
    assert_eq!(
        detect_spectrum2d_write_path_format("two.nmrml")?,
        Spectrum2DWritePathFormat::NmrMl
    );
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
    let prediction_json = write_prediction_json(&prediction)?;
    assert!(prediction_json.contains(PREDICTION_JSON_FORMAT));
    assert!(prediction_json.contains(&format!("\"version\":{PREDICTION_JSON_VERSION}")));
    assert_eq!(read_prediction_json(&prediction_json)?, prediction);

    let formula_atoms = atoms_from_formula("C2H6O")?;
    assert_eq!(formula_atoms.len(), 9);
    let formula_prediction = predict_formula_with_rules(
        "ethanol",
        "C2H6O",
        &ElementShiftPredictor::new().with_rule(ElementShiftRule::new(
            "H",
            Experiment::Proton1D,
            Nucleus::Hydrogen1,
            1.1,
        )),
    )?;
    assert_eq!(formula_prediction.signals_1d.len(), 6);
    Ok(())
}

#[test]
fn prelude_supports_exact_2d_simulation() -> Result<()> {
    let system = SpinHalfSystem::new().with_spin(1.0).with_spin(2.0);
    let spectrum = system
        .simulate_exact()
        .render_2d()
        .with_x_ppm_range(0.95, 1.05)
        .with_y_ppm_range(1.95, 2.05)
        .with_points(5, 5)
        .with_spin_pair(0, 1)
        .run()?;

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
            .with_peak_optimization_options(PeakOptimizationOptions::new())
            .with_range_options(RangeDetectionOptions::new().with_threshold_abs(1.0)),
    )?;

    assert_eq!(analysis.peaks.len(), 2);
    assert_eq!(analysis.optimized_peaks.len(), 2);
    assert_eq!(analysis.ranges.len(), 2);
    assert_eq!(analysis.signals.len(), 2);
    let analysis_csv = write_analysis1d_csv(&analysis)?;
    assert!(analysis_csv.contains("# section=peaks"));
    assert!(analysis_csv.contains("# section=signals"));
    let analysis_json = write_analysis1d_json(&analysis)?;
    assert!(analysis_json.contains(ANALYSIS_1D_JSON_FORMAT));
    assert!(analysis_json.contains(&format!("\"version\":{ANALYSIS_JSON_VERSION}")));
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
    assert!(analysis_2d_json.contains(ANALYSIS_2D_JSON_FORMAT));
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
