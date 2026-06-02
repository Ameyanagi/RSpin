//! Integration tests for the unified spectrum bundle loader.

use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use rspin_core::{Nucleus, RSpinError, Unit};
use rspin_io as io;
use rspin_io::{
    LoadedSource, LoadedSourceDataKind, LoadedSourceFilter, LoadedSourceFormat, LoadedSourceVendor,
    LoadedSpectrum, RSpinReader, SourceDataKindCount, SourcePathCount, SpectrumBundle,
    SpectrumBundleLoader, SpectrumBundleSummary, SpectrumPathReader, WarningPathCount,
    load_spectra, load_spectra_1d, load_spectra_1d_many, load_spectra_1d_many_relative_to,
    load_spectra_1d_many_strict, load_spectra_1d_many_strict_relative_to,
    load_spectra_1d_many_summary, load_spectra_1d_many_summary_relative_to,
    load_spectra_1d_many_summary_strict, load_spectra_1d_many_summary_strict_relative_to,
    load_spectra_1d_relative_to, load_spectra_1d_strict, load_spectra_1d_strict_relative_to,
    load_spectra_1d_summary, load_spectra_1d_summary_relative_to, load_spectra_1d_summary_strict,
    load_spectra_1d_summary_strict_relative_to, load_spectra_2d, load_spectra_2d_many,
    load_spectra_2d_many_relative_to, load_spectra_2d_many_strict,
    load_spectra_2d_many_strict_relative_to, load_spectra_2d_many_summary,
    load_spectra_2d_many_summary_relative_to, load_spectra_2d_many_summary_strict,
    load_spectra_2d_many_summary_strict_relative_to, load_spectra_2d_relative_to,
    load_spectra_2d_strict, load_spectra_2d_strict_relative_to, load_spectra_2d_summary,
    load_spectra_2d_summary_relative_to, load_spectra_2d_summary_strict,
    load_spectra_2d_summary_strict_relative_to, load_spectra_by_source,
    load_spectra_by_source_data_kind, load_spectra_by_source_data_kind_relative_to,
    load_spectra_by_source_data_kinds, load_spectra_by_source_data_kinds_relative_to,
    load_spectra_by_source_format, load_spectra_by_source_format_relative_to,
    load_spectra_by_source_formats, load_spectra_by_source_formats_relative_to,
    load_spectra_by_source_path, load_spectra_by_source_path_prefix,
    load_spectra_by_source_path_prefix_relative_to, load_spectra_by_source_path_relative_to,
    load_spectra_by_source_relative_to, load_spectra_by_source_vendor,
    load_spectra_by_source_vendor_relative_to, load_spectra_by_source_vendors,
    load_spectra_by_source_vendors_relative_to, load_spectra_by_sources,
    load_spectra_by_sources_relative_to, load_spectra_many, load_spectra_many_by_source,
    load_spectra_many_by_source_data_kind, load_spectra_many_by_source_data_kind_relative_to,
    load_spectra_many_by_source_data_kinds, load_spectra_many_by_source_data_kinds_relative_to,
    load_spectra_many_by_source_format, load_spectra_many_by_source_format_relative_to,
    load_spectra_many_by_source_formats, load_spectra_many_by_source_formats_relative_to,
    load_spectra_many_by_source_path, load_spectra_many_by_source_path_prefix,
    load_spectra_many_by_source_path_prefix_relative_to,
    load_spectra_many_by_source_path_relative_to, load_spectra_many_by_source_relative_to,
    load_spectra_many_by_source_vendor, load_spectra_many_by_source_vendor_relative_to,
    load_spectra_many_by_source_vendors, load_spectra_many_by_source_vendors_relative_to,
    load_spectra_many_by_sources, load_spectra_many_by_sources_relative_to,
    load_spectra_many_relative_to, load_spectra_many_strict, load_spectra_many_strict_relative_to,
    load_spectra_many_summary, load_spectra_many_summary_relative_to,
    load_spectra_many_summary_strict, load_spectra_many_summary_strict_relative_to,
    load_spectra_relative_to, load_spectra_strict, load_spectra_strict_relative_to,
    load_spectra_summary, load_spectra_summary_relative_to, load_spectra_summary_strict,
    load_spectra_summary_strict_relative_to, load_spectrum_1d, load_spectrum_1d_many,
    load_spectrum_1d_many_relative_to, load_spectrum_1d_many_with_source,
    load_spectrum_1d_many_with_source_relative_to, load_spectrum_1d_paths,
    load_spectrum_1d_paths_relative_to, load_spectrum_1d_paths_with_source,
    load_spectrum_1d_paths_with_source_relative_to, load_spectrum_1d_relative_to,
    load_spectrum_1d_with_source, load_spectrum_1d_with_source_relative_to, load_spectrum_2d,
    load_spectrum_2d_many, load_spectrum_2d_many_relative_to, load_spectrum_2d_many_with_source,
    load_spectrum_2d_many_with_source_relative_to, load_spectrum_2d_paths,
    load_spectrum_2d_paths_relative_to, load_spectrum_2d_paths_with_source,
    load_spectrum_2d_paths_with_source_relative_to, load_spectrum_2d_relative_to,
    load_spectrum_2d_with_source_relative_to, parse_loaded_source_format,
    parse_loaded_source_vendor, supported_bundle_source_data_kinds,
    supported_bundle_source_formats, supported_bundle_source_vendors, write_spectrum_bundle_json,
    write_spectrum1d_json, write_spectrum2d_json,
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

    let raw_alias = RSpinReader::new()
        .processed_only()
        .raw()
        .read_path(&bruker)?;
    assert_eq!(raw_alias.len(), 1);
    assert_eq!(first_1d(&raw_alias)?.x.unit, Unit::Seconds);

    let processed_alias = RSpinReader::new()
        .raw_only()
        .processed()
        .read_path(&bruker)?;
    assert_eq!(processed_alias.len(), 1);
    assert_eq!(first_1d(&processed_alias)?.x.unit, Unit::Ppm);

    let all_candidate_aliases = RSpinReader::new()
        .raw()
        .raw_and_processed()
        .read_path(&bruker)?;
    assert_eq!(all_candidate_aliases.len(), 2);
    assert_eq!(
        all_candidate_aliases.source_format_count(LoadedSourceFormat::BrukerFid),
        1
    );
    assert_eq!(
        all_candidate_aliases.source_format_count(LoadedSourceFormat::BrukerProcessed),
        1
    );

    let mixed = nmrxiv_fixture_root();
    let one_d_only = RSpinReader::new().one_d_only().read_path(&mixed)?;
    assert_eq!(one_d_only.len_1d(), 5);
    assert_eq!(one_d_only.len_2d(), 0);

    let two_d_only = RSpinReader::new().two_d_only().read_path(&mixed)?;
    assert_eq!(two_d_only.len_1d(), 0);
    assert_eq!(two_d_only.len_2d(), 2);

    let one_d_alias = RSpinReader::new().two_d_only().one_d().read_path(&mixed)?;
    assert_eq!(one_d_alias.len_1d(), 5);
    assert_eq!(one_d_alias.len_2d(), 0);

    let two_d_alias = RSpinReader::new().one_d_only().two_d().read_path(&mixed)?;
    assert_eq!(two_d_alias.len_1d(), 0);
    assert_eq!(two_d_alias.len_2d(), 2);

    let all_dimension_aliases = RSpinReader::new()
        .one_d()
        .one_d_and_two_d()
        .read_path(&mixed)?;
    assert_eq!(all_dimension_aliases.len_1d(), 5);
    assert_eq!(all_dimension_aliases.len_2d(), 2);

    let no_sources = RSpinReader::new()
        .without_source_paths()
        .read_path(&bruker)?;
    assert!(
        no_sources
            .spectra()
            .iter()
            .all(|loaded| loaded.source().path.is_none())
    );

    let hidden_sources = RSpinReader::new().hide_source_paths().read_path(&bruker)?;
    assert!(
        hidden_sources
            .spectra()
            .iter()
            .all(|loaded| loaded.source().path.is_none())
    );

    let tracked_sources = RSpinReader::new()
        .hide_source_paths()
        .track_source_paths()
        .read_path(&bruker)?;
    assert!(
        tracked_sources
            .spectra()
            .iter()
            .all(|loaded| loaded.source().path.is_some())
    );

    let strict_error = RSpinReader::new()
        .strict()
        .read_path(fixture_root().join("empty_jcamp/empty.jdx"));
    let Err(error) = strict_error else {
        anyhow::bail!("strict helper should fail on unreadable candidates");
    };
    assert!(error.to_string().contains("missing XYDATA values"));

    let lenient = RSpinReader::new()
        .strict()
        .lenient()
        .read_path(fixture_root())?;
    assert!(lenient.has_warnings());
    Ok(())
}

#[test]
fn dimension_specific_bundle_helpers_load_matching_spectra() -> anyhow::Result<()> {
    let mixed = nmrxiv_fixture_root();

    let one_d = load_spectra_1d(&mixed)?;
    assert_eq!(one_d.len_1d(), 5);
    assert_eq!(one_d.len_2d(), 0);
    assert!(one_d.spectra().iter().all(LoadedSpectrum::is_1d));

    let two_d = load_spectra_2d(&mixed)?;
    assert_eq!(two_d.len_1d(), 0);
    assert_eq!(two_d.len_2d(), 2);
    assert!(two_d.spectra().iter().all(LoadedSpectrum::is_2d));

    let selected_1d = load_spectra_1d_relative_to(&mixed, "jcamp")?;
    assert_eq!(selected_1d.len_1d(), 2);
    assert!(selected_1d.has_source_path("jcamp/myrcene_1h_400mhz_jcamp_dx_6_link.jdx"));

    let selected_2d = load_spectra_2d_relative_to(&mixed, "bruker_cosy_raw")?;
    assert_eq!(selected_2d.len_2d(), 1);
    assert!(selected_2d.has_source_path("bruker_cosy_raw"));

    let many_1d = load_spectra_1d_many([fixture_root().join("bruker_without_expno")])?;
    assert_eq!(many_1d.len_1d(), 2);

    let many_2d = load_spectra_2d_many([mixed.join("bruker_cosy_raw"), mixed.join("jeol")])?;
    assert_eq!(many_2d.len_2d(), 2);

    let relative_many_1d = load_spectra_1d_many_relative_to(&mixed, ["jcamp", "jeol"])?;
    assert_eq!(relative_many_1d.len_1d(), 4);

    let relative_many_2d = load_spectra_2d_many_relative_to(&mixed, ["bruker_cosy_raw"])?;
    assert_eq!(relative_many_2d.len_2d(), 1);
    Ok(())
}

#[test]
fn dimension_specific_bundle_summary_helpers_match_loaded_bundles() -> anyhow::Result<()> {
    let base = fixture_root();
    let mixed = nmrxiv_fixture_root();

    let one_d = load_spectra_1d(&mixed)?;
    let one_d_summary = load_spectra_1d_summary(&mixed)?;
    assert_eq!(one_d_summary, one_d.summary());
    assert_eq!(one_d_summary.spectra_1d(), 5);
    assert_eq!(one_d_summary.spectra_2d(), 0);

    let two_d = load_spectra_2d(&mixed)?;
    let two_d_summary = load_spectra_2d_summary(&mixed)?;
    assert_eq!(two_d_summary, two_d.summary());
    assert_eq!(two_d_summary.spectra_1d(), 0);
    assert_eq!(two_d_summary.spectra_2d(), 2);

    let selected_1d = load_spectra_1d_relative_to(&mixed, "jcamp")?;
    assert_eq!(
        load_spectra_1d_summary_relative_to(&mixed, "jcamp")?,
        selected_1d.summary()
    );

    let selected_2d = load_spectra_2d_relative_to(&mixed, "bruker_cosy_raw")?;
    assert_eq!(
        load_spectra_2d_summary_relative_to(&mixed, "bruker_cosy_raw")?,
        selected_2d.summary()
    );

    let many_1d = load_spectra_1d_many([base.join("bruker_without_expno")])?;
    assert_eq!(
        load_spectra_1d_many_summary([base.join("bruker_without_expno")])?,
        many_1d.summary()
    );

    let many_2d = load_spectra_2d_many([mixed.join("bruker_cosy_raw"), mixed.join("jeol")])?;
    assert_eq!(
        load_spectra_2d_many_summary([mixed.join("bruker_cosy_raw"), mixed.join("jeol")])?,
        many_2d.summary()
    );

    let relative_many_1d = load_spectra_1d_many_relative_to(&mixed, ["jcamp", "jeol"])?;
    assert_eq!(
        load_spectra_1d_many_summary_relative_to(&mixed, ["jcamp", "jeol"])?,
        relative_many_1d.summary()
    );

    let relative_many_2d = load_spectra_2d_many_relative_to(&mixed, ["bruker_cosy_raw"])?;
    assert_eq!(
        load_spectra_2d_many_summary_relative_to(&mixed, ["bruker_cosy_raw"])?,
        relative_many_2d.summary()
    );

    let strict_1d = load_spectra_1d_strict(base.join("varian_1h"))?;
    assert_eq!(
        load_spectra_1d_summary_strict(base.join("varian_1h"))?,
        strict_1d.summary()
    );

    let strict_1d_relative = load_spectra_1d_strict_relative_to(&base, "varian_1h")?;
    assert_eq!(
        load_spectra_1d_summary_strict_relative_to(&base, "varian_1h")?,
        strict_1d_relative.summary()
    );

    let strict_1d_many = load_spectra_1d_many_strict([base.join("varian_1h")])?;
    assert_eq!(
        load_spectra_1d_many_summary_strict([base.join("varian_1h")])?,
        strict_1d_many.summary()
    );

    let strict_1d_many_relative = load_spectra_1d_many_strict_relative_to(&base, ["varian_1h"])?;
    assert_eq!(
        load_spectra_1d_many_summary_strict_relative_to(&base, ["varian_1h"])?,
        strict_1d_many_relative.summary()
    );

    let strict_2d = load_spectra_2d_strict(mixed.join("bruker_cosy_raw"))?;
    assert_eq!(
        load_spectra_2d_summary_strict(mixed.join("bruker_cosy_raw"))?,
        strict_2d.summary()
    );

    let strict_2d_relative = load_spectra_2d_strict_relative_to(&mixed, "bruker_cosy_raw")?;
    assert_eq!(
        load_spectra_2d_summary_strict_relative_to(&mixed, "bruker_cosy_raw")?,
        strict_2d_relative.summary()
    );

    let strict_2d_many = load_spectra_2d_many_strict([mixed.join("bruker_cosy_raw")])?;
    assert_eq!(
        load_spectra_2d_many_summary_strict([mixed.join("bruker_cosy_raw")])?,
        strict_2d_many.summary()
    );

    let strict_2d_many_relative =
        load_spectra_2d_many_strict_relative_to(&mixed, ["bruker_cosy_raw"])?;
    assert_eq!(
        load_spectra_2d_many_summary_strict_relative_to(&mixed, ["bruker_cosy_raw"])?,
        strict_2d_many_relative.summary()
    );

    Ok(())
}

#[test]
fn strict_dimension_bundle_helpers_abort_on_bad_candidates() -> anyhow::Result<()> {
    let base = fixture_root();
    let mixed = nmrxiv_fixture_root();

    let one_d = load_spectra_1d_strict(base.join("varian_1h"))?;
    assert_eq!(one_d.len_1d(), 1);
    assert!(one_d.warnings().is_empty());

    let one_d_relative = load_spectra_1d_strict_relative_to(&base, "varian_1h")?;
    assert_eq!(one_d_relative.len_1d(), 1);
    assert!(one_d_relative.has_source_path("varian_1h"));

    let one_d_many = load_spectra_1d_many_strict([base.join("varian_1h")])?;
    assert_eq!(one_d_many.len_1d(), 1);

    let one_d_many_relative = load_spectra_1d_many_strict_relative_to(&base, ["varian_1h"])?;
    assert_eq!(one_d_many_relative.len_1d(), 1);

    let two_d = load_spectra_2d_strict(mixed.join("bruker_cosy_raw"))?;
    assert_eq!(two_d.len_2d(), 1);

    let two_d_relative = load_spectra_2d_strict_relative_to(&mixed, "bruker_cosy_raw")?;
    assert_eq!(two_d_relative.len_2d(), 1);

    let two_d_many = load_spectra_2d_many_strict([mixed.join("bruker_cosy_raw")])?;
    assert_eq!(two_d_many.len_2d(), 1);

    let two_d_many_relative = load_spectra_2d_many_strict_relative_to(&mixed, ["bruker_cosy_raw"])?;
    assert_eq!(two_d_many_relative.len_2d(), 1);

    let malformed = load_spectra_1d_strict(base.join("empty_jcamp/empty.jdx"));
    let Err(error) = malformed else {
        anyhow::bail!("strict dimension loading should reject malformed candidates");
    };
    assert!(error.to_string().contains("missing XYDATA values"));
    Ok(())
}

#[test]
fn reader_dimension_bundle_helpers_preserve_other_filters() -> anyhow::Result<()> {
    let mixed = nmrxiv_fixture_root();

    let raw_bruker_2d = RSpinReader::new()
        .raw_sources()
        .source_vendor(LoadedSourceVendor::Bruker)
        .read_bundle_2d(&mixed)?;
    assert_eq!(raw_bruker_2d.len_2d(), 1);
    assert!(raw_bruker_2d.has_source_path("bruker_cosy_raw"));
    let raw_bruker_2d_summary = RSpinReader::new()
        .raw_sources()
        .source_vendor(LoadedSourceVendor::Bruker)
        .read_bundle_2d_summary(&mixed)?;
    assert_eq!(raw_bruker_2d_summary, raw_bruker_2d.summary());

    let jcamp_1d = RSpinReader::new()
        .source_format(LoadedSourceFormat::JcampDx)
        .read_bundle_1d_relative_to(&mixed, "jcamp")?;
    assert_eq!(jcamp_1d.len_1d(), 2);
    assert_eq!(jcamp_1d.source_format_count(LoadedSourceFormat::JcampDx), 2);
    let jcamp_1d_summary = RSpinReader::new()
        .source_format(LoadedSourceFormat::JcampDx)
        .read_bundle_1d_summary_relative_to(&mixed, "jcamp")?;
    assert_eq!(jcamp_1d_summary, jcamp_1d.summary());

    let reader_many = RSpinReader::new()
        .source_vendor("jeol")
        .read_bundle_1d_many_relative_to(&mixed, ["jcamp", "jeol"])?;
    assert_eq!(reader_many.len_1d(), 2);
    assert_eq!(reader_many.source_vendor_count(LoadedSourceVendor::Jeol), 2);
    let reader_many_summary = RSpinReader::new()
        .source_vendor("jeol")
        .read_bundle_1d_summary_many_relative_to(&mixed, ["jcamp", "jeol"])?;
    assert_eq!(reader_many_summary, reader_many.summary());

    let wrong_dimension = RSpinReader::new().read_bundle_2d(fixture_root().join("varian_1h"));
    let Err(error) = wrong_dimension else {
        anyhow::bail!("2D bundle loading should reject one-dimensional-only input");
    };
    assert!(
        error
            .to_string()
            .contains("one-dimensional spectrum candidates are disabled")
    );
    let wrong_dimension_summary =
        RSpinReader::new().read_bundle_2d_summary(fixture_root().join("varian_1h"));
    let Err(error) = wrong_dimension_summary else {
        anyhow::bail!("2D bundle summary loading should reject one-dimensional-only input");
    };
    assert!(
        error
            .to_string()
            .contains("one-dimensional spectrum candidates are disabled")
    );
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
fn reader_first_spectrum_helpers_cover_quick_inspection() -> anyhow::Result<()> {
    let base = fixture_root();
    let mixed = nmrxiv_fixture_root();

    let varian = RSpinReader::new().read_first_1d_relative_to(&base, "varian_1h")?;
    assert_eq!(varian.metadata.nucleus, Some(Nucleus::Hydrogen1));

    let (varian, source) =
        RSpinReader::new().read_first_1d_with_source_relative_to(&base, "varian_1h")?;
    assert_eq!(varian.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(source.path(), Some(Path::new("varian_1h")));

    let first_many = RSpinReader::new()
        .read_first_1d_many_relative_to(&base, ["varian_1h", "bruker_without_expno"])?;
    assert_eq!(first_many.metadata.nucleus, Some(Nucleus::Hydrogen1));

    let (first_many, source) = RSpinReader::new()
        .read_first_1d_many_with_source_relative_to(&base, ["varian_1h", "bruker_without_expno"])?;
    assert_eq!(first_many.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(source.path(), Some(Path::new("varian_1h")));

    let cosy = RSpinReader::new().read_first_2d_relative_to(&mixed, "bruker_cosy_raw")?;
    assert_eq!(cosy.shape(), (2048, 512));

    let (cosy, source) =
        RSpinReader::new().read_first_2d_with_source_relative_to(&mixed, "bruker_cosy_raw")?;
    assert_eq!(cosy.shape(), (2048, 512));
    assert_eq!(source.path(), Some(Path::new("bruker_cosy_raw")));
    Ok(())
}

#[test]
fn reader_first_spectrum_helpers_work_with_direct_and_many_paths() -> anyhow::Result<()> {
    let base = fixture_root();
    let mixed = nmrxiv_fixture_root();

    let one_d = RSpinReader::new().read_first_1d(base.join("varian_1h"))?;
    assert_eq!(one_d.metadata.nucleus, Some(Nucleus::Hydrogen1));

    let (one_d, source) = RSpinReader::new().read_first_1d_with_source(base.join("varian_1h"))?;
    assert_eq!(one_d.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(source.path(), Some(Path::new("varian_1h")));

    let two_d = RSpinReader::new().read_first_2d(mixed.join("bruker_cosy_raw"))?;
    assert_eq!(two_d.shape(), (2048, 512));

    let (two_d, source) =
        RSpinReader::new().read_first_2d_many_with_source([mixed.join("bruker_cosy_raw")])?;
    assert_eq!(two_d.shape(), (2048, 512));
    assert_eq!(source.path(), Some(Path::new("bruker_cosy_raw")));

    let missing = RSpinReader::new().read_first_2d_many([base.join("varian_1h")]);
    let Err(error) = missing else {
        anyhow::bail!("missing first 2D reader helper should fail");
    };
    assert!(
        error
            .to_string()
            .contains("expected at least one two-dimensional spectrum")
    );
    Ok(())
}

#[test]
fn reader_first_any_spectrum_helpers_cover_quick_inspection() -> anyhow::Result<()> {
    let base = fixture_root();
    let mixed = nmrxiv_fixture_root();

    let first = RSpinReader::new().read_first_spectrum_relative_to(&base, "varian_1h")?;
    assert!(first.is_1d());
    assert_eq!(first.source().path(), Some(Path::new("varian_1h")));
    let Some(spectrum) = first.as_1d() else {
        anyhow::bail!("first spectrum should be one-dimensional");
    };
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Hydrogen1));

    let first = RSpinReader::new().read_first_spectrum_many_relative_to(&base, ["varian_1h"])?;
    assert!(first.is_1d());
    assert_eq!(first.source().path(), Some(Path::new("varian_1h")));

    let first = RSpinReader::new().read_first_spectrum(mixed.join("bruker_cosy_raw"))?;
    assert!(first.is_2d());
    assert_eq!(first.source().path(), Some(Path::new("bruker_cosy_raw")));
    let Some(spectrum) = first.as_2d() else {
        anyhow::bail!("first spectrum should be two-dimensional");
    };
    assert_eq!(spectrum.shape(), (2048, 512));

    let first = RSpinReader::new().read_first_spectrum_many([mixed.join("bruker_cosy_raw")])?;
    assert!(first.is_2d());
    assert_eq!(first.source().path(), Some(Path::new("bruker_cosy_raw")));
    Ok(())
}

#[test]
fn free_first_spectrum_helpers_cover_quick_inspection() -> anyhow::Result<()> {
    let base = fixture_root();
    let mixed = nmrxiv_fixture_root();

    let one_d = io::load_first_spectrum_1d_relative_to(&base, "varian_1h")?;
    assert_eq!(one_d.metadata.nucleus, Some(Nucleus::Hydrogen1));

    let (one_d, source) = io::load_first_spectrum_1d_with_source_relative_to(&base, "varian_1h")?;
    assert_eq!(one_d.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(source.path(), Some(Path::new("varian_1h")));

    let one_d = io::load_first_spectrum_1d(base.join("varian_1h"))?;
    assert_eq!(one_d.metadata.nucleus, Some(Nucleus::Hydrogen1));

    let (one_d, source) = io::load_first_spectrum_1d_with_source(base.join("varian_1h"))?;
    assert_eq!(one_d.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(source.path(), Some(Path::new("varian_1h")));

    let first_many =
        io::load_first_spectrum_1d_many_relative_to(&base, ["varian_1h", "bruker_without_expno"])?;
    assert_eq!(first_many.metadata.nucleus, Some(Nucleus::Hydrogen1));

    let (first_many, source) = io::load_first_spectrum_1d_many_with_source_relative_to(
        &base,
        ["varian_1h", "bruker_without_expno"],
    )?;
    assert_eq!(first_many.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(source.path(), Some(Path::new("varian_1h")));

    let first_many = io::load_first_spectrum_1d_many([base.join("varian_1h")])?;
    assert_eq!(first_many.metadata.nucleus, Some(Nucleus::Hydrogen1));

    let (first_many, source) =
        io::load_first_spectrum_1d_many_with_source([base.join("varian_1h")])?;
    assert_eq!(first_many.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(source.path(), Some(Path::new("varian_1h")));

    let two_d = io::load_first_spectrum_2d_relative_to(&mixed, "bruker_cosy_raw")?;
    assert_eq!(two_d.shape(), (2048, 512));

    let (two_d, source) =
        io::load_first_spectrum_2d_with_source_relative_to(&mixed, "bruker_cosy_raw")?;
    assert_eq!(two_d.shape(), (2048, 512));
    assert_eq!(source.path(), Some(Path::new("bruker_cosy_raw")));

    let two_d = io::load_first_spectrum_2d(mixed.join("bruker_cosy_raw"))?;
    assert_eq!(two_d.shape(), (2048, 512));

    let (two_d, source) = io::load_first_spectrum_2d_with_source(mixed.join("bruker_cosy_raw"))?;
    assert_eq!(two_d.shape(), (2048, 512));
    assert_eq!(source.path(), Some(Path::new("bruker_cosy_raw")));

    let two_d = io::load_first_spectrum_2d_many_relative_to(&mixed, ["bruker_cosy_raw"])?;
    assert_eq!(two_d.shape(), (2048, 512));

    let (two_d, source) =
        io::load_first_spectrum_2d_many_with_source_relative_to(&mixed, ["bruker_cosy_raw"])?;
    assert_eq!(two_d.shape(), (2048, 512));
    assert_eq!(source.path(), Some(Path::new("bruker_cosy_raw")));

    let two_d = io::load_first_spectrum_2d_many([mixed.join("bruker_cosy_raw")])?;
    assert_eq!(two_d.shape(), (2048, 512));

    let (two_d, source) =
        io::load_first_spectrum_2d_many_with_source([mixed.join("bruker_cosy_raw")])?;
    assert_eq!(two_d.shape(), (2048, 512));
    assert_eq!(source.path(), Some(Path::new("bruker_cosy_raw")));

    let missing = io::load_first_spectrum_2d_many([base.join("varian_1h")]);
    let Err(error) = missing else {
        anyhow::bail!("missing first 2D free helper should fail");
    };
    assert!(
        error
            .to_string()
            .contains("expected at least one two-dimensional spectrum")
    );
    Ok(())
}

#[test]
fn free_first_any_spectrum_helpers_cover_quick_inspection() -> anyhow::Result<()> {
    let base = fixture_root();
    let mixed = nmrxiv_fixture_root();

    let first = io::load_first_spectrum_relative_to(&base, "varian_1h")?;
    assert!(first.is_1d());
    assert_eq!(first.source().path(), Some(Path::new("varian_1h")));

    let first = io::load_first_spectrum_many_relative_to(&base, ["varian_1h"])?;
    assert!(first.is_1d());
    assert_eq!(first.source().path(), Some(Path::new("varian_1h")));

    let first = io::load_first_spectrum(mixed.join("bruker_cosy_raw"))?;
    assert!(first.is_2d());
    assert_eq!(first.source().path(), Some(Path::new("bruker_cosy_raw")));

    let first = io::load_first_spectrum_many([mixed.join("bruker_cosy_raw")])?;
    assert!(first.is_2d());
    assert_eq!(first.source().path(), Some(Path::new("bruker_cosy_raw")));
    Ok(())
}

#[test]
fn bundle_first_spectrum_accessors_cover_quick_inspection() -> anyhow::Result<()> {
    let one_d_bundle = load_spectra(fixture_root().join("varian_1h"))?;
    let first_1d = one_d_bundle
        .first_1d()
        .ok_or_else(|| anyhow::anyhow!("missing first one-dimensional spectrum"))?;
    assert_eq!(first_1d.metadata.nucleus, Some(Nucleus::Hydrogen1));

    let (loaded_1d, source) = one_d_bundle
        .first_loaded_1d()
        .ok_or_else(|| anyhow::anyhow!("missing first loaded one-dimensional spectrum"))?;
    assert_eq!(loaded_1d.len(), first_1d.len());
    assert_eq!(source.path(), Some(Path::new("varian_1h")));
    assert!(source.is_vendor("varian"));
    assert!(one_d_bundle.first_2d().is_none());
    assert!(one_d_bundle.first_loaded_2d().is_none());

    let two_d_bundle = load_spectra(nmrxiv_fixture_root().join("bruker_cosy_raw"))?;
    let first_2d = two_d_bundle
        .first_2d()
        .ok_or_else(|| anyhow::anyhow!("missing first two-dimensional spectrum"))?;
    assert_eq!(first_2d.shape(), (2048, 512));

    let (loaded_2d, source) = two_d_bundle
        .first_loaded_2d()
        .ok_or_else(|| anyhow::anyhow!("missing first loaded two-dimensional spectrum"))?;
    assert_eq!(loaded_2d.shape(), first_2d.shape());
    assert_eq!(source.path(), Some(Path::new("bruker_cosy_raw")));
    assert!(source.is_vendor("bruker"));
    assert!(two_d_bundle.first_1d().is_none());
    assert!(two_d_bundle.first_loaded_1d().is_none());

    let empty = SpectrumBundle::new();
    assert!(empty.first_1d().is_none());
    assert!(empty.first_loaded_1d().is_none());
    assert!(empty.first_2d().is_none());
    assert!(empty.first_loaded_2d().is_none());
    Ok(())
}

#[test]
fn bundle_first_any_spectrum_accessors_cover_quick_inspection() -> anyhow::Result<()> {
    let one_d_bundle = load_spectra(fixture_root().join("varian_1h"))?;
    let first = one_d_bundle
        .first_spectrum()
        .ok_or_else(|| anyhow::anyhow!("missing first spectrum"))?;
    assert!(first.is_1d());
    assert_eq!(first.source().path(), Some(Path::new("varian_1h")));
    assert_eq!(
        one_d_bundle.require_first_spectrum()?.source(),
        first.source()
    );

    let first = one_d_bundle.clone().into_first_spectrum()?;
    assert!(first.is_1d());
    assert_eq!(first.source().path(), Some(Path::new("varian_1h")));

    let two_d_bundle = load_spectra(nmrxiv_fixture_root().join("bruker_cosy_raw"))?;
    let first = two_d_bundle.require_first_spectrum()?;
    assert!(first.is_2d());
    assert_eq!(first.source().path(), Some(Path::new("bruker_cosy_raw")));

    let empty = SpectrumBundle::new();
    assert!(empty.first_spectrum().is_none());
    let missing = empty.require_first_spectrum();
    let Err(error) = missing else {
        anyhow::bail!("empty bundle should not have a first spectrum");
    };
    assert!(error.to_string().contains("expected at least one spectrum"));
    Ok(())
}

#[test]
fn bundle_required_first_accessors_return_typed_errors() -> anyhow::Result<()> {
    let one_d_bundle = load_spectra(fixture_root().join("varian_1h"))?;
    assert_eq!(
        one_d_bundle.require_first_1d()?.metadata.nucleus,
        Some(Nucleus::Hydrogen1)
    );
    let (first_loaded, source) = one_d_bundle.require_first_loaded_1d()?;
    assert_eq!(first_loaded.len(), one_d_bundle.require_first_1d()?.len());
    assert_eq!(source.path(), Some(Path::new("varian_1h")));

    let missing_2d = one_d_bundle.require_first_2d();
    let Err(error) = missing_2d else {
        anyhow::bail!("missing 2D first accessor should fail");
    };
    assert!(
        error
            .to_string()
            .contains("expected at least one two-dimensional spectrum")
    );

    let two_d_bundle = load_spectra(nmrxiv_fixture_root().join("bruker_cosy_raw"))?;
    assert_eq!(two_d_bundle.require_first_2d()?.shape(), (2048, 512));
    let (first_2d, source) = two_d_bundle.require_first_loaded_2d()?;
    assert_eq!(first_2d.shape(), (2048, 512));
    assert_eq!(source.path(), Some(Path::new("bruker_cosy_raw")));
    Ok(())
}

#[test]
fn bundle_consuming_first_accessors_return_owned_spectra() -> anyhow::Result<()> {
    let one_d = load_spectra(fixture_root().join("bruker_without_expno"))?.into_first_1d()?;
    assert_eq!(one_d.metadata.nucleus, Some(Nucleus::Hydrogen1));

    let (one_d, source) =
        load_spectra(fixture_root().join("bruker_without_expno"))?.into_first_loaded_1d()?;
    assert_eq!(one_d.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert!(source.path().is_some());

    let two_d = load_spectra(nmrxiv_fixture_root().join("bruker_cosy_raw"))?.into_first_2d()?;
    assert_eq!(two_d.shape(), (2048, 512));

    let (two_d, source) =
        load_spectra(nmrxiv_fixture_root().join("bruker_cosy_raw"))?.into_first_loaded_2d()?;
    assert_eq!(two_d.shape(), (2048, 512));
    assert_eq!(source.path(), Some(Path::new("bruker_cosy_raw")));
    Ok(())
}

#[test]
fn bundle_first_source_filter_accessors_cover_quick_selection() -> anyhow::Result<()> {
    let bundle = load_spectra(nmrxiv_fixture_root())?;

    let first_any = bundle
        .first_by_sources(Vec::<LoadedSourceFilter>::new())
        .ok_or_else(|| anyhow::anyhow!("missing first loaded spectrum"))?;
    assert!(first_any.source().path().is_some());

    let jeol = bundle
        .first_by_source(LoadedSourceFilter::jeol())
        .ok_or_else(|| anyhow::anyhow!("missing first JEOL spectrum"))?;
    assert_eq!(jeol.source().vendor(), Some(LoadedSourceVendor::Jeol));

    let jcamp = bundle
        .first_1d_by_source(LoadedSourceFilter::jcamp())
        .ok_or_else(|| anyhow::anyhow!("missing first JCAMP-DX one-dimensional spectrum"))?;
    assert!(jcamp.metadata.nucleus.is_some());

    let (carbon, source) = bundle
        .first_loaded_1d_by_sources([
            LoadedSourceFilter::path("missing"),
            LoadedSourceFilter::path("jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx"),
        ])
        .ok_or_else(|| anyhow::anyhow!("missing selected carbon spectrum"))?;
    assert_eq!(carbon.metadata.nucleus, Some(Nucleus::Carbon13));
    assert_eq!(
        source.path(),
        Some(Path::new("jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx"))
    );

    let bruker_2d = bundle
        .first_2d_by_source(LoadedSourceVendor::Bruker)
        .ok_or_else(|| anyhow::anyhow!("missing Bruker two-dimensional spectrum"))?;
    assert_eq!(bruker_2d.shape(), (2048, 512));

    let (_, jeol_2d_source) = bundle
        .first_loaded_2d_by_sources([LoadedSourceFilter::path_prefix("jeol")])
        .ok_or_else(|| anyhow::anyhow!("missing JEOL two-dimensional spectrum"))?;
    let jeol_2d_path = jeol_2d_source
        .path()
        .ok_or_else(|| anyhow::anyhow!("missing JEOL two-dimensional source path"))?;
    assert!(jeol_2d_path.starts_with("jeol"));

    assert!(
        bundle
            .first_1d_by_source(LoadedSourceFilter::vendor("unknown-vendor"))
            .is_none()
    );
    assert!(
        bundle
            .first_loaded_2d_by_source(LoadedSourceFormat::JcampDx)
            .is_none()
    );

    let bruker_source = LoadedSource::new(None, LoadedSourceFormat::BrukerFid.as_str());
    assert!(LoadedSourceFilter::bruker().matches_source(&bruker_source));
    let varian_source = LoadedSource::new(None, LoadedSourceFormat::AgilentFid.as_str());
    assert!(LoadedSourceFilter::varian().matches_source(&varian_source));
    assert!(LoadedSourceFilter::agilent().matches_source(&varian_source));
    assert!(LoadedSourceFilter::agilent_varian().matches_source(&varian_source));
    let json_source = LoadedSource::new(None, LoadedSourceFormat::Json.as_str());
    assert!(LoadedSourceFilter::json().matches_source(&json_source));
    let csv_source = LoadedSource::new(None, LoadedSourceFormat::Csv.as_str());
    assert!(LoadedSourceFilter::csv().matches_source(&csv_source));
    let nmrml_source = LoadedSource::new(None, LoadedSourceFormat::NmrMl.as_str());
    assert!(LoadedSourceFilter::nmrml().matches_source(&nmrml_source));
    Ok(())
}

#[test]
fn reader_exposes_supported_source_metadata() -> anyhow::Result<()> {
    let formats = RSpinReader::supported_source_formats();
    assert_eq!(formats, supported_bundle_source_formats());
    assert!(formats.iter().any(|info| {
        info.name == "jcamp_dx"
            && info.vendor.is_none()
            && info.data_kind == LoadedSourceDataKind::Other
            && info.extensions.contains(&"jdx")
            && !info.path_markers.contains(&"jdx")
    }));
    assert!(formats.iter().any(|info| {
        info.name == "agilent_fid"
            && info.vendor == Some("agilent_varian")
            && info.data_kind == LoadedSourceDataKind::Raw
            && info.path_markers.contains(&"procpar")
    }));

    let vendors = RSpinReader::supported_source_vendors();
    assert_eq!(vendors, supported_bundle_source_vendors());
    let agilent = vendors
        .iter()
        .find(|info| info.name == "agilent_varian")
        .ok_or_else(|| anyhow::anyhow!("missing Agilent/Varian source vendor metadata"))?;
    assert!(agilent.source_formats.contains(&"agilent_processed"));
    assert!(agilent.source_formats.contains(&"agilent_fid"));

    let data_kinds = RSpinReader::supported_source_data_kinds();
    assert_eq!(data_kinds, supported_bundle_source_data_kinds());
    let processed = data_kinds
        .iter()
        .find(|info| info.name == "processed")
        .ok_or_else(|| anyhow::anyhow!("missing processed source data-kind metadata"))?;
    assert!(processed.source_formats.contains(&"bruker_processed"));
    assert!(processed.source_formats.contains(&"agilent_processed"));
    assert!(!processed.source_formats.contains(&"jcamp_dx"));
    Ok(())
}

#[test]
fn reader_short_source_filter_aliases_cover_common_workflows() -> anyhow::Result<()> {
    let base = fixture_root();

    let bruker_processed = RSpinReader::new()
        .source_format(LoadedSourceFormat::BrukerProcessed)
        .read_relative_to(&base, "bruker_without_expno")?;
    assert_eq!(bruker_processed.len(), 1);
    assert_eq!(first_1d(&bruker_processed)?.x.unit, Unit::Ppm);
    assert!(has_source_path(
        &bruker_processed,
        Path::new("bruker_without_expno/pdata/1")
    ));

    let format_many = RSpinReader::new()
        .source_formats([
            LoadedSourceFormat::BrukerFid,
            LoadedSourceFormat::BrukerProcessed,
        ])
        .read_relative_to(&base, "bruker_without_expno")?;
    assert_eq!(format_many.len(), 2);

    let vendor = RSpinReader::new()
        .source_vendor("varian")
        .read_relative_to(&base, "varian_1h")?;
    assert_eq!(vendor.len(), 1);
    assert_eq!(
        first_1d(&vendor)?.metadata.nucleus,
        Some(Nucleus::Hydrogen1)
    );

    let vendor_many = RSpinReader::new()
        .source_vendors([LoadedSourceVendor::AgilentVarian])
        .read_relative_to(&base, "varian_1h")?;
    assert_eq!(vendor_many.len(), 1);

    let processed = RSpinReader::new()
        .source_data_kind(LoadedSourceDataKind::Processed)
        .read_relative_to(&base, "bruker_without_expno")?;
    assert_eq!(processed.len(), 1);
    assert_eq!(first_1d(&processed)?.x.unit, Unit::Ppm);

    let raw_and_processed = RSpinReader::new()
        .source_data_kinds([LoadedSourceDataKind::Raw, LoadedSourceDataKind::Processed])
        .read_relative_to(&base, "bruker_without_expno")?;
    assert_eq!(raw_and_processed.len(), 2);

    let raw = RSpinReader::new()
        .raw_sources()
        .read_relative_to(&base, "bruker_without_expno")?;
    assert_eq!(raw.len(), 1);
    assert_eq!(first_1d(&raw)?.x.unit, Unit::Seconds);

    let processed_alias = RSpinReader::new()
        .processed_sources()
        .read_relative_to(&base, "bruker_without_expno")?;
    assert_eq!(processed_alias.len(), 1);
    assert_eq!(first_1d(&processed_alias)?.x.unit, Unit::Ppm);

    let other_alias = RSpinReader::new()
        .other_sources()
        .read(nmrxiv_fixture_root())?;
    assert!(other_alias.has_source_data_kind(LoadedSourceDataKind::Other));
    assert_eq!(
        other_alias.source_data_kind_count(LoadedSourceDataKind::Raw),
        0
    );
    assert_eq!(
        other_alias.source_data_kind_count(LoadedSourceDataKind::Processed),
        0
    );

    let path_filtered = RSpinReader::new()
        .source_path("bruker_without_expno/pdata/1")
        .read_relative_to(&base, "bruker_without_expno")?;
    assert_eq!(path_filtered.len(), 1);
    assert_eq!(first_1d(&path_filtered)?.x.unit, Unit::Ppm);

    let path_many = RSpinReader::new()
        .source_paths(["bruker_without_expno", "bruker_without_expno/pdata/1"])
        .read_relative_to(&base, "bruker_without_expno")?;
    assert_eq!(path_many.len(), 2);

    let runtime_filtered = RSpinReader::new()
        .source(LoadedSourceFilter::vendor("varian"))
        .read_relative_to(&base, "varian_1h")?;
    assert_eq!(runtime_filtered.len(), 1);

    let combined = RSpinReader::new()
        .sources([
            LoadedSourceFilter::vendor("varian"),
            LoadedSourceFilter::path("bruker_without_expno/pdata/1"),
        ])
        .read_many_relative_to(&base, ["varian_1h", "bruker_without_expno"])?;
    assert_eq!(combined.len(), 2);
    assert!(has_source_path(&combined, Path::new("varian_1h")));
    assert!(has_source_path(
        &combined,
        Path::new("bruker_without_expno/pdata/1")
    ));
    Ok(())
}

#[test]
fn reader_named_source_shortcuts_cover_vendor_and_format_workflows() -> anyhow::Result<()> {
    let base = fixture_root();

    let bruker_processed = RSpinReader::new()
        .source_format(LoadedSourceFormat::BrukerProcessed)
        .read_relative_to(&base, "bruker_without_expno")?;
    let bruker_shortcut = RSpinReader::new()
        .bruker()
        .processed()
        .read_relative_to(&base, "bruker_without_expno")?;
    assert_eq!(bruker_shortcut.summary(), bruker_processed.summary());

    let varian = RSpinReader::new()
        .source_vendor("varian")
        .read_relative_to(&base, "varian_1h")?;
    let varian_shortcut = RSpinReader::new()
        .varian()
        .read_relative_to(&base, "varian_1h")?;
    assert_eq!(varian_shortcut.summary(), varian.summary());

    let agilent_shortcut = RSpinReader::new()
        .agilent()
        .read_relative_to(&base, "varian_1h")?;
    assert_eq!(agilent_shortcut.summary(), varian.summary());

    let mixed = nmrxiv_fixture_root();
    let jeol_shortcut = RSpinReader::new().jeol().read_relative_to(&mixed, "jeol")?;
    assert_eq!(
        jeol_shortcut.source_vendor_count(LoadedSourceVendor::Jeol),
        3
    );

    let jcamp_shortcut = RSpinReader::new()
        .jcamp()
        .one_d()
        .read_relative_to(&mixed, "jcamp")?;
    assert_eq!(
        jcamp_shortcut.source_format_count(LoadedSourceFormat::JcampDx),
        2
    );
    assert_eq!(
        RSpinReader::new()
            .jcamp_dx()
            .one_d()
            .read_relative_to(&mixed, "jcamp")?
            .summary(),
        jcamp_shortcut.summary()
    );
    Ok(())
}

#[test]
fn reader_short_source_metadata_read_aliases_match_long_methods() -> anyhow::Result<()> {
    let root = nmrxiv_fixture_root();

    let jcamp = RSpinReader::new().read_by_format(&root, LoadedSourceFormat::JcampDx)?;
    assert_eq!(
        jcamp.summary(),
        RSpinReader::new()
            .read_by_source_format(&root, "jcamp dx")?
            .summary()
    );

    let jeol =
        RSpinReader::new().read_by_vendor_relative_to(&root, "jeol", LoadedSourceVendor::Jeol)?;
    assert_eq!(
        jeol.summary(),
        RSpinReader::new()
            .read_by_source_vendor_relative_to(&root, "jeol", "jeol")?
            .summary()
    );

    let raw_summary =
        RSpinReader::new().read_summary_by_data_kind(&root, LoadedSourceDataKind::Raw)?;
    assert_eq!(
        raw_summary,
        RSpinReader::new().read_summary_by_source_data_kind(&root, LoadedSourceDataKind::Raw,)?
    );

    let strict_jeol = RSpinReader::new().read_summary_strict_by_vendor_relative_to(
        &root,
        "jeol",
        LoadedSourceVendor::Jeol,
    )?;
    assert_eq!(
        strict_jeol,
        RSpinReader::new()
            .read_summary_strict_by_source_vendor_relative_to(&root, "jeol", "jeol")?
    );

    let strict_formats = RSpinReader::new().read_many_strict_by_formats_relative_to(
        &root,
        ["jcamp", "jeol"],
        [LoadedSourceFormat::JcampDx, LoadedSourceFormat::JeolJdf],
    )?;
    assert_eq!(
        strict_formats.summary(),
        RSpinReader::new()
            .read_many_strict_by_source_formats_relative_to(
                &root,
                ["jcamp", "jeol"],
                ["jdx", "jdf"],
            )?
            .summary()
    );
    Ok(())
}

#[test]
fn reader_source_path_prefix_aliases_cover_directory_filters() -> anyhow::Result<()> {
    let base = fixture_root();

    let path_prefix = RSpinReader::new()
        .source_path_prefix("bruker_without_expno/pdata")
        .read_relative_to(&base, "bruker_without_expno")?;
    assert_eq!(path_prefix.len(), 1);
    assert_eq!(first_1d(&path_prefix)?.x.unit, Unit::Ppm);
    assert!(has_source_path(
        &path_prefix,
        Path::new("bruker_without_expno/pdata/1")
    ));

    let path_prefix_many = RSpinReader::new()
        .source_path_prefixes(["jcamp", "jeol"])
        .read(nmrxiv_fixture_root())?;
    assert_eq!(path_prefix_many.len(), 5);
    assert_eq!(
        path_prefix_many.source_format_count(LoadedSourceFormat::BrukerFid),
        0
    );
    assert_eq!(
        path_prefix_many.source_format_count(LoadedSourceFormat::BrukerSer),
        0
    );
    assert!(path_prefix_many.has_source(LoadedSourceFilter::path_prefix("jcamp")));
    assert!(path_prefix_many.has_source(LoadedSourceFilter::path_prefix("jeol")));
    Ok(())
}

#[test]
fn reader_typed_source_filters_compose_in_chains() -> anyhow::Result<()> {
    let base = fixture_root();

    let raw_bruker = RSpinReader::new()
        .source_vendor("bruker")
        .raw_sources()
        .read(&base)?;
    assert_eq!(raw_bruker.len(), 1);
    assert_eq!(
        raw_bruker.source_format_count(LoadedSourceFormat::BrukerFid),
        1
    );
    assert_eq!(
        raw_bruker.source_format_count(LoadedSourceFormat::AgilentFid),
        0
    );
    assert!(has_source_path(
        &raw_bruker,
        Path::new("bruker_without_expno")
    ));

    let raw_bruker_prefix = RSpinReader::new()
        .source_path_prefix("bruker_without_expno")
        .raw_sources()
        .read(&base)?;
    assert_eq!(raw_bruker_prefix.len(), 1);
    assert!(has_source_path(
        &raw_bruker_prefix,
        Path::new("bruker_without_expno")
    ));
    assert_eq!(
        raw_bruker_prefix.source_format_count(LoadedSourceFormat::AgilentFid),
        0
    );

    let cleared_kind = RSpinReader::new()
        .source_vendor("bruker")
        .raw_sources()
        .all_source_data_kinds()
        .read(&base)?;
    assert_eq!(cleared_kind.len(), 2);
    assert_eq!(
        cleared_kind.source_vendor_count(LoadedSourceVendor::Bruker),
        2
    );
    assert_eq!(
        cleared_kind.source_vendor_count(LoadedSourceVendor::AgilentVarian),
        0
    );

    let cleared_generic_format = RSpinReader::new()
        .source(LoadedSourceFilter::vendor("bruker"))
        .all_source_formats()
        .read(&base)?;
    assert_eq!(cleared_generic_format.len(), 3);
    assert_eq!(
        cleared_generic_format.source_vendor_count(LoadedSourceVendor::AgilentVarian),
        1
    );
    assert_eq!(
        cleared_generic_format.source_vendor_count(LoadedSourceVendor::Bruker),
        2
    );
    Ok(())
}

#[test]
fn generic_source_path_filters_skip_excluded_selected_paths_in_strict_mode() -> anyhow::Result<()> {
    let base = fixture_root();

    let bundle = RSpinReader::new()
        .sources([LoadedSourceFilter::path("bruker_without_expno/pdata/1")])
        .strict()
        .read_paths_relative_to(&base, ["empty_jcamp/empty.jdx", "bruker_without_expno"])?;
    assert_eq!(bundle.len(), 1);
    assert!(bundle.warnings().is_empty());
    assert!(bundle.has_source_path(Path::new("bruker_without_expno/pdata/1")));
    assert_eq!(
        bundle.source_format_count(LoadedSourceFormat::BrukerProcessed),
        1
    );

    let prefix_filtered = RSpinReader::new()
        .sources([LoadedSourceFilter::path_prefix(
            "bruker_without_expno/pdata",
        )])
        .strict()
        .read_paths_relative_to(&base, ["varian_1h", "bruker_without_expno"])?;
    assert_eq!(prefix_filtered.len(), 1);
    assert!(prefix_filtered.warnings().is_empty());
    assert!(prefix_filtered.has_source_path(Path::new("bruker_without_expno/pdata/1")));

    Ok(())
}

#[test]
fn mixed_generic_source_filters_preflight_excluded_vendor_candidates() -> anyhow::Result<()> {
    let root = temp_dir("mixed-generic-source-filter-preflight")?;
    let bruker = fixture_root().join("bruker_without_expno");

    let good_bruker = root.join("good_bruker");
    fs::create_dir_all(&good_bruker)?;
    fs::copy(bruker.join("fid"), good_bruker.join("fid"))?;
    fs::copy(bruker.join("acqus"), good_bruker.join("acqus"))?;

    let bad_varian = root.join("bad_varian");
    fs::create_dir_all(&bad_varian)?;
    fs::write(bad_varian.join("fid"), [1_u8, 2, 3])?;
    fs::write(bad_varian.join("procpar"), "not a valid procpar file")?;

    let bundle = RSpinReader::new()
        .sources([
            LoadedSourceFilter::path_prefix("good_bruker"),
            LoadedSourceFilter::format(LoadedSourceFormat::JcampDx),
        ])
        .strict()
        .read_paths_relative_to(&root, ["bad_varian/fid", "good_bruker"])?;
    assert_eq!(bundle.len(), 1);
    assert!(bundle.warnings().is_empty());
    assert!(bundle.has_source_path(Path::new("good_bruker")));
    assert_eq!(bundle.source_format_count(LoadedSourceFormat::BrukerFid), 1);

    let bad_selected = RSpinReader::new()
        .sources([
            LoadedSourceFilter::path_prefix("bad_varian"),
            LoadedSourceFilter::format(LoadedSourceFormat::JcampDx),
        ])
        .strict()
        .read_paths_relative_to(&root, ["bad_varian/fid"]);
    let Err(error) = bad_selected else {
        remove_dir(root)?;
        anyhow::bail!("selected malformed Agilent candidate should fail in strict mode");
    };
    assert!(error.to_string().contains("Agilent"));

    remove_dir(root)?;
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
fn source_data_kind_filters_skip_excluded_vendor_candidates_before_reading() -> anyhow::Result<()> {
    let root = temp_dir("data-kind-routing")?;
    let bruker = fixture_root().join("bruker_without_expno");
    fs::copy(bruker.join("fid"), root.join("fid"))?;
    fs::copy(bruker.join("acqus"), root.join("acqus"))?;

    let processed = root.join("pdata/1");
    fs::create_dir_all(&processed)?;
    fs::write(processed.join("procs"), "not a valid Bruker parameter file")?;
    fs::write(processed.join("1r"), [1_u8, 2, 3])?;

    let raw = RSpinReader::new().raw_sources().strict().read_path(&root)?;
    assert_eq!(raw.len(), 1);
    assert_eq!(first_1d(&raw)?.x.unit, Unit::Seconds);
    assert_eq!(raw.source_format_count(LoadedSourceFormat::BrukerFid), 1);
    assert_eq!(
        raw.source_format_count(LoadedSourceFormat::BrukerProcessed),
        0
    );
    assert!(raw.warnings().is_empty());

    let raw_runtime = RSpinReader::new()
        .sources([LoadedSourceFilter::raw()])
        .strict()
        .read_path(&root)?;
    assert_eq!(raw_runtime.len(), 1);
    assert_eq!(
        raw_runtime.source_format_count(LoadedSourceFormat::BrukerFid),
        1
    );
    assert!(raw_runtime.warnings().is_empty());

    let processed_result = RSpinReader::new()
        .processed_sources()
        .strict()
        .read_path(&root);
    let Err(error) = processed_result else {
        remove_dir(root)?;
        anyhow::bail!(
            "malformed processed candidate should fail when processed sources are loaded"
        );
    };
    assert!(error.to_string().contains("Bruker"));

    remove_dir(root)?;
    Ok(())
}

#[test]
fn source_format_filters_skip_excluded_direct_vendor_files_before_reading() -> anyhow::Result<()> {
    let root = temp_dir("source-format-routing")?;
    let bruker = fixture_root().join("bruker_without_expno");

    let good_bruker = root.join("good_bruker");
    fs::create_dir_all(&good_bruker)?;
    fs::copy(bruker.join("fid"), good_bruker.join("fid"))?;
    fs::copy(bruker.join("acqus"), good_bruker.join("acqus"))?;

    let bad_varian = root.join("bad_varian");
    fs::create_dir_all(&bad_varian)?;
    fs::write(bad_varian.join("fid"), [1_u8, 2, 3])?;
    fs::write(bad_varian.join("procpar"), "not a valid procpar file")?;

    let bruker_only = RSpinReader::new()
        .source_vendor("bruker")
        .strict()
        .read_paths([bad_varian.join("fid"), good_bruker.join("fid")])?;
    assert_eq!(bruker_only.len(), 1);
    assert_eq!(
        bruker_only.source_format_count(LoadedSourceFormat::BrukerFid),
        1
    );
    assert_eq!(
        bruker_only.source_format_count(LoadedSourceFormat::AgilentFid),
        0
    );
    assert!(bruker_only.warnings().is_empty());

    let runtime_filter = RSpinReader::new()
        .sources([LoadedSourceFilter::vendor("bruker")])
        .strict()
        .read_paths([bad_varian.join("fid"), good_bruker.join("fid")])?;
    assert_eq!(runtime_filter.len(), 1);
    assert_eq!(
        runtime_filter.source_format_count(LoadedSourceFormat::BrukerFid),
        1
    );
    assert!(runtime_filter.warnings().is_empty());

    let agilent_result = RSpinReader::new()
        .source_vendor("agilent")
        .strict()
        .read_path(bad_varian.join("fid"));
    let Err(error) = agilent_result else {
        remove_dir(root)?;
        anyhow::bail!("malformed Agilent candidate should fail when Agilent sources are loaded");
    };
    assert!(error.to_string().contains("Agilent"));

    remove_dir(root)?;
    Ok(())
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
fn free_strict_bundle_loader_helpers_abort_on_bad_candidates() -> anyhow::Result<()> {
    let base = fixture_root();

    let single = load_spectra_strict(base.join("varian_1h"))?;
    assert_eq!(single.len(), 1);
    assert!(single.warnings().is_empty());

    let relative = load_spectra_strict_relative_to(&base, "varian_1h")?;
    assert_eq!(relative.len(), 1);
    assert!(relative.has_source_path("varian_1h"));

    let many = load_spectra_many_strict([base.join("varian_1h")])?;
    assert_eq!(many.len(), 1);
    assert!(many.warnings().is_empty());

    let many_relative = load_spectra_many_strict_relative_to(&base, ["varian_1h"])?;
    assert_eq!(many_relative.len(), 1);
    assert!(many_relative.has_source_path("varian_1h"));

    let bad_file = load_spectra_strict(base.join("empty_jcamp/empty.jdx"));
    let Err(error) = bad_file else {
        anyhow::bail!("strict loading should reject malformed candidates");
    };
    assert!(error.to_string().contains("missing XYDATA values"));
    Ok(())
}

#[test]
fn bundle_summary_loader_methods_cover_common_paths() -> anyhow::Result<()> {
    let base = fixture_root();
    let reader = RSpinReader::new();

    let direct = reader.read_summary_path(base.join("varian_1h"))?;
    assert_eq!(direct.spectra(), 1);
    assert_eq!(direct.spectra_1d(), 1);
    assert_eq!(
        direct.source_vendor_count(LoadedSourceVendor::AgilentVarian),
        1
    );

    let alias = reader.read_summary(base.join("varian_1h"))?;
    assert_eq!(alias, direct);

    let relative = reader.read_summary_relative_to(&base, "bruker_without_expno")?;
    assert_eq!(relative.spectra(), 2);
    assert_eq!(
        relative.source_format_count(LoadedSourceFormat::BrukerFid),
        1
    );
    assert_eq!(
        relative.source_path_count("bruker_without_expno/pdata/1"),
        1
    );

    let many =
        reader.read_summary_many([base.join("varian_1h"), base.join("bruker_without_expno")])?;
    assert_eq!(many.spectra(), 3);
    assert_eq!(many.source_vendor_count(LoadedSourceVendor::Bruker), 2);

    let many_relative =
        reader.read_summary_many_relative_to(&base, ["varian_1h", "bruker_without_expno"])?;
    assert_eq!(many_relative.spectra(), 3);
    assert_eq!(
        many_relative.source_path_count("bruker_without_expno/pdata/1"),
        1
    );
    Ok(())
}

#[test]
fn free_bundle_summary_helpers_share_loader_behavior() -> anyhow::Result<()> {
    let base = fixture_root();

    assert_eq!(load_spectra_summary(base.join("varian_1h"))?.spectra(), 1);
    assert_eq!(
        load_spectra_summary_relative_to(&base, "bruker_without_expno")?.spectra(),
        2
    );
    assert_eq!(
        load_spectra_many_summary([base.join("varian_1h"), base.join("bruker_without_expno")])?
            .spectra(),
        3
    );
    assert_eq!(
        load_spectra_many_summary_relative_to(&base, ["varian_1h", "bruker_without_expno"])?
            .spectra(),
        3
    );

    assert_eq!(
        load_spectra_summary_strict_relative_to(&base, "varian_1h")?.spectra(),
        1
    );
    assert_eq!(
        load_spectra_many_summary_strict_relative_to(&base, ["varian_1h"])?.spectra(),
        1
    );
    assert_eq!(
        load_spectra_many_summary_strict([base.join("varian_1h")])?.spectra(),
        1
    );

    let malformed = load_spectra_summary_strict(base.join("empty_jcamp/empty.jdx"));
    let Err(error) = malformed else {
        anyhow::bail!("strict summary loading should reject malformed candidates");
    };
    assert!(error.to_string().contains("missing XYDATA values"));
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

    let (direct_with_source, source) = load_spectrum_1d_with_source(&fixture)?;
    assert_eq!(direct_with_source.len(), direct.len());
    assert_eq!(source.format, "agilent_fid");
    assert_eq!(source.path.as_deref(), Some(Path::new("varian_1h")));

    let via_reader = RSpinReader::new().read_1d(&fixture)?;
    assert_eq!(via_reader.len(), direct.len());

    let (via_reader_with_source, source) = RSpinReader::new().read_1d_with_source(&fixture)?;
    assert_eq!(via_reader_with_source.len(), direct.len());
    assert_eq!(source.format, "agilent_fid");
    assert_eq!(source.path.as_deref(), Some(Path::new("varian_1h")));

    let (relative_with_source, source) =
        load_spectrum_1d_with_source_relative_to(fixture_root(), "varian_1h")?;
    assert_eq!(relative_with_source.len(), direct.len());
    assert_eq!(source.format, "agilent_fid");
    assert_eq!(source.path.as_deref(), Some(Path::new("varian_1h")));

    let bundle = load_spectra(&fixture)?;
    assert_eq!(bundle.only_1d()?.len(), direct.len());
    let (borrowed, source) = bundle.only_loaded_1d()?;
    assert_eq!(borrowed.len(), direct.len());
    assert_eq!(source.format, "agilent_fid");
    assert_eq!(source.path.as_deref(), Some(Path::new("varian_1h")));
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

    let (owned_with_source, source) = load_spectra(&fixture)?.into_only_loaded_1d()?;
    assert_eq!(owned_with_source.len(), direct.len());
    assert_eq!(source.format, "agilent_fid");
    assert_eq!(source.path.as_deref(), Some(Path::new("varian_1h")));

    let two_d_bundle = load_spectra(nmrxiv_fixture_root().join("bruker_cosy_raw"))?;
    let (borrowed_2d, source) = two_d_bundle.only_loaded_2d()?;
    assert_eq!(borrowed_2d.shape(), (2048, 512));
    assert_eq!(source.format, "bruker_ser");
    assert_eq!(source.path.as_deref(), Some(Path::new("bruker_cosy_raw")));

    let (owned_2d, source) =
        load_spectra(nmrxiv_fixture_root().join("bruker_cosy_raw"))?.into_only_loaded_2d()?;
    assert_eq!(owned_2d.shape(), (2048, 512));
    assert_eq!(source.format, "bruker_ser");
    assert_eq!(source.path.as_deref(), Some(Path::new("bruker_cosy_raw")));

    let (direct_2d, source) =
        load_spectrum_2d_with_source_relative_to(nmrxiv_fixture_root(), "bruker_cosy_raw")?;
    assert_eq!(direct_2d.shape(), (2048, 512));
    assert_eq!(source.format, "bruker_ser");
    assert_eq!(source.path.as_deref(), Some(Path::new("bruker_cosy_raw")));

    let (reader_2d, source) = RSpinReader::new()
        .raw_only()
        .read_2d_with_source(nmrxiv_fixture_root().join("bruker_cosy_raw"))?;
    assert_eq!(reader_2d.shape(), (2048, 512));
    assert_eq!(source.format, "bruker_ser");
    assert_eq!(source.path.as_deref(), Some(Path::new("bruker_cosy_raw")));
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

    let wrong_dimension = RSpinReader::new().read_2d_with_source(&one_d_fixture);
    assert_single_error(
        wrong_dimension.map(|(spectrum, _)| spectrum.shape()),
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
    assert_single_error(
        bundle.only_loaded_1d().map(|(spectrum, _)| spectrum.len()),
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

    let one_d = load_spectrum_1d_paths([fixture_root().join("varian_1h")])?;
    assert_eq!(one_d.len(), 16_384);

    let one_d = load_spectrum_1d_many_relative_to(fixture_root(), ["varian_1h"])?;
    assert_eq!(one_d.len(), 16_384);

    let one_d = load_spectrum_1d_paths_relative_to(fixture_root(), ["varian_1h"])?;
    assert_eq!(one_d.len(), 16_384);

    let one_d = RSpinReader::new().read_1d_paths([
        fixture_root().join("empty_jcamp/empty.jdx"),
        fixture_root().join("varian_1h"),
    ])?;
    assert_eq!(one_d.len(), 16_384);

    let two_d = load_spectrum_2d_many([nmrxiv_fixture_root().join("bruker_cosy_raw")])?;
    assert_eq!(two_d.shape(), (2048, 512));

    let two_d = load_spectrum_2d_paths([nmrxiv_fixture_root().join("bruker_cosy_raw")])?;
    assert_eq!(two_d.shape(), (2048, 512));

    let two_d = load_spectrum_2d_many_relative_to(nmrxiv_fixture_root(), ["bruker_cosy_raw"])?;
    assert_eq!(two_d.shape(), (2048, 512));

    let two_d = load_spectrum_2d_paths_relative_to(nmrxiv_fixture_root(), ["bruker_cosy_raw"])?;
    assert_eq!(two_d.shape(), (2048, 512));

    let two_d = RSpinReader::new()
        .raw_only()
        .read_2d_many_relative_to(nmrxiv_fixture_root(), ["bruker_cosy_raw"])?;
    assert_eq!(two_d.shape(), (2048, 512));

    let one_d = RSpinReader::new().read_1d_many([
        fixture_root().join("empty_jcamp/empty.jdx"),
        fixture_root().join("varian_1h"),
    ])?;
    assert_eq!(one_d.len(), 16_384);

    let two_d = RSpinReader::new().read_2d_many([nmrxiv_fixture_root().join("bruker_cosy_raw")])?;
    assert_eq!(two_d.shape(), (2048, 512));

    let one_d = RSpinReader::new().read_1d_many_relative_to(fixture_root(), ["varian_1h"])?;
    assert_eq!(one_d.len(), 16_384);

    let wrong_dimension = RSpinReader::new().read_2d_many([fixture_root().join("varian_1h")]);
    assert_single_error(
        wrong_dimension,
        "expected exactly one two-dimensional spectrum",
        "found 1 one-dimensional and 0 two-dimensional spectra",
    )?;

    let ambiguous = RSpinReader::new().read_1d_many([
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
fn exact_single_helpers_support_selected_path_sets_with_sources() -> anyhow::Result<()> {
    let (one_d, source) = load_spectrum_1d_many_with_source([fixture_root().join("varian_1h")])?;
    assert_eq!(one_d.len(), 16_384);
    assert_eq!(source.format, "agilent_fid");
    assert_eq!(source.path.as_deref(), Some(Path::new("varian_1h")));

    let (one_d, source) = load_spectrum_1d_paths_with_source([fixture_root().join("varian_1h")])?;
    assert_eq!(one_d.len(), 16_384);
    assert_eq!(source.format, "agilent_fid");
    assert_eq!(source.path.as_deref(), Some(Path::new("varian_1h")));

    let (one_d, source) =
        load_spectrum_1d_many_with_source_relative_to(fixture_root(), ["varian_1h"])?;
    assert_eq!(one_d.len(), 16_384);
    assert_eq!(source.format, "agilent_fid");
    assert_eq!(source.path.as_deref(), Some(Path::new("varian_1h")));

    let (one_d, source) =
        load_spectrum_1d_paths_with_source_relative_to(fixture_root(), ["varian_1h"])?;
    assert_eq!(one_d.len(), 16_384);
    assert_eq!(source.format, "agilent_fid");
    assert_eq!(source.path.as_deref(), Some(Path::new("varian_1h")));

    let (one_d, source) = RSpinReader::new().read_1d_paths_with_source([
        fixture_root().join("empty_jcamp/empty.jdx"),
        fixture_root().join("varian_1h"),
    ])?;
    assert_eq!(one_d.len(), 16_384);
    assert_eq!(source.format, "agilent_fid");
    assert_eq!(source.path.as_deref(), Some(Path::new("varian_1h")));

    let (one_d, source) = RSpinReader::new().read_1d_many_with_source([
        fixture_root().join("empty_jcamp/empty.jdx"),
        fixture_root().join("varian_1h"),
    ])?;
    assert_eq!(one_d.len(), 16_384);
    assert_eq!(source.format, "agilent_fid");
    assert_eq!(source.path.as_deref(), Some(Path::new("varian_1h")));

    let (one_d, source) =
        RSpinReader::new().read_1d_paths_with_source_relative_to(fixture_root(), ["varian_1h"])?;
    assert_eq!(one_d.len(), 16_384);
    assert_eq!(source.format, "agilent_fid");
    assert_eq!(source.path.as_deref(), Some(Path::new("varian_1h")));

    let (one_d, source) =
        RSpinReader::new().read_1d_many_with_source_relative_to(fixture_root(), ["varian_1h"])?;
    assert_eq!(one_d.len(), 16_384);
    assert_eq!(source.format, "agilent_fid");
    assert_eq!(source.path.as_deref(), Some(Path::new("varian_1h")));

    let (two_d, source) =
        load_spectrum_2d_many_with_source([nmrxiv_fixture_root().join("bruker_cosy_raw")])?;
    assert_eq!(two_d.shape(), (2048, 512));
    assert_eq!(source.format, "bruker_ser");
    assert_eq!(source.path.as_deref(), Some(Path::new("bruker_cosy_raw")));

    let (two_d, source) =
        load_spectrum_2d_paths_with_source([nmrxiv_fixture_root().join("bruker_cosy_raw")])?;
    assert_eq!(two_d.shape(), (2048, 512));
    assert_eq!(source.format, "bruker_ser");
    assert_eq!(source.path.as_deref(), Some(Path::new("bruker_cosy_raw")));

    let (two_d, source) =
        load_spectrum_2d_many_with_source_relative_to(nmrxiv_fixture_root(), ["bruker_cosy_raw"])?;
    assert_eq!(two_d.shape(), (2048, 512));
    assert_eq!(source.format, "bruker_ser");
    assert_eq!(source.path.as_deref(), Some(Path::new("bruker_cosy_raw")));

    let (two_d, source) =
        load_spectrum_2d_paths_with_source_relative_to(nmrxiv_fixture_root(), ["bruker_cosy_raw"])?;
    assert_eq!(two_d.shape(), (2048, 512));
    assert_eq!(source.format, "bruker_ser");
    assert_eq!(source.path.as_deref(), Some(Path::new("bruker_cosy_raw")));

    let (two_d, source) = RSpinReader::new()
        .read_2d_paths_with_source([nmrxiv_fixture_root().join("bruker_cosy_raw")])?;
    assert_eq!(two_d.shape(), (2048, 512));
    assert_eq!(source.format, "bruker_ser");
    assert_eq!(source.path.as_deref(), Some(Path::new("bruker_cosy_raw")));

    let (two_d, source) = RSpinReader::new()
        .read_2d_many_with_source([nmrxiv_fixture_root().join("bruker_cosy_raw")])?;
    assert_eq!(two_d.shape(), (2048, 512));
    assert_eq!(source.format, "bruker_ser");
    assert_eq!(source.path.as_deref(), Some(Path::new("bruker_cosy_raw")));

    let (two_d, source) = RSpinReader::new()
        .raw_only()
        .read_2d_paths_with_source_relative_to(nmrxiv_fixture_root(), ["bruker_cosy_raw"])?;
    assert_eq!(two_d.shape(), (2048, 512));
    assert_eq!(source.format, "bruker_ser");
    assert_eq!(source.path.as_deref(), Some(Path::new("bruker_cosy_raw")));

    let (two_d, source) = RSpinReader::new()
        .raw_only()
        .read_2d_many_with_source_relative_to(nmrxiv_fixture_root(), ["bruker_cosy_raw"])?;
    assert_eq!(two_d.shape(), (2048, 512));
    assert_eq!(source.format, "bruker_ser");
    assert_eq!(source.path.as_deref(), Some(Path::new("bruker_cosy_raw")));

    let ambiguous = RSpinReader::new().read_1d_many_with_source([
        fixture_root().join("varian_1h"),
        fixture_root().join("bruker_without_expno"),
    ]);
    assert_single_error(
        ambiguous.map(|(spectrum, _)| spectrum.len()),
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
fn nmredata_directory_candidates_respect_source_path_filters() -> anyhow::Result<()> {
    let root = temp_dir("nmredata-source-path-filter")?;
    let kept = root.join("kept");
    let skipped = root.join("skipped");
    fs::create_dir_all(&kept)?;
    fs::create_dir_all(&skipped)?;
    fs::copy(
        nmredata_fixture_root().join("ethanol.sdf"),
        kept.join("ethanol.sdf"),
    )?;
    fs::write(
        skipped.join("bad.sdf"),
        ">  <NMREDATA_VERSION>\ninvalid\n\n$$$$\n",
    )?;

    let bundle = RSpinReader::new()
        .source_path("kept/ethanol.sdf")
        .strict()
        .read_path(&root)?;
    assert_eq!(bundle.len(), 0);
    assert_eq!(bundle.molecule_count(), 1);
    assert!(bundle.warnings().is_empty());
    assert_eq!(bundle.molecules()[0].id, "nmredata:kept/ethanol.sdf:1");

    let prefix_bundle = RSpinReader::new()
        .source_path_prefix("kept")
        .strict()
        .read_path(&root)?;
    assert_eq!(prefix_bundle.molecule_count(), 1);
    assert!(prefix_bundle.warnings().is_empty());

    let bad_selected = RSpinReader::new()
        .source_path("skipped/bad.sdf")
        .strict()
        .read_path(&root);
    let Err(error) = bad_selected else {
        remove_dir(root)?;
        anyhow::bail!("selected malformed NMReDATA file should fail in strict mode");
    };
    assert!(error.to_string().contains("NMReDATA"));

    remove_dir(root)?;
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
    assert_eq!(bundle.source_path_count(jcamp_path), 1);
    assert_eq!(bundle.source_path_count("missing"), 0);
    let source_path_counts = bundle.source_path_counts();
    assert_eq!(source_path_counts.len(), bundle.len());
    assert!(source_path_counts.contains(&SourcePathCount::new(jcamp_path, 1)));
    assert!(source_path_counts.contains(&SourcePathCount::new(hsqc_path, 1)));
    let summary = bundle.summary();
    assert_eq!(summary.source_paths, source_path_counts);
    assert_eq!(summary.source_path_count(jcamp_path), 1);
    assert_eq!(summary.source_path_prefix_count("jcamp"), 2);
    assert!(summary.has_source_path(hsqc_path));
    assert!(summary.has_source_path_prefix("jeol"));
    assert!(!summary.has_source_path("missing"));
    assert_eq!(
        summary.source_count(LoadedSourceFilter::path_prefix("jcamp")),
        2
    );
    assert!(summary.has_source(LoadedSourceFilter::path(hsqc_path)));
    assert!(!summary.has_source(LoadedSourceFilter::path("missing")));

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

    let no_sources = RSpinReader::new()
        .without_source_paths()
        .read_path(nmrxiv_fixture_root())?;
    assert!(no_sources.loaded_by_source_path(jcamp_path).is_none());
    assert!(no_sources.source_paths().next().is_none());
    Ok(())
}

#[test]
fn bundle_warning_path_lookup_helpers_find_entries() -> anyhow::Result<()> {
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
    assert_eq!(
        bundle_with_warning.warning_path_counts(),
        vec![WarningPathCount::new("empty_jcamp/empty.jdx", 1)]
    );
    assert_eq!(
        bundle_with_warning.warning_path_count("empty_jcamp/empty.jdx"),
        1
    );
    assert!(bundle_with_warning.has_warning_path("empty_jcamp/empty.jdx"));
    assert!(!bundle_with_warning.has_warning_path("missing"));
    let warning_summary = bundle_with_warning.summary();
    assert_eq!(
        warning_summary.warning_paths,
        vec![WarningPathCount::new("empty_jcamp/empty.jdx", 1)]
    );
    assert_eq!(
        warning_summary.warning_path_count("empty_jcamp/empty.jdx"),
        1
    );
    assert_eq!(warning_summary.warning_path_prefix_count("empty_jcamp"), 1);
    assert!(warning_summary.has_warning_path("empty_jcamp/empty.jdx"));
    assert!(warning_summary.has_warning_path_prefix("empty_jcamp"));
    assert!(!warning_summary.has_warning_path("missing"));
    let warning_messages = bundle_with_warning.warning_messages().collect::<Vec<_>>();
    assert_eq!(warning_messages.len(), 1);
    assert!(warning_messages[0].contains("missing XYDATA values"));
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
    assert_eq!(summary.source_count(LoadedSourceFilter::format("jdx")), 2);
    assert_eq!(
        summary.source_count(LoadedSourceFilter::vendor("bruker")),
        2
    );
    assert!(summary.has_source(LoadedSourceFilter::data_kind(LoadedSourceDataKind::Raw)));
    assert!(!summary.has_source(LoadedSourceFilter::vendor("missing")));
    assert_eq!(summary.source_data_kind_count(LoadedSourceDataKind::Raw), 2);
    assert_eq!(
        summary.source_data_kind_count(LoadedSourceDataKind::Processed),
        0
    );
    assert_eq!(
        summary.source_data_kind_count(LoadedSourceDataKind::Other),
        5
    );
    assert!(summary.has_source_data_kind(LoadedSourceDataKind::Raw));
    assert!(!summary.has_source_data_kind(LoadedSourceDataKind::Processed));

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
fn bundle_generic_source_helpers_filter_entries() -> anyhow::Result<()> {
    let bundle = load_spectra(nmrxiv_fixture_root())?;
    let carbon_path = Path::new("jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx");
    let hsqc_path = Path::new("jeol/myrcene_hsqc_400mhz.jdf");

    assert_eq!(bundle.source_count(LoadedSourceFilter::format("jdx")), 2);
    assert_eq!(bundle.source_count(LoadedSourceFilter::vendor("bruker")), 2);
    assert_eq!(bundle.source_count(LoadedSourceFilter::path(hsqc_path)), 1);
    assert_eq!(
        bundle.source_count(LoadedSourceFilter::path_prefix("jcamp")),
        2
    );
    assert_eq!(
        bundle.source_count(LoadedSourceFilter::path_prefix("jeol")),
        3
    );
    assert!(bundle.has_source(LoadedSourceFilter::path(carbon_path)));
    assert!(bundle.has_source(LoadedSourceFilter::path_prefix("jeol")));
    assert!(!bundle.has_source(LoadedSourceFilter::path("missing.jdx")));

    let jcamp = bundle
        .loaded_by_source(LoadedSourceFilter::format(LoadedSourceFormat::JcampDx))
        .collect::<Vec<_>>();
    assert_eq!(jcamp.len(), 2);
    assert!(jcamp.iter().all(|entry| entry.is_1d()));

    let bruker_1d = bundle
        .loaded_1d_by_source(LoadedSourceFilter::from(LoadedSourceVendor::Bruker))
        .collect::<Vec<_>>();
    assert_eq!(bruker_1d.len(), 1);
    assert_eq!(bruker_1d[0].0.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(bruker_1d[0].1.format(), "bruker_fid");

    let hsqc = bundle
        .loaded_2d_by_source(LoadedSourceFilter::path(hsqc_path))
        .collect::<Vec<_>>();
    assert_eq!(hsqc.len(), 1);
    assert_eq!(hsqc[0].0.shape(), (1024, 32));
    assert_eq!(hsqc[0].1.path(), Some(hsqc_path));

    let jcamp_1d = bundle
        .loaded_1d_by_source(LoadedSourceFilter::path_prefix("jcamp"))
        .collect::<Vec<_>>();
    assert_eq!(jcamp_1d.len(), 2);
    assert!(jcamp_1d.iter().all(|(_, source)| {
        source
            .path()
            .is_some_and(|path| path.starts_with(Path::new("jcamp")))
    }));

    assert_eq!(
        bundle
            .source_paths_for_source(LoadedSourceFilter::vendor("jeol"))
            .collect::<Vec<_>>(),
        vec![
            Path::new("jeol/myrcene_13c_400mhz.jdf"),
            Path::new("jeol/myrcene_1h_400mhz.jdf"),
            Path::new("jeol/myrcene_hsqc_400mhz.jdf")
        ]
    );
    assert_eq!(
        bundle
            .source_paths_for_source(LoadedSourceFilter::path_prefix("jcamp"))
            .collect::<Vec<_>>(),
        vec![
            Path::new("jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx"),
            Path::new("jcamp/myrcene_1h_400mhz_jcamp_dx_6_link.jdx")
        ]
    );
    assert!(
        bundle
            .loaded_by_source(LoadedSourceFilter::vendor("unknown-vendor"))
            .collect::<Vec<_>>()
            .is_empty()
    );
    Ok(())
}

#[test]
fn bundle_source_path_prefix_helpers_filter_entries() -> anyhow::Result<()> {
    let bundle = load_spectra(nmrxiv_fixture_root())?;

    assert_eq!(bundle.source_path_prefix_count("jcamp"), 2);
    assert!(bundle.has_source_path_prefix("jeol"));
    assert!(!bundle.has_source_path_prefix("missing"));
    assert_eq!(
        bundle
            .source_paths_for_path_prefix("jcamp")
            .collect::<Vec<_>>(),
        vec![
            Path::new("jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx"),
            Path::new("jcamp/myrcene_1h_400mhz_jcamp_dx_6_link.jdx")
        ]
    );

    let jcamp = bundle.source_path_prefix_subset("jcamp");
    assert_eq!(jcamp.len(), 2);
    assert_eq!(jcamp.len_1d(), 2);
    assert_eq!(jcamp.len_2d(), 0);

    let jeol_2d = bundle
        .loaded_2d_by_source_path_prefix("jeol")
        .collect::<Vec<_>>();
    assert_eq!(jeol_2d.len(), 1);
    assert_eq!(jeol_2d[0].0.shape(), (1024, 32));

    let jcamp_1d = bundle
        .loaded_1d_by_source_path_prefix("jcamp")
        .collect::<Vec<_>>();
    assert_eq!(jcamp_1d.len(), 2);
    assert!(jcamp_1d.iter().all(|(_, source)| {
        source
            .path()
            .is_some_and(|path| path.starts_with(Path::new("jcamp")))
    }));

    let consumed = bundle.clone().into_source_path_prefix_subset("jeol");
    assert_eq!(consumed.len(), 3);
    assert_eq!(consumed.len_2d(), 1);
    assert_eq!(
        bundle
            .clone()
            .into_loaded_by_source_path_prefix("jcamp")
            .len(),
        2
    );
    assert_eq!(
        bundle
            .clone()
            .into_loaded_1d_by_source_path_prefix("jcamp")
            .len(),
        2
    );
    assert_eq!(
        bundle
            .clone()
            .into_loaded_2d_by_source_path_prefix("jeol")
            .len(),
        1
    );
    assert_eq!(
        bundle
            .clone()
            .into_spectra_1d_by_source_path_prefix("jcamp")
            .len(),
        2
    );
    assert_eq!(
        bundle.into_spectra_2d_by_source_path_prefix("jeol").len(),
        1
    );
    Ok(())
}

#[test]
fn bundle_source_path_prefix_helpers_filter_warnings() -> anyhow::Result<()> {
    let bundle = SpectrumBundleLoader::new()
        .with_source_paths(true)
        .read_path(fixture_root())?;

    let warnings = bundle
        .warnings_for_source_path_prefix("empty_jcamp")
        .collect::<Vec<_>>();
    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0].path(), Some(Path::new("empty_jcamp/empty.jdx")));
    assert_eq!(bundle.warning_path_prefix_count("empty_jcamp"), 1);
    assert!(bundle.has_warning_path_prefix("empty_jcamp"));
    assert!(!bundle.has_warning_path_prefix("missing"));

    let subset = bundle.source_path_prefix_subset("empty_jcamp");
    assert_eq!(subset.len(), 0);
    assert_eq!(subset.warning_count(), 1);

    let missing_subset = bundle.into_source_path_prefix_subset("missing");
    assert_eq!(missing_subset.len(), 0);
    assert_eq!(missing_subset.warning_count(), 0);
    Ok(())
}

#[test]
fn bundle_generic_source_helpers_filter_many_entries() -> anyhow::Result<()> {
    let bundle = load_spectra(nmrxiv_fixture_root())?;
    let carbon_path = Path::new("jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx");
    let hsqc_path = Path::new("jeol/myrcene_hsqc_400mhz.jdf");

    assert_eq!(
        bundle.source_count_by_sources([
            LoadedSourceFilter::format("jdx"),
            LoadedSourceFilter::vendor("bruker"),
            LoadedSourceFilter::vendor("bruker")
        ]),
        4
    );
    assert!(bundle.has_any_source([
        LoadedSourceFilter::vendor("missing"),
        LoadedSourceFilter::path(hsqc_path)
    ]));
    assert_eq!(
        bundle.source_count_by_sources(Vec::<LoadedSourceFilter>::new()),
        bundle.len()
    );
    assert!(bundle.has_any_source(Vec::<LoadedSourceFilter>::new()));

    let selected = bundle
        .loaded_by_sources([
            LoadedSourceFilter::format(LoadedSourceFormat::JcampDx),
            LoadedSourceFilter::from(LoadedSourceVendor::Bruker),
        ])
        .collect::<Vec<_>>();
    assert_eq!(selected.len(), 4);
    assert!(selected.iter().all(|entry| {
        entry.source().is_format(LoadedSourceFormat::JcampDx)
            || entry.source().is_vendor(LoadedSourceVendor::Bruker)
    }));

    let selected_1d = bundle
        .loaded_1d_by_sources([
            LoadedSourceFilter::vendor("jeol"),
            LoadedSourceFilter::path(carbon_path),
        ])
        .collect::<Vec<_>>();
    assert_eq!(selected_1d.len(), 3);
    assert!(
        selected_1d
            .iter()
            .any(|(spectrum, _)| spectrum.metadata.nucleus == Some(Nucleus::Carbon13))
    );

    let selected_2d = bundle
        .loaded_2d_by_sources([
            LoadedSourceFilter::vendor("bruker"),
            LoadedSourceFilter::path(hsqc_path),
        ])
        .collect::<Vec<_>>();
    assert_eq!(selected_2d.len(), 2);

    assert_eq!(
        bundle
            .source_paths_for_sources([
                LoadedSourceFilter::format("jdx"),
                LoadedSourceFilter::path(hsqc_path)
            ])
            .collect::<Vec<_>>(),
        vec![
            Path::new("jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx"),
            Path::new("jcamp/myrcene_1h_400mhz_jcamp_dx_6_link.jdx"),
            Path::new("jeol/myrcene_hsqc_400mhz.jdf")
        ]
    );
    Ok(())
}

#[test]
fn bundle_generic_source_helpers_consume_filtered_entries() -> anyhow::Result<()> {
    let bundle = load_spectra(nmrxiv_fixture_root())?;
    let hsqc_path = Path::new("jeol/myrcene_hsqc_400mhz.jdf");

    let bruker_entries = bundle
        .clone()
        .into_loaded_by_source(LoadedSourceFilter::vendor("bruker"));
    assert_eq!(bruker_entries.len(), 2);
    assert!(
        bruker_entries
            .iter()
            .all(|entry| entry.source().is_vendor(LoadedSourceVendor::Bruker))
    );

    let jeol_1d = bundle
        .clone()
        .into_loaded_1d_by_source(LoadedSourceFilter::from(LoadedSourceVendor::Jeol));
    assert_eq!(jeol_1d.len(), 2);
    assert!(
        jeol_1d
            .iter()
            .all(|(_, source)| source.is_format(LoadedSourceFormat::JeolJdf))
    );
    assert!(
        jeol_1d
            .iter()
            .any(|(spectrum, _)| spectrum.metadata.nucleus == Some(Nucleus::Carbon13))
    );

    let hsqc = bundle
        .clone()
        .into_loaded_2d_by_source(LoadedSourceFilter::path(hsqc_path));
    assert_eq!(hsqc.len(), 1);
    let (hsqc_spectrum, hsqc_source) = hsqc
        .first()
        .ok_or_else(|| anyhow::anyhow!("missing consumed HSQC spectrum"))?;
    assert_eq!(hsqc_spectrum.shape(), (1024, 32));
    assert_eq!(hsqc_source.path(), Some(hsqc_path));

    let jcamp_1d = bundle
        .clone()
        .into_spectra_1d_by_source(LoadedSourceFilter::format("jdx"));
    assert_eq!(jcamp_1d.len(), 2);
    assert!(
        jcamp_1d
            .iter()
            .any(|spectrum| spectrum.metadata.nucleus == Some(Nucleus::Carbon13))
    );

    let bruker_2d = bundle.into_spectra_2d_by_source(LoadedSourceFilter::vendor("bruker"));
    assert_eq!(bruker_2d.len(), 1);
    assert_eq!(bruker_2d[0].shape(), (2048, 512));
    Ok(())
}

#[test]
fn bundle_generic_source_helpers_consume_many_filtered_entries() -> anyhow::Result<()> {
    let bundle = load_spectra(nmrxiv_fixture_root())?;
    let hsqc_path = Path::new("jeol/myrcene_hsqc_400mhz.jdf");

    let selected = bundle.clone().into_loaded_by_sources([
        LoadedSourceFilter::format("jdx"),
        LoadedSourceFilter::vendor("bruker"),
    ]);
    assert_eq!(selected.len(), 4);

    let selected_1d = bundle.clone().into_loaded_1d_by_sources([
        LoadedSourceFilter::format("jdx"),
        LoadedSourceFilter::vendor("bruker"),
    ]);
    assert_eq!(selected_1d.len(), 3);
    assert!(
        selected_1d
            .iter()
            .any(|(_, source)| source.is_format(LoadedSourceFormat::BrukerFid))
    );

    let selected_2d = bundle.clone().into_loaded_2d_by_sources([
        LoadedSourceFilter::vendor("bruker"),
        LoadedSourceFilter::path(hsqc_path),
    ]);
    assert_eq!(selected_2d.len(), 2);

    let spectra_1d = bundle
        .clone()
        .into_spectra_1d_by_sources(Vec::<LoadedSourceFilter>::new());
    assert_eq!(spectra_1d.len(), bundle.len_1d());

    let spectra_2d = bundle.into_spectra_2d_by_sources([
        LoadedSourceFilter::vendor("bruker"),
        LoadedSourceFilter::path(hsqc_path),
    ]);
    assert_eq!(spectra_2d.len(), 2);
    Ok(())
}

#[test]
fn bundle_source_subset_helpers_preserve_relevant_context() -> anyhow::Result<()> {
    let bundle = RSpinReader::new().read_path(fixture_root())?;
    assert_eq!(bundle.len(), 3);
    assert_eq!(bundle.warning_count(), 1);

    let varian_path = bundle.source_subset(LoadedSourceFilter::path("varian_1h"));
    assert_eq!(varian_path.len(), 1);
    assert_eq!(varian_path.warning_count(), 0);
    assert_eq!(
        varian_path.source_vendor_count(LoadedSourceVendor::AgilentVarian),
        1
    );

    let warning_only = bundle.source_subset(LoadedSourceFilter::path("empty_jcamp/empty.jdx"));
    assert_eq!(warning_only.len(), 0);
    assert_eq!(warning_only.warning_count(), 1);
    let warning = warning_only
        .warnings()
        .first()
        .ok_or_else(|| anyhow::anyhow!("missing retained source warning"))?;
    assert!(warning.message().contains("missing XYDATA values"));

    assert_eq!(
        bundle.warning_count_for_source(LoadedSourceFilter::path("empty_jcamp/empty.jdx")),
        1
    );
    assert_eq!(
        bundle.warning_count_for_source(LoadedSourceFilter::path("varian_1h")),
        0
    );
    assert!(bundle.has_warning_for_source(LoadedSourceFilter::path("empty_jcamp/empty.jdx")));
    assert!(!bundle.has_warning_for_source(LoadedSourceFilter::path("varian_1h")));
    assert_eq!(
        bundle.warning_count_for_sources([
            LoadedSourceFilter::path("varian_1h"),
            LoadedSourceFilter::path("empty_jcamp/empty.jdx")
        ]),
        1
    );
    assert_eq!(
        bundle.warning_count_for_sources(Vec::<LoadedSourceFilter>::new()),
        bundle.warning_count()
    );
    assert!(bundle.has_any_warning_for_sources(Vec::<LoadedSourceFilter>::new()));

    let vendor_subset = bundle.source_subset(LoadedSourceFilter::vendor("varian"));
    assert_eq!(vendor_subset.len(), 1);
    assert_eq!(
        vendor_subset.source_vendor_count(LoadedSourceVendor::AgilentVarian),
        1
    );
    assert_eq!(vendor_subset.warning_count(), 1);
    assert_eq!(
        bundle
            .warnings_for_source(LoadedSourceFilter::vendor("varian"))
            .count(),
        bundle.warning_count()
    );

    let all_subset = bundle.source_subset_by_sources(Vec::<LoadedSourceFilter>::new());
    assert_eq!(all_subset, bundle);

    let mixed = load_spectra(nmrxiv_fixture_root())?;
    let jcamp_and_bruker = mixed.clone().into_source_subset_by_sources([
        LoadedSourceFilter::format("jdx"),
        LoadedSourceFilter::vendor("bruker"),
    ]);
    assert_eq!(jcamp_and_bruker.len(), 4);
    assert_eq!(
        jcamp_and_bruker.source_count_by_sources([
            LoadedSourceFilter::format("jdx"),
            LoadedSourceFilter::vendor("bruker")
        ]),
        4
    );
    assert_eq!(mixed.len(), 7);

    let molecule_subset =
        load_spectra(nmredata_fixture_root())?.source_subset(LoadedSourceFilter::vendor("bruker"));
    assert_eq!(molecule_subset.len(), 0);
    assert_eq!(molecule_subset.molecule_count(), 1);
    assert!(!molecule_subset.is_empty());
    Ok(())
}

#[test]
fn bundle_typed_source_subset_helpers_cover_common_filters() -> anyhow::Result<()> {
    let mixed = load_spectra(nmrxiv_fixture_root())?;
    let hsqc_path = Path::new("jeol/myrcene_hsqc_400mhz.jdf");

    let jcamp = mixed.source_format_subset("jdx");
    assert_eq!(jcamp.len(), 2);
    assert_eq!(jcamp.source_format_count(LoadedSourceFormat::JcampDx), 2);

    let bruker = mixed.source_vendor_subset(LoadedSourceVendor::Bruker);
    assert_eq!(bruker.len(), 2);
    assert_eq!(bruker.source_vendor_count(LoadedSourceVendor::Bruker), 2);

    let hsqc = mixed.source_path_subset(hsqc_path);
    assert_eq!(hsqc.len(), 1);
    assert_eq!(first_2d(&hsqc)?.shape(), (1024, 32));

    let jeol = mixed.clone().into_source_vendor_subset("jeol");
    assert_eq!(jeol.len(), 3);
    assert_eq!(jeol.source_vendor_count(LoadedSourceVendor::Jeol), 3);

    let carbon = mixed
        .clone()
        .into_source_format_subset(LoadedSourceFormat::JcampDx);
    assert_eq!(carbon.len(), 2);
    assert_eq!(carbon.len_1d(), 2);

    let hsqc = mixed.into_source_path_subset(hsqc_path);
    assert_eq!(hsqc.len(), 1);
    assert!(hsqc.has_source_path(hsqc_path));

    let warning_subset = RSpinReader::new()
        .read_path(fixture_root())?
        .source_path_subset("empty_jcamp/empty.jdx");
    assert_eq!(warning_subset.len(), 0);
    assert_eq!(warning_subset.warning_count(), 1);
    Ok(())
}

#[test]
fn bundle_source_data_kind_helpers_cover_raw_and_processed_sources() -> anyhow::Result<()> {
    let bundle = load_spectra_relative_to(fixture_root(), "bruker_without_expno")?;

    assert_eq!(bundle.source_data_kind_count(LoadedSourceDataKind::Raw), 1);
    assert_eq!(
        bundle.source_data_kind_count(LoadedSourceDataKind::Processed),
        1
    );
    assert_eq!(
        bundle.source_data_kind_count(LoadedSourceDataKind::Other),
        0
    );
    assert!(bundle.has_source_data_kind(LoadedSourceDataKind::Raw));
    assert!(bundle.loaded_sources().any(LoadedSource::is_raw));
    assert!(bundle.loaded_sources().any(LoadedSource::is_processed));
    assert_eq!(
        bundle.source_data_kinds().collect::<Vec<_>>(),
        vec![LoadedSourceDataKind::Raw, LoadedSourceDataKind::Processed]
    );
    assert_eq!(
        bundle.source_data_kind_counts(),
        vec![
            SourceDataKindCount::new(LoadedSourceDataKind::Raw, 1),
            SourceDataKindCount::new(LoadedSourceDataKind::Processed, 1)
        ]
    );
    let summary = bundle.summary();
    assert_eq!(summary.source_data_kinds, bundle.source_data_kind_counts());
    assert_eq!(
        summary.source_data_kind_counts(),
        bundle.source_data_kind_counts()
    );

    let raw = bundle.raw_source_subset();
    assert_eq!(raw.len(), 1);
    assert_eq!(first_1d(&raw)?.x.unit, Unit::Seconds);
    assert_eq!(
        raw.source_paths_for_data_kind(LoadedSourceDataKind::Raw)
            .collect::<Vec<_>>(),
        vec![Path::new("bruker_without_expno")]
    );
    assert_eq!(
        raw.loaded_by_source_data_kind(LoadedSourceDataKind::Raw)
            .count(),
        1
    );
    assert_eq!(
        raw.loaded_1d_by_source_data_kind(LoadedSourceDataKind::Raw)
            .count(),
        1
    );
    assert_eq!(
        raw.loaded_2d_by_source_data_kind(LoadedSourceDataKind::Raw)
            .count(),
        0
    );

    let processed = bundle.into_processed_source_subset();
    assert_eq!(processed.len(), 1);
    assert_eq!(first_1d(&processed)?.x.unit, Unit::Ppm);
    assert_eq!(
        processed
            .source_paths_for_data_kind(LoadedSourceDataKind::Processed)
            .collect::<Vec<_>>(),
        vec![Path::new("bruker_without_expno/pdata/1")]
    );
    Ok(())
}

#[test]
fn bundle_summary_reconstructs_source_data_kind_counts_from_legacy_json() -> anyhow::Result<()> {
    let summary_json = serde_json::json!({
        "spectra": 3,
        "spectra_1d": 3,
        "spectra_2d": 0,
        "molecules": 0,
        "warnings": 0,
        "source_formats": [
            { "format": "bruker_fid", "count": 1 },
            { "format": "jcamp_dx", "count": 2 }
        ]
    })
    .to_string();
    let summary: SpectrumBundleSummary = serde_json::from_str(&summary_json)?;

    assert!(summary.source_data_kinds.is_empty());
    assert!(summary.source_paths.is_empty());
    assert!(summary.warning_paths.is_empty());
    assert!(summary.source_path_counts().is_empty());
    assert!(summary.warning_path_counts().is_empty());
    assert_eq!(summary.source_path_count("missing"), 0);
    assert_eq!(summary.warning_path_count("missing"), 0);
    assert!(!summary.has_source_path_prefix("missing"));
    assert!(!summary.has_warning_path_prefix("missing"));
    assert_eq!(
        summary.source_data_kind_counts(),
        vec![
            SourceDataKindCount::new(LoadedSourceDataKind::Raw, 1),
            SourceDataKindCount::new(LoadedSourceDataKind::Other, 2)
        ]
    );
    assert_eq!(summary.source_data_kind_count(LoadedSourceDataKind::Raw), 1);
    assert_eq!(
        summary.source_data_kind_count(LoadedSourceDataKind::Other),
        2
    );
    assert!(!summary.has_source_data_kind(LoadedSourceDataKind::Processed));
    Ok(())
}

#[test]
fn loader_can_restrict_source_data_kinds() -> anyhow::Result<()> {
    let raw_loaded = load_spectra_by_source_data_kind_relative_to(
        fixture_root(),
        "bruker_without_expno",
        LoadedSourceDataKind::Raw,
    )?;
    assert_eq!(raw_loaded.len(), 1);
    assert_eq!(first_1d(&raw_loaded)?.x.unit, Unit::Seconds);
    assert_eq!(
        raw_loaded.source_paths().collect::<Vec<_>>(),
        vec![Path::new("bruker_without_expno")]
    );

    let processed_loaded = load_spectra_many_by_source_data_kind_relative_to(
        fixture_root(),
        ["bruker_without_expno"],
        LoadedSourceDataKind::Processed,
    )?;
    assert_eq!(processed_loaded.len(), 1);
    assert_eq!(first_1d(&processed_loaded)?.x.unit, Unit::Ppm);

    let raw_direct = load_spectra_by_source_data_kind(
        fixture_root().join("bruker_without_expno"),
        LoadedSourceDataKind::Raw,
    )?;
    assert_eq!(raw_direct.len(), 1);

    let raw_many = load_spectra_many_by_source_data_kind(
        [fixture_root().join("bruker_without_expno")],
        LoadedSourceDataKind::Raw,
    )?;
    assert_eq!(raw_many.len(), 1);

    let raw_and_processed_direct = load_spectra_by_source_data_kinds(
        fixture_root().join("bruker_without_expno"),
        [LoadedSourceDataKind::Raw, LoadedSourceDataKind::Processed],
    )?;
    assert_eq!(raw_and_processed_direct.len(), 2);

    let raw_and_processed_relative = load_spectra_by_source_data_kinds_relative_to(
        fixture_root(),
        "bruker_without_expno",
        [LoadedSourceDataKind::Raw, LoadedSourceDataKind::Processed],
    )?;
    assert_eq!(raw_and_processed_relative.len(), 2);

    let raw_and_processed_many = load_spectra_many_by_source_data_kinds(
        [fixture_root().join("bruker_without_expno")],
        [LoadedSourceDataKind::Raw, LoadedSourceDataKind::Processed],
    )?;
    assert_eq!(raw_and_processed_many.len(), 2);

    let raw_and_processed_many_relative = load_spectra_many_by_source_data_kinds_relative_to(
        fixture_root(),
        ["bruker_without_expno"],
        [LoadedSourceDataKind::Raw, LoadedSourceDataKind::Processed],
    )?;
    assert_eq!(raw_and_processed_many_relative.len(), 2);

    let unrestricted_data_kinds = load_spectra_by_source_data_kinds(
        fixture_root().join("bruker_without_expno"),
        std::iter::empty::<LoadedSourceDataKind>(),
    )?;
    assert_eq!(unrestricted_data_kinds.len(), 2);

    let other = RSpinReader::new()
        .only_other_sources()
        .read_path(nmrxiv_fixture_root())?;
    assert_eq!(other.len(), 5);
    assert_eq!(other.source_data_kind_count(LoadedSourceDataKind::Other), 5);
    assert_eq!(other.source_data_kind_count(LoadedSourceDataKind::Raw), 0);

    let raw_or_processed = RSpinReader::new()
        .only_source_data_kinds([LoadedSourceDataKind::Raw, LoadedSourceDataKind::Processed])
        .read_path_relative_to(fixture_root(), "bruker_without_expno")?;
    assert_eq!(raw_or_processed.len(), 2);

    let data_kind_filter_cleared = RSpinReader::new()
        .only_raw_sources()
        .all_source_data_kinds()
        .read_path_relative_to(fixture_root(), "bruker_without_expno")?;
    assert_eq!(data_kind_filter_cleared.len(), 2);

    let raw_from_filter = load_spectra_by_source(
        fixture_root().join("bruker_without_expno"),
        LoadedSourceFilter::data_kind(LoadedSourceDataKind::Raw),
    )?;
    assert_eq!(raw_from_filter.len(), 1);

    let processed_by_generic_filter = RSpinReader::new()
        .only_source(LoadedSourceDataKind::Processed)
        .read_path_relative_to(fixture_root(), "bruker_without_expno")?;
    assert_eq!(processed_by_generic_filter.len(), 1);

    let raw_by_convenience_filter = load_spectra_by_sources(
        fixture_root().join("bruker_without_expno"),
        [LoadedSourceFilter::raw()],
    )?;
    assert_eq!(raw_by_convenience_filter.len(), 1);

    let raw_exact = RSpinReader::new().read_1d_by_source_relative_to(
        fixture_root(),
        "bruker_without_expno",
        LoadedSourceDataKind::Raw,
    )?;
    assert_eq!(raw_exact.x.unit, Unit::Seconds);
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
fn loader_can_restrict_source_paths() -> anyhow::Result<()> {
    let carbon_path = Path::new("jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx");
    let hsqc_path = Path::new("jeol/myrcene_hsqc_400mhz.jdf");

    let carbon = RSpinReader::new()
        .only_source_path(carbon_path)
        .read_path(nmrxiv_fixture_root())?;
    assert_eq!(carbon.len(), 1);
    assert_eq!(carbon.len_1d(), 1);
    assert_eq!(first_1d(&carbon)?.metadata.nucleus, Some(Nucleus::Carbon13));
    assert!(carbon.has_source_path(carbon_path));

    let selected = RSpinReader::new()
        .only_source_paths([Path::new("bruker_1h_raw"), hsqc_path])
        .read_path(nmrxiv_fixture_root())?;
    assert_eq!(selected.len(), 2);
    assert_eq!(selected.len_1d(), 1);
    assert_eq!(selected.len_2d(), 1);
    assert_eq!(selected.source_vendor_count(LoadedSourceVendor::Bruker), 1);
    assert_eq!(selected.source_vendor_count(LoadedSourceVendor::Jeol), 1);
    assert!(selected.has_source_path(Path::new("bruker_1h_raw")));
    assert!(selected.has_source_path(hsqc_path));

    let hidden_sources = RSpinReader::new()
        .only_source_path(carbon_path)
        .without_source_paths()
        .read_path(nmrxiv_fixture_root())?;
    assert_eq!(hidden_sources.len(), 1);
    assert_eq!(
        first_1d(&hidden_sources)?.metadata.nucleus,
        Some(Nucleus::Carbon13)
    );
    assert_eq!(hidden_sources.source_paths().count(), 0);

    let cleared = RSpinReader::new()
        .only_source_path(carbon_path)
        .all_source_paths()
        .read_path(nmrxiv_fixture_root())?;
    assert_eq!(cleared.len(), 7);

    let prefix_cleared = RSpinReader::new()
        .only_source_path_prefix("jeol")
        .all_source_paths()
        .read_path(nmrxiv_fixture_root())?;
    assert_eq!(prefix_cleared.len(), 7);

    let runtime_path_cleared = RSpinReader::new()
        .only_sources([
            LoadedSourceFilter::path_prefix("jeol"),
            LoadedSourceFilter::data_kind(LoadedSourceDataKind::Raw),
        ])
        .all_source_paths()
        .read_path(nmrxiv_fixture_root())?;
    assert_eq!(runtime_path_cleared.len(), 2);
    assert_eq!(
        runtime_path_cleared.source_data_kind_count(LoadedSourceDataKind::Raw),
        2
    );
    assert_eq!(
        runtime_path_cleared.source_vendor_count(LoadedSourceVendor::Jeol),
        0
    );

    let filtered_out = RSpinReader::new()
        .only_source_path("missing.jdx")
        .read_path(nmrxiv_fixture_root());
    let Err(error) = filtered_out else {
        anyhow::bail!("missing source-path filter should leave no readable spectra");
    };
    assert!(error.to_string().contains("no readable bundle data found"));
    Ok(())
}

#[test]
fn dimension_source_metadata_helpers_load_matching_bundles() -> anyhow::Result<()> {
    let mixed = nmrxiv_fixture_root();

    let jcamp_1d = io::load_spectra_1d_by_source_format(&mixed, "jdx")?;
    assert_eq!(jcamp_1d.len_1d(), 2);
    assert_eq!(jcamp_1d.len_2d(), 0);
    assert_eq!(jcamp_1d.source_format_count(LoadedSourceFormat::JcampDx), 2);

    let one_d_formats = io::load_spectra_1d_by_source_formats(
        &mixed,
        [LoadedSourceFormat::JcampDx, LoadedSourceFormat::JeolJdf],
    )?;
    assert_eq!(one_d_formats.len_1d(), 4);
    assert_eq!(
        one_d_formats.source_format_count(LoadedSourceFormat::JeolJdf),
        2
    );

    let jeol_1d = io::load_spectra_1d_by_source_vendor(&mixed, "jeol")?;
    assert_eq!(jeol_1d.len_1d(), 2);
    assert_eq!(jeol_1d.source_vendor_count(LoadedSourceVendor::Jeol), 2);

    let raw_1d = io::load_spectra_1d_by_source_data_kind(&mixed, LoadedSourceDataKind::Raw)?;
    assert_eq!(raw_1d.len_1d(), 1);
    assert_eq!(raw_1d.source_format_count(LoadedSourceFormat::BrukerFid), 1);

    let jeol_2d = io::load_spectra_2d_by_source_format(&mixed, LoadedSourceFormat::JeolJdf)?;
    assert_eq!(jeol_2d.len_1d(), 0);
    assert_eq!(jeol_2d.len_2d(), 1);
    assert_eq!(jeol_2d.source_format_count(LoadedSourceFormat::JeolJdf), 1);

    let two_d_vendors = io::load_spectra_2d_by_source_vendors(
        &mixed,
        [LoadedSourceVendor::Bruker, LoadedSourceVendor::Jeol],
    )?;
    assert_eq!(two_d_vendors.len_2d(), 2);
    assert_eq!(
        two_d_vendors.source_vendor_count(LoadedSourceVendor::Bruker),
        1
    );
    assert_eq!(
        two_d_vendors.source_vendor_count(LoadedSourceVendor::Jeol),
        1
    );

    let raw_2d = RSpinReader::new()
        .read_bundle_2d_by_source_data_kinds(&mixed, [LoadedSourceDataKind::Raw])?;
    assert_eq!(raw_2d.len_2d(), 1);
    assert_eq!(raw_2d.source_format_count(LoadedSourceFormat::BrukerSer), 1);
    Ok(())
}

#[test]
fn dimension_source_metadata_summary_helpers_match_loaded_bundles() -> anyhow::Result<()> {
    let mixed = nmrxiv_fixture_root();

    let jcamp_1d = io::load_spectra_1d_by_source_format(&mixed, LoadedSourceFormat::JcampDx)?;
    assert_eq!(
        io::load_spectra_1d_summary_by_source_format(&mixed, "jdx")?,
        jcamp_1d.summary()
    );

    let jeol_1d = RSpinReader::new().read_bundle_1d_by_source_vendor(&mixed, "jeol")?;
    assert_eq!(
        io::load_spectra_1d_summary_by_source_vendors(&mixed, [LoadedSourceVendor::Jeol])?,
        jeol_1d.summary()
    );

    let raw_1d = io::load_spectra_1d_by_source_data_kind(&mixed, LoadedSourceDataKind::Raw)?;
    assert_eq!(
        io::load_spectra_1d_summary_by_source_data_kinds(&mixed, [LoadedSourceDataKind::Raw])?,
        raw_1d.summary()
    );

    let bruker_2d = io::load_spectra_2d_by_source_vendor(&mixed, LoadedSourceVendor::Bruker)?;
    assert_eq!(
        RSpinReader::new().read_bundle_2d_summary_by_source_vendor(&mixed, "bruker")?,
        bruker_2d.summary()
    );

    let two_d_formats = io::load_spectra_2d_by_source_formats(
        &mixed,
        [LoadedSourceFormat::BrukerSer, LoadedSourceFormat::JeolJdf],
    )?;
    assert_eq!(
        io::load_spectra_2d_summary_by_source_formats(
            &mixed,
            [LoadedSourceFormat::BrukerSer, LoadedSourceFormat::JeolJdf],
        )?,
        two_d_formats.summary()
    );

    let raw_2d = io::load_spectra_2d_by_source_data_kind(&mixed, LoadedSourceDataKind::Raw)?;
    assert_eq!(
        io::load_spectra_2d_summary_by_source_data_kind(&mixed, LoadedSourceDataKind::Raw)?,
        raw_2d.summary()
    );
    Ok(())
}

#[test]
fn dimension_source_metadata_relative_helpers_anchor_source_paths() -> anyhow::Result<()> {
    let mixed = nmrxiv_fixture_root();

    let jcamp_1d = io::load_spectra_1d_by_source_format_relative_to(&mixed, "jcamp", "jcamp")?;
    assert_eq!(jcamp_1d.len_1d(), 2);
    assert!(jcamp_1d.has_source_path(Path::new("jcamp/myrcene_1h_400mhz_jcamp_dx_6_link.jdx")));

    let jeol_1d = io::load_spectra_1d_by_source_vendor_relative_to(&mixed, "jeol", "jeol")?;
    assert_eq!(jeol_1d.len_1d(), 2);
    assert_eq!(jeol_1d.source_vendor_count(LoadedSourceVendor::Jeol), 2);

    let raw_1d = RSpinReader::new().read_bundle_1d_by_source_data_kind_relative_to(
        &mixed,
        "bruker_1h_raw",
        LoadedSourceDataKind::Raw,
    )?;
    assert_eq!(raw_1d.len_1d(), 1);
    assert!(raw_1d.has_source_path("bruker_1h_raw"));

    let bruker_2d =
        io::load_spectra_2d_by_source_vendor_relative_to(&mixed, "bruker_cosy_raw", "bruker")?;
    assert_eq!(bruker_2d.len_2d(), 1);
    assert!(bruker_2d.has_source_path("bruker_cosy_raw"));

    assert_eq!(
        io::load_spectra_2d_summary_by_source_format_relative_to(
            &mixed,
            "bruker_cosy_raw",
            LoadedSourceFormat::BrukerSer,
        )?,
        bruker_2d.summary()
    );
    Ok(())
}

#[test]
fn strict_dimension_source_metadata_helpers_match_strict_loader_composition() -> anyhow::Result<()>
{
    let mixed = nmrxiv_fixture_root();
    let sources = RSpinReader::new().discover_path(&mixed)?;

    let jcamp_1d = io::load_spectra_1d_strict_by_source_format(&mixed, "jdx")?;
    assert_eq!(jcamp_1d.len_1d(), 2);
    assert_eq!(jcamp_1d.len_2d(), 0);
    assert_eq!(jcamp_1d.source_format_count(LoadedSourceFormat::JcampDx), 2);
    assert_eq!(
        jcamp_1d.summary(),
        RSpinReader::new()
            .strict()
            .read_discovered_bundle_1d_by_source_format(
                &mixed,
                &sources,
                LoadedSourceFormat::JcampDx,
            )?
            .summary()
    );

    let jeol_1d_summary = RSpinReader::new()
        .read_bundle_1d_summary_strict_by_source_vendor_relative_to(
            &mixed,
            "jeol/myrcene_1h_400mhz.jdf",
            LoadedSourceVendor::Jeol,
        )?;
    assert_eq!(jeol_1d_summary.spectra_1d(), 1);
    assert_eq!(jeol_1d_summary.spectra_2d(), 0);
    assert_eq!(
        jeol_1d_summary,
        RSpinReader::new()
            .strict()
            .read_discovered_bundle_1d_summary_by_source_paths(
                &mixed,
                &sources,
                ["jeol/myrcene_1h_400mhz.jdf"],
            )?
    );

    let raw_2d = io::load_spectra_2d_strict_by_source_data_kind(&mixed, LoadedSourceDataKind::Raw)?;
    assert_eq!(raw_2d.len_1d(), 0);
    assert_eq!(raw_2d.len_2d(), 1);
    assert_eq!(raw_2d.source_format_count(LoadedSourceFormat::BrukerSer), 1);
    assert_eq!(
        raw_2d.summary(),
        RSpinReader::new()
            .strict()
            .read_discovered_bundle_2d_by_source_data_kind(
                &mixed,
                &sources,
                LoadedSourceDataKind::Raw,
            )?
            .summary()
    );

    let two_d_summary = io::load_spectra_2d_summary_strict_by_source_formats_relative_to(
        &mixed,
        "bruker_cosy_raw",
        [LoadedSourceFormat::BrukerSer],
    )?;
    assert_eq!(two_d_summary.spectra_1d(), 0);
    assert_eq!(two_d_summary.spectra_2d(), 1);
    assert_eq!(
        two_d_summary,
        RSpinReader::new()
            .strict()
            .read_discovered_bundle_2d_summary_by_source_paths(
                &mixed,
                &sources,
                ["bruker_cosy_raw"]
            )?
    );

    let jeol_2d_summary = RSpinReader::new()
        .read_bundle_2d_summary_strict_by_source_format_relative_to(
            &mixed,
            "jeol/myrcene_hsqc_400mhz.jdf",
            "jdf",
        )?;
    assert_eq!(jeol_2d_summary.spectra_1d(), 0);
    assert_eq!(jeol_2d_summary.spectra_2d(), 1);
    assert_eq!(
        jeol_2d_summary.source_format_count(LoadedSourceFormat::JeolJdf),
        1
    );
    Ok(())
}

#[test]
fn strict_dimension_source_metadata_helpers_return_parser_errors() -> anyhow::Result<()> {
    let root = fixture_root();
    let Err(error) = io::load_spectra_1d_strict_by_source_format(&root, "jdx") else {
        anyhow::bail!("strict dimension source-format loading should reject malformed JCAMP-DX");
    };
    assert!(error.to_string().contains("missing XYDATA values"));

    let Err(error) = io::load_spectra_1d_summary_strict_by_source_format(&root, "jcamp dx") else {
        anyhow::bail!(
            "strict dimension source-format summary loading should reject malformed JCAMP-DX"
        );
    };
    assert!(error.to_string().contains("missing XYDATA values"));
    Ok(())
}

#[test]
fn dimension_source_path_helpers_load_matching_bundles() -> anyhow::Result<()> {
    let mixed = nmrxiv_fixture_root();
    let jcamp_1h = Path::new("jcamp/myrcene_1h_400mhz_jcamp_dx_6_link.jdx");
    let jcamp_13c = Path::new("jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx");
    let jeol_proton_path = Path::new("jeol/myrcene_1h_400mhz.jdf");
    let hsqc = Path::new("jeol/myrcene_hsqc_400mhz.jdf");

    let proton = io::load_spectra_1d_by_source_path(&mixed, jcamp_1h)?;
    assert_eq!(proton.len_1d(), 1);
    assert_eq!(proton.len_2d(), 0);
    assert!(proton.has_source_path(jcamp_1h));

    let selected_1d = io::load_spectra_1d_by_source_paths(&mixed, [jcamp_13c, jeol_proton_path])?;
    assert_eq!(selected_1d.len_1d(), 2);
    assert!(selected_1d.has_source_path(jcamp_13c));
    assert!(selected_1d.has_source_path(jeol_proton_path));

    let jcamp_prefix = io::load_spectra_1d_by_source_path_prefix(&mixed, "jcamp")?;
    assert_eq!(jcamp_prefix.len_1d(), 2);
    assert_eq!(jcamp_prefix.len_2d(), 0);

    let jeol_prefixes = io::load_spectra_1d_by_source_path_prefixes(&mixed, ["missing", "jeol"])?;
    assert_eq!(jeol_prefixes.len_1d(), 2);
    assert!(jeol_prefixes.has_source_path(jeol_proton_path));

    let jeol_hsqc = io::load_spectra_2d_by_source_path(&mixed, hsqc)?;
    assert_eq!(jeol_hsqc.len_1d(), 0);
    assert_eq!(jeol_hsqc.len_2d(), 1);
    assert!(jeol_hsqc.has_source_path(hsqc));

    let selected_2d = RSpinReader::new()
        .read_bundle_2d_by_source_paths(&mixed, [Path::new("bruker_cosy_raw"), hsqc])?;
    assert_eq!(selected_2d.len_2d(), 2);
    assert!(selected_2d.has_source_path("bruker_cosy_raw"));
    assert!(selected_2d.has_source_path(hsqc));

    let two_d_prefix =
        io::load_spectra_2d_by_source_path_prefixes(&mixed, ["bruker_cosy_raw", "jeol"])?;
    assert_eq!(two_d_prefix.len_2d(), 2);
    assert_eq!(two_d_prefix.len_1d(), 0);
    Ok(())
}

#[test]
fn dimension_source_path_summary_helpers_match_loaded_bundles() -> anyhow::Result<()> {
    let mixed = nmrxiv_fixture_root();
    let jcamp_1h = Path::new("jcamp/myrcene_1h_400mhz_jcamp_dx_6_link.jdx");
    let jcamp_13c = Path::new("jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx");
    let hsqc = Path::new("jeol/myrcene_hsqc_400mhz.jdf");

    let one_d_path = io::load_spectra_1d_by_source_path(&mixed, jcamp_1h)?;
    assert_eq!(
        io::load_spectra_1d_summary_by_source_path(&mixed, jcamp_1h)?,
        one_d_path.summary()
    );

    let one_d_paths = io::load_spectra_1d_by_source_paths(&mixed, [jcamp_1h, jcamp_13c])?;
    assert_eq!(
        RSpinReader::new().read_bundle_1d_summary_by_source_paths(&mixed, [jcamp_1h, jcamp_13c],)?,
        one_d_paths.summary()
    );

    let jcamp_prefix = io::load_spectra_1d_by_source_path_prefix(&mixed, "jcamp")?;
    assert_eq!(
        io::load_spectra_1d_summary_by_source_path_prefix(&mixed, "jcamp")?,
        jcamp_prefix.summary()
    );

    let two_d_path = io::load_spectra_2d_by_source_path(&mixed, hsqc)?;
    assert_eq!(
        io::load_spectra_2d_summary_by_source_path(&mixed, hsqc)?,
        two_d_path.summary()
    );

    let two_d_prefix =
        RSpinReader::new().read_bundle_2d_by_source_path_prefix(&mixed, "bruker_cosy_raw")?;
    assert_eq!(
        io::load_spectra_2d_summary_by_source_path_prefixes(&mixed, ["bruker_cosy_raw"])?,
        two_d_prefix.summary()
    );
    Ok(())
}

#[test]
fn dimension_source_path_relative_helpers_anchor_source_paths() -> anyhow::Result<()> {
    let mixed = nmrxiv_fixture_root();
    let jcamp_1h = Path::new("jcamp/myrcene_1h_400mhz_jcamp_dx_6_link.jdx");
    let jcamp_13c = Path::new("jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx");
    let jeol_proton_path = Path::new("jeol/myrcene_1h_400mhz.jdf");
    let hsqc = Path::new("jeol/myrcene_hsqc_400mhz.jdf");

    let jcamp = io::load_spectra_1d_by_source_path_prefix_relative_to(&mixed, "jcamp", "jcamp")?;
    assert_eq!(jcamp.len_1d(), 2);
    assert!(jcamp.has_source_path(jcamp_1h));
    assert!(jcamp.has_source_path(jcamp_13c));

    let jeol = io::load_spectra_1d_by_source_paths_relative_to(
        &mixed,
        "jeol",
        [jeol_proton_path, Path::new("jeol/myrcene_13c_400mhz.jdf")],
    )?;
    assert_eq!(jeol.len_1d(), 2);
    assert!(jeol.has_source_path(jeol_proton_path));

    let bruker_2d = io::load_spectra_2d_by_source_path_relative_to(
        &mixed,
        "bruker_cosy_raw",
        "bruker_cosy_raw",
    )?;
    assert_eq!(bruker_2d.len_2d(), 1);
    assert!(bruker_2d.has_source_path("bruker_cosy_raw"));

    let jeol_hsqc_bundle = RSpinReader::new()
        .read_bundle_2d_by_source_path_prefix_relative_to(&mixed, "jeol", "jeol")?;
    assert_eq!(jeol_hsqc_bundle.len_2d(), 1);
    assert!(jeol_hsqc_bundle.has_source_path(hsqc));

    assert_eq!(
        io::load_spectra_2d_summary_by_source_path_prefix_relative_to(&mixed, "jeol", "jeol")?,
        jeol_hsqc_bundle.summary()
    );
    Ok(())
}

#[test]
fn source_filtered_reader_methods_match_free_helpers() -> anyhow::Result<()> {
    let root = nmrxiv_fixture_root();
    let paths = vec![root.join("jcamp"), root.join("jeol")];
    let carbon_path = Path::new("jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx");
    let hsqc_path = Path::new("jeol/myrcene_hsqc_400mhz.jdf");

    assert_eq!(
        RSpinReader::new()
            .read_by_source_format(&root, LoadedSourceFormat::JcampDx)?
            .summary(),
        load_spectra_by_source_format(&root, "jdx")?.summary()
    );

    assert_eq!(
        RSpinReader::new()
            .read_by_source_vendor_relative_to(&root, "jeol", LoadedSourceVendor::Jeol)?
            .summary(),
        load_spectra_by_source_vendor_relative_to(&root, "jeol", "jeol")?.summary()
    );

    assert_eq!(
        RSpinReader::new()
            .read_many_by_source_data_kind(paths.clone(), LoadedSourceDataKind::Other)?
            .summary(),
        load_spectra_many_by_source_data_kind(paths.clone(), LoadedSourceDataKind::Other)?
            .summary()
    );

    assert_eq!(
        RSpinReader::new()
            .read_by_sources(
                &root,
                [
                    LoadedSourceFilter::path(carbon_path),
                    LoadedSourceFilter::path(hsqc_path),
                ],
            )?
            .summary(),
        load_spectra_by_sources(
            &root,
            [
                LoadedSourceFilter::path(carbon_path),
                LoadedSourceFilter::path(hsqc_path),
            ],
        )?
        .summary()
    );

    assert_eq!(
        RSpinReader::new()
            .read_many_by_source_path_prefix_relative_to(&root, ["jcamp", "jeol"], "jeol")?
            .summary(),
        io::load_spectra_many_by_source_path_prefix_relative_to(&root, ["jcamp", "jeol"], "jeol",)?
            .summary()
    );
    Ok(())
}

#[test]
fn source_path_set_helpers_load_matching_bundles() -> anyhow::Result<()> {
    let root = nmrxiv_fixture_root();
    let jcamp_1h = Path::new("jcamp/myrcene_1h_400mhz_jcamp_dx_6_link.jdx");
    let jcamp_13c = Path::new("jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx");
    let hsqc_path = Path::new("jeol/myrcene_hsqc_400mhz.jdf");

    let selected = io::load_spectra_by_source_paths(&root, [jcamp_1h, hsqc_path])?;
    assert_eq!(selected.len(), 2);
    assert!(selected.has_source_path(jcamp_1h));
    assert!(selected.has_source_path(hsqc_path));

    let relative =
        io::load_spectra_by_source_paths_relative_to(&root, "jcamp", [jcamp_1h, jcamp_13c])?;
    assert_eq!(relative.len_1d(), 2);
    assert!(relative.has_source_path(jcamp_13c));

    let paths = vec![root.join("jcamp"), root.join("jeol")];
    let local_carbon_path = Path::new("myrcene_13c_400mhz_jcamp_dx_6_link.jdx");
    let local_proton_path = Path::new("myrcene_1h_400mhz.jdf");
    let many =
        io::load_spectra_many_by_source_paths(paths, [local_carbon_path, local_proton_path])?;
    assert_eq!(many.len(), 2);
    assert!(many.has_source_path(local_carbon_path));
    assert!(many.has_source_path(local_proton_path));

    let relative_many = RSpinReader::new().read_many_by_source_paths_relative_to(
        &root,
        ["jcamp", "jeol"],
        [jcamp_13c, hsqc_path],
    )?;
    assert_eq!(relative_many.len(), 2);
    assert!(relative_many.has_source_path(jcamp_13c));
    assert!(relative_many.has_source_path(hsqc_path));

    assert_eq!(
        io::load_spectra_many_by_source_paths_relative_to(
            &root,
            ["jcamp", "jeol"],
            [jcamp_13c, hsqc_path],
        )?
        .summary(),
        relative_many.summary()
    );
    Ok(())
}

#[test]
fn strict_source_filtered_helpers_load_matching_bundles() -> anyhow::Result<()> {
    let root = nmrxiv_fixture_root();
    let proton_path = Path::new("jcamp/myrcene_1h_400mhz_jcamp_dx_6_link.jdx");
    let carbon_path = Path::new("jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx");
    let hsqc_path = Path::new("jeol/myrcene_hsqc_400mhz.jdf");

    let jcamp = io::load_spectra_strict_by_source_format(&root, "jdx")?;
    assert_eq!(jcamp.len(), 2);
    assert_eq!(jcamp.source_format_count(LoadedSourceFormat::JcampDx), 2);
    assert!(jcamp.has_source_path(proton_path));
    assert!(jcamp.has_source_path(carbon_path));

    let jeol = io::load_spectra_strict_by_source_vendor_relative_to(&root, "jeol", "jeol")?;
    assert_eq!(jeol.len(), 3);
    assert_eq!(jeol.source_vendor_count(LoadedSourceVendor::Jeol), 3);
    assert!(jeol.has_source_path(hsqc_path));

    let selected = io::load_spectra_strict_by_source_paths(&root, [carbon_path, hsqc_path])?;
    assert_eq!(selected.len(), 2);
    assert!(selected.has_source_path(carbon_path));
    assert!(selected.has_source_path(hsqc_path));

    let summary = io::load_spectra_many_summary_strict_by_source_paths_relative_to(
        &root,
        ["jcamp", "jeol"],
        [carbon_path, hsqc_path],
    )?;
    assert_eq!(summary.spectra(), 2);
    assert_eq!(summary.spectra_1d(), 1);
    assert_eq!(summary.spectra_2d(), 1);
    Ok(())
}

#[test]
fn strict_source_filtered_helpers_return_parser_errors() -> anyhow::Result<()> {
    let root = fixture_root();
    let Err(error) = io::load_spectra_strict_by_source_format(&root, "jdx") else {
        anyhow::bail!("strict source-format loading should reject malformed JCAMP-DX");
    };
    assert!(error.to_string().contains("missing XYDATA values"));

    let Err(error) =
        io::load_spectra_summary_strict_by_source_path(&root, Path::new("empty_jcamp/empty.jdx"))
    else {
        anyhow::bail!("strict source-path summary should reject malformed JCAMP-DX");
    };
    assert!(error.to_string().contains("missing XYDATA values"));
    Ok(())
}

#[test]
fn strict_source_filtered_reader_methods_match_helpers() -> anyhow::Result<()> {
    let root = nmrxiv_fixture_root();
    let carbon_path = Path::new("jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx");
    let hsqc_path = Path::new("jeol/myrcene_hsqc_400mhz.jdf");

    let jcamp =
        RSpinReader::new().read_strict_by_source_format(&root, LoadedSourceFormat::JcampDx)?;
    assert_eq!(
        jcamp.summary(),
        io::load_spectra_strict_by_source_format(&root, "jdx")?.summary()
    );

    let jeol = RSpinReader::new().read_strict_by_source_vendor_relative_to(
        &root,
        "jeol",
        LoadedSourceVendor::Jeol,
    )?;
    assert_eq!(
        jeol.summary(),
        io::load_spectra_strict_by_source_vendor_relative_to(&root, "jeol", "jeol")?.summary()
    );

    let selected = RSpinReader::new().read_strict_by_sources(
        &root,
        [
            LoadedSourceFilter::path(carbon_path),
            LoadedSourceFilter::path(hsqc_path),
        ],
    )?;
    assert_eq!(
        selected.summary(),
        io::load_spectra_strict_by_sources(
            &root,
            [
                LoadedSourceFilter::path(carbon_path),
                LoadedSourceFilter::path(hsqc_path),
            ],
        )?
        .summary()
    );

    let summary = RSpinReader::new().read_summary_many_strict_by_source_paths_relative_to(
        &root,
        ["jcamp", "jeol"],
        [carbon_path, hsqc_path],
    )?;
    assert_eq!(
        summary,
        io::load_spectra_many_summary_strict_by_source_paths_relative_to(
            &root,
            ["jcamp", "jeol"],
            [carbon_path, hsqc_path],
        )?
    );
    Ok(())
}

#[test]
fn strict_source_filtered_reader_methods_return_parser_errors() -> anyhow::Result<()> {
    let root = fixture_root();
    let Err(error) = RSpinReader::new()
        .read_summary_strict_by_source_path(&root, Path::new("empty_jcamp/empty.jdx"))
    else {
        anyhow::bail!("strict source-path summary reader should reject malformed JCAMP-DX");
    };
    assert!(error.to_string().contains("missing XYDATA values"));
    Ok(())
}

#[test]
fn source_filtered_summary_helpers_match_loaded_bundles() -> anyhow::Result<()> {
    let root = nmrxiv_fixture_root();
    let proton_path = Path::new("jcamp/myrcene_1h_400mhz_jcamp_dx_6_link.jdx");
    let carbon_path = Path::new("jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx");
    let hsqc_path = Path::new("jeol/myrcene_hsqc_400mhz.jdf");

    let jcamp = load_spectra_by_source_format(&root, "jdx")?;
    assert_eq!(
        io::load_spectra_summary_by_source_format(&root, "jcamp dx")?,
        jcamp.summary()
    );

    let jeol = load_spectra_by_source_vendor(&root, LoadedSourceVendor::Jeol)?;
    assert_eq!(
        io::load_spectra_summary_by_source_vendor(&root, "jeol")?,
        jeol.summary()
    );

    let raw = load_spectra_by_source_data_kind(&root, LoadedSourceDataKind::Raw)?;
    assert_eq!(
        io::load_spectra_summary_by_source_data_kind(&root, LoadedSourceDataKind::Raw)?,
        raw.summary()
    );

    let generic = load_spectra_by_sources(
        &root,
        [
            LoadedSourceFilter::format("jdx"),
            LoadedSourceFilter::path(hsqc_path),
        ],
    )?;
    assert_eq!(
        RSpinReader::new().read_summary_by_sources(
            &root,
            [
                LoadedSourceFilter::format("jdx"),
                LoadedSourceFilter::path(hsqc_path),
            ],
        )?,
        generic.summary()
    );

    let carbon = load_spectra_by_source_path(&root, carbon_path)?;
    assert_eq!(
        io::load_spectra_summary_by_source_path(&root, carbon_path)?,
        carbon.summary()
    );

    let selected_paths = RSpinReader::new()
        .only_source_paths([proton_path, carbon_path])
        .read_path(&root)?;
    assert_eq!(
        io::load_spectra_summary_by_source_paths(&root, [proton_path, carbon_path])?,
        selected_paths.summary()
    );

    let jeol_prefix = io::load_spectra_by_source_path_prefix(&root, "jeol")?;
    assert_eq!(
        io::load_spectra_summary_by_source_path_prefix(&root, "jeol")?,
        jeol_prefix.summary()
    );

    let mixed_prefixes =
        io::load_spectra_by_source_path_prefixes(&root, ["jcamp", "bruker_cosy_raw"])?;
    assert_eq!(
        io::load_spectra_summary_by_source_path_prefixes(&root, ["jcamp", "bruker_cosy_raw"])?,
        mixed_prefixes.summary()
    );
    Ok(())
}

#[test]
fn source_filtered_summary_relative_helpers_anchor_source_paths() -> anyhow::Result<()> {
    let root = nmrxiv_fixture_root();
    let carbon_path = Path::new("jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx");

    let jcamp = load_spectra_by_source_format_relative_to(&root, "jcamp", "jdx")?;
    assert_eq!(
        io::load_spectra_summary_by_source_format_relative_to(&root, "jcamp", "jcamp dx")?,
        jcamp.summary()
    );

    let jeol = load_spectra_by_source_vendor_relative_to(&root, "jeol", LoadedSourceVendor::Jeol)?;
    assert_eq!(
        RSpinReader::new().read_summary_by_source_vendor_relative_to(&root, "jeol", "jeol")?,
        jeol.summary()
    );

    let carbon = load_spectra_by_source_path_relative_to(&root, "jcamp", carbon_path)?;
    assert_eq!(
        io::load_spectra_summary_by_source_path_relative_to(&root, "jcamp", carbon_path)?,
        carbon.summary()
    );

    let bruker = load_spectra_by_source_relative_to(
        &root,
        "bruker_cosy_raw",
        LoadedSourceFilter::path_prefix("bruker_cosy_raw"),
    )?;
    assert_eq!(
        RSpinReader::new().read_summary_by_source_relative_to(
            &root,
            "bruker_cosy_raw",
            LoadedSourceFilter::path_prefix("bruker_cosy_raw"),
        )?,
        bruker.summary()
    );

    let jeol_prefix = io::load_spectra_by_source_path_prefix_relative_to(&root, "jeol", "jeol")?;
    assert_eq!(
        io::load_spectra_summary_by_source_path_prefix_relative_to(&root, "jeol", "jeol")?,
        jeol_prefix.summary()
    );

    let combined = load_spectra_by_sources_relative_to(
        &root,
        "jcamp",
        [
            LoadedSourceFilter::format("jdx"),
            LoadedSourceFilter::vendor("jeol"),
        ],
    )?;
    assert_eq!(
        io::load_spectra_summary_by_sources_relative_to(
            &root,
            "jcamp",
            [
                LoadedSourceFilter::format("jdx"),
                LoadedSourceFilter::vendor("jeol"),
            ],
        )?,
        combined.summary()
    );
    Ok(())
}

#[test]
fn source_filtered_many_summary_helpers_match_loaded_bundles() -> anyhow::Result<()> {
    let root = nmrxiv_fixture_root();
    let paths = vec![root.join("jcamp"), root.join("jeol")];
    let carbon_path = Path::new("jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx");
    let proton_path = Path::new("jeol/myrcene_1h_400mhz.jdf");

    let jdf = load_spectra_many_by_source_format(paths.clone(), LoadedSourceFormat::JeolJdf)?;
    assert_eq!(
        io::load_spectra_many_summary_by_source_format(paths.clone(), "jdf")?,
        jdf.summary()
    );

    let jeol = load_spectra_many_by_source_vendors(paths.clone(), [LoadedSourceVendor::Jeol])?;
    assert_eq!(
        io::load_spectra_many_summary_by_source_vendors(paths.clone(), ["jeol"])?,
        jeol.summary()
    );

    let other = load_spectra_many_by_source_data_kind(paths.clone(), LoadedSourceDataKind::Other)?;
    assert_eq!(
        io::load_spectra_many_summary_by_source_data_kind(
            paths.clone(),
            LoadedSourceDataKind::Other,
        )?,
        other.summary()
    );

    let relative_jeol =
        load_spectra_many_by_source_vendor_relative_to(&root, ["jcamp", "jeol"], "jeol")?;
    assert_eq!(
        RSpinReader::new().read_summary_many_by_source_vendor_relative_to(
            &root,
            ["jcamp", "jeol"],
            LoadedSourceVendor::Jeol,
        )?,
        relative_jeol.summary()
    );

    let relative_paths = RSpinReader::new()
        .only_source_paths([carbon_path, proton_path])
        .read_paths_relative_to(&root, ["jcamp", "jeol"])?;
    assert_eq!(
        io::load_spectra_many_summary_by_source_paths_relative_to(
            &root,
            ["jcamp", "jeol"],
            [carbon_path, proton_path],
        )?,
        relative_paths.summary()
    );

    let relative_prefix =
        io::load_spectra_many_by_source_path_prefix_relative_to(&root, ["jcamp", "jeol"], "jeol")?;
    assert_eq!(
        io::load_spectra_many_summary_by_source_path_prefix_relative_to(
            &root,
            ["jcamp", "jeol"],
            "jeol",
        )?,
        relative_prefix.summary()
    );
    Ok(())
}

#[test]
fn free_bundle_loader_helpers_can_restrict_sources() -> anyhow::Result<()> {
    let root = nmrxiv_fixture_root();
    let carbon_path = Path::new("jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx");

    let jcamp = load_spectra_by_source_format(&root, "jdx")?;
    assert_eq!(jcamp.len(), 2);
    assert_eq!(jcamp.source_format_count(LoadedSourceFormat::JcampDx), 2);

    let relative_jcamp = load_spectra_by_source_format_relative_to(&root, "jcamp", "jcamp dx")?;
    assert_eq!(relative_jcamp.len(), 2);
    assert!(
        relative_jcamp
            .source_paths()
            .all(|path| path.starts_with(Path::new("jcamp")))
    );

    let bruker = load_spectra_by_source_vendor(&root, LoadedSourceVendor::Bruker)?;
    assert_eq!(bruker.len(), 2);
    assert_eq!(bruker.source_vendor_count(LoadedSourceVendor::Bruker), 2);

    let relative_jeol = load_spectra_by_source_vendor_relative_to(&root, "jeol", "jeol")?;
    assert_eq!(relative_jeol.len(), 3);
    assert_eq!(
        relative_jeol.source_vendor_count(LoadedSourceVendor::Jeol),
        3
    );

    let carbon = load_spectra_by_source_path(&root, carbon_path)?;
    assert_eq!(carbon.len(), 1);
    assert_eq!(first_1d(&carbon)?.metadata.nucleus, Some(Nucleus::Carbon13));
    assert!(carbon.has_source_path(carbon_path));

    let relative_carbon = load_spectra_by_source_path_relative_to(&root, "jcamp", carbon_path)?;
    assert_eq!(relative_carbon.len(), 1);
    assert_eq!(
        first_1d(&relative_carbon)?.metadata.nucleus,
        Some(Nucleus::Carbon13)
    );
    assert!(relative_carbon.has_source_path(carbon_path));

    let jcamp_prefix = load_spectra_by_source_path_prefix(&root, "jcamp")?;
    assert_eq!(jcamp_prefix.len(), 2);
    assert_eq!(
        jcamp_prefix.source_format_count(LoadedSourceFormat::JcampDx),
        2
    );
    assert!(jcamp_prefix.has_source_path_prefix("jcamp"));

    let relative_jcamp_prefix =
        load_spectra_by_source_path_prefix_relative_to(&root, "jcamp", "jcamp")?;
    assert_eq!(relative_jcamp_prefix.len(), 2);
    assert!(
        relative_jcamp_prefix
            .source_paths()
            .all(|path| path.starts_with(Path::new("jcamp")))
    );

    let combined = load_spectra_by_sources(
        &root,
        [
            LoadedSourceFilter::format("jdx"),
            LoadedSourceFilter::vendor("bruker"),
        ],
    )?;
    assert_eq!(combined.len(), 4);
    assert_eq!(combined.source_format_count(LoadedSourceFormat::JcampDx), 2);
    assert_eq!(combined.source_vendor_count(LoadedSourceVendor::Bruker), 2);

    let relative_combined = load_spectra_by_sources_relative_to(
        &root,
        "jcamp",
        [
            LoadedSourceFilter::from(LoadedSourceFormat::JcampDx),
            LoadedSourceFilter::vendor("bruker"),
        ],
    )?;
    assert_eq!(relative_combined.len(), 2);
    assert_eq!(
        relative_combined.source_format_count(LoadedSourceFormat::JcampDx),
        2
    );
    Ok(())
}

#[test]
fn free_bundle_loader_set_helpers_can_restrict_sources() -> anyhow::Result<()> {
    let root = nmrxiv_fixture_root();

    let jcamp_or_jeol = load_spectra_by_source_formats(
        &root,
        [LoadedSourceFormat::JcampDx, LoadedSourceFormat::JeolJdf],
    )?;
    assert_eq!(jcamp_or_jeol.len(), 5);
    assert_eq!(
        jcamp_or_jeol.source_format_count(LoadedSourceFormat::JcampDx),
        2
    );
    assert_eq!(
        jcamp_or_jeol.source_format_count(LoadedSourceFormat::JeolJdf),
        3
    );

    let relative_jcamp_or_jeol =
        load_spectra_by_source_formats_relative_to(&root, "jcamp", ["jdx", "jdf"])?;
    assert_eq!(relative_jcamp_or_jeol.len(), 2);
    assert!(
        relative_jcamp_or_jeol
            .source_paths()
            .all(|path| path.starts_with(Path::new("jcamp")))
    );

    let bruker_or_jeol = load_spectra_by_source_vendors(
        &root,
        [LoadedSourceVendor::Bruker, LoadedSourceVendor::Jeol],
    )?;
    assert_eq!(bruker_or_jeol.len(), 5);
    assert_eq!(
        bruker_or_jeol.source_vendor_count(LoadedSourceVendor::Bruker),
        2
    );
    assert_eq!(
        bruker_or_jeol.source_vendor_count(LoadedSourceVendor::Jeol),
        3
    );

    let relative_vendor_set =
        load_spectra_by_source_vendors_relative_to(&root, "jeol", ["bruker", "jeol"])?;
    assert_eq!(relative_vendor_set.len(), 3);
    assert_eq!(
        relative_vendor_set.source_vendor_count(LoadedSourceVendor::Jeol),
        3
    );

    let unrestricted_formats = load_spectra_by_source_formats(&root, std::iter::empty::<&str>())?;
    assert_eq!(unrestricted_formats.len(), 7);

    let unrestricted_vendors = load_spectra_by_source_vendors(&root, std::iter::empty::<&str>())?;
    assert_eq!(unrestricted_vendors.len(), 7);
    Ok(())
}

#[test]
fn generic_source_filter_helpers_can_restrict_sources() -> anyhow::Result<()> {
    let root = nmrxiv_fixture_root();
    let carbon_path = Path::new("jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx");
    let jcamp_filter = LoadedSourceFilter::format("jdx");
    let vendor_filter = LoadedSourceFilter::from(LoadedSourceVendor::Bruker);
    let path_filter = LoadedSourceFilter::path(carbon_path);

    let jcamp = load_spectra_by_source(&root, &jcamp_filter)?;
    assert_eq!(jcamp.len(), 2);
    assert_eq!(jcamp.source_format_count(LoadedSourceFormat::JcampDx), 2);
    assert!(
        jcamp
            .loaded_by_source_format(LoadedSourceFormat::JcampDx)
            .all(|loaded| jcamp_filter.matches_source(loaded.source()))
    );

    let bruker = RSpinReader::new()
        .only_source(vendor_filter.clone())
        .read_path(&root)?;
    assert_eq!(bruker.len(), 2);
    assert_eq!(bruker.source_vendor_count(LoadedSourceVendor::Bruker), 2);
    assert!(
        bruker
            .spectra()
            .iter()
            .all(|loaded| vendor_filter.matches_source(loaded.source()))
    );

    let relative_carbon = load_spectra_by_source_relative_to(&root, "jcamp", path_filter.clone())?;
    assert_eq!(relative_carbon.len(), 1);
    assert_eq!(
        first_1d(&relative_carbon)?.metadata.nucleus,
        Some(Nucleus::Carbon13)
    );
    assert!(relative_carbon.has_source_path(carbon_path));

    let base = fixture_root();
    let paths = [base.join("varian_1h"), base.join("bruker_without_expno")];
    let agilent = load_spectra_many_by_source(&paths, LoadedSourceFilter::vendor("varian"))?;
    assert_eq!(agilent.len(), 1);
    assert_eq!(
        agilent.source_vendor_count(LoadedSourceVendor::AgilentVarian),
        1
    );

    let processed = load_spectra_many_by_source_relative_to(
        &base,
        ["varian_1h", "bruker_without_expno"],
        LoadedSourceFilter::path("bruker_without_expno/pdata/1"),
    )?;
    assert_eq!(processed.len(), 1);
    assert_eq!(first_1d(&processed)?.x.unit, Unit::Ppm);

    let combined = RSpinReader::new()
        .only_sources([jcamp_filter, vendor_filter])
        .read_path(&root)?;
    assert_eq!(combined.len(), 4);
    assert_eq!(combined.source_format_count(LoadedSourceFormat::JcampDx), 2);
    assert_eq!(combined.source_vendor_count(LoadedSourceVendor::Bruker), 2);

    let hsqc_path = Path::new("jeol/myrcene_hsqc_400mhz.jdf");
    let path_selected = RSpinReader::new()
        .only_sources([path_filter, LoadedSourceFilter::path(hsqc_path)])
        .read_path(&root)?;
    assert_eq!(path_selected.len(), 2);
    assert_eq!(path_selected.len_1d(), 1);
    assert_eq!(path_selected.len_2d(), 1);
    assert!(path_selected.has_source_path(carbon_path));
    assert!(path_selected.has_source_path(hsqc_path));

    let cleared = RSpinReader::new()
        .only_sources([LoadedSourceFilter::vendor("bruker")])
        .all_sources()
        .read_path(&root)?;
    assert_eq!(cleared.len(), 7);
    Ok(())
}

#[test]
fn free_multi_path_bundle_loader_helpers_can_restrict_sources() -> anyhow::Result<()> {
    let base = fixture_root();
    let paths = [base.join("varian_1h"), base.join("bruker_without_expno")];

    let processed =
        load_spectra_many_by_source_format(&paths, LoadedSourceFormat::BrukerProcessed)?;
    assert_eq!(processed.len(), 1);
    assert_eq!(
        processed.source_format_count(LoadedSourceFormat::BrukerProcessed),
        1
    );
    assert_eq!(first_1d(&processed)?.x.unit, Unit::Ppm);

    let relative_varian = load_spectra_many_by_source_format_relative_to(
        &base,
        ["varian_1h", "bruker_without_expno"],
        "varian fid",
    )?;
    assert_eq!(relative_varian.len(), 1);
    assert!(relative_varian.has_source_path(Path::new("varian_1h")));

    let bruker = load_spectra_many_by_source_vendor(&paths, "bruker")?;
    assert_eq!(bruker.len(), 2);
    assert_eq!(bruker.source_vendor_count(LoadedSourceVendor::Bruker), 2);

    let relative_vendor =
        load_spectra_many_by_source_vendor_relative_to(&base, ["varian_1h"], "varian")?;
    assert_eq!(relative_vendor.len(), 1);
    assert_eq!(
        relative_vendor.source_vendor_count(LoadedSourceVendor::AgilentVarian),
        1
    );

    let varian = load_spectra_many_by_source_path(&paths, "varian_1h")?;
    assert_eq!(varian.len(), 1);
    assert_eq!(
        first_1d(&varian)?.metadata.nucleus,
        Some(Nucleus::Hydrogen1)
    );

    let relative_processed = load_spectra_many_by_source_path_relative_to(
        &base,
        ["varian_1h", "bruker_without_expno"],
        "bruker_without_expno/pdata/1",
    )?;
    assert_eq!(relative_processed.len(), 1);
    assert_eq!(first_1d(&relative_processed)?.x.unit, Unit::Ppm);

    let processed_prefix = load_spectra_many_by_source_path_prefix(&paths, "pdata")?;
    assert_eq!(processed_prefix.len(), 1);
    assert_eq!(first_1d(&processed_prefix)?.x.unit, Unit::Ppm);

    let relative_processed_prefix = load_spectra_many_by_source_path_prefix_relative_to(
        &base,
        ["varian_1h", "bruker_without_expno"],
        "bruker_without_expno/pdata",
    )?;
    assert_eq!(relative_processed_prefix.len(), 1);
    assert_eq!(first_1d(&relative_processed_prefix)?.x.unit, Unit::Ppm);

    let combined = load_spectra_many_by_sources(
        &paths,
        [
            LoadedSourceFilter::vendor("varian"),
            LoadedSourceFilter::path("pdata/1"),
        ],
    )?;
    assert_eq!(combined.len(), 2);
    assert_eq!(
        combined.source_vendor_count(LoadedSourceVendor::AgilentVarian),
        1
    );
    assert_eq!(
        combined.source_format_count(LoadedSourceFormat::BrukerProcessed),
        1
    );

    let relative_combined = load_spectra_many_by_sources_relative_to(
        &base,
        ["varian_1h", "bruker_without_expno"],
        [
            LoadedSourceFilter::path("varian_1h"),
            LoadedSourceFilter::path("bruker_without_expno/pdata/1"),
        ],
    )?;
    assert_eq!(relative_combined.len(), 2);
    assert!(relative_combined.has_source_path(Path::new("varian_1h")));
    assert!(relative_combined.has_source_path(Path::new("bruker_without_expno/pdata/1")));
    Ok(())
}

#[test]
fn free_multi_path_bundle_loader_set_helpers_can_restrict_sources() -> anyhow::Result<()> {
    let base = fixture_root();
    let paths = [base.join("varian_1h"), base.join("bruker_without_expno")];

    let selected_formats = load_spectra_many_by_source_formats(
        &paths,
        [
            LoadedSourceFormat::AgilentFid,
            LoadedSourceFormat::BrukerProcessed,
        ],
    )?;
    assert_eq!(selected_formats.len(), 2);
    assert_eq!(
        selected_formats.source_format_count(LoadedSourceFormat::AgilentFid),
        1
    );
    assert_eq!(
        selected_formats.source_format_count(LoadedSourceFormat::BrukerProcessed),
        1
    );

    let relative_selected_formats = load_spectra_many_by_source_formats_relative_to(
        &base,
        ["varian_1h", "bruker_without_expno"],
        [
            LoadedSourceFormat::AgilentFid,
            LoadedSourceFormat::BrukerFid,
        ],
    )?;
    assert_eq!(relative_selected_formats.len(), 2);
    assert_eq!(
        relative_selected_formats.source_format_count(LoadedSourceFormat::AgilentFid),
        1
    );
    assert_eq!(
        relative_selected_formats.source_format_count(LoadedSourceFormat::BrukerFid),
        1
    );

    let selected_vendors = load_spectra_many_by_source_vendors(&paths, ["varian", "bruker"])?;
    assert_eq!(selected_vendors.len(), 3);
    assert_eq!(
        selected_vendors.source_vendor_count(LoadedSourceVendor::AgilentVarian),
        1
    );
    assert_eq!(
        selected_vendors.source_vendor_count(LoadedSourceVendor::Bruker),
        2
    );

    let relative_selected_vendors = load_spectra_many_by_source_vendors_relative_to(
        &base,
        ["varian_1h", "bruker_without_expno"],
        [
            LoadedSourceVendor::AgilentVarian,
            LoadedSourceVendor::Bruker,
        ],
    )?;
    assert_eq!(relative_selected_vendors.len(), 3);
    assert_eq!(
        relative_selected_vendors.source_vendor_count(LoadedSourceVendor::AgilentVarian),
        1
    );
    assert_eq!(
        relative_selected_vendors.source_vendor_count(LoadedSourceVendor::Bruker),
        2
    );
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
fn loader_source_path_filter_applies_to_nested_bundle_json() -> anyhow::Result<()> {
    let source_bundle = load_spectra(nmrxiv_fixture_root())?;
    let root = temp_dir("source-path-bundle")?;
    fs::write(
        root.join("bundle.json"),
        write_spectrum_bundle_json(&source_bundle)?,
    )?;

    let selected_path = Path::new("bundle.json/jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx");
    let bundle = RSpinReader::new()
        .only_source_path(selected_path)
        .read_path(&root)?;
    assert_eq!(bundle.len(), 1);
    assert_eq!(first_1d(&bundle)?.metadata.nucleus, Some(Nucleus::Carbon13));
    assert!(bundle.has_source_path(selected_path));

    let generic_path = RSpinReader::new()
        .sources([LoadedSourceFilter::path(selected_path)])
        .read_path(&root)?;
    assert_eq!(generic_path.len(), 1);
    assert_eq!(
        first_1d(&generic_path)?.metadata.nucleus,
        Some(Nucleus::Carbon13)
    );
    assert!(generic_path.has_source_path(selected_path));

    let hidden_sources = RSpinReader::new()
        .only_source_path(selected_path)
        .without_source_paths()
        .read_path(&root)?;
    assert_eq!(hidden_sources.len(), 1);
    assert_eq!(
        first_1d(&hidden_sources)?.metadata.nucleus,
        Some(Nucleus::Carbon13)
    );
    assert_eq!(hidden_sources.source_paths().count(), 0);

    let filtered_out = RSpinReader::new()
        .only_source_path("bundle.json/missing.jdx")
        .read_path(&root);
    let Err(error) = filtered_out else {
        anyhow::bail!("missing nested source-path filter should leave no readable spectra");
    };
    assert!(error.to_string().contains("no readable bundle data found"));

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
