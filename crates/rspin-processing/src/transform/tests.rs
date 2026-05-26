use std::f64::consts::{LN_2, PI};

use rspin_core::{Axis, Metadata, RSpinError, Spectrum1D, Unit};

use super::*;

#[test]
fn remove_group_delay_drops_leading_samples_and_zero_pads() -> anyhow::Result<()> {
    // The discarded pre-acquisition samples are dropped, not wrapped.
    // This avoids the wrap-around baseline artefact on the spectrum
    // edges that a circular shift produced.
    let axis = Axis::linear("time", Unit::Seconds, 0.0, 4.0, 5)?;
    let spectrum = Spectrum1D::new_complex(
        axis,
        vec![10.0, 20.0, 30.0, 40.0, 50.0],
        Some(vec![1.0, 2.0, 3.0, 4.0, 5.0]),
        Metadata::default(),
    )?;
    let shifted = remove_group_delay(&spectrum, 2.0)?;
    assert_eq!(shifted.intensities, vec![30.0, 40.0, 50.0, 0.0, 0.0]);
    assert_eq!(shifted.imaginary, Some(vec![3.0, 4.0, 5.0, 0.0, 0.0]));
    assert_eq!(
        shifted.processing.last().map(|r| r.operation.as_str()),
        Some("remove_group_delay")
    );
    Ok(())
}

#[test]
fn remove_group_delay_rejects_invalid_input() -> anyhow::Result<()> {
    let axis = Axis::linear("time", Unit::Seconds, 0.0, 1.0, 2)?;
    let spectrum = Spectrum1D::new_complex(
        axis,
        vec![1.0, 2.0],
        Some(vec![0.0, 0.0]),
        Metadata::default(),
    )?;
    assert!(remove_group_delay(&spectrum, -1.0).is_err());
    assert!(remove_group_delay(&spectrum, f64::NAN).is_err());
    Ok(())
}

#[test]
fn apodization_decays_real_and_imaginary_channels() -> anyhow::Result<()> {
    let spectrum = complex_spectrum()?;
    let processed = exponential_apodization(&spectrum, 1.0, 0.1)?;
    assert_close(processed.intensities[0], 1.0);
    assert!(processed.intensities[1] < 2.0);
    let imaginary = require_imaginary(&processed)?;
    assert_close(imaginary[0], 0.5);
    assert!(imaginary[1] < 1.0);
    Ok(())
}

#[test]
fn gaussian_apodization_damps_real_and_imaginary_channels() -> anyhow::Result<()> {
    let spectrum = complex_spectrum()?;
    let processed = gaussian_apodization(&spectrum, 1.0, 0.1)?;
    let weight_one = (-(PI * 0.1_f64).powi(2) / (4.0 * LN_2)).exp();
    let weight_two = (-(PI * 0.2_f64).powi(2) / (4.0 * LN_2)).exp();

    assert_close(processed.intensities[0], 1.0);
    assert_close(processed.intensities[1], 2.0 * weight_one);
    assert_close(processed.intensities[2], 4.0 * weight_two);
    let imaginary = require_imaginary(&processed)?;
    assert_close(imaginary[0], 0.5);
    assert_close(imaginary[1], weight_one);
    assert_close(imaginary[2], 0.0);
    assert_eq!(processed.processing[0].operation, "gaussian_apodization");
    Ok(())
}

#[test]
fn sine_bell_apodization_weights_real_and_imaginary_channels() -> anyhow::Result<()> {
    let spectrum = complex_spectrum()?;
    let processed = sine_bell_apodization(&spectrum, 0.0, 180.0, 1.0)?;

    assert_close(processed.intensities[0], 0.0);
    assert_close(processed.intensities[1], 2.0);
    assert_close(processed.intensities[2], 0.0);
    let imaginary = require_imaginary(&processed)?;
    assert_close(imaginary[0], 0.0);
    assert_close(imaginary[1], 1.0);
    assert_close(imaginary[2], 0.0);
    assert_eq!(processed.processing[0].operation, "sine_bell_apodization");
    Ok(())
}

#[test]
fn lorentz_to_gauss_identity_when_both_broadenings_zero() -> anyhow::Result<()> {
    let spectrum = complex_spectrum()?;
    let processed = lorentz_to_gauss_apodization(&spectrum, 0.0, 0.0, 0.0, 0.1)?;
    assert_vec_close(&processed.intensities, &spectrum.intensities);
    assert_vec_close(
        require_imaginary(&processed)?,
        require_imaginary(&spectrum)?,
    );
    assert_eq!(
        processed.processing[0].operation,
        "lorentz_to_gauss_apodization"
    );
    Ok(())
}

#[test]
fn lorentz_to_gauss_undoes_exponential_decay() -> anyhow::Result<()> {
    // After multiplying by exp(-pi*lb*t) and then by Lorentz-to-Gauss with
    // the same lb (and no Gaussian), the FID must equal the original.
    let spectrum = complex_spectrum()?;
    let lb = 2.0_f64;
    let dwell = 0.1_f64;
    let decayed = exponential_apodization(&spectrum, lb, dwell)?;
    let restored = lorentz_to_gauss_apodization(&decayed, lb, 0.0, 0.0, dwell)?;
    assert_vec_close(&restored.intensities, &spectrum.intensities);
    assert_vec_close(require_imaginary(&restored)?, require_imaginary(&spectrum)?);
    Ok(())
}

#[test]
fn lorentz_to_gauss_matches_gaussian_when_lorentz_is_zero() -> anyhow::Result<()> {
    let spectrum = complex_spectrum()?;
    let gauss = gaussian_apodization(&spectrum, 1.0, 0.1)?;
    let l2g = lorentz_to_gauss_apodization(&spectrum, 0.0, 1.0, 0.0, 0.1)?;
    assert_vec_close(&l2g.intensities, &gauss.intensities);
    assert_vec_close(require_imaginary(&l2g)?, require_imaginary(&gauss)?);
    Ok(())
}

#[test]
fn lorentz_to_gauss_rejects_invalid_parameters() -> anyhow::Result<()> {
    let spectrum = complex_spectrum()?;
    assert!(lorentz_to_gauss_apodization(&spectrum, -1.0, 1.0, 0.0, 0.1).is_err());
    assert!(lorentz_to_gauss_apodization(&spectrum, 1.0, -1.0, 0.0, 0.1).is_err());
    assert!(lorentz_to_gauss_apodization(&spectrum, 1.0, 1.0, 1.5, 0.1).is_err());
    assert!(lorentz_to_gauss_apodization(&spectrum, 1.0, 1.0, 0.0, 0.0).is_err());
    Ok(())
}

#[test]
fn trapezoidal_identity_when_full_window() -> anyhow::Result<()> {
    let spectrum = complex_spectrum()?;
    let processed = trapezoidal_apodization(&spectrum, 0.0, 1.0)?;
    assert_vec_close(&processed.intensities, &spectrum.intensities);
    assert_vec_close(
        require_imaginary(&processed)?,
        require_imaginary(&spectrum)?,
    );
    assert_eq!(processed.processing[0].operation, "trapezoidal_apodization");
    Ok(())
}

#[test]
fn trapezoidal_ramps_in_and_out_linearly() -> anyhow::Result<()> {
    let axis = Axis::linear("time", Unit::Seconds, 0.0, 0.4, 5)?;
    let spectrum = Spectrum1D::new_complex(
        axis,
        vec![1.0, 1.0, 1.0, 1.0, 1.0],
        Some(vec![1.0, 1.0, 1.0, 1.0, 1.0]),
        Metadata::default(),
    )?;
    // fractions: 0.0, 0.25, 0.5, 0.75, 1.0
    // rise=0.5 → weight at 0.0 → 0, 0.25 → 0.5, 0.5 → 1.0
    // fall=0.5 → weight at 0.75 → 0.5, 1.0 → 0.0
    let processed = trapezoidal_apodization(&spectrum, 0.5, 0.5)?;
    assert_vec_close(&processed.intensities, &[0.0, 0.5, 1.0, 0.5, 0.0]);
    assert_vec_close(require_imaginary(&processed)?, &[0.0, 0.5, 1.0, 0.5, 0.0]);
    Ok(())
}

#[test]
fn trapezoidal_fall_only_keeps_head_at_one() -> anyhow::Result<()> {
    let axis = Axis::linear("time", Unit::Seconds, 0.0, 0.3, 4)?;
    let spectrum = Spectrum1D::new(axis, vec![2.0; 4], Metadata::default())?;
    // fractions: 0, 1/3, 2/3, 1; fall_start=2/3 → weights 1, 1, 1, 0.
    let processed = trapezoidal_apodization(&spectrum, 0.0, 2.0 / 3.0)?;
    assert_vec_close(&processed.intensities, &[2.0, 2.0, 2.0, 0.0]);
    Ok(())
}

#[test]
fn trapezoidal_rejects_invalid_parameters() -> anyhow::Result<()> {
    let spectrum = complex_spectrum()?;
    assert!(trapezoidal_apodization(&spectrum, -0.1, 1.0).is_err());
    assert!(trapezoidal_apodization(&spectrum, 0.0, 1.5).is_err());
    assert!(trapezoidal_apodization(&spectrum, 0.6, 0.4).is_err());
    assert!(trapezoidal_apodization(&spectrum, f64::NAN, 0.5).is_err());
    Ok(())
}

#[test]
fn traf_zero_broadening_gives_uniform_half_weight() -> anyhow::Result<()> {
    let spectrum = complex_spectrum()?;
    let processed = traf_apodization(&spectrum, 0.0, 0.1)?;
    // E = R = 1 → w = 1/2 everywhere.
    assert_vec_close(
        &processed.intensities,
        &spectrum
            .intensities
            .iter()
            .map(|v| v * 0.5)
            .collect::<Vec<_>>(),
    );
    assert_eq!(processed.processing[0].operation, "traf_apodization");
    Ok(())
}

#[test]
fn traf_matches_analytic_formula() -> anyhow::Result<()> {
    let axis = Axis::linear("time", Unit::Seconds, 0.0, 0.3, 4)?;
    let spectrum = Spectrum1D::new(axis, vec![1.0, 1.0, 1.0, 1.0], Metadata::default())?;
    let lb = 2.0_f64;
    let dwell = 0.1_f64;
    let last = 3.0_f64;
    let scale = -PI * lb * dwell;
    let processed = traf_apodization(&spectrum, lb, dwell)?;
    for index in 0..4_u32 {
        let i_f = f64::from(index);
        let e = (scale * i_f).exp();
        let r = (scale * (last - i_f)).exp();
        let expected = e.powi(2) / (e.powi(3) + r.powi(3));
        assert_close(processed.intensities[usize::try_from(index)?], expected);
    }
    Ok(())
}

#[test]
fn traf_rejects_invalid_parameters() -> anyhow::Result<()> {
    let spectrum = complex_spectrum()?;
    assert!(traf_apodization(&spectrum, -1.0, 0.1).is_err());
    assert!(traf_apodization(&spectrum, 1.0, 0.0).is_err());
    assert!(traf_apodization(&spectrum, f64::NAN, 0.1).is_err());
    Ok(())
}

#[test]
fn bruker_gmb_identity_when_both_parameters_zero() -> anyhow::Result<()> {
    let spectrum = complex_spectrum()?;
    let processed = gauss_multiply_bruker_apodization(&spectrum, 0.0, 0.0, 0.1)?;
    assert_vec_close(&processed.intensities, &spectrum.intensities);
    assert_vec_close(
        require_imaginary(&processed)?,
        require_imaginary(&spectrum)?,
    );
    assert_eq!(
        processed.processing[0].operation,
        "gauss_multiply_bruker_apodization"
    );
    Ok(())
}

#[test]
fn bruker_gmb_reduces_to_exponential_when_gauss_zero() -> anyhow::Result<()> {
    let spectrum = complex_spectrum()?;
    let lb = 1.5_f64;
    let dwell = 0.1_f64;
    let bruker = gauss_multiply_bruker_apodization(&spectrum, lb, 0.0, dwell)?;
    let em = exponential_apodization(&spectrum, lb, dwell)?;
    assert_vec_close(&bruker.intensities, &em.intensities);
    assert_vec_close(require_imaginary(&bruker)?, require_imaginary(&em)?);
    Ok(())
}

#[test]
fn bruker_gmb_peaks_near_gb_fraction_for_resolution_enhancement() -> anyhow::Result<()> {
    // LB < 0 with GB > 0 should produce a window that peaks at i = GB*(N-1).
    let axis = Axis::linear("time", Unit::Seconds, 0.0, 0.99, 100)?;
    let spectrum = Spectrum1D::new(axis, vec![1.0; 100], Metadata::default())?;
    let processed = gauss_multiply_bruker_apodization(&spectrum, -1.0, 0.5, 0.01)?;
    let (peak_index, _) = processed.intensities.iter().enumerate().fold(
        (0_usize, f64::NEG_INFINITY),
        |(best_index, best), (index, value)| {
            if *value > best {
                (index, *value)
            } else {
                (best_index, best)
            }
        },
    );
    assert!(
        (47..=52).contains(&peak_index),
        "expected peak near 49, got {peak_index}"
    );
    Ok(())
}

#[test]
fn bruker_gmb_rejects_invalid_parameters() -> anyhow::Result<()> {
    let spectrum = complex_spectrum()?;
    assert!(gauss_multiply_bruker_apodization(&spectrum, 1.0, -0.1, 0.1).is_err());
    assert!(gauss_multiply_bruker_apodization(&spectrum, 1.0, 1.5, 0.1).is_err());
    assert!(gauss_multiply_bruker_apodization(&spectrum, 1.0, 0.5, 0.0).is_err());
    assert!(gauss_multiply_bruker_apodization(&spectrum, f64::NAN, 0.5, 0.1).is_err());
    Ok(())
}

#[test]
fn convolution_difference_identity_when_mixing_is_zero() -> anyhow::Result<()> {
    let spectrum = complex_spectrum()?;
    let processed = convolution_difference_apodization(&spectrum, 0.0, 5.0, 0.0, 0.1)?;
    assert_vec_close(&processed.intensities, &spectrum.intensities);
    assert_vec_close(
        require_imaginary(&processed)?,
        require_imaginary(&spectrum)?,
    );
    assert_eq!(
        processed.processing[0].operation,
        "convolution_difference_apodization"
    );
    Ok(())
}

#[test]
fn convolution_difference_matches_analytic_formula() -> anyhow::Result<()> {
    let axis = Axis::linear("time", Unit::Seconds, 0.0, 0.3, 4)?;
    let spectrum = Spectrum1D::new(axis, vec![1.0, 1.0, 1.0, 1.0], Metadata::default())?;
    let lb_n = 1.0_f64;
    let lb_b = 5.0_f64;
    let dwell = 0.1_f64;
    let mix = 0.4_f64;
    let processed = convolution_difference_apodization(&spectrum, lb_n, lb_b, mix, dwell)?;
    for index in 0..4_u32 {
        let i_f = f64::from(index);
        let expected = (-PI * lb_n * dwell * i_f).exp() - mix * (-PI * lb_b * dwell * i_f).exp();
        assert_close(processed.intensities[usize::try_from(index)?], expected);
    }
    Ok(())
}

#[test]
fn convolution_difference_rejects_invalid_parameters() -> anyhow::Result<()> {
    let spectrum = complex_spectrum()?;
    assert!(convolution_difference_apodization(&spectrum, -1.0, 1.0, 0.5, 0.1).is_err());
    assert!(convolution_difference_apodization(&spectrum, 1.0, -1.0, 0.5, 0.1).is_err());
    assert!(convolution_difference_apodization(&spectrum, 1.0, 1.0, 1.5, 0.1).is_err());
    assert!(convolution_difference_apodization(&spectrum, 1.0, 1.0, -0.1, 0.1).is_err());
    assert!(convolution_difference_apodization(&spectrum, 1.0, 1.0, 0.5, 0.0).is_err());
    Ok(())
}

#[test]
fn sine_bell_convenience_constructors_match_nmrpipe_defaults() {
    let sine_squared = SineBellApodization::sine_squared();
    assert_close(sine_squared.start_angle_deg, 0.0);
    assert_close(sine_squared.end_angle_deg, 180.0);
    assert_close(sine_squared.exponent, 2.0);

    let cosine_bell = SineBellApodization::cosine_bell();
    assert_close(cosine_bell.start_angle_deg, 90.0);
    assert_close(cosine_bell.exponent, 1.0);

    let cosine_squared = SineBellApodization::cosine_squared();
    assert_close(cosine_squared.start_angle_deg, 90.0);
    assert_close(cosine_squared.exponent, 2.0);

    let shifted = SineBellApodization::shifted_sine(0.25, 1.5);
    assert_close(shifted.start_angle_deg, 45.0);
    assert_close(shifted.end_angle_deg, 180.0);
    assert_close(shifted.exponent, 1.5);
}

#[test]
fn matched_filter_em_recovers_lorentzian_fwhm() -> anyhow::Result<()> {
    // Quadrature FID at a single off-centre frequency. The complex
    // (cosine+i·sine) carrier gives a one-sided Lorentzian in the
    // magnitude spectrum whose FWHM in Hz equals the decay constant
    // `lb_true_hz` exactly (the textbook matched-filter case).
    let lb_true_hz = 5.0_f64;
    let n: u32 = 2048;
    let dwell = 1.0e-3_f64;
    let carrier_hz = 80.0_f64;
    let mut times = Vec::with_capacity(usize::try_from(n)?);
    let mut real = Vec::with_capacity(usize::try_from(n)?);
    let mut imag = Vec::with_capacity(usize::try_from(n)?);
    for i in 0..n {
        let t = f64::from(i) * dwell;
        times.push(t);
        let decay = (-PI * lb_true_hz * t).exp();
        let phase = 2.0 * PI * carrier_hz * t;
        real.push(decay * phase.cos());
        imag.push(decay * phase.sin());
    }
    let axis = Axis::new("time", Unit::Seconds, times)?;
    let fid = Spectrum1D::new_complex(axis, real, Some(imag), Metadata::default())?;
    let recommended = matched_filter_em(&fid)?;
    let relative_error = (recommended.line_broadening_hz - lb_true_hz).abs() / lb_true_hz;
    assert!(
        relative_error < 0.2,
        "expected lb ≈ {lb_true_hz}, got {} (rel. error {:.2})",
        recommended.line_broadening_hz,
        relative_error
    );
    assert_close(recommended.dwell_time_s, dwell);
    Ok(())
}

#[test]
fn matched_filter_em_rejects_frequency_domain_input() -> anyhow::Result<()> {
    let axis = Axis::linear("shift", Unit::Ppm, 0.0, 1.0, 16)?;
    let spectrum = Spectrum1D::new(axis, vec![1.0; 16], Metadata::default())?;
    assert!(matched_filter_em(&spectrum).is_err());
    Ok(())
}

#[test]
fn apply_subsample_shift_zero_is_identity() -> anyhow::Result<()> {
    let axis = Axis::linear("freq", Unit::Hertz, -2.0, 2.0, 5)?;
    let spectrum = Spectrum1D::new_complex(
        axis,
        vec![1.0, 2.0, 3.0, 4.0, 5.0],
        Some(vec![0.5, 1.5, 2.5, 3.5, 4.5]),
        Metadata::default(),
    )?;
    let shifted = apply_subsample_shift(&spectrum, 0.0)?;
    assert_vec_close(&shifted.intensities, &spectrum.intensities);
    assert_vec_close(require_imaginary(&shifted)?, require_imaginary(&spectrum)?);
    assert_eq!(
        shifted.processing.last().map(|r| r.operation.as_str()),
        Some("apply_subsample_shift")
    );
    Ok(())
}

#[test]
fn apply_subsample_shift_round_trip_via_fft() -> anyhow::Result<()> {
    // Build a small complex FID, FFT, apply +0.4 sub-sample shift,
    // apply -0.4 sub-sample shift, inverse FFT — should recover the
    // original FID to floating-point precision.
    let n: u32 = 16;
    let dwell = 0.01_f64;
    let mut times = Vec::with_capacity(usize::try_from(n)?);
    let mut real = Vec::with_capacity(usize::try_from(n)?);
    let mut imag = Vec::with_capacity(usize::try_from(n)?);
    for i in 0..n {
        let t = f64::from(i) * dwell;
        times.push(t);
        real.push((2.0 * PI * 10.0 * t).cos() * (-PI * 2.0 * t).exp());
        imag.push((2.0 * PI * 10.0 * t).sin() * (-PI * 2.0 * t).exp());
    }
    let axis = Axis::new("time", Unit::Seconds, times)?;
    let fid = Spectrum1D::new_complex(axis, real.clone(), Some(imag.clone()), Metadata::default())?;
    let forward = fft_1d(&fid, FftDirection::Forward)?;
    let plus = apply_subsample_shift(&forward, 0.4)?;
    let restored = apply_subsample_shift(&plus, -0.4)?;
    let inverse = fft_1d(&restored, FftDirection::Inverse)?;
    assert_vec_close(&inverse.intensities, &real);
    assert_vec_close(require_imaginary(&inverse)?, &imag);
    Ok(())
}

#[test]
fn apply_subsample_shift_rejects_invalid_input() -> anyhow::Result<()> {
    let freq_axis = Axis::linear("freq", Unit::Hertz, -1.0, 1.0, 3)?;
    let real_only = Spectrum1D::new(freq_axis.clone(), vec![1.0, 2.0, 3.0], Metadata::default())?;
    assert!(apply_subsample_shift(&real_only, 0.3).is_err());

    let time_axis = Axis::linear("time", Unit::Seconds, 0.0, 0.2, 3)?;
    let time_domain = Spectrum1D::new_complex(
        time_axis,
        vec![1.0, 2.0, 3.0],
        Some(vec![0.0; 3]),
        Metadata::default(),
    )?;
    assert!(apply_subsample_shift(&time_domain, 0.3).is_err());

    let valid = Spectrum1D::new_complex(
        freq_axis,
        vec![1.0, 2.0, 3.0],
        Some(vec![0.0; 3]),
        Metadata::default(),
    )?;
    assert!(apply_subsample_shift(&valid, f64::NAN).is_err());
    Ok(())
}

#[test]
fn first_point_scale_halves_only_the_first_sample() -> anyhow::Result<()> {
    let axis = Axis::linear("time", Unit::Seconds, 0.0, 0.3, 4)?;
    let spectrum = Spectrum1D::new_complex(
        axis,
        vec![2.0, 4.0, 6.0, 8.0],
        Some(vec![1.0, 3.0, 5.0, 7.0]),
        Metadata::default(),
    )?;
    let processed = first_point_scale(&spectrum, 0.5)?;
    assert_vec_close(&processed.intensities, &[1.0, 4.0, 6.0, 8.0]);
    assert_vec_close(require_imaginary(&processed)?, &[0.5, 3.0, 5.0, 7.0]);
    assert_eq!(processed.processing[0].operation, "first_point_scale");
    Ok(())
}

#[test]
fn first_point_scale_with_unit_scale_is_identity() -> anyhow::Result<()> {
    let spectrum = complex_spectrum()?;
    let processed = first_point_scale(&spectrum, 1.0)?;
    assert_vec_close(&processed.intensities, &spectrum.intensities);
    assert_vec_close(
        require_imaginary(&processed)?,
        require_imaginary(&spectrum)?,
    );
    Ok(())
}

#[test]
fn first_point_scale_rejects_invalid_scale() -> anyhow::Result<()> {
    let spectrum = complex_spectrum()?;
    assert!(first_point_scale(&spectrum, 0.0).is_err());
    assert!(first_point_scale(&spectrum, -1.0).is_err());
    assert!(first_point_scale(&spectrum, f64::NAN).is_err());
    Ok(())
}

#[test]
fn magnitude_combines_real_and_imaginary_channels() -> anyhow::Result<()> {
    let spectrum = complex_spectrum()?;
    let processed = Magnitude.apply(&spectrum)?;
    assert_vec_close(
        &processed.intensities,
        &[1.118_033_988_749_895, 2.236_067_977_499_79, 4.0],
    );
    assert!(processed.imaginary.is_none());
    Ok(())
}

#[test]
fn fft_inverse_roundtrip_recovers_complex_data() -> anyhow::Result<()> {
    let spectrum = complex_spectrum()?;
    let transformed = Fft1D {
        direction: FftDirection::Forward,
    }
    .apply(&spectrum)?;
    let recovered = fft_1d(&transformed, FftDirection::Inverse)?;
    assert_vec_close(&recovered.intensities, &spectrum.intensities);
    assert_vec_close(
        require_imaginary(&recovered)?,
        require_imaginary(&spectrum)?,
    );
    Ok(())
}

#[test]
fn applies_zero_order_phase_correction() -> anyhow::Result<()> {
    let spectrum = Spectrum1D::new_complex(
        Axis::linear("shift", Unit::Ppm, 0.0, 1.0, 2)?,
        vec![1.0, 0.0],
        Some(vec![0.0, 1.0]),
        Metadata::default(),
    )?;

    let processed = PhaseCorrection {
        zero_order_deg: 90.0,
        first_order_deg: 0.0,
        pivot_fraction: 0.0,
    }
    .apply(&spectrum)?;

    assert_vec_close(&processed.intensities, &[0.0, -1.0]);
    assert_vec_close(require_imaginary(&processed)?, &[1.0, 0.0]);
    assert_eq!(processed.processing[0].operation, "phase_correct");
    Ok(())
}

#[test]
fn applies_first_order_phase_around_pivot() -> anyhow::Result<()> {
    let spectrum = Spectrum1D::new_complex(
        Axis::linear("shift", Unit::Ppm, 0.0, 2.0, 3)?,
        vec![1.0, 1.0, 1.0],
        Some(vec![0.0, 0.0, 0.0]),
        Metadata::default(),
    )?;

    let processed = phase_correct(&spectrum, 0.0, 180.0, 0.5)?;

    assert_vec_close(&processed.intensities, &[0.0, 1.0, 0.0]);
    assert_vec_close(require_imaginary(&processed)?, &[-1.0, 0.0, 1.0]);
    Ok(())
}

#[test]
fn rejects_invalid_phase_pivot() -> anyhow::Result<()> {
    let spectrum = complex_spectrum()?;
    let error = phase_correct(&spectrum, 0.0, 0.0, 1.5).expect_err("invalid pivot should fail");
    assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));
    Ok(())
}

#[test]
fn rejects_negative_line_broadening() -> anyhow::Result<()> {
    let spectrum = complex_spectrum()?;
    let error = exponential_apodization(&spectrum, -1.0, 0.1)
        .expect_err("negative line broadening should fail");
    assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));
    Ok(())
}

#[test]
fn rejects_negative_gaussian_broadening() -> anyhow::Result<()> {
    let spectrum = complex_spectrum()?;
    let error = GaussianApodization {
        gaussian_broadening_hz: -1.0,
        dwell_time_s: 0.1,
    }
    .apply(&spectrum)
    .expect_err("negative Gaussian broadening should fail");
    assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));
    Ok(())
}

#[test]
fn rejects_invalid_sine_bell_parameters() -> anyhow::Result<()> {
    let spectrum = complex_spectrum()?;
    let angle_error = sine_bell_apodization(&spectrum, -1.0, 180.0, 1.0)
        .expect_err("negative sine-bell angle should fail");
    assert!(matches!(angle_error, RSpinError::InvalidSpectrum { .. }));

    let exponent_error = SineBellApodization {
        start_angle_deg: 0.0,
        end_angle_deg: 180.0,
        exponent: 0.0,
    }
    .apply(&spectrum)
    .expect_err("zero sine-bell exponent should fail");
    assert!(matches!(exponent_error, RSpinError::InvalidSpectrum { .. }));
    Ok(())
}

fn complex_spectrum() -> anyhow::Result<Spectrum1D> {
    let axis = Axis::linear("time", Unit::Seconds, 0.0, 0.2, 3)?;
    Ok(Spectrum1D::new_complex(
        axis,
        vec![1.0, 2.0, 4.0],
        Some(vec![0.5, 1.0, 0.0]),
        Metadata::default(),
    )?)
}

fn require_imaginary(spectrum: &Spectrum1D) -> anyhow::Result<&[f64]> {
    match &spectrum.imaginary {
        Some(imaginary) => Ok(imaginary),
        None => anyhow::bail!("missing imaginary channel"),
    }
}

fn assert_vec_close(actual: &[f64], expected: &[f64]) {
    assert_eq!(actual.len(), expected.len());
    for (left, right) in actual.iter().zip(expected) {
        assert_close(*left, *right);
    }
}

fn assert_close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-10, "{actual} != {expected}");
}

#[test]
fn fft_forward_relabels_time_axis_to_hertz() -> anyhow::Result<()> {
    let dwell = 0.001_f64;
    let len = 8;
    let axis_values: Vec<f64> = (0..u32::try_from(len)?)
        .map(|i| f64::from(i) * dwell)
        .collect();
    let axis = Axis::new("time", Unit::Seconds, axis_values)?;
    let spectrum = Spectrum1D::new_complex(
        axis,
        vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Some(vec![0.0; 8]),
        Metadata::default(),
    )?;
    let transformed = fft_1d(&spectrum, FftDirection::Forward)?;
    assert_eq!(transformed.x.unit, Unit::Hertz);
    let sw = 1.0 / dwell;
    let expected_first = -sw / 2.0;
    assert_close(transformed.x.values[0], expected_first);
    let dc_index = len / 2;
    assert_close(transformed.x.values[dc_index], 0.0);
    Ok(())
}

#[test]
fn fft_forward_relabels_to_ppm_when_metadata_has_frequency() -> anyhow::Result<()> {
    let dwell = 0.001_f64;
    let len = 8;
    let axis_values: Vec<f64> = (0..u32::try_from(len)?)
        .map(|i| f64::from(i) * dwell)
        .collect();
    let axis = Axis::new("time", Unit::Seconds, axis_values)?;
    let metadata = Metadata::default().with_frequency_mhz(500.0);
    let spectrum = Spectrum1D::new_complex(
        axis,
        vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Some(vec![0.0; 8]),
        metadata,
    )?;
    let transformed = fft_1d(&spectrum, FftDirection::Forward)?;
    assert_eq!(transformed.x.unit, Unit::Ppm);
    let sw_hz = 1.0 / dwell;
    let expected_first_ppm = -sw_hz / 2.0 / 500.0;
    assert_close(transformed.x.values[0], expected_first_ppm);
    Ok(())
}
