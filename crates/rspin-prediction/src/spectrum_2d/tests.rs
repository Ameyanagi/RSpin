use rspin_core::Nucleus;

use super::*;
use crate::{PredictedSignal1D, PredictionProvenance};

#[test]
fn renders_prediction_correlations_to_2d_spectrum() -> anyhow::Result<()> {
    let spectrum = render_prediction_2d(
        &prediction_set(),
        &PredictionSpectrum2DOptions {
            experiment: Some(Experiment::Hsqc),
            x_nucleus: Some(Nucleus::Hydrogen1),
            y_nucleus: Some(Nucleus::Carbon13),
            x_from_ppm: 0.99,
            x_to_ppm: 1.01,
            x_points: 3,
            y_from_ppm: 19.9,
            y_to_ppm: 20.1,
            y_points: 3,
            x_spectrometer_mhz: 400.0,
            y_spectrometer_mhz: 100.0,
            x_line_width_hz: 2.0,
            y_line_width_hz: 4.0,
            line_shape: PredictionLineShape::Lorentzian,
            volume_scale: 2.0,
        },
    )?;

    assert_eq!(spectrum.shape(), (3, 3));
    assert_eq!(spectrum.metadata.origin, Some("static".to_owned()));
    assert_eq!(spectrum.metadata.nucleus, None);
    assert_eq!(spectrum.metadata.frequency_mhz, None);
    assert_eq!(spectrum.processing.len(), 1);
    assert_eq!(spectrum.processing[0].operation, "render_prediction_2d");
    let Some(center) = spectrum.value_at(1, 1) else {
        panic!("center point should exist");
    };
    let Some(lower_corner) = spectrum.value_at(0, 0) else {
        panic!("lower corner should exist");
    };
    let Some(upper_corner) = spectrum.value_at(2, 2) else {
        panic!("upper corner should exist");
    };
    assert!(center > lower_corner);
    assert!(center > upper_corner);
    Ok(())
}

#[test]
fn builder_options_render_2d_prediction_spectrum() -> anyhow::Result<()> {
    let spectrum = render_prediction_2d(
        &prediction_set(),
        &PredictionSpectrum2DOptions::new()
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
            .with_volume_scale(2.0),
    )?;

    assert_eq!(spectrum.shape(), (3, 3));
    assert!(spectrum.value_at(1, 1).is_some_and(|value| value > 0.0));
    Ok(())
}

#[test]
fn filters_2d_correlations_and_renders_zero_for_empty_selection() -> anyhow::Result<()> {
    let spectrum = render_prediction_2d(
        &prediction_set(),
        &PredictionSpectrum2DOptions {
            experiment: Some(Experiment::Cosy),
            x_points: 2,
            y_points: 2,
            ..PredictionSpectrum2DOptions::default()
        },
    )?;

    assert_eq!(spectrum.shape(), (2, 2));
    assert!(spectrum.z.iter().all(|value| value.abs() <= f64::EPSILON));
    Ok(())
}

#[test]
fn infers_common_metadata_for_homonuclear_2d_prediction() -> anyhow::Result<()> {
    let prediction =
        PredictionSet::new()
            .with_name("cosy")
            .with_correlation_2d(PredictedCorrelation2D::new(
                Experiment::Cosy,
                Nucleus::Hydrogen1,
                Nucleus::Hydrogen1,
                1.0,
                1.2,
            ));

    let spectrum = render_prediction_2d(
        &prediction,
        &PredictionSpectrum2DOptions::new()
            .with_experiment(Experiment::Cosy)
            .with_x_nucleus(Nucleus::Hydrogen1)
            .with_y_nucleus(Nucleus::Hydrogen1)
            .with_x_axis(0.9, 1.1, 3)
            .with_y_axis(1.1, 1.3, 3)
            .with_y_spectrometer_mhz(400.0),
    )?;

    assert_eq!(
        spectrum.metadata.name.as_deref(),
        Some("cosy predicted 2D spectrum")
    );
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(spectrum.metadata.frequency_mhz, Some(400.0));
    Ok(())
}

#[test]
fn rejects_invalid_2d_options() {
    let error = render_prediction_2d(
        &prediction_set(),
        &PredictionSpectrum2DOptions {
            x_points: 0,
            ..PredictionSpectrum2DOptions::default()
        },
    )
    .expect_err("zero x points should fail");
    assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));

    let error = render_prediction_2d(
        &prediction_set(),
        &PredictionSpectrum2DOptions {
            y_line_width_hz: 0.0,
            ..PredictionSpectrum2DOptions::default()
        },
    )
    .expect_err("zero y line width should fail");
    assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));
}

fn prediction_set() -> PredictionSet {
    PredictionSet {
        name: Some("demo".to_owned()),
        signals_1d: vec![PredictedSignal1D {
            experiment: Experiment::Proton1D,
            nucleus: Nucleus::Hydrogen1,
            delta_ppm: 1.0,
            intensity: 1.0,
            confidence: Some(0.8),
            assignments: vec!["H1".to_owned()],
        }],
        correlations_2d: vec![PredictedCorrelation2D {
            experiment: Experiment::Hsqc,
            x_nucleus: Nucleus::Hydrogen1,
            y_nucleus: Nucleus::Carbon13,
            x_ppm: 1.0,
            y_ppm: 20.0,
            intensity: 1.5,
            confidence: None,
            assignments: vec!["H1-C1".to_owned()],
        }],
        provenance: Some(PredictionProvenance {
            source: "static".to_owned(),
            version: Some("1".to_owned()),
        }),
    }
}
