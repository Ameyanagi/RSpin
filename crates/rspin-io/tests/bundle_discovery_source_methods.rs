//! Tests for loading directly from discovered source metadata.

use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use rspin_io::{
    DiscoveredSpectrumSource, LoadedSourceFormat, discover_spectra,
    select_discovered_spectra_by_source,
};

#[test]
fn discovered_source_strict_methods_load_exact_spectra() -> Result<()> {
    let sources = discover_spectra(fixture_root())?;
    let varian = discovered_source(&sources, "varian_1h", LoadedSourceFormat::AgilentFid)?;

    let bundle = varian.load_strict_relative_to(fixture_root())?;
    assert_eq!(bundle.len(), 1);
    let bundle = varian.load_strict(fixture_root())?;
    assert_eq!(bundle.len(), 1);

    let spectrum = varian.load_1d_strict_relative_to(fixture_root())?;
    assert_eq!(spectrum.len(), 16_384);
    let spectrum = varian.load_1d_strict(fixture_root())?;
    assert_eq!(spectrum.len(), 16_384);
    let (spectrum, source) = varian.load_1d_with_source_strict_relative_to(fixture_root())?;
    assert_eq!(spectrum.len(), 16_384);
    assert_eq!(source.path(), Some(Path::new("varian_1h")));
    let (_, source) = varian.load_1d_with_source_strict(fixture_root())?;
    assert_eq!(source.format(), "agilent_fid");

    let sources = discover_spectra(cc0_myrcene_fixture_root())?;
    let hsqc_path = "jeol/myrcene_hsqc_400mhz.jdf";
    let hsqc = discovered_source(&sources, hsqc_path, LoadedSourceFormat::JeolJdf)?;
    let spectrum = hsqc.load_2d_strict_relative_to(cc0_myrcene_fixture_root())?;
    assert_eq!(spectrum.shape(), (1024, 32));
    let spectrum = hsqc.load_2d_strict(cc0_myrcene_fixture_root())?;
    assert_eq!(spectrum.shape(), (1024, 32));
    let (spectrum, source) =
        hsqc.load_2d_with_source_strict_relative_to(cc0_myrcene_fixture_root())?;
    assert_eq!(spectrum.shape(), (1024, 32));
    assert_eq!(source.path(), Some(Path::new(hsqc_path)));
    let (_, source) = hsqc.load_2d_with_source_strict(cc0_myrcene_fixture_root())?;
    assert_eq!(source.format(), "jeol_jdf");
    Ok(())
}

#[test]
fn discovered_source_strict_methods_return_parser_errors() -> Result<()> {
    let sources = discover_spectra(fixture_root())?;
    let selected = select_discovered_spectra_by_source(&sources, LoadedSourceFormat::JcampDx);
    let Some(empty_jcamp) = selected.first() else {
        return Err(anyhow!("missing empty JCAMP-DX discovered source"));
    };

    let Err(error) = empty_jcamp.load_1d_strict_relative_to(fixture_root()) else {
        return Err(anyhow!(
            "strict selected-source loading should reject malformed JCAMP-DX"
        ));
    };
    assert!(error.to_string().contains("missing XYDATA values"));
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
