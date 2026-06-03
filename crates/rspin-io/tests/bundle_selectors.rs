//! Tests for exact source-filtered bundle selectors.

use std::path::{Path, PathBuf};

use rspin_core::Nucleus;
use rspin_io::{
    LoadedSourceDataKind, LoadedSourceFilter, LoadedSourceFormat, LoadedSourceVendor, RSpinReader,
    load_spectra, load_spectrum_1d_by_source, load_spectrum_1d_by_source_data_kind,
    load_spectrum_1d_by_source_data_kind_relative_to, load_spectrum_1d_by_source_format,
    load_spectrum_1d_by_source_format_relative_to, load_spectrum_1d_by_source_path,
    load_spectrum_1d_by_source_path_prefix, load_spectrum_1d_by_source_path_prefix_relative_to,
    load_spectrum_1d_by_source_path_relative_to, load_spectrum_1d_by_source_relative_to,
    load_spectrum_1d_by_source_vendor, load_spectrum_1d_by_source_vendor_relative_to,
    load_spectrum_1d_by_sources, load_spectrum_1d_many_by_source,
    load_spectrum_1d_many_by_source_data_kind,
    load_spectrum_1d_many_by_source_data_kind_relative_to, load_spectrum_1d_many_by_source_format,
    load_spectrum_1d_many_by_source_path_prefix,
    load_spectrum_1d_many_by_source_path_prefix_relative_to,
    load_spectrum_1d_many_by_source_path_relative_to, load_spectrum_1d_many_by_sources,
    load_spectrum_1d_many_with_source_by_source,
    load_spectrum_1d_many_with_source_by_source_data_kind,
    load_spectrum_1d_many_with_source_by_source_data_kind_relative_to,
    load_spectrum_1d_many_with_source_by_source_path_prefix,
    load_spectrum_1d_many_with_source_by_source_path_prefix_relative_to,
    load_spectrum_1d_many_with_source_by_source_relative_to,
    load_spectrum_1d_many_with_source_by_source_vendor,
    load_spectrum_1d_many_with_source_by_sources_relative_to,
    load_spectrum_1d_with_source_by_source, load_spectrum_1d_with_source_by_source_data_kind,
    load_spectrum_1d_with_source_by_source_data_kind_relative_to,
    load_spectrum_1d_with_source_by_source_format,
    load_spectrum_1d_with_source_by_source_format_relative_to,
    load_spectrum_1d_with_source_by_source_path,
    load_spectrum_1d_with_source_by_source_path_prefix,
    load_spectrum_1d_with_source_by_source_path_prefix_relative_to,
    load_spectrum_1d_with_source_by_source_path_relative_to,
    load_spectrum_1d_with_source_by_source_relative_to,
    load_spectrum_1d_with_source_by_source_vendor,
    load_spectrum_1d_with_source_by_source_vendor_relative_to, load_spectrum_2d_by_source,
    load_spectrum_2d_by_source_data_kind, load_spectrum_2d_by_source_data_kind_relative_to,
    load_spectrum_2d_by_source_format, load_spectrum_2d_by_source_format_relative_to,
    load_spectrum_2d_by_source_path, load_spectrum_2d_by_source_path_prefix,
    load_spectrum_2d_by_source_path_prefix_relative_to,
    load_spectrum_2d_by_source_path_relative_to, load_spectrum_2d_by_source_relative_to,
    load_spectrum_2d_by_source_vendor, load_spectrum_2d_by_source_vendor_relative_to,
    load_spectrum_2d_by_sources, load_spectrum_2d_many_by_source_data_kind,
    load_spectrum_2d_many_by_source_data_kind_relative_to,
    load_spectrum_2d_many_by_source_format_relative_to,
    load_spectrum_2d_many_by_source_path_prefix_relative_to,
    load_spectrum_2d_many_by_source_relative_to, load_spectrum_2d_many_by_sources_relative_to,
    load_spectrum_2d_many_with_source_by_source,
    load_spectrum_2d_many_with_source_by_source_data_kind,
    load_spectrum_2d_many_with_source_by_source_data_kind_relative_to,
    load_spectrum_2d_many_with_source_by_source_path,
    load_spectrum_2d_many_with_source_by_source_path_prefix,
    load_spectrum_2d_with_source_by_source, load_spectrum_2d_with_source_by_source_data_kind,
    load_spectrum_2d_with_source_by_source_data_kind_relative_to,
    load_spectrum_2d_with_source_by_source_format,
    load_spectrum_2d_with_source_by_source_format_relative_to,
    load_spectrum_2d_with_source_by_source_path,
    load_spectrum_2d_with_source_by_source_path_prefix,
    load_spectrum_2d_with_source_by_source_path_prefix_relative_to,
    load_spectrum_2d_with_source_by_source_path_relative_to,
    load_spectrum_2d_with_source_by_source_vendor,
    load_spectrum_2d_with_source_by_source_vendor_relative_to,
    load_spectrum_2d_with_source_by_sources,
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
fn source_path_exact_helpers_select_matching_dimension() -> anyhow::Result<()> {
    let bundle = load_spectra(nmrxiv_fixture_root())?;
    let carbon_path = Path::new("jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx");
    let hsqc_path = Path::new("jeol/myrcene_hsqc_400mhz.jdf");

    let carbon = bundle.only_1d_by_source_path(carbon_path)?;
    assert_eq!(carbon.metadata.nucleus, Some(Nucleus::Carbon13));

    let (hsqc, hsqc_source) = bundle.only_loaded_2d_by_source_path(hsqc_path)?;
    assert_eq!(hsqc.shape(), (1024, 32));
    assert_eq!(hsqc_source.path(), Some(hsqc_path));

    assert_single_error(
        bundle.only_loaded_1d_by_source_path(hsqc_path),
        "expected exactly one one-dimensional spectrum for source path jeol/myrcene_hsqc_400mhz.jdf",
        "found 0 one-dimensional and 1 two-dimensional spectra",
    )?;
    assert_single_error(
        bundle.only_2d_by_source_path("missing.jdx"),
        "expected exactly one two-dimensional spectrum for source path missing.jdx",
        "found 0 one-dimensional and 0 two-dimensional spectra",
    )?;
    Ok(())
}

#[test]
fn short_path_exact_helpers_select_matching_dimension() -> anyhow::Result<()> {
    let bundle = load_spectra(nmrxiv_fixture_root())?;
    let carbon_path = Path::new("jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx");
    let hsqc_path = Path::new("jeol/myrcene_hsqc_400mhz.jdf");

    let loaded = bundle
        .loaded_by_path(carbon_path)
        .ok_or_else(|| anyhow::anyhow!("missing loaded entry at {}", carbon_path.display()))?;
    assert!(loaded.is_1d());

    let (carbon, carbon_source) = bundle
        .loaded_1d_by_path(carbon_path)
        .ok_or_else(|| anyhow::anyhow!("missing 1D entry at {}", carbon_path.display()))?;
    assert_eq!(carbon.metadata.nucleus, Some(Nucleus::Carbon13));
    assert_eq!(carbon_source.path(), Some(carbon_path));

    let hsqc = bundle.only_2d_by_path(hsqc_path)?;
    assert_eq!(hsqc.shape(), (1024, 32));
    let (_, hsqc_source) = bundle.only_loaded_2d_by_path(hsqc_path)?;
    assert_eq!(hsqc_source.path(), Some(hsqc_path));

    assert!(bundle.loaded_2d_by_path(carbon_path).is_none());
    assert_single_error(
        bundle.only_loaded_1d_by_path(hsqc_path),
        "expected exactly one one-dimensional spectrum for source path jeol/myrcene_hsqc_400mhz.jdf",
        "found 0 one-dimensional and 1 two-dimensional spectra",
    )?;

    let owned_carbon = bundle.clone().into_only_1d_by_path(carbon_path)?;
    assert_eq!(owned_carbon.metadata.nucleus, Some(Nucleus::Carbon13));
    let (owned_hsqc, owned_source) = bundle.into_only_loaded_2d_by_path(hsqc_path)?;
    assert_eq!(owned_hsqc.shape(), (1024, 32));
    assert_eq!(owned_source.path(), Some(hsqc_path));

    Ok(())
}

#[test]
fn source_path_prefix_exact_helpers_select_matching_dimension() -> anyhow::Result<()> {
    let bundle = load_spectra(nmrxiv_fixture_root())?;
    let carbon_path = Path::new("jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx");
    let hsqc_path = Path::new("jeol/myrcene_hsqc_400mhz.jdf");

    let carbon = bundle.only_1d_by_source_path_prefix(carbon_path)?;
    assert_eq!(carbon.metadata.nucleus, Some(Nucleus::Carbon13));

    let (hsqc, hsqc_source) = bundle.only_loaded_2d_by_source_path_prefix("jeol")?;
    assert_eq!(hsqc.shape(), (1024, 32));
    assert_eq!(hsqc_source.path(), Some(hsqc_path));

    assert_single_error(
        bundle.only_loaded_1d_by_source_path_prefix("jeol"),
        "expected exactly one one-dimensional spectrum for source path prefix jeol",
        "found 2 one-dimensional and 1 two-dimensional spectra",
    )?;
    assert_single_error(
        bundle.only_2d_by_source_path_prefix("missing"),
        "expected exactly one two-dimensional spectrum for source path prefix missing",
        "found 0 one-dimensional and 0 two-dimensional spectra",
    )?;

    let carbon =
        load_spectra(nmrxiv_fixture_root())?.into_only_1d_by_source_path_prefix(carbon_path)?;
    assert_eq!(carbon.metadata.nucleus, Some(Nucleus::Carbon13));

    let (hsqc, hsqc_source) =
        load_spectra(nmrxiv_fixture_root())?.into_only_loaded_2d_by_source_path_prefix("jeol")?;
    assert_eq!(hsqc.shape(), (1024, 32));
    assert_eq!(hsqc_source.path(), Some(hsqc_path));
    Ok(())
}

#[test]
fn reader_source_path_prefix_exact_helpers_select_matching_dimension() -> anyhow::Result<()> {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/zenodo_7100132");
    let bruker = base.join("bruker_without_expno");

    let processed = RSpinReader::new().read_1d_by_source_path_prefix(&bruker, "pdata")?;
    assert_eq!(processed.metadata.nucleus, Some(Nucleus::Hydrogen1));

    let (_, processed_source) =
        RSpinReader::new().read_1d_with_source_by_source_path_prefix(&bruker, "pdata")?;
    assert_eq!(processed_source.path(), Some(Path::new("pdata/1")));

    let processed = RSpinReader::new().read_1d_by_source_path_prefix_relative_to(
        &base,
        "bruker_without_expno",
        "bruker_without_expno/pdata",
    )?;
    assert_eq!(processed.metadata.nucleus, Some(Nucleus::Hydrogen1));

    let (_, processed_source) = RSpinReader::new()
        .read_1d_with_source_by_source_path_prefix_relative_to(
            &base,
            "bruker_without_expno",
            "bruker_without_expno/pdata",
        )?;
    assert_eq!(
        processed_source.path(),
        Some(Path::new("bruker_without_expno/pdata/1"))
    );

    let hsqc = RSpinReader::new().read_2d_by_source_path_prefix(nmrxiv_fixture_root(), "jeol")?;
    assert_eq!(hsqc.shape(), (1024, 32));

    let (_, hsqc_source) = RSpinReader::new()
        .read_2d_with_source_by_source_path_prefix(nmrxiv_fixture_root(), "jeol")?;
    assert_eq!(
        hsqc_source.path(),
        Some(Path::new("jeol/myrcene_hsqc_400mhz.jdf"))
    );

    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/nmrxiv/cc0");
    let hsqc = RSpinReader::new().read_2d_by_source_path_prefix_relative_to(
        &base,
        "myrcene",
        "myrcene/jeol",
    )?;
    assert_eq!(hsqc.shape(), (1024, 32));

    let (_, hsqc_source) = RSpinReader::new()
        .read_2d_with_source_by_source_path_prefix_relative_to(&base, "myrcene", "myrcene/jeol")?;
    assert_eq!(
        hsqc_source.path(),
        Some(Path::new("myrcene/jeol/myrcene_hsqc_400mhz.jdf"))
    );
    Ok(())
}

#[test]
fn generic_source_filter_exact_helpers_select_matching_dimension() -> anyhow::Result<()> {
    let bundle = load_spectra(nmrxiv_fixture_root())?;
    let carbon_path = Path::new("jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx");
    let hsqc_path = Path::new("jeol/myrcene_hsqc_400mhz.jdf");

    let bruker_1d = bundle.only_1d_by_source(LoadedSourceFilter::vendor("bruker"))?;
    assert_eq!(bruker_1d.metadata.nucleus, Some(Nucleus::Hydrogen1));

    let raw_1d = bundle.only_1d_by_source(LoadedSourceDataKind::Raw)?;
    assert_eq!(raw_1d.metadata.nucleus, Some(Nucleus::Hydrogen1));

    let raw_2d = bundle.only_2d_by_source(LoadedSourceFilter::raw())?;
    assert_eq!(raw_2d.shape(), (2048, 512));

    let (hsqc, hsqc_source) =
        bundle.only_loaded_2d_by_source(LoadedSourceFilter::path(hsqc_path))?;
    assert_eq!(hsqc.shape(), (1024, 32));
    assert_eq!(hsqc_source.path(), Some(hsqc_path));

    assert_single_error(
        bundle.only_loaded_1d_by_source(LoadedSourceFilter::from(LoadedSourceFormat::JeolJdf)),
        "expected exactly one one-dimensional spectrum for source format jeol_jdf",
        "found 2 one-dimensional and 1 two-dimensional spectra",
    )?;

    let carbon = load_spectra(nmrxiv_fixture_root())?
        .into_only_1d_by_source(LoadedSourceFilter::path(carbon_path))?;
    assert_eq!(carbon.metadata.nucleus, Some(Nucleus::Carbon13));

    let (bruker_2d, bruker_source) = load_spectra(nmrxiv_fixture_root())?
        .into_only_loaded_2d_by_source(LoadedSourceFilter::from(LoadedSourceVendor::Bruker))?;
    assert_eq!(bruker_2d.shape(), (2048, 512));
    assert_eq!(bruker_source.format(), "bruker_ser");
    Ok(())
}

#[test]
fn generic_source_set_exact_helpers_select_matching_dimension() -> anyhow::Result<()> {
    let bundle = load_spectra(nmrxiv_fixture_root())?;
    let carbon_path = Path::new("jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx");
    let hsqc_path = Path::new("jeol/myrcene_hsqc_400mhz.jdf");

    let carbon = bundle.only_1d_by_sources([
        LoadedSourceFilter::path(carbon_path),
        LoadedSourceFilter::vendor("missing"),
    ])?;
    assert_eq!(carbon.metadata.nucleus, Some(Nucleus::Carbon13));

    let (hsqc, source) = bundle.only_loaded_2d_by_sources([
        LoadedSourceFilter::path(hsqc_path),
        LoadedSourceFilter::vendor("missing"),
    ])?;
    assert_eq!(hsqc.shape(), (1024, 32));
    assert_eq!(source.path(), Some(hsqc_path));

    assert_single_error(
        bundle.only_1d_by_sources([LoadedSourceFilter::vendor("jeol")]),
        "expected exactly one one-dimensional spectrum for source vendor jeol",
        "found 2 one-dimensional and 1 two-dimensional spectra",
    )?;

    let varian = load_spectra(fixture_root().join("varian_1h"))?;
    let only = varian.only_1d_by_sources(std::iter::empty::<LoadedSourceFilter>())?;
    assert_eq!(only.metadata.nucleus, Some(Nucleus::Hydrogen1));
    Ok(())
}

#[test]
fn consuming_generic_source_set_exact_helpers_select_matching_dimension() -> anyhow::Result<()> {
    let carbon_path = Path::new("jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx");
    let hsqc_path = Path::new("jeol/myrcene_hsqc_400mhz.jdf");

    let carbon = load_spectra(nmrxiv_fixture_root())?.into_only_1d_by_sources([
        LoadedSourceFilter::path(carbon_path),
        LoadedSourceFilter::vendor("missing"),
    ])?;
    assert_eq!(carbon.metadata.nucleus, Some(Nucleus::Carbon13));

    let (hsqc, source) = load_spectra(nmrxiv_fixture_root())?.into_only_loaded_2d_by_sources([
        LoadedSourceFilter::path(hsqc_path),
        LoadedSourceFilter::vendor("missing"),
    ])?;
    assert_eq!(hsqc.shape(), (1024, 32));
    assert_eq!(source.path(), Some(hsqc_path));
    Ok(())
}

#[test]
fn source_data_kind_exact_helpers_select_matching_dimension() -> anyhow::Result<()> {
    let bundle = load_spectra(nmrxiv_fixture_root())?;

    let raw_1d = bundle.only_1d_by_source_data_kind(LoadedSourceDataKind::Raw)?;
    assert_eq!(raw_1d.metadata.nucleus, Some(Nucleus::Hydrogen1));

    let (raw_2d, raw_source) =
        bundle.only_loaded_2d_by_source_data_kind(LoadedSourceDataKind::Raw)?;
    assert_eq!(raw_2d.shape(), (2048, 512));
    assert_eq!(raw_source.data_kind(), LoadedSourceDataKind::Raw);

    assert_single_error(
        bundle.only_loaded_1d_by_source_data_kind(LoadedSourceDataKind::Other),
        "expected exactly one one-dimensional spectrum for source data kind other",
        "found 4 one-dimensional and 1 two-dimensional spectra",
    )?;

    let raw_1d = load_spectra(nmrxiv_fixture_root())?
        .into_only_1d_by_source_data_kind(LoadedSourceDataKind::Raw)?;
    assert_eq!(raw_1d.metadata.nucleus, Some(Nucleus::Hydrogen1));

    let (raw_2d, raw_source) = load_spectra(nmrxiv_fixture_root())?
        .into_only_loaded_2d_by_source_data_kind(LoadedSourceDataKind::Raw)?;
    assert_eq!(raw_2d.shape(), (2048, 512));
    assert_eq!(raw_source.data_kind(), LoadedSourceDataKind::Raw);
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
fn consuming_source_path_exact_helpers_select_matching_dimension() -> anyhow::Result<()> {
    let carbon_path = Path::new("jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx");
    let hsqc_path = Path::new("jeol/myrcene_hsqc_400mhz.jdf");

    let carbon = load_spectra(nmrxiv_fixture_root())?.into_only_1d_by_source_path(carbon_path)?;
    assert_eq!(carbon.metadata.nucleus, Some(Nucleus::Carbon13));

    let (hsqc, hsqc_source) =
        load_spectra(nmrxiv_fixture_root())?.into_only_loaded_2d_by_source_path(hsqc_path)?;
    assert_eq!(hsqc.shape(), (1024, 32));
    assert_eq!(hsqc_source.path(), Some(hsqc_path));

    assert_single_error(
        load_spectra(nmrxiv_fixture_root())?.into_only_loaded_1d_by_source_path(hsqc_path),
        "expected exactly one one-dimensional spectrum for source path jeol/myrcene_hsqc_400mhz.jdf",
        "found 0 one-dimensional and 1 two-dimensional spectra",
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
fn reader_source_path_exact_helpers_select_matching_dimension() -> anyhow::Result<()> {
    let carbon_path = Path::new("jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx");
    let hsqc_path = Path::new("jeol/myrcene_hsqc_400mhz.jdf");

    let carbon = RSpinReader::new().read_1d_by_source_path(nmrxiv_fixture_root(), carbon_path)?;
    assert_eq!(carbon.metadata.nucleus, Some(Nucleus::Carbon13));

    let (hsqc, hsqc_source) =
        RSpinReader::new().read_2d_with_source_by_source_path(nmrxiv_fixture_root(), hsqc_path)?;
    assert_eq!(hsqc.shape(), (1024, 32));
    assert_eq!(hsqc_source.path(), Some(hsqc_path));

    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/nmrxiv/cc0");
    let anchored_carbon_path = Path::new("myrcene/jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx");
    let (carbon, carbon_source) = RSpinReader::new()
        .read_1d_with_source_by_source_path_relative_to(&base, "myrcene", anchored_carbon_path)?;
    assert_eq!(carbon.metadata.nucleus, Some(Nucleus::Carbon13));
    assert_eq!(carbon_source.path(), Some(anchored_carbon_path));

    let hsqc = RSpinReader::new().read_2d_by_source_path_relative_to(
        &base,
        "myrcene",
        Path::new("myrcene/jeol/myrcene_hsqc_400mhz.jdf"),
    )?;
    assert_eq!(hsqc.shape(), (1024, 32));
    Ok(())
}

#[test]
fn reader_generic_source_filter_exact_helpers_select_matching_dimension() -> anyhow::Result<()> {
    let reader = RSpinReader::new();
    let root = nmrxiv_fixture_root();
    let hsqc_path = Path::new("jeol/myrcene_hsqc_400mhz.jdf");

    let bruker_1d = reader.read_1d_by_source(&root, LoadedSourceFilter::vendor("bruker"))?;
    assert_eq!(bruker_1d.metadata.nucleus, Some(Nucleus::Hydrogen1));

    let (bruker_2d, bruker_source) =
        reader.read_2d_with_source_by_source(&root, LoadedSourceFilter::vendor("bruker"))?;
    assert_eq!(bruker_2d.shape(), (2048, 512));
    assert_eq!(bruker_source.format(), "bruker_ser");

    let hsqc = reader.read_2d_by_source(&root, LoadedSourceFilter::path(hsqc_path))?;
    assert_eq!(hsqc.shape(), (1024, 32));

    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/nmrxiv/cc0");
    let anchored_bruker_path = Path::new("myrcene/bruker_1h_raw");
    let (bruker_1d, bruker_source) = reader.read_1d_with_source_by_source_relative_to(
        &base,
        "myrcene",
        LoadedSourceFilter::path(anchored_bruker_path),
    )?;
    assert_eq!(bruker_1d.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(bruker_source.path(), Some(anchored_bruker_path));

    let bruker_2d = reader.read_2d_by_source_relative_to(
        &base,
        "myrcene",
        LoadedSourceFilter::from(LoadedSourceVendor::Bruker),
    )?;
    assert_eq!(bruker_2d.shape(), (2048, 512));
    Ok(())
}

#[test]
fn reader_generic_source_set_exact_helpers_select_matching_dimension() -> anyhow::Result<()> {
    let reader = RSpinReader::new();
    let root = nmrxiv_fixture_root();
    let hsqc_path = Path::new("jeol/myrcene_hsqc_400mhz.jdf");

    let bruker_1d = reader.read_1d_by_sources(
        &root,
        [
            LoadedSourceFilter::vendor("bruker"),
            LoadedSourceFilter::path("missing"),
        ],
    )?;
    assert_eq!(bruker_1d.metadata.nucleus, Some(Nucleus::Hydrogen1));

    let (hsqc, source) = reader.read_2d_with_source_by_sources(
        &root,
        [
            LoadedSourceFilter::path(hsqc_path),
            LoadedSourceFilter::vendor("missing"),
        ],
    )?;
    assert_eq!(hsqc.shape(), (1024, 32));
    assert_eq!(source.path(), Some(hsqc_path));

    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/nmrxiv/cc0");
    let anchored_bruker_path = Path::new("myrcene/bruker_1h_raw");
    let (bruker_1d, bruker_source) = reader.read_1d_with_source_by_sources_relative_to(
        &base,
        "myrcene",
        [LoadedSourceFilter::path(anchored_bruker_path)],
    )?;
    assert_eq!(bruker_1d.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(bruker_source.path(), Some(anchored_bruker_path));
    Ok(())
}

#[test]
fn reader_source_data_kind_exact_helpers_select_matching_dimension() -> anyhow::Result<()> {
    let reader = RSpinReader::new();
    let root = nmrxiv_fixture_root();

    let raw_1d = reader.read_1d_by_source_data_kind(&root, LoadedSourceDataKind::Raw)?;
    assert_eq!(raw_1d.metadata.nucleus, Some(Nucleus::Hydrogen1));

    let (raw_2d, raw_source) =
        reader.read_2d_with_source_by_source_data_kind(&root, LoadedSourceDataKind::Raw)?;
    assert_eq!(raw_2d.shape(), (2048, 512));
    assert_eq!(raw_source.data_kind(), LoadedSourceDataKind::Raw);

    assert_single_error(
        reader.read_1d_by_source_data_kind(&root, LoadedSourceDataKind::Other),
        "expected exactly one one-dimensional spectrum for source data kind other",
        "found 4 one-dimensional and 1 two-dimensional spectra",
    )?;

    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/nmrxiv/cc0");
    let (raw_1d, raw_source) = reader.read_1d_with_source_by_source_data_kind_relative_to(
        &base,
        "myrcene",
        LoadedSourceDataKind::Raw,
    )?;
    assert_eq!(raw_1d.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(raw_source.path(), Some(Path::new("myrcene/bruker_1h_raw")));

    let raw_2d = reader.read_2d_by_source_data_kind_relative_to(
        &base,
        "myrcene",
        LoadedSourceDataKind::Raw,
    )?;
    assert_eq!(raw_2d.shape(), (2048, 512));
    Ok(())
}

#[test]
fn relative_source_filtered_helpers_anchor_source_paths() -> anyhow::Result<()> {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/nmrxiv/cc0");
    let reader = RSpinReader::new();

    let bruker_1d = reader.read_1d_by_source_format_relative_to(
        &base,
        "myrcene",
        LoadedSourceFormat::BrukerFid,
    )?;
    assert_eq!(bruker_1d.metadata.nucleus, Some(Nucleus::Hydrogen1));

    let (bruker_1d, bruker_source) = reader.read_1d_with_source_by_source_format_relative_to(
        &base,
        "myrcene",
        LoadedSourceFormat::BrukerFid,
    )?;
    assert_eq!(bruker_1d.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(bruker_source.format(), "bruker_fid");
    assert_eq!(
        bruker_source.path(),
        Some(Path::new("myrcene/bruker_1h_raw"))
    );

    let bruker_2d = reader.read_2d_by_source_vendor_relative_to(&base, "myrcene", "bruker")?;
    assert_eq!(bruker_2d.shape(), (2048, 512));

    let (bruker_2d, bruker_source) = reader.read_2d_with_source_by_source_vendor_relative_to(
        &base,
        "myrcene",
        LoadedSourceVendor::Bruker,
    )?;
    assert_eq!(bruker_2d.shape(), (2048, 512));
    assert_eq!(bruker_source.format(), "bruker_ser");
    assert_eq!(
        bruker_source.path(),
        Some(Path::new("myrcene/bruker_cosy_raw"))
    );

    let bruker_1d = load_spectrum_1d_by_source_format_relative_to(&base, "myrcene", "bruker_fid")?;
    assert_eq!(bruker_1d.metadata.nucleus, Some(Nucleus::Hydrogen1));

    let bruker_1d = load_spectrum_1d_by_source_vendor_relative_to(&base, "myrcene", "bruker")?;
    assert_eq!(bruker_1d.metadata.nucleus, Some(Nucleus::Hydrogen1));

    let (bruker_1d, bruker_source) = load_spectrum_1d_with_source_by_source_format_relative_to(
        &base,
        "myrcene",
        LoadedSourceFormat::BrukerFid,
    )?;
    assert_eq!(bruker_1d.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(
        bruker_source.path(),
        Some(Path::new("myrcene/bruker_1h_raw"))
    );

    let (_, bruker_source) = load_spectrum_1d_with_source_by_source_vendor_relative_to(
        &base,
        "myrcene",
        LoadedSourceVendor::Bruker,
    )?;
    assert_eq!(bruker_source.format(), "bruker_fid");

    let jeol_2d = load_spectrum_2d_by_source_format_relative_to(&base, "myrcene", "jdf")?;
    assert_eq!(jeol_2d.shape(), (1024, 32));

    let bruker_2d = load_spectrum_2d_by_source_vendor_relative_to(
        &base,
        "myrcene",
        LoadedSourceVendor::Bruker,
    )?;
    assert_eq!(bruker_2d.shape(), (2048, 512));

    let (jeol_2d, jeol_source) =
        load_spectrum_2d_with_source_by_source_format_relative_to(&base, "myrcene", "jdf")?;
    assert_eq!(jeol_2d.shape(), (1024, 32));
    assert_eq!(
        jeol_source.path(),
        Some(Path::new("myrcene/jeol/myrcene_hsqc_400mhz.jdf"))
    );

    let (bruker_2d, bruker_source) =
        load_spectrum_2d_with_source_by_source_vendor_relative_to(&base, "myrcene", "bruker")?;
    assert_eq!(bruker_2d.shape(), (2048, 512));
    assert_eq!(
        bruker_source.path(),
        Some(Path::new("myrcene/bruker_cosy_raw"))
    );
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

#[test]
fn free_source_data_kind_exact_helpers_select_matching_dimension() -> anyhow::Result<()> {
    let root = nmrxiv_fixture_root();

    let raw_1d = load_spectrum_1d_by_source_data_kind(&root, LoadedSourceDataKind::Raw)?;
    assert_eq!(raw_1d.metadata.nucleus, Some(Nucleus::Hydrogen1));

    let (raw_1d, raw_source) =
        load_spectrum_1d_with_source_by_source_data_kind(&root, LoadedSourceDataKind::Raw)?;
    assert_eq!(raw_1d.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(raw_source.data_kind(), LoadedSourceDataKind::Raw);

    let raw_2d = load_spectrum_2d_by_source_data_kind(&root, LoadedSourceDataKind::Raw)?;
    assert_eq!(raw_2d.shape(), (2048, 512));

    let (raw_2d, raw_source) =
        load_spectrum_2d_with_source_by_source_data_kind(&root, LoadedSourceDataKind::Raw)?;
    assert_eq!(raw_2d.shape(), (2048, 512));
    assert_eq!(raw_source.data_kind(), LoadedSourceDataKind::Raw);

    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/nmrxiv/cc0");
    let raw_1d = load_spectrum_1d_by_source_data_kind_relative_to(
        &base,
        "myrcene",
        LoadedSourceDataKind::Raw,
    )?;
    assert_eq!(raw_1d.metadata.nucleus, Some(Nucleus::Hydrogen1));

    let (_, raw_source) = load_spectrum_1d_with_source_by_source_data_kind_relative_to(
        &base,
        "myrcene",
        LoadedSourceDataKind::Raw,
    )?;
    assert_eq!(raw_source.path(), Some(Path::new("myrcene/bruker_1h_raw")));

    let raw_2d = load_spectrum_2d_by_source_data_kind_relative_to(
        &base,
        "myrcene",
        LoadedSourceDataKind::Raw,
    )?;
    assert_eq!(raw_2d.shape(), (2048, 512));

    let (_, raw_source) = load_spectrum_2d_with_source_by_source_data_kind_relative_to(
        &base,
        "myrcene",
        LoadedSourceDataKind::Raw,
    )?;
    assert_eq!(
        raw_source.path(),
        Some(Path::new("myrcene/bruker_cosy_raw"))
    );
    Ok(())
}

#[test]
fn free_source_path_exact_helpers_select_matching_dimension() -> anyhow::Result<()> {
    let carbon_path = Path::new("jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx");
    let hsqc_path = Path::new("jeol/myrcene_hsqc_400mhz.jdf");

    let carbon = load_spectrum_1d_by_source_path(nmrxiv_fixture_root(), carbon_path)?;
    assert_eq!(carbon.metadata.nucleus, Some(Nucleus::Carbon13));

    let (carbon, carbon_source) =
        load_spectrum_1d_with_source_by_source_path(nmrxiv_fixture_root(), carbon_path)?;
    assert_eq!(carbon.metadata.nucleus, Some(Nucleus::Carbon13));
    assert_eq!(carbon_source.path(), Some(carbon_path));

    let hsqc = load_spectrum_2d_by_source_path(nmrxiv_fixture_root(), hsqc_path)?;
    assert_eq!(hsqc.shape(), (1024, 32));

    let (hsqc, hsqc_source) =
        load_spectrum_2d_with_source_by_source_path(nmrxiv_fixture_root(), hsqc_path)?;
    assert_eq!(hsqc.shape(), (1024, 32));
    assert_eq!(hsqc_source.path(), Some(hsqc_path));

    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/nmrxiv/cc0");
    let anchored_carbon_path = Path::new("myrcene/jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx");
    let anchored_hsqc_path = Path::new("myrcene/jeol/myrcene_hsqc_400mhz.jdf");

    let carbon =
        load_spectrum_1d_by_source_path_relative_to(&base, "myrcene", anchored_carbon_path)?;
    assert_eq!(carbon.metadata.nucleus, Some(Nucleus::Carbon13));

    let (_, carbon_source) = load_spectrum_1d_with_source_by_source_path_relative_to(
        &base,
        "myrcene",
        anchored_carbon_path,
    )?;
    assert_eq!(carbon_source.path(), Some(anchored_carbon_path));

    let hsqc = load_spectrum_2d_by_source_path_relative_to(&base, "myrcene", anchored_hsqc_path)?;
    assert_eq!(hsqc.shape(), (1024, 32));

    let (_, hsqc_source) = load_spectrum_2d_with_source_by_source_path_relative_to(
        &base,
        "myrcene",
        anchored_hsqc_path,
    )?;
    assert_eq!(hsqc_source.path(), Some(anchored_hsqc_path));
    Ok(())
}

#[test]
fn free_source_path_prefix_exact_helpers_select_matching_dimension() -> anyhow::Result<()> {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/zenodo_7100132");
    let bruker = base.join("bruker_without_expno");

    let processed = load_spectrum_1d_by_source_path_prefix(&bruker, "pdata")?;
    assert_eq!(processed.metadata.nucleus, Some(Nucleus::Hydrogen1));

    let (_, processed_source) =
        load_spectrum_1d_with_source_by_source_path_prefix(&bruker, "pdata")?;
    assert_eq!(processed_source.path(), Some(Path::new("pdata/1")));

    let processed = load_spectrum_1d_by_source_path_prefix_relative_to(
        &base,
        "bruker_without_expno",
        "bruker_without_expno/pdata",
    )?;
    assert_eq!(processed.metadata.nucleus, Some(Nucleus::Hydrogen1));

    let (_, processed_source) = load_spectrum_1d_with_source_by_source_path_prefix_relative_to(
        &base,
        "bruker_without_expno",
        "bruker_without_expno/pdata",
    )?;
    assert_eq!(
        processed_source.path(),
        Some(Path::new("bruker_without_expno/pdata/1"))
    );

    let hsqc = load_spectrum_2d_by_source_path_prefix(nmrxiv_fixture_root(), "jeol")?;
    assert_eq!(hsqc.shape(), (1024, 32));

    let (_, hsqc_source) =
        load_spectrum_2d_with_source_by_source_path_prefix(nmrxiv_fixture_root(), "jeol")?;
    assert_eq!(
        hsqc_source.path(),
        Some(Path::new("jeol/myrcene_hsqc_400mhz.jdf"))
    );

    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/nmrxiv/cc0");
    let hsqc =
        load_spectrum_2d_by_source_path_prefix_relative_to(&base, "myrcene", "myrcene/jeol")?;
    assert_eq!(hsqc.shape(), (1024, 32));

    let (_, hsqc_source) = load_spectrum_2d_with_source_by_source_path_prefix_relative_to(
        &base,
        "myrcene",
        "myrcene/jeol",
    )?;
    assert_eq!(
        hsqc_source.path(),
        Some(Path::new("myrcene/jeol/myrcene_hsqc_400mhz.jdf"))
    );
    Ok(())
}

#[test]
fn free_generic_source_filter_exact_helpers_select_matching_dimension() -> anyhow::Result<()> {
    let root = nmrxiv_fixture_root();
    let hsqc_path = Path::new("jeol/myrcene_hsqc_400mhz.jdf");

    let bruker_1d = load_spectrum_1d_by_source(&root, LoadedSourceFilter::vendor("bruker"))?;
    assert_eq!(bruker_1d.metadata.nucleus, Some(Nucleus::Hydrogen1));

    let (bruker_1d, bruker_source) =
        load_spectrum_1d_with_source_by_source(&root, LoadedSourceFilter::format("bruker_fid"))?;
    assert_eq!(bruker_1d.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(bruker_source.format(), "bruker_fid");

    let bruker_2d = load_spectrum_2d_by_source(&root, LoadedSourceFilter::vendor("bruker"))?;
    assert_eq!(bruker_2d.shape(), (2048, 512));

    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/nmrxiv/cc0");
    let anchored_hsqc_path = Path::new("myrcene/jeol/myrcene_hsqc_400mhz.jdf");
    let hsqc = load_spectrum_2d_by_source_relative_to(
        &base,
        "myrcene",
        LoadedSourceFilter::path(anchored_hsqc_path),
    )?;
    assert_eq!(hsqc.shape(), (1024, 32));

    let (_, hsqc_source) =
        load_spectrum_2d_with_source_by_source(&root, LoadedSourceFilter::path(hsqc_path))?;
    assert_eq!(hsqc_source.path(), Some(hsqc_path));

    let carbon = load_spectrum_1d_by_source_relative_to(
        &base,
        "myrcene",
        LoadedSourceFilter::path("myrcene/jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx"),
    )?;
    assert_eq!(carbon.metadata.nucleus, Some(Nucleus::Carbon13));

    let (_, carbon_source) = load_spectrum_1d_with_source_by_source_relative_to(
        &base,
        "myrcene",
        LoadedSourceFilter::path("myrcene/jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx"),
    )?;
    assert_eq!(
        carbon_source.path(),
        Some(Path::new(
            "myrcene/jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx"
        ))
    );
    Ok(())
}

#[test]
fn free_generic_source_set_exact_helpers_select_matching_dimension() -> anyhow::Result<()> {
    let root = nmrxiv_fixture_root();
    let carbon_path = Path::new("jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx");
    let hsqc_path = Path::new("jeol/myrcene_hsqc_400mhz.jdf");

    let carbon = load_spectrum_1d_by_sources(&root, [LoadedSourceFilter::path(carbon_path)])?;
    assert_eq!(carbon.metadata.nucleus, Some(Nucleus::Carbon13));

    let (hsqc, source) =
        load_spectrum_2d_with_source_by_sources(&root, [LoadedSourceFilter::path(hsqc_path)])?;
    assert_eq!(hsqc.shape(), (1024, 32));
    assert_eq!(source.path(), Some(hsqc_path));

    let hsqc = load_spectrum_2d_by_sources(
        &root,
        [
            LoadedSourceFilter::vendor("missing"),
            LoadedSourceFilter::path(hsqc_path),
        ],
    )?;
    assert_eq!(hsqc.shape(), (1024, 32));

    let base = fixture_root();
    let paths = [base.join("varian_1h"), base.join("bruker_without_expno")];
    let processed =
        load_spectrum_1d_many_by_sources(&paths, [LoadedSourceFilter::path("pdata/1")])?;
    assert_eq!(processed.metadata.nucleus, Some(Nucleus::Hydrogen1));

    let (varian, source) = load_spectrum_1d_many_with_source_by_sources_relative_to(
        &base,
        ["varian_1h", "bruker_without_expno"],
        [LoadedSourceFilter::path("varian_1h")],
    )?;
    assert_eq!(varian.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(source.path(), Some(Path::new("varian_1h")));

    let nmrxiv_base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/nmrxiv/cc0");
    let bruker_2d = load_spectrum_2d_many_by_sources_relative_to(
        &nmrxiv_base,
        ["myrcene/bruker_cosy_raw"],
        [LoadedSourceFilter::vendor("bruker")],
    )?;
    assert_eq!(bruker_2d.shape(), (2048, 512));
    Ok(())
}

#[test]
fn free_many_source_filtered_exact_helpers_select_matching_dimension() -> anyhow::Result<()> {
    let root = nmrxiv_fixture_root();
    let proton_jcamp = root.join("jcamp/myrcene_1h_400mhz_jcamp_dx_6_link.jdx");
    let carbon_jcamp = root.join("jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx");
    let bruker_2d = root.join("bruker_cosy_raw");
    let jeol_1d = root.join("jeol/myrcene_1h_400mhz.jdf");
    let jeol_2d = root.join("jeol/myrcene_hsqc_400mhz.jdf");

    let proton = load_spectrum_1d_many_by_source_format(
        [&proton_jcamp, &bruker_2d],
        LoadedSourceFormat::JcampDx,
    )?;
    assert_eq!(proton.metadata.nucleus, Some(Nucleus::Hydrogen1));

    let (jeol, jeol_source) =
        load_spectrum_1d_many_with_source_by_source_vendor([&jeol_1d, &bruker_2d], "jeol")?;
    assert_eq!(jeol.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(jeol_source.format(), "jeol_jdf");

    let raw_1d = load_spectrum_1d_many_by_source_data_kind(
        [root.join("bruker_1h_raw"), bruker_2d.clone()],
        LoadedSourceDataKind::Raw,
    )?;
    assert_eq!(raw_1d.metadata.nucleus, Some(Nucleus::Hydrogen1));

    let (raw_1d, raw_source) = load_spectrum_1d_many_with_source_by_source_data_kind(
        [root.join("bruker_1h_raw"), bruker_2d.clone()],
        LoadedSourceDataKind::Raw,
    )?;
    assert_eq!(raw_1d.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(raw_source.data_kind(), LoadedSourceDataKind::Raw);

    let raw_2d = load_spectrum_2d_many_by_source_data_kind(
        [root.join("bruker_1h_raw"), bruker_2d.clone()],
        LoadedSourceDataKind::Raw,
    )?;
    assert_eq!(raw_2d.shape(), (2048, 512));

    let (raw_2d, raw_source) = load_spectrum_2d_many_with_source_by_source_data_kind(
        [root.join("bruker_1h_raw"), bruker_2d.clone()],
        LoadedSourceDataKind::Raw,
    )?;
    assert_eq!(raw_2d.shape(), (2048, 512));
    assert_eq!(raw_source.data_kind(), LoadedSourceDataKind::Raw);

    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/nmrxiv/cc0");
    let hsqc = load_spectrum_2d_many_by_source_format_relative_to(
        &base,
        [
            "myrcene/jcamp/myrcene_1h_400mhz_jcamp_dx_6_link.jdx",
            "myrcene",
        ],
        "jdf",
    )?;
    assert_eq!(hsqc.shape(), (1024, 32));

    let (hsqc, hsqc_source) = load_spectrum_2d_many_with_source_by_source_path(
        [&jeol_1d, &jeol_2d],
        "myrcene_hsqc_400mhz.jdf",
    )?;
    assert_eq!(hsqc.shape(), (1024, 32));
    assert_eq!(hsqc_source.format(), "jeol_jdf");

    let carbon_path = Path::new("myrcene/jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx");
    let carbon = load_spectrum_1d_many_by_source_path_relative_to(
        &base,
        ["myrcene", "myrcene/bruker_cosy_raw"],
        carbon_path,
    )?;
    assert_eq!(carbon.metadata.nucleus, Some(Nucleus::Carbon13));

    let (_, bruker_source) = RSpinReader::new()
        .read_2d_many_with_source_by_source_vendor_relative_to(
            &base,
            ["myrcene/bruker_1h_raw", "myrcene/bruker_cosy_raw"],
            LoadedSourceVendor::Bruker,
        )?;
    assert_eq!(bruker_source.format(), "bruker_ser");

    let raw_1d = load_spectrum_1d_many_by_source_data_kind_relative_to(
        &base,
        ["myrcene/bruker_1h_raw", "myrcene/bruker_cosy_raw"],
        LoadedSourceDataKind::Raw,
    )?;
    assert_eq!(raw_1d.metadata.nucleus, Some(Nucleus::Hydrogen1));

    let (_, raw_source) = load_spectrum_1d_many_with_source_by_source_data_kind_relative_to(
        &base,
        ["myrcene/bruker_1h_raw", "myrcene/bruker_cosy_raw"],
        LoadedSourceDataKind::Raw,
    )?;
    assert_eq!(raw_source.path(), Some(Path::new("myrcene/bruker_1h_raw")));

    let raw_2d = load_spectrum_2d_many_by_source_data_kind_relative_to(
        &base,
        ["myrcene/bruker_1h_raw", "myrcene/bruker_cosy_raw"],
        LoadedSourceDataKind::Raw,
    )?;
    assert_eq!(raw_2d.shape(), (2048, 512));

    let (_, raw_source) = load_spectrum_2d_many_with_source_by_source_data_kind_relative_to(
        &base,
        ["myrcene/bruker_1h_raw", "myrcene/bruker_cosy_raw"],
        LoadedSourceDataKind::Raw,
    )?;
    assert_eq!(
        raw_source.path(),
        Some(Path::new("myrcene/bruker_cosy_raw"))
    );

    assert_single_error(
        load_spectrum_1d_many_by_source_format([&proton_jcamp, &carbon_jcamp], "jdx"),
        "expected exactly one one-dimensional spectrum for source format jcamp_dx",
        "found 2 one-dimensional and 0 two-dimensional spectra",
    )?;
    Ok(())
}

#[test]
fn free_many_source_path_prefix_exact_helpers_select_matching_dimension() -> anyhow::Result<()> {
    let root = nmrxiv_fixture_root();
    let proton_jcamp = root.join("jcamp/myrcene_1h_400mhz_jcamp_dx_6_link.jdx");
    let jeol_1d = root.join("jeol/myrcene_1h_400mhz.jdf");
    let jeol_2d = root.join("jeol/myrcene_hsqc_400mhz.jdf");
    let bruker_2d = root.join("bruker_cosy_raw");

    let proton = load_spectrum_1d_many_by_source_path_prefix(
        [&proton_jcamp, &bruker_2d],
        "myrcene_1h_400mhz_jcamp_dx_6_link.jdx",
    )?;
    assert_eq!(proton.metadata.nucleus, Some(Nucleus::Hydrogen1));

    let (_, jeol_source) = load_spectrum_1d_many_with_source_by_source_path_prefix(
        [&jeol_1d, &bruker_2d],
        "myrcene_1h_400mhz.jdf",
    )?;
    assert_eq!(jeol_source.format(), "jeol_jdf");

    let (hsqc, hsqc_source) = load_spectrum_2d_many_with_source_by_source_path_prefix(
        [&jeol_1d, &jeol_2d],
        "myrcene_hsqc_400mhz.jdf",
    )?;
    assert_eq!(hsqc.shape(), (1024, 32));
    assert_eq!(hsqc_source.format(), "jeol_jdf");

    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/nmrxiv/cc0");
    let carbon = load_spectrum_1d_many_by_source_path_prefix_relative_to(
        &base,
        ["myrcene", "myrcene/bruker_cosy_raw"],
        "myrcene/jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx",
    )?;
    assert_eq!(carbon.metadata.nucleus, Some(Nucleus::Carbon13));

    let (_, carbon_source) = load_spectrum_1d_many_with_source_by_source_path_prefix_relative_to(
        &base,
        ["myrcene", "myrcene/bruker_cosy_raw"],
        "myrcene/jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx",
    )?;
    assert_eq!(
        carbon_source.path(),
        Some(Path::new(
            "myrcene/jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx"
        ))
    );

    let cosy = load_spectrum_2d_many_by_source_path_prefix_relative_to(
        &base,
        ["myrcene/bruker_1h_raw", "myrcene/bruker_cosy_raw"],
        "myrcene/bruker_cosy_raw",
    )?;
    assert_eq!(cosy.shape(), (2048, 512));
    Ok(())
}

#[test]
fn free_many_generic_source_filter_exact_helpers_select_matching_dimension() -> anyhow::Result<()> {
    let root = nmrxiv_fixture_root();
    let bruker_2d = root.join("bruker_cosy_raw");
    let jeol_1d = root.join("jeol/myrcene_1h_400mhz.jdf");
    let jeol_2d = root.join("jeol/myrcene_hsqc_400mhz.jdf");

    let jeol = load_spectrum_1d_many_by_source(
        [&jeol_1d, &bruker_2d],
        LoadedSourceFilter::vendor("jeol"),
    )?;
    assert_eq!(jeol.metadata.nucleus, Some(Nucleus::Hydrogen1));

    let (jeol, jeol_source) = load_spectrum_1d_many_with_source_by_source(
        [&jeol_1d, &bruker_2d],
        LoadedSourceFilter::path("myrcene_1h_400mhz.jdf"),
    )?;
    assert_eq!(jeol.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(jeol_source.format(), "jeol_jdf");

    let (hsqc, hsqc_source) = load_spectrum_2d_many_with_source_by_source(
        [&jeol_1d, &jeol_2d],
        LoadedSourceFilter::path("myrcene_hsqc_400mhz.jdf"),
    )?;
    assert_eq!(hsqc.shape(), (1024, 32));
    assert_eq!(hsqc_source.format(), "jeol_jdf");

    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/nmrxiv/cc0");
    let bruker_2d = load_spectrum_2d_many_by_source_relative_to(
        &base,
        ["myrcene/bruker_1h_raw", "myrcene/bruker_cosy_raw"],
        LoadedSourceFilter::from(LoadedSourceVendor::Bruker),
    )?;
    assert_eq!(bruker_2d.shape(), (2048, 512));

    let (_, bruker_source) = load_spectrum_1d_many_with_source_by_source_relative_to(
        &base,
        ["myrcene/bruker_1h_raw", "myrcene/bruker_cosy_raw"],
        LoadedSourceFilter::path("myrcene/bruker_1h_raw"),
    )?;
    assert_eq!(bruker_source.format(), "bruker_fid");

    assert_single_error(
        RSpinReader::new().read_1d_many_by_source_relative_to(
            &base,
            ["myrcene"],
            LoadedSourceFilter::format("jdx"),
        ),
        "expected exactly one one-dimensional spectrum for source format jcamp_dx",
        "found 2 one-dimensional and 0 two-dimensional spectra",
    )?;
    Ok(())
}

fn nmrxiv_fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/nmrxiv/cc0/myrcene")
}

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/zenodo_7100132")
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
