use rspin_core::{Nucleus, RSpinError};

use super::*;

#[test]
fn validates_prediction_payload() -> anyhow::Result<()> {
    let prediction = demo_prediction();
    prediction.validate()?;
    Ok(())
}

#[test]
fn rejects_invalid_confidence() {
    let mut prediction = demo_prediction();
    prediction.signals_1d[0].confidence = Some(1.2);
    let error = prediction
        .validate()
        .expect_err("invalid confidence should fail");
    assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));
}

#[test]
fn builders_create_prediction_payloads() -> anyhow::Result<()> {
    let prediction = PredictionSet::new()
        .with_name("ethanol")
        .with_signal_1d(
            PredictedSignal1D::new(Experiment::Proton1D, Nucleus::Hydrogen1, 1.2)
                .with_intensity(3.0)
                .with_confidence(0.8)
                .with_assignment("H1"),
        )
        .with_correlation_2d(
            PredictedCorrelation2D::new(
                Experiment::Hsqc,
                Nucleus::Hydrogen1,
                Nucleus::Carbon13,
                1.2,
                18.0,
            )
            .with_assignment("H1-C1"),
        )
        .with_provenance(PredictionProvenance::new("static-fixture").with_version("1"));

    prediction.validate()?;
    assert_eq!(prediction.name, Some("ethanol".to_owned()));
    assert_eq!(prediction.signals_1d[0].assignments, vec!["H1".to_owned()]);
    assert_eq!(
        prediction
            .provenance
            .as_ref()
            .and_then(|item| item.version.as_deref()),
        Some("1")
    );

    let predictor = StaticPrediction::new(prediction);
    let predicted = predictor.predict(&"CCO")?;
    assert_eq!(predicted.signals_1d.len(), 1);
    assert_eq!(predicted.correlations_2d.len(), 1);
    Ok(())
}

#[test]
fn static_predictor_returns_validated_payload() -> anyhow::Result<()> {
    let predictor = StaticPrediction {
        prediction: demo_prediction(),
    };
    let prediction = predictor.predict(&"CCO")?;
    assert_eq!(prediction.signals_1d.len(), 1);
    assert_eq!(prediction.correlations_2d.len(), 1);
    Ok(())
}

fn demo_prediction() -> PredictionSet {
    PredictionSet {
        name: Some("ethanol".to_owned()),
        signals_1d: vec![PredictedSignal1D {
            experiment: Experiment::Proton1D,
            nucleus: Nucleus::Hydrogen1,
            delta_ppm: 1.2,
            intensity: 3.0,
            confidence: Some(0.8),
            assignments: vec!["H1".to_owned()],
        }],
        correlations_2d: vec![PredictedCorrelation2D {
            experiment: Experiment::Hsqc,
            x_nucleus: Nucleus::Hydrogen1,
            y_nucleus: Nucleus::Carbon13,
            x_ppm: 1.2,
            y_ppm: 18.0,
            intensity: 1.0,
            confidence: None,
            assignments: vec!["H1-C1".to_owned()],
        }],
        provenance: Some(PredictionProvenance {
            source: "static-fixture".to_owned(),
            version: None,
        }),
    }
}
