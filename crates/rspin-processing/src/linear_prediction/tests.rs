use std::f64::consts::PI;

use rspin_core::{Axis, Metadata, RSpinError, Spectrum1D, Unit};

use super::*;

/// Builds a noise-free quadrature FID containing a single complex
/// exponential at `freq_hz` with decay `lb_hz`.
fn single_exponential_fid(
    n: u32,
    dwell: f64,
    freq_hz: f64,
    lb_hz: f64,
) -> anyhow::Result<Spectrum1D> {
    let mut times = Vec::with_capacity(usize::try_from(n)?);
    let mut real = Vec::with_capacity(usize::try_from(n)?);
    let mut imag = Vec::with_capacity(usize::try_from(n)?);
    for i in 0..n {
        let t = f64::from(i) * dwell;
        times.push(t);
        let envelope = (-PI * lb_hz * t).exp();
        let phase = 2.0 * PI * freq_hz * t;
        real.push(envelope * phase.cos());
        imag.push(envelope * phase.sin());
    }
    let axis = Axis::new("time", Unit::Seconds, times)?;
    Ok(Spectrum1D::new_complex(
        axis,
        real,
        Some(imag),
        Metadata::default(),
    )?)
}

fn assert_complex_close(
    actual_re: f64,
    actual_im: f64,
    expected_re: f64,
    expected_im: f64,
    tol: f64,
) {
    let dre = (actual_re - expected_re).abs();
    let dim = (actual_im - expected_im).abs();
    assert!(
        dre < tol && dim < tol,
        "expected ({expected_re}, {expected_im}), got ({actual_re}, {actual_im}); tol={tol}"
    );
}

#[test]
fn forward_lp_predicts_pure_exponential_within_tolerance() -> anyhow::Result<()> {
    // A noise-free complex exponential is a single-pole AR(1) process,
    // so Burg should recover the decay essentially perfectly and
    // forward extension should be very close to the analytic value.
    let dwell = 1.0e-3_f64;
    let lb_hz = 4.0_f64;
    let freq_hz = 50.0_f64;
    let n: u32 = 256;
    let fid = single_exponential_fid(n, dwell, freq_hz, lb_hz)?;
    let n_extend: usize = 16;
    let extended = linear_predict_forward(&fid, 8, n_extend)?;
    assert_eq!(extended.len(), usize::try_from(n)? + n_extend);
    let imag = extended
        .imaginary
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("imaginary lost"))?;
    for i in 0..n_extend {
        let idx = usize::try_from(n)? + i;
        let t = f64::from(u32::try_from(idx)?) * dwell;
        let envelope = (-PI * lb_hz * t).exp();
        let phase = 2.0 * PI * freq_hz * t;
        let expected_re = envelope * phase.cos();
        let expected_im = envelope * phase.sin();
        assert_complex_close(
            extended.intensities[idx],
            imag[idx],
            expected_re,
            expected_im,
            5.0e-3,
        );
    }
    assert_eq!(
        extended.processing.last().map(|r| r.operation.as_str()),
        Some("linear_predict_forward")
    );
    Ok(())
}

#[test]
fn backward_lp_repairs_corrupted_first_samples() -> anyhow::Result<()> {
    // Build a noise-free FID, overwrite the first 4 samples with
    // bogus values, then ask backward LP to repair them. The repaired
    // values should agree with the analytic FID within a small
    // tolerance.
    let dwell = 1.0e-3_f64;
    let lb_hz = 4.0_f64;
    let freq_hz = 50.0_f64;
    let n: u32 = 256;
    let truth = single_exponential_fid(n, dwell, freq_hz, lb_hz)?;
    let n_repair: usize = 4;
    let mut corrupted = truth.clone();
    for i in 0..n_repair {
        corrupted.intensities[i] = 99.0;
        if let Some(imag) = corrupted.imaginary.as_mut() {
            imag[i] = -99.0;
        }
    }
    let repaired = linear_predict_backward(&corrupted, 8, n_repair)?;
    let imag = repaired
        .imaginary
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("imaginary lost"))?;
    let truth_imag = truth
        .imaginary
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("imaginary lost"))?;
    for i in 0..n_repair {
        assert_complex_close(
            repaired.intensities[i],
            imag[i],
            truth.intensities[i],
            truth_imag[i],
            5.0e-3,
        );
    }
    // The surviving tail must be untouched.
    for i in n_repair..usize::try_from(n)? {
        assert!((repaired.intensities[i] - truth.intensities[i]).abs() < 1.0e-15);
        assert!((imag[i] - truth_imag[i]).abs() < 1.0e-15);
    }
    assert_eq!(
        repaired.processing.last().map(|r| r.operation.as_str()),
        Some("linear_predict_backward")
    );
    Ok(())
}

#[test]
fn linear_predict_forward_extends_time_axis_uniformly() -> anyhow::Result<()> {
    let dwell = 1.0e-3_f64;
    let fid = single_exponential_fid(64, dwell, 10.0, 2.0)?;
    let extended = linear_predict_forward(&fid, 4, 8)?;
    // Step between consecutive axis values should match the dwell.
    for window in extended.x.values.windows(2) {
        let step = window[1] - window[0];
        assert!((step - dwell).abs() < 1.0e-12);
    }
    Ok(())
}

#[test]
fn linear_prediction_rejects_invalid_inputs() -> anyhow::Result<()> {
    let fid = single_exponential_fid(64, 1.0e-3, 10.0, 2.0)?;
    // Frequency-domain spectrum: error.
    let freq_axis = Axis::linear("freq", Unit::Hertz, -1.0, 1.0, 64)?;
    let freq_spectrum = Spectrum1D::new_complex(
        freq_axis,
        vec![1.0; 64],
        Some(vec![0.0; 64]),
        Metadata::default(),
    )?;
    let err = linear_predict_backward(&freq_spectrum, 8, 4).expect_err("freq input should fail");
    assert!(matches!(err, RSpinError::InvalidSpectrum { .. }));

    // Order = 0.
    assert!(linear_predict_forward(&fid, 0, 4).is_err());
    assert!(linear_predict_backward(&fid, 0, 4).is_err());
    // n_repair / n_extend = 0.
    assert!(linear_predict_forward(&fid, 4, 0).is_err());
    assert!(linear_predict_backward(&fid, 4, 0).is_err());
    // Order too large.
    assert!(linear_predict_forward(&fid, 64, 4).is_err());
    assert!(linear_predict_backward(&fid, 64, 4).is_err());
    Ok(())
}
