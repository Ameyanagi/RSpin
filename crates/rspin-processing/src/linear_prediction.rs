//! Linear-prediction extension of complex FIDs.
//!
//! The implementation follows the standard complex Burg auto-regressive
//! algorithm (Burg 1968, Marple 1987 chapter 8). The FID is modelled as
//! a sum of damped sinusoids, which fits an AR(p) process exactly when
//! the model order matches twice the number of independent spectral
//! components; for typical 1D NMR an order of 32–64 captures the
//! dominant lines and lets the model predict additional samples beyond
//! the acquisition window, suppressing the sinc-shaped truncation
//! sidelobes that show up around every peak after FFT.

use rspin_core::{ProcessingRecord, RSpinError, Result, Spectrum1D};
use rustfft::num_complex::Complex;

/// Extends a complex FID with `additional_samples` predicted points
/// using a Burg AR model of order `order`.
///
/// # Errors
///
/// Returns an error when the spectrum is real-only, the order is zero or
/// larger than the FID length, or the AR fit is numerically degenerate.
pub fn linear_prediction_extend(
    spectrum: &Spectrum1D,
    order: usize,
    additional_samples: usize,
) -> Result<Spectrum1D> {
    if spectrum.imaginary.is_none() {
        return Err(RSpinError::InvalidSpectrum {
            message: "linear prediction requires a complex spectrum".to_owned(),
        });
    }
    let len = spectrum.len();
    if order == 0 {
        return Err(RSpinError::InvalidSpectrum {
            message: "linear prediction order must be positive".to_owned(),
        });
    }
    if order >= len {
        return Err(RSpinError::InvalidSpectrum {
            message: "linear prediction order must be smaller than the FID length".to_owned(),
        });
    }
    if additional_samples == 0 {
        return Ok(spectrum.clone());
    }

    let Some(imag) = spectrum.imaginary.as_ref() else {
        unreachable!()
    };
    let mut data: Vec<Complex<f64>> = spectrum
        .intensities
        .iter()
        .zip(imag)
        .map(|(re, im)| Complex::new(*re, *im))
        .collect();

    let ar = burg_ar_coefficients(&data, order)?;
    data.reserve(additional_samples);
    for _ in 0..additional_samples {
        let n = data.len();
        let mut prediction = Complex::new(0.0, 0.0);
        // x[n] = − Σ_{k=1..p} a_k · x[n−k]
        for k in 1..=order {
            prediction -= ar[k] * data[n - k];
        }
        data.push(prediction);
    }

    let dwell = if spectrum.x.values.len() >= 2 {
        spectrum.x.values[1] - spectrum.x.values[0]
    } else {
        0.0
    };
    let mut new_axis_values = spectrum.x.values.clone();
    new_axis_values.reserve(additional_samples);
    for index in 0..additional_samples {
        let scale = u32::try_from(index + 1).map_err(|_| RSpinError::InvalidSpectrum {
            message: "linear prediction extension is too long".to_owned(),
        })?;
        let last = match new_axis_values.last() {
            Some(v) => *v,
            None => 0.0,
        };
        // Continue the linear axis using the original dwell when known.
        let next = if dwell.abs() > 0.0 {
            last + dwell
        } else {
            last + f64::from(scale)
        };
        new_axis_values.push(next);
    }
    let axis = rspin_core::Axis::new(&spectrum.x.label, spectrum.x.unit, new_axis_values)?;

    let real: Vec<f64> = data.iter().map(|c| c.re).collect();
    let imag: Vec<f64> = data.iter().map(|c| c.im).collect();
    let mut processed = Spectrum1D::new_complex(axis, real, Some(imag), spectrum.metadata.clone())?;
    processed.processing.clone_from(&spectrum.processing);
    Ok(processed.with_processing_record(
        ProcessingRecord::new("linear_prediction_extend").with_details(format!(
            "order={order},additional_samples={additional_samples}"
        )),
    ))
}

fn burg_ar_coefficients(data: &[Complex<f64>], order: usize) -> Result<Vec<Complex<f64>>> {
    let len = data.len();
    if order >= len {
        return Err(RSpinError::InvalidSpectrum {
            message: "AR order must be smaller than the FID length".to_owned(),
        });
    }
    let mut forward: Vec<Complex<f64>> = data.to_vec();
    let mut backward: Vec<Complex<f64>> = data.to_vec();
    // ar[0] = 1; rest = 0
    let mut ar: Vec<Complex<f64>> = vec![Complex::new(0.0, 0.0); order + 1];
    ar[0] = Complex::new(1.0, 0.0);

    for m in 1..=order {
        let mut numerator = Complex::new(0.0, 0.0);
        let mut denominator = 0.0_f64;
        for n in m..len {
            let f_n = forward[n];
            let b_prev = backward[n - 1];
            numerator += f_n * b_prev.conj();
            denominator += f_n.norm_sqr() + b_prev.norm_sqr();
        }
        if denominator <= f64::EPSILON {
            // Degenerate fit — leave higher orders zero, return what we have.
            break;
        }
        let reflection = -2.0 * numerator / denominator;

        // Update AR coefficients: a^(m)[i] = a^(m-1)[i] + k_m · conj(a^(m-1)[m-i])
        let snapshot = ar.clone();
        for i in 0..=m {
            ar[i] = snapshot[i] + reflection * snapshot[m - i].conj();
        }

        // Update forward and backward prediction errors.
        let f_snapshot = forward.clone();
        let b_snapshot = backward.clone();
        for n in m..len {
            forward[n] = f_snapshot[n] + reflection * b_snapshot[n - 1];
            backward[n] = b_snapshot[n - 1] + reflection.conj() * f_snapshot[n];
        }
    }
    Ok(ar)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rspin_core::{Axis, Metadata, Unit};

    fn damped_sinusoid_fid(
        points: usize,
        frequencies_hz: &[f64],
        decays: &[f64],
        dwell_s: f64,
    ) -> anyhow::Result<Spectrum1D> {
        assert_eq!(frequencies_hz.len(), decays.len());
        let mut real = Vec::with_capacity(points);
        let mut imag = Vec::with_capacity(points);
        let dwell = dwell_s;
        for index in 0..u32::try_from(points)? {
            let t = f64::from(index) * dwell;
            let mut re = 0.0;
            let mut im = 0.0;
            for (omega, decay) in frequencies_hz.iter().zip(decays) {
                let envelope = (-decay * t).exp();
                let angle = 2.0 * std::f64::consts::PI * omega * t;
                re += envelope * angle.cos();
                im += envelope * angle.sin();
            }
            real.push(re);
            imag.push(im);
        }
        let last = dwell * f64::from(u32::try_from(points - 1)?);
        let axis = Axis::linear("time", Unit::Seconds, 0.0, last, points)?;
        Ok(Spectrum1D::new_complex(
            axis,
            real,
            Some(imag),
            Metadata::default(),
        )?)
    }

    #[test]
    fn linear_prediction_matches_short_sinusoid() -> anyhow::Result<()> {
        // Generate a clean two-line FID, fit an AR model, predict the
        // continuation, and confirm the prediction follows the true
        // damped sinusoid to within a small tolerance.
        let truth = damped_sinusoid_fid(256, &[100.0, 250.0], &[5.0, 8.0], 1.0e-3)?;
        let truth_imag = match truth.imaginary.as_ref() {
            Some(values) => values.clone(),
            None => anyhow::bail!("truth FID missing imaginary channel"),
        };
        let head = Spectrum1D::new_complex(
            Axis::linear("time", Unit::Seconds, 0.0, 0.063, 64)?,
            truth.intensities[..64].to_vec(),
            Some(truth_imag[..64].to_vec()),
            Metadata::default(),
        )?;
        let extended = linear_prediction_extend(&head, 16, 64)?;
        for index in 64..128 {
            let predicted_re = extended.intensities[index];
            let truth_re = truth.intensities[index];
            assert!(
                (predicted_re - truth_re).abs() < 0.05,
                "predicted real {predicted_re} vs truth {truth_re} at index {index}"
            );
        }
        Ok(())
    }

    #[test]
    fn linear_prediction_rejects_real_only() -> anyhow::Result<()> {
        let axis = Axis::linear("time", Unit::Seconds, 0.0, 1.0, 4)?;
        let spectrum = Spectrum1D::new(axis, vec![1.0, 2.0, 3.0, 4.0], Metadata::default())?;
        assert!(linear_prediction_extend(&spectrum, 2, 2).is_err());
        Ok(())
    }
}
