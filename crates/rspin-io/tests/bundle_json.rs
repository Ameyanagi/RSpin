//! Integration tests for spectrum bundle JSON serialization.

use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

use rspin_core::{Axis, Metadata, RSpinError, Spectrum1D, Spectrum2D, Unit};
use rspin_io::{
    JsonSpectrumBundle, LoadWarning, LoadedSource, LoadedSpectrum, SPECTRUM_BUNDLE_JSON_FORMAT,
    SPECTRUM_BUNDLE_JSON_VERSION, SpectrumBundle, SpectrumPathReader, SpectrumPathWriter,
    SpectrumReader, SpectrumWriter, load_spectra, read_spectrum_bundle_json,
    write_spectrum_bundle_json,
};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

#[test]
fn round_trips_versioned_bundle_json() -> anyhow::Result<()> {
    let bundle = load_spectra(zenodo_fixture_root().join("varian_1h"))?;

    let text = write_spectrum_bundle_json(&bundle)?;
    assert!(text.contains(&format!("\"format\":\"{SPECTRUM_BUNDLE_JSON_FORMAT}\"")));
    assert!(text.contains(&format!("\"version\":{SPECTRUM_BUNDLE_JSON_VERSION}")));
    assert!(text.contains("\"bundle\""));

    let parsed = read_spectrum_bundle_json(&text)?;
    assert_bundle_summary(&parsed, &bundle);

    let trait_text = SpectrumWriter::write_string(&JsonSpectrumBundle, &bundle)?;
    let trait_bundle = SpectrumReader::read_str(&JsonSpectrumBundle, &trait_text)?;
    assert_bundle_summary(&trait_bundle, &bundle);
    assert_eq!(format!("{JsonSpectrumBundle:?}"), "JsonSpectrumBundle");
    Ok(())
}

#[test]
fn reads_legacy_raw_bundle_json() -> anyhow::Result<()> {
    let bundle = load_spectra(nmredata_fixture_root())?;
    let raw = serde_json::to_string(&bundle)?;

    let parsed = read_spectrum_bundle_json(&raw)?;
    assert_eq!(parsed, bundle);
    assert_eq!(parsed.molecules().len(), 1);
    Ok(())
}

#[test]
fn path_traits_read_and_write_bundle_json() -> anyhow::Result<()> {
    let bundle = load_spectra(zenodo_fixture_root().join("bruker_without_expno"))?;
    let path = temp_json_path("rspin-bundle-path");

    SpectrumPathWriter::write_path(&JsonSpectrumBundle, &bundle, &path)?;
    let parsed = SpectrumPathReader::read_path(&JsonSpectrumBundle, &path)?;
    let _ = fs::remove_file(&path);

    assert_bundle_summary(&parsed, &bundle);
    Ok(())
}

#[test]
fn unified_loader_reads_bundle_json_file() -> anyhow::Result<()> {
    let bundle = load_spectra(zenodo_fixture_root().join("varian_1h"))?;
    let path = temp_json_path("rspin-bundle-loader");
    fs::write(&path, write_spectrum_bundle_json(&bundle)?)?;

    let loaded = load_spectra(&path)?;
    let _ = fs::remove_file(&path);

    assert_bundle_summary(&loaded, &bundle);
    Ok(())
}

#[test]
fn constructs_bundle_with_chainable_public_api() -> anyhow::Result<()> {
    let one_d = Spectrum1D::new(
        Axis::linear_ppm(0.0, 1.0, 2)?,
        vec![1.0, 2.0],
        Metadata::named("constructed 1d"),
    )?;
    let two_d = Spectrum2D::new(
        Axis::linear("direct", Unit::Seconds, 0.0, 0.001, 2)?,
        Axis::linear("indirect", Unit::Seconds, 0.0, 0.002, 2)?,
        vec![1.0, 2.0, 3.0, 4.0],
        Metadata::named("constructed 2d"),
    )?;

    let bundle = SpectrumBundle::new()
        .with_1d(
            one_d,
            LoadedSource::new(Some(PathBuf::from("one.jdx")), "jcamp_dx"),
        )
        .with_2d(
            two_d,
            LoadedSource::new(Some(PathBuf::from("two.jdf")), "jeol_jdf"),
        )
        .with_warning(LoadWarning::new(
            Some(PathBuf::from("ignored.txt")),
            "unsupported sidecar",
        ));

    assert_eq!(bundle.len(), 2);
    assert_eq!(bundle.spectra_1d().count(), 1);
    assert_eq!(bundle.spectra_2d().count(), 1);
    assert_eq!(bundle.warnings().len(), 1);

    let text = write_spectrum_bundle_json(&bundle)?;
    let parsed = read_spectrum_bundle_json(&text)?;
    assert_bundle_summary(&parsed, &bundle);
    Ok(())
}

#[test]
fn rejects_wrong_bundle_json_headers() {
    let wrong_format = read_spectrum_bundle_json(
        r#"{"format":"rspin.spectrum_1d","version":1,"bundle":{"spectra":[]}}"#,
    );
    assert!(matches!(wrong_format, Err(RSpinError::Parse { .. })));

    let unsupported_version = read_spectrum_bundle_json(
        r#"{"format":"rspin.spectrum_bundle","version":2,"bundle":{"spectra":[]}}"#,
    );
    assert!(matches!(
        unsupported_version,
        Err(RSpinError::Unsupported {
            feature: "spectrum bundle JSON version"
        })
    ));
}

fn zenodo_fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/zenodo_7100132")
}

fn nmredata_fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/bundle_nmredata")
}

fn temp_json_path(prefix: &str) -> PathBuf {
    let stamp = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("{prefix}-{}-{stamp}.json", std::process::id()))
}

fn assert_bundle_summary(actual: &SpectrumBundle, expected: &SpectrumBundle) {
    assert_eq!(actual.len(), expected.len());
    assert_eq!(actual.molecules(), expected.molecules());
    assert_eq!(actual.warnings(), expected.warnings());

    for (actual, expected) in actual.spectra().iter().zip(expected.spectra()) {
        assert_eq!(actual.source(), expected.source());
        match (actual, expected) {
            (
                LoadedSpectrum::OneD {
                    spectrum: actual, ..
                },
                LoadedSpectrum::OneD {
                    spectrum: expected, ..
                },
            ) => {
                assert_eq!(actual.len(), expected.len());
                assert_eq!(actual.x.unit, expected.x.unit);
                assert_eq!(actual.metadata.nucleus, expected.metadata.nucleus);
            }
            (
                LoadedSpectrum::TwoD {
                    spectrum: actual, ..
                },
                LoadedSpectrum::TwoD {
                    spectrum: expected, ..
                },
            ) => {
                assert_eq!(actual.shape(), expected.shape());
                assert_eq!(actual.x.unit, expected.x.unit);
                assert_eq!(actual.y.unit, expected.y.unit);
                assert_eq!(actual.metadata.nucleus, expected.metadata.nucleus);
            }
            _ => panic!("bundle spectrum dimensions differ"),
        }
    }
}
