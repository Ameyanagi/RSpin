//! Tests for discovered-source metadata filter loading.

use std::path::PathBuf;

use anyhow::Result;
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

    assert_eq!(
        load_discovered_spectra_strict_by_source_format(&root, &sources, "agilent fid")?,
        fid
    );
    assert_eq!(
        load_discovered_spectra_strict_by_source_vendor(
            &root,
            &sources,
            LoadedSourceVendor::AgilentVarian,
        )?,
        varian
    );
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

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/zenodo_7100132")
}
