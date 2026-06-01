//! Tests for unified bundle source discovery.

use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use rspin_io::{
    DiscoveredSpectrumDimension, DiscoveredSpectrumDimensionCount, DiscoveredSpectrumPathCount,
    DiscoveredSpectrumSource, DiscoveredSpectrumSummary, LoadedSourceDataKind, LoadedSourceFilter,
    LoadedSourceFormat, LoadedSourceVendor, RSpinReader, discover_spectra,
    discover_spectra_by_source, discover_spectra_by_source_relative_to,
    discover_spectra_by_sources, discover_spectra_by_sources_relative_to, discover_spectra_many,
    discover_spectra_many_by_source, discover_spectra_many_by_source_relative_to,
    discover_spectra_many_by_sources, discover_spectra_many_by_sources_relative_to,
    discover_spectra_many_relative_to, load_discovered_spectra,
    load_discovered_spectra_relative_to, summarize_discovered_spectra,
};

#[test]
fn discovers_committed_loader_source_candidates() -> Result<()> {
    let sources = discover_spectra(fixture_root())?;

    let varian = discovered_source(&sources, "varian_1h", LoadedSourceFormat::AgilentFid)?;
    assert_eq!(varian.dimension(), DiscoveredSpectrumDimension::OneD);
    assert_eq!(varian.vendor(), Some(LoadedSourceVendor::AgilentVarian));
    assert_eq!(varian.data_kind(), LoadedSourceDataKind::Raw);
    assert!(varian.is_raw());

    let bruker_fid = discovered_source(
        &sources,
        "bruker_without_expno",
        LoadedSourceFormat::BrukerFid,
    )?;
    assert_eq!(bruker_fid.dimension(), DiscoveredSpectrumDimension::OneD);
    assert_eq!(bruker_fid.vendor(), Some(LoadedSourceVendor::Bruker));

    let bruker_processed = discovered_source(
        &sources,
        "bruker_without_expno/pdata/1",
        LoadedSourceFormat::BrukerProcessed,
    )?;
    assert_eq!(
        bruker_processed.dimension(),
        DiscoveredSpectrumDimension::OneD
    );
    assert!(bruker_processed.is_processed());

    let empty_jcamp = discovered_source(
        &sources,
        "empty_jcamp/empty.jdx",
        LoadedSourceFormat::JcampDx,
    )?;
    assert_eq!(empty_jcamp.dimension(), DiscoveredSpectrumDimension::OneD);
    assert_eq!(empty_jcamp.data_kind(), LoadedSourceDataKind::Other);

    Ok(())
}

#[test]
fn source_discovery_respects_chainable_loader_filters() -> Result<()> {
    let sources = RSpinReader::new()
        .processed_sources()
        .one_d_only()
        .discover_path(fixture_root())?;
    let source = only_discovered_source(&sources)?;
    assert_eq!(
        source.path(),
        Some(Path::new("bruker_without_expno/pdata/1"))
    );
    assert!(source.is_format(LoadedSourceFormat::BrukerProcessed));
    assert_eq!(source.dimension(), DiscoveredSpectrumDimension::OneD);

    let hidden_paths = RSpinReader::new()
        .source_vendor("bruker")
        .without_source_paths()
        .discover_relative_to(fixture_root(), "bruker_without_expno")?;
    assert_eq!(hidden_paths.len(), 2);
    assert!(hidden_paths.iter().all(|source| source.path().is_none()));
    assert!(
        hidden_paths
            .iter()
            .all(|source| source.vendor() == Some(LoadedSourceVendor::Bruker))
    );

    Ok(())
}

#[test]
fn discovered_sources_match_generic_source_filters() -> Result<()> {
    let sources = discover_spectra(fixture_root())?;

    let varian = discovered_source(&sources, "varian_1h", LoadedSourceFormat::AgilentFid)?;
    assert!(varian.matches_source(LoadedSourceFilter::vendor("varian")));
    assert!(varian.matches_source(LoadedSourceFilter::format("agilent_fid")));
    assert!(varian.matches_source(LoadedSourceDataKind::Raw));
    assert!(varian.matches_source(LoadedSourceFilter::path("varian_1h")));
    assert!(!varian.matches_source(LoadedSourceFilter::path_prefix("bruker_without_expno")));
    assert!(varian.is_1d());
    assert!(!varian.is_2d());
    assert!(!varian.is_unknown_dimension());

    let bruker_processed = discovered_source(
        &sources,
        "bruker_without_expno/pdata/1",
        LoadedSourceFormat::BrukerProcessed,
    )?;
    assert!(
        bruker_processed.matches_source(LoadedSourceFilter::path_prefix("bruker_without_expno"))
    );

    let empty_jcamp = discovered_source(
        &sources,
        "empty_jcamp/empty.jdx",
        LoadedSourceFormat::JcampDx,
    )?;
    assert!(empty_jcamp.matches_any_source([
        LoadedSourceFilter::format("jdx"),
        LoadedSourceFilter::vendor("bruker"),
    ]));
    assert!(!empty_jcamp.matches_any_source(Vec::<LoadedSourceFilter>::new()));

    Ok(())
}

#[test]
fn source_discovery_summary_counts_candidates() -> Result<()> {
    let sources = discover_spectra(fixture_root())?;
    let summary: DiscoveredSpectrumSummary = summarize_discovered_spectra(&sources);

    assert_eq!(summary.sources(), sources.len());
    assert_eq!(
        summary.sources(),
        summary.sources_1d() + summary.sources_2d() + summary.sources_unknown()
    );
    assert_eq!(
        summary.dimension_count(DiscoveredSpectrumDimension::OneD),
        summary.sources_1d()
    );
    assert!(summary.has_dimension(DiscoveredSpectrumDimension::OneD));
    assert_eq!(
        summary.source_format_count(LoadedSourceFormat::AgilentFid),
        1
    );
    assert_eq!(summary.source_format_count("jdx"), 2);
    assert!(summary.has_source_format(LoadedSourceFormat::BrukerProcessed));
    assert_eq!(summary.source_vendor_count(LoadedSourceVendor::Bruker), 2);
    assert!(summary.has_source_vendor("varian"));
    assert_eq!(summary.source_vendor_count("missing"), 0);
    assert_eq!(summary.source_count(LoadedSourceFilter::format("jdx")), 2);
    assert_eq!(
        summary.source_count(LoadedSourceFilter::vendor("bruker")),
        2
    );
    assert_eq!(summary.source_data_kind_count(LoadedSourceDataKind::Raw), 2);
    assert_eq!(
        summary.source_data_kind_count(LoadedSourceDataKind::Processed),
        1
    );
    assert_eq!(
        summary.source_data_kind_count(LoadedSourceDataKind::Other),
        2
    );
    assert!(summary.has_source_data_kind(LoadedSourceDataKind::Other));
    assert_eq!(
        summary.dimensions,
        vec![
            DiscoveredSpectrumDimensionCount::new(DiscoveredSpectrumDimension::OneD, 4),
            DiscoveredSpectrumDimensionCount::new(DiscoveredSpectrumDimension::TwoD, 1),
        ]
    );
    assert_eq!(summary.source_path_count("varian_1h"), 1);
    assert_eq!(summary.source_path_prefix_count("bruker_without_expno"), 2);
    assert_eq!(summary.source_path_count("empty_jcamp/empty.jdx"), 2);
    assert!(summary.has_source_path("bruker_without_expno/pdata/1"));
    assert!(summary.has_source_path_prefix("empty_jcamp"));
    assert!(!summary.has_source_path("missing"));
    assert_eq!(
        summary.source_count(LoadedSourceFilter::path_prefix("empty_jcamp")),
        2
    );
    assert!(summary.has_source(LoadedSourceFilter::data_kind(LoadedSourceDataKind::Raw)));
    assert!(!summary.has_source(LoadedSourceFilter::path("missing")));
    assert_eq!(summary.source_paths.len(), 4);
    assert!(
        summary
            .source_paths
            .contains(&DiscoveredSpectrumPathCount::new("varian_1h", 1))
    );
    assert!(
        summary
            .source_paths
            .contains(&DiscoveredSpectrumPathCount::new(
                "bruker_without_expno/pdata/1",
                1
            ))
    );

    Ok(())
}

#[test]
fn source_discovery_supports_relative_and_many_inputs() -> Result<()> {
    let sources = discover_spectra_many_relative_to(
        fixture_root(),
        ["varian_1h", "bruker_without_expno/pdata/1"],
    )?;
    assert_eq!(sources.len(), 2);
    assert!(has_discovered_source(
        &sources,
        "varian_1h",
        LoadedSourceFormat::AgilentFid,
        DiscoveredSpectrumDimension::OneD,
    ));
    assert!(has_discovered_source(
        &sources,
        "bruker_without_expno/pdata/1",
        LoadedSourceFormat::BrukerProcessed,
        DiscoveredSpectrumDimension::OneD,
    ));

    let many = discover_spectra_many([
        fixture_root().join("varian_1h"),
        fixture_root().join("empty_jcamp").join("empty.jdx"),
    ])?;
    assert!(
        many.iter()
            .any(|source| source.path() == Some(Path::new("varian_1h")))
    );
    assert!(
        many.iter()
            .any(|source| source.path() == Some(Path::new("empty.jdx")))
    );

    let Err(error) = RSpinReader::new().discover_many(Vec::<PathBuf>::new()) else {
        return Err(anyhow!("empty discovery inputs should fail"));
    };
    assert!(error.to_string().contains("no input paths provided"));

    Ok(())
}

#[test]
fn filtered_discovery_free_helpers_match_single_path_filters() -> Result<()> {
    let bruker = discover_spectra_by_source(fixture_root(), LoadedSourceFilter::vendor("bruker"))?;
    assert_eq!(bruker.len(), 2);
    assert!(
        bruker
            .iter()
            .all(|source| source.vendor() == Some(LoadedSourceVendor::Bruker))
    );

    let processed = discover_spectra_by_source_relative_to(
        fixture_root(),
        "bruker_without_expno",
        LoadedSourceFilter::processed(),
    )?;
    assert_eq!(processed.len(), 1);
    assert!(has_discovered_source(
        &processed,
        "bruker_without_expno/pdata/1",
        LoadedSourceFormat::BrukerProcessed,
        DiscoveredSpectrumDimension::OneD
    ));

    let jcamp_or_varian = discover_spectra_by_sources(
        fixture_root(),
        [
            LoadedSourceFilter::format("jdx"),
            LoadedSourceFilter::path("varian_1h"),
        ],
    )?;
    let summary = summarize_discovered_spectra(&jcamp_or_varian);
    assert_eq!(summary.sources(), 3);
    assert_eq!(summary.source_format_count(LoadedSourceFormat::JcampDx), 2);
    assert_eq!(
        summary.source_format_count(LoadedSourceFormat::AgilentFid),
        1
    );

    let unrestricted =
        discover_spectra_by_sources(fixture_root(), Vec::<LoadedSourceFilter>::new())?;
    assert_eq!(unrestricted, discover_spectra(fixture_root())?);

    let relative_jcamp = discover_spectra_by_sources_relative_to(
        fixture_root(),
        "empty_jcamp/empty.jdx",
        [LoadedSourceFilter::format("jdx")],
    )?;
    assert_eq!(relative_jcamp.len(), 2);
    assert!(
        relative_jcamp
            .iter()
            .all(|source| source.is_format(LoadedSourceFormat::JcampDx))
    );

    Ok(())
}

#[test]
fn filtered_discovery_free_helpers_match_many_path_filters() -> Result<()> {
    let many_raw = discover_spectra_many_by_source([fixture_root()], LoadedSourceFilter::raw())?;
    assert_eq!(
        summarize_discovered_spectra(&many_raw).source_data_kind_count(LoadedSourceDataKind::Raw),
        2
    );

    let relative_many_raw = discover_spectra_many_by_source_relative_to(
        fixture_root(),
        ["varian_1h", "bruker_without_expno"],
        LoadedSourceFilter::raw(),
    )?;
    assert_eq!(relative_many_raw.len(), 2);
    assert!(has_discovered_source(
        &relative_many_raw,
        "varian_1h",
        LoadedSourceFormat::AgilentFid,
        DiscoveredSpectrumDimension::OneD
    ));
    assert!(has_discovered_source(
        &relative_many_raw,
        "bruker_without_expno",
        LoadedSourceFormat::BrukerFid,
        DiscoveredSpectrumDimension::OneD
    ));

    let many_jcamp_or_varian = discover_spectra_many_by_sources(
        [
            fixture_root().join("varian_1h"),
            fixture_root().join("empty_jcamp/empty.jdx"),
        ],
        [
            LoadedSourceFilter::vendor("varian"),
            LoadedSourceFilter::path("empty.jdx"),
        ],
    )?;
    assert_eq!(many_jcamp_or_varian.len(), 3);
    assert!(has_discovered_source(
        &many_jcamp_or_varian,
        "varian_1h",
        LoadedSourceFormat::AgilentFid,
        DiscoveredSpectrumDimension::OneD
    ));

    let relative_many_jcamp_or_processed = discover_spectra_many_by_sources_relative_to(
        fixture_root(),
        ["empty_jcamp/empty.jdx", "bruker_without_expno"],
        [
            LoadedSourceFilter::format("jdx"),
            LoadedSourceFilter::processed(),
        ],
    )?;
    let summary = summarize_discovered_spectra(&relative_many_jcamp_or_processed);
    assert_eq!(summary.sources(), 3);
    assert_eq!(summary.source_format_count(LoadedSourceFormat::JcampDx), 2);
    assert_eq!(
        summary.source_format_count(LoadedSourceFormat::BrukerProcessed),
        1
    );

    Ok(())
}

#[test]
fn loads_selected_discovered_sources_relative_to_base() -> Result<()> {
    let sources = discover_spectra(fixture_root())?;
    let selected = sources
        .iter()
        .filter(|source| source.is_format(LoadedSourceFormat::AgilentFid) || source.is_processed())
        .collect::<Vec<_>>();

    let bundle = RSpinReader::new().read_discovered_relative_to(fixture_root(), selected)?;

    assert_eq!(bundle.len(), 2);
    assert_eq!(
        bundle.source_format_count(LoadedSourceFormat::AgilentFid),
        1
    );
    assert_eq!(
        bundle.source_format_count(LoadedSourceFormat::BrukerProcessed),
        1
    );
    assert!(bundle.has_source_path(Path::new("varian_1h")));
    assert!(bundle.has_source_path(Path::new("bruker_without_expno/pdata/1")));
    Ok(())
}

#[test]
fn discovered_source_free_helpers_load_selected_sources() -> Result<()> {
    let sources = discover_spectra(fixture_root())?;
    let selected = sources
        .iter()
        .filter(|source| source.is_format(LoadedSourceFormat::BrukerProcessed))
        .collect::<Vec<_>>();

    let bundle = load_discovered_spectra_relative_to(fixture_root(), selected)?;
    assert_eq!(bundle.len(), 1);
    assert_eq!(
        bundle.source_format_count(LoadedSourceFormat::BrukerProcessed),
        1
    );

    let selected = sources
        .iter()
        .filter(|source| source.is_format(LoadedSourceFormat::AgilentFid))
        .collect::<Vec<_>>();
    let bundle = load_discovered_spectra(fixture_root(), selected)?;
    assert_eq!(bundle.len(), 1);
    assert_eq!(
        bundle.source_format_count(LoadedSourceFormat::AgilentFid),
        1
    );
    Ok(())
}

#[test]
fn discovered_source_loading_records_bad_candidates_in_non_strict_mode() -> Result<()> {
    let good = discovered_source(
        &discover_spectra(fixture_root())?,
        "varian_1h",
        LoadedSourceFormat::AgilentFid,
    )?
    .clone();
    let bad = DiscoveredSpectrumSource::new(
        Some(PathBuf::from("empty_jcamp/empty.jdx")),
        "jcamp_dx",
        DiscoveredSpectrumDimension::OneD,
    );
    let selected = vec![bad, good];

    let bundle = RSpinReader::new().read_discovered_relative_to(fixture_root(), &selected)?;

    assert_eq!(bundle.len(), 1);
    assert_eq!(bundle.warning_count(), 1);
    let warning = bundle
        .warnings()
        .first()
        .ok_or_else(|| anyhow!("missing warning for bad discovered source"))?;
    assert_eq!(warning.path(), Some(Path::new("empty_jcamp/empty.jdx")));
    assert!(warning.message().contains("missing XYDATA values"));
    assert!(bundle.has_source_path(Path::new("varian_1h")));
    Ok(())
}

#[test]
fn discovered_source_loading_rejects_missing_source_paths() -> Result<()> {
    let sources = RSpinReader::new()
        .without_source_paths()
        .discover(fixture_root())?;
    let Err(error) = RSpinReader::new().read_discovered_relative_to(fixture_root(), &sources)
    else {
        return Err(anyhow!(
            "loading discovered sources without tracked paths should fail"
        ));
    };

    assert!(error.to_string().contains("tracked source path"));
    Ok(())
}

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/zenodo_7100132")
}

fn discovered_source<'a>(
    sources: &'a [DiscoveredSpectrumSource],
    path: &str,
    format: LoadedSourceFormat,
) -> Result<&'a DiscoveredSpectrumSource> {
    sources
        .iter()
        .find(|source| source.path() == Some(Path::new(path)) && source.is_format(format))
        .ok_or_else(|| anyhow!("missing discovered source {path} with format {format}"))
}

fn only_discovered_source(
    sources: &[DiscoveredSpectrumSource],
) -> Result<&DiscoveredSpectrumSource> {
    match sources {
        [source] => Ok(source),
        _ => Err(anyhow!(
            "expected exactly one discovered source, found {}",
            sources.len()
        )),
    }
}

fn has_discovered_source(
    sources: &[DiscoveredSpectrumSource],
    path: &str,
    format: LoadedSourceFormat,
    dimension: DiscoveredSpectrumDimension,
) -> bool {
    sources.iter().any(|source| {
        source.path() == Some(Path::new(path))
            && source.is_format(format)
            && source.dimension() == dimension
    })
}
