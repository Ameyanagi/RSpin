// Synthetic fixtures use small integer indices as floats; precision loss is
// irrelevant here.
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_lossless,
    clippy::manual_is_multiple_of
)]

use std::f64::consts::PI;

use rspin_core::{Axis, Metadata, QuadMode, RSpinError, Spectrum2D, Unit};

use super::*;

/// Builds a synthetic raw `ser`-style spectrum. Each acquired row is a complex
/// direct-dimension tone at bin `kx`, scaled by a per-row complex modulation.
/// After the direct FFT every row becomes a delta at `kx` carrying its
/// modulation, which is exactly what the quadrature assembly consumes.
fn synthetic_raw(
    width: usize,
    rows: usize,
    kx: usize,
    quad_mode: QuadMode,
    modulation: impl Fn(usize) -> (f64, f64),
) -> anyhow::Result<Spectrum2D> {
    let mut z = Vec::with_capacity(width * rows);
    let mut imaginary = Vec::with_capacity(width * rows);
    for row in 0..rows {
        let (mod_re, mod_im) = modulation(row);
        for t in 0..width {
            let angle = 2.0 * PI * (kx as f64) * (t as f64) / (width as f64);
            let (sin_t, cos_t) = angle.sin_cos();
            // (cos + i sin) * (mod_re + i mod_im)
            z.push(cos_t * mod_re - sin_t * mod_im);
            imaginary.push(sin_t * mod_re + cos_t * mod_im);
        }
    }
    let x = Axis::linear("t2", Unit::Seconds, 0.0, 1.0, width)?;
    let y = Axis::linear("t1", Unit::Seconds, 0.0, 1.0, rows)?;
    let metadata = Metadata::default()
        .with_quad_mode(quad_mode)
        .with_frequency_mhz(100.0)
        .with_indirect_frequency_mhz(25.0);
    Ok(Spectrum2D::new_complex(x, y, z, Some(imaginary), metadata)?)
}

/// Index and value of the largest real-channel magnitude.
fn argmax(spectrum: &Spectrum2D) -> (usize, usize, f64) {
    let (width, height) = spectrum.shape();
    let mut best = (0usize, 0usize, f64::NEG_INFINITY);
    for y in 0..height {
        for x in 0..width {
            let value = spectrum.value_at(x, y).unwrap_or(0.0);
            if value > best.2 {
                best = (x, y, value);
            }
        }
    }
    best
}

/// Counts indirect-profile peaks (local maxima above half the global maximum)
/// in the column at direct bin `x`.
fn count_indirect_peaks(spectrum: &Spectrum2D, x: usize) -> usize {
    let (_, height) = spectrum.shape();
    let column: Vec<f64> = (0..height)
        .map(|y| spectrum.value_at(x, y).unwrap_or(0.0))
        .collect();
    let max = column.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if max <= 0.0 {
        return 0;
    }
    let threshold = 0.5 * max;
    let mut count = 0;
    for (index, value) in column.iter().enumerate() {
        if *value < threshold {
            continue;
        }
        let left = index
            .checked_sub(1)
            .map_or(f64::NEG_INFINITY, |i| column[i]);
        let right = column.get(index + 1).copied().unwrap_or(f64::NEG_INFINITY);
        if *value >= left && *value >= right {
            count += 1;
        }
    }
    count
}

#[test]
fn states_assembly_places_single_quadrature_resolved_peak() -> anyhow::Result<()> {
    let width = 8;
    let points = 16;
    let kx = 2;
    let ky = 3;
    // Row 2m = cosine-modulated, row 2m+1 = sine-modulated.
    let raw = synthetic_raw(width, 2 * points, kx, QuadMode::States, |row| {
        let m = row / 2;
        let theta = 2.0 * PI * (ky as f64) * (m as f64) / (points as f64);
        if row % 2 == 0 {
            (theta.cos(), 0.0)
        } else {
            (theta.sin(), 0.0)
        }
    })?;

    let spectrum = process_hypercomplex_2d(&raw, &HyperComplex2DOptions::default())?;
    assert_eq!(spectrum.shape(), (width, points));

    let (peak_x, _, peak_value) = argmax(&spectrum);
    assert!(
        peak_value > 0.0,
        "States peak should be positive: {peak_value}"
    );
    // Quadrature detection resolves the sign: exactly one indirect peak, no
    // mirror image at the conjugate frequency.
    assert_eq!(
        count_indirect_peaks(&spectrum, peak_x),
        1,
        "States indirect dimension should show a single quadrature-resolved peak"
    );
    Ok(())
}

#[test]
fn quadrature_failure_mode_shows_mirror_image() -> anyhow::Result<()> {
    // The same cosine modulation acquired single-channel (QF) cannot resolve
    // the sign, so the indirect FFT shows BOTH the peak and its mirror image.
    let width = 8;
    let points = 16;
    let kx = 2;
    let ky = 3;
    let raw = synthetic_raw(width, points, kx, QuadMode::Qf, |row| {
        let theta = 2.0 * PI * (ky as f64) * (row as f64) / (points as f64);
        (theta.cos(), 0.0)
    })?;

    let spectrum = process_hypercomplex_2d(&raw, &HyperComplex2DOptions::default())?;
    let (peak_x, _, _) = argmax(&spectrum);
    assert_eq!(
        count_indirect_peaks(&spectrum, peak_x),
        2,
        "single-channel QF should show a mirror image (two peaks)"
    );
    Ok(())
}

#[test]
fn states_tppi_moves_axial_peak_to_the_edge() -> anyhow::Result<()> {
    let width = 8;
    let points = 16;
    let kx = 2;
    // Axial component: zero indirect frequency.
    let modulation = |row: usize, sign_flip: bool| -> (f64, f64) {
        let m = row / 2;
        let base = if row % 2 == 0 { 1.0 } else { 0.0 };
        if sign_flip && m % 2 == 1 {
            (-base, 0.0)
        } else {
            (base, 0.0)
        }
    };

    let states = synthetic_raw(width, 2 * points, kx, QuadMode::States, |row| {
        modulation(row, false)
    })?;
    let states_tppi = synthetic_raw(width, 2 * points, kx, QuadMode::StatesTppi, |row| {
        // The same acquired data; the StatesTppi (-1)^m flip is applied during
        // assembly, so feed the unflipped rows here.
        modulation(row, false)
    })?;

    let states_spectrum = process_hypercomplex_2d(&states, &HyperComplex2DOptions::default())?;
    let tppi_spectrum = process_hypercomplex_2d(&states_tppi, &HyperComplex2DOptions::default())?;

    let (_, states_y, _) = argmax(&states_spectrum);
    let (_, tppi_y, _) = argmax(&tppi_spectrum);

    let center = points / 2;
    assert_eq!(
        states_y, center,
        "States axial peak should sit at the center"
    );
    assert!(
        tppi_y == 0 || tppi_y == points - 1,
        "States-TPPI axial peak should sit at the spectrum edge, got {tppi_y}"
    );
    Ok(())
}

#[test]
fn echo_antiecho_resolves_a_single_peak() -> anyhow::Result<()> {
    let width = 8;
    let points = 16;
    let kx = 2;
    let ky = 3;
    // P (echo) and N (anti-echo) modulations chosen so the documented
    // echo/anti-echo recombination yields a clean single indirect frequency.
    let raw = synthetic_raw(width, 2 * points, kx, QuadMode::EchoAntiecho, |row| {
        let m = row / 2;
        let theta = 2.0 * PI * (ky as f64) * (m as f64) / (points as f64);
        if row % 2 == 0 {
            // P_m = 0.5 * exp(-i theta)
            (0.5 * theta.cos(), -0.5 * theta.sin())
        } else {
            // N_m = -0.5 cos(theta) - i 0.5 sin(theta)
            (-0.5 * theta.cos(), -0.5 * theta.sin())
        }
    })?;

    let spectrum = process_hypercomplex_2d(&raw, &HyperComplex2DOptions::default())?;
    let (peak_x, _, peak_value) = argmax(&spectrum);
    assert!(peak_value > 0.0, "echo/anti-echo peak should be positive");
    assert_eq!(
        count_indirect_peaks(&spectrum, peak_x),
        1,
        "echo/anti-echo should resolve a single peak"
    );
    Ok(())
}

#[test]
fn rejects_odd_row_count_for_paired_modes() -> anyhow::Result<()> {
    let raw = synthetic_raw(4, 5, 1, QuadMode::States, |_| (1.0, 0.0))?;
    assert!(matches!(
        assemble_hypercomplex_2d(&raw),
        Err(RSpinError::InvalidSpectrum { .. })
    ));
    Ok(())
}

#[test]
fn new_rejects_mismatched_and_non_finite_planes() -> anyhow::Result<()> {
    let x = Axis::linear("x", Unit::Hertz, 0.0, 1.0, 2)?;
    let y = Axis::linear("y", Unit::Hertz, 0.0, 1.0, 2)?;
    // Mismatched plane length.
    assert!(matches!(
        HyperComplex2D::new(
            x.clone(),
            y.clone(),
            vec![0.0, 0.0, 0.0],
            vec![0.0; 4],
            vec![0.0; 4],
            vec![0.0; 4],
            Metadata::default(),
        ),
        Err(RSpinError::InvalidSpectrum { .. })
    ));
    // Non-finite value.
    assert!(matches!(
        HyperComplex2D::new(
            x,
            y,
            vec![0.0, f64::NAN, 0.0, 0.0],
            vec![0.0; 4],
            vec![0.0; 4],
            vec![0.0; 4],
            Metadata::default(),
        ),
        Err(RSpinError::NonFinite { .. })
    ));
    Ok(())
}

#[test]
fn qf_path_has_no_indirect_imaginary_channel() -> anyhow::Result<()> {
    let raw = synthetic_raw(4, 8, 1, QuadMode::Qf, |row| ((row as f64 * 0.3).cos(), 0.0))?;
    let hc = assemble_hypercomplex_2d(&raw)?;
    assert!(
        hc.ir.iter().all(|v| v.abs() < 1.0e-12) && hc.ii.iter().all(|v| v.abs() < 1.0e-12),
        "QF assembly should leave the indirect imaginary quadrants empty"
    );
    // The downgrade and full pipeline still succeed.
    let _ = process_hypercomplex_2d(&raw, &HyperComplex2DOptions::default())?;
    Ok(())
}

#[test]
fn indirect_zero_fill_extends_the_indirect_dimension() -> anyhow::Result<()> {
    let raw = synthetic_raw(4, 16, 1, QuadMode::States, |row| {
        let m = row / 2;
        let theta = 2.0 * PI * 2.0 * (m as f64) / 8.0;
        if row % 2 == 0 {
            (theta.cos(), 0.0)
        } else {
            (theta.sin(), 0.0)
        }
    })?;
    let options = HyperComplex2DOptions::default().with_indirect_zero_fill(32);
    let spectrum = process_hypercomplex_2d(&raw, &options)?;
    assert_eq!(spectrum.shape(), (4, 32));
    Ok(())
}

fn states_fixture(width: usize, points: usize, kx: usize, ky: usize) -> anyhow::Result<Spectrum2D> {
    synthetic_raw(width, 2 * points, kx, QuadMode::States, move |row| {
        let m = row / 2;
        let theta = 2.0 * PI * (ky as f64) * (m as f64) / (points as f64);
        if row % 2 == 0 {
            (theta.cos(), 0.0)
        } else {
            (theta.sin(), 0.0)
        }
    })
}

#[test]
fn recipe_round_trips_and_applies() -> anyhow::Result<()> {
    let raw = states_fixture(8, 16, 2, 3)?;
    let recipe = crate::ProcessingRecipe2D::new()
        .hypercomplex_process(HyperComplex2DOptions::default().with_indirect_zero_fill(32));
    let json = serde_json::to_string(&recipe)?;
    assert!(json.contains("hyper_complex_process"));
    let restored: crate::ProcessingRecipe2D = serde_json::from_str(&json)?;
    assert_eq!(restored, recipe);

    let processed = crate::apply_processing_recipe_2d(&raw, &recipe)?;
    assert_eq!(processed.shape(), (8, 32));
    Ok(())
}

#[test]
fn pipeline_from_raw_hypercomplex_then_continues() -> anyhow::Result<()> {
    let raw = states_fixture(8, 16, 2, 3)?;
    let processed =
        crate::Spectrum2DPipeline::from_raw_hypercomplex(&raw, &HyperComplex2DOptions::default())
            .scale(2.0)
            .finish()?;
    assert_eq!(processed.shape(), (8, 16));
    Ok(())
}

#[test]
fn options_round_trip_through_json() -> anyhow::Result<()> {
    let options = HyperComplex2DOptions::default()
        .with_indirect_zero_fill(256)
        .with_indirect_line_broadening_hz(5.0)
        .with_phase(PhaseCorrection2D::new().x_phase(10.0, 0.0, 0.5));
    let json = serde_json::to_string(&options)?;
    let restored: HyperComplex2DOptions = serde_json::from_str(&json)?;
    assert_eq!(restored, options);
    Ok(())
}
