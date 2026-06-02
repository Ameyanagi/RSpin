//! Tests for dimension-specific loaded summaries from discovered source metadata.

use std::path::{Path, PathBuf};

use anyhow::Result;
use rspin_io::{
    LoadedSourceFilter, RSpinReader, discover_spectra, load_discovered_spectra_1d_by_source,
    load_discovered_spectra_1d_by_sources, load_discovered_spectra_1d_relative_to,
    load_discovered_spectra_1d_summary, load_discovered_spectra_1d_summary_by_source,
    load_discovered_spectra_1d_summary_by_source_relative_to,
    load_discovered_spectra_1d_summary_by_sources,
    load_discovered_spectra_1d_summary_by_sources_relative_to,
    load_discovered_spectra_1d_summary_relative_to, load_discovered_spectra_1d_summary_strict,
    load_discovered_spectra_1d_summary_strict_by_source,
    load_discovered_spectra_1d_summary_strict_by_sources, load_discovered_spectra_2d_by_source,
    load_discovered_spectra_2d_by_sources, load_discovered_spectra_2d_relative_to,
    load_discovered_spectra_2d_summary, load_discovered_spectra_2d_summary_by_source,
    load_discovered_spectra_2d_summary_by_source_relative_to,
    load_discovered_spectra_2d_summary_by_sources,
    load_discovered_spectra_2d_summary_by_sources_relative_to,
    load_discovered_spectra_2d_summary_relative_to, load_discovered_spectra_2d_summary_strict,
    load_discovered_spectra_2d_summary_strict_by_source,
    load_discovered_spectra_2d_summary_strict_by_sources,
};

#[test]
fn discovered_1d_summary_source_helpers_match_loaded_bundles() -> Result<()> {
    let root = cc0_myrcene_fixture_root();
    let sources = discover_spectra(&root)?;

    let all = load_discovered_spectra_1d_relative_to(&root, &sources)?;
    let summary = load_discovered_spectra_1d_summary_relative_to(&root, &sources)?;
    assert_eq!(summary, all.summary());
    assert_eq!(
        load_discovered_spectra_1d_summary(&root, &sources)?,
        summary
    );
    assert_eq!(
        load_discovered_spectra_1d_summary_strict(&root, &sources)?,
        summary
    );

    let jeol =
        load_discovered_spectra_1d_by_source(&root, &sources, LoadedSourceFilter::vendor("jeol"))?;
    let jeol_summary = load_discovered_spectra_1d_summary_by_source_relative_to(
        &root,
        &sources,
        LoadedSourceFilter::vendor("jeol"),
    )?;
    assert_eq!(jeol_summary, jeol.summary());
    assert_eq!(
        load_discovered_spectra_1d_summary_by_source(
            &root,
            &sources,
            LoadedSourceFilter::vendor("jeol"),
        )?,
        jeol_summary
    );
    assert_eq!(
        load_discovered_spectra_1d_summary_strict_by_source(
            &root,
            &sources,
            LoadedSourceFilter::vendor("jeol"),
        )?,
        jeol_summary
    );
    assert_eq!(
        RSpinReader::new().read_discovered_bundle_1d_summary_by_source(
            &root,
            &sources,
            LoadedSourceFilter::vendor("jeol"),
        )?,
        jeol_summary
    );

    let filters = [
        LoadedSourceFilter::path_prefix("jcamp"),
        LoadedSourceFilter::path("jeol/myrcene_1h_400mhz.jdf"),
    ];
    let selected = load_discovered_spectra_1d_by_sources(&root, &sources, filters.clone())?;
    let selected_summary = load_discovered_spectra_1d_summary_by_sources_relative_to(
        &root,
        &sources,
        filters.clone(),
    )?;
    assert_eq!(selected_summary, selected.summary());
    assert_eq!(
        load_discovered_spectra_1d_summary_by_sources(&root, &sources, filters.clone())?,
        selected_summary
    );
    assert_eq!(
        load_discovered_spectra_1d_summary_strict_by_sources(&root, &sources, filters)?,
        selected_summary
    );
    assert_eq!(
        RSpinReader::new().read_discovered_bundle_1d_summary_by_sources_relative_to(
            &root,
            &sources,
            std::iter::empty::<LoadedSourceFilter>(),
        )?,
        summary
    );

    Ok(())
}

#[test]
fn discovered_2d_summary_source_helpers_match_loaded_bundles() -> Result<()> {
    let root = cc0_myrcene_fixture_root();
    let sources = discover_spectra(&root)?;
    let two_d_sources: Vec<_> = sources.iter().filter(|source| source.is_2d()).collect();

    let all = load_discovered_spectra_2d_relative_to(&root, two_d_sources.clone())?;
    let summary = load_discovered_spectra_2d_summary_relative_to(&root, two_d_sources.clone())?;
    assert_eq!(summary, all.summary());
    assert_eq!(
        load_discovered_spectra_2d_summary(&root, two_d_sources.clone())?,
        summary
    );

    let strict_two_d_sources: Vec<_> = sources
        .iter()
        .filter(|source| {
            source.path().is_some_and(|path| {
                path.starts_with("bruker_cosy_raw")
                    || path == Path::new("jeol/myrcene_hsqc_400mhz.jdf")
            })
        })
        .collect();
    let strict_all = load_discovered_spectra_2d_relative_to(&root, strict_two_d_sources.clone())?;
    assert_eq!(
        load_discovered_spectra_2d_summary_strict(&root, strict_two_d_sources)?,
        strict_all.summary()
    );

    let jeol =
        load_discovered_spectra_2d_by_source(&root, &sources, LoadedSourceFilter::vendor("jeol"))?;
    let jeol_summary = load_discovered_spectra_2d_summary_by_source_relative_to(
        &root,
        &sources,
        LoadedSourceFilter::vendor("jeol"),
    )?;
    assert_eq!(jeol_summary, jeol.summary());
    assert_eq!(
        load_discovered_spectra_2d_summary_by_source(
            &root,
            &sources,
            LoadedSourceFilter::vendor("jeol"),
        )?,
        jeol_summary
    );
    assert_eq!(
        load_discovered_spectra_2d_summary_strict_by_source(
            &root,
            &sources,
            LoadedSourceFilter::vendor("jeol"),
        )?,
        jeol_summary
    );
    assert_eq!(
        RSpinReader::new().read_discovered_bundle_2d_summary_by_source(
            &root,
            &sources,
            LoadedSourceFilter::vendor("jeol"),
        )?,
        jeol_summary
    );

    let filters = [
        LoadedSourceFilter::path_prefix("bruker_cosy_raw"),
        LoadedSourceFilter::path("jeol/myrcene_hsqc_400mhz.jdf"),
    ];
    let selected = load_discovered_spectra_2d_by_sources(&root, &sources, filters.clone())?;
    let selected_summary = load_discovered_spectra_2d_summary_by_sources_relative_to(
        &root,
        &sources,
        filters.clone(),
    )?;
    assert_eq!(selected_summary, selected.summary());
    assert_eq!(
        load_discovered_spectra_2d_summary_by_sources(&root, &sources, filters.clone())?,
        selected_summary
    );
    assert_eq!(
        load_discovered_spectra_2d_summary_strict_by_sources(&root, &sources, filters)?,
        selected_summary
    );
    assert_eq!(
        RSpinReader::new().read_discovered_bundle_2d_summary_by_sources_relative_to(
            &root,
            two_d_sources,
            std::iter::empty::<LoadedSourceFilter>(),
        )?,
        summary
    );

    Ok(())
}

fn cc0_myrcene_fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/nmrxiv/cc0/myrcene")
}
