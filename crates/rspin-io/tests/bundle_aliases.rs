//! Tests for short bundle loading aliases.

use std::path::{Path, PathBuf};

use anyhow::Result;
use rspin_io::{
    LoadedSourceVendor, load, load_many, load_many_relative_to, load_many_strict,
    load_many_strict_relative_to, load_many_summary, load_many_summary_relative_to,
    load_many_summary_strict, load_many_summary_strict_relative_to, load_relative_to, load_strict,
    load_strict_relative_to, load_summary, load_summary_relative_to, load_summary_strict,
    load_summary_strict_relative_to,
};

#[test]
fn short_single_path_load_aliases_match_common_bundle_helpers() -> Result<()> {
    let root = fixture_root();
    let varian = root.join("varian_1h");

    let bundle = load(&varian)?;
    assert_eq!(bundle.len(), 1);
    assert_eq!(
        bundle.source_vendor_count(LoadedSourceVendor::AgilentVarian),
        1
    );

    assert_eq!(load_strict(&varian)?.len(), 1);
    assert_eq!(load_relative_to(&root, "varian_1h")?.len(), 1);
    assert_eq!(load_strict_relative_to(&root, "varian_1h")?.len(), 1);
    assert_eq!(load_summary(&varian)?.spectra(), 1);
    assert_eq!(load_summary_strict(&varian)?.spectra(), 1);
    assert_eq!(load_summary_relative_to(&root, "varian_1h")?.spectra(), 1);
    assert_eq!(
        load_summary_strict_relative_to(&root, "varian_1h")?.spectra(),
        1
    );

    Ok(())
}

#[test]
fn short_many_path_load_aliases_match_common_bundle_helpers() -> Result<()> {
    let root = fixture_root();
    let paths = [
        root.join("varian_1h"),
        root.join("bruker_without_expno/pdata/1"),
    ];

    assert_eq!(load_many(&paths)?.len(), 2);
    assert_eq!(load_many_strict(&paths)?.len(), 2);
    assert_eq!(
        load_many_relative_to(&root, ["varian_1h", "bruker_without_expno/pdata/1"])?.len(),
        2
    );
    assert_eq!(
        load_many_strict_relative_to(&root, ["varian_1h", "bruker_without_expno/pdata/1"])?.len(),
        2
    );
    assert_eq!(load_many_summary(&paths)?.spectra(), 2);
    assert_eq!(load_many_summary_strict(&paths)?.spectra(), 2);
    assert_eq!(
        load_many_summary_relative_to(&root, ["varian_1h", "bruker_without_expno/pdata/1"])?
            .spectra(),
        2
    );
    assert_eq!(
        load_many_summary_strict_relative_to(&root, ["varian_1h", "bruker_without_expno/pdata/1"])?
            .spectra(),
        2
    );

    Ok(())
}

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/zenodo_7100132")
}
