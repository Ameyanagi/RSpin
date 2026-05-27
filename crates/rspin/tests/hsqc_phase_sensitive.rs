//! Validates the four-plane phase-sensitive JEOL HSQC path: submatrix
//! de-tiling, four-plane hypercomplex read, ¹H/¹³C axis calibration, the
//! direct/indirect transform, and the hypercomplex-modulus display.
//!
//! - The committed CC0 eucalyptol fixture (32 t₁ increments) is an always-on
//!   structural check (correct axes, aliphatic ¹H peak).
//! - The higher-resolution Rutin fixture (256 t₁) from the `jeol-data-test`
//!   submodule is a strong correctness check: its strongest cross-peak is the
//!   rhamnose methyl at ¹H ≈ 0.9 ppm / ¹³C ≈ 18 ppm. Skipped when the submodule
//!   is not initialized.

use std::path::{Path, PathBuf};

use rspin::core::{Spectrum2D, Unit};
use rspin::io::read_jeol_jdf_2d_hypercomplex_file;
use rspin::processing::{HyperComplex2DOptions, process_hypercomplex_planes_magnitude};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// `(x_index, y_index)` of the strongest magnitude point.
fn argmax(spectrum: &Spectrum2D) -> (f64, f64) {
    let (width, height) = spectrum.shape();
    let (mut bx, mut by, mut best) = (0usize, 0usize, f64::NEG_INFINITY);
    for y in 0..height {
        for x in 0..width {
            let value = spectrum.value_at(x, y).unwrap_or(0.0);
            if value > best {
                best = value;
                bx = x;
                by = y;
            }
        }
    }
    (spectrum.x.values[bx], spectrum.y.values[by])
}

#[test]
fn eucalyptol_hsqc_has_calibrated_axes_and_aliphatic_peak() -> rspin::core::Result<()> {
    let fixture = repo_root()
        .join("crates/rspin-io/testdata/nmrxiv/cc0/eucalyptol/jeol/eucalyptol_hsqc_400mhz.jdf");
    let raw = read_jeol_jdf_2d_hypercomplex_file(&fixture)?;
    assert_eq!(raw.shape(), (1024, 32));

    let options = HyperComplex2DOptions::default().with_indirect_zero_fill(128);
    let spectrum = process_hypercomplex_planes_magnitude(&raw, &options)?;

    assert_eq!(spectrum.x.unit, Unit::Ppm);
    assert_eq!(spectrum.y.unit, Unit::Ppm);
    // 13C axis spans the experiment's ~170 ppm sweep (from y_sweep).
    let y_span = (spectrum.y.values[spectrum.y.values.len() - 1] - spectrum.y.values[0]).abs();
    assert!(y_span > 120.0, "indirect 13C span {y_span} ppm too small");

    let (proton_ppm, _carbon_ppm) = argmax(&spectrum);
    assert!(
        (0.3..=2.8).contains(&proton_ppm),
        "strongest cross-peak 1H should be aliphatic, got {proton_ppm} ppm"
    );
    Ok(())
}

#[test]
fn rutin_hsqc_resolves_the_rhamnose_methyl_cross_peak() -> rspin::core::Result<()> {
    // High-resolution (256 t1) HSQC from the jeol-data-test submodule.
    let fixture = repo_root().join(
        "external-testdata/cheminfo/jeol-data-test/data/Rutin_3080ug200uL_DMSOd6_HSQC_400MHz_Jeol.jdf",
    );
    if !fixture.exists() {
        // Submodule not initialized; skip cleanly.
        return Ok(());
    }

    let raw = read_jeol_jdf_2d_hypercomplex_file(&fixture)?;
    let options = HyperComplex2DOptions::default().with_indirect_zero_fill(512);
    let spectrum = process_hypercomplex_planes_magnitude(&raw, &options)?;

    // The strongest cross-peak is rutin's rhamnose methyl: 1H ~0.9 / 13C ~18.
    let (proton_ppm, carbon_ppm) = argmax(&spectrum);
    assert!(
        (0.6..=1.3).contains(&proton_ppm),
        "rhamnose methyl 1H should be ~0.9 ppm, got {proton_ppm}"
    );
    assert!(
        (13.0..=23.0).contains(&carbon_ppm),
        "rhamnose methyl 13C should be ~18 ppm, got {carbon_ppm}"
    );
    Ok(())
}
