use std::f64::consts::PI;

use rspin_core::{Axis, Metadata, Nucleus, RSpinError, Spectrum1D, Unit};

use super::*;

fn synthetic_complex_fid(
    n: u32,
    dwell: f64,
    peaks: &[(f64, f64, f64)],
) -> anyhow::Result<Spectrum1D> {
    let mut times = Vec::with_capacity(usize::try_from(n)?);
    let mut real = Vec::with_capacity(usize::try_from(n)?);
    let mut imag = Vec::with_capacity(usize::try_from(n)?);
    for i in 0..n {
        let t = f64::from(i) * dwell;
        times.push(t);
        let mut re = 0.0;
        let mut im = 0.0;
        for (freq_hz, amplitude, lb_hz) in peaks {
            let envelope = amplitude * (-PI * lb_hz * t).exp();
            let phase = 2.0 * PI * freq_hz * t;
            re += envelope * phase.cos();
            im += envelope * phase.sin();
        }
        real.push(re);
        imag.push(im);
    }
    let axis = Axis::new("time", Unit::Seconds, times)?;
    Ok(Spectrum1D::new_complex(
        axis,
        real,
        Some(imag),
        Metadata::default().with_nucleus(Nucleus::Hydrogen1),
    )?)
}

#[test]
fn process_spectrum_auto_emits_frequency_domain_phased_spectrum() -> anyhow::Result<()> {
    // Three-peak synthetic FID at +40 / +0 / -30 Hz with Lorentzian
    // linewidths of 4 Hz. The pipeline must produce a frequency-domain
    // spectrum with sharp positive peaks at those frequencies.
    let fid = synthetic_complex_fid(128, 2.0e-3, &[(40.0, 1.0, 4.0), (-30.0, 1.0, 4.0)])?;
    let processed = process_spectrum_auto(&fid, &AutoProcessingOptions::default())?;
    assert!(matches!(processed.x.unit, Unit::Hertz | Unit::Ppm));
    // Processing history records every step.
    let ops: Vec<&str> = processed
        .processing
        .iter()
        .map(|r| r.operation.as_str())
        .collect();
    assert!(ops.iter().any(|op| op == &"exponential_apodization"));
    assert!(ops.iter().any(|op| op == &"first_point_scale"));
    assert!(ops.iter().any(|op| op == &"fft_1d"));
    assert!(
        ops.iter()
            .any(|op| op == &"auto_phase_correct" || op == &"auto_phase_correct_regions")
    );
    Ok(())
}

#[test]
fn process_spectrum_auto_rejects_frequency_domain_input() -> anyhow::Result<()> {
    let freq_axis = Axis::linear("freq", Unit::Hertz, -1.0, 1.0, 64)?;
    let spectrum = Spectrum1D::new_complex(
        freq_axis,
        vec![1.0; 64],
        Some(vec![0.0; 64]),
        Metadata::default(),
    )?;
    let err = process_spectrum_auto(&spectrum, &AutoProcessingOptions::default())
        .expect_err("frequency-domain input should fail");
    assert!(matches!(err, RSpinError::InvalidSpectrum { .. }));
    Ok(())
}

#[test]
fn process_spectrum_auto_rejects_invalid_group_delay_override() -> anyhow::Result<()> {
    let fid = synthetic_complex_fid(128, 2.0e-3, &[(40.0, 1.0, 4.0)])?;
    for value in [f64::NAN, f64::INFINITY, -0.25] {
        let options = AutoProcessingOptions {
            group_delay_samples: Some(value),
            ..AutoProcessingOptions::default()
        };
        let err = process_spectrum_auto(&fid, &options)
            .expect_err("invalid group-delay override should fail");
        match err {
            RSpinError::InvalidSpectrum { .. } | RSpinError::NonFinite { .. } => {}
            other => return Err(anyhow::anyhow!("unexpected error for {value}: {other}")),
        }
    }
    Ok(())
}

#[test]
fn process_spectrum_auto_can_disable_baseline_and_phase() -> anyhow::Result<()> {
    let fid = synthetic_complex_fid(256, 1.0e-3, &[(20.0, 1.0, 3.0)])?;
    let opts = AutoProcessingOptions {
        auto_phase: false,
        subtract_baseline: false,
        backward_lp_n_repair: 0,
        ..AutoProcessingOptions::default()
    };
    let processed = process_spectrum_auto(&fid, &opts)?;
    let operations: Vec<&str> = processed
        .processing
        .iter()
        .map(|record| record.operation.as_str())
        .collect();
    assert!(
        !operations
            .iter()
            .any(|op| op == &"auto_phase_correct" || op == &"auto_phase_correct_regions")
    );
    assert!(!operations.iter().any(|op| op == &"subtract_baseline"));
    assert!(!operations.iter().any(|op| op == &"linear_predict_backward"));
    Ok(())
}

#[test]
fn process_spectrum_auto_can_apply_opt_in_dc_correction() -> anyhow::Result<()> {
    let fid = synthetic_complex_fid(128, 1.0e-3, &[(20.0, 1.0, 3.0)])?;
    let opts = AutoProcessingOptions {
        dc_offset_method: Some(DcOffsetMethod::Explicit {
            real_offset: 0.1,
            imaginary_offset: Some(-0.2),
        }),
        auto_phase: false,
        subtract_baseline: false,
        zero_fill_multiplier: 1,
        ..AutoProcessingOptions::default()
    };
    let processed = process_spectrum_auto(&fid, &opts)?;
    let operations = processed
        .processing
        .iter()
        .map(|record| record.operation.as_str())
        .collect::<Vec<_>>();

    assert_eq!(operations[0], "correct_dc_offset");
    assert!(operations.iter().any(|op| op == &"fft_1d"));
    Ok(())
}

#[test]
fn process_spectrum_auto_nucleus_lookup_picks_correct_lb() -> anyhow::Result<()> {
    // 13C nucleus should pull LB = 1.0 Hz from the default look-up.
    let mut fid = synthetic_complex_fid(256, 1.0e-3, &[(15.0, 1.0, 5.0)])?;
    fid.metadata.nucleus = Some(Nucleus::Carbon13);
    let processed = process_spectrum_auto(&fid, &AutoProcessingOptions::default())?;
    let apod = processed
        .processing
        .iter()
        .find(|r| r.operation == "exponential_apodization")
        .ok_or_else(|| anyhow::anyhow!("missing apodization record"))?;
    let details = apod
        .details
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("apodization record has no details"))?;
    assert!(details.contains("line_broadening_hz=1"));
    Ok(())
}

#[test]
fn group_delay_sweep_returns_one_of_the_candidates() -> anyhow::Result<()> {
    // Sanity check: with a synthetic FID where the cascade prediction
    // is zero (no vendor metadata), the sweep tries `delta_samples`
    // candidates around 0 and returns a successful processed spectrum.
    let fid = synthetic_complex_fid(256, 1.0e-3, &[(30.0, 1.0, 5.0)])?;
    let options = AutoProcessingOptions {
        subtract_baseline: false,
        auto_group_delay_sweep: Some(GroupDelaySweepOptions {
            delta_samples: 1.0,
            step_samples: 0.5,
        }),
        ..AutoProcessingOptions::default()
    };
    let processed = process_spectrum_auto(&fid, &options)?;
    assert_eq!(processed.x.unit, Unit::Hertz);
    assert!(!processed.intensities.is_empty());
    Ok(())
}

#[test]
fn group_delay_sweep_scores_before_auto_phase() -> anyhow::Result<()> {
    let mut fid = synthetic_complex_fid(
        256,
        1.0e-3,
        &[(30.0, 1.0, 2.0), (-85.0, 0.7, 3.0), (120.0, 0.4, 4.0)],
    )?;
    fid.intensities.copy_within(0..255, 1);
    fid.intensities[0] = 0.0;
    if let Some(imaginary) = fid.imaginary.as_mut() {
        imaginary.copy_within(0..255, 1);
        imaginary[0] = 0.0;
    }

    let options = AutoProcessingOptions {
        group_delay_samples: Some(1.0),
        auto_group_delay_sweep: Some(GroupDelaySweepOptions {
            delta_samples: 1.0,
            step_samples: 1.0,
        }),
        first_point_scale: false,
        zero_fill_multiplier: 1,
        subtract_baseline: false,
        ..AutoProcessingOptions::default()
    };
    let processed = process_spectrum_auto(&fid, &options)?;
    let group_delay_record = processed
        .processing
        .iter()
        .find(|record| record.operation == "remove_group_delay")
        .ok_or_else(|| anyhow::anyhow!("sweep did not select the one-sample correction"))?;
    assert_eq!(
        group_delay_record.details.as_deref(),
        Some("samples=1"),
        "sweep should select the actual one-sample delay before auto-phase"
    );
    Ok(())
}
