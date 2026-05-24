use rspin_core::Nucleus;

use super::*;
use crate::{PredictedCorrelation2D, PredictionProvenance};

#[test]
fn renders_prediction_signals_to_spectrum() -> anyhow::Result<()> {
    let prediction = prediction_set();
    let spectrum = render_prediction_1d(
        &prediction,
        &PredictionSpectrumOptions {
            experiment: Some(Experiment::Proton1D),
            nucleus: Some(Nucleus::Hydrogen1),
            from_ppm: 0.99,
            to_ppm: 1.01,
            points: 3,
            spectrometer_mhz: 400.0,
            line_width_hz: 2.0,
            line_shape: PredictionLineShape::Lorentzian,
            area_scale: 2.0,
        },
    )?;

    assert_eq!(spectrum.len(), 3);
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(spectrum.metadata.frequency_mhz, Some(400.0));
    assert_eq!(spectrum.metadata.origin, Some("static".to_owned()));
    assert_eq!(spectrum.processing.len(), 1);
    assert!(spectrum.intensities[1] > spectrum.intensities[0]);
    assert!(spectrum.intensities[1] > spectrum.intensities[2]);
    Ok(())
}

#[test]
fn filters_by_experiment_and_renders_zero_for_empty_selection() -> anyhow::Result<()> {
    let spectrum = render_prediction_1d(
        &prediction_set(),
        &PredictionSpectrumOptions {
            experiment: Some(Experiment::Carbon13_1D),
            nucleus: Some(Nucleus::Hydrogen1),
            points: 4,
            ..PredictionSpectrumOptions::default()
        },
    )?;

    assert_eq!(spectrum.len(), 4);
    assert!(
        spectrum
            .intensities
            .iter()
            .all(|value| value.abs() <= f64::EPSILON)
    );
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Hydrogen1));
    Ok(())
}

#[test]
fn supports_gaussian_line_shape() -> anyhow::Result<()> {
    let spectrum = render_prediction_1d(
        &prediction_set(),
        &PredictionSpectrumOptions {
            experiment: Some(Experiment::Proton1D),
            nucleus: Some(Nucleus::Hydrogen1),
            from_ppm: 1.0,
            to_ppm: 1.0 + 1.0e-6,
            points: 2,
            line_shape: PredictionLineShape::Gaussian,
            ..PredictionSpectrumOptions::default()
        },
    )?;

    assert!(spectrum.intensities.iter().any(|value| *value > 0.0));
    Ok(())
}

#[test]
fn rejects_invalid_options() {
    let error = render_prediction_1d(
        &prediction_set(),
        &PredictionSpectrumOptions {
            points: 0,
            ..PredictionSpectrumOptions::default()
        },
    )
    .expect_err("zero points should fail");
    assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));

    let error = render_prediction_1d(
        &prediction_set(),
        &PredictionSpectrumOptions {
            line_width_hz: 0.0,
            ..PredictionSpectrumOptions::default()
        },
    )
    .expect_err("zero line width should fail");
    assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));
}

fn prediction_set() -> PredictionSet {
    PredictionSet {
        name: Some("demo".to_owned()),
        signals_1d: vec![
            PredictedSignal1D {
                experiment: Experiment::Proton1D,
                nucleus: Nucleus::Hydrogen1,
                delta_ppm: 1.0,
                intensity: 1.5,
                confidence: Some(0.8),
                assignments: vec!["H1".to_owned()],
            },
            PredictedSignal1D {
                experiment: Experiment::Carbon13_1D,
                nucleus: Nucleus::Carbon13,
                delta_ppm: 20.0,
                intensity: 1.0,
                confidence: None,
                assignments: vec!["C1".to_owned()],
            },
        ],
        correlations_2d: vec![PredictedCorrelation2D {
            experiment: Experiment::Hsqc,
            x_nucleus: Nucleus::Hydrogen1,
            y_nucleus: Nucleus::Carbon13,
            x_ppm: 1.0,
            y_ppm: 20.0,
            intensity: 1.0,
            confidence: None,
            assignments: vec!["H1-C1".to_owned()],
        }],
        provenance: Some(PredictionProvenance {
            source: "static".to_owned(),
            version: Some("1".to_owned()),
        }),
    }
}
