//! Integration tests for the unified spectrum bundle loader.

use std::path::{Path, PathBuf};

use rspin_core::{Nucleus, RSpinError, Unit};
use rspin_io::{
    LoadedSpectrum, RSpinReader, SpectrumBundle, SpectrumBundleLoader, SpectrumPathReader,
    load_spectra, load_spectra_many, load_spectrum_1d, load_spectrum_2d,
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

    assert_eq!(bundle.len(), 5);
    assert_eq!(bundle.spectra_1d().count(), 3);
    assert_eq!(bundle.spectra_2d().count(), 2);
    assert_eq!(bundle.warnings().len(), 1);

    let warning = bundle
        .warnings()
        .first()
        .ok_or_else(|| anyhow::anyhow!("missing unsupported JCAMP-DX warning"))?;
    assert_eq!(
        warning.path.as_deref(),
        Some(Path::new("jcamp/myrcene_1h_400mhz_jcamp_dx_6_link.jdx"))
    );
    assert!(warning.message.contains("JCAMP-DX version"));

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

fn first_1d(bundle: &rspin_io::SpectrumBundle) -> anyhow::Result<&rspin_core::Spectrum1D> {
    bundle
        .spectra_1d()
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing one-dimensional spectrum"))
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
