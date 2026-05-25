//! Tests for exact source-filtered bundle selectors.

use std::path::{Path, PathBuf};

use rspin_core::Nucleus;
use rspin_io::{
    LoadedSourceFormat, LoadedSourceVendor, RSpinReader, load_spectra,
    load_spectrum_1d_by_source_format, load_spectrum_1d_by_source_vendor,
    load_spectrum_1d_with_source_by_source_format, load_spectrum_1d_with_source_by_source_vendor,
    load_spectrum_2d_by_source_format, load_spectrum_2d_by_source_vendor,
    load_spectrum_2d_with_source_by_source_format, load_spectrum_2d_with_source_by_source_vendor,
};

#[test]
fn source_format_exact_helpers_select_matching_dimension() -> anyhow::Result<()> {
    let bundle = load_spectra(nmrxiv_fixture_root())?;

    let bruker_1d = bundle.only_1d_by_source_format(LoadedSourceFormat::BrukerFid)?;
    assert_eq!(bruker_1d.len(), 108_399);
    assert_eq!(bruker_1d.metadata.nucleus, Some(Nucleus::Hydrogen1));

    let (bruker_2d, bruker_source) =
        bundle.only_loaded_2d_by_source_format(LoadedSourceFormat::BrukerSer)?;
    assert_eq!(bruker_2d.shape(), (2048, 512));
    assert_eq!(bruker_source.format(), "bruker_ser");
    assert_eq!(bruker_source.path(), Some(Path::new("bruker_cosy_raw")));

    let (jeol_2d, jeol_source) = bundle.only_loaded_2d_by_source_format("jdf")?;
    assert_eq!(jeol_2d.shape(), (1024, 32));
    assert_eq!(jeol_source.format(), "jeol_jdf");
    assert_eq!(
        jeol_source.path(),
        Some(Path::new("jeol/myrcene_hsqc_400mhz.jdf"))
    );

    assert_single_error(
        bundle.only_loaded_1d_by_source_format("jdx"),
        "expected exactly one one-dimensional spectrum for source format jcamp_dx",
        "found 2 one-dimensional and 0 two-dimensional spectra",
    )?;
    assert_single_error(
        bundle.only_2d_by_source_format("jdx"),
        "expected exactly one two-dimensional spectrum for source format jcamp_dx",
        "found 2 one-dimensional and 0 two-dimensional spectra",
    )?;
    assert_single_error(
        bundle.only_loaded_1d_by_source_format("missing"),
        "expected exactly one one-dimensional spectrum for source format missing",
        "found 0 one-dimensional and 0 two-dimensional spectra",
    )?;
    Ok(())
}

#[test]
fn source_vendor_exact_helpers_select_matching_dimension() -> anyhow::Result<()> {
    let bundle = load_spectra(nmrxiv_fixture_root())?;

    let (bruker_1d, bruker_1d_source) =
        bundle.only_loaded_1d_by_source_vendor(LoadedSourceVendor::Bruker)?;
    assert_eq!(bruker_1d.len(), 108_399);
    assert_eq!(bruker_1d_source.format(), "bruker_fid");

    let bruker_2d = bundle.only_2d_by_source_vendor("bruker")?;
    assert_eq!(bruker_2d.shape(), (2048, 512));

    let (jeol_2d, jeol_source) =
        bundle.only_loaded_2d_by_source_vendor(LoadedSourceVendor::Jeol)?;
    assert_eq!(jeol_2d.shape(), (1024, 32));
    assert_eq!(jeol_source.format(), "jeol_jdf");

    assert_single_error(
        bundle.only_loaded_1d_by_source_vendor("jeol"),
        "expected exactly one one-dimensional spectrum for source vendor jeol",
        "found 2 one-dimensional and 1 two-dimensional spectra",
    )?;
    assert_single_error(
        bundle.only_1d_by_source_vendor("unknown-vendor"),
        "expected exactly one one-dimensional spectrum for source vendor unknown-vendor",
        "found 0 one-dimensional and 0 two-dimensional spectra",
    )?;
    Ok(())
}

#[test]
fn consuming_source_format_exact_helpers_select_matching_dimension() -> anyhow::Result<()> {
    let bruker_1d =
        load_spectra(nmrxiv_fixture_root())?.into_only_1d_by_source_format("bruker_fid")?;
    assert_eq!(bruker_1d.len(), 108_399);
    assert_eq!(bruker_1d.metadata.nucleus, Some(Nucleus::Hydrogen1));

    let (bruker_2d, bruker_source) = load_spectra(nmrxiv_fixture_root())?
        .into_only_loaded_2d_by_source_format(LoadedSourceFormat::BrukerSer)?;
    assert_eq!(bruker_2d.shape(), (2048, 512));
    assert_eq!(bruker_source.format(), "bruker_ser");
    assert_eq!(bruker_source.path(), Some(Path::new("bruker_cosy_raw")));

    let (jeol_2d, jeol_source) =
        load_spectra(nmrxiv_fixture_root())?.into_only_loaded_2d_by_source_format("jdf")?;
    assert_eq!(jeol_2d.shape(), (1024, 32));
    assert_eq!(
        jeol_source.path(),
        Some(Path::new("jeol/myrcene_hsqc_400mhz.jdf"))
    );

    assert_single_error(
        load_spectra(nmrxiv_fixture_root())?.into_only_loaded_1d_by_source_format("jdf"),
        "expected exactly one one-dimensional spectrum for source format jeol_jdf",
        "found 2 one-dimensional and 1 two-dimensional spectra",
    )?;
    assert_single_error(
        load_spectra(nmrxiv_fixture_root())?.into_only_2d_by_source_format("missing"),
        "expected exactly one two-dimensional spectrum for source format missing",
        "found 0 one-dimensional and 0 two-dimensional spectra",
    )?;
    Ok(())
}

#[test]
fn consuming_source_vendor_exact_helpers_select_matching_dimension() -> anyhow::Result<()> {
    let (bruker_1d, bruker_source) = load_spectra(nmrxiv_fixture_root())?
        .into_only_loaded_1d_by_source_vendor(LoadedSourceVendor::Bruker)?;
    assert_eq!(bruker_1d.len(), 108_399);
    assert_eq!(bruker_source.format(), "bruker_fid");

    let bruker_2d = load_spectra(nmrxiv_fixture_root())?.into_only_2d_by_source_vendor("bruker")?;
    assert_eq!(bruker_2d.shape(), (2048, 512));

    let (jeol_2d, jeol_source) =
        load_spectra(nmrxiv_fixture_root())?.into_only_loaded_2d_by_source_vendor("jeol")?;
    assert_eq!(jeol_2d.shape(), (1024, 32));
    assert_eq!(jeol_source.format(), "jeol_jdf");

    assert_single_error(
        load_spectra(nmrxiv_fixture_root())?.into_only_loaded_1d_by_source_vendor("jeol"),
        "expected exactly one one-dimensional spectrum for source vendor jeol",
        "found 2 one-dimensional and 1 two-dimensional spectra",
    )?;
    assert_single_error(
        load_spectra(nmrxiv_fixture_root())?.into_only_1d_by_source_vendor("unknown-vendor"),
        "expected exactly one one-dimensional spectrum for source vendor unknown-vendor",
        "found 0 one-dimensional and 0 two-dimensional spectra",
    )?;
    Ok(())
}

#[test]
fn reader_source_format_exact_helpers_select_matching_dimension() -> anyhow::Result<()> {
    let reader = RSpinReader::new();

    let bruker_1d =
        reader.read_1d_by_source_format(nmrxiv_fixture_root(), LoadedSourceFormat::BrukerFid)?;
    assert_eq!(bruker_1d.len(), 108_399);
    assert_eq!(bruker_1d.metadata.nucleus, Some(Nucleus::Hydrogen1));

    let (bruker_2d, bruker_source) = reader.read_2d_with_source_by_source_format(
        nmrxiv_fixture_root(),
        LoadedSourceFormat::BrukerSer,
    )?;
    assert_eq!(bruker_2d.shape(), (2048, 512));
    assert_eq!(bruker_source.format(), "bruker_ser");
    assert_eq!(bruker_source.path(), Some(Path::new("bruker_cosy_raw")));

    let (jeol_2d, jeol_source) =
        reader.read_2d_with_source_by_source_format(nmrxiv_fixture_root(), "jdf")?;
    assert_eq!(jeol_2d.shape(), (1024, 32));
    assert_eq!(
        jeol_source.path(),
        Some(Path::new("jeol/myrcene_hsqc_400mhz.jdf"))
    );

    assert_single_error(
        reader.read_1d_with_source_by_source_format(nmrxiv_fixture_root(), "jdf"),
        "expected exactly one one-dimensional spectrum for source format jeol_jdf",
        "found 2 one-dimensional and 1 two-dimensional spectra",
    )?;
    assert_single_error(
        reader.read_2d_by_source_format(nmrxiv_fixture_root(), "missing"),
        "expected exactly one two-dimensional spectrum for source format missing",
        "found 0 one-dimensional and 0 two-dimensional spectra",
    )?;
    Ok(())
}

#[test]
fn reader_source_vendor_exact_helpers_select_matching_dimension() -> anyhow::Result<()> {
    let reader = RSpinReader::new();

    let (bruker_1d, bruker_source) = reader
        .read_1d_with_source_by_source_vendor(nmrxiv_fixture_root(), LoadedSourceVendor::Bruker)?;
    assert_eq!(bruker_1d.len(), 108_399);
    assert_eq!(bruker_source.format(), "bruker_fid");

    let bruker_2d = reader.read_2d_by_source_vendor(nmrxiv_fixture_root(), "bruker")?;
    assert_eq!(bruker_2d.shape(), (2048, 512));

    let (jeol_2d, jeol_source) =
        reader.read_2d_with_source_by_source_vendor(nmrxiv_fixture_root(), "jeol")?;
    assert_eq!(jeol_2d.shape(), (1024, 32));
    assert_eq!(jeol_source.format(), "jeol_jdf");

    assert_single_error(
        reader.read_1d_with_source_by_source_vendor(nmrxiv_fixture_root(), "jeol"),
        "expected exactly one one-dimensional spectrum for source vendor jeol",
        "found 2 one-dimensional and 1 two-dimensional spectra",
    )?;
    assert_single_error(
        reader.read_1d_by_source_vendor(nmrxiv_fixture_root(), "unknown-vendor"),
        "expected exactly one one-dimensional spectrum for source vendor unknown-vendor",
        "found 0 one-dimensional and 0 two-dimensional spectra",
    )?;
    Ok(())
}

#[test]
fn free_source_format_exact_helpers_select_matching_dimension() -> anyhow::Result<()> {
    let bruker_1d =
        load_spectrum_1d_by_source_format(nmrxiv_fixture_root(), LoadedSourceFormat::BrukerFid)?;
    assert_eq!(bruker_1d.len(), 108_399);
    assert_eq!(bruker_1d.metadata.nucleus, Some(Nucleus::Hydrogen1));

    let (bruker_2d, bruker_source) = load_spectrum_2d_with_source_by_source_format(
        nmrxiv_fixture_root(),
        LoadedSourceFormat::BrukerSer,
    )?;
    assert_eq!(bruker_2d.shape(), (2048, 512));
    assert_eq!(bruker_source.format(), "bruker_ser");
    assert_eq!(bruker_source.path(), Some(Path::new("bruker_cosy_raw")));

    let jeol_2d = load_spectrum_2d_by_source_format(nmrxiv_fixture_root(), "jdf")?;
    assert_eq!(jeol_2d.shape(), (1024, 32));

    assert_single_error(
        load_spectrum_1d_with_source_by_source_format(nmrxiv_fixture_root(), "jdf"),
        "expected exactly one one-dimensional spectrum for source format jeol_jdf",
        "found 2 one-dimensional and 1 two-dimensional spectra",
    )?;
    Ok(())
}

#[test]
fn free_source_vendor_exact_helpers_select_matching_dimension() -> anyhow::Result<()> {
    let (bruker_1d, bruker_source) =
        load_spectrum_1d_with_source_by_source_vendor(nmrxiv_fixture_root(), "bruker")?;
    assert_eq!(bruker_1d.len(), 108_399);
    assert_eq!(bruker_source.format(), "bruker_fid");

    let bruker_2d =
        load_spectrum_2d_by_source_vendor(nmrxiv_fixture_root(), LoadedSourceVendor::Bruker)?;
    assert_eq!(bruker_2d.shape(), (2048, 512));

    let (jeol_2d, jeol_source) =
        load_spectrum_2d_with_source_by_source_vendor(nmrxiv_fixture_root(), "jeol")?;
    assert_eq!(jeol_2d.shape(), (1024, 32));
    assert_eq!(jeol_source.format(), "jeol_jdf");

    assert_single_error(
        load_spectrum_1d_by_source_vendor(nmrxiv_fixture_root(), "jeol"),
        "expected exactly one one-dimensional spectrum for source vendor jeol",
        "found 2 one-dimensional and 1 two-dimensional spectra",
    )?;
    Ok(())
}

fn nmrxiv_fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/nmrxiv/cc0/myrcene")
}

fn assert_single_error<T>(
    result: rspin_core::Result<T>,
    expected_prefix: &str,
    expected_counts: &str,
) -> anyhow::Result<()> {
    let Err(error) = result else {
        anyhow::bail!("single-spectrum helper should fail");
    };
    let message = error.to_string();
    assert!(
        message.contains(expected_prefix),
        "expected {expected_prefix:?} in {message:?}"
    );
    assert!(
        message.contains(expected_counts),
        "expected {expected_counts:?} in {message:?}"
    );
    Ok(())
}
