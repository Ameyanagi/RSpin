//! Opt-in local checks for external NMR fixture caches.
//!
//! These tests intentionally skip unless `RSPIN_EXTERNAL_TESTDATA` points to a
//! local cache. Fixture files are not vendored into the repository.

use std::{
    env, fs,
    path::{Path, PathBuf},
};

use rspin_core::{Nucleus, Unit};
use rspin_io::{
    read_agilent_fid_1d_dir, read_agilent_fid_2d_dir, read_bruker_fid_1d_dir,
    read_bruker_ser_2d_dir, read_jcamp_dx_1d, read_jeol_jdf_1d_file,
};

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

#[test]
fn parses_external_agilent_1d_fid_when_available() -> anyhow::Result<()> {
    let Some(root) = external_testdata_root() else {
        return Ok(());
    };
    let fixture = root.join("unpacked/nmrglue-test-data-v0.4-dev/agilent_1d");
    require_fixture(&fixture.join("fid"))?;
    require_fixture(&fixture.join("procpar"))?;

    let spectrum = read_agilent_fid_1d_dir(&fixture)?;

    assert_eq!(spectrum.len(), 1500);
    assert_eq!(spectrum.x.unit, Unit::Seconds);
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Carbon13));
    assert_close(spectrum.metadata.frequency_mhz, Some(125.681_110_7));
    assert_eq!(spectrum.metadata.solvent.as_deref(), Some("none"));
    assert!(spectrum.imaginary.is_some());
    assert!(spectrum.intensities.iter().any(|value| value.abs() > 1.0));
    Ok(())
}

#[test]
fn parses_external_agilent_2d_fid_when_available() -> anyhow::Result<()> {
    let Some(root) = external_testdata_root() else {
        return Ok(());
    };
    let fixture = root.join("unpacked/nmrglue-test-data-v0.4-dev/agilent_2d");
    require_fixture(&fixture.join("fid"))?;
    require_fixture(&fixture.join("procpar"))?;

    let spectrum = read_agilent_fid_2d_dir(&fixture)?;

    assert_eq!(spectrum.shape(), (1500, 332));
    assert_eq!(spectrum.x.unit, Unit::Seconds);
    assert_eq!(spectrum.y.unit, Unit::Points);
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Carbon13));
    assert_close(spectrum.metadata.frequency_mhz, Some(125.690_610_7));
    assert_eq!(spectrum.metadata.solvent.as_deref(), Some("none"));
    assert!(spectrum.imaginary.is_some());
    assert!(spectrum.z.iter().any(|value| value.abs() > 1.0));
    Ok(())
}

#[test]
fn parses_external_bruker_1d_fid_when_available() -> anyhow::Result<()> {
    let Some(root) = external_testdata_root() else {
        return Ok(());
    };
    let fixture = root.join("unpacked/nmrglue-test-data-v0.4-dev/bruker_1d");
    require_fixture(&fixture.join("fid"))?;
    require_fixture(&fixture.join("acqus"))?;

    let spectrum = read_bruker_fid_1d_dir(&fixture)?;

    assert_eq!(spectrum.len(), 2048);
    assert_eq!(spectrum.x.unit, Unit::Seconds);
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_close(spectrum.metadata.frequency_mhz, Some(600.132_820_611));
    assert_eq!(spectrum.metadata.solvent.as_deref(), Some("DMSO"));
    assert_close(spectrum.metadata.temperature_k, Some(298.0));
    assert!(spectrum.imaginary.is_some());
    assert!(
        spectrum
            .intensities
            .iter()
            .any(|value| value.abs() > 1_000.0)
    );
    Ok(())
}

#[test]
fn parses_external_bruker_2d_ser_when_available() -> anyhow::Result<()> {
    let Some(root) = external_testdata_root() else {
        return Ok(());
    };
    let fixture = root.join("unpacked/nmrglue-test-data-v0.4-dev/bruker_2d");
    require_fixture(&fixture.join("ser"))?;
    require_fixture(&fixture.join("acqus"))?;
    require_fixture(&fixture.join("acqu2s"))?;

    let spectrum = read_bruker_ser_2d_dir(&fixture)?;

    assert_eq!(spectrum.shape(), (650, 600));
    assert_eq!(spectrum.x.unit, Unit::Seconds);
    assert_eq!(spectrum.y.unit, Unit::Seconds);
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_close(spectrum.metadata.frequency_mhz, Some(800.133_756));
    assert_eq!(spectrum.metadata.solvent.as_deref(), Some("H2O+D2O"));
    assert_close(spectrum.metadata.temperature_k, Some(297.9844));
    assert!(spectrum.imaginary.is_some());
    assert!(spectrum.z.iter().any(|value| value.abs() > 1_000.0));
    Ok(())
}

#[test]
fn parses_external_jeol_1d_jdf_when_available() -> anyhow::Result<()> {
    let Some(root) = external_testdata_root() else {
        return Ok(());
    };
    let fixture = root
        .join("unpacked/jeol-data-test-1.0.0/data/Rutin_3080ug200uL_DMSOd6_qHNMR_400MHz_Jeol.jdf");
    require_fixture(&fixture)?;

    let spectrum = read_jeol_jdf_1d_file(&fixture)?;

    assert_eq!(spectrum.len(), 32_768);
    assert_eq!(spectrum.x.unit, Unit::Seconds);
    assert_eq!(spectrum.metadata.origin.as_deref(), Some("JEOL"));
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(spectrum.metadata.solvent.as_deref(), Some("DMSO-D6"));
    assert_close(spectrum.metadata.frequency_mhz, Some(399.782_198_378_25));
    assert_close(spectrum.metadata.temperature_k, Some(298.15));
    assert!(spectrum.imaginary.is_some());
    assert!(spectrum.intensities.iter().any(|value| value.abs() > 1.0));
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
