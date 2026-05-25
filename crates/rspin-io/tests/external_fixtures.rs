//! Opt-in local checks for external NMR fixture caches.
//!
//! These tests intentionally skip unless `RSPIN_EXTERNAL_TESTDATA` points to a
//! local cache. Fixture files are not vendored into the repository.

use std::{
    env, fs,
    path::{Path, PathBuf},
};

use rspin_core::{Nucleus, RSpinError, Unit};
use rspin_io::{
    RSpinReader, Spectrum1DBytesFormat, Spectrum1DPathFormat, Spectrum2DBytesFormat,
    Spectrum2DPathFormat, detect_spectrum1d_path_format, detect_spectrum2d_path_format,
    read_agilent_arrayed_fid_1d_dir, read_agilent_arrayed_fid_2d_dir, read_agilent_fid_1d_dir,
    read_agilent_fid_2d_dir, read_bruker_fid_1d_dir, read_bruker_ser_2d_dir, read_jcamp_dx_1d,
    read_jeol_jdf_1d_file, read_jeol_jdf_2d_file, read_nmrml_1d_file, read_nmrml_2d_file,
    read_spectrum1d_bytes_as, read_spectrum2d_bytes_as,
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
fn parses_external_varian_arrayed_1d_fids_when_available() -> anyhow::Result<()> {
    let Some(root) = external_testdata_root() else {
        return Ok(());
    };
    let fixture = root.join(
        "unpacked/nmrglue-example-data-all-none/separate/separate_1d_varian/arrayed_data.dir",
    );
    require_fixture(&fixture.join("fid"))?;
    require_fixture(&fixture.join("procpar"))?;

    let spectra = read_agilent_arrayed_fid_1d_dir(&fixture)?;
    let spectra_from_file = read_agilent_arrayed_fid_1d_dir(fixture.join("fid"))?;
    let bundle = RSpinReader::new().read_path(&fixture)?;
    let file_bundle = RSpinReader::new().read_path(fixture.join("fid"))?;

    assert_eq!(spectra.len(), 26);
    assert_eq!(spectra_from_file.len(), 26);
    assert!(spectra.iter().all(|spectrum| spectrum.len() == 1500));
    assert!(
        spectra
            .iter()
            .all(|spectrum| spectrum.x.unit == Unit::Seconds)
    );
    assert!(
        spectra
            .iter()
            .all(|spectrum| spectrum.metadata.nucleus == Some(Nucleus::Carbon13))
    );
    assert_eq!(
        spectra[0].metadata.property("agilent.array.index"),
        Some("0")
    );
    assert_eq!(
        spectra[25].metadata.property("agilent.array.index"),
        Some("25")
    );
    assert_eq!(
        spectra[25].metadata.property("agilent.array.count"),
        Some("26")
    );
    assert!(
        spectra
            .iter()
            .any(|spectrum| spectrum.intensities.iter().any(|value| value.abs() > 1.0))
    );

    assert_eq!(bundle.len(), 26);
    assert_eq!(bundle.spectra_1d().count(), 26);
    assert_eq!(bundle.spectra_2d().count(), 0);
    assert!(bundle.warnings().is_empty());
    assert_eq!(source_format_count(&bundle, "agilent_fid"), 26);
    assert_eq!(file_bundle.len(), 26);
    assert_eq!(file_bundle.spectra_1d().count(), 26);
    assert!(file_bundle.warnings().is_empty());
    Ok(())
}

#[test]
fn parses_external_varian_arrayed_2d_fids_when_available() -> anyhow::Result<()> {
    let Some(root) = external_testdata_root() else {
        return Ok(());
    };
    let fixture = root.join(
        "unpacked/nmrglue-example-data-all-none/separate/separate_2d_varian/arrayed_data.dir",
    );
    require_fixture(&fixture.join("fid"))?;
    require_fixture(&fixture.join("procpar"))?;

    let spectra = read_agilent_arrayed_fid_2d_dir(&fixture)?;
    let spectra_from_file = read_agilent_arrayed_fid_2d_dir(fixture.join("fid"))?;
    let bundle = RSpinReader::new().read_path(&fixture)?;
    let file_bundle = RSpinReader::new().read_path(fixture.join("fid"))?;

    assert_eq!(spectra.len(), 6);
    assert_eq!(spectra_from_file.len(), 6);
    assert!(
        spectra
            .iter()
            .all(|spectrum| spectrum.shape() == (1400, 810))
    );
    assert!(
        spectra
            .iter()
            .all(|spectrum| spectrum.x.unit == Unit::Seconds)
    );
    assert!(
        spectra
            .iter()
            .all(|spectrum| spectrum.y.unit == Unit::Points)
    );
    assert!(
        spectra
            .iter()
            .all(|spectrum| spectrum.metadata.nucleus == Some(Nucleus::Carbon13))
    );
    assert_eq!(
        spectra[0].metadata.property("agilent.array.index"),
        Some("0")
    );
    assert_eq!(
        spectra[5].metadata.property("agilent.array.index"),
        Some("5")
    );
    assert_eq!(
        spectra[5].metadata.property("agilent.array.count"),
        Some("6")
    );
    assert_eq!(
        spectra[5]
            .metadata
            .property("agilent.array.traces_per_spectrum"),
        Some("810")
    );
    assert!(
        spectra
            .iter()
            .any(|spectrum| { spectrum.z.iter().any(|value| value.abs() > 1.0) })
    );

    assert_eq!(bundle.len(), 6);
    assert_eq!(bundle.spectra_1d().count(), 0);
    assert_eq!(bundle.spectra_2d().count(), 6);
    assert!(bundle.warnings().is_empty());
    assert_eq!(source_format_count(&bundle, "agilent_fid"), 6);
    assert_eq!(file_bundle.len(), 6);
    assert_eq!(file_bundle.spectra_2d().count(), 6);
    assert!(file_bundle.warnings().is_empty());
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

#[test]
fn parses_external_jeol_2d_jdf_when_available() -> anyhow::Result<()> {
    let Some(root) = external_testdata_root() else {
        return Ok(());
    };
    let fixture = root
        .join("unpacked/jeol-data-test-1.0.0/data/Rutin_3080ug200uL_DMSOd6_HSQC_400MHz_Jeol.jdf");
    require_fixture(&fixture)?;

    let spectrum = read_jeol_jdf_2d_file(&fixture)?;

    assert_eq!(spectrum.shape(), (4096, 256));
    assert_eq!(spectrum.x.unit, Unit::Seconds);
    assert_eq!(spectrum.y.unit, Unit::Seconds);
    assert_eq!(spectrum.metadata.origin.as_deref(), Some("JEOL"));
    assert!(spectrum.imaginary.is_some());
    assert!(spectrum.z.iter().any(|value| value.abs() > 1.0e-12));
    Ok(())
}

#[test]
fn routes_external_jeol_jdf_bytes_when_available() -> anyhow::Result<()> {
    let Some(root) = external_testdata_root() else {
        return Ok(());
    };
    let one_d_fixture = root
        .join("unpacked/jeol-data-test-1.0.0/data/Rutin_3080ug200uL_DMSOd6_qHNMR_400MHz_Jeol.jdf");
    let two_d_fixture = root
        .join("unpacked/jeol-data-test-1.0.0/data/Rutin_3080ug200uL_DMSOd6_HSQC_400MHz_Jeol.jdf");
    require_fixture(&one_d_fixture)?;
    require_fixture(&two_d_fixture)?;

    let one_d = read_spectrum1d_bytes_as(
        &fs::read(&one_d_fixture)?,
        Spectrum1DBytesFormat::JeolJdf,
        None,
    )?;
    let two_d = read_spectrum2d_bytes_as(
        &fs::read(&two_d_fixture)?,
        Spectrum2DBytesFormat::JeolJdf,
        None,
        None,
    )?;

    assert_eq!(one_d.len(), 32_768);
    assert_eq!(one_d.x.unit, Unit::Seconds);
    assert_eq!(one_d.metadata.origin.as_deref(), Some("JEOL"));
    assert_eq!(one_d.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert!(one_d.imaginary.is_some());
    assert_eq!(two_d.shape(), (4096, 256));
    assert_eq!(two_d.x.unit, Unit::Seconds);
    assert_eq!(two_d.y.unit, Unit::Seconds);
    assert_eq!(two_d.metadata.origin.as_deref(), Some("JEOL"));
    assert!(two_d.imaginary.is_some());
    Ok(())
}

#[test]
fn unified_loader_reads_external_vendor_paths_when_available() -> anyhow::Result<()> {
    let Some(root) = external_testdata_root() else {
        return Ok(());
    };
    let agilent_1d = root.join("unpacked/nmrglue-test-data-v0.4-dev/agilent_1d");
    let agilent_2d = root.join("unpacked/nmrglue-test-data-v0.4-dev/agilent_2d");
    let bruker_1d = root.join("unpacked/nmrglue-test-data-v0.4-dev/bruker_1d");
    let bruker_2d = root.join("unpacked/nmrglue-test-data-v0.4-dev/bruker_2d");
    let jeol_1d = root
        .join("unpacked/jeol-data-test-1.0.0/data/Rutin_3080ug200uL_DMSOd6_qHNMR_400MHz_Jeol.jdf");
    let jeol_2d = root
        .join("unpacked/jeol-data-test-1.0.0/data/Rutin_3080ug200uL_DMSOd6_HSQC_400MHz_Jeol.jdf");

    for path in [
        &agilent_1d,
        &agilent_2d,
        &bruker_1d,
        &bruker_2d,
        &jeol_1d,
        &jeol_2d,
    ] {
        require_path(path)?;
    }

    let bundle = RSpinReader::new().read_paths([
        agilent_1d, agilent_2d, bruker_1d, bruker_2d, jeol_1d, jeol_2d,
    ])?;

    assert_eq!(bundle.len(), 6);
    assert_eq!(bundle.spectra_1d().count(), 3);
    assert_eq!(bundle.spectra_2d().count(), 3);
    assert!(bundle.warnings().is_empty());
    assert_eq!(source_format_count(&bundle, "agilent_fid"), 2);
    assert_eq!(source_format_count(&bundle, "bruker_fid"), 1);
    assert_eq!(source_format_count(&bundle, "bruker_ser"), 1);
    assert_eq!(source_format_count(&bundle, "jeol_jdf"), 2);
    Ok(())
}

#[test]
fn auto_detects_external_vendor_path_formats_when_available() -> anyhow::Result<()> {
    let Some(root) = external_testdata_root() else {
        return Ok(());
    };
    let agilent_1d = root.join("unpacked/nmrglue-test-data-v0.4-dev/agilent_1d");
    let agilent_2d = root.join("unpacked/nmrglue-test-data-v0.4-dev/agilent_2d");
    let bruker_1d = root.join("unpacked/nmrglue-test-data-v0.4-dev/bruker_1d");
    let bruker_2d = root.join("unpacked/nmrglue-test-data-v0.4-dev/bruker_2d");
    let jeol_1d = root
        .join("unpacked/jeol-data-test-1.0.0/data/Rutin_3080ug200uL_DMSOd6_qHNMR_400MHz_Jeol.jdf");
    let jeol_2d = root
        .join("unpacked/jeol-data-test-1.0.0/data/Rutin_3080ug200uL_DMSOd6_HSQC_400MHz_Jeol.jdf");

    assert_eq!(
        detect_spectrum1d_path_format(&agilent_1d)?,
        Spectrum1DPathFormat::AgilentFid
    );
    let wrong_dimension = detect_spectrum2d_path_format(&agilent_1d)
        .expect_err("one-dimensional Agilent FID should not route to 2D");
    assert_unsupported(&wrong_dimension);
    assert_eq!(
        detect_spectrum2d_path_format(&agilent_2d)?,
        Spectrum2DPathFormat::AgilentFid
    );
    let wrong_dimension = detect_spectrum1d_path_format(&agilent_2d)
        .expect_err("two-dimensional Agilent FID should not route to 1D");
    assert_unsupported(&wrong_dimension);
    assert_eq!(
        detect_spectrum1d_path_format(&bruker_1d)?,
        Spectrum1DPathFormat::BrukerFid
    );
    assert_eq!(
        detect_spectrum2d_path_format(&bruker_2d)?,
        Spectrum2DPathFormat::BrukerSer
    );
    assert_eq!(
        detect_spectrum1d_path_format(&jeol_1d)?,
        Spectrum1DPathFormat::JeolJdf
    );
    let wrong_dimension = detect_spectrum2d_path_format(&jeol_1d)
        .expect_err("one-dimensional JEOL JDF should not route to 2D");
    assert_unsupported(&wrong_dimension);
    assert_eq!(
        detect_spectrum2d_path_format(&jeol_2d)?,
        Spectrum2DPathFormat::JeolJdf
    );
    let wrong_dimension = detect_spectrum1d_path_format(&jeol_2d)
        .expect_err("two-dimensional JEOL JDF should not route to 1D");
    assert_unsupported(&wrong_dimension);
    Ok(())
}

#[test]
fn unified_loader_reads_external_nmrxiv_varian_fids_when_available() -> anyhow::Result<()> {
    let Some(root) = external_testdata_root() else {
        return Ok(());
    };
    let study = root.join("nmrxiv/cc-by-4.0/unpacked/S332_11a_Varian/11a_Varian");
    require_path(&study)?;

    let bundle = RSpinReader::new().read_path(&study)?;

    assert_eq!(bundle.len(), 3);
    assert_eq!(bundle.spectra_1d().count(), 3);
    assert_eq!(bundle.spectra_2d().count(), 0);
    assert!(bundle.warnings().is_empty());
    assert_eq!(source_format_count(&bundle, "agilent_fid"), 3);
    assert_eq!(
        bundle
            .spectra_1d()
            .filter(|spectrum| spectrum.metadata.nucleus == Some(Nucleus::Hydrogen1))
            .count(),
        1
    );
    assert_eq!(
        bundle
            .spectra_1d()
            .filter(|spectrum| spectrum.metadata.nucleus == Some(Nucleus::Carbon13))
            .count(),
        2
    );
    assert!(bundle.spectra_1d().all(|spectrum| {
        spectrum.imaginary.is_some() && spectrum.intensities.iter().any(|value| value.abs() > 1.0)
    }));
    Ok(())
}

#[test]
fn parses_external_nmrml_complex128_fixture_when_available() -> anyhow::Result<()> {
    let Some(root) = external_testdata_root() else {
        return Ok(());
    };
    let fixture = root.join("nmrml/examples/HMDB00005.nmrML");
    require_fixture(&fixture)?;

    let spectrum = read_nmrml_1d_file(&fixture)?;

    assert_eq!(spectrum.len(), 32_768);
    assert_eq!(spectrum.x.unit, Unit::Ppm);
    assert_close(spectrum.x.values.first().copied(), Some(10.791_613_341_5));
    assert_close(spectrum.x.values.last().copied(), Some(-1.219_962_947_1));
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_close(spectrum.metadata.frequency_mhz, Some(599.4));
    assert_close(spectrum.metadata.temperature_k, Some(299.15));
    assert!(spectrum.intensities.iter().any(|value| value.abs() > 0.5));
    Ok(())
}

#[test]
fn parses_external_nmrml_compressed_float64_fixture_when_available() -> anyhow::Result<()> {
    let Some(root) = external_testdata_root() else {
        return Ok(());
    };
    let fixture = root.join("nmrml/examples/MMBBI_10M12-CE01-1a.nmrML");
    require_fixture(&fixture)?;

    let spectrum = read_nmrml_1d_file(&fixture)?;

    assert_eq!(spectrum.len(), 32_768);
    assert_eq!(spectrum.x.unit, Unit::Ppm);
    assert_close(spectrum.x.values.first().copied(), Some(11.099_15));
    assert_close(spectrum.x.values.last().copied(), Some(-0.901_812));
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_close(spectrum.metadata.frequency_mhz, Some(500.162_500_8));
    assert_close(spectrum.metadata.temperature_k, Some(300.0));
    assert!(
        spectrum
            .intensities
            .iter()
            .any(|value| value.abs() > 1_000.0)
    );
    Ok(())
}

#[test]
fn parses_external_nmrml_fid_fixture_when_available() -> anyhow::Result<()> {
    let Some(root) = external_testdata_root() else {
        return Ok(());
    };
    let fixture = root.join("nmrml/examples/FAM013_AHTM.PROTON_04.nmrML");
    require_fixture(&fixture)?;

    let spectrum = read_nmrml_1d_file(&fixture)?;

    assert_eq!(spectrum.len(), 32_768);
    assert_eq!(spectrum.x.unit, Unit::Seconds);
    assert_close(spectrum.x.values.first().copied(), Some(0.0));
    assert_close(
        spectrum.x.values.last().copied(),
        Some(2.726_214_400_006_979_2),
    );
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_close(spectrum.metadata.frequency_mhz, Some(599.831_161_7));
    assert_close(spectrum.metadata.temperature_k, Some(299.15));
    assert!(spectrum.imaginary.is_some());
    assert!(
        spectrum
            .intensities
            .iter()
            .any(|value| value.abs() > 100_000.0)
    );
    Ok(())
}

#[test]
fn parses_external_nmrml_2d_fid_fixture_when_available() -> anyhow::Result<()> {
    let Some(root) = external_testdata_root() else {
        return Ok(());
    };
    let fixture = root.join("nmrml/examples/bmse000400-exp06.nmrML");
    require_fixture(&fixture)?;

    let spectrum = read_nmrml_2d_file(&fixture)?;

    assert_eq!(spectrum.shape(), (4096, 256));
    assert_eq!(spectrum.x.unit, Unit::Seconds);
    assert_eq!(spectrum.y.unit, Unit::Seconds);
    assert_close(spectrum.x.values.first().copied(), Some(0.0));
    assert_close(
        spectrum.x.values.last().copied(),
        Some(0.584_765_999_999_999_9),
    );
    assert_close(spectrum.y.values.first().copied(), Some(0.0));
    assert_close(
        spectrum.y.values.last().copied(),
        Some(0.033_813_000_000_000_02),
    );
    assert_eq!(spectrum.metadata.name.as_deref(), Some("2D [1H,13C]-HSQC"));
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_close(spectrum.metadata.frequency_mhz, Some(499.842_349_248));
    assert!(spectrum.imaginary.is_some());
    assert!(
        spectrum
            .imaginary
            .as_deref()
            .is_some_and(|values| values.iter().any(|value| value.abs() > 1.0))
    );
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

fn require_path(path: &Path) -> anyhow::Result<()> {
    if path.exists() {
        return Ok(());
    }

    anyhow::bail!(
        "missing external fixture path at {}; check RSPIN_EXTERNAL_TESTDATA",
        path.display()
    );
}

fn source_format_count(bundle: &rspin_io::SpectrumBundle, format: &str) -> usize {
    bundle
        .spectra()
        .iter()
        .filter(|spectrum| spectrum.source().format == format)
        .count()
}

fn assert_close(actual: Option<f64>, expected: Option<f64>) {
    match (actual, expected) {
        (Some(left), Some(right)) => assert!((left - right).abs() < 1e-12, "{left} != {right}"),
        (left, right) => assert_eq!(left, right),
    }
}

fn assert_unsupported(error: &RSpinError) {
    assert!(matches!(error, RSpinError::Unsupported { .. }));
}
