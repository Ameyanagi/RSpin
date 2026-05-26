//! End-to-end check of the hypercomplex pipeline on a real, raw phase-sensitive
//! HSQC dataset (committed CC0 nmrxiv eucalyptol fixture).
//!
//! The synthetic per-quadrature-mode tests in `rspin-processing` validate the
//! assembly/transform math against known peak positions and signs. This test
//! exercises the same path on genuine raw instrument data to confirm it runs
//! end-to-end and yields a finite frequency-domain spectrum with signal.
//!
//! Exact cross-peak (1H/13C ppm) validation is intentionally not asserted: it
//! depends on the confirmed JEOL indirect-quadrature convention and reference
//! values, which are tracked separately.

use std::path::{Path, PathBuf};

use rspin::core::{QuadMode, Result, Unit};
use rspin::io::read_jeol_jdf_2d_file;
use rspin::processing::{HyperComplex2DOptions, process_hypercomplex_2d};

fn nmrxiv_fixture(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../rspin-io/testdata/nmrxiv/cc0")
        .join(relative)
}

/// Reads a raw, time-domain JEOL HSQC `.jdf`, drives the hypercomplex pipeline
/// with States-style quadrature, and asserts a finite frequency-domain spectrum
/// with signal. The JEOL reader does not record the indirect quadrature mode,
/// so it is set explicitly; HSQC is States-acquired.
fn check_raw_hsqc_pipeline(relative: &str) -> Result<()> {
    let mut raw = read_jeol_jdf_2d_file(nmrxiv_fixture(relative))?;

    // Raw, time-domain HSQC: both axes are in seconds.
    assert_eq!(raw.x.unit, Unit::Seconds);
    assert_eq!(raw.y.unit, Unit::Seconds);
    let (width, rows) = raw.shape();
    assert_eq!((width, rows), (1024, 32));

    raw.metadata.quad_mode = Some(QuadMode::States);

    let processed = process_hypercomplex_2d(&raw, &HyperComplex2DOptions::default())?;

    // 32 acquired rows collapse to 16 indirect complex points.
    assert_eq!(processed.shape(), (width, rows / 2));
    // The transform produced a frequency-domain spectrum (seconds -> ppm/Hz).
    assert_ne!(processed.x.unit, Unit::Seconds);
    assert_ne!(processed.y.unit, Unit::Seconds);
    // Output is finite and carries real signal.
    assert!(processed.z.iter().all(|value| value.is_finite()));
    assert!(processed.z.iter().any(|value| value.abs() > 0.0));

    Ok(())
}

#[test]
fn processes_raw_eucalyptol_hsqc_through_hypercomplex_pipeline() -> Result<()> {
    check_raw_hsqc_pipeline("eucalyptol/jeol/eucalyptol_hsqc_400mhz.jdf")
}

#[test]
fn processes_raw_myrcene_hsqc_through_hypercomplex_pipeline() -> Result<()> {
    check_raw_hsqc_pipeline("myrcene/jeol/myrcene_hsqc_400mhz.jdf")
}
