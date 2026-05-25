//! Always-on parser tests for redistributed Harvard Dataverse fixtures.

use std::{
    fs,
    path::{Path, PathBuf},
};

use rspin_core::{Nucleus, Unit};
use rspin_io::{RSpinReader, inspect_jeol_jdf_file, read_jcamp_dx_1d, read_jeol_jdf_1d_file};

#[test]
fn reads_dataverse_cc0_rutin_jcamp_1h() -> anyhow::Result<()> {
    let fixture = rutin_fixture_root().join("jcamp/rutin_qh_400mhz.jdx");
    let input = fs::read_to_string(&fixture)?;
    let spectrum = read_jcamp_dx_1d(&input)?;

    assert_eq!(spectrum.len(), 52_430);
    assert_eq!(spectrum.x.unit, Unit::Hertz);
    assert_close(
        spectrum.x.values.first().copied(),
        Some(7_604.450_041_814_471),
    );
    assert_close(
        spectrum.x.values.last().copied(),
        Some(-408.370_471_006_009_9),
    );
    assert_eq!(spectrum.metadata.origin.as_deref(), Some("JEOL"));
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(spectrum.metadata.solvent.as_deref(), Some("DMSO-D6"));
    assert_close(spectrum.metadata.frequency_mhz, Some(399.782_198_378_25));
    assert_eq!(spectrum.metadata.property("jcamp_dx.version"), Some("6.0"));
    assert!(has_signal(&spectrum.intensities, 1.0e-6));
    Ok(())
}

#[test]
fn reads_dataverse_cc0_rutin_jcamp_13c() -> anyhow::Result<()> {
    let fixture = rutin_fixture_root().join("jcamp/rutin_13c_400mhz.jdx");
    let input = fs::read_to_string(&fixture)?;
    let spectrum = read_jcamp_dx_1d(&input)?;

    assert_eq!(spectrum.len(), 52_430);
    assert_eq!(spectrum.x.unit, Unit::Hertz);
    assert_close(
        spectrum.x.values.first().copied(),
        Some(22_678.792_958_779_202),
    );
    assert_close(
        spectrum.x.values.last().copied(),
        Some(-2_573.732_293_746_08),
    );
    assert_eq!(spectrum.metadata.origin.as_deref(), Some("JEOL"));
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Carbon13));
    assert_eq!(spectrum.metadata.solvent.as_deref(), Some("DMSO-D6"));
    assert_close(spectrum.metadata.frequency_mhz, Some(100.525_303_325_165));
    assert_eq!(spectrum.metadata.property("jcamp_dx.version"), Some("6.0"));
    assert!(has_signal(&spectrum.intensities, 1.0e-6));
    Ok(())
}

#[test]
fn reads_dataverse_cc0_rutin_jeol_1h_and_13c() -> anyhow::Result<()> {
    let proton_path = rutin_fixture_root().join("jeol/rutin_qhnmr_400mhz.jdf");
    let carbon_path = rutin_fixture_root().join("jeol/rutin_13cnmr_400mhz.jdf");

    let proton_info = inspect_jeol_jdf_file(&proton_path)?;
    let proton = read_jeol_jdf_1d_file(&proton_path)?;
    assert_eq!(proton_info.dimension_count, 1);
    assert_eq!(
        proton_info.point_counts.first().copied(),
        Some(proton.len())
    );
    assert_eq!(proton.x.unit, Unit::Seconds);
    assert_eq!(proton.metadata.origin.as_deref(), Some("JEOL"));
    assert_eq!(proton.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(proton.metadata.solvent.as_deref(), Some("DMSO-D6"));
    assert_close(proton.metadata.frequency_mhz, Some(399.782_198_378_250_03));
    assert!(has_signal(&proton.intensities, 1.0e-6));

    let carbon_info = inspect_jeol_jdf_file(&carbon_path)?;
    let carbon = read_jeol_jdf_1d_file(&carbon_path)?;
    assert_eq!(carbon_info.dimension_count, 1);
    assert_eq!(
        carbon_info.point_counts.first().copied(),
        Some(carbon.len())
    );
    assert_eq!(carbon.x.unit, Unit::Seconds);
    assert_eq!(carbon.metadata.origin.as_deref(), Some("JEOL"));
    assert_eq!(carbon.metadata.nucleus, Some(Nucleus::Carbon13));
    assert_eq!(carbon.metadata.solvent.as_deref(), Some("DMSO-D6"));
    assert_close(carbon.metadata.frequency_mhz, Some(100.525_303_325_165_41));
    assert!(has_signal(&carbon.intensities, 1.0e-6));
    Ok(())
}

#[test]
fn unified_loader_reads_dataverse_cc0_rutin_subset() -> anyhow::Result<()> {
    let bundle = RSpinReader::new().read_path(rutin_fixture_root())?;

    assert_eq!(bundle.len(), 4);
    assert_eq!(bundle.len_1d(), 4);
    assert_eq!(bundle.len_2d(), 0);
    assert!(bundle.warnings().is_empty());
    assert_eq!(bundle.source_format_count("jcamp_dx"), 2);
    assert_eq!(bundle.source_format_count("jdf"), 2);
    assert!(has_source_path(
        &bundle,
        Path::new("jcamp/rutin_qh_400mhz.jdx")
    ));
    assert!(has_source_path(
        &bundle,
        Path::new("jcamp/rutin_13c_400mhz.jdx")
    ));
    assert!(has_source_path(
        &bundle,
        Path::new("jeol/rutin_qhnmr_400mhz.jdf")
    ));
    assert!(has_source_path(
        &bundle,
        Path::new("jeol/rutin_13cnmr_400mhz.jdf")
    ));
    Ok(())
}

fn rutin_fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/dataverse/cc0/rutin")
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
