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
        .normalize_abs_area(1.0)
        .finish()?;

    assert_eq!(processed.intensities, vec![1.0, 0.5, 2.0]);
    assert!((spectrum_area(&processed, true)? - 1.0).abs() < 1.0e-12);
    assert_eq!(processed.processing.len(), 6);

    let recipe = ProcessingRecipe1D::new()
        .scale(2.0)
        .offset(-2.0)
        .absolute_value()
        .normalize_max_abs()
        .normalize_abs_area(3.0);
    let recipe_json = write_processing_recipe_1d_json(&recipe)?;
    assert!(recipe_json.contains(PROCESSING_RECIPE_1D_FORMAT));
    assert!(recipe_json.contains(&format!("\"version\":{PROCESSING_RECIPE_JSON_VERSION}")));
    assert_eq!(read_processing_recipe_1d_json(&recipe_json)?, recipe);

    let recipe_2d = ProcessingRecipe2D::new()
        .scale(2.0)
        .zero_fill(4, 4)
        .normalize_max_abs()
        .normalize_abs_volume(2.0);
    let recipe_2d_json = write_processing_recipe_2d_json(&recipe_2d)?;
    assert!(recipe_2d_json.contains(PROCESSING_RECIPE_2D_FORMAT));
    assert_eq!(read_processing_recipe_2d_json(&recipe_2d_json)?, recipe_2d);

    let normalized_2d = Spectrum2D::new(
        Axis::linear_ppm(0.0, 1.0, 2)?,
        Axis::linear_ppm(0.0, 1.0, 2)?,
        vec![1.0, -1.0, 1.0, -1.0],
        Metadata::named("2d volume"),
    )?
    .process()
    .normalize_abs_volume(3.0)
    .finish()?;
    assert!((spectrum_volume_2d(&normalized_2d, true)? - 3.0).abs() < 1.0e-12);

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
    assert_eq!(analysis.integrals.len(), 1);

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
    assert_eq!(analysis_2d.integrals.len(), 1);
    assert_eq!(analysis_2d.signals.len(), 1);
    Ok(())
}

#[test]
fn prelude_supports_common_io_and_exact_simulation() -> Result<()> {
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
fn prelude_supports_io_reader_markers_and_versions() -> Result<()> {
    let agilent_2d_reader = AgilentFid2D;
    assert_eq!(format!("{agilent_2d_reader:?}"), "AgilentFid2D");
    let jcamp_reader = JcampDx;
    assert_eq!(format!("{jcamp_reader:?}"), "JcampDx");
    let jcamp_2d_reader = JcampDx2D;
    assert_eq!(format!("{jcamp_2d_reader:?}"), "JcampDx2D");

    let jcamp_version = parse_jcamp_dx_version("5.00")?;
    assert_eq!(jcamp_version.major, 5);
    assert!(jcamp_version.is_supported_by_current_reader());
    let jcamp_2d = read_jcamp_dx_2d(
        "\
##TITLE=prelude 2d jcamp
##FIRSTX=1
##LASTX=0
##FIRSTY=10
##LASTY=11
##VAR_DIM=2,2,2
##PAGE=N=1
##DATA TABLE=(X++(Y..Y)), XYDATA
1 1 2
##PAGE=N=2
##DATA TABLE=(X++(Y..Y)), XYDATA
1 3 4
##END=
",
    )?;
    assert_eq!(jcamp_2d.shape(), (2, 2));
    assert_eq!(jcamp_2d.z, vec![1.0, 2.0, 3.0, 4.0]);
    let jcamp_2d_text = write_jcamp_dx_2d(&jcamp_2d)?;
    assert_eq!(read_jcamp_dx_2d(&jcamp_2d_text)?.z, jcamp_2d.z);
    assert_eq!(
        parse_spectrum2d_path_format("jdx")?,
        Spectrum2DPathFormat::JcampDx
    );
    let agilent_info = inspect_agilent_procpar("acqdim 7 1 32767 0 0 2 1 0 1 64\n1 2\n0\n")?;
    assert_eq!(agilent_info.acquisition_dimension, Some(2));
    assert!(agilent_info.is_supported_by_current_readers());
    let agilent_error = read_agilent_fid_1d_bytes("", b"not fid")
        .expect_err("invalid Agilent FID bytes should fail");
    assert!(matches!(agilent_error, RSpinError::Parse { .. }));
    assert_eq!(
        parse_spectrum1d_bytes_format("varian phasefile")?,
        Spectrum1DBytesFormat::AgilentProcessed
    );
    let bruker_info = inspect_bruker_parameter_file("##JCAMPDX= 5.00\n##DATATYPE= Parameters\n")?;
    assert_eq!(bruker_info.data_type.as_deref(), Some("Parameters"));
    assert!(bruker_info.is_supported_by_current_readers());
    let bruker_error =
        read_bruker_fid_1d_bytes("", b"not fid").expect_err("invalid Bruker FID bytes should fail");
    assert!(matches!(bruker_error, RSpinError::Parse { .. }));
    assert_eq!(
        "bruker 1r".parse::<Spectrum1DBytesFormat>()?,
        Spectrum1DBytesFormat::BrukerProcessed
    );
    let routed_error = Spectrum1DBytes::new(Spectrum1DBytesFormat::BrukerFid, b"not fid")
        .read()
        .expect_err("missing routed Bruker parameters should fail");
    assert!(matches!(routed_error, RSpinError::Parse { .. }));
    let bruker_ser_error = read_bruker_ser_2d_bytes("", "", b"not ser")
        .expect_err("invalid Bruker SER bytes should fail");
    assert!(matches!(bruker_ser_error, RSpinError::Parse { .. }));
    assert_eq!(
        parse_spectrum2d_bytes_format("ser")?,
        Spectrum2DBytesFormat::BrukerSer
    );
    let routed_2d_error = read_spectrum2d_bytes_as(
        b"not ser",
        Spectrum2DBytesFormat::BrukerSer,
        Some("##$TD= 4\n"),
        None,
    )
    .expect_err("missing routed Bruker 2D indirect parameters should fail");
    assert!(matches!(routed_2d_error, RSpinError::Parse { .. }));
    let jeol_version = JeolJdfVersion::new(1, 2);
    assert_eq!(jeol_version.raw, "1.2");
    assert!(jeol_version.is_supported_by_current_reader());
    let error = inspect_jeol_jdf_bytes(b"not jdf")
        .expect_err("invalid JEOL JDF bytes should fail inspection");
    assert!(matches!(error, RSpinError::Parse { .. }));
    Ok(())
}

#[test]
fn prelude_supports_simple_multi_path_bundle_loading() -> Result<()> {
    let fixture_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../rspin-io/testdata/zenodo_7100132");

    let bundle = load_spectra_many([
        fixture_root.join("varian_1h"),
        fixture_root.join("bruker_without_expno"),
    ])?;

    assert_eq!(bundle.len(), 3);
    assert_eq!(bundle.spectra_1d().count(), 3);
    assert_eq!(bundle.spectra_2d().count(), 0);
    assert!(bundle.warnings().is_empty());
    assert_eq!(
        bundle.source_format_count(LoadedSourceFormat::AgilentFid),
        1
    );
    assert_eq!(bundle.source_format_count("bruker_fid"), 1);
    assert_eq!(bundle.source_format_count("bruker_processed"), 1);
    let summary = bundle.summary();
    assert_eq!(summary.spectra(), 3);
    assert_eq!(summary.spectra_1d(), 3);
    assert_eq!(summary.spectra_2d(), 0);
    assert!(summary.has_source_format(LoadedSourceFormat::AgilentFid));
    assert_eq!(summary.source_format_count("bruker_processed"), 1);
    assert_eq!(
        summary.source_vendor_count(LoadedSourceVendor::AgilentVarian),
        1
    );
    assert_eq!(bundle.source_vendor_count(LoadedSourceVendor::Bruker), 2);
    assert_eq!(
        parse_loaded_source_format("varian fid")?,
        LoadedSourceFormat::AgilentFid
    );
    assert_eq!(
        parse_loaded_source_vendor("varian")?,
        LoadedSourceVendor::AgilentVarian
    );
    assert_eq!(
        parse_loaded_source_data_kind("processed")?,
        LoadedSourceDataKind::Processed
    );
    let source_format_counts = bundle.source_format_counts();
    assert!(
        source_format_counts
            .iter()
            .any(|count| { count.format() == "agilent_fid" && count.count() == 1 })
    );
    assert!(
        source_format_counts
            .iter()
            .any(|count| { count.format() == "bruker_processed" && count.count() == 1 })
    );
    assert_multi_path_source_counts(&bundle, &summary);

    let anchored = load_spectra_relative_to(&fixture_root, "bruker_without_expno")?;
    assert_eq!(anchored.len(), 2);
    let filtered = RSpinReader::new()
        .only_source_format(LoadedSourceFormat::AgilentFid)
        .read_path(fixture_root.join("varian_1h"))?;
    assert_eq!(filtered.len(), 1);
    assert_eq!(
        filtered.source_format_count(LoadedSourceFormat::AgilentFid),
        1
    );
    let vendor_filtered = RSpinReader::new()
        .only_source_vendor(LoadedSourceVendor::AgilentVarian)
        .read_path(fixture_root.join("varian_1h"))?;
    assert_eq!(vendor_filtered.len(), 1);
    let path_filtered = RSpinReader::new()
        .only_source_path("varian_1h")
        .read_path(&fixture_root)?;
    assert_eq!(path_filtered.len(), 1);
    assert_eq!(
        path_filtered.source_format_count(LoadedSourceFormat::AgilentFid),
        1
    );
    let exact = load_spectrum_1d_many_relative_to(&fixture_root, ["varian_1h"])?;
    assert_eq!(exact.metadata.nucleus, Some(Nucleus::Hydrogen1));
    let exact = load_spectrum_1d_paths_relative_to(&fixture_root, ["varian_1h"])?;
    assert_eq!(exact.metadata.nucleus, Some(Nucleus::Hydrogen1));
    let exact = load_spectrum_1d_relative_to(&fixture_root, "varian_1h")?;
    assert_eq!(exact.metadata.nucleus, Some(Nucleus::Hydrogen1));
    let (exact, source) = load_spectrum_1d_with_source_relative_to(&fixture_root, "varian_1h")?;
    assert_eq!(exact.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(source.format(), "agilent_fid");
    assert_eq!(source.path(), Some(std::path::Path::new("varian_1h")));
    let (exact, source) =
        load_spectrum_1d_paths_with_source_relative_to(&fixture_root, ["varian_1h"])?;
    assert_eq!(exact.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(source.format(), "agilent_fid");

    let agilent = bundle
        .spectra()
        .iter()
        .find(|loaded| loaded.source().format == "agilent_fid")
        .and_then(LoadedSpectrum::as_1d)
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "missing Agilent/Varian spectrum from facade bundle loader".to_owned(),
        })?;
    assert_eq!(agilent.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(agilent.x.unit, Unit::Seconds);
    Ok(())
}

#[test]
fn prelude_supports_first_bundle_accessors() -> Result<()> {
    let fixture_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../rspin-io/testdata/zenodo_7100132");
    let bundle = load_spectra_many([
        fixture_root.join("varian_1h"),
        fixture_root.join("bruker_without_expno"),
    ])?;

    let first = bundle.first_1d().ok_or_else(|| RSpinError::Parse {
        format: "facade bundle accessor",
        message: "missing first one-dimensional spectrum".to_owned(),
    })?;
    assert_eq!(first.metadata.nucleus, Some(Nucleus::Hydrogen1));

    let (first_loaded, source) = bundle.first_loaded_1d().ok_or_else(|| RSpinError::Parse {
        format: "facade bundle accessor",
        message: "missing first loaded one-dimensional spectrum".to_owned(),
    })?;
    assert_eq!(first_loaded.len(), first.len());
    assert!(source.path().is_some());
    assert!(bundle.first_2d().is_none());
    assert!(bundle.first_loaded_2d().is_none());
    Ok(())
}

#[test]
fn prelude_supports_first_source_filter_accessors() -> Result<()> {
    let fixture_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../rspin-io/testdata/nmrxiv/cc0/myrcene");
    let bundle = load_spectra(&fixture_root)?;

    let first = bundle
        .first_by_source(LoadedSourceFilter::vendor("bruker"))
        .ok_or_else(|| RSpinError::Parse {
            format: "facade bundle accessor",
            message: "missing first Bruker spectrum".to_owned(),
        })?;
    assert_eq!(first.source().vendor(), Some(LoadedSourceVendor::Bruker));

    let (carbon, source) = bundle
        .first_loaded_1d_by_source(LoadedSourceFilter::path(
            "jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx",
        ))
        .ok_or_else(|| RSpinError::Parse {
            format: "facade bundle accessor",
            message: "missing selected carbon spectrum".to_owned(),
        })?;
    assert_eq!(carbon.metadata.nucleus, Some(Nucleus::Carbon13));
    assert_eq!(source.format(), "jcamp_dx");

    let hsqc = bundle
        .first_2d_by_sources([LoadedSourceFilter::path_prefix("jeol")])
        .ok_or_else(|| RSpinError::Parse {
            format: "facade bundle accessor",
            message: "missing JEOL two-dimensional spectrum".to_owned(),
        })?;
    assert!(hsqc.shape().0 > 0);
    Ok(())
}

#[test]
fn prelude_exports_strict_bundle_loader_helpers() -> Result<()> {
    let fixture_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../rspin-io/testdata/zenodo_7100132");

    let single = load_spectra_strict_relative_to(&fixture_root, "varian_1h")?;
    assert_eq!(single.len(), 1);
    assert!(single.has_source_path("varian_1h"));

    let many = load_spectra_many_strict_relative_to(&fixture_root, ["varian_1h"])?;
    assert_eq!(many.len(), 1);
    assert!(many.warnings().is_empty());

    let malformed_fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../rspin-io/testdata/zenodo_7100132/empty_jcamp/empty.jdx");
    let bad = load_spectra_strict(malformed_fixture);
    let Err(error) = bad else {
        return Err(RSpinError::Parse {
            format: "prelude strict bundle loader",
            message: "strict loading should reject malformed candidates".to_owned(),
        });
    };
    assert!(error.to_string().contains("missing XYDATA values"));
    Ok(())
}

#[test]
fn prelude_exports_bundle_summary_loader_helpers() -> Result<()> {
    let fixture_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../rspin-io/testdata/zenodo_7100132");

    let summary = load_spectra_summary_relative_to(&fixture_root, "varian_1h")?;
    assert_eq!(summary.spectra(), 1);
    assert_eq!(
        summary.source_vendor_count(LoadedSourceVendor::AgilentVarian),
        1
    );

    let many = load_spectra_many_summary_relative_to(
        &fixture_root,
        ["varian_1h", "bruker_without_expno"],
    )?;
    assert_eq!(many.spectra(), 3);
    assert_eq!(many.source_vendor_count(LoadedSourceVendor::Bruker), 2);

    let strict = load_spectra_many_summary_strict_relative_to(&fixture_root, ["varian_1h"])?;
    assert_eq!(strict.spectra_1d(), 1);

    let raw = RSpinReader::new()
        .raw_sources()
        .read_summary_relative_to(&fixture_root, "bruker_without_expno")?;
    assert_eq!(raw.source_data_kind_count(LoadedSourceDataKind::Raw), 1);
    Ok(())
}

#[test]
fn prelude_exports_short_bundle_load_aliases() -> Result<()> {
    let fixture_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../rspin-io/testdata/zenodo_7100132");

    let bundle = load_relative_to(&fixture_root, "varian_1h")?;
    assert_eq!(bundle.len(), 1);
    assert_eq!(
        bundle.source_vendor_count(LoadedSourceVendor::AgilentVarian),
        1
    );
    assert_eq!(
        load_summary_relative_to(&fixture_root, "varian_1h")?.spectra(),
        1
    );
    assert_eq!(
        load_many_summary_relative_to(&fixture_root, ["varian_1h"])?.spectra(),
        1
    );
    Ok(())
}

#[test]
fn prelude_exports_discovered_summary_loader_helpers() -> Result<()> {
    let fixture_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../rspin-io/testdata/zenodo_7100132");
    let sources: Vec<DiscoveredSpectrumSource> = discover_spectra(&fixture_root)?;

    let raw = load_discovered_spectra_summary_by_source_relative_to(
        &fixture_root,
        &sources,
        LoadedSourceFilter::raw(),
    )?;
    assert_eq!(raw.spectra(), 2);
    assert_eq!(raw.source_data_kind_count(LoadedSourceDataKind::Raw), 2);

    let selected = RSpinReader::new().read_discovered_summary_by_sources(
        &fixture_root,
        &sources,
        [LoadedSourceFilter::path("bruker_without_expno/pdata/1")],
    )?;
    assert_eq!(selected.spectra(), 1);
    assert_eq!(
        selected.source_format_count(LoadedSourceFormat::BrukerProcessed),
        1
    );

    let selected_by_path = load_discovered_spectra_summary_by_source_path(
        &fixture_root,
        &sources,
        "bruker_without_expno/pdata/1",
    )?;
    assert_eq!(selected_by_path, selected);

    let selected_by_prefix = load_discovered_spectra_summary_by_source_path_prefix(
        &fixture_root,
        &sources,
        "bruker_without_expno",
    )?;
    assert_eq!(selected_by_prefix.spectra(), 2);
    assert_eq!(
        selected_by_prefix.source_vendor_count(LoadedSourceVendor::Bruker),
        2
    );

    let selected_by_path_reader = RSpinReader::new().read_discovered_summary_by_source_path(
        &fixture_root,
        &sources,
        "bruker_without_expno/pdata/1",
    )?;
    assert_eq!(selected_by_path_reader, selected);

    let selected_by_prefix_reader = RSpinReader::new()
        .read_discovered_summary_by_source_path_prefix(
            &fixture_root,
            &sources,
            "bruker_without_expno",
        )?;
    assert_eq!(selected_by_prefix_reader.spectra(), 2);

    let strict = load_discovered_spectra_summary_strict_by_source(
        &fixture_root,
        &sources,
        LoadedSourceFilter::vendor("varian"),
    )?;
    assert_eq!(strict.spectra(), 1);
    let strict_by_path = load_discovered_spectra_summary_strict_by_source_path(
        &fixture_root,
        &sources,
        "varian_1h",
    )?;
    assert_eq!(strict_by_path, strict);
    let strict_by_prefix = load_discovered_spectra_summary_strict_by_source_path_prefix(
        &fixture_root,
        &sources,
        "bruker_without_expno",
    )?;
    assert_eq!(strict_by_prefix.spectra(), 2);
    Ok(())
}

#[test]
fn prelude_exports_discovered_summary_prefix_set_helpers() -> Result<()> {
    let fixture_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../rspin-io/testdata/zenodo_7100132");
    let sources: Vec<DiscoveredSpectrumSource> = discover_spectra(&fixture_root)?;

    let selected_by_prefixes = load_discovered_spectra_summary_by_source_path_prefixes_relative_to(
        &fixture_root,
        &sources,
        ["missing", "bruker_without_expno", "varian_1h"],
    )?;
    assert_eq!(selected_by_prefixes.spectra(), 3);
    assert_eq!(selected_by_prefixes.warnings(), 0);
    assert_eq!(
        selected_by_prefixes.source_vendor_count(LoadedSourceVendor::Bruker),
        2
    );
    assert_eq!(
        selected_by_prefixes.source_vendor_count(LoadedSourceVendor::AgilentVarian),
        1
    );

    let selected_by_prefixes_alias = load_discovered_spectra_summary_by_source_path_prefixes(
        &fixture_root,
        &sources,
        ["bruker_without_expno", "varian_1h"],
    )?;
    assert_eq!(selected_by_prefixes_alias, selected_by_prefixes);

    let selected_by_prefixes_reader = RSpinReader::new()
        .read_discovered_summary_by_source_path_prefixes(
            &fixture_root,
            &sources,
            ["bruker_without_expno", "varian_1h"],
        )?;
    assert_eq!(selected_by_prefixes_reader, selected_by_prefixes);

    let strict_by_prefixes =
        load_discovered_spectra_summary_strict_by_source_path_prefixes_relative_to(
            &fixture_root,
            &sources,
            ["missing", "bruker_without_expno", "varian_1h"],
        )?;
    assert_eq!(strict_by_prefixes.spectra(), 3);
    let strict_by_prefixes_alias = load_discovered_spectra_summary_strict_by_source_path_prefixes(
        &fixture_root,
        &sources,
        ["bruker_without_expno", "varian_1h"],
    )?;
    assert_eq!(strict_by_prefixes_alias, strict_by_prefixes);
    Ok(())
}

#[test]
fn prelude_exports_dimension_bundle_loader_helpers() -> Result<()> {
    let fixture_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../rspin-io/testdata/zenodo_7100132");
    let mixed = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../rspin-io/testdata/nmrxiv/cc0/myrcene");

    let one_d = load_spectra_1d_relative_to(&fixture_root, "bruker_without_expno")?;
    assert_eq!(one_d.len_1d(), 2);
    assert_eq!(one_d.len_2d(), 0);
    let one_d_summary = load_spectra_1d_summary_relative_to(&fixture_root, "bruker_without_expno")?;
    assert_eq!(one_d_summary, one_d.summary());

    let two_d = load_spectra_2d_relative_to(&mixed, "bruker_cosy_raw")?;
    assert_eq!(two_d.len_2d(), 1);
    assert_eq!(two_d.source_vendor_count(LoadedSourceVendor::Bruker), 1);
    let two_d_summary = load_spectra_2d_summary_relative_to(&mixed, "bruker_cosy_raw")?;
    assert_eq!(two_d_summary, two_d.summary());

    let many = load_spectra_1d_many_relative_to(&fixture_root, ["varian_1h"])?;
    assert_eq!(many.len_1d(), 1);
    let many_summary = load_spectra_1d_many_summary_relative_to(&fixture_root, ["varian_1h"])?;
    assert_eq!(many_summary, many.summary());

    let strict = load_spectra_1d_many_strict_relative_to(&fixture_root, ["varian_1h"])?;
    assert_eq!(strict.len_1d(), 1);
    let strict_summary =
        load_spectra_1d_many_summary_strict_relative_to(&fixture_root, ["varian_1h"])?;
    assert_eq!(strict_summary, strict.summary());

    let strict_2d = load_spectra_2d_strict_relative_to(&mixed, "bruker_cosy_raw")?;
    assert_eq!(strict_2d.len_2d(), 1);
    let strict_2d_summary = load_spectra_2d_summary_strict_relative_to(&mixed, "bruker_cosy_raw")?;
    assert_eq!(strict_2d_summary, strict_2d.summary());

    let reader = RSpinReader::new()
        .source_vendor("bruker")
        .read_bundle_2d_many_relative_to(&mixed, ["bruker_cosy_raw", "jeol"])?;
    assert_eq!(reader.len_2d(), 1);
    assert!(reader.has_source_path("bruker_cosy_raw"));
    let reader_summary = RSpinReader::new()
        .source_vendor("bruker")
        .read_bundle_2d_summary_many_relative_to(&mixed, ["bruker_cosy_raw", "jeol"])?;
    assert_eq!(reader_summary, reader.summary());
    Ok(())
}

fn assert_multi_path_source_counts(bundle: &SpectrumBundle, summary: &SpectrumBundleSummary) {
    assert_eq!(bundle.source_paths().count(), 3);
    assert_eq!(
        bundle.source_path_count(std::path::Path::new("varian_1h")),
        1
    );
    assert!(
        bundle
            .source_path_counts()
            .contains(&SourcePathCount::new("varian_1h", 1))
    );
    assert_eq!(summary.source_path_count("varian_1h"), 1);
    assert!(summary.has_source_path_prefix("bruker_without_expno"));
    assert!(bundle.has_source_path(std::path::Path::new("varian_1h")));
    let empty_warning_paths: Vec<WarningPathCount> = Vec::new();
    assert_eq!(summary.warning_path_counts(), empty_warning_paths);
}

#[test]
fn prelude_exports_source_discovery_metadata() -> Result<()> {
    let formats: Vec<LoadedSourceFormatInfo> = supported_bundle_source_formats();
    assert_eq!(formats, RSpinReader::supported_source_formats());
    assert!(formats.iter().any(|info| {
        info.name == "jcamp_dx"
            && info.vendor.is_none()
            && info.data_kind == LoadedSourceDataKind::Other
            && info.extensions.contains(&"jdx")
            && info.path_markers.is_empty()
    }));

    let vendors: Vec<LoadedSourceVendorInfo> = supported_bundle_source_vendors();
    assert_eq!(vendors, RSpinReader::supported_source_vendors());
    let bruker = vendors
        .iter()
        .find(|info| info.name == "bruker")
        .ok_or_else(|| RSpinError::Parse {
            format: "spectrum bundle source metadata",
            message: "missing Bruker source vendor metadata".to_owned(),
        })?;
    assert!(bruker.source_formats.contains(&"bruker_fid"));
    assert!(bruker.source_formats.contains(&"bruker_ser"));

    let data_kinds: Vec<LoadedSourceDataKindInfo> = supported_bundle_source_data_kinds();
    assert_eq!(data_kinds, RSpinReader::supported_source_data_kinds());
    let raw = data_kinds
        .iter()
        .find(|info| info.name == "raw")
        .ok_or_else(|| RSpinError::Parse {
            format: "spectrum bundle source metadata",
            message: "missing raw source data-kind metadata".to_owned(),
        })?;
    assert!(raw.source_formats.contains(&"agilent_fid"));
    assert!(!raw.source_formats.contains(&"jcamp_dx"));
    Ok(())
}

#[test]
fn prelude_exports_source_candidate_discovery() -> Result<()> {
    let fixture_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../rspin-io/testdata/zenodo_7100132");

    let sources: Vec<DiscoveredSpectrumSource> = discover_spectra(&fixture_root)?;
    let summary: DiscoveredSpectrumSummary = summarize_discovered_spectra(&sources);
    let direct_summary = discover_spectra_summary(&fixture_root)?;
    assert_eq!(direct_summary, summary);
    assert_eq!(summary.source_format_count("jdx"), 2);
    assert_eq!(summary.source_data_kind_count(LoadedSourceDataKind::Raw), 2);
    assert_eq!(summary.source_path_count("varian_1h"), 1);
    assert!(summary.has_source_path_prefix("bruker_without_expno"));
    assert_eq!(summary.source_count(LoadedSourceFilter::format("jdx")), 2);
    assert_eq!(summary.source_count(LoadedSourceFilter::raw()), 2);
    assert!(summary.has_source(LoadedSourceFilter::path_prefix("bruker_without_expno")));
    assert!(
        summary
            .source_paths
            .contains(&DiscoveredSpectrumPathCount::new("varian_1h", 1))
    );
    assert!(summary.has_dimension(DiscoveredSpectrumDimension::OneD));
    let varian = sources
        .iter()
        .find(|source| {
            source.path() == Some(std::path::Path::new("varian_1h"))
                && source.is_format(LoadedSourceFormat::AgilentFid)
        })
        .ok_or_else(|| RSpinError::Parse {
            format: "spectrum bundle source discovery",
            message: "missing varian source candidate".to_owned(),
        })?;
    assert!(varian.is_1d());
    assert_eq!(varian.data_kind(), LoadedSourceDataKind::Raw);
    assert!(varian.matches_source(LoadedSourceFilter::vendor("varian")));

    let discovered_bruker =
        discover_spectra_by_source(&fixture_root, LoadedSourceFilter::vendor("bruker"))?;
    assert_eq!(discovered_bruker.len(), 2);
    let discovered_selected = discover_spectra_by_sources_relative_to(
        &fixture_root,
        "empty_jcamp/empty.jdx",
        [LoadedSourceFilter::format("jdx")],
    )?;
    assert_eq!(discovered_selected.len(), 2);
    let discovered_many = discover_spectra_many_by_source_relative_to(
        &fixture_root,
        ["varian_1h", "bruker_without_expno"],
        LoadedSourceFilter::raw(),
    )?;
    assert_eq!(discovered_many.len(), 2);
    let discovered_many_sources = discover_spectra_many_by_sources_relative_to(
        &fixture_root,
        ["empty_jcamp/empty.jdx", "bruker_without_expno"],
        [
            LoadedSourceFilter::format("jdx"),
            LoadedSourceFilter::processed(),
        ],
    )?;
    assert_eq!(discovered_many_sources.len(), 3);
    let discovered_summary =
        discover_spectra_many_summary_relative_to(&fixture_root, ["varian_1h"])?;
    assert_eq!(discovered_summary.sources(), 1);
    assert_eq!(
        discovered_summary.source_vendor_count(LoadedSourceVendor::AgilentVarian),
        1
    );
    let discovered_1d = discover_spectra_1d_relative_to(&fixture_root, "varian_1h")?;
    assert_eq!(discovered_1d.len(), 1);
    assert!(discovered_1d.iter().all(DiscoveredSpectrumSource::is_1d));
    assert_eq!(
        discover_spectra_1d_many_summary_relative_to(&fixture_root, ["varian_1h"])?.sources_1d(),
        1
    );

    let myrcene_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../rspin-io/testdata/nmrxiv/cc0/myrcene");
    let discovered_2d =
        discover_spectra_2d_relative_to(&myrcene_root, "jeol/myrcene_hsqc_400mhz.jdf")?;
    assert_eq!(discovered_2d.len(), 1);
    assert!(discovered_2d.iter().all(DiscoveredSpectrumSource::is_2d));
    let discovered_2d_summary = discover_spectra_2d_summary(&myrcene_root)?;
    assert_eq!(discovered_2d_summary.sources_1d(), 0);
    assert!(discovered_2d_summary.sources_2d() >= discovered_2d.len());

    let processed = RSpinReader::new()
        .processed_sources()
        .discover_relative_to(&fixture_root, "bruker_without_expno")?;
    assert_eq!(processed.len(), 1);
    assert!(processed.iter().all(DiscoveredSpectrumSource::is_processed));
    let processed_summary = RSpinReader::new()
        .processed_sources()
        .discover_summary_relative_to(&fixture_root, "bruker_without_expno")?;
    assert_eq!(processed_summary.sources(), processed.len());

    let loaded = load_discovered_spectra_relative_to(&fixture_root, &processed)?;
    assert_eq!(loaded.len(), 1);
    assert_eq!(
        loaded.source_format_count(LoadedSourceFormat::BrukerProcessed),
        1
    );

    let loaded = RSpinReader::new().read_discovered(&fixture_root, &processed)?;
    assert_eq!(loaded.len(), 1);
    Ok(())
}

#[test]
fn prelude_exports_discovered_source_filter_loaders() -> Result<()> {
    let fixture_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../rspin-io/testdata/zenodo_7100132");
    let sources: Vec<DiscoveredSpectrumSource> = discover_spectra(&fixture_root)?;

    let selected =
        select_discovered_spectra_by_source(&sources, LoadedSourceFilter::vendor("varian"));
    assert_eq!(selected.len(), 1);
    assert_eq!(
        selected[0].vendor(),
        Some(LoadedSourceVendor::AgilentVarian)
    );
    let loaded = selected[0].load_relative_to(&fixture_root)?;
    assert_eq!(loaded.len(), 1);
    assert_eq!(
        loaded.source_vendor_count(LoadedSourceVendor::AgilentVarian),
        1
    );
    let spectrum = selected[0].load_1d_relative_to(&fixture_root)?;
    assert_eq!(spectrum.len(), 16_384);
    let (_, source) = selected[0].load_1d_with_source_relative_to(&fixture_root)?;
    assert_eq!(source.path(), Some(std::path::Path::new("varian_1h")));
    let spectrum =
        RSpinReader::new().read_discovered_1d_relative_to(&fixture_root, [selected[0]])?;
    assert_eq!(spectrum.len(), 16_384);
    let (_, source) =
        RSpinReader::new().read_discovered_1d_with_source(&fixture_root, [selected[0]])?;
    assert_eq!(source.path(), Some(std::path::Path::new("varian_1h")));
    let spectrum = RSpinReader::new().read_discovered_1d_by_source_relative_to(
        &fixture_root,
        &sources,
        LoadedSourceFilter::vendor("varian"),
    )?;
    assert_eq!(spectrum.len(), 16_384);
    let (_, source) = RSpinReader::new().read_discovered_1d_with_source_by_sources(
        &fixture_root,
        &sources,
        [LoadedSourceFilter::path("varian_1h")],
    )?;
    assert_eq!(source.format(), "agilent_fid");

    let selected = select_discovered_spectra_by_sources(
        &sources,
        [
            LoadedSourceFilter::path("varian_1h"),
            LoadedSourceFilter::path("bruker_without_expno/pdata/1"),
        ],
    );
    assert_eq!(selected.len(), 2);

    let loaded = load_discovered_spectra_by_source_relative_to(
        &fixture_root,
        &sources,
        LoadedSourceFilter::vendor("varian"),
    )?;
    assert_eq!(loaded.len(), 1);
    assert_eq!(
        loaded.source_vendor_count(LoadedSourceVendor::AgilentVarian),
        1
    );

    let loaded = load_discovered_spectra_by_source(
        &fixture_root,
        &sources,
        LoadedSourceFilter::processed(),
    )?;
    assert_eq!(loaded.len(), 1);
    assert_eq!(
        loaded.source_format_count(LoadedSourceFormat::BrukerProcessed),
        1
    );

    let loaded = load_discovered_spectra_by_sources(
        &fixture_root,
        &sources,
        [
            LoadedSourceFilter::path("varian_1h"),
            LoadedSourceFilter::path("bruker_without_expno/pdata/1"),
        ],
    )?;
    assert_eq!(loaded.len(), 2);
    assert!(loaded.has_source_path(std::path::Path::new("varian_1h")));

    let loaded = load_discovered_spectra_by_sources_relative_to(
        &fixture_root,
        &sources,
        [LoadedSourceFilter::path("bruker_without_expno/pdata/1")],
    )?;
    assert_eq!(loaded.len(), 1);

    let loaded = RSpinReader::new().read_discovered_by_source(
        &fixture_root,
        &sources,
        LoadedSourceFilter::processed(),
    )?;
    assert_eq!(loaded.len(), 1);

    let loaded = RSpinReader::new().read_discovered_by_sources(
        &fixture_root,
        &sources,
        [LoadedSourceFilter::path("varian_1h")],
    )?;
    assert_eq!(loaded.len(), 1);
    Ok(())
}

#[test]
fn prelude_exports_discovered_source_path_loaders() -> Result<()> {
    let fixture_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../rspin-io/testdata/zenodo_7100132");
    let sources: Vec<DiscoveredSpectrumSource> = discover_spectra(&fixture_root)?;

    let loaded = load_discovered_spectra_by_source_path(&fixture_root, &sources, "varian_1h")?;
    assert_eq!(loaded.len(), 1);
    assert!(loaded.has_source_path(std::path::Path::new("varian_1h")));

    let loaded = load_discovered_spectra_by_source_path_prefix(
        &fixture_root,
        &sources,
        "bruker_without_expno",
    )?;
    assert_eq!(loaded.len(), 2);

    let loaded =
        RSpinReader::new().read_discovered_by_source_path(&fixture_root, &sources, "varian_1h")?;
    assert_eq!(loaded.len(), 1);

    let loaded = RSpinReader::new().read_discovered_by_source_path_prefix(
        &fixture_root,
        &sources,
        "bruker_without_expno",
    )?;
    assert_eq!(loaded.len(), 2);

    let loaded = load_discovered_spectra_by_source_path_prefixes(
        &fixture_root,
        &sources,
        ["missing", "bruker_without_expno", "varian_1h"],
    )?;
    assert_eq!(loaded.len(), 3);
    assert!(loaded.has_source_path(std::path::Path::new("varian_1h")));

    let loaded = RSpinReader::new().read_discovered_by_source_path_prefixes_relative_to(
        &fixture_root,
        &sources,
        ["missing", "bruker_without_expno", "varian_1h"],
    )?;
    assert_eq!(loaded.len(), 3);

    let loaded =
        load_discovered_spectra_strict_by_source_path(&fixture_root, &sources, "varian_1h")?;
    assert_eq!(loaded.len(), 1);

    let loaded = load_discovered_spectra_strict_by_source_path_prefix(
        &fixture_root,
        &sources,
        "bruker_without_expno",
    )?;
    assert_eq!(loaded.len(), 2);
    Ok(())
}

#[test]
fn prelude_exports_discovered_dimension_bundle_loaders() -> Result<()> {
    let myrcene_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../rspin-io/testdata/nmrxiv/cc0/myrcene");
    let myrcene_sources: Vec<DiscoveredSpectrumSource> = discover_spectra(&myrcene_root)?;

    let selected_1d = select_discovered_spectra_1d(&myrcene_sources);
    assert_eq!(selected_1d.len(), 5);
    let jeol_1d = select_discovered_spectra_1d_by_source(
        &myrcene_sources,
        LoadedSourceFilter::vendor("jeol"),
    );
    assert_eq!(jeol_1d.len(), 2);
    let jeol_1d_many = select_discovered_spectra_1d_by_sources(
        &myrcene_sources,
        [LoadedSourceFilter::vendor("jeol")],
    );
    assert_eq!(jeol_1d_many, jeol_1d);
    let selected_2d =
        select_discovered_spectra_by_dimension(&myrcene_sources, DiscoveredSpectrumDimension::TwoD);
    assert_eq!(
        selected_2d.len(),
        select_discovered_spectra_2d(&myrcene_sources).len()
    );
    let hsqc_path = "jeol/myrcene_hsqc_400mhz.jdf";
    let hsqc_candidate = select_discovered_spectra_2d_by_source(
        &myrcene_sources,
        LoadedSourceFilter::path(hsqc_path),
    );
    assert_eq!(hsqc_candidate.len(), 1);
    assert_eq!(
        select_discovered_spectra_2d_by_source_path(&myrcene_sources, hsqc_path),
        hsqc_candidate
    );
    let hsqc_candidates = select_discovered_spectra_2d_by_sources(
        &myrcene_sources,
        [LoadedSourceFilter::path(hsqc_path)],
    );
    assert_eq!(hsqc_candidates, hsqc_candidate);
    let hsqc_candidates_by_dimension = select_discovered_spectra_by_dimension_and_sources(
        &myrcene_sources,
        DiscoveredSpectrumDimension::TwoD,
        [LoadedSourceFilter::path(hsqc_path)],
    );
    assert_eq!(hsqc_candidates_by_dimension, hsqc_candidate);
    let proton_path = "jeol/myrcene_1h_400mhz.jdf";
    assert_eq!(
        select_discovered_spectra_1d_by_source_path(&myrcene_sources, proton_path).len(),
        1
    );
    assert_eq!(
        select_discovered_spectra_1d_by_source_path_prefix(&myrcene_sources, "jeol").len(),
        2
    );
    assert_eq!(
        select_discovered_spectra_by_source_path(&myrcene_sources, proton_path).len(),
        1
    );
    assert_eq!(
        select_discovered_spectra_by_source_path_prefix(&myrcene_sources, "jeol").len(),
        3
    );
    let selected_unknown = select_discovered_spectra_by_dimension_and_source(
        &myrcene_sources,
        DiscoveredSpectrumDimension::Unknown,
        LoadedSourceFilter::vendor("jeol"),
    );
    assert!(selected_unknown.is_empty());

    let jeol_1d = load_discovered_spectra_1d_by_source_relative_to(
        &myrcene_root,
        &myrcene_sources,
        LoadedSourceFilter::vendor("jeol"),
    )?;
    assert_eq!(jeol_1d.len_1d(), 2);
    assert_eq!(jeol_1d.len_2d(), 0);

    let hsqc = RSpinReader::new().read_discovered_bundle_2d_by_sources(
        &myrcene_root,
        &myrcene_sources,
        [LoadedSourceFilter::path(hsqc_path)],
    )?;
    assert_eq!(hsqc.len_2d(), 1);

    let strict_hsqc = load_discovered_spectra_2d_strict_by_sources(
        &myrcene_root,
        &myrcene_sources,
        [LoadedSourceFilter::path(hsqc_path)],
    )?;
    assert_eq!(strict_hsqc.len_2d(), 1);
    Ok(())
}

#[test]
fn prelude_exports_discovered_dimension_source_path_loaders() -> Result<()> {
    let myrcene_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../rspin-io/testdata/nmrxiv/cc0/myrcene");
    let sources: Vec<DiscoveredSpectrumSource> = discover_spectra(&myrcene_root)?;
    let proton_path = "jeol/myrcene_1h_400mhz.jdf";
    let hsqc_path = "jeol/myrcene_hsqc_400mhz.jdf";

    let proton = load_discovered_spectra_1d_by_source_path(&myrcene_root, &sources, proton_path)?;
    assert_eq!(proton.len_1d(), 1);
    assert!(proton.has_source_path(proton_path));

    let jeol_1d =
        load_discovered_spectra_1d_by_source_path_prefix(&myrcene_root, &sources, "jeol")?;
    assert_eq!(jeol_1d.len_1d(), 2);

    let strict_jeol_1d =
        load_discovered_spectra_1d_strict_by_source_path_prefix(&myrcene_root, &sources, "jeol")?;
    assert_eq!(strict_jeol_1d.len_1d(), 2);

    let hsqc = load_discovered_spectra_2d_by_source_path(&myrcene_root, &sources, hsqc_path)?;
    assert_eq!(hsqc.len_2d(), 1);
    assert!(hsqc.has_source_path(hsqc_path));

    let hsqc = RSpinReader::new().read_discovered_bundle_2d_by_source_path_prefix(
        &myrcene_root,
        &sources,
        "jeol",
    )?;
    assert_eq!(hsqc.len_2d(), 1);

    let strict_hsqc =
        load_discovered_spectra_2d_strict_by_source_path(&myrcene_root, &sources, hsqc_path)?;
    assert_eq!(strict_hsqc.len_2d(), 1);
    Ok(())
}

#[test]
fn prelude_exports_discovered_dimension_source_path_prefix_set_loaders() -> Result<()> {
    let myrcene_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../rspin-io/testdata/nmrxiv/cc0/myrcene");
    let sources: Vec<DiscoveredSpectrumSource> = discover_spectra(&myrcene_root)?;

    let one_d = load_discovered_spectra_1d_by_source_path_prefixes(
        &myrcene_root,
        &sources,
        ["jcamp", "jeol/myrcene_1h_400mhz.jdf"],
    )?;
    assert_eq!(one_d.len_1d(), 3);
    assert_eq!(one_d.len_2d(), 0);

    let one_d_reader = RSpinReader::new()
        .read_discovered_bundle_1d_by_source_path_prefixes_relative_to(
            &myrcene_root,
            &sources,
            ["jcamp", "jeol/myrcene_1h_400mhz.jdf"],
        )?;
    assert_eq!(one_d_reader, one_d);
    let one_d_summary = load_discovered_spectra_1d_summary_by_source_path_prefixes(
        &myrcene_root,
        &sources,
        ["jcamp", "jeol/myrcene_1h_400mhz.jdf"],
    )?;
    assert_eq!(one_d_summary, one_d.summary());
    let one_d_summary_reader = RSpinReader::new()
        .read_discovered_bundle_1d_summary_by_source_path_prefixes_relative_to(
            &myrcene_root,
            &sources,
            ["jcamp", "jeol/myrcene_1h_400mhz.jdf"],
        )?;
    assert_eq!(one_d_summary_reader, one_d_summary);

    let strict_one_d = load_discovered_spectra_1d_strict_by_source_path_prefixes(
        &myrcene_root,
        &sources,
        ["jcamp", "jeol/myrcene_1h_400mhz.jdf"],
    )?;
    assert_eq!(strict_one_d, one_d);
    let strict_one_d_summary =
        load_discovered_spectra_1d_summary_strict_by_source_path_prefixes_relative_to(
            &myrcene_root,
            &sources,
            ["jcamp", "jeol/myrcene_1h_400mhz.jdf"],
        )?;
    assert_eq!(strict_one_d_summary, strict_one_d.summary());

    let two_d = load_discovered_spectra_2d_by_source_path_prefixes_relative_to(
        &myrcene_root,
        &sources,
        ["bruker_cosy_raw", "jeol/myrcene_hsqc_400mhz.jdf"],
    )?;
    assert_eq!(two_d.len_1d(), 0);
    assert_eq!(two_d.len_2d(), 2);
    let two_d_summary = load_discovered_spectra_2d_summary_by_source_path_prefixes(
        &myrcene_root,
        &sources,
        ["bruker_cosy_raw", "jeol/myrcene_hsqc_400mhz.jdf"],
    )?;
    assert_eq!(two_d_summary, two_d.summary());
    let two_d_summary_reader = RSpinReader::new()
        .read_discovered_bundle_2d_summary_by_source_path_prefixes(
            &myrcene_root,
            &sources,
            ["bruker_cosy_raw", "jeol/myrcene_hsqc_400mhz.jdf"],
        )?;
    assert_eq!(two_d_summary_reader, two_d_summary);

    let strict_two_d = load_discovered_spectra_2d_strict_by_source_path_prefixes(
        &myrcene_root,
        &sources,
        ["bruker_cosy_raw", "jeol/myrcene_hsqc_400mhz.jdf"],
    )?;
    assert_eq!(strict_two_d, two_d);
    let strict_two_d_summary =
        load_discovered_spectra_2d_summary_strict_by_source_path_prefixes_relative_to(
            &myrcene_root,
            &sources,
            ["bruker_cosy_raw", "jeol/myrcene_hsqc_400mhz.jdf"],
        )?;
    assert_eq!(strict_two_d_summary, strict_two_d.summary());
    Ok(())
}

#[test]
fn prelude_exports_discovered_dimension_summary_source_loaders() -> Result<()> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../rspin-io/testdata/nmrxiv/cc0/myrcene");
    let sources: Vec<DiscoveredSpectrumSource> = discover_spectra(&root)?;
    let two_d_sources: Vec<_> = sources.iter().filter(|source| source.is_2d()).collect();

    let one_d =
        load_discovered_spectra_1d_by_source(&root, &sources, LoadedSourceFilter::vendor("jeol"))?;
    let one_d_summary = load_discovered_spectra_1d_summary_by_source(
        &root,
        &sources,
        LoadedSourceFilter::vendor("jeol"),
    )?;
    assert_eq!(one_d_summary, one_d.summary());
    assert_eq!(
        load_discovered_spectra_1d_summary_strict_by_source(
            &root,
            &sources,
            LoadedSourceFilter::vendor("jeol"),
        )?,
        one_d_summary
    );

    let one_d_all_summary = load_discovered_spectra_1d_summary(&root, &sources)?;
    assert_eq!(
        RSpinReader::new().read_discovered_bundle_1d_summary_by_sources(
            &root,
            &sources,
            std::iter::empty::<LoadedSourceFilter>(),
        )?,
        one_d_all_summary
    );

    let two_d_filters = [
        LoadedSourceFilter::path_prefix("bruker_cosy_raw"),
        LoadedSourceFilter::path("jeol/myrcene_hsqc_400mhz.jdf"),
    ];
    let two_d = load_discovered_spectra_2d_by_sources(&root, &sources, two_d_filters.clone())?;
    let two_d_summary =
        load_discovered_spectra_2d_summary_by_sources(&root, &sources, two_d_filters.clone())?;
    assert_eq!(two_d_summary, two_d.summary());
    assert_eq!(
        load_discovered_spectra_2d_summary_strict_by_sources(&root, &sources, two_d_filters)?,
        two_d_summary
    );

    let two_d_all_summary = load_discovered_spectra_2d_summary(&root, two_d_sources.clone())?;
    assert_eq!(
        RSpinReader::new().read_discovered_bundle_2d_summary_by_sources_relative_to(
            &root,
            two_d_sources,
            std::iter::empty::<LoadedSourceFilter>(),
        )?,
        two_d_all_summary
    );

    Ok(())
}

#[test]
fn prelude_exports_discovered_source_slice_methods() -> Result<()> {
    let myrcene_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../rspin-io/testdata/nmrxiv/cc0/myrcene");
    let sources: Vec<DiscoveredSpectrumSource> = discover_spectra(&myrcene_root)?;
    let hsqc_path = "jeol/myrcene_hsqc_400mhz.jdf";
    let proton_path = "jeol/myrcene_1h_400mhz.jdf";

    assert_eq!(sources.summarize().sources(), sources.len());
    assert_eq!(sources.select_1d(), select_discovered_spectra_1d(&sources));
    assert_eq!(sources.select_2d(), select_discovered_spectra_2d(&sources));
    assert_eq!(
        sources.select_1d_by_source(LoadedSourceVendor::Jeol),
        select_discovered_spectra_1d_by_source(&sources, LoadedSourceVendor::Jeol)
    );
    assert_eq!(
        sources.select_1d_by_sources([LoadedSourceFilter::vendor("jeol")]),
        select_discovered_spectra_1d_by_sources(&sources, [LoadedSourceFilter::vendor("jeol")])
    );
    assert_eq!(
        sources.select_1d_by_source_path(proton_path),
        select_discovered_spectra_1d_by_source_path(&sources, proton_path)
    );
    assert_eq!(
        sources.select_1d_by_source_path_prefix("jeol"),
        select_discovered_spectra_1d_by_source_path_prefix(&sources, "jeol")
    );
    assert_eq!(
        sources.select_2d_by_source(LoadedSourceFilter::path(hsqc_path)),
        select_discovered_spectra_2d_by_source(&sources, LoadedSourceFilter::path(hsqc_path))
    );
    assert_eq!(
        sources.select_2d_by_source_path(hsqc_path),
        select_discovered_spectra_2d_by_source_path(&sources, hsqc_path)
    );
    assert_eq!(
        sources.select_2d_by_source_path_prefix("bruker_cosy_raw"),
        select_discovered_spectra_2d_by_source_path_prefix(&sources, "bruker_cosy_raw")
    );
    assert_eq!(
        sources.select_2d_by_sources([LoadedSourceFilter::path(hsqc_path)]),
        select_discovered_spectra_2d_by_sources(&sources, [LoadedSourceFilter::path(hsqc_path)])
    );
    assert_eq!(
        sources.select_by_dimension(DiscoveredSpectrumDimension::TwoD),
        select_discovered_spectra_by_dimension(&sources, DiscoveredSpectrumDimension::TwoD)
    );
    assert_eq!(
        sources.select_by_dimension_and_source(
            DiscoveredSpectrumDimension::Unknown,
            LoadedSourceFilter::vendor("jeol"),
        ),
        select_discovered_spectra_by_dimension_and_source(
            &sources,
            DiscoveredSpectrumDimension::Unknown,
            LoadedSourceFilter::vendor("jeol"),
        )
    );
    assert_eq!(
        sources.select_by_dimension_and_sources(
            DiscoveredSpectrumDimension::OneD,
            [LoadedSourceFilter::path(proton_path)],
        ),
        select_discovered_spectra_by_dimension_and_sources(
            &sources,
            DiscoveredSpectrumDimension::OneD,
            [LoadedSourceFilter::path(proton_path)],
        )
    );
    assert_eq!(
        sources.select_by_source(LoadedSourceFilter::path(proton_path)),
        select_discovered_spectra_by_source(&sources, LoadedSourceFilter::path(proton_path))
    );
    assert_eq!(
        sources.select_by_source_path(proton_path),
        select_discovered_spectra_by_source_path(&sources, proton_path)
    );
    assert_eq!(
        sources.select_by_source_path_prefix("jeol"),
        select_discovered_spectra_by_source_path_prefix(&sources, "jeol")
    );
    assert_eq!(
        sources.select_by_source_path_prefixes(["jcamp", "jeol"]),
        select_discovered_spectra_by_source_path_prefixes(&sources, ["jcamp", "jeol"])
    );
    assert_eq!(
        sources.select_1d_by_source_path_prefixes(["jcamp", "jeol"]),
        select_discovered_spectra_1d_by_source_path_prefixes(&sources, ["jcamp", "jeol"])
    );
    assert_eq!(
        sources.select_2d_by_source_path_prefixes(["bruker_cosy_raw", "jeol"]),
        select_discovered_spectra_2d_by_source_path_prefixes(&sources, ["bruker_cosy_raw", "jeol"],)
    );
    assert_eq!(
        sources.select_by_sources([LoadedSourceFilter::path(proton_path)]),
        select_discovered_spectra_by_sources(&sources, [LoadedSourceFilter::path(proton_path)])
    );
    Ok(())
}

#[test]
fn prelude_exports_selected_discovered_source_load_methods() -> Result<()> {
    let myrcene_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../rspin-io/testdata/nmrxiv/cc0/myrcene");
    let sources: Vec<DiscoveredSpectrumSource> = discover_spectra(&myrcene_root)?;

    let jeol_1d = sources.select_1d_by_source(LoadedSourceVendor::Jeol);
    let loaded_1d = jeol_1d.load_1d_relative_to(&myrcene_root)?;
    assert_eq!(loaded_1d.len_1d(), 2);
    assert_eq!(loaded_1d.len_2d(), 0);
    assert_eq!(jeol_1d.load_summary(&myrcene_root)?.spectra_1d(), 2);
    assert_eq!(jeol_1d.load_1d_summary(&myrcene_root)?, loaded_1d.summary());
    assert_eq!(
        jeol_1d.load_1d_summary_strict_relative_to(&myrcene_root)?,
        loaded_1d.summary()
    );

    let hsqc =
        sources.select_2d_by_source(LoadedSourceFilter::path("jeol/myrcene_hsqc_400mhz.jdf"));
    let loaded_2d = hsqc.load_2d(&myrcene_root)?;
    assert_eq!(loaded_2d.len_1d(), 0);
    assert_eq!(loaded_2d.len_2d(), 1);
    assert_eq!(
        hsqc.load_2d_summary_relative_to(&myrcene_root)?,
        loaded_2d.summary()
    );
    assert_eq!(
        hsqc.load_2d_summary_strict(&myrcene_root)?,
        loaded_2d.summary()
    );
    Ok(())
}

#[test]
fn prelude_exports_exact_discovered_free_helpers() -> Result<()> {
    let fixture_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../rspin-io/testdata/zenodo_7100132");
    let sources: Vec<DiscoveredSpectrumSource> = discover_spectra(&fixture_root)?;

    let spectrum = load_discovered_spectrum_1d_by_source_relative_to(
        &fixture_root,
        &sources,
        LoadedSourceFilter::vendor("varian"),
    )?;
    assert_eq!(spectrum.len(), 16_384);
    let (_, source) = load_discovered_spectrum_1d_with_source_by_sources(
        &fixture_root,
        &sources,
        [LoadedSourceFilter::path("varian_1h")],
    )?;
    assert_eq!(source.format(), "agilent_fid");
    Ok(())
}

#[test]
fn prelude_exports_exact_discovered_source_path_helpers() -> Result<()> {
    let fixture_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../rspin-io/testdata/zenodo_7100132");
    let sources: Vec<DiscoveredSpectrumSource> = discover_spectra(&fixture_root)?;

    let spectrum =
        load_discovered_spectrum_1d_by_source_path(&fixture_root, &sources, "varian_1h")?;
    assert_eq!(spectrum.len(), 16_384);
    let (_, source) = load_discovered_spectrum_1d_with_source_by_source_path_prefix(
        &fixture_root,
        &sources,
        "varian_1h",
    )?;
    assert_eq!(source.format(), "agilent_fid");
    let spectrum = load_discovered_spectrum_1d_by_source_path_prefixes(
        &fixture_root,
        &sources,
        ["missing", "varian_1h"],
    )?;
    assert_eq!(spectrum.len(), 16_384);
    let (_, source) = RSpinReader::new().read_discovered_1d_with_source_by_source_path_prefixes(
        &fixture_root,
        &sources,
        ["missing", "varian_1h"],
    )?;
    assert_eq!(source.path(), Some(std::path::Path::new("varian_1h")));

    let myrcene_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../rspin-io/testdata/nmrxiv/cc0/myrcene");
    let myrcene_sources: Vec<DiscoveredSpectrumSource> = discover_spectra(&myrcene_root)?;
    let hsqc_path = "jeol/myrcene_hsqc_400mhz.jdf";

    let hsqc =
        load_discovered_spectrum_2d_by_source_path(&myrcene_root, &myrcene_sources, hsqc_path)?;
    assert_eq!(hsqc.shape(), (1024, 32));
    let (_, source) = RSpinReader::new().read_discovered_2d_with_source_by_source_path_prefix(
        &myrcene_root,
        &myrcene_sources,
        hsqc_path,
    )?;
    assert_eq!(source.path(), Some(std::path::Path::new(hsqc_path)));
    let hsqc = load_discovered_spectrum_2d_by_source_path_prefixes(
        &myrcene_root,
        &myrcene_sources,
        ["missing", hsqc_path],
    )?;
    assert_eq!(hsqc.shape(), (1024, 32));
    let (_, source) = load_discovered_spectrum_2d_with_source_by_source_path_prefixes_relative_to(
        &myrcene_root,
        &myrcene_sources,
        ["missing", hsqc_path],
    )?;
    assert_eq!(source.format(), "jeol_jdf");
    Ok(())
}

#[test]
fn prelude_exports_strict_discovered_free_helpers() -> Result<()> {
    let fixture_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../rspin-io/testdata/zenodo_7100132");
    let sources: Vec<DiscoveredSpectrumSource> = discover_spectra(&fixture_root)?;

    let bundle = load_discovered_spectra_strict_by_source_relative_to(
        &fixture_root,
        &sources,
        LoadedSourceFilter::vendor("varian"),
    )?;
    assert_eq!(bundle.len(), 1);
    let bundle = load_discovered_spectra_strict_by_sources(
        &fixture_root,
        &sources,
        [LoadedSourceFilter::path("varian_1h")],
    )?;
    assert_eq!(bundle.len(), 1);
    Ok(())
}

#[test]
fn prelude_exports_filtered_bundle_loader_helpers() -> Result<()> {
    let fixture_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../rspin-io/testdata/zenodo_7100132");

    let source_path = load_spectra_by_source_path(&fixture_root, "varian_1h")?;
    assert_eq!(source_path.len(), 1);
    assert_eq!(
        source_path.source_vendor_count(LoadedSourceVendor::AgilentVarian),
        1
    );

    let generic_source =
        load_spectra_by_source(&fixture_root, LoadedSourceFilter::vendor("varian"))?;
    assert_eq!(generic_source.len(), 1);
    assert_eq!(
        generic_source.source_vendor_count(LoadedSourceVendor::AgilentVarian),
        1
    );

    let generic_sources = load_spectra_by_sources(
        &fixture_root,
        [
            LoadedSourceFilter::vendor("varian"),
            LoadedSourceFilter::path("bruker_without_expno/pdata/1"),
        ],
    )?;
    assert_eq!(generic_sources.len(), 2);
    assert_eq!(
        generic_sources.source_vendor_count(LoadedSourceVendor::AgilentVarian),
        1
    );
    assert_eq!(
        generic_sources.source_format_count(LoadedSourceFormat::BrukerProcessed),
        1
    );
    assert_eq!(
        generic_sources.source_count(LoadedSourceFilter::vendor("varian")),
        1
    );
    assert_eq!(
        generic_sources
            .loaded_1d_by_source(LoadedSourceFilter::path("bruker_without_expno/pdata/1"))
            .count(),
        1
    );
    assert_eq!(
        generic_sources
            .source_paths_for_source(LoadedSourceFilter::vendor("bruker"))
            .collect::<Vec<_>>(),
        vec![std::path::Path::new("bruker_without_expno/pdata/1")]
    );

    let source_format = load_spectra_by_source_format(&fixture_root, "varian fid")?;
    assert_eq!(source_format.len(), 1);
    assert_eq!(
        source_format.source_format_count(LoadedSourceFormat::AgilentFid),
        1
    );

    let source_vendor =
        load_spectra_by_source_vendor(&fixture_root, LoadedSourceVendor::AgilentVarian)?;
    assert_eq!(source_vendor.len(), 1);
    assert_eq!(
        source_vendor.source_vendor_count(LoadedSourceVendor::AgilentVarian),
        1
    );

    let multi_vendor =
        load_spectra_many_by_source_vendor([fixture_root.join("varian_1h")], "varian")?;
    assert_eq!(multi_vendor.len(), 1);
    assert_eq!(
        multi_vendor.source_vendor_count(LoadedSourceVendor::AgilentVarian),
        1
    );

    let multi_path =
        load_spectra_many_by_source_path_relative_to(&fixture_root, ["varian_1h"], "varian_1h")?;
    assert_eq!(multi_path.len(), 1);
    assert!(multi_path.has_source_path(std::path::Path::new("varian_1h")));

    let generic_many = load_spectra_many_by_source_relative_to(
        &fixture_root,
        ["varian_1h", "bruker_without_expno"],
        LoadedSourceFilter::path("varian_1h"),
    )?;
    assert_eq!(generic_many.len(), 1);
    assert!(generic_many.has_source_path(std::path::Path::new("varian_1h")));

    let generic_many_sources = load_spectra_many_by_sources_relative_to(
        &fixture_root,
        ["varian_1h", "bruker_without_expno"],
        [
            LoadedSourceFilter::path("varian_1h"),
            LoadedSourceFilter::path("bruker_without_expno/pdata/1"),
        ],
    )?;
    assert_eq!(generic_many_sources.len(), 2);
    assert!(generic_many_sources.has_source_path(std::path::Path::new("varian_1h")));
    assert!(
        generic_many_sources.has_source_path(std::path::Path::new("bruker_without_expno/pdata/1"))
    );
    Ok(())
}

#[test]
fn prelude_exports_source_set_loader_helpers() -> Result<()> {
    let fixture_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../rspin-io/testdata/zenodo_7100132");

    let source_formats = load_spectra_by_source_formats(
        &fixture_root,
        [
            LoadedSourceFormat::AgilentFid,
            LoadedSourceFormat::BrukerProcessed,
        ],
    )?;
    assert_eq!(source_formats.len(), 2);
    assert_eq!(
        source_formats.source_format_count(LoadedSourceFormat::AgilentFid),
        1
    );
    assert_eq!(
        source_formats.source_format_count(LoadedSourceFormat::BrukerProcessed),
        1
    );

    let source_vendors = load_spectra_by_source_vendors(
        &fixture_root,
        [
            LoadedSourceVendor::AgilentVarian,
            LoadedSourceVendor::Bruker,
        ],
    )?;
    assert_eq!(source_vendors.len(), 3);
    assert_eq!(
        source_vendors.source_vendor_count(LoadedSourceVendor::AgilentVarian),
        1
    );
    assert_eq!(
        source_vendors.source_vendor_count(LoadedSourceVendor::Bruker),
        2
    );

    let multi_formats = load_spectra_many_by_source_formats(
        [
            fixture_root.join("varian_1h"),
            fixture_root.join("bruker_without_expno"),
        ],
        [
            LoadedSourceFormat::AgilentFid,
            LoadedSourceFormat::BrukerFid,
        ],
    )?;
    assert_eq!(multi_formats.len(), 2);
    assert_eq!(
        multi_formats.source_format_count(LoadedSourceFormat::AgilentFid),
        1
    );
    assert_eq!(
        multi_formats.source_format_count(LoadedSourceFormat::BrukerFid),
        1
    );

    let multi_vendors = load_spectra_many_by_source_vendors_relative_to(
        &fixture_root,
        ["varian_1h", "bruker_without_expno"],
        ["varian", "bruker"],
    )?;
    assert_eq!(multi_vendors.len(), 3);
    assert_eq!(
        multi_vendors.source_vendor_count(LoadedSourceVendor::AgilentVarian),
        1
    );
    assert_eq!(
        multi_vendors.source_vendor_count(LoadedSourceVendor::Bruker),
        2
    );
    Ok(())
}

#[test]
fn prelude_exports_source_path_prefix_loader_helpers() -> Result<()> {
    let fixture_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../rspin-io/testdata/zenodo_7100132");

    let prefix_path =
        load_spectra_by_source_path_prefix(&fixture_root, "bruker_without_expno/pdata")?;
    assert_eq!(prefix_path.len(), 1);
    assert_eq!(
        prefix_path.source_format_count(LoadedSourceFormat::BrukerProcessed),
        1
    );

    let relative_prefix_path = load_spectra_by_source_path_prefix_relative_to(
        &fixture_root,
        "bruker_without_expno",
        "bruker_without_expno/pdata",
    )?;
    assert_eq!(relative_prefix_path.len(), 1);
    assert_eq!(relative_prefix_path.only_1d()?.x.unit, Unit::Ppm);

    let multi_prefix_path = load_spectra_many_by_source_path_prefix_relative_to(
        &fixture_root,
        ["varian_1h", "bruker_without_expno"],
        "bruker_without_expno/pdata",
    )?;
    assert_eq!(multi_prefix_path.len(), 1);
    assert_eq!(multi_prefix_path.only_1d()?.x.unit, Unit::Ppm);

    let prefix_paths = load_spectra_by_source_path_prefixes(
        &fixture_root,
        ["varian_1h", "bruker_without_expno/pdata"],
    )?;
    assert_eq!(prefix_paths.len(), 2);
    assert_eq!(
        prefix_paths.source_format_count(LoadedSourceFormat::AgilentFid),
        1
    );
    assert_eq!(
        prefix_paths.source_format_count(LoadedSourceFormat::BrukerProcessed),
        1
    );

    let multi_prefix_paths = load_spectra_many_by_source_path_prefixes_relative_to(
        &fixture_root,
        ["varian_1h", "bruker_without_expno"],
        ["varian_1h", "bruker_without_expno/pdata"],
    )?;
    assert_eq!(multi_prefix_paths.len(), 2);

    let exact_prefix =
        load_spectrum_1d_by_source_path_prefixes(&fixture_root, ["varian_1h", "missing"])?;
    assert_eq!(exact_prefix.metadata.nucleus, Some(Nucleus::Hydrogen1));
    Ok(())
}

#[test]
fn prelude_exports_owned_source_filtered_bundle_extractors() -> Result<()> {
    let fixture_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../rspin-io/testdata/zenodo_7100132");

    let bundle = load_spectra_many_by_sources_relative_to(
        &fixture_root,
        ["varian_1h", "bruker_without_expno"],
        [
            LoadedSourceFilter::path("varian_1h"),
            LoadedSourceFilter::path("bruker_without_expno/pdata/1"),
        ],
    )?;

    let processed = bundle
        .clone()
        .into_spectra_1d_by_source(LoadedSourceFilter::vendor("bruker"));
    assert_eq!(processed.len(), 1);
    assert_eq!(processed[0].metadata.nucleus, Some(Nucleus::Hydrogen1));

    assert_eq!(
        bundle.source_count_by_sources([
            LoadedSourceFilter::vendor("varian"),
            LoadedSourceFilter::vendor("bruker")
        ]),
        2
    );
    assert_eq!(
        bundle
            .loaded_1d_by_sources([
                LoadedSourceFilter::path("varian_1h"),
                LoadedSourceFilter::vendor("bruker")
            ])
            .count(),
        2
    );
    assert_eq!(
        bundle
            .clone()
            .into_loaded_1d_by_sources([
                LoadedSourceFilter::path("varian_1h"),
                LoadedSourceFilter::vendor("bruker")
            ])
            .len(),
        2
    );

    let loaded_varian = bundle.into_loaded_by_source(LoadedSourceFilter::vendor("varian"));
    assert_eq!(loaded_varian.len(), 1);
    assert_eq!(
        loaded_varian[0].source().vendor(),
        Some(LoadedSourceVendor::AgilentVarian)
    );
    Ok(())
}

#[test]
fn prelude_exports_source_subset_helpers() -> Result<()> {
    let fixture_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../rspin-io/testdata/zenodo_7100132");

    let bundle =
        load_spectra_many_relative_to(&fixture_root, ["varian_1h", "bruker_without_expno"])?;
    let subset = bundle.source_subset_by_sources([
        LoadedSourceFilter::vendor("varian"),
        LoadedSourceFilter::path("bruker_without_expno/pdata/1"),
    ]);

    assert_eq!(subset.len(), 2);
    assert_eq!(
        subset.source_vendor_count(LoadedSourceVendor::AgilentVarian),
        1
    );
    assert_eq!(
        subset.source_format_count(LoadedSourceFormat::BrukerProcessed),
        1
    );

    let consumed = subset.into_source_subset(LoadedSourceFilter::vendor("bruker"));
    assert_eq!(consumed.len(), 1);
    assert_eq!(
        consumed.source_paths().collect::<Vec<_>>(),
        vec![std::path::Path::new("bruker_without_expno/pdata/1")]
    );

    let processed = bundle.source_format_subset(LoadedSourceFormat::BrukerProcessed);
    assert_eq!(processed.len(), 1);
    assert_eq!(
        processed.source_paths().collect::<Vec<_>>(),
        vec![std::path::Path::new("bruker_without_expno/pdata/1")]
    );
    let varian = bundle.into_source_vendor_subset("varian");
    assert_eq!(
        varian.source_vendor_count(LoadedSourceVendor::AgilentVarian),
        1
    );

    let warning_bundle = load_spectra(&fixture_root)?;
    assert_eq!(
        warning_bundle.warning_count_for_source(LoadedSourceFilter::path("empty_jcamp/empty.jdx")),
        1
    );
    assert!(warning_bundle.has_warning_for_source(LoadedSourceFilter::vendor("varian")));

    let mixed = load_spectra(fixture_root.join("bruker_without_expno"))?;
    assert_eq!(mixed.source_data_kind_count(LoadedSourceDataKind::Raw), 1);
    assert_eq!(
        mixed.source_data_kind_count(LoadedSourceDataKind::Processed),
        1
    );
    assert_eq!(
        mixed.summary().source_data_kind_counts(),
        vec![
            SourceDataKindCount::new(LoadedSourceDataKind::Raw, 1),
            SourceDataKindCount::new(LoadedSourceDataKind::Processed, 1)
        ]
    );
    assert_eq!(mixed.raw_source_subset().len(), 1);
    assert_eq!(mixed.into_processed_source_subset().len(), 1);

    let raw = load_spectra_by_source_data_kind(
        fixture_root.join("bruker_without_expno"),
        LoadedSourceDataKind::Raw,
    )?;
    assert_eq!(raw.len(), 1);
    let raw_or_processed = load_spectra_by_source_data_kinds(
        fixture_root.join("bruker_without_expno"),
        [LoadedSourceDataKind::Raw, LoadedSourceDataKind::Processed],
    )?;
    assert_eq!(raw_or_processed.len(), 2);
    let raw_or_processed = load_spectra_many_by_source_data_kinds_relative_to(
        &fixture_root,
        ["bruker_without_expno"],
        [LoadedSourceDataKind::Raw, LoadedSourceDataKind::Processed],
    )?;
    assert_eq!(raw_or_processed.len(), 2);
    let processed = RSpinReader::new()
        .only_source(LoadedSourceDataKind::Processed)
        .read_path_relative_to(&fixture_root, "bruker_without_expno")?;
    assert_eq!(processed.len(), 1);
    let processed = RSpinReader::new()
        .source_vendor("bruker")
        .processed_sources()
        .read_relative_to(&fixture_root, "bruker_without_expno")?;
    assert_eq!(processed.len(), 1);
    assert_eq!(
        processed.source_paths().collect::<Vec<_>>(),
        vec![std::path::Path::new("bruker_without_expno/pdata/1")]
    );
    let processed = RSpinReader::new()
        .source_path_prefix("bruker_without_expno/pdata")
        .read_relative_to(&fixture_root, "bruker_without_expno")?;
    assert_eq!(processed.len(), 1);
    assert!(processed.has_source(LoadedSourceFilter::path_prefix(
        "bruker_without_expno/pdata"
    )));
    Ok(())
}

#[test]
fn prelude_exports_source_path_prefix_bundle_helpers() -> Result<()> {
    let mixed_vendor_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../rspin-io/testdata/nmrxiv/cc0/myrcene");
    let bundle = load_spectra(mixed_vendor_root)?;

    assert_eq!(bundle.source_path_prefix_count("jcamp"), 2);
    assert!(bundle.has_source_path_prefix("jeol"));
    assert_eq!(bundle.warning_path_prefix_count("jcamp"), 0);
    assert!(!bundle.has_warning_path_prefix("jcamp"));
    assert_eq!(
        bundle
            .source_paths_for_path_prefix("jcamp")
            .collect::<Vec<_>>(),
        vec![
            std::path::Path::new("jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx"),
            std::path::Path::new("jcamp/myrcene_1h_400mhz_jcamp_dx_6_link.jdx")
        ]
    );

    let jeol = bundle.source_path_prefix_subset("jeol");
    assert_eq!(jeol.len(), 3);
    assert_eq!(jeol.loaded_2d_by_source_path_prefix("jeol").count(), 1);
    let hsqc = jeol.only_2d_by_source_path_prefix("jeol")?;
    assert_eq!(hsqc.shape(), (1024, 32));
    let (_, hsqc_source) = jeol.only_loaded_2d_by_source_path_prefix("jeol")?;
    assert_eq!(
        hsqc_source.path(),
        Some(std::path::Path::new("jeol/myrcene_hsqc_400mhz.jdf"))
    );
    assert_eq!(
        bundle.into_spectra_1d_by_source_path_prefix("jcamp").len(),
        2
    );
    Ok(())
}

#[test]
fn prelude_exports_source_filtered_exact_bundle_loaders() -> Result<()> {
    let mixed_vendor_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../rspin-io/testdata/nmrxiv/cc0/myrcene");

    assert!(LoadedSourceFormat::all().contains(&LoadedSourceFormat::BrukerSer));
    assert!(LoadedSourceVendor::all().contains(&LoadedSourceVendor::Bruker));
    assert_eq!(
        LoadedSourceFormat::JcampDx.file_extensions(),
        &["jdx", "dx", "jcamp"]
    );
    assert_eq!(
        LoadedSourceFormat::BrukerSer.path_markers(),
        &["ser", "acqus", "acqu2s"]
    );

    let bruker_1d =
        load_spectrum_1d_by_source_format(&mixed_vendor_root, LoadedSourceFormat::BrukerFid)?;
    assert_eq!(bruker_1d.metadata.nucleus, Some(Nucleus::Hydrogen1));
    let (bruker_1d, source) = load_spectrum_1d_with_source_by_source_format(
        &mixed_vendor_root,
        LoadedSourceFormat::BrukerFid,
    )?;
    assert_eq!(bruker_1d.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(source.format(), "bruker_fid");

    let bruker_2d =
        load_spectrum_2d_by_source_format(&mixed_vendor_root, LoadedSourceFormat::BrukerSer)?;
    assert_eq!(bruker_2d.shape(), (2048, 512));
    let (bruker_2d, source) = load_spectrum_2d_with_source_by_source_format(
        &mixed_vendor_root,
        LoadedSourceFormat::BrukerSer,
    )?;
    assert_eq!(bruker_2d.shape(), (2048, 512));
    assert_eq!(source.format(), "bruker_ser");

    let bruker_1d =
        load_spectrum_1d_by_source_vendor(&mixed_vendor_root, LoadedSourceVendor::Bruker)?;
    assert_eq!(bruker_1d.metadata.nucleus, Some(Nucleus::Hydrogen1));
    let (bruker_1d, source) = load_spectrum_1d_with_source_by_source_vendor(
        &mixed_vendor_root,
        LoadedSourceVendor::Bruker,
    )?;
    assert_eq!(bruker_1d.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(source.format(), "bruker_fid");

    let bruker_2d =
        load_spectrum_2d_by_source_vendor(&mixed_vendor_root, LoadedSourceVendor::Bruker)?;
    assert_eq!(bruker_2d.shape(), (2048, 512));
    let (bruker_2d, source) = load_spectrum_2d_with_source_by_source_vendor(
        &mixed_vendor_root,
        LoadedSourceVendor::Bruker,
    )?;
    assert_eq!(bruker_2d.shape(), (2048, 512));
    assert_eq!(source.format(), "bruker_ser");

    let mixed_vendor_base =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../rspin-io/testdata/nmrxiv/cc0");
    let (bruker_1d, source) = load_spectrum_1d_with_source_by_source_format_relative_to(
        &mixed_vendor_base,
        "myrcene",
        LoadedSourceFormat::BrukerFid,
    )?;
    assert_eq!(bruker_1d.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(
        source.path(),
        Some(std::path::Path::new("myrcene/bruker_1h_raw"))
    );
    let bruker_2d = load_spectrum_2d_by_source_vendor_relative_to(
        &mixed_vendor_base,
        "myrcene",
        LoadedSourceVendor::Bruker,
    )?;
    assert_eq!(bruker_2d.shape(), (2048, 512));
    let (bruker_2d, source) = load_spectrum_2d_many_with_source_by_source_vendor_relative_to(
        &mixed_vendor_base,
        ["myrcene/bruker_1h_raw", "myrcene/bruker_cosy_raw"],
        LoadedSourceVendor::Bruker,
    )?;
    assert_eq!(bruker_2d.shape(), (2048, 512));
    assert_eq!(
        source.path(),
        Some(std::path::Path::new("myrcene/bruker_cosy_raw"))
    );

    let carbon = load_spectrum_1d_by_source_path_relative_to(
        &mixed_vendor_base,
        "myrcene",
        std::path::Path::new("myrcene/jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx"),
    )?;
    assert_eq!(carbon.metadata.nucleus, Some(Nucleus::Carbon13));
    let carbon = load_spectrum_1d_many_by_source_path_relative_to(
        &mixed_vendor_base,
        ["myrcene/bruker_cosy_raw", "myrcene"],
        std::path::Path::new("myrcene/jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx"),
    )?;
    assert_eq!(carbon.metadata.nucleus, Some(Nucleus::Carbon13));
    let (hsqc, source) = load_spectrum_2d_with_source_by_source_path_relative_to(
        &mixed_vendor_base,
        "myrcene",
        std::path::Path::new("myrcene/jeol/myrcene_hsqc_400mhz.jdf"),
    )?;
    assert_eq!(hsqc.shape(), (1024, 32));
    assert_eq!(
        source.path(),
        Some(std::path::Path::new("myrcene/jeol/myrcene_hsqc_400mhz.jdf"))
    );

    Ok(())
}

#[test]
fn prelude_exports_source_path_prefix_exact_bundle_loaders() -> Result<()> {
    let mixed_vendor_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../rspin-io/testdata/nmrxiv/cc0/myrcene");
    let mixed_vendor_base =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../rspin-io/testdata/nmrxiv/cc0");

    let carbon = load_spectrum_1d_by_source_path_prefix(
        &mixed_vendor_root,
        "jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx",
    )?;
    assert_eq!(carbon.metadata.nucleus, Some(Nucleus::Carbon13));

    let (_, carbon_source) = load_spectrum_1d_with_source_by_source_path_prefix(
        &mixed_vendor_root,
        "jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx",
    )?;
    assert_eq!(
        carbon_source.path(),
        Some(std::path::Path::new(
            "jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx"
        ))
    );

    let hsqc = load_spectrum_2d_by_source_path_prefix(&mixed_vendor_root, "jeol")?;
    assert_eq!(hsqc.shape(), (1024, 32));

    let (_, hsqc_source) =
        load_spectrum_2d_with_source_by_source_path_prefix(&mixed_vendor_root, "jeol")?;
    assert_eq!(
        hsqc_source.path(),
        Some(std::path::Path::new("jeol/myrcene_hsqc_400mhz.jdf"))
    );

    let carbon = load_spectrum_1d_many_by_source_path_prefix_relative_to(
        &mixed_vendor_base,
        ["myrcene/bruker_cosy_raw", "myrcene"],
        "myrcene/jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx",
    )?;
    assert_eq!(carbon.metadata.nucleus, Some(Nucleus::Carbon13));

    let (hsqc, hsqc_source) = load_spectrum_2d_many_with_source_by_source_path_prefix_relative_to(
        &mixed_vendor_base,
        ["myrcene/jeol/myrcene_1h_400mhz.jdf", "myrcene"],
        "myrcene/jeol",
    )?;
    assert_eq!(hsqc.shape(), (1024, 32));
    assert_eq!(
        hsqc_source.path(),
        Some(std::path::Path::new("myrcene/jeol/myrcene_hsqc_400mhz.jdf"))
    );
    Ok(())
}

#[test]
fn prelude_exports_source_data_kind_exact_bundle_loaders() -> Result<()> {
    let mixed_vendor_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../rspin-io/testdata/nmrxiv/cc0/myrcene");
    let mixed_vendor_base =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../rspin-io/testdata/nmrxiv/cc0");

    let raw_1d =
        load_spectrum_1d_by_source_data_kind(&mixed_vendor_root, LoadedSourceDataKind::Raw)?;
    assert_eq!(raw_1d.metadata.nucleus, Some(Nucleus::Hydrogen1));
    let (raw_1d, source) = load_spectrum_1d_with_source_by_source_data_kind(
        &mixed_vendor_root,
        LoadedSourceDataKind::Raw,
    )?;
    assert_eq!(raw_1d.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(source.data_kind(), LoadedSourceDataKind::Raw);

    let raw_2d = RSpinReader::new()
        .read_2d_by_source_data_kind(&mixed_vendor_root, LoadedSourceDataKind::Raw)?;
    assert_eq!(raw_2d.shape(), (2048, 512));
    let (raw_2d, source) = load_spectrum_2d_with_source_by_source_data_kind_relative_to(
        &mixed_vendor_base,
        "myrcene",
        LoadedSourceDataKind::Raw,
    )?;
    assert_eq!(raw_2d.shape(), (2048, 512));
    assert_eq!(
        source.path(),
        Some(std::path::Path::new("myrcene/bruker_cosy_raw"))
    );

    let raw_1d = load_spectra(&mixed_vendor_root)?
        .into_only_1d_by_source_data_kind(LoadedSourceDataKind::Raw)?;
    assert_eq!(raw_1d.metadata.nucleus, Some(Nucleus::Hydrogen1));

    let raw_1d = load_spectrum_1d_many_by_source_data_kind_relative_to(
        &mixed_vendor_base,
        ["myrcene/bruker_1h_raw", "myrcene/bruker_cosy_raw"],
        LoadedSourceDataKind::Raw,
    )?;
    assert_eq!(raw_1d.metadata.nucleus, Some(Nucleus::Hydrogen1));

    let (_, source) = load_spectrum_2d_many_with_source_by_source_data_kind_relative_to(
        &mixed_vendor_base,
        ["myrcene/bruker_1h_raw", "myrcene/bruker_cosy_raw"],
        LoadedSourceDataKind::Raw,
    )?;
    assert_eq!(
        source.path(),
        Some(std::path::Path::new("myrcene/bruker_cosy_raw"))
    );

    let raw_2d = RSpinReader::new().read_2d_many_by_source_data_kind_relative_to(
        &mixed_vendor_base,
        ["myrcene/bruker_1h_raw", "myrcene/bruker_cosy_raw"],
        LoadedSourceDataKind::Raw,
    )?;
    assert_eq!(raw_2d.shape(), (2048, 512));
    Ok(())
}

#[test]
fn prelude_exports_generic_source_filtered_exact_bundle_loaders() -> Result<()> {
    let mixed_vendor_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../rspin-io/testdata/nmrxiv/cc0/myrcene");
    let mixed_vendor_base =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../rspin-io/testdata/nmrxiv/cc0");

    let bruker_1d =
        load_spectrum_1d_by_source(&mixed_vendor_root, LoadedSourceFilter::vendor("bruker"))?;
    assert_eq!(bruker_1d.metadata.nucleus, Some(Nucleus::Hydrogen1));
    let (bruker_2d, source) = load_spectrum_2d_with_source_by_source(
        &mixed_vendor_root,
        LoadedSourceFilter::from(LoadedSourceVendor::Bruker),
    )?;
    assert_eq!(bruker_2d.shape(), (2048, 512));
    assert_eq!(source.format(), "bruker_ser");
    let carbon = load_spectrum_1d_many_by_source_relative_to(
        &mixed_vendor_base,
        ["myrcene/bruker_cosy_raw", "myrcene"],
        LoadedSourceFilter::path("myrcene/jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx"),
    )?;
    assert_eq!(carbon.metadata.nucleus, Some(Nucleus::Carbon13));
    let (bruker_1d, source) = load_spectrum_1d_many_with_source_by_source_relative_to(
        &mixed_vendor_base,
        ["myrcene/bruker_1h_raw", "myrcene/bruker_cosy_raw"],
        LoadedSourceFilter::path("myrcene/bruker_1h_raw"),
    )?;
    assert_eq!(bruker_1d.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(
        source.path(),
        Some(std::path::Path::new("myrcene/bruker_1h_raw"))
    );
    Ok(())
}

#[test]
fn prelude_exports_generic_source_set_exact_bundle_loaders() -> Result<()> {
    let mixed_vendor_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../rspin-io/testdata/nmrxiv/cc0/myrcene");
    let carbon_path = std::path::Path::new("jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx");
    let hsqc_path = std::path::Path::new("jeol/myrcene_hsqc_400mhz.jdf");

    let carbon =
        load_spectrum_1d_by_sources(&mixed_vendor_root, [LoadedSourceFilter::path(carbon_path)])?;
    assert_eq!(carbon.metadata.nucleus, Some(Nucleus::Carbon13));

    let hsqc =
        load_spectrum_2d_by_sources(&mixed_vendor_root, [LoadedSourceFilter::path(hsqc_path)])?;
    assert_eq!(hsqc.shape(), (1024, 32));

    let raw =
        RSpinReader::new().read_1d_by_sources(&mixed_vendor_root, [LoadedSourceFilter::raw()])?;
    assert_eq!(raw.metadata.nucleus, Some(Nucleus::Hydrogen1));
    Ok(())
}

#[test]
fn prelude_supports_batch_integration() -> Result<()> {
    let integrals = integrate_regions(
        &read_spectrum1d_csv("x,intensity\n0,0\n1,1\n2,2\n")?,
        &[
            IntegralRegion { from: 0.0, to: 1.0 },
            IntegralRegion { from: 1.0, to: 2.0 },
        ],
    )?;
    assert_eq!(integrals.len(), 2);
    assert!((integrals[0].area - 0.5).abs() < 1.0e-12);
    assert!((integrals[1].area - 1.5).abs() < 1.0e-12);

    let integrals_2d = integrate_regions_2d(
        &Spectrum2D::new(
            Axis::linear_ppm(0.0, 2.0, 3)?,
            Axis::linear_ppm(0.0, 2.0, 3)?,
            vec![0.0, 1.0, 2.0, 1.0, 2.0, 3.0, 2.0, 3.0, 4.0],
            Metadata::named("integrated-2d"),
        )?,
        &[
            IntegralRegion2D {
                x_from: 0.0,
                x_to: 1.0,
                y_from: 0.0,
                y_to: 1.0,
            },
            IntegralRegion2D {
                x_from: 1.0,
                x_to: 2.0,
                y_from: 1.0,
                y_to: 2.0,
            },
        ],
    )?;
    assert_eq!(integrals_2d.len(), 2);
    assert!((integrals_2d[0].volume - 1.0).abs() < 1.0e-12);
    assert!((integrals_2d[1].volume - 3.0).abs() < 1.0e-12);
    Ok(())
}

#[test]
fn prelude_supports_detected_feature_integration() -> Result<()> {
    let spectrum = read_spectrum1d_csv("x,intensity\n0,0\n1,2\n2,2\n3,0\n")?;
    let ranges = detect_ranges(
        &spectrum,
        RangeDetectionOptions::new().with_threshold_abs(1.0),
    )?;
    let integrals = integrate_ranges(&spectrum, &ranges)?;

    assert_eq!(integrals.len(), 1);
    assert!((integrals[0].area - 2.0).abs() < 1.0e-12);

    let spectrum_2d = Spectrum2D::new(
        Axis::linear_ppm(0.0, 4.0, 5)?,
        Axis::linear_ppm(0.0, 1.0, 2)?,
        vec![1.0, 1.0, 0.0, 2.0, 2.0, 1.0, 1.0, 0.0, 2.0, 2.0],
        Metadata::named("detected-zone-integration"),
    )?;
    let zones = detect_zones(
        &spectrum_2d,
        ZoneDetectionOptions::new().with_threshold_abs(0.5),
    )?;
    let integrals_2d = integrate_zones_2d(&spectrum_2d, &zones)?;

    assert_eq!(integrals_2d.len(), 2);
    assert!((integrals_2d[0].volume - 1.0).abs() < 1.0e-12);
    assert!((integrals_2d[1].volume - 2.0).abs() < 1.0e-12);
    Ok(())
}

#[test]
fn prelude_supports_exact_simulation_json() -> Result<()> {
    let system = SpinHalfSystem::new().with_spin(1.0);
    system.validate()?;
    validate_spin_half_system(&system)?;
    let system_json = write_spin_half_system_json(&system)?;
    assert!(system_json.contains(SPIN_HALF_SYSTEM_JSON_FORMAT));
    assert!(system_json.contains(&format!("\"version\":{SIMULATION_JSON_VERSION}")));
    assert_eq!(read_spin_half_system_json(&system_json)?, system);

    let options = ExactSpinOptions::new().with_spectrometer_mhz(400.0);
    options.validate()?;
    options.validate_for_system(&system)?;
    validate_exact_spin_half_inputs(&system, &options)?;
    let options_json = write_exact_spin_options_json(&options)?;
    assert!(options_json.contains(EXACT_SPIN_OPTIONS_JSON_FORMAT));
    assert_eq!(read_exact_spin_options_json(&options_json)?, options);

    let render_options = ExactSpectrumOptions::new()
        .with_points(8)
        .with_transition_options(options.clone());
    render_options.validate()?;
    render_options.validate_for_system(&system)?;
    validate_exact_spin_half_spectrum_inputs(&system, &render_options)?;

    let render_2d_options = ExactSpectrum2DOptions::new()
        .with_points(4, 4)
        .without_spin_pairs()
        .with_transition_options(options.clone());
    render_2d_options.validate()?;
    render_2d_options.validate_for_system(&system)?;
    validate_exact_spin_half_spectrum_2d_inputs(&system, &render_2d_options)?;

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

    let transitions_csv = <CsvExactTransitions as SpectrumWriter<[ExactTransition]>>::write_string(
        &CsvExactTransitions,
        &transitions,
    )?;
    assert!(transitions_csv.starts_with("# format=RSpin Exact Transitions CSV"));
    assert_eq!(transitions_csv, write_exact_transitions_csv(&transitions)?);
    let parsed_csv = read_exact_transitions_csv(&transitions_csv)?;
    assert_eq!(parsed_csv.len(), transitions.len());
    assert!((parsed_csv[0].center_ppm - transitions[0].center_ppm).abs() < 1.0e-9);
    let parsed_csv_trait: Vec<ExactTransition> =
        SpectrumReader::read_str(&CsvExactTransitions, &transitions_csv)?;
    assert_eq!(parsed_csv_trait.len(), transitions.len());
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
    assert_eq!(
        parse_spectrum1d_write_format("jdx")?,
        Spectrum1DWriteFormat::JcampDx
    );
    assert_eq!(
        parse_spectrum2d_write_format("csv")?,
        Spectrum2DWriteFormat::Csv
    );

    let spectrum = Spectrum1D::new(
        Axis::linear_ppm(10.0, 8.0, 3)?,
        vec![1.0, -2.0, 3.0],
        Metadata::named("text export"),
    )?;
    let text = Spectrum1DTextWriter::jcamp_dx().write_string(&spectrum)?;
    assert!(text.contains("##JCAMP-DX=5.00"));
    assert_eq!(
        read_spectrum1d_text(&write_spectrum1d_text(
            &spectrum,
            Spectrum1DWriteFormat::Csv,
        )?)?
        .intensities,
        spectrum.intensities
    );

    let spectrum_2d = Spectrum2D::new(
        Axis::linear_ppm(0.0, 1.0, 2)?,
        Axis::linear_ppm(10.0, 11.0, 2)?,
        vec![1.0, 2.0, 3.0, 4.0],
        Metadata::named("text export 2d"),
    )?;
    let two_d_text = Spectrum2DTextWriter::csv().write_string(&spectrum_2d)?;
    assert_eq!(
        read_spectrum2d_text(&two_d_text)?.z,
        write_spectrum2d_text(&spectrum_2d, Spectrum2DWriteFormat::Csv)
            .and_then(|text| read_spectrum2d_text(&text))?
            .z
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
    let prediction_csv = write_prediction_csv(&prediction)?;
    assert!(prediction_csv.starts_with("# format=RSpin Prediction CSV"));
    assert_eq!(read_prediction_csv(&prediction_csv)?, prediction);
    let trait_prediction: PredictionSet =
        SpectrumReader::read_str(&CsvPrediction, &prediction_csv)?;
    assert_eq!(trait_prediction, prediction);

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
    assert_eq!(analysis.integrals.len(), 2);
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
    assert_eq!(analysis_2d.integrals.len(), 2);
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

#[test]
fn prelude_supports_disk_load_process_analyze_chain() -> Result<()> {
    let fixture_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../rspin-io/testdata/zenodo_7100132");

    let raw = load_spectrum_1d_relative_to(&fixture_root, "varian_1h")?;
    assert_eq!(raw.x.unit, Unit::Seconds);
    let raw_len = raw.len();
    assert!(raw_len > 0);

    let dwell_seconds = (raw.x.values[1] - raw.x.values[0]).abs();
    assert!(dwell_seconds.is_finite() && dwell_seconds > 0.0);

    let target_len = raw_len
        .checked_mul(2)
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "raw FID length overflow".into(),
        })?;

    let recipe = ProcessingRecipe1D::new()
        .exponential_apodization(1.0, dwell_seconds)
        .zero_fill(target_len)
        .fft(FftDirection::Forward)
        .magnitude()
        .normalize_max_abs();
    let processed = recipe.apply(&raw)?;

    assert_eq!(processed.len(), target_len);
    assert!(processed.processing.len() >= 5);
    let max_abs = processed
        .intensities
        .iter()
        .copied()
        .fold(0.0_f64, |acc, v| acc.max(v.abs()));
    assert!((max_abs - 1.0).abs() < 1.0e-9);

    let analysis = processed
        .clone()
        .process()
        .analyze()
        .with_peak_options(
            PeakPickOptions::new()
                .with_min_abs_intensity(0.05)
                .with_min_prominence(0.0),
        )
        .with_range_options(RangeDetectionOptions::new().with_threshold_abs(0.05))
        .run()?;

    assert_eq!(analysis.ranges.len(), analysis.integrals.len());
    for peak in &analysis.peaks {
        assert!(peak.intensity.abs() >= 0.05 - 1.0e-12);
    }
    Ok(())
}
