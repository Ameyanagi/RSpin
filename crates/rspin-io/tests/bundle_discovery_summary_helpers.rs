//! Tests for loaded bundle summaries from discovered source metadata.

use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use rspin_io::{
    LoadedSourceDataKind, LoadedSourceFilter, LoadedSourceFormat, LoadedSourceVendor, RSpinReader,
    discover_spectra, load_discovered_spectra_summary, load_discovered_spectra_summary_by_source,
    load_discovered_spectra_summary_by_source_path,
    load_discovered_spectra_summary_by_source_path_prefix,
    load_discovered_spectra_summary_by_source_path_prefix_relative_to,
    load_discovered_spectra_summary_by_source_path_relative_to,
    load_discovered_spectra_summary_by_source_relative_to,
    load_discovered_spectra_summary_by_sources,
    load_discovered_spectra_summary_by_sources_relative_to,
    load_discovered_spectra_summary_relative_to, load_discovered_spectra_summary_strict,
    load_discovered_spectra_summary_strict_by_source,
    load_discovered_spectra_summary_strict_by_source_path,
    load_discovered_spectra_summary_strict_by_source_path_prefix,
    load_discovered_spectra_summary_strict_by_source_path_prefix_relative_to,
    load_discovered_spectra_summary_strict_by_source_path_relative_to,
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

    let selected = load_discovered_spectra_summary_by_sources(
        &root,
        &sources,
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
        &root,
        &sources,
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

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/zenodo_7100132")
}
