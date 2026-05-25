//! Integration tests for the unified spectrum bundle loader.

use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use rspin_core::{Nucleus, RSpinError, Unit};
use rspin_io::{
    LoadedSpectrum, RSpinReader, SpectrumBundle, SpectrumBundleLoader, SpectrumPathReader,
    load_spectra, load_spectra_many, load_spectrum_1d, load_spectrum_2d,
    write_spectrum_bundle_json, write_spectrum1d_json, write_spectrum2d_json,
};

#[test]
fn loads_varian_agilent_1h_directory_as_bundle() -> anyhow::Result<()> {
    let bundle = load_spectra(fixture_root().join("varian_1h"))?;
    assert_eq!(bundle.len(), 1);
    assert!(bundle.warnings().is_empty());

    let spectrum = first_1d(&bundle)?;
    assert_eq!(spectrum.len(), 16_384);
    assert_eq!(spectrum.x.unit, Unit::Seconds);
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(spectrum.metadata.solvent.as_deref(), Some("cdcl3"));
    assert_close(spectrum.metadata.frequency_mhz, Some(599.793_175_8));
    assert_close(spectrum.metadata.temperature_k, Some(298.15));
    assert!(spectrum.imaginary.is_some());
    assert!(spectrum.intensities.iter().any(|value| value.abs() > 1.0));

    let loaded = bundle
        .spectra()
        .first()
        .ok_or_else(|| anyhow::anyhow!("missing loaded spectrum"))?;
    assert_eq!(loaded.source().format, "agilent_fid");
    assert_source_path(loaded, Path::new("varian_1h"));
    Ok(())
}

#[test]
fn loads_bruker_directory_without_experiment_number() -> anyhow::Result<()> {
    let bundle = RSpinReader::new().read_path(fixture_root().join("bruker_without_expno"))?;
    assert_eq!(bundle.len(), 2);
    assert_eq!(bundle.spectra_2d().count(), 0);
    assert!(bundle.warnings().is_empty());

    let one_d = bundle.spectra_1d().collect::<Vec<_>>();
    let raw = one_d
        .iter()
        .copied()
        .find(|spectrum| spectrum.x.unit == Unit::Seconds)
        .ok_or_else(|| anyhow::anyhow!("missing raw Bruker FID"))?;
    let processed = one_d
        .iter()
        .copied()
        .find(|spectrum| spectrum.x.unit == Unit::Ppm)
        .ok_or_else(|| anyhow::anyhow!("missing processed Bruker spectrum"))?;

    assert_eq!(raw.len(), 32_768);
    assert_eq!(processed.len(), 32_768);
    assert_eq!(raw.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(processed.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert!(raw.imaginary.is_some());
    assert!(processed.imaginary.is_some());
    assert!(raw.intensities.iter().any(|value| value.abs() > 1_000.0));
    assert!(
        processed
            .intensities
            .iter()
            .any(|value| value.abs() > 1_000.0)
    );

    assert!(
        bundle.spectra().iter().any(
            |loaded| loaded.source().path.as_deref() == Some(Path::new("bruker_without_expno"))
        )
    );
    assert!(
        bundle
            .spectra()
            .iter()
            .any(|loaded| loaded.source().path.as_deref() == Some(Path::new("pdata/1")))
    );
    Ok(())
}

#[test]
fn loader_records_warnings_for_bad_candidates() -> anyhow::Result<()> {
    let bundle = SpectrumBundleLoader::new()
        .with_source_paths(true)
        .read_path(fixture_root())?;

    assert_eq!(bundle.len(), 3);
    assert_eq!(bundle.warnings().len(), 1);
    let warning = bundle
        .warnings()
        .first()
        .ok_or_else(|| anyhow::anyhow!("missing loader warning"))?;
    assert_eq!(
        warning.path.as_deref(),
        Some(Path::new("empty_jcamp/empty.jdx"))
    );
    assert!(warning.message.contains("missing XYDATA values"));
    Ok(())
}

#[test]
fn strict_loader_fails_on_bad_candidate() -> anyhow::Result<()> {
    let Err(error) = RSpinReader::new()
        .with_strict(true)
        .read_path(fixture_root())
    else {
        anyhow::bail!("strict loader should fail on empty JCAMP-DX candidate");
    };

    assert!(matches!(error, RSpinError::Parse { .. }));
    Ok(())
}

#[test]
fn loader_can_disable_raw_or_processed_candidates() -> anyhow::Result<()> {
    let fixture = fixture_root().join("bruker_without_expno");

    let raw_only = RSpinReader::new()
        .with_processed(false)
        .read_path(&fixture)?;
    assert_eq!(raw_only.len(), 1);
    assert_eq!(first_1d(&raw_only)?.x.unit, Unit::Seconds);

    let processed_only = RSpinReader::new().with_raw(false).read_path(&fixture)?;
    assert_eq!(processed_only.len(), 1);
    assert_eq!(first_1d(&processed_only)?.x.unit, Unit::Ppm);
    Ok(())
}

#[test]
fn loader_can_filter_spectrum_dimensions() -> anyhow::Result<()> {
    let mixed = nmrxiv_fixture_root();

    let one_d_only = RSpinReader::new().with_2d(false).read_path(&mixed)?;
    assert_eq!(one_d_only.spectra_1d().count(), 3);
    assert_eq!(one_d_only.spectra_2d().count(), 0);
    assert!(one_d_only.spectra().iter().all(LoadedSpectrum::is_1d));

    let two_d_only = RSpinReader::new().with_1d(false).read_path(&mixed)?;
    assert_eq!(two_d_only.spectra_1d().count(), 0);
    assert_eq!(two_d_only.spectra_2d().count(), 2);
    assert!(two_d_only.spectra().iter().all(LoadedSpectrum::is_2d));

    let disabled_dimensions = RSpinReader::new()
        .with_1d(false)
        .with_2d(false)
        .read_path(fixture_root().join("varian_1h"));
    let Err(error) = disabled_dimensions else {
        anyhow::bail!("disabled spectrum dimensions should leave no readable spectra");
    };
    assert!(error.to_string().contains("no readable bundle data found"));
    Ok(())
}

#[test]
fn loader_toggles_apply_to_direct_vendor_files() -> anyhow::Result<()> {
    let raw_file = fixture_root().join("bruker_without_expno/fid");
    let processed_file = fixture_root().join("bruker_without_expno/pdata/1/1r");

    let raw_disabled = RSpinReader::new()
        .with_raw(false)
        .read_paths([&raw_file, &processed_file])?;

    assert_eq!(raw_disabled.len(), 1);
    assert_eq!(first_1d(&raw_disabled)?.x.unit, Unit::Ppm);
    assert!(has_source_path(&raw_disabled, Path::new("1r")));
    let raw_warning = raw_disabled
        .warnings()
        .first()
        .ok_or_else(|| anyhow::anyhow!("missing disabled raw warning"))?;
    assert_eq!(raw_warning.path.as_deref(), Some(Path::new("fid")));
    assert!(
        raw_warning
            .message
            .contains("raw spectrum candidates are disabled")
    );

    let processed_disabled = RSpinReader::new()
        .with_processed(false)
        .read_paths([&processed_file, &raw_file])?;

    assert_eq!(processed_disabled.len(), 1);
    assert_eq!(first_1d(&processed_disabled)?.x.unit, Unit::Seconds);
    assert!(has_source_path(&processed_disabled, Path::new("fid")));
    let processed_warning = processed_disabled
        .warnings()
        .first()
        .ok_or_else(|| anyhow::anyhow!("missing disabled processed warning"))?;
    assert_eq!(processed_warning.path.as_deref(), Some(Path::new("1r")));
    assert!(
        processed_warning
            .message
            .contains("processed spectrum candidates are disabled")
    );
    Ok(())
}

#[test]
fn direct_vendor_files_record_detected_source_format() -> anyhow::Result<()> {
    let bruker_raw =
        RSpinReader::new().read_path(fixture_root().join("bruker_without_expno/fid"))?;
    assert_eq!(
        loaded_source_format(&bruker_raw, Path::new("fid"))?,
        "bruker_fid"
    );

    let bruker_processed =
        RSpinReader::new().read_path(fixture_root().join("bruker_without_expno/pdata/1/1r"))?;
    assert_eq!(
        loaded_source_format(&bruker_processed, Path::new("1r"))?,
        "bruker_processed"
    );

    let agilent_raw = RSpinReader::new().read_path(fixture_root().join("varian_1h/fid"))?;
    assert_eq!(
        loaded_source_format(&agilent_raw, Path::new("fid"))?,
        "agilent_fid"
    );

    let bruker_ser =
        RSpinReader::new().read_path(nmrxiv_fixture_root().join("bruker_cosy_raw/ser"))?;
    assert_eq!(
        loaded_source_format(&bruker_ser, Path::new("ser"))?,
        "bruker_ser"
    );
    assert_eq!(first_2d(&bruker_ser)?.shape(), (2048, 512));
    Ok(())
}

#[test]
fn direct_file_dimension_toggles_report_disabled_dimension() -> anyhow::Result<()> {
    let one_d_disabled = RSpinReader::new()
        .with_1d(false)
        .read_path(fixture_root().join("bruker_without_expno/fid"));
    let Err(error) = one_d_disabled else {
        anyhow::bail!("direct one-dimensional file should not load when 1D is disabled");
    };
    assert_no_data_warning(&error, "one-dimensional spectrum candidates are disabled");

    let two_d_disabled = RSpinReader::new()
        .with_2d(false)
        .read_path(nmrxiv_fixture_root().join("bruker_cosy_raw/ser"));
    let Err(error) = two_d_disabled else {
        anyhow::bail!("direct two-dimensional file should not load when 2D is disabled");
    };
    assert_no_data_warning(&error, "two-dimensional spectrum candidates are disabled");
    Ok(())
}

#[test]
fn json_spectrum_dimension_toggles_report_disabled_dimension() -> anyhow::Result<()> {
    let root = temp_dir("json-disabled-dimensions")?;

    let one_d_bundle = load_spectra(fixture_root().join("varian_1h"))?;
    let one_d_json = root.join("one.json");
    fs::write(
        &one_d_json,
        write_spectrum1d_json(first_1d(&one_d_bundle)?)?,
    )?;

    let one_d_disabled = RSpinReader::new().with_1d(false).read_path(&one_d_json);
    let Err(error) = one_d_disabled else {
        anyhow::bail!("direct one-dimensional JSON should not load when 1D is disabled");
    };
    assert_no_data_warning(&error, "one-dimensional spectrum candidates are disabled");

    let two_d_bundle = load_spectra(nmrxiv_fixture_root().join("bruker_cosy_raw"))?;
    let two_d_json = root.join("two.json");
    fs::write(
        &two_d_json,
        write_spectrum2d_json(first_2d(&two_d_bundle)?)?,
    )?;

    let two_d_disabled = RSpinReader::new().with_2d(false).read_path(&two_d_json);
    let Err(error) = two_d_disabled else {
        anyhow::bail!("direct two-dimensional JSON should not load when 2D is disabled");
    };
    assert_no_data_warning(&error, "two-dimensional spectrum candidates are disabled");

    remove_dir(root)?;
    Ok(())
}

#[test]
fn selected_vendor_directories_report_disabled_dimensions() -> anyhow::Result<()> {
    let one_d_disabled = RSpinReader::new()
        .with_1d(false)
        .read_path(fixture_root().join("varian_1h"));
    let Err(error) = one_d_disabled else {
        anyhow::bail!(
            "selected one-dimensional vendor directory should not load when 1D is disabled"
        );
    };
    assert_no_data_warning(&error, "one-dimensional spectrum candidates are disabled");

    let two_d_disabled = RSpinReader::new()
        .with_2d(false)
        .read_path(nmrxiv_fixture_root().join("bruker_cosy_raw"));
    let Err(error) = two_d_disabled else {
        anyhow::bail!(
            "selected two-dimensional vendor directory should not load when 2D is disabled"
        );
    };
    assert_no_data_warning(&error, "two-dimensional spectrum candidates are disabled");
    Ok(())
}

#[test]
fn selected_vendor_directories_report_disabled_raw_or_processed() {
    let raw_disabled = RSpinReader::new()
        .with_raw(false)
        .read_path(fixture_root().join("varian_1h"))
        .expect_err("selected raw vendor directory should not load when raw is disabled");
    assert_no_data_warning(&raw_disabled, "raw spectrum candidates are disabled");

    let processed_disabled = RSpinReader::new()
        .with_processed(false)
        .read_path(fixture_root().join("bruker_without_expno/pdata/1"))
        .expect_err(
            "selected processed vendor directory should not load when processed is disabled",
        );
    assert_no_data_warning(
        &processed_disabled,
        "processed spectrum candidates are disabled",
    );
}

#[test]
fn loads_multiple_selected_paths_as_one_bundle() -> anyhow::Result<()> {
    let bundle = load_spectra_many([
        fixture_root().join("varian_1h"),
        fixture_root().join("bruker_without_expno"),
    ])?;

    assert_eq!(bundle.len(), 3);
    assert_eq!(bundle.spectra_1d().count(), 3);
    assert!(bundle.warnings().is_empty());
    assert!(bundle.spectra_2d().next().is_none());
    assert!(has_source_path(&bundle, Path::new("varian_1h")));
    assert!(has_source_path(&bundle, Path::new("bruker_without_expno")));
    assert!(has_source_path(&bundle, Path::new("pdata/1")));
    Ok(())
}

#[test]
fn directory_loader_anchors_nested_bundle_json_sources() -> anyhow::Result<()> {
    let source_bundle = load_spectra(fixture_root().join("varian_1h"))?;
    let root = temp_dir("nested-bundle")?;
    let nested = root.join("nested");
    fs::create_dir_all(&nested)?;
    fs::write(
        nested.join("bundle.json"),
        write_spectrum_bundle_json(&source_bundle)?,
    )?;

    let loaded = RSpinReader::new().read_path(&root)?;
    assert_eq!(loaded.len(), 1);
    let loaded_spectrum = loaded
        .spectra()
        .first()
        .ok_or_else(|| anyhow::anyhow!("missing loaded bundle spectrum"))?;
    assert_source_path(loaded_spectrum, Path::new("nested/bundle.json/varian_1h"));

    let no_sources = RSpinReader::new()
        .with_source_paths(false)
        .read_path(&root)?;
    assert!(
        no_sources
            .spectra()
            .iter()
            .all(|loaded| loaded.source().path.is_none())
    );

    remove_dir(root)?;
    Ok(())
}

#[test]
fn loader_dimension_filters_apply_to_nested_bundle_json() -> anyhow::Result<()> {
    let source_bundle = load_spectra(nmrxiv_fixture_root())?;
    let root = temp_dir("dimension-filter-bundle")?;
    fs::write(
        root.join("bundle.json"),
        write_spectrum_bundle_json(&source_bundle)?,
    )?;

    let loaded = RSpinReader::new().with_1d(false).read_path(&root)?;
    assert_eq!(loaded.spectra_1d().count(), 0);
    assert_eq!(loaded.spectra_2d().count(), 2);
    assert!(loaded.spectra().iter().all(LoadedSpectrum::is_2d));
    assert!(loaded.spectra().iter().all(|entry| {
        entry
            .source()
            .path
            .as_deref()
            .is_some_and(|path| path.starts_with(Path::new("bundle.json")))
    }));

    remove_dir(root)?;
    Ok(())
}

#[test]
fn bundle_loader_implements_path_reader_trait() -> anyhow::Result<()> {
    fn read_with_trait<R>(reader: &R, path: &Path) -> rspin_core::Result<SpectrumBundle>
    where
        R: SpectrumPathReader<Output = SpectrumBundle>,
    {
        reader.read_path(path)
    }

    let fixture = fixture_root().join("bruker_without_expno");
    let bundle = read_with_trait(&RSpinReader::new().with_raw(false), &fixture)?;

    assert_eq!(bundle.len(), 1);
    assert_eq!(first_1d(&bundle)?.x.unit, Unit::Ppm);
    assert!(has_source_path(&bundle, Path::new("pdata/1")));
    Ok(())
}

#[test]
fn multi_path_loader_records_bad_selected_paths_in_non_strict_mode() -> anyhow::Result<()> {
    let bundle = RSpinReader::new().read_paths([
        fixture_root().join("varian_1h"),
        fixture_root().join("empty_jcamp/empty.jdx"),
    ])?;

    assert_eq!(bundle.len(), 1);
    assert_eq!(bundle.warnings().len(), 1);
    let warning = bundle
        .warnings()
        .first()
        .ok_or_else(|| anyhow::anyhow!("missing warning for bad selected path"))?;
    assert_eq!(warning.path.as_deref(), Some(Path::new("empty.jdx")));
    assert!(warning.message.contains("missing XYDATA values"));
    Ok(())
}

#[test]
fn multi_path_loader_rejects_bad_selected_paths_in_strict_mode() -> anyhow::Result<()> {
    let result = RSpinReader::new().with_strict(true).read_paths([
        fixture_root().join("varian_1h"),
        fixture_root().join("empty_jcamp/empty.jdx"),
    ]);

    let Err(error) = result else {
        anyhow::bail!("strict multi-path loader should reject bad selected path");
    };
    assert!(error.to_string().contains("missing XYDATA values"));
    Ok(())
}

#[test]
fn multi_path_loader_rejects_empty_input() {
    let empty: Vec<PathBuf> = Vec::new();
    let result = RSpinReader::new().read_paths(empty);
    assert!(matches!(result, Err(RSpinError::Parse { .. })));
}

#[test]
fn loader_no_data_errors_include_first_warning() {
    let single_error = RSpinReader::new()
        .read_path(fixture_root().join("empty_jcamp/empty.jdx"))
        .expect_err("empty JCAMP-DX path should fail");
    assert_no_data_warning(&single_error, "missing XYDATA values");

    let many_error = RSpinReader::new()
        .read_paths([fixture_root().join("empty_jcamp/empty.jdx")])
        .expect_err("unreadable selected paths should fail");
    assert_no_data_warning(&many_error, "missing XYDATA values");

    let disabled_error = RSpinReader::new()
        .with_raw(false)
        .read_path(fixture_root().join("bruker_without_expno/fid"))
        .expect_err("disabled direct raw file should fail");
    assert_no_data_warning(&disabled_error, "raw spectrum candidates are disabled");
}

#[test]
fn exact_single_helpers_return_owned_and_borrowed_spectra() -> anyhow::Result<()> {
    let fixture = fixture_root().join("varian_1h");

    let direct = load_spectrum_1d(&fixture)?;
    assert_eq!(direct.len(), 16_384);

    let via_reader = RSpinReader::new().read_1d(&fixture)?;
    assert_eq!(via_reader.len(), direct.len());

    let bundle = load_spectra(&fixture)?;
    assert_eq!(bundle.only_1d()?.len(), direct.len());
    let loaded = bundle.loaded_1d().collect::<Vec<_>>();
    assert_eq!(loaded.len(), 1);
    let (_, source) = loaded
        .first()
        .ok_or_else(|| anyhow::anyhow!("missing loaded source"))?;
    assert_eq!(source.format, "agilent_fid");
    assert_eq!(source.path.as_deref(), Some(Path::new("varian_1h")));

    let (mut entries, molecules, warnings) = bundle.into_parts();
    assert!(molecules.is_empty());
    assert!(warnings.is_empty());
    let entry = entries
        .pop()
        .ok_or_else(|| anyhow::anyhow!("missing consumed spectrum entry"))?;
    let owned = entry
        .into_1d()
        .ok_or_else(|| anyhow::anyhow!("expected one-dimensional entry"))?;
    assert_eq!(owned.len(), direct.len());

    let owned_from_bundle = load_spectra(&fixture)?.into_only_1d()?;
    assert_eq!(owned_from_bundle.len(), direct.len());
    Ok(())
}

#[test]
fn exact_single_helpers_reject_wrong_or_ambiguous_dimensions() -> anyhow::Result<()> {
    let one_d_fixture = fixture_root().join("varian_1h");
    let multi_fixture = fixture_root().join("bruker_without_expno");

    let wrong_dimension = load_spectrum_2d(&one_d_fixture);
    assert_single_error(
        wrong_dimension,
        "expected exactly one two-dimensional spectrum",
        "found 1 one-dimensional and 0 two-dimensional spectra",
    )?;

    let ambiguous = RSpinReader::new().read_1d(&multi_fixture);
    assert_single_error(
        ambiguous,
        "expected exactly one one-dimensional spectrum",
        "found 2 one-dimensional and 0 two-dimensional spectra",
    )?;

    let bundle = load_spectra(&multi_fixture)?;
    assert_single_error(
        bundle.only_1d().map(rspin_core::Spectrum1D::len),
        "expected exactly one one-dimensional spectrum",
        "found 2 one-dimensional and 0 two-dimensional spectra",
    )?;
    Ok(())
}

#[test]
fn loads_nmredata_file_as_bundle_molecule_metadata() -> anyhow::Result<()> {
    let fixture = nmredata_fixture_root().join("ethanol.sdf");

    let bundle = load_spectra(&fixture)?;
    assert_eq!(bundle.len(), 0);
    assert!(!bundle.is_empty());
    assert!(bundle.warnings().is_empty());
    assert_eq!(bundle.molecules().len(), 1);

    let molecule = bundle
        .molecules()
        .first()
        .ok_or_else(|| anyhow::anyhow!("missing NMReDATA molecule"))?;
    assert_eq!(molecule.id, "nmredata:ethanol.sdf:1");
    assert_eq!(molecule.formula.as_deref(), Some("C2H6O"));
    assert_eq!(molecule.atoms.len(), 9);
    assert_eq!(molecule.atoms[0].id, "C1");
    assert_eq!(molecule.atoms[8].id, "O1");
    Ok(())
}

#[test]
fn scans_nmredata_directory_without_requiring_spectra() -> anyhow::Result<()> {
    let bundle = RSpinReader::new().read_path(nmredata_fixture_root())?;

    assert_eq!(bundle.len(), 0);
    assert!(!bundle.is_empty());
    assert_eq!(bundle.molecules().len(), 1);
    assert_eq!(
        bundle.molecules()[0].id,
        "nmredata:ethanol.sdf:1",
        "directory scans should use relative source paths in stable molecule ids"
    );
    Ok(())
}

#[test]
fn loads_nmrxiv_cc0_mixed_vendor_directory_as_bundle() -> anyhow::Result<()> {
    let bundle = load_spectra(nmrxiv_fixture_root())?;

    assert_eq!(bundle.len(), 6);
    assert_eq!(bundle.spectra_1d().count(), 4);
    assert_eq!(bundle.spectra_2d().count(), 2);
    assert!(bundle.warnings().is_empty());

    let bruker_1h = loaded_1d_by_path(&bundle, Path::new("bruker_1h_raw"))?;
    assert_eq!(bruker_1h.len(), 108_399);
    assert_eq!(bruker_1h.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(bruker_1h.metadata.solvent.as_deref(), Some("CDCl3"));
    assert_eq!(
        loaded_source_format(&bundle, Path::new("bruker_1h_raw"))?,
        "bruker_fid"
    );

    let jeol_1h = loaded_1d_by_path(&bundle, Path::new("jeol/myrcene_1h_400mhz.jdf"))?;
    assert_eq!(jeol_1h.len(), 65_536);
    assert_eq!(jeol_1h.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(jeol_1h.metadata.origin.as_deref(), Some("JEOL"));
    assert_eq!(
        loaded_source_format(&bundle, Path::new("jeol/myrcene_1h_400mhz.jdf"))?,
        "jeol_jdf"
    );

    let jeol_13c = loaded_1d_by_path(&bundle, Path::new("jeol/myrcene_13c_400mhz.jdf"))?;
    assert_eq!(jeol_13c.metadata.nucleus, Some(Nucleus::Carbon13));

    let jcamp_1h = loaded_1d_by_path(
        &bundle,
        Path::new("jcamp/myrcene_1h_400mhz_jcamp_dx_6_link.jdx"),
    )?;
    assert_eq!(jcamp_1h.len(), 104_858);
    assert_eq!(jcamp_1h.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(
        loaded_source_format(
            &bundle,
            Path::new("jcamp/myrcene_1h_400mhz_jcamp_dx_6_link.jdx")
        )?,
        "jcamp_dx"
    );

    let bruker_cosy = loaded_2d_by_path(&bundle, Path::new("bruker_cosy_raw"))?;
    assert_eq!(bruker_cosy.shape(), (2048, 512));
    assert_eq!(bruker_cosy.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(
        loaded_source_format(&bundle, Path::new("bruker_cosy_raw"))?,
        "bruker_ser"
    );

    let jeol_hsqc = loaded_2d_by_path(&bundle, Path::new("jeol/myrcene_hsqc_400mhz.jdf"))?;
    assert_eq!(jeol_hsqc.shape(), (1024, 32));
    assert_eq!(jeol_hsqc.metadata.origin.as_deref(), Some("JEOL"));
    Ok(())
}

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/zenodo_7100132")
}

fn nmredata_fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/bundle_nmredata")
}

fn nmrxiv_fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/nmrxiv/cc0/myrcene")
}

fn temp_dir(name: &str) -> anyhow::Result<PathBuf> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let mut path = std::env::temp_dir();
    path.push(format!(
        "rspin-bundle-{name}-{}-{nanos}",
        std::process::id()
    ));
    fs::create_dir_all(&path)?;
    Ok(path)
}

fn remove_dir(path: PathBuf) -> anyhow::Result<()> {
    fs::remove_dir_all(path)?;
    Ok(())
}

fn first_1d(bundle: &rspin_io::SpectrumBundle) -> anyhow::Result<&rspin_core::Spectrum1D> {
    bundle
        .spectra_1d()
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing one-dimensional spectrum"))
}

fn first_2d(bundle: &rspin_io::SpectrumBundle) -> anyhow::Result<&rspin_core::Spectrum2D> {
    bundle
        .spectra_2d()
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing two-dimensional spectrum"))
}

fn assert_source_path(loaded: &LoadedSpectrum, expected: &Path) {
    assert_eq!(loaded.source().path.as_deref(), Some(expected));
}

fn has_source_path(bundle: &SpectrumBundle, path: &Path) -> bool {
    bundle
        .spectra()
        .iter()
        .any(|loaded| loaded.source().path.as_deref() == Some(path))
}

fn loaded_1d_by_path<'a>(
    bundle: &'a SpectrumBundle,
    path: &Path,
) -> anyhow::Result<&'a rspin_core::Spectrum1D> {
    bundle
        .loaded_1d()
        .find(|(_, source)| source.path.as_deref() == Some(path))
        .map(|(spectrum, _)| spectrum)
        .ok_or_else(|| anyhow::anyhow!("missing one-dimensional spectrum at {}", path.display()))
}

fn loaded_2d_by_path<'a>(
    bundle: &'a SpectrumBundle,
    path: &Path,
) -> anyhow::Result<&'a rspin_core::Spectrum2D> {
    bundle
        .loaded_2d()
        .find(|(_, source)| source.path.as_deref() == Some(path))
        .map(|(spectrum, _)| spectrum)
        .ok_or_else(|| anyhow::anyhow!("missing two-dimensional spectrum at {}", path.display()))
}

fn loaded_source_format<'a>(bundle: &'a SpectrumBundle, path: &Path) -> anyhow::Result<&'a str> {
    bundle
        .spectra()
        .iter()
        .find(|loaded| loaded.source().path.as_deref() == Some(path))
        .map(|loaded| loaded.source().format.as_str())
        .ok_or_else(|| anyhow::anyhow!("missing loaded source at {}", path.display()))
}

fn assert_close(actual: Option<f64>, expected: Option<f64>) {
    match (actual, expected) {
        (Some(actual), Some(expected)) => {
            assert!(
                (actual - expected).abs() <= 1.0e-9,
                "expected {expected}, got {actual}"
            );
        }
        _ => assert_eq!(actual, expected),
    }
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

fn assert_no_data_warning(error: &RSpinError, expected_warning: &str) {
    let message = error.to_string();
    assert!(
        message.contains("no readable bundle data found"),
        "expected no-data message in {message:?}"
    );
    assert!(
        message.contains("first warning"),
        "expected first warning context in {message:?}"
    );
    assert!(
        message.contains(expected_warning),
        "expected warning {expected_warning:?} in {message:?}"
    );
}
