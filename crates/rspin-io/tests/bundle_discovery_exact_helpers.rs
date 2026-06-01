//! Tests for exact single-spectrum loading from discovered sources.

use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use rspin_io::{
    DiscoveredSpectrumSource, LoadedSourceFilter, LoadedSourceFormat, discover_spectra,
    load_discovered_spectrum_1d, load_discovered_spectrum_1d_by_source,
    load_discovered_spectrum_1d_by_source_relative_to, load_discovered_spectrum_1d_by_sources,
    load_discovered_spectrum_1d_by_sources_relative_to, load_discovered_spectrum_1d_relative_to,
    load_discovered_spectrum_1d_with_source, load_discovered_spectrum_1d_with_source_by_source,
    load_discovered_spectrum_1d_with_source_by_source_relative_to,
    load_discovered_spectrum_1d_with_source_by_sources,
    load_discovered_spectrum_1d_with_source_by_sources_relative_to,
    load_discovered_spectrum_1d_with_source_relative_to, load_discovered_spectrum_2d,
    load_discovered_spectrum_2d_by_source, load_discovered_spectrum_2d_by_source_relative_to,
    load_discovered_spectrum_2d_by_sources, load_discovered_spectrum_2d_by_sources_relative_to,
    load_discovered_spectrum_2d_relative_to, load_discovered_spectrum_2d_with_source,
    load_discovered_spectrum_2d_with_source_by_source,
    load_discovered_spectrum_2d_with_source_by_source_relative_to,
    load_discovered_spectrum_2d_with_source_by_sources,
    load_discovered_spectrum_2d_with_source_by_sources_relative_to,
    load_discovered_spectrum_2d_with_source_relative_to,
};

#[test]
fn free_helpers_load_exact_discovered_1d_sources() -> Result<()> {
    let sources = discover_spectra(fixture_root())?;
    let varian = discovered_source(&sources, "varian_1h", LoadedSourceFormat::AgilentFid)?;

    let spectrum = load_discovered_spectrum_1d_relative_to(fixture_root(), [varian])?;
    assert_eq!(spectrum.len(), 16_384);
    let spectrum = load_discovered_spectrum_1d(fixture_root(), [varian])?;
    assert_eq!(spectrum.len(), 16_384);

    let spectrum = load_discovered_spectrum_1d_by_source_relative_to(
        fixture_root(),
        &sources,
        LoadedSourceFilter::vendor("varian"),
    )?;
    assert_eq!(spectrum.len(), 16_384);
    let spectrum = load_discovered_spectrum_1d_by_source(
        fixture_root(),
        &sources,
        LoadedSourceFilter::path("varian_1h"),
    )?;
    assert_eq!(spectrum.len(), 16_384);
    let spectrum = load_discovered_spectrum_1d_by_sources_relative_to(
        fixture_root(),
        &sources,
        [
            LoadedSourceFilter::path("missing"),
            LoadedSourceFilter::format("agilent_fid"),
        ],
    )?;
    assert_eq!(spectrum.len(), 16_384);
    let spectrum = load_discovered_spectrum_1d_by_sources(
        fixture_root(),
        &sources,
        [LoadedSourceFilter::path("varian_1h")],
    )?;
    assert_eq!(spectrum.len(), 16_384);

    let (spectrum, source) =
        load_discovered_spectrum_1d_with_source_relative_to(fixture_root(), [varian])?;
    assert_eq!(spectrum.len(), 16_384);
    assert_eq!(source.path(), Some(Path::new("varian_1h")));
    let (_, source) = load_discovered_spectrum_1d_with_source(fixture_root(), [varian])?;
    assert_eq!(source.format(), "agilent_fid");
    let (_, source) = load_discovered_spectrum_1d_with_source_by_source_relative_to(
        fixture_root(),
        &sources,
        LoadedSourceFilter::format("agilent_fid"),
    )?;
    assert_eq!(source.path(), Some(Path::new("varian_1h")));
    let (_, source) = load_discovered_spectrum_1d_with_source_by_source(
        fixture_root(),
        &sources,
        LoadedSourceFilter::vendor("varian"),
    )?;
    assert_eq!(source.format(), "agilent_fid");
    let (_, source) = load_discovered_spectrum_1d_with_source_by_sources_relative_to(
        fixture_root(),
        &sources,
        [LoadedSourceFilter::path("varian_1h")],
    )?;
    assert_eq!(source.path(), Some(Path::new("varian_1h")));
    let (_, source) = load_discovered_spectrum_1d_with_source_by_sources(
        fixture_root(),
        &sources,
        [LoadedSourceFilter::vendor("varian")],
    )?;
    assert_eq!(source.format(), "agilent_fid");
    Ok(())
}

#[test]
fn free_helpers_load_exact_discovered_2d_sources() -> Result<()> {
    let sources = discover_spectra(cc0_myrcene_fixture_root())?;
    let hsqc_path = "jeol/myrcene_hsqc_400mhz.jdf";
    let hsqc = discovered_source(&sources, hsqc_path, LoadedSourceFormat::JeolJdf)?;

    let spectrum = load_discovered_spectrum_2d_relative_to(cc0_myrcene_fixture_root(), [hsqc])?;
    assert_eq!(spectrum.shape(), (1024, 32));
    let spectrum = load_discovered_spectrum_2d(cc0_myrcene_fixture_root(), [hsqc])?;
    assert_eq!(spectrum.shape(), (1024, 32));
    let spectrum = load_discovered_spectrum_2d_by_source_relative_to(
        cc0_myrcene_fixture_root(),
        &sources,
        LoadedSourceFilter::path(hsqc_path),
    )?;
    assert_eq!(spectrum.shape(), (1024, 32));
    let spectrum = load_discovered_spectrum_2d_by_source(
        cc0_myrcene_fixture_root(),
        &sources,
        LoadedSourceFilter::path(hsqc_path),
    )?;
    assert_eq!(spectrum.shape(), (1024, 32));
    let spectrum = load_discovered_spectrum_2d_by_sources_relative_to(
        cc0_myrcene_fixture_root(),
        &sources,
        [
            LoadedSourceFilter::path("missing"),
            LoadedSourceFilter::path(hsqc_path),
        ],
    )?;
    assert_eq!(spectrum.shape(), (1024, 32));
    let spectrum = load_discovered_spectrum_2d_by_sources(
        cc0_myrcene_fixture_root(),
        &sources,
        [LoadedSourceFilter::path(hsqc_path)],
    )?;
    assert_eq!(spectrum.shape(), (1024, 32));

    let (spectrum, source) =
        load_discovered_spectrum_2d_with_source_relative_to(cc0_myrcene_fixture_root(), [hsqc])?;
    assert_eq!(spectrum.shape(), (1024, 32));
    assert_eq!(source.path(), Some(Path::new(hsqc_path)));
    let (_, source) = load_discovered_spectrum_2d_with_source(cc0_myrcene_fixture_root(), [hsqc])?;
    assert_eq!(source.format(), "jeol_jdf");
    let (_, source) = load_discovered_spectrum_2d_with_source_by_source_relative_to(
        cc0_myrcene_fixture_root(),
        &sources,
        LoadedSourceFilter::path(hsqc_path),
    )?;
    assert_eq!(source.path(), Some(Path::new(hsqc_path)));
    let (_, source) = load_discovered_spectrum_2d_with_source_by_source(
        cc0_myrcene_fixture_root(),
        &sources,
        LoadedSourceFilter::path(hsqc_path),
    )?;
    assert_eq!(source.format(), "jeol_jdf");
    let (_, source) = load_discovered_spectrum_2d_with_source_by_sources_relative_to(
        cc0_myrcene_fixture_root(),
        &sources,
        [LoadedSourceFilter::path(hsqc_path)],
    )?;
    assert_eq!(source.path(), Some(Path::new(hsqc_path)));
    let (_, source) = load_discovered_spectrum_2d_with_source_by_sources(
        cc0_myrcene_fixture_root(),
        &sources,
        [LoadedSourceFilter::path(hsqc_path)],
    )?;
    assert_eq!(source.format(), "jeol_jdf");
    Ok(())
}

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/zenodo_7100132")
}

fn cc0_myrcene_fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/nmrxiv/cc0/myrcene")
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
