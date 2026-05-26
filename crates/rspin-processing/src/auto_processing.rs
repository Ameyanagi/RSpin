//! One-call automatic processing pipeline for time-domain FIDs.
//!
//! [`process_spectrum_auto`] runs the canonical `RSpin` chain in order
//! (group-delay correction → optional backward LP → nucleus-aware
//! apodization → first-point scaling → zero-fill → FFT → fractional
//! sub-sample shift → Regions auto-phase → `AsLS` baseline). The defaults
//! are tuned to give a "looks like `nmrPipe` / `TopSpin` default output"
//! result on the curated JEOL and Bruker fixtures without the caller
//! having to set anything beyond the FID itself.
//!
//! Every step records its operation in the spectrum's processing
//! history; the caller can inspect [`Spectrum1D::processing`] after the
//! call to see exactly what was applied.

use rspin_core::{Metadata, Nucleus, RSpinError, Result, Spectrum1D, Unit};
use serde::{Deserialize, Serialize};

use crate::{
    AutoPhaseOptions, BaselineMethod, FftDirection, ProcessingRecipe1D, apply_subsample_shift,
    auto_phase_correct, exponential_apodization, first_point_scale, linear_predict_backward,
    matched_filter_em, remove_group_delay, subtract_baseline,
};

/// Options controlling [`process_spectrum_auto`].
///
/// Every field has a sensible default; tweak only when you need to
/// override the default behaviour for a specific dataset.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AutoProcessingOptions {
    /// Override the digital-filter group-delay value (`integer +
    /// fractional` samples). When `None` the value is recovered from
    /// `metadata.properties` (JEOL `decimation_reg / filter_factor`).
    pub group_delay_samples: Option<f64>,
    /// Number of leading samples to repair with backward LP. Set to
    /// `0` to skip backward LP entirely. Default 8.
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
    /// Apply the first-point scaling `s[0] *= 0.5` (FCOR=0.5 default).
    pub first_point_scale: bool,
    /// Zero-fill target multiplier relative to the (post-LP) FID
    /// length. The actual target is the next power of two greater
    /// than or equal to `multiplier * len`. Default 2 (one round of
    /// doubling).
    pub zero_fill_multiplier: usize,
    /// When `true`, run [`auto_phase_correct`] with the default
    /// Regions options on the FFT'd spectrum.
    pub auto_phase: bool,
    /// When `true`, apply [`subtract_baseline`] with the Whittaker
    /// AsLS-family default after auto-phase.
    pub subtract_baseline: bool,
}

impl Default for AutoProcessingOptions {
    fn default() -> Self {
        Self {
            group_delay_samples: None,
            backward_lp_n_repair: 8,
            backward_lp_order: 16,
            apodization_lb_hz: None,
            nucleus_lb_hz: NucleusLbDefaults::default(),
            first_point_scale: true,
            zero_fill_multiplier: 2,
            auto_phase: true,
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

    // 1. Group-delay handling. Integer part is applied in the time
    //    domain (rotate_left); the fractional residual is remembered
    //    and applied as a frequency-domain phase ramp after FFT.
    let group_delay = options
        .group_delay_samples
        .unwrap_or_else(|| jeol_group_delay_from_metadata(&fid.metadata));
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

    // 8. Auto-phase with Regions defaults.
    let after_phase = if options.auto_phase {
        auto_phase_correct(&after_fft, AutoPhaseOptions::default())?.spectrum
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

fn jeol_group_delay_from_metadata(metadata: &Metadata) -> f64 {
    let factor = metadata
        .properties
        .get("jeol.parameter.filter_factor")
        .and_then(|v| v.parse::<f64>().ok());
    let decim_raw = metadata
        .properties
        .get("jeol.parameter.decimation_reg")
        .and_then(|v| parse_decimation_reg(v));
    match (decim_raw, factor) {
        (Some(raw), Some(f)) if f > 0.0 => raw / f,
        _ => 0.0,
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
