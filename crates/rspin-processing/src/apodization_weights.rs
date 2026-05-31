//! Shared apodization weight kernels used by 1D and 2D processing.

use std::f64::consts::{LN_2, PI};

use rspin_core::{RSpinError, Result};

pub(crate) fn traf_weights(
    len: usize,
    line_broadening_hz: f64,
    dwell_time_s: f64,
    context: &'static str,
) -> Result<Vec<f64>> {
    ensure_non_negative("line_broadening_hz", line_broadening_hz)?;
    ensure_positive("dwell_time_s", dwell_time_s)?;
    let last_index = len.saturating_sub(1);
    let last_index_f = if last_index == 0 {
        0.0
    } else {
        f64::from(
            u32::try_from(last_index).map_err(|_| RSpinError::InvalidSpectrum {
                message: format!("{context} input is too large"),
            })?,
        )
    };
    let scale = -PI * line_broadening_hz * dwell_time_s;
    (0..len)
        .map(|index| {
            let index_f =
                f64::from(
                    u32::try_from(index).map_err(|_| RSpinError::InvalidSpectrum {
                        message: format!("{context} input is too large"),
                    })?,
                );
            let e_decay = (scale * index_f).exp();
            let r_decay = (scale * (last_index_f - index_f)).exp();
            let denominator = e_decay.powi(3) + r_decay.powi(3);
            let weight = if denominator <= 0.0 {
                0.0
            } else {
                e_decay.powi(2) / denominator
            };
            Ok(weight)
        })
        .collect()
}

pub(crate) fn trapezoidal_weights(
    len: usize,
    rise_end_fraction: f64,
    fall_start_fraction: f64,
    context: &'static str,
) -> Result<Vec<f64>> {
    ensure_finite("rise_end_fraction", rise_end_fraction)?;
    ensure_finite("fall_start_fraction", fall_start_fraction)?;
    if !(0.0..=1.0).contains(&rise_end_fraction) {
        return Err(RSpinError::InvalidSpectrum {
            message: "rise_end_fraction must be between 0 and 1".to_owned(),
        });
    }
    if !(0.0..=1.0).contains(&fall_start_fraction) {
        return Err(RSpinError::InvalidSpectrum {
            message: "fall_start_fraction must be between 0 and 1".to_owned(),
        });
    }
    if rise_end_fraction > fall_start_fraction {
        return Err(RSpinError::InvalidSpectrum {
            message: "rise_end_fraction must not exceed fall_start_fraction".to_owned(),
        });
    }

    let denominator = if len <= 1 {
        0.0
    } else {
        f64::from(
            u32::try_from(len - 1).map_err(|_| RSpinError::InvalidSpectrum {
                message: format!("{context} input is too large"),
            })?,
        )
    };
    (0..len)
        .map(|index| {
            let index_f =
                f64::from(
                    u32::try_from(index).map_err(|_| RSpinError::InvalidSpectrum {
                        message: format!("{context} input is too large"),
                    })?,
                );
            let fraction = if denominator == 0.0 {
                0.0
            } else {
                index_f / denominator
            };
            let weight = if fraction < rise_end_fraction {
                if rise_end_fraction <= 0.0 {
                    1.0
                } else {
                    fraction / rise_end_fraction
                }
            } else if fraction > fall_start_fraction {
                if fall_start_fraction >= 1.0 {
                    1.0
                } else {
                    (1.0 - fraction) / (1.0 - fall_start_fraction)
                }
            } else {
                1.0
            };
            Ok(weight.max(0.0))
        })
        .collect()
}

pub(crate) fn lorentz_to_gauss_weights(
    len: usize,
    lorentz_to_undo_hz: f64,
    gauss_fwhm_hz: f64,
    gauss_shift: f64,
    dwell_time_s: f64,
    context: &'static str,
) -> Result<Vec<f64>> {
    ensure_non_negative("lorentz_to_undo_hz", lorentz_to_undo_hz)?;
    ensure_non_negative("gauss_fwhm_hz", gauss_fwhm_hz)?;
    ensure_finite("gauss_shift", gauss_shift)?;
    if !(0.0..=1.0).contains(&gauss_shift) {
        return Err(RSpinError::InvalidSpectrum {
            message: "gauss_shift must be between 0 and 1".to_owned(),
        });
    }
    ensure_positive("dwell_time_s", dwell_time_s)?;

    let last_index = len.saturating_sub(1);
    let last_index_f = if last_index == 0 {
        0.0
    } else {
        f64::from(
            u32::try_from(last_index).map_err(|_| RSpinError::InvalidSpectrum {
                message: format!("{context} input is too large"),
            })?,
        )
    };
    let t_max = last_index_f * dwell_time_s;
    let lorentz_scale = PI * lorentz_to_undo_hz * dwell_time_s;
    let gauss_scale = PI * gauss_fwhm_hz;
    let gauss_norm = 4.0 * LN_2;
    let center_time = gauss_shift * t_max;
    (0..len)
        .map(|index| {
            let index_f =
                f64::from(
                    u32::try_from(index).map_err(|_| RSpinError::InvalidSpectrum {
                        message: format!("{context} input is too large"),
                    })?,
                );
            let t = index_f * dwell_time_s;
            let lorentz = (lorentz_scale * index_f).exp();
            let shifted = t - center_time;
            let gaussian = (-(gauss_scale * shifted).powi(2) / gauss_norm).exp();
            Ok(lorentz * gaussian)
        })
        .collect()
}

fn ensure_non_negative(field: &'static str, value: f64) -> Result<()> {
    ensure_finite(field, value)?;
    if value < 0.0 {
        return Err(RSpinError::InvalidSpectrum {
            message: format!("{field} must be non-negative"),
        });
    }
    Ok(())
}

fn ensure_positive(field: &'static str, value: f64) -> Result<()> {
    ensure_finite(field, value)?;
    if value <= 0.0 {
        return Err(RSpinError::InvalidSpectrum {
            message: format!("{field} must be positive"),
        });
    }
    Ok(())
}

fn ensure_finite(field: &'static str, value: f64) -> Result<()> {
    if !value.is_finite() {
        return Err(RSpinError::NonFinite { field });
    }
    Ok(())
}
