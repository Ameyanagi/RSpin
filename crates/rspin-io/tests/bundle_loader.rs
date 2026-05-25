//! Integration tests for the unified spectrum bundle loader.

use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use rspin_core::{Nucleus, RSpinError, Unit};
use rspin_io::{
    LoadedSource, LoadedSourceFormat, LoadedSourceVendor, LoadedSpectrum, RSpinReader,
    SpectrumBundle, SpectrumBundleLoader, SpectrumPathReader, load_spectra, load_spectra_many,
    load_spectra_many_relative_to, load_spectra_relative_to, load_spectrum_1d,
    load_spectrum_1d_many, load_spectrum_1d_many_relative_to, load_spectrum_1d_relative_to,
    load_spectrum_2d, load_spectrum_2d_many, load_spectrum_2d_many_relative_to,
    load_spectrum_2d_relative_to, parse_loaded_source_format, parse_loaded_source_vendor,
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
fn reader_named_option_helpers_cover_common_modes() -> anyhow::Result<()> {
    let bruker = fixture_root().join("bruker_without_expno");

    let raw_only = RSpinReader::new().raw_only().read_path(&bruker)?;
    assert_eq!(raw_only.len(), 1);
    assert_eq!(first_1d(&raw_only)?.x.unit, Unit::Seconds);

    let processed_only = RSpinReader::new().processed_only().read_path(&bruker)?;
    assert_eq!(processed_only.len(), 1);
    assert_eq!(first_1d(&processed_only)?.x.unit, Unit::Ppm);

    let mixed = nmrxiv_fixture_root();
    let one_d_only = RSpinReader::new().one_d_only().read_path(&mixed)?;
    assert_eq!(one_d_only.len_1d(), 5);
    assert_eq!(one_d_only.len_2d(), 0);

    let two_d_only = RSpinReader::new().two_d_only().read_path(&mixed)?;
    assert_eq!(two_d_only.len_1d(), 0);
    assert_eq!(two_d_only.len_2d(), 2);

    let no_sources = RSpinReader::new()
        .without_source_paths()
        .read_path(&bruker)?;
    assert!(
        no_sources
            .spectra()
            .iter()
            .all(|loaded| loaded.source().path.is_none())
    );

    let strict_error = RSpinReader::new()
        .strict()
        .read_path(fixture_root().join("empty_jcamp/empty.jdx"));
    let Err(error) = strict_error else {
        anyhow::bail!("strict helper should fail on unreadable candidates");
    };
    assert!(error.to_string().contains("missing XYDATA values"));
    Ok(())
}

#[test]
fn reader_short_read_aliases_cover_common_workflows() -> anyhow::Result<()> {
    let base = fixture_root();
    let varian = base.join("varian_1h");
    let processed_bruker = base.join("bruker_without_expno/pdata/1");

    let bundle = RSpinReader::new().read(&varian)?;
    assert_eq!(bundle.len(), 1);
    assert_eq!(
        first_1d(&bundle)?.metadata.nucleus,
        Some(Nucleus::Hydrogen1)
    );
    assert!(has_source_path(&bundle, Path::new("varian_1h")));

    let relative = RSpinReader::new().read_relative_to(&base, "varian_1h")?;
    assert_eq!(relative.len(), 1);
    assert!(has_source_path(&relative, Path::new("varian_1h")));

    let many = RSpinReader::new().read_many([&varian, &processed_bruker])?;
    assert_eq!(many.len(), 2);
    assert_eq!(many.len_1d(), 2);
    assert!(many.warnings().is_empty());

    let relative_many = RSpinReader::new()
        .processed_only()
        .read_many_relative_to(&base, ["bruker_without_expno"])?;
    assert_eq!(relative_many.len(), 1);
    assert_eq!(first_1d(&relative_many)?.x.unit, Unit::Ppm);
    assert!(has_source_path(
        &relative_many,
        Path::new("bruker_without_expno/pdata/1")
    ));
    Ok(())
}

#[test]
fn loader_can_filter_spectrum_dimensions() -> anyhow::Result<()> {
    let mixed = nmrxiv_fixture_root();

    let one_d_only = RSpinReader::new().with_2d(false).read_path(&mixed)?;
    assert_eq!(one_d_only.spectra_1d().count(), 5);
    assert_eq!(one_d_only.spectra_2d().count(), 0);
    assert!(one_d_only.spectra().iter().all(LoadedSpectrum::is_1d));
    assert!(has_source_path(
        &one_d_only,
        Path::new("jcamp/myrcene_1h_400mhz_jcamp_dx_6_link.jdx")
    ));
    assert!(has_source_path(
        &one_d_only,
        Path::new("jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx")
    ));

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
fn multi_path_loader_can_anchor_sources_to_common_base() -> anyhow::Result<()> {
    let base = fixture_root();
    let bundle = load_spectra_many_relative_to(&base, ["varian_1h", "bruker_without_expno"])?;

    assert_eq!(bundle.len(), 3);
    assert_eq!(bundle.len_1d(), 3);
    assert!(bundle.warnings().is_empty());
    assert!(has_source_path(&bundle, Path::new("varian_1h")));
    assert!(has_source_path(&bundle, Path::new("bruker_without_expno")));
    assert!(has_source_path(
        &bundle,
        Path::new("bruker_without_expno/pdata/1")
    ));

    let bundle = RSpinReader::new().read_paths_relative_to(
        &base,
        [base.join("empty_jcamp/empty.jdx"), base.join("varian_1h")],
    )?;
    assert_eq!(bundle.len(), 1);
    let warning = bundle
        .warnings()
        .first()
        .ok_or_else(|| anyhow::anyhow!("missing anchored warning"))?;
    assert_eq!(
        warning.path.as_deref(),
        Some(Path::new("empty_jcamp/empty.jdx"))
    );

    let bad_base = load_spectra_many_relative_to(base.join("varian_1h/fid"), ["fid"]);
    let Err(error) = bad_base else {
        anyhow::bail!("file base should be rejected");
    };
    assert!(error.to_string().contains("is not a directory"));
    Ok(())
}

#[test]
fn single_path_relative_helpers_anchor_sources_to_common_base() -> anyhow::Result<()> {
    let base = fixture_root();

    let bundle = load_spectra_relative_to(&base, "bruker_without_expno")?;
    assert_eq!(bundle.len(), 2);
    assert!(has_source_path(&bundle, Path::new("bruker_without_expno")));
    assert!(has_source_path(
        &bundle,
        Path::new("bruker_without_expno/pdata/1")
    ));

    let processed = RSpinReader::new()
        .processed_only()
        .read_path_relative_to(&base, "bruker_without_expno")?;
    assert_eq!(processed.len(), 1);
    assert!(has_source_path(
        &processed,
        Path::new("bruker_without_expno/pdata/1")
    ));

    let one_d = load_spectrum_1d_relative_to(&base, "varian_1h")?;
    assert_eq!(one_d.len(), 16_384);
    assert_eq!(one_d.metadata.nucleus, Some(Nucleus::Hydrogen1));

    let one_d = RSpinReader::new().read_1d_relative_to(&base, "varian_1h")?;
    assert_eq!(one_d.len(), 16_384);

    let two_d = load_spectrum_2d_relative_to(nmrxiv_fixture_root(), "bruker_cosy_raw")?;
    assert_eq!(two_d.shape(), (2048, 512));

    let two_d = RSpinReader::new().read_2d_relative_to(nmrxiv_fixture_root(), "bruker_cosy_raw")?;
    assert_eq!(two_d.shape(), (2048, 512));

    let empty = RSpinReader::new().read_path_relative_to(&base, "empty_jcamp/empty.jdx");
    let Err(error) = empty else {
        anyhow::bail!("empty JCAMP-DX relative path should fail");
    };
    assert_no_data_warning(&error, "missing XYDATA values");
    assert!(error.to_string().contains("empty_jcamp/empty.jdx"));
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
fn loader_raw_processed_filters_apply_to_nested_bundle_json() -> anyhow::Result<()> {
    let source_bundle = load_spectra(fixture_root().join("bruker_without_expno"))?;
    assert_eq!(source_bundle.len(), 2);

    let root = temp_dir("raw-processed-filter-bundle")?;
    fs::write(
        root.join("bundle.json"),
        write_spectrum_bundle_json(&source_bundle)?,
    )?;

    let raw_only = RSpinReader::new().raw_only().read_path(&root)?;
    assert_eq!(raw_only.len(), 1);
    assert_eq!(first_1d(&raw_only)?.x.unit, Unit::Seconds);
    assert_eq!(
        raw_only.source_format_count(LoadedSourceFormat::BrukerFid),
        1
    );
    assert_eq!(
        raw_only.source_format_count(LoadedSourceFormat::BrukerProcessed),
        0
    );
    assert!(has_source_path(
        &raw_only,
        Path::new("bundle.json/bruker_without_expno")
    ));

    let processed_only = RSpinReader::new().processed_only().read_path(&root)?;
    assert_eq!(processed_only.len(), 1);
    assert_eq!(first_1d(&processed_only)?.x.unit, Unit::Ppm);
    assert_eq!(
        processed_only.source_format_count(LoadedSourceFormat::BrukerFid),
        0
    );
    assert_eq!(
        processed_only.source_format_count(LoadedSourceFormat::BrukerProcessed),
        1
    );
    assert!(has_source_path(
        &processed_only,
        Path::new("bundle.json/pdata/1")
    ));

    let none = RSpinReader::new()
        .with_raw(false)
        .with_processed(false)
        .read_path(&root);
    let Err(error) = none else {
        anyhow::bail!("disabling raw and processed should filter nested vendor bundle spectra");
    };
    assert!(error.to_string().contains("no readable bundle data found"));

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
fn exact_single_helpers_support_selected_path_sets() -> anyhow::Result<()> {
    let one_d = load_spectrum_1d_many([fixture_root().join("varian_1h")])?;
    assert_eq!(one_d.len(), 16_384);
    assert_eq!(one_d.metadata.nucleus, Some(Nucleus::Hydrogen1));

    let one_d = load_spectrum_1d_many_relative_to(fixture_root(), ["varian_1h"])?;
    assert_eq!(one_d.len(), 16_384);

    let one_d = RSpinReader::new().read_1d_paths([
        fixture_root().join("empty_jcamp/empty.jdx"),
        fixture_root().join("varian_1h"),
    ])?;
    assert_eq!(one_d.len(), 16_384);

    let two_d = load_spectrum_2d_many([nmrxiv_fixture_root().join("bruker_cosy_raw")])?;
    assert_eq!(two_d.shape(), (2048, 512));

    let two_d = load_spectrum_2d_many_relative_to(nmrxiv_fixture_root(), ["bruker_cosy_raw"])?;
    assert_eq!(two_d.shape(), (2048, 512));

    let two_d = RSpinReader::new()
        .raw_only()
        .read_2d_paths_relative_to(nmrxiv_fixture_root(), ["bruker_cosy_raw"])?;
    assert_eq!(two_d.shape(), (2048, 512));

    let wrong_dimension = RSpinReader::new().read_2d_paths([fixture_root().join("varian_1h")]);
    assert_single_error(
        wrong_dimension,
        "expected exactly one two-dimensional spectrum",
        "found 1 one-dimensional and 0 two-dimensional spectra",
    )?;

    let ambiguous = RSpinReader::new().read_1d_paths([
        fixture_root().join("varian_1h"),
        fixture_root().join("bruker_without_expno"),
    ]);
    assert_single_error(
        ambiguous,
        "expected exactly one one-dimensional spectrum",
        "found 3 one-dimensional and 0 two-dimensional spectra",
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
fn bundle_accessors_count_and_consume_loaded_dimensions() -> anyhow::Result<()> {
    let bundle = load_spectra(nmrxiv_fixture_root())?;

    assert_eq!(bundle.len(), 7);
    assert_eq!(bundle.len_1d(), 5);
    assert_eq!(bundle.len_2d(), 2);
    assert_eq!(bundle.molecule_count(), 0);
    assert_eq!(bundle.warning_count(), 0);
    assert!(!bundle.has_warnings());

    let loaded_1d = bundle.clone().into_loaded_1d();
    assert_eq!(loaded_1d.len(), 5);
    assert!(loaded_1d.iter().any(|(spectrum, source)| {
        spectrum.metadata.nucleus == Some(Nucleus::Carbon13)
            && source.path.as_deref()
                == Some(Path::new("jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx"))
    }));

    let spectra_1d = bundle.clone().into_spectra_1d();
    assert_eq!(spectra_1d.len(), 5);
    assert!(
        spectra_1d
            .iter()
            .any(|spectrum| spectrum.metadata.nucleus == Some(Nucleus::Carbon13))
    );

    let loaded_2d = bundle.clone().into_loaded_2d();
    assert_eq!(loaded_2d.len(), 2);
    assert!(
        loaded_2d
            .iter()
            .any(|(spectrum, _)| spectrum.shape() == (2048, 512))
    );

    let spectra_2d = bundle.into_spectra_2d();
    assert_eq!(spectra_2d.len(), 2);
    assert!(
        spectra_2d
            .iter()
            .any(|spectrum| spectrum.shape() == (1024, 32))
    );
    Ok(())
}

#[test]
fn bundle_source_path_lookup_helpers_find_entries_and_warnings() -> anyhow::Result<()> {
    let bundle = load_spectra(nmrxiv_fixture_root())?;
    let jcamp_path = Path::new("jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx");
    let hsqc_path = Path::new("jeol/myrcene_hsqc_400mhz.jdf");

    let source_paths = bundle.source_paths().collect::<Vec<_>>();
    assert_eq!(source_paths.len(), bundle.len());
    assert!(source_paths.contains(&jcamp_path));
    assert!(source_paths.contains(&hsqc_path));
    assert!(bundle.has_source_path(jcamp_path));

    let loaded_sources = bundle.loaded_sources().collect::<Vec<_>>();
    assert_eq!(loaded_sources.len(), bundle.len());
    assert!(
        loaded_sources
            .iter()
            .all(|source| !source.format().is_empty())
    );
    assert_eq!(
        loaded_sources
            .iter()
            .filter_map(|source| source.path())
            .count(),
        bundle.len()
    );

    let loaded = bundle
        .loaded_by_source_path(jcamp_path)
        .ok_or_else(|| anyhow::anyhow!("missing loaded entry at {}", jcamp_path.display()))?;
    assert!(loaded.is_1d());
    assert_eq!(loaded.source().format, "jcamp_dx");

    let (carbon, carbon_source) = bundle
        .loaded_1d_by_source_path(jcamp_path)
        .ok_or_else(|| anyhow::anyhow!("missing 1D entry at {}", jcamp_path.display()))?;
    assert_eq!(carbon.metadata.nucleus, Some(Nucleus::Carbon13));
    assert_eq!(carbon_source.format, "jcamp_dx");

    let (hsqc, hsqc_source) = bundle
        .loaded_2d_by_source_path(hsqc_path)
        .ok_or_else(|| anyhow::anyhow!("missing 2D entry at {}", hsqc_path.display()))?;
    assert_eq!(hsqc.shape(), (1024, 32));
    assert_eq!(hsqc_source.format, "jeol_jdf");

    assert!(bundle.loaded_by_source_path("missing").is_none());
    assert!(!bundle.has_source_path("missing"));
    assert!(bundle.loaded_2d_by_source_path(jcamp_path).is_none());

    let bundle_with_warning = RSpinReader::new().read_path(fixture_root())?;
    let warnings = bundle_with_warning
        .warnings_for_source_path(Path::new("empty_jcamp/empty.jdx"))
        .collect::<Vec<_>>();
    assert_eq!(warnings.len(), 1);
    let warning = warnings
        .first()
        .ok_or_else(|| anyhow::anyhow!("missing lookup warning"))?;
    assert_eq!(warning.path(), Some(Path::new("empty_jcamp/empty.jdx")));
    assert!(warning.message().contains("missing XYDATA values"));
    assert_eq!(
        bundle_with_warning.warning_paths().collect::<Vec<_>>(),
        vec![Path::new("empty_jcamp/empty.jdx")]
    );
    let warning_messages = bundle_with_warning.warning_messages().collect::<Vec<_>>();
    assert_eq!(warning_messages.len(), 1);
    assert!(warning_messages[0].contains("missing XYDATA values"));

    let no_sources = RSpinReader::new()
        .without_source_paths()
        .read_path(nmrxiv_fixture_root())?;
    assert!(no_sources.loaded_by_source_path(jcamp_path).is_none());
    assert!(no_sources.source_paths().next().is_none());
    Ok(())
}

#[test]
fn bundle_source_format_helpers_count_entries() -> anyhow::Result<()> {
    let bundle = load_spectra(nmrxiv_fixture_root())?;

    assert_eq!(bundle.source_format_count("bruker_fid"), 1);
    assert_eq!(bundle.source_format_count("bruker_ser"), 1);
    assert_eq!(bundle.source_format_count(LoadedSourceFormat::JcampDx), 2);
    assert_eq!(bundle.source_format_count("jdx"), 2);
    assert_eq!(bundle.source_format_count(LoadedSourceFormat::JeolJdf), 3);
    assert_eq!(bundle.source_format_count("jdf"), 3);
    assert_eq!(bundle.source_format_count("missing"), 0);
    assert!(bundle.has_source_format(LoadedSourceFormat::JcampDx));
    assert!(bundle.has_source_format("jcamp"));
    assert!(!bundle.has_source_format("missing"));
    assert_eq!(
        parse_loaded_source_format("jdx")?,
        LoadedSourceFormat::JcampDx
    );

    let summary = bundle.summary();
    assert_eq!(summary.spectra(), 7);
    assert_eq!(summary.spectra_1d(), 5);
    assert_eq!(summary.spectra_2d(), 2);
    assert_eq!(summary.molecules(), 0);
    assert_eq!(summary.warnings(), 0);
    assert_eq!(summary.source_format_count(LoadedSourceFormat::JcampDx), 2);
    assert_eq!(summary.source_format_count("jdx"), 2);
    assert!(summary.has_source_format(LoadedSourceFormat::JeolJdf));
    assert!(summary.has_source_format("jdf"));
    assert!(!summary.has_source_format("missing"));

    assert_eq!(
        bundle
            .source_format_counts()
            .iter()
            .map(|count| (count.format(), count.count()))
            .collect::<Vec<_>>(),
        vec![
            ("bruker_fid", 1),
            ("bruker_ser", 1),
            ("jcamp_dx", 2),
            ("jeol_jdf", 3)
        ]
    );
    Ok(())
}

#[test]
fn bundle_source_format_helpers_filter_entries() -> anyhow::Result<()> {
    let bundle = load_spectra(nmrxiv_fixture_root())?;

    let source_formats = bundle.source_formats().collect::<Vec<_>>();
    assert_eq!(source_formats.len(), bundle.len());
    assert!(source_formats.contains(&"jcamp_dx"));
    assert!(source_formats.contains(&"jeol_jdf"));

    let loaded_jcamp = bundle
        .loaded_by_source_format(LoadedSourceFormat::JcampDx)
        .collect::<Vec<_>>();
    assert_eq!(loaded_jcamp.len(), 2);
    assert_eq!(bundle.loaded_by_source_format("jdx").count(), 2);
    assert!(loaded_jcamp.iter().all(|entry| entry.is_1d()));
    assert!(bundle.loaded_by_source_format("missing").next().is_none());

    let jcamp_1d = bundle
        .loaded_1d_by_source_format(LoadedSourceFormat::JcampDx)
        .collect::<Vec<_>>();
    assert_eq!(jcamp_1d.len(), 2);
    assert!(
        jcamp_1d
            .iter()
            .any(|(spectrum, _)| spectrum.metadata.nucleus == Some(Nucleus::Carbon13))
    );
    assert!(
        jcamp_1d
            .iter()
            .all(|(_, source)| source.is_format(LoadedSourceFormat::JcampDx))
    );
    assert!(jcamp_1d.iter().all(|(_, source)| source.is_format("jdx")));
    assert!(
        jcamp_1d
            .iter()
            .all(|(_, source)| source.format_kind() == Some(LoadedSourceFormat::JcampDx))
    );
    assert_eq!(
        bundle
            .source_paths_for_format(LoadedSourceFormat::JcampDx)
            .collect::<Vec<_>>(),
        vec![
            Path::new("jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx"),
            Path::new("jcamp/myrcene_1h_400mhz_jcamp_dx_6_link.jdx")
        ]
    );
    assert_eq!(bundle.source_paths_for_format("jdx").count(), 2);
    assert_eq!(
        bundle
            .loaded_2d_by_source_format(LoadedSourceFormat::JeolJdf)
            .collect::<Vec<_>>()
            .len(),
        1
    );
    assert_eq!(bundle.loaded_2d_by_source_format("jdf").count(), 1);
    Ok(())
}

#[test]
fn bundle_source_vendor_helpers_group_entries() -> anyhow::Result<()> {
    let bundle = load_spectra(nmrxiv_fixture_root())?;

    assert_eq!(
        parse_loaded_source_vendor("bruker")?,
        LoadedSourceVendor::Bruker
    );
    assert_eq!(bundle.source_vendor_count(LoadedSourceVendor::Bruker), 2);
    assert_eq!(bundle.source_vendor_count(LoadedSourceVendor::Jeol), 3);
    assert_eq!(
        bundle.source_vendor_count(LoadedSourceVendor::AgilentVarian),
        0
    );
    assert!(bundle.has_source_vendor(LoadedSourceVendor::Jeol));
    assert!(!bundle.has_source_vendor("agilent"));

    let vendor_counts = bundle.source_vendor_counts();
    assert_eq!(vendor_counts.len(), 2);
    assert_eq!(vendor_counts[0].vendor(), "bruker");
    assert_eq!(vendor_counts[0].count(), 2);
    assert_eq!(
        vendor_counts[0].vendor_kind(),
        Some(LoadedSourceVendor::Bruker)
    );
    assert_eq!(vendor_counts[1].vendor(), "jeol");
    assert_eq!(vendor_counts[1].count(), 3);

    let summary = bundle.summary();
    assert_eq!(summary.source_vendor_count(LoadedSourceVendor::Bruker), 2);
    assert!(summary.has_source_vendor(LoadedSourceVendor::Jeol));
    assert!(!summary.has_source_vendor(LoadedSourceVendor::AgilentVarian));
    assert_eq!(summary.source_vendors, vendor_counts);
    assert_eq!(summary.source_vendor_counts(), vendor_counts);

    let vendors = bundle.source_vendors().collect::<Vec<_>>();
    assert_eq!(vendors.len(), 5);
    assert!(vendors.contains(&LoadedSourceVendor::Bruker));
    assert!(vendors.contains(&LoadedSourceVendor::Jeol));

    let bruker = bundle
        .loaded_by_source_vendor(LoadedSourceVendor::Bruker)
        .collect::<Vec<_>>();
    assert_eq!(bruker.len(), 2);
    assert!(
        bruker
            .iter()
            .all(|entry| entry.source().is_vendor(LoadedSourceVendor::Bruker))
    );
    assert_eq!(
        bundle
            .loaded_1d_by_source_vendor(LoadedSourceVendor::Jeol)
            .collect::<Vec<_>>()
            .len(),
        2
    );
    assert_eq!(
        bundle
            .loaded_2d_by_source_vendor(LoadedSourceVendor::Jeol)
            .collect::<Vec<_>>()
            .len(),
        1
    );
    assert_eq!(
        bundle
            .source_paths_for_vendor(LoadedSourceVendor::Jeol)
            .collect::<Vec<_>>(),
        vec![
            Path::new("jeol/myrcene_13c_400mhz.jdf"),
            Path::new("jeol/myrcene_1h_400mhz.jdf"),
            Path::new("jeol/myrcene_hsqc_400mhz.jdf")
        ]
    );
    assert!(
        bundle
            .loaded_by_source_vendor("jcamp")
            .collect::<Vec<_>>()
            .is_empty()
    );
    assert!(
        bundle
            .loaded_1d_by_source_vendor("csv")
            .collect::<Vec<_>>()
            .is_empty()
    );
    assert!(
        bundle
            .source_paths_for_vendor("unknown-vendor")
            .collect::<Vec<_>>()
            .is_empty()
    );
    Ok(())
}

#[test]
fn loader_can_restrict_source_formats() -> anyhow::Result<()> {
    let jcamp = RSpinReader::new()
        .only_source_format(LoadedSourceFormat::JcampDx)
        .read_path(nmrxiv_fixture_root())?;
    assert_eq!(jcamp.len(), 2);
    assert_eq!(jcamp.source_format_count(LoadedSourceFormat::JcampDx), 2);
    assert!(jcamp.warnings().is_empty());
    assert!(
        jcamp
            .spectra()
            .iter()
            .all(|entry| entry.source().is_format(LoadedSourceFormat::JcampDx))
    );

    let alias = RSpinReader::new()
        .only_source_format("jdx")
        .read_path(nmrxiv_fixture_root())?;
    assert_eq!(alias.len(), jcamp.len());

    let selected = RSpinReader::new()
        .only_source_formats([LoadedSourceFormat::JeolJdf, LoadedSourceFormat::BrukerSer])
        .read_path(nmrxiv_fixture_root())?;
    assert_eq!(selected.len(), 4);
    assert_eq!(selected.len_1d(), 2);
    assert_eq!(selected.len_2d(), 2);
    assert_eq!(selected.source_format_count(LoadedSourceFormat::JeolJdf), 3);
    assert_eq!(
        selected.source_format_count(LoadedSourceFormat::BrukerSer),
        1
    );

    let cleared = RSpinReader::new()
        .only_source_format(LoadedSourceFormat::JcampDx)
        .all_source_formats()
        .read_path(nmrxiv_fixture_root())?;
    assert_eq!(cleared.len(), 7);

    let filtered_out = RSpinReader::new()
        .only_source_format(LoadedSourceFormat::Csv)
        .read_path(nmrxiv_fixture_root());
    let Err(error) = filtered_out else {
        anyhow::bail!("CSV-only source filter should leave no readable spectra");
    };
    assert!(error.to_string().contains("no readable bundle data found"));
    Ok(())
}

#[test]
fn loader_can_restrict_source_vendors() -> anyhow::Result<()> {
    let bruker = RSpinReader::new()
        .only_source_vendor(LoadedSourceVendor::Bruker)
        .read_path(nmrxiv_fixture_root())?;
    assert_eq!(bruker.len(), 2);
    assert_eq!(bruker.len_1d(), 1);
    assert_eq!(bruker.len_2d(), 1);
    assert_eq!(bruker.source_vendor_count(LoadedSourceVendor::Bruker), 2);

    let jeol = RSpinReader::new()
        .only_source_vendor("jeol")
        .read_path(nmrxiv_fixture_root())?;
    assert_eq!(jeol.len(), 3);
    assert!(jeol.has_source_vendor(LoadedSourceVendor::Jeol));
    assert!(
        jeol.spectra()
            .iter()
            .all(|entry| entry.source().vendor() == Some(LoadedSourceVendor::Jeol))
    );

    let selected = RSpinReader::new()
        .only_source_vendors([LoadedSourceVendor::Bruker, LoadedSourceVendor::Jeol])
        .read_path(nmrxiv_fixture_root())?;
    assert_eq!(selected.len(), 5);
    assert_eq!(selected.source_vendor_count(LoadedSourceVendor::Bruker), 2);
    assert_eq!(selected.source_vendor_count(LoadedSourceVendor::Jeol), 3);

    let filtered_out = RSpinReader::new()
        .only_source_vendor(LoadedSourceVendor::AgilentVarian)
        .read_path(nmrxiv_fixture_root());
    let Err(error) = filtered_out else {
        anyhow::bail!("Agilent/Varian-only vendor filter should leave no readable spectra");
    };
    assert!(error.to_string().contains("no readable bundle data found"));

    let invalid_vendor = RSpinReader::new()
        .only_source_vendor("jcamp")
        .read_path(nmrxiv_fixture_root());
    let Err(error) = invalid_vendor else {
        anyhow::bail!("unknown vendor filter should not fall back to a source format");
    };
    assert!(error.to_string().contains("no readable bundle data found"));
    Ok(())
}

#[test]
fn loader_source_format_filter_applies_to_nested_bundle_json() -> anyhow::Result<()> {
    let source_bundle = load_spectra(nmrxiv_fixture_root())?;
    let root = temp_dir("source-format-bundle")?;
    fs::write(
        root.join("bundle.json"),
        write_spectrum_bundle_json(&source_bundle)?,
    )?;

    let bundle = RSpinReader::new()
        .only_source_format(LoadedSourceFormat::JcampDx)
        .read_path(&root)?;
    assert_eq!(bundle.len(), 2);
    assert!(
        bundle
            .source_paths()
            .all(|path| path.starts_with(Path::new("bundle.json/jcamp")))
    );

    remove_dir(root)?;

    let root = temp_dir("source-format-alias-bundle")?;
    let spectrum = load_spectrum_1d(
        nmrxiv_fixture_root().join("jcamp/myrcene_1h_400mhz_jcamp_dx_6_link.jdx"),
    )?;
    let alias_bundle = SpectrumBundle::new().with_1d(
        spectrum,
        LoadedSource::new(Some(PathBuf::from("aliased-source.jdx")), "jdx"),
    );
    fs::write(
        root.join("bundle.json"),
        write_spectrum_bundle_json(&alias_bundle)?,
    )?;
    let bundle = RSpinReader::new()
        .only_source_format(LoadedSourceFormat::JcampDx)
        .read_path(&root)?;
    assert_eq!(bundle.len(), 1);
    assert_eq!(bundle.source_format_count("jcamp_dx"), 1);
    assert_eq!(bundle.source_format_count("jdx"), 1);
    assert_eq!(bundle.source_format_counts()[0].format(), "jcamp_dx");

    remove_dir(root)?;
    Ok(())
}

#[test]
fn loads_nmrxiv_cc0_mixed_vendor_directory_as_bundle() -> anyhow::Result<()> {
    let bundle = load_spectra(nmrxiv_fixture_root())?;

    assert_eq!(bundle.len(), 7);
    assert_eq!(bundle.spectra_1d().count(), 5);
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

    let jcamp_13c = loaded_1d_by_path(
        &bundle,
        Path::new("jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx"),
    )?;
    assert_eq!(jcamp_13c.len(), 104_858);
    assert_eq!(jcamp_13c.metadata.nucleus, Some(Nucleus::Carbon13));
    assert_eq!(
        loaded_source_format(
            &bundle,
            Path::new("jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx")
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
