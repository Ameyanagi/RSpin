//! Always-on parser tests for redistributed nmrML fixtures.

use std::path::{Path, PathBuf};

use rspin_core::{Nucleus, Unit};
use rspin_io::{LoadedSourceFormat, RSpinReader, read_nmrml_1d_file};

#[test]
fn reads_official_mit_nmrml_processed_1d_fixture() -> anyhow::Result<()> {
    let fixture = nmrml_fixture_root().join("MMBBI_10M12-CE01-1a.nmrML");
    let spectrum = read_nmrml_1d_file(&fixture)?;

    assert_eq!(spectrum.len(), 32_768);
    assert_eq!(spectrum.x.unit, Unit::Ppm);
    assert_close(spectrum.x.values.first().copied(), Some(11.099_15));
    assert_close(spectrum.x.values.last().copied(), Some(-0.901_812));
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_close(spectrum.metadata.frequency_mhz, Some(500.162_500_8));
    assert_close(spectrum.metadata.temperature_k, Some(300.0));
    assert!(has_signal(&spectrum.intensities, 1_000.0));
    Ok(())
}

#[test]
fn unified_loader_reads_official_mit_nmrml_fixture() -> anyhow::Result<()> {
    let bundle = RSpinReader::new().read_path(nmrml_fixture_root())?;

    assert_eq!(bundle.len(), 1);
    assert_eq!(bundle.len_1d(), 1);
    assert_eq!(bundle.len_2d(), 0);
    assert!(bundle.warnings().is_empty());
    assert_eq!(bundle.source_format_count(LoadedSourceFormat::NmrMl), 1);
    assert!(has_source_path(
        &bundle,
        Path::new("MMBBI_10M12-CE01-1a.nmrML")
    ));
    Ok(())
}

fn nmrml_fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/nmrml/mit")
}

fn has_signal(values: &[f64], threshold: f64) -> bool {
    values.iter().any(|value| value.abs() > threshold)
}

fn has_source_path(bundle: &rspin_io::SpectrumBundle, path: &Path) -> bool {
    bundle
        .spectra()
        .iter()
        .any(|loaded| loaded.source().path.as_deref() == Some(path))
}

fn assert_close(actual: Option<f64>, expected: Option<f64>) {
    let (Some(actual), Some(expected)) = (actual, expected) else {
        panic!("expected {expected:?}, got {actual:?}");
    };
    assert!(
        (actual - expected).abs() < 1.0e-6,
        "expected {expected}, got {actual}"
    );
}
