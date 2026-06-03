//! Tests for exact single-spectrum loading from discovered sources.

use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use rspin_io::{
    DiscoveredSpectrumSource, LoadedSourceDataKind, LoadedSourceFilter, LoadedSourceFormat,
    discover_spectra, load_discovered_spectrum_1d, load_discovered_spectrum_1d_by_source,
    load_discovered_spectrum_1d_by_source_path,
    load_discovered_spectrum_1d_by_source_path_prefix_relative_to,
    load_discovered_spectrum_1d_by_source_path_prefixes,
    load_discovered_spectrum_1d_by_source_relative_to, load_discovered_spectrum_1d_by_sources,
    load_discovered_spectrum_1d_by_sources_relative_to, load_discovered_spectrum_1d_relative_to,
    load_discovered_spectrum_1d_with_source, load_discovered_spectrum_1d_with_source_by_source,
    load_discovered_spectrum_1d_with_source_by_source_path,
    load_discovered_spectrum_1d_with_source_by_source_path_prefix_relative_to,
    load_discovered_spectrum_1d_with_source_by_source_path_prefixes_relative_to,
    load_discovered_spectrum_1d_with_source_by_source_relative_to,
    load_discovered_spectrum_1d_with_source_by_sources,
    load_discovered_spectrum_1d_with_source_by_sources_relative_to,
    load_discovered_spectrum_1d_with_source_relative_to, load_discovered_spectrum_2d,
    load_discovered_spectrum_2d_by_source, load_discovered_spectrum_2d_by_source_path_prefix,
    load_discovered_spectrum_2d_by_source_path_prefixes_relative_to,
    load_discovered_spectrum_2d_by_source_path_relative_to,
    load_discovered_spectrum_2d_by_source_relative_to, load_discovered_spectrum_2d_by_sources,
    load_discovered_spectrum_2d_by_sources_relative_to, load_discovered_spectrum_2d_relative_to,
    load_discovered_spectrum_2d_with_source, load_discovered_spectrum_2d_with_source_by_source,
    load_discovered_spectrum_2d_with_source_by_source_path_prefix,
    load_discovered_spectrum_2d_with_source_by_source_path_prefixes,
    load_discovered_spectrum_2d_with_source_by_source_path_relative_to,
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
fn free_helpers_load_exact_discovered_1d_sources_by_path() -> Result<()> {
    let sources = discover_spectra(fixture_root())?;

    let spectrum =
        load_discovered_spectrum_1d_by_source_path(fixture_root(), &sources, "varian_1h")?;
    assert_eq!(spectrum.len(), 16_384);

    let spectrum = load_discovered_spectrum_1d_by_source_path_prefix_relative_to(
        fixture_root(),
        &sources,
        "varian_1h",
    )?;
    assert_eq!(spectrum.len(), 16_384);

    let (_, source) = load_discovered_spectrum_1d_with_source_by_source_path(
        fixture_root(),
        &sources,
        "varian_1h",
    )?;
    assert_eq!(source.path(), Some(Path::new("varian_1h")));

    let (_, source) = load_discovered_spectrum_1d_with_source_by_source_path_prefix_relative_to(
        fixture_root(),
        &sources,
        "varian_1h",
    )?;
    assert_eq!(source.format(), "agilent_fid");

    let spectrum = rspin_io::RSpinReader::new().read_discovered_1d_by_source_path(
        fixture_root(),
        &sources,
        "varian_1h",
    )?;
    assert_eq!(spectrum.len(), 16_384);

    let (_, source) = rspin_io::RSpinReader::new()
        .read_discovered_1d_with_source_by_source_path_prefix(
            fixture_root(),
            &sources,
            "varian_1h",
        )?;
    assert_eq!(source.path(), Some(Path::new("varian_1h")));
    Ok(())
}

#[test]
fn free_helpers_load_exact_discovered_1d_sources_by_path_prefixes() -> Result<()> {
    let sources = discover_spectra(fixture_root())?;

    let spectrum = load_discovered_spectrum_1d_by_source_path_prefixes(
        fixture_root(),
        &sources,
        ["missing", "varian_1h"],
    )?;
    assert_eq!(spectrum.len(), 16_384);

    let (_, source) = load_discovered_spectrum_1d_with_source_by_source_path_prefixes_relative_to(
        fixture_root(),
        &sources,
        ["missing", "varian_1h"],
    )?;
    assert_eq!(source.path(), Some(Path::new("varian_1h")));

    let spectrum = rspin_io::RSpinReader::new().read_discovered_1d_by_source_path_prefixes(
        fixture_root(),
        &sources,
        ["missing", "varian_1h"],
    )?;
    assert_eq!(spectrum.len(), 16_384);

    let (_, source) = rspin_io::RSpinReader::new()
        .read_discovered_1d_with_source_by_source_path_prefixes_relative_to(
            fixture_root(),
            &sources,
            ["missing", "varian_1h"],
        )?;
    assert_eq!(source.format(), "agilent_fid");
    Ok(())
}

#[test]
fn short_source_aliases_load_exact_discovered_sources() -> Result<()> {
    let sources = discover_spectra(fixture_root())?;
    let varian = discovered_source(&sources, "varian_1h", LoadedSourceFormat::AgilentFid)?;

    let spectrum =
        rspin_io::load_discovered_spectrum_1d_by_format(fixture_root(), [varian], "varian fid")?;
    assert_eq!(spectrum.len(), 16_384);

    let (_, source) = rspin_io::load_discovered_spectrum_1d_with_source_by_vendor(
        fixture_root(),
        [varian],
        "varian",
    )?;
    assert_eq!(source.path(), Some(Path::new("varian_1h")));

    let raw = rspin_io::RSpinReader::new().read_discovered_1d_by_data_kind(
        fixture_root(),
        [varian],
        LoadedSourceDataKind::Raw,
    )?;
    assert_eq!(raw.len(), 16_384);

    let (_, source) = rspin_io::RSpinReader::new().read_discovered_1d_with_source_by_path_prefix(
        fixture_root(),
        [varian],
        "varian_1h",
    )?;
    assert_eq!(source.format(), "agilent_fid");

    let myrcene_root = cc0_myrcene_fixture_root();
    let myrcene_sources = discover_spectra(&myrcene_root)?;
    let hsqc_source = discovered_source(
        &myrcene_sources,
        "jeol/myrcene_hsqc_400mhz.jdf",
        LoadedSourceFormat::JeolJdf,
    )?;
    let hsqc = rspin_io::RSpinReader::new().read_discovered_2d_by_vendor(
        &myrcene_root,
        [hsqc_source],
        "jeol",
    )?;
    assert_eq!(hsqc.shape(), (1024, 32));

    let (_, source) = rspin_io::load_discovered_spectrum_2d_with_source_by_path_prefix(
        &myrcene_root,
        [hsqc_source],
        "jeol",
    )?;
    assert_eq!(
        source.path(),
        Some(Path::new("jeol/myrcene_hsqc_400mhz.jdf"))
    );
    Ok(())
}

#[test]
fn short_path_aliases_load_exact_discovered_sources() -> Result<()> {
    let sources = discover_spectra(fixture_root())?;

    let spectrum =
        rspin_io::load_discovered_spectrum_1d_by_path(fixture_root(), &sources, "varian_1h")?;
    assert_eq!(spectrum.len(), 16_384);

    let (_, source) = rspin_io::RSpinReader::new()
        .read_discovered_1d_with_source_by_path_relative_to(
            fixture_root(),
            &sources,
            "varian_1h",
        )?;
    assert_eq!(source.path(), Some(Path::new("varian_1h")));

    let spectrum = rspin_io::load_discovered_spectrum_1d_by_path_prefixes(
        fixture_root(),
        &sources,
        ["missing", "varian_1h"],
    )?;
    assert_eq!(spectrum.len(), 16_384);

    let (_, source) = rspin_io::RSpinReader::new()
        .read_discovered_1d_with_source_by_path_prefixes(
            fixture_root(),
            &sources,
            ["missing", "varian_1h"],
        )?;
    assert_eq!(source.format(), "agilent_fid");

    let myrcene_root = cc0_myrcene_fixture_root();
    let myrcene_sources = discover_spectra(&myrcene_root)?;
    let hsqc_path = "jeol/myrcene_hsqc_400mhz.jdf";
    let hsqc = rspin_io::RSpinReader::new().read_discovered_2d_by_path(
        &myrcene_root,
        &myrcene_sources,
        hsqc_path,
    )?;
    assert_eq!(hsqc.shape(), (1024, 32));

    let (_, source) = rspin_io::load_discovered_spectrum_2d_with_source_by_path(
        &myrcene_root,
        &myrcene_sources,
        hsqc_path,
    )?;
    assert_eq!(source.path(), Some(Path::new(hsqc_path)));

    let hsqc = rspin_io::load_discovered_spectrum_2d_by_path_prefixes(
        &myrcene_root,
        &myrcene_sources,
        ["missing", hsqc_path],
    )?;
    assert_eq!(hsqc.shape(), (1024, 32));

    let (_, source) = rspin_io::RSpinReader::new()
        .read_discovered_2d_with_source_by_path_prefixes_relative_to(
            &myrcene_root,
            &myrcene_sources,
            ["missing", hsqc_path],
        )?;
    assert_eq!(source.format(), "jeol_jdf");

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

#[test]
fn free_helpers_load_exact_discovered_2d_sources_by_path() -> Result<()> {
    let sources = discover_spectra(cc0_myrcene_fixture_root())?;
    let hsqc_path = "jeol/myrcene_hsqc_400mhz.jdf";
    let hsqc_prefix = hsqc_path;

    let spectrum = load_discovered_spectrum_2d_by_source_path_relative_to(
        cc0_myrcene_fixture_root(),
        &sources,
        hsqc_path,
    )?;
    assert_eq!(spectrum.shape(), (1024, 32));

    let spectrum = load_discovered_spectrum_2d_by_source_path_prefix(
        cc0_myrcene_fixture_root(),
        &sources,
        hsqc_prefix,
    )?;
    assert_eq!(spectrum.shape(), (1024, 32));

    let (_, source) = load_discovered_spectrum_2d_with_source_by_source_path_relative_to(
        cc0_myrcene_fixture_root(),
        &sources,
        hsqc_path,
    )?;
    assert_eq!(source.path(), Some(Path::new(hsqc_path)));

    let (_, source) = load_discovered_spectrum_2d_with_source_by_source_path_prefix(
        cc0_myrcene_fixture_root(),
        &sources,
        hsqc_prefix,
    )?;
    assert_eq!(source.format(), "jeol_jdf");

    let spectrum = rspin_io::RSpinReader::new().read_discovered_2d_by_source_path_prefix(
        cc0_myrcene_fixture_root(),
        &sources,
        hsqc_prefix,
    )?;
    assert_eq!(spectrum.shape(), (1024, 32));

    let (_, source) = rspin_io::RSpinReader::new().read_discovered_2d_with_source_by_source_path(
        cc0_myrcene_fixture_root(),
        &sources,
        hsqc_path,
    )?;
    assert_eq!(source.path(), Some(Path::new(hsqc_path)));
    Ok(())
}

#[test]
fn free_helpers_load_exact_discovered_2d_sources_by_path_prefixes() -> Result<()> {
    let sources = discover_spectra(cc0_myrcene_fixture_root())?;
    let hsqc_path = "jeol/myrcene_hsqc_400mhz.jdf";

    let spectrum = load_discovered_spectrum_2d_by_source_path_prefixes_relative_to(
        cc0_myrcene_fixture_root(),
        &sources,
        ["missing", hsqc_path],
    )?;
    assert_eq!(spectrum.shape(), (1024, 32));

    let (_, source) = load_discovered_spectrum_2d_with_source_by_source_path_prefixes(
        cc0_myrcene_fixture_root(),
        &sources,
        ["missing", hsqc_path],
    )?;
    assert_eq!(source.path(), Some(Path::new(hsqc_path)));

    let spectrum = rspin_io::RSpinReader::new()
        .read_discovered_2d_by_source_path_prefixes_relative_to(
            cc0_myrcene_fixture_root(),
            &sources,
            ["missing", hsqc_path],
        )?;
    assert_eq!(spectrum.shape(), (1024, 32));

    let (_, source) = rspin_io::RSpinReader::new()
        .read_discovered_2d_with_source_by_source_path_prefixes(
            cc0_myrcene_fixture_root(),
            &sources,
            ["missing", hsqc_path],
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
