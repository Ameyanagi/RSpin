//! Tests for discovered-source metadata filter loading.

use std::path::PathBuf;

use anyhow::Result;
use rspin_io as io;
use rspin_io::{
    LoadedSourceDataKind, LoadedSourceFormat, LoadedSourceVendor, RSpinReader, discover_spectra,
    load_discovered_spectra_by_source_data_kind, load_discovered_spectra_by_source_data_kinds,
    load_discovered_spectra_by_source_format, load_discovered_spectra_by_source_format_relative_to,
    load_discovered_spectra_by_source_formats, load_discovered_spectra_by_source_vendor,
    load_discovered_spectra_by_source_vendors, load_discovered_spectra_relative_to,
    load_discovered_spectra_strict_by_source_data_kind,
    load_discovered_spectra_strict_by_source_format,
    load_discovered_spectra_strict_by_source_vendor,
    load_discovered_spectra_summary_by_source_data_kind,
    load_discovered_spectra_summary_by_source_data_kinds,
    load_discovered_spectra_summary_by_source_format,
    load_discovered_spectra_summary_by_source_format_relative_to,
    load_discovered_spectra_summary_by_source_formats,
    load_discovered_spectra_summary_by_source_vendor,
    load_discovered_spectra_summary_by_source_vendors,
    load_discovered_spectra_summary_strict_by_source_data_kind,
    load_discovered_spectra_summary_strict_by_source_format,
    load_discovered_spectra_summary_strict_by_source_vendor,
};

#[test]
fn metadata_filter_helpers_load_discovered_bundles() -> Result<()> {
    let root = fixture_root();
    let sources = discover_spectra(&root)?;

    let fid = load_discovered_spectra_by_source_format_relative_to(&root, &sources, "varian fid")?;
    assert_eq!(fid.len(), 1);
    assert_eq!(fid.source_format_count(LoadedSourceFormat::AgilentFid), 1);
    assert_eq!(
        load_discovered_spectra_by_source_format(&root, &sources, LoadedSourceFormat::AgilentFid)?,
        fid
    );

    let selected_formats = load_discovered_spectra_by_source_formats(
        &root,
        &sources,
        [
            LoadedSourceFormat::AgilentFid,
            LoadedSourceFormat::BrukerProcessed,
        ],
    )?;
    assert_eq!(
        selected_formats.source_format_count(LoadedSourceFormat::AgilentFid),
        1
    );
    assert_eq!(
        selected_formats.source_format_count(LoadedSourceFormat::BrukerProcessed),
        1
    );

    let selected_formats_reader = RSpinReader::new()
        .read_discovered_by_source_formats_relative_to(
            &root,
            &sources,
            ["agilent fid", "bruker processed"],
        )?;
    assert_eq!(selected_formats_reader, selected_formats);

    let varian = load_discovered_spectra_by_source_vendor(&root, &sources, "varian")?;
    assert_eq!(varian.len(), 1);
    assert_eq!(
        varian.source_vendor_count(LoadedSourceVendor::AgilentVarian),
        1
    );

    let known_vendors = load_discovered_spectra_by_source_vendors(
        &root,
        &sources,
        [
            LoadedSourceVendor::Bruker,
            LoadedSourceVendor::AgilentVarian,
        ],
    )?;
    assert_eq!(
        known_vendors.source_vendor_count(LoadedSourceVendor::Bruker),
        2
    );
    assert_eq!(
        known_vendors.source_vendor_count(LoadedSourceVendor::AgilentVarian),
        1
    );

    let raw =
        load_discovered_spectra_by_source_data_kind(&root, &sources, LoadedSourceDataKind::Raw)?;
    assert_eq!(raw.len(), 2);
    assert_eq!(raw.source_data_kind_count(LoadedSourceDataKind::Raw), 2);

    let raw_or_processed = load_discovered_spectra_by_source_data_kinds(
        &root,
        &sources,
        [LoadedSourceDataKind::Raw, LoadedSourceDataKind::Processed],
    )?;
    assert_eq!(raw_or_processed, known_vendors);

    let unrestricted = RSpinReader::new().read_discovered_by_source_formats(
        &root,
        &sources,
        std::iter::empty::<&str>(),
    )?;
    assert_eq!(
        unrestricted,
        load_discovered_spectra_relative_to(&root, &sources)?
    );

    Ok(())
}

#[test]
fn strict_metadata_filter_helpers_match_loaded_bundles() -> Result<()> {
    let root = fixture_root();
    let sources = discover_spectra(&root)?;

    let fid = load_discovered_spectra_by_source_format(&root, &sources, "agilent fid")?;
    assert_eq!(
        load_discovered_spectra_strict_by_source_format(&root, &sources, "agilent fid")?,
        fid
    );

    let varian = load_discovered_spectra_by_source_vendor(&root, &sources, "varian")?;
    assert_eq!(
        load_discovered_spectra_strict_by_source_vendor(
            &root,
            &sources,
            LoadedSourceVendor::AgilentVarian,
        )?,
        varian
    );

    let raw =
        load_discovered_spectra_by_source_data_kind(&root, &sources, LoadedSourceDataKind::Raw)?;
    assert_eq!(
        load_discovered_spectra_strict_by_source_data_kind(
            &root,
            &sources,
            LoadedSourceDataKind::Raw,
        )?,
        raw
    );
    Ok(())
}

#[test]
fn short_metadata_filter_aliases_load_discovered_bundles() -> Result<()> {
    let root = fixture_root();
    let sources = discover_spectra(&root)?;

    let fid = load_discovered_spectra_by_source_format(&root, &sources, "agilent fid")?;
    assert_eq!(
        io::load_discovered_spectra_by_format(&root, &sources, "varian fid")?,
        fid
    );

    let selected_formats = load_discovered_spectra_by_source_formats(
        &root,
        &sources,
        ["agilent fid", "bruker processed"],
    )?;
    assert_eq!(
        RSpinReader::new().read_discovered_by_formats_relative_to(
            &root,
            &sources,
            ["agilent fid", "bruker processed"],
        )?,
        selected_formats
    );

    let varian = load_discovered_spectra_by_source_vendor(&root, &sources, "varian")?;
    assert_eq!(
        io::load_discovered_spectra_by_vendor(&root, &sources, "varian")?,
        varian
    );

    let known_vendors =
        load_discovered_spectra_by_source_vendors(&root, &sources, ["bruker", "varian"])?;
    assert_eq!(
        RSpinReader::new().read_discovered_by_vendors(&root, &sources, ["bruker", "varian"])?,
        known_vendors
    );

    let raw =
        load_discovered_spectra_by_source_data_kind(&root, &sources, LoadedSourceDataKind::Raw)?;
    assert_eq!(
        io::load_discovered_spectra_by_data_kind(&root, &sources, LoadedSourceDataKind::Raw)?,
        raw
    );

    assert_eq!(
        RSpinReader::new().read_discovered_by_data_kinds(
            &root,
            &sources,
            [LoadedSourceDataKind::Raw, LoadedSourceDataKind::Processed],
        )?,
        known_vendors
    );

    let bruker_prefix =
        io::load_discovered_spectra_by_path_prefix(&root, &sources, "bruker_without_expno")?;
    assert_eq!(bruker_prefix.len(), 2);
    assert_eq!(
        bruker_prefix.source_vendor_count(LoadedSourceVendor::Bruker),
        2
    );

    let prefix_set = RSpinReader::new().read_discovered_by_path_prefixes(
        &root,
        &sources,
        ["bruker_without_expno", "varian_1h"],
    )?;
    assert_eq!(prefix_set, known_vendors);
    Ok(())
}

#[test]
fn metadata_filter_summary_helpers_match_loaded_bundles() -> Result<()> {
    let root = fixture_root();
    let sources = discover_spectra(&root)?;
    let fid = load_discovered_spectra_by_source_format(&root, &sources, "agilent fid")?;
    let varian = load_discovered_spectra_by_source_vendor(&root, &sources, "varian")?;
    let raw =
        load_discovered_spectra_by_source_data_kind(&root, &sources, LoadedSourceDataKind::Raw)?;

    let fid_summary = load_discovered_spectra_summary_by_source_format_relative_to(
        &root,
        &sources,
        "agilent fid",
    )?;
    assert_eq!(fid_summary, fid.summary());
    assert_eq!(
        load_discovered_spectra_summary_by_source_format(
            &root,
            &sources,
            LoadedSourceFormat::AgilentFid,
        )?,
        fid_summary
    );
    assert_eq!(
        RSpinReader::new().read_discovered_summary_by_source_formats(
            &root,
            &sources,
            ["agilent fid"],
        )?,
        fid_summary
    );

    let format_summary = load_discovered_spectra_summary_by_source_formats(
        &root,
        &sources,
        [
            LoadedSourceFormat::AgilentFid,
            LoadedSourceFormat::BrukerProcessed,
        ],
    )?;
    assert_eq!(format_summary.spectra(), 2);

    let varian_summary =
        load_discovered_spectra_summary_by_source_vendor(&root, &sources, "varian")?;
    assert_eq!(varian_summary, varian.summary());
    assert_eq!(
        load_discovered_spectra_summary_by_source_vendors(
            &root,
            &sources,
            [LoadedSourceVendor::AgilentVarian],
        )?,
        varian_summary
    );

    let raw_summary = load_discovered_spectra_summary_by_source_data_kind(
        &root,
        &sources,
        LoadedSourceDataKind::Raw,
    )?;
    assert_eq!(raw_summary, raw.summary());
    assert_eq!(
        load_discovered_spectra_summary_by_source_data_kinds(
            &root,
            &sources,
            [LoadedSourceDataKind::Raw],
        )?,
        raw_summary
    );

    assert_eq!(
        load_discovered_spectra_summary_strict_by_source_format(&root, &sources, "agilent fid",)?,
        fid.summary()
    );
    assert_eq!(
        load_discovered_spectra_summary_strict_by_source_vendor(
            &root,
            &sources,
            LoadedSourceVendor::AgilentVarian,
        )?,
        varian.summary()
    );
    assert_eq!(
        load_discovered_spectra_summary_strict_by_source_data_kind(
            &root,
            &sources,
            LoadedSourceDataKind::Raw,
        )?,
        raw.summary()
    );

    Ok(())
}

#[test]
fn dimension_metadata_filter_helpers_load_discovered_1d_bundles() -> Result<()> {
    let root = cc0_myrcene_fixture_root();
    let sources = discover_spectra(&root)?;

    let jcamp_1d =
        io::load_discovered_spectra_1d_by_source_format_relative_to(&root, &sources, "jdx")?;
    assert_eq!(jcamp_1d.len(), 2);
    assert_eq!(jcamp_1d.source_format_count(LoadedSourceFormat::JcampDx), 2);
    assert_eq!(
        io::load_discovered_spectra_1d_by_source_format(
            &root,
            &sources,
            LoadedSourceFormat::JcampDx,
        )?,
        jcamp_1d
    );

    let one_d_formats = io::load_discovered_spectra_1d_by_source_formats(
        &root,
        &sources,
        [LoadedSourceFormat::JcampDx, LoadedSourceFormat::JeolJdf],
    )?;
    assert_eq!(
        one_d_formats.source_format_count(LoadedSourceFormat::JcampDx),
        2
    );
    assert_eq!(
        one_d_formats.source_format_count(LoadedSourceFormat::JeolJdf),
        2
    );
    assert_eq!(
        RSpinReader::new().read_discovered_bundle_1d_by_source_formats_relative_to(
            &root,
            &sources,
            ["jdx", "jdf"],
        )?,
        one_d_formats
    );

    let jeol_1d = io::load_discovered_spectra_1d_by_source_vendor(&root, &sources, "jeol")?;
    assert_eq!(jeol_1d.len(), 2);
    assert_eq!(jeol_1d.source_vendor_count(LoadedSourceVendor::Jeol), 2);
    let one_d_vendors = io::load_discovered_spectra_1d_by_source_vendors(
        &root,
        &sources,
        [LoadedSourceVendor::Bruker, LoadedSourceVendor::Jeol],
    )?;
    assert_eq!(
        one_d_vendors.source_vendor_count(LoadedSourceVendor::Bruker),
        1
    );
    assert_eq!(
        one_d_vendors.source_vendor_count(LoadedSourceVendor::Jeol),
        2
    );

    let raw_1d = io::load_discovered_spectra_1d_by_source_data_kind(
        &root,
        &sources,
        LoadedSourceDataKind::Raw,
    )?;
    assert_eq!(raw_1d.len(), 1);
    assert_eq!(raw_1d.source_data_kind_count(LoadedSourceDataKind::Raw), 1);
    assert_eq!(
        io::load_discovered_spectra_1d_by_source_data_kinds(
            &root,
            &sources,
            [LoadedSourceDataKind::Raw],
        )?,
        raw_1d
    );
    assert_eq!(
        io::load_discovered_spectra_1d_strict_by_source_format(&root, &sources, "jcamp")?,
        jcamp_1d
    );

    Ok(())
}

#[test]
fn dimension_metadata_filter_helpers_load_discovered_2d_bundles() -> Result<()> {
    let root = cc0_myrcene_fixture_root();
    let sources = discover_spectra(&root)?;

    let jeol_2d = io::load_discovered_spectra_2d_by_source_format(
        &root,
        &sources,
        LoadedSourceFormat::JeolJdf,
    )?;
    assert_eq!(jeol_2d.len(), 1);
    assert_eq!(jeol_2d.source_format_count(LoadedSourceFormat::JeolJdf), 1);
    let two_d_formats = io::load_discovered_spectra_2d_by_source_formats(
        &root,
        &sources,
        [LoadedSourceFormat::BrukerSer, LoadedSourceFormat::JeolJdf],
    )?;
    assert_eq!(
        two_d_formats.source_format_count(LoadedSourceFormat::BrukerSer),
        1
    );
    assert_eq!(
        two_d_formats.source_format_count(LoadedSourceFormat::JeolJdf),
        1
    );
    assert_eq!(
        RSpinReader::new().read_discovered_bundle_2d_by_source_formats(
            &root,
            &sources,
            ["bruker ser", "jdf"],
        )?,
        two_d_formats
    );

    let bruker_2d = io::load_discovered_spectra_2d_by_source_vendor(&root, &sources, "bruker")?;
    assert_eq!(bruker_2d.len(), 1);
    assert_eq!(bruker_2d.source_vendor_count(LoadedSourceVendor::Bruker), 1);
    let two_d_vendors = io::load_discovered_spectra_2d_by_source_vendors(
        &root,
        &sources,
        [LoadedSourceVendor::Bruker, LoadedSourceVendor::Jeol],
    )?;
    assert_eq!(two_d_vendors, two_d_formats);

    let raw_2d = io::load_discovered_spectra_2d_by_source_data_kind(
        &root,
        &sources,
        LoadedSourceDataKind::Raw,
    )?;
    assert_eq!(raw_2d.len(), 1);
    assert_eq!(raw_2d.source_format_count(LoadedSourceFormat::BrukerSer), 1);
    assert_eq!(
        io::load_discovered_spectra_2d_by_source_data_kinds(
            &root,
            &sources,
            [LoadedSourceDataKind::Raw],
        )?,
        raw_2d
    );
    assert_eq!(
        io::load_discovered_spectra_2d_strict_by_source_vendor(
            &root,
            &sources,
            LoadedSourceVendor::Bruker,
        )?,
        bruker_2d
    );

    Ok(())
}

#[test]
fn dimension_metadata_filter_1d_summary_helpers_match_loaded_bundles() -> Result<()> {
    let root = cc0_myrcene_fixture_root();
    let sources = discover_spectra(&root)?;

    let jcamp_1d = io::load_discovered_spectra_1d_by_source_format(&root, &sources, "jcamp")?;
    assert_eq!(
        io::load_discovered_spectra_1d_summary_by_source_format(&root, &sources, "jdx")?,
        jcamp_1d.summary()
    );
    assert_eq!(
        io::load_discovered_spectra_1d_summary_by_source_formats(
            &root,
            &sources,
            [LoadedSourceFormat::JcampDx],
        )?,
        jcamp_1d.summary()
    );

    let jeol_1d = io::load_discovered_spectra_1d_by_source_vendor(&root, &sources, "jeol")?;
    assert_eq!(
        io::load_discovered_spectra_1d_summary_by_source_vendor(
            &root,
            &sources,
            LoadedSourceVendor::Jeol,
        )?,
        jeol_1d.summary()
    );
    assert_eq!(
        io::load_discovered_spectra_1d_summary_by_source_vendors(
            &root,
            &sources,
            [LoadedSourceVendor::Jeol],
        )?,
        jeol_1d.summary()
    );

    let raw_1d = io::load_discovered_spectra_1d_by_source_data_kind(
        &root,
        &sources,
        LoadedSourceDataKind::Raw,
    )?;
    assert_eq!(
        io::load_discovered_spectra_1d_summary_by_source_data_kind(
            &root,
            &sources,
            LoadedSourceDataKind::Raw,
        )?,
        raw_1d.summary()
    );
    assert_eq!(
        io::load_discovered_spectra_1d_summary_by_source_data_kinds(
            &root,
            &sources,
            [LoadedSourceDataKind::Raw],
        )?,
        raw_1d.summary()
    );
    assert_eq!(
        io::load_discovered_spectra_1d_summary_strict_by_source_vendor(&root, &sources, "jeol",)?,
        jeol_1d.summary()
    );

    Ok(())
}

#[test]
fn dimension_metadata_filter_2d_summary_helpers_match_loaded_bundles() -> Result<()> {
    let root = cc0_myrcene_fixture_root();
    let sources = discover_spectra(&root)?;

    let bruker_2d = io::load_discovered_spectra_2d_by_source_format(
        &root,
        &sources,
        LoadedSourceFormat::BrukerSer,
    )?;
    assert_eq!(
        io::load_discovered_spectra_2d_summary_by_source_format(&root, &sources, "bruker ser")?,
        bruker_2d.summary()
    );
    assert_eq!(
        io::load_discovered_spectra_2d_summary_by_source_formats(
            &root,
            &sources,
            [LoadedSourceFormat::BrukerSer],
        )?,
        bruker_2d.summary()
    );

    let jeol_2d = io::load_discovered_spectra_2d_by_source_vendor(&root, &sources, "jeol")?;
    assert_eq!(
        io::load_discovered_spectra_2d_summary_by_source_vendor(&root, &sources, "jeol")?,
        jeol_2d.summary()
    );
    assert_eq!(
        io::load_discovered_spectra_2d_summary_by_source_vendors(
            &root,
            &sources,
            [LoadedSourceVendor::Jeol],
        )?,
        jeol_2d.summary()
    );

    let raw_2d = io::load_discovered_spectra_2d_by_source_data_kind(
        &root,
        &sources,
        LoadedSourceDataKind::Raw,
    )?;
    assert_eq!(
        RSpinReader::new().read_discovered_bundle_2d_summary_by_source_data_kind(
            &root,
            &sources,
            LoadedSourceDataKind::Raw,
        )?,
        raw_2d.summary()
    );
    assert_eq!(
        io::load_discovered_spectra_2d_summary_strict_by_source_data_kinds(
            &root,
            &sources,
            [LoadedSourceDataKind::Raw],
        )?,
        raw_2d.summary()
    );
    assert_eq!(
        io::load_discovered_spectra_2d_summary_strict_by_source_format(
            &root,
            &sources,
            LoadedSourceFormat::BrukerSer,
        )?,
        bruker_2d.summary()
    );

    Ok(())
}

#[test]
fn short_dimension_metadata_aliases_load_discovered_bundles() -> Result<()> {
    let root = cc0_myrcene_fixture_root();
    let sources = discover_spectra(&root)?;

    let jcamp_1d = io::load_discovered_spectra_1d_by_source_format(&root, &sources, "jcamp")?;
    assert_eq!(
        io::load_discovered_spectra_1d_by_format(&root, &sources, "jdx")?,
        jcamp_1d
    );
    assert_eq!(
        io::load_discovered_spectra_1d_by_path_prefix(&root, &sources, "jcamp")?,
        jcamp_1d
    );

    let one_d_formats = io::load_discovered_spectra_1d_by_source_formats(
        &root,
        &sources,
        [LoadedSourceFormat::JcampDx, LoadedSourceFormat::JeolJdf],
    )?;
    assert_eq!(
        RSpinReader::new().read_discovered_bundle_1d_by_formats_relative_to(
            &root,
            &sources,
            ["jdx", "jdf"],
        )?,
        one_d_formats
    );

    let jeol_1d = io::load_discovered_spectra_1d_by_source_vendor(&root, &sources, "jeol")?;
    assert_eq!(
        io::load_discovered_spectra_1d_by_vendor(&root, &sources, "jeol")?,
        jeol_1d
    );
    let one_d_vendors = io::load_discovered_spectra_1d_by_source_vendors(
        &root,
        &sources,
        [LoadedSourceVendor::Bruker, LoadedSourceVendor::Jeol],
    )?;
    assert_eq!(
        RSpinReader::new().read_discovered_bundle_1d_by_vendors(
            &root,
            &sources,
            [LoadedSourceVendor::Bruker, LoadedSourceVendor::Jeol],
        )?,
        one_d_vendors
    );

    let raw_1d = io::load_discovered_spectra_1d_by_source_data_kind(
        &root,
        &sources,
        LoadedSourceDataKind::Raw,
    )?;
    assert_eq!(
        io::load_discovered_spectra_1d_by_data_kind(&root, &sources, LoadedSourceDataKind::Raw,)?,
        raw_1d
    );
    assert_eq!(
        RSpinReader::new()
            .strict()
            .read_discovered_bundle_1d_by_format(&root, &sources, "jcamp")?,
        jcamp_1d
    );

    let bruker_2d = io::load_discovered_spectra_2d_by_source_vendor(&root, &sources, "bruker")?;
    assert_eq!(
        io::load_discovered_spectra_2d_by_vendor(&root, &sources, LoadedSourceVendor::Bruker)?,
        bruker_2d
    );
    assert_eq!(
        io::load_discovered_spectra_2d_by_path_prefix(&root, &sources, "bruker_cosy_raw")?,
        bruker_2d
    );

    let two_d_formats = io::load_discovered_spectra_2d_by_source_formats(
        &root,
        &sources,
        [LoadedSourceFormat::BrukerSer, LoadedSourceFormat::JeolJdf],
    )?;
    assert_eq!(
        RSpinReader::new().read_discovered_bundle_2d_by_formats(
            &root,
            &sources,
            ["bruker ser", "jdf"],
        )?,
        two_d_formats
    );

    let raw_2d = io::load_discovered_spectra_2d_by_source_data_kind(
        &root,
        &sources,
        LoadedSourceDataKind::Raw,
    )?;
    assert_eq!(
        io::load_discovered_spectra_2d_by_data_kind(&root, &sources, LoadedSourceDataKind::Raw,)?,
        raw_2d
    );
    assert_eq!(
        RSpinReader::new()
            .strict()
            .read_discovered_bundle_2d_by_vendor(&root, &sources, "bruker")?,
        bruker_2d
    );

    Ok(())
}

#[test]
fn short_dimension_metadata_summary_aliases_match_loaded_bundles() -> Result<()> {
    let root = cc0_myrcene_fixture_root();
    let sources = discover_spectra(&root)?;

    let jcamp_1d = io::load_discovered_spectra_1d_by_source_format(&root, &sources, "jcamp")?;
    assert_eq!(
        io::load_discovered_spectra_1d_summary_by_format(&root, &sources, "jdx")?,
        jcamp_1d.summary()
    );
    assert_eq!(
        RSpinReader::new().read_discovered_bundle_1d_summary_by_formats(
            &root,
            &sources,
            [LoadedSourceFormat::JcampDx],
        )?,
        jcamp_1d.summary()
    );
    assert_eq!(
        io::load_discovered_spectra_1d_summary_by_path_prefix(&root, &sources, "jcamp")?,
        jcamp_1d.summary()
    );

    let jeol_1d = io::load_discovered_spectra_1d_by_source_vendor(&root, &sources, "jeol")?;
    assert_eq!(
        io::load_discovered_spectra_1d_summary_by_vendor(
            &root,
            &sources,
            LoadedSourceVendor::Jeol,
        )?,
        jeol_1d.summary()
    );
    assert_eq!(
        RSpinReader::new()
            .strict()
            .read_discovered_bundle_1d_summary_by_vendor(&root, &sources, "jeol")?,
        jeol_1d.summary()
    );

    let raw_1d = io::load_discovered_spectra_1d_by_source_data_kind(
        &root,
        &sources,
        LoadedSourceDataKind::Raw,
    )?;
    assert_eq!(
        io::load_discovered_spectra_1d_summary_by_data_kind(
            &root,
            &sources,
            LoadedSourceDataKind::Raw,
        )?,
        raw_1d.summary()
    );

    let bruker_2d = io::load_discovered_spectra_2d_by_source_format(
        &root,
        &sources,
        LoadedSourceFormat::BrukerSer,
    )?;
    assert_eq!(
        io::load_discovered_spectra_2d_summary_by_format(&root, &sources, "bruker ser")?,
        bruker_2d.summary()
    );
    assert_eq!(
        io::load_discovered_spectra_2d_summary_by_path_prefix(&root, &sources, "bruker_cosy_raw",)?,
        bruker_2d.summary()
    );

    let jeol_2d = io::load_discovered_spectra_2d_by_source_vendor(&root, &sources, "jeol")?;
    assert_eq!(
        io::load_discovered_spectra_2d_summary_by_vendor(&root, &sources, "jeol")?,
        jeol_2d.summary()
    );

    let raw_2d = io::load_discovered_spectra_2d_by_source_data_kind(
        &root,
        &sources,
        LoadedSourceDataKind::Raw,
    )?;
    assert_eq!(
        RSpinReader::new().read_discovered_bundle_2d_summary_by_data_kind(
            &root,
            &sources,
            LoadedSourceDataKind::Raw,
        )?,
        raw_2d.summary()
    );
    assert_eq!(
        RSpinReader::new()
            .strict()
            .read_discovered_bundle_2d_summary_by_format(&root, &sources, "bruker ser")?,
        bruker_2d.summary()
    );

    Ok(())
}

fn cc0_myrcene_fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/nmrxiv/cc0/myrcene")
}

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/zenodo_7100132")
}
