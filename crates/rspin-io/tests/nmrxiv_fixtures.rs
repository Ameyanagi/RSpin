//! Always-on parser tests for redistributed `NMRXiv` fixtures.

use std::{
    fs,
    path::{Path, PathBuf},
};

use rspin_core::{Nucleus, Unit};
use rspin_io::{
    RSpinReader, read_agilent_fid_1d_dir, read_bruker_fid_1d_dir, read_bruker_ser_2d_dir,
    read_jcamp_dx_1d, read_jeol_jdf_1d_file, read_jeol_jdf_2d_file,
};

#[test]
fn reads_nmrxiv_cc0_myrcene_bruker_1h_raw() -> anyhow::Result<()> {
    let fixture = cc0_myrcene_fixture_root().join("bruker_1h_raw");
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
    let fixture = cc0_myrcene_fixture_root().join("bruker_cosy_raw");
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
    let fixture = cc0_myrcene_fixture_root().join("jeol/myrcene_1h_400mhz.jdf");
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
    let fixture = cc0_myrcene_fixture_root().join("jeol/myrcene_13c_400mhz.jdf");
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
    let fixture = cc0_myrcene_fixture_root().join("jeol/myrcene_hsqc_400mhz.jdf");
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
fn reads_nmrxiv_cc0_myrcene_jcamp_dx_6_link() -> anyhow::Result<()> {
    let fixture = cc0_myrcene_fixture_root().join("jcamp/myrcene_1h_400mhz_jcamp_dx_6_link.jdx");
    let input = fs::read_to_string(&fixture)?;
    let spectrum = read_jcamp_dx_1d(&input)?;

    assert_eq!(spectrum.len(), 104_858);
    assert_eq!(spectrum.x.unit, Unit::Hertz);
    assert_close(
        spectrum.x.values.first().copied(),
        Some(4_996.512_910_356_473),
    );
    assert_close(
        spectrum.x.values.last().copied(),
        Some(-998.690_926_573_982_9),
    );
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_close(spectrum.metadata.frequency_mhz, Some(399.782_198_378_25));
    assert_eq!(spectrum.metadata.property("jcamp_dx.version"), Some("6.0"));
    assert!(has_signal(&spectrum.intensities, 1.0e-6));
    Ok(())
}

#[test]
fn reads_nmrxiv_cc0_myrcene_13c_jcamp_dx_6_link() -> anyhow::Result<()> {
    let fixture = cc0_myrcene_fixture_root().join("jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx");
    let input = fs::read_to_string(&fixture)?;
    let spectrum = read_jcamp_dx_1d(&input)?;

    assert_eq!(spectrum.len(), 104_858);
    assert_eq!(spectrum.x.unit, Unit::Hertz);
    assert_close(
        spectrum.x.values.first().copied(),
        Some(22_678.792_958_779_2),
    );
    assert_close(
        spectrum.x.values.last().copied(),
        Some(-2_573.732_293_746_08),
    );
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Carbon13));
    assert_close(spectrum.metadata.frequency_mhz, Some(100.525_303_325_165));
    assert_eq!(spectrum.metadata.property("jcamp_dx.version"), Some("6.0"));
    assert!(has_signal(&spectrum.intensities, 1.0e-6));
    Ok(())
}

#[test]
fn reads_nmrxiv_cc_by_varian_11a_directory_bundle() -> anyhow::Result<()> {
    let fixture = cc_by_varian_fixture_root();
    let bundle = RSpinReader::new().read_path(&fixture)?;

    assert_eq!(bundle.len(), 3);
    assert_eq!(bundle.len_1d(), 3);
    assert_eq!(bundle.len_2d(), 0);
    assert!(bundle.warnings().is_empty());
    assert!(
        bundle
            .spectra()
            .iter()
            .all(|loaded| loaded.source().format == "agilent_fid")
    );
    assert!(has_source_path(&bundle, Path::new("proton_1h")));
    assert!(has_source_path(&bundle, Path::new("carbon_13c")));
    assert!(has_source_path(&bundle, Path::new("dept_13c")));

    let proton = read_agilent_fid_1d_dir(fixture.join("proton_1h"))?;
    assert_eq!(proton.len(), 16_384);
    assert_eq!(proton.x.unit, Unit::Seconds);
    assert_eq!(proton.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(proton.metadata.solvent.as_deref(), Some("cdcl3"));
    assert_close(proton.metadata.frequency_mhz, Some(399.814_675_9));
    assert!(proton.imaginary.is_some());
    assert!(has_signal(&proton.intensities, 1.0));

    let carbon = read_agilent_fid_1d_dir(fixture.join("carbon_13c"))?;
    assert_eq!(carbon.len(), 32_768);
    assert_eq!(carbon.x.unit, Unit::Seconds);
    assert_eq!(carbon.metadata.nucleus, Some(Nucleus::Carbon13));
    assert_eq!(carbon.metadata.solvent.as_deref(), Some("cdcl3"));
    assert_close(carbon.metadata.frequency_mhz, Some(100.543_835_1));
    assert!(carbon.imaginary.is_some());
    assert!(has_signal(&carbon.intensities, 1.0));

    let carbon_count = bundle
        .spectra_1d()
        .filter(|spectrum| spectrum.metadata.nucleus == Some(Nucleus::Carbon13))
        .count();
    assert_eq!(carbon_count, 2);
    Ok(())
}

fn cc0_myrcene_fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/nmrxiv/cc0/myrcene")
}

fn cc_by_varian_fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/nmrxiv/cc-by-4.0/varian_11a")
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
