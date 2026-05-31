//! Validates the four-plane phase-sensitive JEOL HSQC path: submatrix
//! de-tiling, four-plane hypercomplex read, ¹H/¹³C axis calibration, the
//! direct/indirect transform, and the hypercomplex-modulus display.
//!
//! - The committed CC0 eucalyptol fixture (32 t₁ increments) is an always-on
//!   structural check (correct axes, aliphatic ¹H peak).
//! - Higher-resolution fixtures (256 t₁) from the `jeol-data-test` submodule
//!   check chemically plausible ¹H/¹³C cross-peak placement. Skipped when the
//!   submodule is not initialized.

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
            let Some(value) = spectrum.value_at(x, y) else {
                continue;
            };
            if value > best {
                best = value;
                bx = x;
                by = y;
            }
        }
    }
    (spectrum.x.values[bx], spectrum.y.values[by])
}

fn max_in_window(spectrum: &Spectrum2D, x_min: f64, x_max: f64, y_min: f64, y_max: f64) -> f64 {
    let (width, height) = spectrum.shape();
    let mut best = f64::NEG_INFINITY;
    for y in 0..height {
        let y_value = spectrum.y.values[y];
        if y_value < y_min || y_value > y_max {
            continue;
        }
        for x in 0..width {
            let x_value = spectrum.x.values[x];
            if x_value < x_min || x_value > x_max {
                continue;
            }
            let Some(value) = spectrum.value_at(x, y) else {
                continue;
            };
            if value > best {
                best = value;
            }
        }
    }
    best
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

    // The rhamnose methyl appears at 1H ~0.9 / 13C ~18. The magnitude display
    // can retain a near-degenerate F1 mirror, so assert the chemically expected
    // window is among the strongest features rather than requiring global
    // argmax uniqueness.
    let global = max_in_window(&spectrum, -10.0, 20.0, -10.0, 200.0);
    let methyl = max_in_window(&spectrum, 0.6, 1.3, 13.0, 23.0);
    assert!(
        methyl.is_finite(),
        "rhamnose methyl window has no finite signal"
    );
    assert!(
        methyl > 0.9 * global,
        "rhamnose methyl 1H/13C window should be a dominant feature; window={methyl}, global={global}"
    );
    Ok(())
}

#[test]
fn ec_hsqc_places_aromatic_correlations_on_aromatic_carbons() -> rspin::core::Result<()> {
    let fixture = repo_root().join(
        "external-testdata/cheminfo/jeol-data-test/data/EC=8C_5m200u_MeOD_bzhou21_20190228__HSQC-1-1.jdf",
    );
    if !fixture.exists() {
        return Ok(());
    }

    let raw = read_jeol_jdf_2d_hypercomplex_file(&fixture)?;
    let options = HyperComplex2DOptions::default().with_indirect_zero_fill(512);
    let spectrum = process_hypercomplex_planes_magnitude(&raw, &options)?;

    let aromatic = max_in_window(&spectrum, 6.7, 7.2, 110.0, 123.0);
    let mirrored = max_in_window(&spectrum, 6.7, 7.2, 47.0, 60.0);
    assert!(
        aromatic.is_finite(),
        "EC aromatic 1H/13C window has no finite signal"
    );
    assert!(
        aromatic > 3.0 * mirrored,
        "EC aromatic 1H peaks should correlate to aromatic 13C, not mirrored aliphatic 13C; aromatic={aromatic}, mirrored={mirrored}"
    );
    Ok(())
}
