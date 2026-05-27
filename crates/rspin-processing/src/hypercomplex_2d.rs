//! In-flight hypercomplex 2D processing for phase-sensitive experiments.
//!
//! Phase-sensitive 2D NMR (for example HSQC) acquires the indirect dimension
//! with a quadrature scheme (States, States-TPPI, echo/anti-echo) that encodes
//! the indirect-dimension sign in *pairs* of acquired rows. To phase such a
//! spectrum the four hypercomplex quadrants (RR/RI/IR/II) must be carried
//! together while the indirect dimension is still time-domain — which the
//! single-companion [`Spectrum2D`] cannot represent.
//!
//! [`HyperComplex2D`] is a transient container for that processing; the
//! pipeline assembles it from a raw `ser`-style [`Spectrum2D`], transforms and
//! phases it, then downgrades back to a displayable [`Spectrum2D`].
//!
//! The quadrature assembly is a clean-room implementation of the documented
//! Bruker / nmrglue behavior (behavior only; no code was copied).

// The four hypercomplex quadrants (rr/ri/ir/ii) and their per-step rotations
// (rr_x/ri_x/...) are intrinsically similar names; renaming them hurts clarity.
#![allow(clippy::similar_names)]

use rspin_core::{
    Axis, HyperComplex2D, ProcessingRecord, QuadMode, RSpinError, Result, Spectrum2D,
};
use rustfft::{FftPlanner, num_complex::Complex};
use serde::{Deserialize, Serialize};

use crate::PhaseCorrection2D;
use crate::transform::{fftshift_in_place, frequency_axis_from_time};

/// Options for the one-shot raw → phasable hypercomplex pipeline.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct HyperComplex2DOptions {
    /// Optional indirect-dimension zero-fill target (number of points). Only
    /// applied when larger than the assembled indirect point count.
    pub indirect_zero_fill: Option<usize>,
    /// Optional indirect-dimension exponential line broadening, in Hz, applied
    /// in the indirect time domain before the indirect transform.
    pub indirect_line_broadening_hz: Option<f64>,
    /// Phase correction applied after the indirect transform.
    pub phase: PhaseCorrection2D,
}

impl Default for HyperComplex2DOptions {
    fn default() -> Self {
        Self {
            indirect_zero_fill: None,
            indirect_line_broadening_hz: None,
            phase: PhaseCorrection2D::new(),
        }
    }
}

impl HyperComplex2DOptions {
    /// Returns options with an indirect zero-fill target.
    #[must_use]
    pub fn with_indirect_zero_fill(mut self, target: usize) -> Self {
        self.indirect_zero_fill = Some(target);
        self
    }

    /// Returns options with an indirect exponential line broadening (Hz).
    #[must_use]
    pub fn with_indirect_line_broadening_hz(mut self, line_broadening_hz: f64) -> Self {
        self.indirect_line_broadening_hz = Some(line_broadening_hz);
        self
    }

    /// Returns options with a phase correction.
    #[must_use]
    pub fn with_phase(mut self, phase: PhaseCorrection2D) -> Self {
        self.phase = phase;
        self
    }
}

/// Whether a quadrature mode pairs consecutive acquired rows into one indirect
/// complex point.
fn mode_pairs_rows(mode: QuadMode) -> bool {
    matches!(
        mode,
        QuadMode::States | QuadMode::StatesTppi | QuadMode::EchoAntiecho
    )
}

/// Forward-FFTs every acquired row of a raw `ser`-style spectrum along the
/// direct dimension, returning the row-major complex spectra.
fn direct_ft_rows(raw: &Spectrum2D, width: usize, rows: usize) -> Vec<Complex<f64>> {
    let mut buffer: Vec<Complex<f64>> = match &raw.imaginary {
        Some(imaginary) => raw
            .z
            .iter()
            .zip(imaginary)
            .map(|(re, im)| Complex::new(*re, *im))
            .collect(),
        None => raw.z.iter().map(|re| Complex::new(*re, 0.0)).collect(),
    };
    let mut planner = FftPlanner::<f64>::new();
    let fft = planner.plan_fft_forward(width);
    for row in buffer.chunks_exact_mut(width) {
        fft.process(row);
        fftshift_in_place(row);
    }
    debug_assert_eq!(buffer.len(), width * rows);
    buffer
}

/// Assembles a raw `ser`-style [`Spectrum2D`] into a [`HyperComplex2D`] in the
/// direct-frequency / indirect-time domain, using the quadrature mode recorded
/// in `raw.metadata.quad_mode` (defaulting to [`QuadMode::Qf`] when absent).
///
/// # Errors
///
/// Returns an error when the spectrum is empty, a paired quadrature mode has an
/// odd acquired-row count, or the result fails validation.
pub fn assemble_hypercomplex_2d(raw: &Spectrum2D) -> Result<HyperComplex2D> {
    let (width, rows) = raw.shape();
    if width == 0 || rows == 0 {
        return Err(RSpinError::InvalidSpectrum {
            message: "hypercomplex assembly requires a non-empty spectrum".to_owned(),
        });
    }
    let mode = raw.metadata.quad_mode.unwrap_or(QuadMode::Qf);
    let direct = direct_ft_rows(raw, width, rows);

    let paired = mode_pairs_rows(mode);
    let points = if paired {
        if rows % 2 != 0 {
            return Err(RSpinError::InvalidSpectrum {
                message: format!(
                    "{mode:?} quadrature requires an even acquired-row count, found {rows}"
                ),
            });
        }
        rows / 2
    } else {
        rows
    };

    let plane_len = width
        .checked_mul(points)
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "hypercomplex assembly size overflow".to_owned(),
        })?;
    let mut rr = vec![0.0_f64; plane_len];
    let mut ri = vec![0.0_f64; plane_len];
    let mut ir = vec![0.0_f64; plane_len];
    let mut ii = vec![0.0_f64; plane_len];

    for point in 0..points {
        for k in 0..width {
            let (a, b) = if paired {
                (
                    direct[(2 * point) * width + k],
                    direct[(2 * point + 1) * width + k],
                )
            } else {
                (direct[point * width + k], Complex::new(0.0, 0.0))
            };
            let (rr_v, ri_v, ir_v, ii_v) = match mode {
                QuadMode::States => (a.re, a.im, b.re, b.im),
                QuadMode::StatesTppi => {
                    let sign = if point % 2 == 0 { 1.0 } else { -1.0 };
                    (sign * a.re, sign * a.im, sign * b.re, sign * b.im)
                }
                QuadMode::EchoAntiecho => {
                    // Recover absorptive cosine/sine pair from the P/N
                    // (echo/anti-echo) acquisitions before forming the
                    // indirect-complex point. The 90 degree indirect phase
                    // implied by the `i` factor is absorbed by y-phasing.
                    let cosine = a - b;
                    let sine = Complex::new(0.0, 1.0) * (a + b);
                    (cosine.re, cosine.im, sine.re, sine.im)
                }
                // Single-channel / degenerate quadrature: no indirect
                // imaginary channel. Tppi and Qseq fall back to this path.
                QuadMode::Qf | QuadMode::Qseq | QuadMode::Tppi | QuadMode::None => {
                    (a.re, a.im, 0.0, 0.0)
                }
            };
            let idx = point * width + k;
            rr[idx] = rr_v;
            ri[idx] = ri_v;
            ir[idx] = ir_v;
            ii[idx] = ii_v;
        }
    }

    let new_x = frequency_axis_from_time(&raw.x, &raw.metadata, width, 0)?;
    // The assembled indirect time grid keeps one sample per indirect point.
    let y_values = raw
        .y
        .values
        .get(0..points)
        .map_or_else(|| raw.y.values.clone(), <[f64]>::to_vec);
    let new_y = Axis::new(raw.y.label.clone(), raw.y.unit, y_values)?;

    let hc = HyperComplex2D::new(new_x, new_y, rr, ri, ir, ii, raw.metadata.clone())?;
    Ok(hc.with_processing_record(
        ProcessingRecord::new("assemble_hypercomplex_2d")
            .with_details(format!("quad_mode={mode:?},indirect_points={points}")),
    ))
}

/// Applies the indirect-dimension complex FFT, transforming a direct-frequency
/// / indirect-time [`HyperComplex2D`] into the fully frequency-domain RR/RI/IR/II
/// quadrants.
///
/// # Errors
///
/// Returns an error when the result fails validation.
pub fn indirect_ft_hypercomplex_2d(hc: &HyperComplex2D) -> Result<HyperComplex2D> {
    let (width, points) = hc.shape();
    let mut rr = hc.rr.clone();
    let mut ri = hc.ri.clone();
    let mut ir = hc.ir.clone();
    let mut ii = hc.ii.clone();

    let mut planner = FftPlanner::<f64>::new();
    let fft = planner.plan_fft_forward(points);
    for k in 0..width {
        let mut cosine = Vec::with_capacity(points);
        let mut sine = Vec::with_capacity(points);
        for point in 0..points {
            let idx = point * width + k;
            cosine.push(Complex::new(hc.rr[idx], hc.ir[idx]));
            sine.push(Complex::new(hc.ri[idx], hc.ii[idx]));
        }
        fft.process(&mut cosine);
        fftshift_in_place(&mut cosine);
        fft.process(&mut sine);
        fftshift_in_place(&mut sine);
        for point in 0..points {
            let idx = point * width + k;
            rr[idx] = cosine[point].re;
            ir[idx] = cosine[point].im;
            ri[idx] = sine[point].re;
            ii[idx] = sine[point].im;
        }
    }

    // The indirect axis carries its own carrier; relabel using
    // `indirect_frequency_mhz` when available, mirroring `fft_2d`.
    let indirect_metadata = match hc.metadata.indirect_frequency_mhz {
        Some(freq) => {
            let mut shim = hc.metadata.clone();
            shim.frequency_mhz = Some(freq);
            shim
        }
        None => hc.metadata.clone(),
    };
    let new_y = frequency_axis_from_time(&hc.y, &indirect_metadata, points, 1)?;

    let mut result = HyperComplex2D::new(hc.x.clone(), new_y, rr, ri, ir, ii, hc.metadata.clone())?;
    result.processing.clone_from(&hc.processing);
    Ok(result.with_processing_record(ProcessingRecord::new("indirect_ft_hypercomplex_2d")))
}

/// Applies separable direct/indirect phase correction to all four quadrants.
///
/// The direct phase rotates each direct-complex pair `(rr, ri)` and `(ir, ii)`;
/// the indirect phase then rotates each indirect-complex pair `(rr, ir)` and
/// `(ri, ii)`. Phase terms reuse the [`PhaseCorrection2D`] convention.
///
/// # Errors
///
/// Returns an error when phase parameters are invalid or the result fails
/// validation.
pub fn phase_hypercomplex_2d(
    hc: &HyperComplex2D,
    correction: PhaseCorrection2D,
) -> Result<HyperComplex2D> {
    correction.validate()?;
    let (width, points) = hc.shape();
    let x_denominator = denominator(width);
    let y_denominator = denominator(points);

    let mut rr = hc.rr.clone();
    let mut ri = hc.ri.clone();
    let mut ir = hc.ir.clone();
    let mut ii = hc.ii.clone();

    for point in 0..points {
        let y_fraction = fraction(point, y_denominator);
        let phase_y_rad = (correction.y_zero_order_deg
            + correction.y_first_order_deg * (y_fraction - correction.y_pivot_fraction))
            .to_radians();
        let (sin_y, cos_y) = phase_y_rad.sin_cos();
        for k in 0..width {
            let x_fraction = fraction(k, x_denominator);
            let phase_x_rad = (correction.x_zero_order_deg
                + correction.x_first_order_deg * (x_fraction - correction.x_pivot_fraction))
                .to_radians();
            let (sin_x, cos_x) = phase_x_rad.sin_cos();
            let idx = point * width + k;

            // Direct-dimension rotation within each indirect channel.
            let rr_x = rr[idx] * cos_x - ri[idx] * sin_x;
            let ri_x = rr[idx] * sin_x + ri[idx] * cos_x;
            let ir_x = ir[idx] * cos_x - ii[idx] * sin_x;
            let ii_x = ir[idx] * sin_x + ii[idx] * cos_x;

            // Indirect-dimension rotation within each direct channel.
            rr[idx] = rr_x * cos_y - ir_x * sin_y;
            ir[idx] = rr_x * sin_y + ir_x * cos_y;
            ri[idx] = ri_x * cos_y - ii_x * sin_y;
            ii[idx] = ri_x * sin_y + ii_x * cos_y;
        }
    }

    let mut result = HyperComplex2D::new(
        hc.x.clone(),
        hc.y.clone(),
        rr,
        ri,
        ir,
        ii,
        hc.metadata.clone(),
    )?;
    result.processing.clone_from(&hc.processing);
    Ok(
        result.with_processing_record(ProcessingRecord::new("phase_hypercomplex_2d").with_details(
            format!(
                "x_zero_order_deg={},x_first_order_deg={},y_zero_order_deg={},y_first_order_deg={}",
                correction.x_zero_order_deg,
                correction.x_first_order_deg,
                correction.y_zero_order_deg,
                correction.y_first_order_deg,
            ),
        )),
    )
}

/// Processes a raw `ser`-style [`Spectrum2D`] into a phasable [`Spectrum2D`]:
/// direct FT, quadrature assembly, optional indirect apodization/zero-fill,
/// indirect FT, phase correction, then downgrade.
///
/// # Errors
///
/// Returns an error when any stage fails (see the individual functions).
pub fn process_hypercomplex_2d(
    raw: &Spectrum2D,
    options: &HyperComplex2DOptions,
) -> Result<Spectrum2D> {
    let mut hc = assemble_hypercomplex_2d(raw)?;
    if let Some(line_broadening_hz) = options.indirect_line_broadening_hz {
        hc = indirect_exponential_apodization(&hc, line_broadening_hz)?;
    }
    if let Some(target) = options.indirect_zero_fill {
        hc = indirect_zero_fill(&hc, target)?;
    }
    hc = indirect_ft_hypercomplex_2d(&hc)?;
    hc = phase_hypercomplex_2d(&hc, options.phase)?;
    hc.into_spectrum_2d()
}

/// Applies the direct-dimension complex FFT to a time-domain
/// [`HyperComplex2D`], transforming each row's direct-complex pairs
/// `(rr, ri)` and `(ir, ii)` and producing a direct-frequency /
/// indirect-time hypercomplex spectrum.
///
/// # Errors
///
/// Returns an error when the result fails validation.
pub fn direct_ft_hypercomplex_2d(hc: &HyperComplex2D) -> Result<HyperComplex2D> {
    let (width, points) = hc.shape();
    let mut rr = hc.rr.clone();
    let mut ri = hc.ri.clone();
    let mut ir = hc.ir.clone();
    let mut ii = hc.ii.clone();

    let mut planner = FftPlanner::<f64>::new();
    let fft = planner.plan_fft_forward(width);
    for point in 0..points {
        let base = point * width;
        let mut cosine: Vec<Complex<f64>> = (0..width)
            .map(|k| Complex::new(hc.rr[base + k], hc.ri[base + k]))
            .collect();
        let mut sine: Vec<Complex<f64>> = (0..width)
            .map(|k| Complex::new(hc.ir[base + k], hc.ii[base + k]))
            .collect();
        fft.process(&mut cosine);
        fftshift_in_place(&mut cosine);
        fft.process(&mut sine);
        fftshift_in_place(&mut sine);
        for k in 0..width {
            rr[base + k] = cosine[k].re;
            ri[base + k] = cosine[k].im;
            ir[base + k] = sine[k].re;
            ii[base + k] = sine[k].im;
        }
    }

    let new_x = frequency_axis_from_time(&hc.x, &hc.metadata, width, 0)?;
    let mut result = HyperComplex2D::new(new_x, hc.y.clone(), rr, ri, ir, ii, hc.metadata.clone())?;
    result.processing.clone_from(&hc.processing);
    Ok(result.with_processing_record(ProcessingRecord::new("direct_ft_hypercomplex_2d")))
}

/// Processes a time-domain four-plane [`HyperComplex2D`] (as produced by a
/// reader that already separates the RR/RI/IR/II quadrants, e.g. JEOL) into a
/// phasable [`Spectrum2D`]: direct FT, optional indirect apodization/zero-fill,
/// indirect complex FT, phase correction, then downgrade.
///
/// Unlike [`process_hypercomplex_2d`], this does not assemble quadrants from
/// interleaved acquisition rows — the planes are already separated.
///
/// # Errors
///
/// Returns an error when any stage fails (see the individual functions).
pub fn process_hypercomplex_planes(
    hc: &HyperComplex2D,
    options: &HyperComplex2DOptions,
) -> Result<Spectrum2D> {
    // Window both dimensions with a 90-degree-shifted squared sine-bell before
    // each transform. This tapers the (short, truncated) interferograms to zero
    // and suppresses the F1/F2 truncation ridges, following the standard
    // NMRPipe/nmrglue 2D recipe (`sp` window, off~0.5, end~0.98, pow=2).
    const SINE_BELL_OFF: f64 = 0.5;
    const SINE_BELL_END: f64 = 0.98;
    const SINE_BELL_POW: f64 = 2.0;

    let mut hc = direct_sine_bell(hc, SINE_BELL_OFF, SINE_BELL_END, SINE_BELL_POW);
    hc = direct_ft_hypercomplex_2d(&hc)?;
    if let Some(line_broadening_hz) = options.indirect_line_broadening_hz {
        hc = indirect_exponential_apodization(&hc, line_broadening_hz)?;
    }
    hc = indirect_sine_bell(&hc, SINE_BELL_OFF, SINE_BELL_END, SINE_BELL_POW);
    // Halve the first t1 increment to suppress the F1 baseline offset (the
    // standard first-point correction, applied in the indirect dimension).
    hc = indirect_first_point_scale(&hc, 0.5);
    if let Some(target) = options.indirect_zero_fill {
        hc = indirect_zero_fill(&hc, target)?;
    }
    hc = indirect_ft_hypercomplex_2d(&hc)?;
    hc = phase_hypercomplex_2d(&hc, options.phase)?;
    hc.into_spectrum_2d()
}

/// Processes a time-domain four-plane [`HyperComplex2D`] into a phase-insensitive
/// magnitude [`Spectrum2D`]: identical to [`process_hypercomplex_planes`] up to
/// the indirect FT, then takes the hypercomplex modulus
/// `sqrt(rr^2 + ri^2 + ir^2 + ii^2)` over all four quadrants instead of phasing.
///
/// This is the robust display path when a reliable direct/indirect phase is not
/// available: the four-quadrant modulus is phase-insensitive, so dispersive
/// content does not leak into the contour as t1/t2 ridges.
///
/// # Errors
///
/// Returns an error when any stage fails (see the individual functions).
pub fn process_hypercomplex_planes_magnitude(
    hc: &HyperComplex2D,
    options: &HyperComplex2DOptions,
) -> Result<Spectrum2D> {
    const SINE_BELL_OFF: f64 = 0.5;
    const SINE_BELL_END: f64 = 0.98;
    const SINE_BELL_POW: f64 = 2.0;

    let mut hc = direct_sine_bell(hc, SINE_BELL_OFF, SINE_BELL_END, SINE_BELL_POW);
    hc = direct_ft_hypercomplex_2d(&hc)?;
    if let Some(line_broadening_hz) = options.indirect_line_broadening_hz {
        hc = indirect_exponential_apodization(&hc, line_broadening_hz)?;
    }
    hc = indirect_sine_bell(&hc, SINE_BELL_OFF, SINE_BELL_END, SINE_BELL_POW);
    hc = indirect_first_point_scale(&hc, 0.5);
    if let Some(target) = options.indirect_zero_fill {
        hc = indirect_zero_fill(&hc, target)?;
    }
    hc = indirect_ft_hypercomplex_2d(&hc)?;
    hc.into_magnitude_spectrum_2d()
}

/// Shifted sine-bell apodization factor for index `i` of `n` points:
/// `sin(pi*off + pi*(end-off)*i/(n-1))^pow` (`NMRPipe` `sp` window).
fn sine_bell_factor(i: usize, n: usize, off: f64, end: f64, pow: f64) -> f64 {
    if n <= 1 {
        return 1.0;
    }
    let denom = u32::try_from(n - 1).map_or(1.0, f64::from);
    let index = u32::try_from(i).map_or(0.0, f64::from);
    let angle = std::f64::consts::PI * off + std::f64::consts::PI * (end - off) * (index / denom);
    angle.sin().powf(pow)
}

/// Applies a shifted sine-bell window along the direct (x) dimension.
fn direct_sine_bell(hc: &HyperComplex2D, off: f64, end: f64, pow: f64) -> HyperComplex2D {
    let (width, points) = hc.shape();
    let window: Vec<f64> = (0..width)
        .map(|k| sine_bell_factor(k, width, off, end, pow))
        .collect();
    let mut result = hc.clone();
    for plane in [
        &mut result.rr,
        &mut result.ri,
        &mut result.ir,
        &mut result.ii,
    ] {
        for point in 0..points {
            let base = point * width;
            for (k, factor) in window.iter().enumerate() {
                plane[base + k] *= *factor;
            }
        }
    }
    result
}

/// Applies a shifted sine-bell window along the indirect (y) dimension.
fn indirect_sine_bell(hc: &HyperComplex2D, off: f64, end: f64, pow: f64) -> HyperComplex2D {
    let (width, points) = hc.shape();
    let mut result = hc.clone();
    for point in 0..points {
        let factor = sine_bell_factor(point, points, off, end, pow);
        let base = point * width;
        for plane in [
            &mut result.rr,
            &mut result.ri,
            &mut result.ir,
            &mut result.ii,
        ] {
            for value in &mut plane[base..base + width] {
                *value *= factor;
            }
        }
    }
    result
}

/// Scales the first indirect (t1) increment of all four quadrants, suppressing
/// the F1 baseline ridge that an unscaled first point introduces.
fn indirect_first_point_scale(hc: &HyperComplex2D, scale: f64) -> HyperComplex2D {
    let (width, points) = hc.shape();
    if points == 0 {
        return hc.clone();
    }
    let mut result = hc.clone();
    for plane in [
        &mut result.rr,
        &mut result.ri,
        &mut result.ir,
        &mut result.ii,
    ] {
        for value in plane.iter_mut().take(width) {
            *value *= scale;
        }
    }
    result
}

/// Applies an indirect-dimension exponential window in the indirect time
/// domain. No-op unless the indirect axis is in seconds.
fn indirect_exponential_apodization(
    hc: &HyperComplex2D,
    line_broadening_hz: f64,
) -> Result<HyperComplex2D> {
    if !line_broadening_hz.is_finite() {
        return Err(RSpinError::NonFinite {
            field: "indirect_line_broadening_hz",
        });
    }
    if hc.y.unit != rspin_core::Unit::Seconds {
        return Ok(hc.clone());
    }
    let (width, points) = hc.shape();
    let mut rr = hc.rr.clone();
    let mut ri = hc.ri.clone();
    let mut ir = hc.ir.clone();
    let mut ii = hc.ii.clone();
    for point in 0..points {
        let time_s = hc.y.values.get(point).copied().unwrap_or(0.0);
        let window = (-std::f64::consts::PI * line_broadening_hz * time_s).exp();
        for k in 0..width {
            let idx = point * width + k;
            rr[idx] *= window;
            ri[idx] *= window;
            ir[idx] *= window;
            ii[idx] *= window;
        }
    }
    let mut result = HyperComplex2D::new(
        hc.x.clone(),
        hc.y.clone(),
        rr,
        ri,
        ir,
        ii,
        hc.metadata.clone(),
    )?;
    result.processing.clone_from(&hc.processing);
    Ok(result.with_processing_record(
        ProcessingRecord::new("indirect_exponential_apodization")
            .with_details(format!("line_broadening_hz={line_broadening_hz}")),
    ))
}

/// Extends the indirect time dimension with zeros up to `target` points. A
/// `target` not larger than the current point count is a no-op.
fn indirect_zero_fill(hc: &HyperComplex2D, target: usize) -> Result<HyperComplex2D> {
    let (width, points) = hc.shape();
    if target <= points {
        return Ok(hc.clone());
    }
    let plane_len = width
        .checked_mul(target)
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "indirect zero-fill size overflow".to_owned(),
        })?;
    let extend = |source: &[f64]| -> Vec<f64> {
        let mut plane = vec![0.0_f64; plane_len];
        plane[..source.len()].copy_from_slice(source);
        plane
    };
    let rr = extend(&hc.rr);
    let ri = extend(&hc.ri);
    let ir = extend(&hc.ir);
    let ii = extend(&hc.ii);

    let step = if hc.y.values.len() >= 2 {
        hc.y.values[1] - hc.y.values[0]
    } else {
        0.0
    };
    let start = hc.y.values.first().copied().unwrap_or(0.0);
    let mut y_values = Vec::with_capacity(target);
    for point in 0..target {
        if let Some(value) = hc.y.values.get(point) {
            y_values.push(*value);
        } else {
            let offset = u32::try_from(point).map_err(|_| RSpinError::InvalidSpectrum {
                message: "indirect zero-fill target too large".to_owned(),
            })?;
            y_values.push(start + f64::from(offset) * step);
        }
    }
    let new_y = Axis::new(hc.y.label.clone(), hc.y.unit, y_values)?;

    let mut result = HyperComplex2D::new(hc.x.clone(), new_y, rr, ri, ir, ii, hc.metadata.clone())?;
    result.processing.clone_from(&hc.processing);
    Ok(result.with_processing_record(
        ProcessingRecord::new("indirect_zero_fill").with_details(format!("target={target}")),
    ))
}

fn denominator(len: usize) -> f64 {
    if len <= 1 {
        return 0.0;
    }
    u32::try_from(len - 1).map_or(0.0, f64::from)
}

fn fraction(index: usize, denominator: f64) -> f64 {
    if denominator == 0.0 {
        return 0.0;
    }
    u32::try_from(index).map_or(0.0, |value| f64::from(value) / denominator)
}

#[cfg(test)]
mod tests;
