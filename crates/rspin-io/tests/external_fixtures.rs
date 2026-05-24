//! Opt-in local checks for external NMR fixture caches.
//!
//! These tests intentionally skip unless `RSPIN_EXTERNAL_TESTDATA` points to a
//! local cache. Fixture files are not vendored into the repository.

use std::{
    env, fs,
    path::{Path, PathBuf},
};

use rspin_core::{Nucleus, Unit};
use rspin_io::read_jcamp_dx_1d;

#[test]
fn parses_external_jcamp_peak_table_fixture_when_available() -> anyhow::Result<()> {
    let Some(root) = external_testdata_root() else {
        return Ok(());
    };
    let fixture = root.join("unpacked/jcamp-data-test-2.5.0/data/nmr/simulated/d1-2_j7.jdx");
    require_fixture(&fixture)?;

    let input = fs::read_to_string(&fixture)?;
    let spectrum = read_jcamp_dx_1d(&input)?;

    assert_eq!(spectrum.len(), 131_072);
    assert_eq!(spectrum.x.unit, Unit::Ppm);
    assert!(spectrum.intensities.iter().any(|value| *value > 0.0));
    Ok(())
}

#[test]
fn parses_external_jcamp_baseline_fixture_when_available() -> anyhow::Result<()> {
    let Some(root) = external_testdata_root() else {
        return Ok(());
    };
    let fixture = root
        .join("unpacked/jcamp-data-test-2.5.0/data/nmr/simulated/d1-2-3-4-5-6-7-8_baseline.jdx");
    require_fixture(&fixture)?;

    let input = fs::read_to_string(&fixture)?;
    let spectrum = read_jcamp_dx_1d(&input)?;

    assert_eq!(spectrum.len(), 131_072);
    assert_eq!(spectrum.x.unit, Unit::Ppm);
    assert!(spectrum.intensities.iter().any(|value| *value > 0.0));
    Ok(())
}

#[test]
fn parses_external_jcamp_numeric_data_table_fixture_when_available() -> anyhow::Result<()> {
    let Some(root) = external_testdata_root() else {
        return Ok(());
    };
    let fixture = root.join("unpacked/jcamp-data-test-2.5.0/data/nmr/nanalysis/1h.jdx");
    require_fixture(&fixture)?;

    let input = fs::read_to_string(&fixture)?;
    let spectrum = read_jcamp_dx_1d(&input)?;

    assert_eq!(spectrum.len(), 2048);
    assert_eq!(spectrum.x.unit, Unit::Seconds);
    assert_eq!(spectrum.metadata.origin.as_deref(), Some("Nanalysis Corp."));
    assert_eq!(spectrum.metadata.solvent.as_deref(), Some("Chloroform-d"));
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_close(spectrum.metadata.temperature_k, Some(306.150_009));
    assert!(spectrum.imaginary.is_some());
    assert!(spectrum.intensities.iter().any(|value| value.abs() > 1.0));
    Ok(())
}

#[test]
fn parses_external_jcamp_decimal_count_fixture_when_available() -> anyhow::Result<()> {
    let Some(root) = external_testdata_root() else {
        return Ok(());
    };
    let fixture = root.join("unpacked/jcamp-data-test-2.5.0/data/nmr/varian/1h.jdx");
    require_fixture(&fixture)?;

    let input = fs::read_to_string(&fixture)?;
    let spectrum = read_jcamp_dx_1d(&input)?;

    assert_eq!(spectrum.len(), 16_384);
    assert_eq!(spectrum.x.unit, Unit::Ppm);
    assert_eq!(spectrum.metadata.origin.as_deref(), Some("agfavnmr"));
    assert_eq!(spectrum.metadata.solvent.as_deref(), Some("CDCL3"));
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert!(spectrum.intensities.iter().any(|value| value.abs() > 1.0));
    Ok(())
}

#[test]
fn parses_external_jcamp_asdf_fixture_when_available() -> anyhow::Result<()> {
    let Some(root) = external_testdata_root() else {
        return Ok(());
    };
    let fixture = root.join("unpacked/jcamp-data-test-2.5.0/data/nmr/jeol/1h.dx");
    require_fixture(&fixture)?;

    let input = fs::read_to_string(&fixture)?;
    let spectrum = read_jcamp_dx_1d(&input)?;

    assert_eq!(spectrum.len(), 16_384);
    assert_eq!(spectrum.x.unit, Unit::Hertz);
    assert_eq!(spectrum.metadata.origin.as_deref(), Some("JEOL"));
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert!(spectrum.intensities.iter().any(|value| *value > 1.0));
    Ok(())
}

fn external_testdata_root() -> Option<PathBuf> {
    env::var_os("RSPIN_EXTERNAL_TESTDATA")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

fn require_fixture(path: &Path) -> anyhow::Result<()> {
    if path.is_file() {
        return Ok(());
    }

    anyhow::bail!(
        "missing external fixture at {}; check RSPIN_EXTERNAL_TESTDATA",
        path.display()
    );
}

fn assert_close(actual: Option<f64>, expected: Option<f64>) {
    match (actual, expected) {
        (Some(left), Some(right)) => assert!((left - right).abs() < 1e-12, "{left} != {right}"),
        (left, right) => assert_eq!(left, right),
    }
}
