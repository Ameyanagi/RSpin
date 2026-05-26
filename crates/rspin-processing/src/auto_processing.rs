//! One-call automatic processing pipeline for time-domain FIDs.
//!
//! [`process_spectrum_auto`] runs the canonical `RSpin` chain in order.
//! The pipeline is organised around three deliberately-separate
//! concepts (do not collapse them — most "phase mysteries" come from
//! conflating them):
//!
//! 1. **Time-origin correction**: vendor digital-filter group delay
//!    (Bruker `GRPDLY`, JEOL `decimation_reg / filter_factor`),
//!    integer-sample drop + zero-pad ([`remove_group_delay`]), and the
//!    matching fractional residual via the Fourier-shift theorem
//!    ([`apply_subsample_shift`]).
//!
//! 2. **Numerical FT conditioning**: optional backward linear
//!    prediction to repair leading samples, apodization, the FCOR=0.5
//!    first-point scale, and zero-filling.
//!
//! 3. **Spectral display correction**: residual `(ph0, ph1)` from
//!    [`auto_phase_correct`] and the [`subtract_baseline`] pass on
//!    the phased real spectrum.
//!
//! Auto-phase only ever sees the **residual** phase; if you see
//! `|ph1|` in the hundreds of degrees, the time-origin stage is wrong
//! (not the phaser). The default chain is tuned to give a "looks like
//! `nmrPipe` / `TopSpin` default output" result on the curated JEOL and
//! Bruker fixtures without the caller having to set anything beyond
//! the FID itself.
//!
//! Every step records its operation in the spectrum's processing
//! history; the caller can inspect [`Spectrum1D::processing`] after the
//! call to see exactly what was applied.

use rspin_core::{Metadata, Nucleus, RSpinError, Result, Spectrum1D, Unit};
use serde::{Deserialize, Serialize};

use crate::{
    AutoPhaseOptions, BaselineMethod, FftDirection, ProcessingRecipe1D, apply_subsample_shift,
    auto_phase_correct, auto_phase_correct_polynomial, exponential_apodization, first_point_scale,
    linear_predict_backward, matched_filter_em, remove_group_delay, subtract_baseline,
};

/// Options controlling [`process_spectrum_auto`].
///
/// Every field has a sensible default; tweak only when you need to
/// override the default behaviour for a specific dataset.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
#[allow(clippy::struct_excessive_bools)]
pub struct AutoProcessingOptions {
    /// Override the digital-filter group-delay value (`integer +
    /// fractional` samples). When `None` the value is recovered from
    /// `metadata.properties` for the supported vendors:
    ///
    /// - Bruker: `bruker.acqus.GRPDLY` (modern AVANCE; legacy
    ///   spectrometers without that field need an explicit override).
    /// - JEOL: `jeol.parameter.decimation_reg / jeol.parameter.filter_factor`.
    /// - Agilent/Varian: not auto-detected; modern Agilent FIDs are
    ///   typically pre-corrected by the spectrometer, but pass an
    ///   explicit value here if your dataset needs one.
    pub group_delay_samples: Option<f64>,
    /// Number of leading samples to repair with backward LP. Set to
    /// `0` (the default) to skip backward LP entirely. LP is opt-in to
    /// keep the default chain reversible and to avoid silently
    /// fabricating data on routine spectra.
    pub backward_lp_n_repair: usize,
    /// AR order for backward LP. Default 16.
    pub backward_lp_order: usize,
    /// Override the apodization line-broadening in Hz. When `None`
    /// the LB is picked from [`AutoProcessingOptions::nucleus_lb_hz`]
    /// using the spectrum's `Metadata::nucleus`.
    pub apodization_lb_hz: Option<f64>,
    /// LB look-up by nucleus when `apodization_lb_hz` is not set.
    /// Defaults: 1H → 0.3 Hz, 13C → 1.0 Hz, 15N/19F/31P → 2.0 Hz,
    /// unknown → matched-filter EM (auto).
    pub nucleus_lb_hz: NucleusLbDefaults,
    /// Apply the first-point scaling `s[0] *= 0.5` (`FCOR = 0.5`).
    ///
    /// Set to `false` when residual `ph1` after group-delay correction
    /// is expected to be nonzero (Bruker/JEOL with partial correction,
    /// non-causal FIDs) — halving here would inject a half-amplitude
    /// DC offset that the residual phase ramp turns into a visible
    /// dispersive baseline distortion.
    pub first_point_scale: bool,
    /// Zero-fill target multiplier relative to the (post-LP) FID
    /// length. The actual target is the next power of two greater
    /// than or equal to `multiplier * len`. Default 2 (one round of
    /// doubling).
    pub zero_fill_multiplier: usize,
    /// When `true`, run [`auto_phase_correct`] with the default
    /// Regions options on the FFT'd spectrum.
    pub auto_phase: bool,
    /// When `true`, run [`auto_phase_correct_polynomial`] to refine
    /// the linear `(ph0, ph1)` with quadratic + cubic terms (ph2, ph3).
    /// Has no effect when `auto_phase = false`.
    ///
    /// **Experimental — only enable when the linear fit is already
    /// near-zero.** Polynomial refinement is only meaningful once the
    /// group-delay correction has reduced the linear residual to a
    /// few degrees. When `|ph1|` is still in the hundreds (e.g.
    /// because the integer group-delay extraction is off by several
    /// samples), the polynomial optimiser amplifies the multi-turn
    /// ramp and produces visibly wrong dispersive spikes. Fix the
    /// upstream group-delay first.
    pub polynomial_phase_refine: bool,
    /// When `true`, apply [`subtract_baseline`] with the Whittaker
    /// AsLS-family default after auto-phase.
    pub subtract_baseline: bool,
}

impl Default for AutoProcessingOptions {
    fn default() -> Self {
        Self {
            group_delay_samples: None,
            // LP is off by default. Turning it on silently changes
            // quantitative data and is not safe as an unsupervised
            // default; users repair leading samples explicitly when
            // they know the FID needs it.
            backward_lp_n_repair: 0,
            backward_lp_order: 16,
            apodization_lb_hz: None,
            nucleus_lb_hz: NucleusLbDefaults::default(),
            first_point_scale: true,
            zero_fill_multiplier: 2,
            auto_phase: true,
            polynomial_phase_refine: false,
            subtract_baseline: true,
        }
    }
}

/// Per-nucleus default line-broadening overrides for
/// [`process_spectrum_auto`].
///
/// Any field set to `None` falls back to [`matched_filter_em`] for
/// that nucleus (or for spectra without a recognised nucleus label).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct NucleusLbDefaults {
    /// 1H default LB in Hz.
    pub hydrogen1_hz: Option<f64>,
    /// 13C default LB in Hz.
    pub carbon13_hz: Option<f64>,
    /// 15N default LB in Hz.
    pub nitrogen15_hz: Option<f64>,
    /// 19F default LB in Hz.
    pub fluorine19_hz: Option<f64>,
    /// 31P default LB in Hz.
    pub phosphorus31_hz: Option<f64>,
}

impl Default for NucleusLbDefaults {
    fn default() -> Self {
        Self {
            hydrogen1_hz: Some(0.3),
            carbon13_hz: Some(1.0),
            nitrogen15_hz: Some(2.0),
            fluorine19_hz: Some(2.0),
            phosphorus31_hz: Some(2.0),
        }
    }
}

impl NucleusLbDefaults {
    fn lookup(&self, nucleus: Option<&Nucleus>) -> Option<f64> {
        match nucleus? {
            Nucleus::Hydrogen1 => self.hydrogen1_hz,
            Nucleus::Carbon13 => self.carbon13_hz,
            Nucleus::Nitrogen15 => self.nitrogen15_hz,
            Nucleus::Fluorine19 => self.fluorine19_hz,
            Nucleus::Phosphorus31 => self.phosphorus31_hz,
            _ => None,
        }
    }
}

/// Runs the canonical `RSpin` processing pipeline on a time-domain FID
/// with sensible defaults, returning a phased + baseline-corrected
/// frequency-domain spectrum.
///
/// See [`AutoProcessingOptions`] for what each step does and how to
/// override it.
///
/// # Errors
///
/// Returns the first processing error from any of the underlying
/// steps; the FID must be time-domain (`axis.unit == Unit::Seconds`)
/// and have uniform dwell.
pub fn process_spectrum_auto(
    fid: &Spectrum1D,
    options: &AutoProcessingOptions,
) -> Result<Spectrum1D> {
    if fid.x.unit != Unit::Seconds {
        return Err(RSpinError::InvalidSpectrum {
            message: "process_spectrum_auto requires a time-domain FID (axis unit = Seconds)"
                .to_owned(),
        });
    }
    let dwell = uniform_dwell(&fid.x.values).ok_or(RSpinError::InvalidSpectrum {
        message: "process_spectrum_auto requires a uniformly-spaced time axis".to_owned(),
    })?;
    if dwell <= 0.0 {
        return Err(RSpinError::InvalidSpectrum {
            message: "process_spectrum_auto requires a positive dwell".to_owned(),
        });
    }

    // 1. Group-delay handling. Integer part is dropped + zero-padded
    //    in the time domain; the fractional residual is remembered and
    //    applied as a frequency-domain phase ramp after FFT.
    //
    //    Guard against double-correction: if the FID already carries a
    //    `remove_group_delay` record (e.g. it was re-imported from a
    //    pipeline that already corrected it), do not re-apply unless
    //    the caller explicitly overrides via `group_delay_samples`.
    let already_corrected = fid
        .processing
        .iter()
        .any(|record| record.operation == "remove_group_delay");
    let group_delay = match options.group_delay_samples {
        Some(value) => value,
        None if already_corrected => 0.0,
        None => group_delay_from_metadata(&fid.metadata),
    };
    let group_delay_integer = group_delay.trunc().max(0.0);
    let group_delay_frac = group_delay - group_delay_integer;
    let after_group_delay = if group_delay_integer > 0.0 {
        remove_group_delay(fid, group_delay_integer)?
    } else {
        fid.clone()
    };

    // 2. Optional backward linear-prediction repair on the first few
    //    samples that survive digital-filter ringing even after
    //    sub-sample group-delay correction.
    let after_lp = if options.backward_lp_n_repair > 0
        && options.backward_lp_order > 0
        && after_group_delay.imaginary.is_some()
        && after_group_delay.len() > options.backward_lp_n_repair * 2
        && after_group_delay.len() > options.backward_lp_order * 2
    {
        linear_predict_backward(
            &after_group_delay,
            options.backward_lp_order,
            options.backward_lp_n_repair,
        )?
    } else {
        after_group_delay
    };

    // 3. Nucleus-aware apodization. When no explicit LB is given, look
    //    up by nucleus; if neither hits, fall back to the matched
    //    filter (which derives LB from the FID itself).
    let lb_hz = match options.apodization_lb_hz {
        Some(lb) => lb,
        None => options
            .nucleus_lb_hz
            .lookup(after_lp.metadata.nucleus.as_ref())
            .unwrap_or_else(|| match matched_filter_em(&after_lp) {
                Ok(step) => step.line_broadening_hz,
                Err(_) => 0.5,
            }),
    };
    let after_apod = exponential_apodization(&after_lp, lb_hz, dwell)?;

    // 4. First-point scaling (FCOR=0.5).
    let after_fcor = if options.first_point_scale {
        first_point_scale(&after_apod, 0.5)?
    } else {
        after_apod
    };

    // 5. Zero-fill to the next power of two >= multiplier * len.
    let zero_fill_target = {
        let want = options
            .zero_fill_multiplier
            .max(1)
            .saturating_mul(after_fcor.len());
        next_power_of_two(want)
    };

    // 6 + 7. Apply zero-fill, FFT, and the fractional sub-sample shift.
    let mut recipe = ProcessingRecipe1D::new();
    if zero_fill_target > after_fcor.len() {
        recipe = recipe.zero_fill(zero_fill_target);
    }
    recipe = recipe.fft(FftDirection::Forward);
    let mut after_fft = recipe.apply(&after_fcor)?;
    if group_delay_frac.abs() > f64::EPSILON {
        after_fft = apply_subsample_shift(&after_fft, group_delay_frac)?;
    }

    // 8. Auto-phase. Linear (ph0, ph1) by default; refines with
    //    polynomial (ph2, ph3) when requested for spectra carrying
    //    residual frequency-dependent phase from a digital-filter
    //    compensator.
    let after_phase = if options.auto_phase {
        if options.polynomial_phase_refine {
            auto_phase_correct_polynomial(&after_fft, AutoPhaseOptions::default())?.spectrum
        } else {
            auto_phase_correct(&after_fft, AutoPhaseOptions::default())?.spectrum
        }
    } else {
        after_fft
    };

    // 9. Baseline subtraction with the default Whittaker method.
    let after_baseline = if options.subtract_baseline {
        match subtract_baseline(&after_phase, BaselineMethod::default()) {
            Ok(spectrum) => spectrum,
            // Baseline correction has its own validation; if it can't
            // run on this spectrum we silently skip rather than abort.
            Err(_) => after_phase,
        }
    } else {
        after_phase
    };

    Ok(after_baseline)
}

fn uniform_dwell(values: &[f64]) -> Option<f64> {
    if values.len() < 2 {
        return None;
    }
    let step = values[1] - values[0];
    if !step.is_finite() || step.abs() <= 0.0 {
        return None;
    }
    let tolerance = step.abs() * 1.0e-6;
    for window in values.windows(2) {
        let local = window[1] - window[0];
        if (local - step).abs() > tolerance {
            return None;
        }
    }
    Some(step.abs())
}

fn next_power_of_two(value: usize) -> usize {
    if value <= 1 {
        return 1;
    }
    let mut result = 1_usize;
    while result < value {
        result = match result.checked_mul(2) {
            Some(next) => next,
            None => return value,
        };
    }
    result
}

/// Recovers the digital-filter group delay from vendor metadata.
///
/// Per-vendor policy:
///
/// - **Bruker raw** (`fid` / `ser`): correct by default. Prefer
///   `GRPDLY` from the modern AVANCE `acqus` file; fall back to a
///   `DSPFVS + DECIM` lookup table for legacy spectrometers.
/// - **JEOL / Delta raw**: correct by default using
///   `decimation_reg / filter_factor`.
/// - **Varian / Agilent / `VnmrJ`**: no correction. Modern Agilent
///   FIDs come out of the spectrometer's inline DSP already time-
///   corrected; applying a Bruker-style shift on top would damage
///   them. Pass an explicit override via
///   [`AutoProcessingOptions::group_delay_samples`] when an unusual
///   dataset needs one.
/// - **`nmrPipe`-converted data, other formats, unknown vendors**: no
///   auto-correction — the conversion pipeline has likely already
///   handled it. Provide an explicit override when you have provenance.
///
/// Returns `0.0` when no recognised metadata is present.
fn group_delay_from_metadata(metadata: &Metadata) -> f64 {
    // Bruker: GRPDLY is the canonical digital-filter delay (samples,
    // typically a value like 67.98 on modern AVANCE). Where GRPDLY is
    // missing or non-positive (legacy spectrometers), fall back to the
    // documented DSPFVS+DECIM lookup table.
    let bruker = bruker_group_delay(metadata);
    if bruker > 0.0 {
        return bruker;
    }

    // JEOL: `decimation_reg / filter_factor` is the canonical formula
    // (decimation_reg is the FIR delay at the original ADC rate; the
    // ratio converts to stored-rate samples).
    //
    // Calibration on the curated Myrcene fixtures shows this matches
    // the empirically-required delay almost exactly for ¹H (≈ 52.125
    // samples, residual ph1 ≈ +5°), but underestimates ¹³C by ~5
    // stored-rate samples (formula gives 12.375, empirical best is
    // ≈ 17). The remaining discrepancy is not explained by any of
    // `orders`, `factors`, `filter_width`, `acq_delay`, or
    // `irr_dec_merit_factor` from the public JEOL `.jdf` schema.
    // We accept the formula as-is here; users with broader 13C
    // fixtures should pass `group_delay_samples` explicitly.
    let factor = metadata
        .properties
        .get("jeol.parameter.filter_factor")
        .and_then(|v| v.parse::<f64>().ok());
    let decim_raw = metadata
        .properties
        .get("jeol.parameter.decimation_reg")
        .and_then(|v| parse_decimation_reg(v));
    if let (Some(raw), Some(f)) = (decim_raw, factor) {
        if f.is_finite() && f > 0.0 {
            return raw / f;
        }
    }

    0.0
}

fn bruker_group_delay(metadata: &Metadata) -> f64 {
    if let Some(grpdly) = metadata
        .properties
        .get("bruker.acqus.GRPDLY")
        .and_then(|raw| raw.trim().parse::<f64>().ok())
    {
        if grpdly.is_finite() && grpdly > 0.0 {
            return grpdly;
        }
    }
    let dspfvs = metadata
        .properties
        .get("bruker.acqus.DSPFVS")
        .and_then(|raw| raw.trim().parse::<i32>().ok());
    let decim = metadata
        .properties
        .get("bruker.acqus.DECIM")
        .and_then(|raw| raw.trim().parse::<i32>().ok());
    match (dspfvs, decim) {
        (Some(d), Some(c)) => bruker_dsp_table(d, c).unwrap_or(0.0),
        _ => 0.0,
    }
}

/// Legacy Bruker `DSPFVS + DECIM → GRPDLY` lookup table.
///
/// Values are numerical constants documented in the Bruker XWIN-NMR
/// digital-filter manual and reproduced in every Bruker-aware
/// processing toolkit (`nmrglue` BSD-3, `nmrPipe` documentation,
/// `relax`). Used only when the FID lacks an explicit `GRPDLY`
/// (pre-`AVANCE`-III spectrometers).
#[allow(clippy::too_many_lines, clippy::match_same_arms)]
fn bruker_dsp_table(dspfvs: i32, decim: i32) -> Option<f64> {
    match (dspfvs, decim) {
        (10, 2) => Some(44.75),
        (10, 3) => Some(33.5),
        (10, 4) => Some(66.625),
        (10, 6) => Some(59.083_333_333_3),
        (10, 8) => Some(68.562_5),
        (10, 12) => Some(60.375),
        (10, 16) => Some(69.531_25),
        (10, 24) => Some(61.020_833_333_3),
        (10, 32) => Some(70.015_625),
        (10, 48) => Some(61.510_416_666_7),
        (10, 64) => Some(70.257_812_5),
        (10, 96) => Some(61.755_208_333_3),
        (10, 128) => Some(70.378_906_25),
        (10, 192) => Some(61.877_604_166_7),
        (10, 256) => Some(70.439_453_125),
        (10, 384) => Some(61.938_802_083_3),
        (10, 512) => Some(70.469_726_562_5),
        (10, 768) => Some(61.969_401_041_7),
        (10, 1024) => Some(70.484_863_281_3),
        (10, 1536) => Some(61.984_700_520_8),
        (10, 2048) => Some(70.492_431_640_6),

        (11, 2) => Some(46.0),
        (11, 3) => Some(36.5),
        (11, 4) => Some(48.0),
        (11, 6) => Some(50.166_666_666_7),
        (11, 8) => Some(53.25),
        (11, 12) => Some(69.5),
        (11, 16) => Some(72.25),
        (11, 24) => Some(70.166_666_666_7),
        (11, 32) => Some(72.75),
        (11, 48) => Some(70.5),
        (11, 64) => Some(73.0),
        (11, 96) => Some(70.666_666_666_7),
        (11, 128) => Some(72.5),
        (11, 192) => Some(71.333_333_333_3),
        (11, 256) => Some(72.25),
        (11, 384) => Some(71.666_666_666_7),
        (11, 512) => Some(72.125),
        (11, 768) => Some(71.833_333_333_3),
        (11, 1024) => Some(72.062_5),
        (11, 1536) => Some(71.916_666_666_7),
        (11, 2048) => Some(72.031_25),

        (12, 2) => Some(46.0),
        (12, 3) => Some(36.5),
        (12, 4) => Some(48.0),
        (12, 6) => Some(50.166_666_666_7),
        (12, 8) => Some(53.25),
        (12, 12) => Some(69.5),
        (12, 16) => Some(71.625),
        (12, 24) => Some(70.166_666_666_7),
        (12, 32) => Some(72.125),
        (12, 48) => Some(70.5),
        (12, 64) => Some(72.375),
        (12, 96) => Some(70.666_666_666_7),
        (12, 128) => Some(72.5),

        (13, 2) => Some(2.75),
        (13, 3) => Some(2.833_333_333_3),
        (13, 4) => Some(2.875),
        (13, 6) => Some(2.916_666_666_7),
        (13, 8) => Some(2.937_5),
        (13, 12) => Some(2.958_333_333_3),
        (13, 16) => Some(2.968_75),
        (13, 24) => Some(2.979_166_666_7),
        (13, 32) => Some(2.984_375),
        (13, 48) => Some(2.989_583_333_3),
        (13, 64) => Some(2.992_187_5),
        (13, 96) => Some(2.994_791_666_7),
        (13, 128) => Some(2.996_093_75),

        _ => None,
    }
}

fn parse_decimation_reg(raw: &str) -> Option<f64> {
    let trimmed = raw.trim();
    let after = trimmed.strip_prefix("r:")?.trim_start();
    let first_token = after.split(|c: char| !c.is_ascii_digit()).next()?;
    first_token.parse::<f64>().ok()
}

#[cfg(test)]
mod tests;
