use rspin_core::{Nucleus, RSpinError, Result};

use super::*;
use crate::{
    PredictedCorrelation2D, PredictedSignal1D, PredictionProvenance, PredictionSpectrum2DOptions,
    PredictionSpectrumOptions,
};

#[test]
fn borrowed_1d_workflow_renders_prediction() -> anyhow::Result<()> {
    let prediction = prediction_set();
    let spectrum = prediction
        .render_1d()
        .with_experiment(Experiment::Proton1D)
        .with_nucleus(Nucleus::Hydrogen1)
        .with_ppm_range(0.99, 1.01)
        .with_points(3)
        .with_spectrometer_mhz(500.0)
        .with_line_width_hz(2.0)
        .with_line_shape(PredictionLineShape::PseudoVoigt)
        .with_area_scale(2.0)
        .run()?;

    assert_eq!(spectrum.len(), 3);
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(spectrum.metadata.frequency_mhz, Some(500.0));
    assert!(spectrum.intensities[1] > spectrum.intensities[0]);
    Ok(())
}

#[test]
fn result_1d_workflow_renders_prediction() -> anyhow::Result<()> {
    let prediction: Result<PredictionSet> = Ok(prediction_set());
    let spectrum = prediction
        .render_1d()
        .with_options(
            PredictionSpectrumOptions::new()
                .with_experiment(Experiment::Proton1D)
                .with_nucleus(Nucleus::Hydrogen1)
                .with_ppm_range(0.99, 1.01)
                .with_points(3),
        )
        .run()?;

    assert_eq!(spectrum.len(), 3);
    assert!(spectrum.intensities[1] > 0.0);
    Ok(())
}

#[test]
fn result_1d_workflow_preserves_initial_error() {
    let prediction: Result<PredictionSet> = Err(parse_error());
    let error = prediction
        .render_1d()
        .with_points(0)
        .run()
        .expect_err("initial prediction error should be returned first");

    assert!(matches!(
        error,
        RSpinError::Parse {
            format: "prediction",
            ..
        }
    ));
}

#[test]
fn borrowed_2d_workflow_renders_prediction() -> anyhow::Result<()> {
    let prediction = prediction_set();
    let spectrum = prediction
        .render_2d()
        .with_experiment(Experiment::Hsqc)
        .with_x_nucleus(Nucleus::Hydrogen1)
        .with_y_nucleus(Nucleus::Carbon13)
        .with_x_axis(0.99, 1.01, 3)
        .with_y_axis(19.9, 20.1, 3)
        .with_x_spectrometer_mhz(500.0)
        .with_y_spectrometer_mhz(125.0)
        .with_x_line_width_hz(2.0)
        .with_y_line_width_hz(4.0)
        .with_line_shape(PredictionLineShape::PseudoVoigt)
        .with_volume_scale(2.0)
        .run()?;

    assert_eq!(spectrum.shape(), (3, 3));
    assert!(spectrum.value_at(1, 1).is_some_and(|value| value > 0.0));
    Ok(())
}

#[test]
fn result_2d_workflow_renders_prediction() -> anyhow::Result<()> {
    let prediction: Result<PredictionSet> = Ok(prediction_set());
    let spectrum = prediction
        .render_2d()
        .with_options(
            PredictionSpectrum2DOptions::new()
                .with_experiment(Experiment::Hsqc)
                .with_x_nucleus(Nucleus::Hydrogen1)
                .with_y_nucleus(Nucleus::Carbon13)
                .with_x_axis(0.99, 1.01, 3)
                .with_y_axis(19.9, 20.1, 3),
        )
        .run()?;

    assert_eq!(spectrum.shape(), (3, 3));
    assert!(spectrum.value_at(1, 1).is_some_and(|value| value > 0.0));
    Ok(())
}

#[test]
fn result_2d_workflow_preserves_initial_error() {
    let prediction: Result<PredictionSet> = Err(parse_error());
    let error = prediction
        .render_2d()
        .with_x_axis(0.0, 0.0, 0)
        .run()
        .expect_err("initial prediction error should be returned first");

    assert!(matches!(
        error,
        RSpinError::Parse {
            format: "prediction",
            ..
        }
    ));
}

fn prediction_set() -> PredictionSet {
    PredictionSet::new()
        .with_name("demo")
        .with_signal_1d(
            PredictedSignal1D::new(Experiment::Proton1D, Nucleus::Hydrogen1, 1.0)
                .with_intensity(1.5)
                .with_confidence(0.8)
                .with_assignment("H1"),
        )
        .with_correlation_2d(
            PredictedCorrelation2D::new(
                Experiment::Hsqc,
                Nucleus::Hydrogen1,
                Nucleus::Carbon13,
                1.0,
                20.0,
            )
            .with_intensity(1.5)
            .with_assignment("H1-C1"),
        )
        .with_provenance(PredictionProvenance::new("static").with_version("1"))
}

fn parse_error() -> RSpinError {
    RSpinError::Parse {
        format: "prediction",
        message: "synthetic failure".to_owned(),
    }
}
