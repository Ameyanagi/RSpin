//! Complex linear prediction (Burg algorithm) for FID repair and extension.
//!
//! The complex Burg algorithm fits an autoregressive (AR) model
//!
//! ```text
//! s[n] = -Σ_{k=1..p} a_k · s[n-k] + ε[n]
//! ```
//!
//! to a complex FID and uses the AR coefficients to predict missing or
//! extrapolated samples. Two directions are supported:
//!
//! - [`linear_predict_backward`] repairs the first `n_repair` samples of
//!   a FID, fixing the digital-filter ringing and receiver dead-time
//!   artefacts that survive even fractional sub-sample group-delay
//!   correction. Apply *before* apodization.
//! - [`linear_predict_forward`] extends the FID tail by `n_extend`
//!   samples, doubling effective acquisition time when SNR permits.
//!
//! References (clean-room — no code copied):
//!
//! - Burg, *Maximum Entropy Spectral Analysis*, Stanford thesis (1975).
//! - Marple, *Digital Spectral Analysis with Applications* (1987), §8.4.
//! - `nmrglue` `proc_lp` (BSD-3) — behaviour reference for NMR conventions.
//!
//! Notes / limitations:
//!
//! - The current implementation fits the Burg coefficients but does not
//!   perform pole-reflection. For well-conditioned FIDs (modest noise,
//!   decaying complex exponentials) the poles naturally lie inside the
//!   unit circle and the prediction is stable. For very low-SNR data the
//!   forward extension can diverge; consider reducing the model order or
//!   using a smaller `n_extend`.

use rspin_core::{Axis, ProcessingRecord, RSpinError, Result, Spectrum1D, Unit};
use rustfft::num_complex::Complex;
use serde::{Deserialize, Serialize};

use crate::ProcessingStep;
use crate::transform::complex_buffer;

/// Processing step that repairs the first `n_repair` FID samples with
/// backward complex Burg linear prediction.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct LinearPredictionBackward {
    /// AR model order; typical values 8-32, must be `< spectrum.len() / 2`.
    pub order: usize,
    /// Number of leading samples to overwrite with predicted values.
    pub n_repair: usize,
}

impl LinearPredictionBackward {
    /// Creates a backward LP step.
    #[must_use]
    pub fn new(order: usize, n_repair: usize) -> Self {
        Self { order, n_repair }
    }
}

impl ProcessingStep<Spectrum1D> for LinearPredictionBackward {
    fn apply(&self, spectrum: &Spectrum1D) -> Result<Spectrum1D> {
        linear_predict_backward(spectrum, self.order, self.n_repair)
    }
}

/// Processing step that extends the FID tail by `n_extend` samples
/// with forward complex Burg linear prediction.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct LinearPredictionForward {
    /// AR model order; typical values 8-32, must be `< spectrum.len() / 2`.
    pub order: usize,
    /// Number of samples to append after the existing FID tail.
    pub n_extend: usize,
}

impl LinearPredictionForward {
    /// Creates a forward LP step.
    #[must_use]
    pub fn new(order: usize, n_extend: usize) -> Self {
        Self { order, n_extend }
    }
}

impl ProcessingStep<Spectrum1D> for LinearPredictionForward {
    fn apply(&self, spectrum: &Spectrum1D) -> Result<Spectrum1D> {
        linear_predict_forward(spectrum, self.order, self.n_extend)
    }
}

/// Repairs the first `n_repair` samples of a complex FID via backward
/// linear prediction.
///
/// Internally: reverse the FID (and conjugate the imaginary part to
/// preserve the AR model direction), fit complex Burg coefficients of
/// the requested order on the reversed FID, predict `n_repair` extra
/// samples (which correspond to the corrupted leading samples of the
/// original FID), reverse the prediction back, and splice it in front
/// of the surviving samples.
///
/// # Errors
///
/// Returns an error when the axis unit is not [`Unit::Seconds`], the
/// spectrum lacks an imaginary channel, the requested `order` exceeds
/// `(spectrum.len() - n_repair) / 2`, or the input is too short for
/// the fit.
pub fn linear_predict_backward(
    spectrum: &Spectrum1D,
    order: usize,
    n_repair: usize,
) -> Result<Spectrum1D> {
    validate_time_domain(spectrum, "linear_predict_backward")?;
    if order == 0 || n_repair == 0 {
        return Err(RSpinError::InvalidSpectrum {
            message: "linear_predict_backward requires order >= 1 and n_repair >= 1".to_owned(),
        });
    }
    let total = spectrum.len();
    if total <= n_repair {
        return Err(RSpinError::InvalidSpectrum {
            message: "linear_predict_backward requires more samples than n_repair".to_owned(),
        });
    }
    let usable = total - n_repair;
    if order >= usable / 2 || order >= usable {
        return Err(RSpinError::InvalidSpectrum {
            message: format!(
                "linear_predict_backward requires order < (len - n_repair) / 2; got order={order}, usable={usable}"
            ),
        });
    }

    // Take the surviving tail (samples after the corrupted first
    // n_repair) and time-reverse it (with conjugation) so forward
    // prediction on it corresponds to backward prediction on the
    // original FID.
    let buffer = complex_buffer(spectrum);
    let tail: Vec<Complex<f64>> = buffer[n_repair..].iter().rev().map(Complex::conj).collect();

    // Fit Burg on the reversed tail and predict n_repair more samples.
    let coeffs = burg_complex(&tail, order)?;
    let extended = extend_forward(&tail, &coeffs, n_repair);

    // Convert the n_repair predicted (reversed, conjugated) samples
    // back into the original leading samples of the FID.
    let predicted_leading: Vec<Complex<f64>> = extended[tail.len()..]
        .iter()
        .rev()
        .map(Complex::conj)
        .collect();
    debug_assert_eq!(predicted_leading.len(), n_repair);

    let mut new_buffer = Vec::with_capacity(total);
    new_buffer.extend_from_slice(&predicted_leading);
    new_buffer.extend_from_slice(&buffer[n_repair..]);

    write_complex_buffer(
        spectrum,
        &new_buffer,
        ProcessingRecord::new("linear_predict_backward")
            .with_details(format!("order={order},n_repair={n_repair}")),
    )
}

/// Extends the FID tail by `n_extend` samples via forward linear
/// prediction.
///
/// # Errors
///
/// Returns an error when the axis unit is not [`Unit::Seconds`], the
/// spectrum lacks an imaginary channel, the requested `order` exceeds
/// `spectrum.len() / 2`, or the input is too short for the fit.
pub fn linear_predict_forward(
    spectrum: &Spectrum1D,
    order: usize,
    n_extend: usize,
) -> Result<Spectrum1D> {
    validate_time_domain(spectrum, "linear_predict_forward")?;
    if order == 0 || n_extend == 0 {
        return Err(RSpinError::InvalidSpectrum {
            message: "linear_predict_forward requires order >= 1 and n_extend >= 1".to_owned(),
        });
    }
    let total = spectrum.len();
    if order >= total / 2 || order >= total {
        return Err(RSpinError::InvalidSpectrum {
            message: format!(
                "linear_predict_forward requires order < len/2; got order={order}, len={total}"
            ),
        });
    }

    let buffer = complex_buffer(spectrum);
    let coeffs = burg_complex(&buffer, order)?;
    let extended = extend_forward(&buffer, &coeffs, n_extend);

    let mut new_spectrum = write_complex_buffer(
        spectrum,
        &extended,
        ProcessingRecord::new("linear_predict_forward")
            .with_details(format!("order={order},n_extend={n_extend}")),
    )?;
    // Extend the time axis with uniform-step continuation.
    new_spectrum.x = extend_time_axis(&spectrum.x, n_extend)?;
    Ok(new_spectrum)
}

fn validate_time_domain(spectrum: &Spectrum1D, op: &'static str) -> Result<()> {
    if spectrum.x.unit != Unit::Seconds {
        return Err(RSpinError::InvalidSpectrum {
            message: format!("{op} requires a time-domain FID (axis unit = Seconds)"),
        });
    }
    if spectrum.imaginary.is_none() {
        return Err(RSpinError::InvalidSpectrum {
            message: format!("{op} requires a complex spectrum with an imaginary channel"),
        });
    }
    Ok(())
}

/// Complex Burg algorithm — returns AR coefficients `a_1..a_p` so that
/// `s[n] ≈ -Σ a_k · s[n-k]`.
fn burg_complex(samples: &[Complex<f64>], order: usize) -> Result<Vec<Complex<f64>>> {
    let n = samples.len();
    if order == 0 || order >= n {
        return Err(RSpinError::InvalidSpectrum {
            message: format!("burg_complex needs order < len; got order={order}, len={n}"),
        });
    }

    // Forward and backward prediction error sequences.
    let mut forward: Vec<Complex<f64>> = samples.to_vec();
    let mut backward: Vec<Complex<f64>> = samples.to_vec();
    // AR coefficients (length-`order` vector, initially zero).
    let mut a: Vec<Complex<f64>> = vec![Complex::new(0.0, 0.0); order];

    for m in 0..order {
        // Numerator and denominator of the reflection coefficient.
        let mut numerator = Complex::new(0.0, 0.0);
        let mut denominator = 0.0_f64;
        for i in (m + 1)..n {
            numerator += forward[i] * backward[i - 1].conj();
            denominator += forward[i].norm_sqr() + backward[i - 1].norm_sqr();
        }
        if denominator <= 0.0 {
            return Err(RSpinError::InvalidSpectrum {
                message: "burg_complex prediction error vanished; reduce order".to_owned(),
            });
        }
        let k = numerator * (-2.0 / denominator);

        // Update the forward and backward error sequences.
        for i in ((m + 1)..n).rev() {
            let f_old = forward[i];
            let b_old = backward[i - 1];
            forward[i] = f_old + k * b_old;
            backward[i] = b_old + k.conj() * f_old;
        }

        // Update the AR coefficient vector using the Levinson-Durbin
        // recursion on top of the new reflection coefficient.
        let prev = a[..m].to_vec();
        a[m] = k;
        for j in 0..m {
            a[j] = prev[j] + k * prev[m - 1 - j].conj();
        }
    }
    Ok(a)
}

/// Extends `samples` by predicting `n_extend` more samples using the
/// AR model `s[n] = -Σ a_k · s[n-k]`.
fn extend_forward(
    samples: &[Complex<f64>],
    coeffs: &[Complex<f64>],
    n_extend: usize,
) -> Vec<Complex<f64>> {
    let mut extended = Vec::with_capacity(samples.len() + n_extend);
    extended.extend_from_slice(samples);
    for _ in 0..n_extend {
        let len = extended.len();
        let mut next = Complex::new(0.0, 0.0);
        for (k, coef) in coeffs.iter().enumerate() {
            next -= *coef * extended[len - 1 - k];
        }
        extended.push(next);
    }
    extended
}

fn write_complex_buffer(
    spectrum: &Spectrum1D,
    buffer: &[Complex<f64>],
    record: ProcessingRecord,
) -> Result<Spectrum1D> {
    let real: Vec<f64> = buffer.iter().map(|c| c.re).collect();
    let imag: Vec<f64> = buffer.iter().map(|c| c.im).collect();
    let new_axis = if buffer.len() == spectrum.len() {
        spectrum.x.clone()
    } else {
        extend_time_axis(&spectrum.x, buffer.len() - spectrum.len())?
    };
    let mut processed =
        Spectrum1D::new_complex(new_axis, real, Some(imag), spectrum.metadata.clone())?;
    processed.processing.clone_from(&spectrum.processing);
    Ok(processed.with_processing_record(record))
}

fn extend_time_axis(axis: &Axis, n_extra: usize) -> Result<Axis> {
    if axis.values.len() < 2 {
        return Ok(axis.clone());
    }
    let step = axis.values[1] - axis.values[0];
    if !step.is_finite() {
        return Err(RSpinError::InvalidSpectrum {
            message: "linear-prediction axis step is non-finite".to_owned(),
        });
    }
    let mut extended = axis.values.clone();
    let last = axis
        .values
        .last()
        .copied()
        .ok_or(RSpinError::InvalidSpectrum {
            message: "linear-prediction axis has no samples".to_owned(),
        })?;
    for i in 0..n_extra {
        let index_u32 = u32::try_from(i + 1).map_err(|_| RSpinError::InvalidSpectrum {
            message: "linear-prediction extension is too long".to_owned(),
        })?;
        extended.push(last + step * f64::from(index_u32));
    }
    Axis::new(axis.label.as_str(), axis.unit, extended)
}

#[cfg(test)]
mod tests;
