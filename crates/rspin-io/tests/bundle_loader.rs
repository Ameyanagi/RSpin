//! Integration tests for the unified spectrum bundle loader.

use std::path::{Path, PathBuf};

use rspin_core::{Nucleus, RSpinError, Unit};
use rspin_io::{LoadedSpectrum, RSpinReader, SpectrumBundleLoader, load_spectra};

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

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/zenodo_7100132")
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
