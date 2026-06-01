//! Tests for source-path prefix set loader helpers.

use std::path::{Path, PathBuf};

use rspin_core::{Nucleus, Unit};
use rspin_io::{
    LoadedSourceFormat, SpectrumBundle, load_spectra_by_source_path_prefixes,
    load_spectra_by_source_path_prefixes_relative_to, load_spectra_many_by_source_path_prefixes,
    load_spectra_many_by_source_path_prefixes_relative_to,
};

#[test]
fn load_spectra_by_source_path_prefixes_selects_multiple_directory_groups() -> anyhow::Result<()> {
    let bundle = load_spectra_by_source_path_prefixes(nmrxiv_fixture_root(), ["jcamp", "jeol"])?;

    assert_eq!(bundle.len(), 5);
    assert_eq!(bundle.len_1d(), 4);
    assert_eq!(bundle.len_2d(), 1);
    assert_eq!(bundle.source_format_count(LoadedSourceFormat::JcampDx), 2);
    assert_eq!(bundle.source_format_count(LoadedSourceFormat::JeolJdf), 3);
    assert!(!has_source_path(&bundle, Path::new("bruker_1h_raw")));
    assert!(has_source_path(
        &bundle,
        Path::new("jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx")
    ));
    assert!(has_source_path(
        &bundle,
        Path::new("jeol/myrcene_hsqc_400mhz.jdf")
    ));
    Ok(())
}

#[test]
fn relative_source_path_prefixes_are_matched_after_base_anchoring() -> anyhow::Result<()> {
    let base = nmrxiv_cc0_root();
    let bundle = load_spectra_by_source_path_prefixes_relative_to(
        &base,
        "myrcene",
        ["myrcene/jcamp", "myrcene/jeol/myrcene_hsqc_400mhz.jdf"],
    )?;

    assert_eq!(bundle.len(), 3);
    assert_eq!(bundle.len_1d(), 2);
    assert_eq!(bundle.len_2d(), 1);
    assert!(has_source_path(
        &bundle,
        Path::new("myrcene/jcamp/myrcene_1h_400mhz_jcamp_dx_6_link.jdx")
    ));
    assert!(has_source_path(
        &bundle,
        Path::new("myrcene/jeol/myrcene_hsqc_400mhz.jdf")
    ));
    Ok(())
}

#[test]
fn many_source_path_prefixes_work_for_selected_inputs() -> anyhow::Result<()> {
    let root = nmrxiv_fixture_root();
    let bundle = load_spectra_many_by_source_path_prefixes(
        [root.join("jcamp"), root.join("jeol")],
        [
            "myrcene_13c_400mhz_jcamp_dx_6_link.jdx",
            "myrcene_hsqc_400mhz.jdf",
        ],
    )?;

    assert_eq!(bundle.len(), 2);
    assert_eq!(bundle.len_1d(), 1);
    assert_eq!(bundle.len_2d(), 1);
    let carbon = bundle.only_1d_by_source_path_prefix("myrcene_13c_400mhz_jcamp_dx_6_link.jdx")?;
    assert_eq!(carbon.metadata.nucleus, Some(Nucleus::Carbon13));
    let hsqc = bundle.only_2d_by_source_path_prefix("myrcene_hsqc_400mhz.jdf")?;
    assert_eq!(hsqc.shape(), (1024, 32));
    Ok(())
}

#[test]
fn many_relative_source_path_prefixes_share_one_base() -> anyhow::Result<()> {
    let base = fixture_root();
    let bundle = load_spectra_many_by_source_path_prefixes_relative_to(
        &base,
        ["varian_1h", "bruker_without_expno"],
        ["varian_1h", "bruker_without_expno/pdata"],
    )?;

    assert_eq!(bundle.len(), 2);
    assert_eq!(bundle.len_1d(), 2);
    assert_eq!(
        bundle.source_format_count(LoadedSourceFormat::AgilentFid),
        1
    );
    assert_eq!(
        bundle.source_format_count(LoadedSourceFormat::BrukerProcessed),
        1
    );
    assert!(has_source_path(&bundle, Path::new("varian_1h")));
    assert!(has_source_path(
        &bundle,
        Path::new("bruker_without_expno/pdata/1")
    ));
    assert_eq!(
        bundle.only_1d_by_source_path("varian_1h")?.x.unit,
        Unit::Seconds
    );
    assert_eq!(
        bundle
            .only_1d_by_source_path_prefix("bruker_without_expno/pdata")?
            .x
            .unit,
        Unit::Ppm
    );
    Ok(())
}

fn nmrxiv_fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/nmrxiv/cc0/myrcene")
}

fn nmrxiv_cc0_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/nmrxiv/cc0")
}

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/zenodo_7100132")
}

fn has_source_path(bundle: &SpectrumBundle, path: &Path) -> bool {
    bundle
        .spectra()
        .iter()
        .any(|loaded| loaded.source().path.as_deref() == Some(path))
}
