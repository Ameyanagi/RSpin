use rspin_core::{Axis, Metadata, RSpinError, Spectrum1D, Unit};

use super::*;

#[test]
fn subtracts_constant_baseline() -> anyhow::Result<()> {
    let spectrum = spectrum(&[2.0, 3.5, 5.0])?;
    let processed = subtract_baseline(&spectrum, BaselineMethod::Constant { value: 2.0 })?;

    assert_eq!(processed.intensities, vec![0.0, 1.5, 3.0]);
    assert_eq!(processed.processing[0].operation, "baseline_constant");
    assert_eq!(processed.imaginary, None);
    Ok(())
}

#[test]
fn fits_moving_minimum_baseline() -> anyhow::Result<()> {
    let spectrum = spectrum(&[3.0, 2.0, 5.0, 1.0, 4.0])?;
    let fit = fit_baseline(&spectrum, BaselineMethod::MovingMinimum { half_window: 1 })?;

    assert_eq!(fit.baseline, vec![2.0, 2.0, 1.0, 1.0, 1.0]);
    assert_eq!(fit.corrected, vec![1.0, 0.0, 4.0, 0.0, 3.0]);
    assert!(fit.report.converged);
    Ok(())
}

#[test]
fn whittaker_asls_preserves_flat_baseline() -> anyhow::Result<()> {
    let spectrum = spectrum(&[2.0, 2.0, 2.0, 2.0, 2.0])?;
    let fit = fit_baseline(
        &spectrum,
        BaselineMethod::WhittakerAsls {
            lambda: 1.0e3,
            p: 0.01,
            max_iter: 20,
            tolerance: 1.0e-6,
        },
    )?;

    assert!(fit.report.iterations > 0);
    for value in &fit.baseline {
        assert_close(*value, 2.0, 1.0e-9);
    }
    for value in &fit.corrected {
        assert_close(*value, 0.0, 1.0e-9);
    }
    Ok(())
}

#[test]
fn whittaker_asls_estimates_under_positive_peak() -> anyhow::Result<()> {
    let spectrum = spectrum(&[1.0, 1.0, 4.0, 1.0, 1.0])?;
    let processed = SubtractBaseline {
        method: BaselineMethod::WhittakerAsls {
            lambda: 1.0e4,
            p: 0.01,
            max_iter: 25,
            tolerance: 1.0e-6,
        },
    }
    .apply(&spectrum)?;

    assert!(processed.intensities[2] > 2.0);
    assert!(processed.intensities[0].abs() < 0.2);
    assert_eq!(processed.processing[0].operation, "baseline_whittaker_asls");
    Ok(())
}

#[test]
fn rejects_invalid_whittaker_options() -> anyhow::Result<()> {
    let spectrum = spectrum(&[1.0, 2.0, 3.0])?;
    let error = fit_baseline(
        &spectrum,
        BaselineMethod::WhittakerAsls {
            lambda: -1.0,
            p: 0.01,
            max_iter: 50,
            tolerance: 1.0e-3,
        },
    )
    .expect_err("negative lambda should fail");

    assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));
    Ok(())
}

fn spectrum(intensities: &[f64]) -> anyhow::Result<Spectrum1D> {
    let end = f64::from(u32::try_from(intensities.len() - 1)?);
    Ok(Spectrum1D::new(
        Axis::linear("x", Unit::Ppm, 0.0, end, intensities.len())?,
        intensities.to_vec(),
        Metadata::default(),
    )?)
}

fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual} != {expected}"
    );
}
