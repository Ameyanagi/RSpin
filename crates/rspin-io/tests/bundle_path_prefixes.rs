//! Tests for source-path prefix set loader helpers.

use std::path::{Path, PathBuf};

use rspin_core::{Nucleus, Unit};
use rspin_io::{
    LoadedSourceFormat, SpectrumBundle, load_spectra_by_source_path_prefixes,
    load_spectra_by_source_path_prefixes_relative_to, load_spectra_many_by_source_path_prefixes,
    load_spectra_many_by_source_path_prefixes_relative_to,
    load_spectrum_1d_by_source_path_prefixes, load_spectrum_1d_by_source_path_prefixes_relative_to,
    load_spectrum_1d_many_by_source_path_prefixes,
    load_spectrum_1d_with_source_by_source_path_prefixes, load_spectrum_2d_by_source_path_prefixes,
    load_spectrum_2d_many_with_source_by_source_path_prefixes_relative_to,
    load_spectrum_2d_with_source_by_source_path_prefixes,
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

#[test]
fn bundle_source_path_prefix_set_selectors_cover_borrowed_and_owned_flows() -> anyhow::Result<()> {
    let bundle = load_spectra_by_source_path_prefixes(nmrxiv_fixture_root(), ["jcamp", "jeol"])?;

    let prefixes = ["jcamp", "jeol/myrcene_hsqc_400mhz.jdf"];
    assert_eq!(bundle.loaded_by_source_path_prefixes(prefixes).count(), 3);
    assert_eq!(
        bundle.loaded_1d_by_source_path_prefixes(prefixes).count(),
        2
    );
    assert_eq!(
        bundle.loaded_2d_by_source_path_prefixes(prefixes).count(),
        1
    );
    assert_eq!(bundle.source_path_prefix_count_by_prefixes(prefixes), 3);
    assert!(bundle.has_any_source_path_prefix(prefixes));

    let selected_paths = bundle
        .source_paths_for_path_prefixes(prefixes)
        .map(Path::to_path_buf)
        .collect::<Vec<_>>();
    assert_eq!(selected_paths.len(), 3);
    assert!(selected_paths.contains(&PathBuf::from(
        "jcamp/myrcene_1h_400mhz_jcamp_dx_6_link.jdx"
    )));
    assert!(selected_paths.contains(&PathBuf::from(
        "jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx"
    )));
    assert!(selected_paths.contains(&PathBuf::from("jeol/myrcene_hsqc_400mhz.jdf")));

    let subset = bundle.source_path_prefix_subset_by_prefixes(prefixes);
    assert_eq!(subset.len(), 3);
    assert_eq!(subset.len_1d(), 2);
    assert_eq!(subset.len_2d(), 1);
    assert_eq!(subset.source_format_count(LoadedSourceFormat::JcampDx), 2);
    assert_eq!(subset.source_format_count(LoadedSourceFormat::JeolJdf), 1);

    let empty_subset = bundle.source_path_prefix_subset_by_prefixes(std::iter::empty::<PathBuf>());
    assert_eq!(empty_subset.len(), bundle.len());

    let owned_loaded = bundle.clone().into_loaded_by_source_path_prefixes(prefixes);
    assert_eq!(owned_loaded.len(), 3);

    let owned_one_d = bundle
        .clone()
        .into_loaded_1d_by_source_path_prefixes(prefixes);
    assert_eq!(owned_one_d.len(), 2);

    let owned_two_d = bundle
        .clone()
        .into_loaded_2d_by_source_path_prefixes(prefixes);
    assert_eq!(owned_two_d.len(), 1);

    let spectra_1d = bundle
        .clone()
        .into_spectra_1d_by_source_path_prefixes(prefixes);
    assert_eq!(spectra_1d.len(), 2);

    let spectra_2d = bundle.into_spectra_2d_by_source_path_prefixes(prefixes);
    assert_eq!(spectra_2d.len(), 1);
    Ok(())
}

#[test]
fn bundle_source_path_prefix_set_selectors_filter_warnings() -> anyhow::Result<()> {
    let bundle = rspin_io::SpectrumBundleLoader::new()
        .with_source_paths(true)
        .read_path(fixture_root())?;

    let warnings = bundle
        .warnings_for_source_path_prefixes(["empty_jcamp", "missing"])
        .collect::<Vec<_>>();
    assert_eq!(warnings.len(), 1);
    assert_eq!(
        warnings
            .first()
            .and_then(|warning| warning.path())
            .map(Path::to_path_buf),
        Some(PathBuf::from("empty_jcamp/empty.jdx"))
    );

    let all_warnings = bundle
        .warnings_for_source_path_prefixes(std::iter::empty::<PathBuf>())
        .count();
    assert_eq!(all_warnings, bundle.warning_count());

    let subset = bundle.source_path_prefix_subset_by_prefixes(["empty_jcamp"]);
    assert_eq!(subset.len(), 0);
    assert_eq!(subset.warning_count(), 1);
    Ok(())
}

#[test]
fn exact_source_path_prefixes_select_single_spectra() -> anyhow::Result<()> {
    let carbon = load_spectrum_1d_by_source_path_prefixes(
        nmrxiv_fixture_root(),
        ["jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx", "missing"],
    )?;
    assert_eq!(carbon.metadata.nucleus, Some(Nucleus::Carbon13));

    let (carbon, carbon_source) = load_spectrum_1d_with_source_by_source_path_prefixes(
        nmrxiv_fixture_root(),
        ["jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx", "missing"],
    )?;
    assert_eq!(carbon.metadata.nucleus, Some(Nucleus::Carbon13));
    assert_eq!(
        carbon_source.path(),
        Some(Path::new("jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx"))
    );

    let hsqc =
        load_spectrum_2d_by_source_path_prefixes(nmrxiv_fixture_root(), ["jeol", "missing"])?;
    assert_eq!(hsqc.shape(), (1024, 32));

    let (_, hsqc_source) = load_spectrum_2d_with_source_by_source_path_prefixes(
        nmrxiv_fixture_root(),
        ["jeol", "missing"],
    )?;
    assert_eq!(
        hsqc_source.path(),
        Some(Path::new("jeol/myrcene_hsqc_400mhz.jdf"))
    );
    Ok(())
}

#[test]
fn exact_source_path_prefixes_work_after_base_anchoring() -> anyhow::Result<()> {
    let base = nmrxiv_cc0_root();
    let carbon = load_spectrum_1d_by_source_path_prefixes_relative_to(
        &base,
        "myrcene",
        [
            "myrcene/jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx",
            "missing",
        ],
    )?;

    assert_eq!(carbon.metadata.nucleus, Some(Nucleus::Carbon13));
    Ok(())
}

#[test]
fn exact_many_source_path_prefixes_work_for_selected_inputs() -> anyhow::Result<()> {
    let root = nmrxiv_fixture_root();
    let carbon = load_spectrum_1d_many_by_source_path_prefixes(
        [root.join("jcamp"), root.join("jeol")],
        ["myrcene_13c_400mhz_jcamp_dx_6_link.jdx", "missing"],
    )?;
    assert_eq!(carbon.metadata.nucleus, Some(Nucleus::Carbon13));

    let base = nmrxiv_cc0_root();
    let (hsqc, hsqc_source) =
        load_spectrum_2d_many_with_source_by_source_path_prefixes_relative_to(
            &base,
            ["myrcene/jeol/myrcene_1h_400mhz.jdf", "myrcene"],
            ["myrcene/jeol", "missing"],
        )?;
    assert_eq!(hsqc.shape(), (1024, 32));
    assert_eq!(
        hsqc_source.path(),
        Some(Path::new("myrcene/jeol/myrcene_hsqc_400mhz.jdf"))
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
