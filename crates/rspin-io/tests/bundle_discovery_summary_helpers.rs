//! Tests for loaded bundle summaries from discovered source metadata.

use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use rspin_io::{
    DiscoveredSpectrumSource, LoadedSourceDataKind, LoadedSourceFilter, LoadedSourceFormat,
    LoadedSourceVendor, RSpinReader, SpectrumBundleSummary, discover_spectra,
    load_discovered_spectra_summary, load_discovered_spectra_summary_by_source,
    load_discovered_spectra_summary_by_source_path,
    load_discovered_spectra_summary_by_source_path_prefix,
    load_discovered_spectra_summary_by_source_path_prefix_relative_to,
    load_discovered_spectra_summary_by_source_path_prefixes,
    load_discovered_spectra_summary_by_source_path_prefixes_relative_to,
    load_discovered_spectra_summary_by_source_path_relative_to,
    load_discovered_spectra_summary_by_source_paths,
    load_discovered_spectra_summary_by_source_paths_relative_to,
    load_discovered_spectra_summary_by_source_relative_to,
    load_discovered_spectra_summary_by_sources,
    load_discovered_spectra_summary_by_sources_relative_to,
    load_discovered_spectra_summary_relative_to, load_discovered_spectra_summary_strict,
    load_discovered_spectra_summary_strict_by_source,
    load_discovered_spectra_summary_strict_by_source_path,
    load_discovered_spectra_summary_strict_by_source_path_prefix,
    load_discovered_spectra_summary_strict_by_source_path_prefix_relative_to,
    load_discovered_spectra_summary_strict_by_source_path_prefixes,
    load_discovered_spectra_summary_strict_by_source_path_prefixes_relative_to,
    load_discovered_spectra_summary_strict_by_source_path_relative_to,
    load_discovered_spectra_summary_strict_by_source_paths,
    load_discovered_spectra_summary_strict_by_source_paths_relative_to,
    load_discovered_spectra_summary_strict_by_source_relative_to,
    load_discovered_spectra_summary_strict_by_sources,
};

#[test]
fn loaded_summary_helpers_report_selected_discovered_sources() -> Result<()> {
    let root = fixture_root();
    let sources = discover_spectra(&root)?;

    let summary = load_discovered_spectra_summary_relative_to(&root, &sources)?;
    assert_eq!(summary.spectra(), 3);
    assert_eq!(summary.spectra_1d(), 3);
    assert_eq!(summary.warnings(), 2);
    assert_eq!(
        summary.source_format_count(LoadedSourceFormat::AgilentFid),
        1
    );
    assert_eq!(summary.source_vendor_count(LoadedSourceVendor::Bruker), 2);
    assert_eq!(summary.source_data_kind_count(LoadedSourceDataKind::Raw), 2);
    assert_eq!(summary.warning_path_count("empty_jcamp/empty.jdx"), 2);

    let alias = load_discovered_spectra_summary(&root, &sources)?;
    assert_eq!(alias, summary);

    let raw =
        load_discovered_spectra_summary_by_source(&root, &sources, LoadedSourceFilter::raw())?;
    assert_eq!(raw.spectra(), 2);
    assert_eq!(raw.warnings(), 0);

    let varian = load_discovered_spectra_summary_by_source_relative_to(
        &root,
        &sources,
        LoadedSourceFilter::vendor("varian"),
    )?;
    assert_eq!(varian.spectra(), 1);
    assert_eq!(varian.source_path_count("varian_1h"), 1);

    let varian_by_path =
        load_discovered_spectra_summary_by_source_path_relative_to(&root, &sources, "varian_1h")?;
    assert_eq!(varian_by_path, varian);

    let varian_by_path_alias =
        load_discovered_spectra_summary_by_source_path(&root, &sources, "varian_1h")?;
    assert_eq!(varian_by_path_alias, varian);

    assert_loaded_summary_source_path_sets(&root, &sources, &summary)?;

    let bruker_by_prefix = load_discovered_spectra_summary_by_source_path_prefix_relative_to(
        &root,
        &sources,
        "bruker_without_expno",
    )?;
    assert_eq!(bruker_by_prefix.spectra(), 2);
    assert_eq!(
        bruker_by_prefix.source_vendor_count(LoadedSourceVendor::Bruker),
        2
    );

    let bruker_by_prefix_alias = load_discovered_spectra_summary_by_source_path_prefix(
        &root,
        &sources,
        "bruker_without_expno",
    )?;
    assert_eq!(bruker_by_prefix_alias, bruker_by_prefix);

    let selected_by_prefixes = load_discovered_spectra_summary_by_source_path_prefixes_relative_to(
        &root,
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
        &root,
        &sources,
        ["bruker_without_expno", "varian_1h"],
    )?;
    assert_eq!(selected_by_prefixes_alias, selected_by_prefixes);

    let unrestricted_by_empty_prefixes =
        load_discovered_spectra_summary_by_source_path_prefixes_relative_to(
            &root,
            &sources,
            std::iter::empty::<&str>(),
        )?;
    assert_eq!(unrestricted_by_empty_prefixes, summary);

    assert_loaded_summary_generic_source_filters(&root, &sources)?;

    Ok(())
}

fn assert_loaded_summary_generic_source_filters(
    root: &Path,
    sources: &[DiscoveredSpectrumSource],
) -> Result<()> {
    let selected = load_discovered_spectra_summary_by_sources(
        root,
        sources,
        [
            LoadedSourceFilter::path("varian_1h"),
            LoadedSourceFilter::path("bruker_without_expno/pdata/1"),
        ],
    )?;
    assert_eq!(selected.spectra(), 2);
    assert_eq!(
        selected.source_format_count(LoadedSourceFormat::BrukerProcessed),
        1
    );

    let selected_relative = load_discovered_spectra_summary_by_sources_relative_to(
        root,
        sources,
        [LoadedSourceFilter::path("bruker_without_expno/pdata/1")],
    )?;
    assert_eq!(selected_relative.spectra(), 1);
    assert_eq!(
        selected_relative.source_data_kind_count(LoadedSourceDataKind::Processed),
        1
    );
    Ok(())
}

#[test]
fn reader_loaded_summary_methods_preserve_filters_and_strict_mode() -> Result<()> {
    let root = fixture_root();
    let sources = discover_spectra(&root)?;

    let processed = RSpinReader::new().read_discovered_summary_by_source_relative_to(
        &root,
        &sources,
        LoadedSourceFilter::processed(),
    )?;
    assert_eq!(processed.spectra(), 1);
    assert_eq!(
        processed.source_format_count(LoadedSourceFormat::BrukerProcessed),
        1
    );

    let processed_alias = RSpinReader::new().read_discovered_summary_by_source(
        &root,
        &sources,
        LoadedSourceFilter::processed(),
    )?;
    assert_eq!(processed_alias, processed);

    let selected = RSpinReader::new().read_discovered_summary_by_sources_relative_to(
        &root,
        &sources,
        [LoadedSourceFilter::path("varian_1h")],
    )?;
    assert_eq!(selected.spectra(), 1);

    let selected_alias = RSpinReader::new().read_discovered_summary_by_sources(
        &root,
        &sources,
        [LoadedSourceFilter::path("varian_1h")],
    )?;
    assert_eq!(selected_alias, selected);

    let selected_by_path = RSpinReader::new().read_discovered_summary_by_source_path_relative_to(
        &root,
        &sources,
        "varian_1h",
    )?;
    assert_eq!(selected_by_path, selected);

    let selected_by_path_alias =
        RSpinReader::new().read_discovered_summary_by_source_path(&root, &sources, "varian_1h")?;
    assert_eq!(selected_by_path_alias, selected);

    assert_reader_loaded_summary_source_path_sets(&root, &sources, &selected)?;

    let selected_by_prefix = RSpinReader::new()
        .read_discovered_summary_by_source_path_prefix_relative_to(
            &root,
            &sources,
            "bruker_without_expno",
        )?;
    assert_eq!(selected_by_prefix.spectra(), 2);

    let selected_by_prefix_alias = RSpinReader::new()
        .read_discovered_summary_by_source_path_prefix(&root, &sources, "bruker_without_expno")?;
    assert_eq!(selected_by_prefix_alias, selected_by_prefix);

    let selected_direct = RSpinReader::new().read_discovered_summary(&root, &sources)?;
    assert_eq!(selected_direct.spectra(), 3);

    let strict = load_discovered_spectra_summary_strict_by_source(
        &root,
        &sources,
        LoadedSourceFilter::vendor("varian"),
    )?;
    assert_eq!(strict.spectra(), 1);

    let strict_relative = load_discovered_spectra_summary_strict_by_source_relative_to(
        &root,
        &sources,
        LoadedSourceFilter::vendor("varian"),
    )?;
    assert_eq!(strict_relative, strict);

    let strict_path = load_discovered_spectra_summary_strict_by_source_path_relative_to(
        &root,
        &sources,
        "varian_1h",
    )?;
    assert_eq!(strict_path, strict);

    let strict_path_alias =
        load_discovered_spectra_summary_strict_by_source_path(&root, &sources, "varian_1h")?;
    assert_eq!(strict_path_alias, strict);

    assert_strict_loaded_summary_source_path_sets(&root, &sources, &strict)?;

    let strict_prefix = load_discovered_spectra_summary_strict_by_source_path_prefix_relative_to(
        &root,
        &sources,
        "bruker_without_expno",
    )?;
    assert_eq!(strict_prefix.spectra(), 2);

    let strict_prefix_alias = load_discovered_spectra_summary_strict_by_source_path_prefix(
        &root,
        &sources,
        "bruker_without_expno",
    )?;
    assert_eq!(strict_prefix_alias, strict_prefix);

    let strict_selected = load_discovered_spectra_summary_strict_by_sources(
        &root,
        &sources,
        [LoadedSourceFilter::path("varian_1h")],
    )?;
    assert_eq!(strict_selected.spectra(), 1);

    let Err(error) = load_discovered_spectra_summary_strict(&root, &sources) else {
        return Err(anyhow!(
            "strict discovered summary loading should reject malformed candidates"
        ));
    };
    assert!(error.to_string().contains("missing XYDATA values"));

    Ok(())
}

fn assert_loaded_summary_source_path_sets(
    root: &Path,
    sources: &[DiscoveredSpectrumSource],
    summary: &SpectrumBundleSummary,
) -> Result<()> {
    let selected = load_discovered_spectra_summary_by_source_paths_relative_to(
        root,
        sources,
        ["varian_1h", "bruker_without_expno/pdata/1", "missing"],
    )?;
    assert_eq!(selected.spectra(), 2);
    assert_eq!(
        selected.source_data_kind_count(LoadedSourceDataKind::Processed),
        1
    );
    assert_eq!(
        selected.source_vendor_count(LoadedSourceVendor::AgilentVarian),
        1
    );

    let selected_alias = load_discovered_spectra_summary_by_source_paths(
        root,
        sources,
        ["varian_1h", "bruker_without_expno/pdata/1"],
    )?;
    assert_eq!(selected_alias, selected);

    let unrestricted = load_discovered_spectra_summary_by_source_paths_relative_to(
        root,
        sources,
        std::iter::empty::<&str>(),
    )?;
    assert_eq!(unrestricted, *summary);
    Ok(())
}

fn assert_reader_loaded_summary_source_path_sets(
    root: &Path,
    sources: &[DiscoveredSpectrumSource],
    selected: &SpectrumBundleSummary,
) -> Result<()> {
    let selected_paths = RSpinReader::new().read_discovered_summary_by_source_paths_relative_to(
        root,
        sources,
        ["varian_1h", "missing"],
    )?;
    assert_eq!(selected_paths, *selected);

    let selected_paths_reader =
        RSpinReader::new().read_discovered_summary_by_source_paths(root, sources, ["varian_1h"])?;
    assert_eq!(selected_paths_reader, *selected);
    Ok(())
}

fn assert_strict_loaded_summary_source_path_sets(
    root: &Path,
    sources: &[DiscoveredSpectrumSource],
    strict: &SpectrumBundleSummary,
) -> Result<()> {
    let strict_paths = load_discovered_spectra_summary_strict_by_source_paths_relative_to(
        root,
        sources,
        ["varian_1h", "missing"],
    )?;
    assert_eq!(strict_paths, *strict);

    let strict_paths_reader =
        load_discovered_spectra_summary_strict_by_source_paths(root, sources, ["varian_1h"])?;
    assert_eq!(strict_paths_reader, *strict);
    Ok(())
}

#[test]
fn reader_loaded_summary_prefix_set_methods_preserve_filters_and_strict_mode() -> Result<()> {
    let root = fixture_root();
    let sources = discover_spectra(&root)?;

    let selected_by_prefixes = RSpinReader::new()
        .read_discovered_summary_by_source_path_prefixes_relative_to(
            &root,
            &sources,
            ["missing", "bruker_without_expno", "varian_1h"],
        )?;
    assert_eq!(selected_by_prefixes.spectra(), 3);
    assert_eq!(
        selected_by_prefixes.source_vendor_count(LoadedSourceVendor::Bruker),
        2
    );
    assert_eq!(
        selected_by_prefixes.source_vendor_count(LoadedSourceVendor::AgilentVarian),
        1
    );

    let selected_by_prefixes_alias = RSpinReader::new()
        .read_discovered_summary_by_source_path_prefixes(
            &root,
            &sources,
            ["bruker_without_expno", "varian_1h"],
        )?;
    assert_eq!(selected_by_prefixes_alias, selected_by_prefixes);

    let strict_prefixes =
        load_discovered_spectra_summary_strict_by_source_path_prefixes_relative_to(
            &root,
            &sources,
            ["missing", "bruker_without_expno", "varian_1h"],
        )?;
    assert_eq!(strict_prefixes.spectra(), 3);
    assert_eq!(
        strict_prefixes.source_vendor_count(LoadedSourceVendor::AgilentVarian),
        1
    );
    assert_eq!(
        strict_prefixes.source_vendor_count(LoadedSourceVendor::Bruker),
        2
    );

    let strict_prefixes_alias = load_discovered_spectra_summary_strict_by_source_path_prefixes(
        &root,
        &sources,
        ["bruker_without_expno", "varian_1h"],
    )?;
    assert_eq!(strict_prefixes_alias, strict_prefixes);
    Ok(())
}

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/zenodo_7100132")
}
