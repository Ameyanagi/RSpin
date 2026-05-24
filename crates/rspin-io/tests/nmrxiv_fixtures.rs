//! Always-on parser tests for redistributed `NMRXiv` fixtures.

use std::{fs, path::PathBuf};

use rspin_core::{Nucleus, RSpinError, Unit};
use rspin_io::{
    read_bruker_fid_1d_dir, read_bruker_ser_2d_dir, read_jcamp_dx_1d, read_jeol_jdf_1d_file,
    read_jeol_jdf_2d_file,
};

#[test]
fn reads_nmrxiv_cc0_myrcene_bruker_1h_raw() -> anyhow::Result<()> {
    let fixture = fixture_root().join("bruker_1h_raw");
    let spectrum = read_bruker_fid_1d_dir(&fixture)?;

    assert_eq!(spectrum.len(), 108_399);
    assert_eq!(spectrum.x.unit, Unit::Seconds);
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(spectrum.metadata.solvent.as_deref(), Some("CDCl3"));
    assert_close(spectrum.metadata.frequency_mhz, Some(900.077_600_296));
    assert!(spectrum.imaginary.is_some());
    assert!(has_signal(&spectrum.intensities, 1_000.0));
    Ok(())
}

#[test]
fn reads_nmrxiv_cc0_myrcene_bruker_cosy_raw() -> anyhow::Result<()> {
    let fixture = fixture_root().join("bruker_cosy_raw");
    let spectrum = read_bruker_ser_2d_dir(&fixture)?;

    assert_eq!(spectrum.shape(), (2048, 512));
    assert_eq!(spectrum.x.unit, Unit::Seconds);
    assert_eq!(spectrum.y.unit, Unit::Points);
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(spectrum.metadata.solvent.as_deref(), Some("CDCl3"));
    assert_close(spectrum.metadata.frequency_mhz, Some(900.076_700_222));
    assert!(spectrum.imaginary.is_some());
    assert!(has_signal(&spectrum.z, 1_000.0));
    Ok(())
}

#[test]
fn reads_nmrxiv_cc0_myrcene_jeol_1h_jdf() -> anyhow::Result<()> {
    let fixture = fixture_root().join("jeol/myrcene_1h_400mhz.jdf");
    let spectrum = read_jeol_jdf_1d_file(&fixture)?;

    assert_eq!(spectrum.len(), 65_536);
    assert_eq!(spectrum.x.unit, Unit::Seconds);
    assert_eq!(spectrum.metadata.origin.as_deref(), Some("JEOL"));
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(spectrum.metadata.solvent.as_deref(), Some("CHLOROFORM-D"));
    assert_close(
        spectrum.metadata.frequency_mhz,
        Some(399.782_198_378_250_03),
    );
    assert!(spectrum.imaginary.is_some());
    assert!(has_signal(&spectrum.intensities, 1.0e-6));
    Ok(())
}

#[test]
fn reads_nmrxiv_cc0_myrcene_jeol_13c_jdf() -> anyhow::Result<()> {
    let fixture = fixture_root().join("jeol/myrcene_13c_400mhz.jdf");
    let spectrum = read_jeol_jdf_1d_file(&fixture)?;

    assert_eq!(spectrum.len(), 65_536);
    assert_eq!(spectrum.x.unit, Unit::Seconds);
    assert_eq!(spectrum.metadata.origin.as_deref(), Some("JEOL"));
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Carbon13));
    assert_eq!(spectrum.metadata.solvent.as_deref(), Some("CHLOROFORM-D"));
    assert_close(
        spectrum.metadata.frequency_mhz,
        Some(100.525_303_325_165_41),
    );
    assert!(spectrum.imaginary.is_some());
    assert!(has_signal(&spectrum.intensities, 1.0e-6));
    Ok(())
}

#[test]
fn reads_nmrxiv_cc0_myrcene_jeol_hsqc_jdf() -> anyhow::Result<()> {
    let fixture = fixture_root().join("jeol/myrcene_hsqc_400mhz.jdf");
    let spectrum = read_jeol_jdf_2d_file(&fixture)?;

    assert_eq!(spectrum.shape(), (1024, 32));
    assert_eq!(spectrum.x.unit, Unit::Seconds);
    assert_eq!(spectrum.y.unit, Unit::Seconds);
    assert_eq!(spectrum.metadata.origin.as_deref(), Some("JEOL"));
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_close(
        spectrum.metadata.frequency_mhz,
        Some(399.782_198_378_250_03),
    );
    assert!(spectrum.imaginary.is_some());
    assert!(has_signal(&spectrum.z, 1.0e-12));
    Ok(())
}

#[test]
fn rejects_nmrxiv_cc0_myrcene_jcamp_dx_6_link_until_supported() -> anyhow::Result<()> {
    let fixture = fixture_root().join("jcamp/myrcene_1h_400mhz_jcamp_dx_6_link.jdx");
    let input = fs::read_to_string(&fixture)?;

    match read_jcamp_dx_1d(&input) {
        Ok(_) => anyhow::bail!("JCAMP-DX 6.0 LINK fixture should remain version-gated"),
        Err(RSpinError::Unsupported { feature }) => {
            assert_eq!(feature, "JCAMP-DX version");
            Ok(())
        }
        Err(error) => Err(error.into()),
    }
}

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/nmrxiv/cc0/myrcene")
}

fn has_signal(values: &[f64], threshold: f64) -> bool {
    values.iter().any(|value| value.abs() > threshold)
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
