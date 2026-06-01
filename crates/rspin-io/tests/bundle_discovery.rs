//! Tests for unified bundle source discovery.

use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use rspin_io::{
    DiscoveredSpectrumDimension, DiscoveredSpectrumSource, LoadedSourceDataKind,
    LoadedSourceFormat, LoadedSourceVendor, RSpinReader, discover_spectra, discover_spectra_many,
    discover_spectra_many_relative_to,
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
