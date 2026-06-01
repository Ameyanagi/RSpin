//! Tests for strict discovered-source free loading helpers.

use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use rspin_io::{
    DiscoveredSpectrumSource, LoadedSourceFilter, LoadedSourceFormat, LoadedSourceVendor,
    discover_spectra, load_discovered_spectra_strict, load_discovered_spectra_strict_by_source,
    load_discovered_spectra_strict_by_source_path,
    load_discovered_spectra_strict_by_source_path_prefix,
    load_discovered_spectra_strict_by_source_path_prefix_relative_to,
    load_discovered_spectra_strict_by_source_path_relative_to,
    load_discovered_spectra_strict_by_source_relative_to,
    load_discovered_spectra_strict_by_sources,
    load_discovered_spectra_strict_by_sources_relative_to,
    load_discovered_spectra_strict_relative_to,
};

#[test]
fn strict_free_helpers_load_selected_discovered_sources() -> Result<()> {
    let sources = discover_spectra(fixture_root())?;
    let varian = discovered_source(&sources, "varian_1h", LoadedSourceFormat::AgilentFid)?;

    let bundle = load_discovered_spectra_strict_relative_to(fixture_root(), [varian])?;
    assert_eq!(bundle.len(), 1);
    assert_eq!(
        bundle.source_vendor_count(LoadedSourceVendor::AgilentVarian),
        1
    );

    let bundle = load_discovered_spectra_strict(fixture_root(), [varian])?;
    assert_eq!(bundle.len(), 1);
    assert!(bundle.warnings().is_empty());

    let bundle = load_discovered_spectra_strict_by_source_relative_to(
        fixture_root(),
        &sources,
        LoadedSourceFilter::vendor("varian"),
    )?;
    assert_eq!(bundle.len(), 1);
    assert!(bundle.has_source_path("varian_1h"));

    let bundle = load_discovered_spectra_strict_by_source(
        fixture_root(),
        &sources,
        LoadedSourceFormat::AgilentFid,
    )?;
    assert_eq!(bundle.len(), 1);

    let bundle = load_discovered_spectra_strict_by_source_path_relative_to(
        fixture_root(),
        &sources,
        "varian_1h",
    )?;
    assert_eq!(bundle.len(), 1);
    assert!(bundle.has_source_path("varian_1h"));

    let bundle =
        load_discovered_spectra_strict_by_source_path(fixture_root(), &sources, "varian_1h")?;
    assert_eq!(bundle.len(), 1);

    let bundle = load_discovered_spectra_strict_by_source_path_prefix_relative_to(
        fixture_root(),
        &sources,
        "bruker_without_expno",
    )?;
    assert_eq!(bundle.len(), 2);
    assert_eq!(bundle.source_vendor_count(LoadedSourceVendor::Bruker), 2);

    let bundle = load_discovered_spectra_strict_by_source_path_prefix(
        fixture_root(),
        &sources,
        "bruker_without_expno",
    )?;
    assert_eq!(bundle.len(), 2);

    let bundle = load_discovered_spectra_strict_by_sources_relative_to(
        fixture_root(),
        &sources,
        [
            LoadedSourceFilter::path("missing"),
            LoadedSourceFilter::path("varian_1h"),
        ],
    )?;
    assert_eq!(bundle.len(), 1);

    let bundle = load_discovered_spectra_strict_by_sources(
        fixture_root(),
        &sources,
        [LoadedSourceFilter::path("varian_1h")],
    )?;
    assert_eq!(bundle.len(), 1);
    Ok(())
}

#[test]
fn strict_free_helpers_return_selected_parser_errors() -> Result<()> {
    let sources = discover_spectra(fixture_root())?;
    let empty_jcamp = discovered_source(
        &sources,
        "empty_jcamp/empty.jdx",
        LoadedSourceFormat::JcampDx,
    )?;
    let varian = discovered_source(&sources, "varian_1h", LoadedSourceFormat::AgilentFid)?;

    let Err(error) =
        load_discovered_spectra_strict_relative_to(fixture_root(), [empty_jcamp, varian])
    else {
        return Err(anyhow!(
            "strict discovered loading should reject malformed JCAMP-DX"
        ));
    };
    assert!(error.to_string().contains("missing XYDATA values"));

    let Err(error) = load_discovered_spectra_strict_by_source_relative_to(
        fixture_root(),
        &sources,
        LoadedSourceFormat::JcampDx,
    ) else {
        return Err(anyhow!(
            "strict source-filtered discovered loading should reject malformed JCAMP-DX"
        ));
    };
    assert!(error.to_string().contains("missing XYDATA values"));
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
    let Some(source) = sources
        .iter()
        .find(|source| source.path() == Some(Path::new(path)) && source.is_format(format))
    else {
        return Err(anyhow!("missing discovered {format:?} source at {path}"));
    };
    Ok(source)
}
