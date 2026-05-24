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
fn fits_polynomial_baseline_on_sloped_data() -> anyhow::Result<()> {
    let spectrum = spectrum_with_axis(&[0.0, 1.0, 2.0, 3.0], &[1.0, 3.0, 5.0, 7.0])?;
    let fit = fit_baseline(&spectrum, BaselineMethod::Polynomial { degree: 1 })?;
    let processed = subtract_baseline(&spectrum, BaselineMethod::Polynomial { degree: 1 })?;

    for (actual, expected) in fit.baseline.iter().zip([1.0, 3.0, 5.0, 7.0]) {
        assert_close(*actual, expected, 1.0e-12);
    }
    for value in &processed.intensities {
        assert_close(*value, 0.0, 1.0e-12);
    }
    assert_eq!(processed.processing[0].operation, "baseline_polynomial");
    Ok(())
}

#[test]
fn fits_constant_polynomial_baseline() -> anyhow::Result<()> {
    let spectrum = spectrum(&[1.0, 3.0, 5.0])?;
    let fit = fit_baseline(&spectrum, BaselineMethod::Polynomial { degree: 0 })?;

    assert_eq!(fit.baseline, vec![3.0, 3.0, 3.0]);
    assert_eq!(fit.corrected, vec![-2.0, 0.0, 2.0]);
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

#[cfg(feature = "external-baselines")]
#[test]
fn external_baselines_asls_is_feature_gated() -> anyhow::Result<()> {
    let spectrum = spectrum(&[1.0, 1.0, 4.0, 1.0, 1.0])?;
    let method = BaselineMethod::BaselinesAsls {
        lambda: 1.0e4,
        p: 0.01,
        max_iter: 25,
        tolerance: 1.0e-6,
    };
    let fit = fit_baseline(&spectrum, method)?;
    let processed = subtract_baseline(&spectrum, method)?;
    let json = serde_json::to_string(&method)?;

    assert!(fit.baseline[2] < 4.0);
    assert!(processed.intensities[2] > 2.0);
    assert_eq!(processed.processing[0].operation, "baseline_baselines_asls");
    assert!(json.contains("baselines_asls"));
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

#[test]
fn rejects_invalid_polynomial_options() -> anyhow::Result<()> {
    let spectrum = spectrum(&[1.0, 2.0, 3.0])?;
    let too_high = fit_baseline(&spectrum, BaselineMethod::Polynomial { degree: 3 })
        .expect_err("too-high degree should fail");
    assert!(matches!(too_high, RSpinError::InvalidSpectrum { .. }));

    let degenerate = Spectrum1D::new(
        Axis::new("x", Unit::Ppm, vec![1.0, 1.0, 1.0])?,
        vec![1.0, 2.0, 3.0],
        Metadata::default(),
    )?;
    let degenerate_error = fit_baseline(&degenerate, BaselineMethod::Polynomial { degree: 1 })
        .expect_err("degenerate x axis should fail");
    assert!(matches!(
        degenerate_error,
        RSpinError::InvalidSpectrum { .. }
    ));
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

fn spectrum_with_axis(x_values: &[f64], intensities: &[f64]) -> anyhow::Result<Spectrum1D> {
    Ok(Spectrum1D::new(
        Axis::new("x", Unit::Ppm, x_values.to_vec())?,
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
