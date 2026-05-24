use rspin_core::{Nucleus, RSpinError};
use rspin_prediction::{
    Experiment, PredictedCorrelation2D, PredictedSignal1D, PredictionProvenance, PredictionSet,
};

use crate::{SpectrumReader, SpectrumWriter};

use super::*;

#[test]
fn round_trips_prediction_json() -> anyhow::Result<()> {
    let prediction = prediction_fixture();
    let text = write_prediction_json(&prediction)?;
    let parsed = read_prediction_json(&text)?;

    assert!(text.contains(&format!("\"format\":\"{PREDICTION_JSON_FORMAT}\"")));
    assert!(text.contains(&format!("\"version\":{PREDICTION_JSON_VERSION}")));
    assert!(text.contains("\"prediction\""));
    assert_eq!(parsed, prediction);
    Ok(())
}

#[test]
fn reads_legacy_raw_prediction_json() -> anyhow::Result<()> {
    let prediction = prediction_fixture();
    let text = serde_json::to_string(&prediction)?;
    let parsed = read_prediction_json(&text)?;

    assert_eq!(parsed, prediction);
    Ok(())
}

#[test]
fn rejects_wrong_prediction_json_header() {
    let wrong_format = read_prediction_json(
        r#"{"format":"rspin.spectrum_1d","version":1,"prediction":{"name":null,"signals_1d":[],"correlations_2d":[],"provenance":null}}"#,
    )
    .expect_err("wrong prediction format should fail");
    assert!(matches!(wrong_format, RSpinError::Parse { .. }));

    let unsupported_version = read_prediction_json(
        r#"{"format":"rspin.prediction","version":2,"prediction":{"name":null,"signals_1d":[],"correlations_2d":[],"provenance":null}}"#,
    )
    .expect_err("unsupported prediction JSON version should fail");
    assert!(matches!(
        unsupported_version,
        RSpinError::Unsupported {
            feature: "prediction JSON version"
        }
    ));
}

#[test]
fn prediction_json_codec_implements_traits() -> anyhow::Result<()> {
    let prediction = prediction_fixture();
    let text = <JsonPrediction as SpectrumWriter<PredictionSet>>::write_string(
        &JsonPrediction,
        &prediction,
    )?;
    let parsed: PredictionSet = SpectrumReader::read_str(&JsonPrediction, &text)?;

    assert_eq!(format!("{JsonPrediction:?}"), "JsonPrediction");
    assert_eq!(parsed, prediction);
    Ok(())
}

fn prediction_fixture() -> PredictionSet {
    PredictionSet::new()
        .with_name("demo")
        .with_signal_1d(
            PredictedSignal1D::new(Experiment::Proton1D, Nucleus::Hydrogen1, 1.25)
                .with_intensity(2.0)
                .with_confidence(0.8)
                .with_assignment("H1"),
        )
        .with_correlation_2d(
            PredictedCorrelation2D::new(
                Experiment::Hsqc,
                Nucleus::Hydrogen1,
                Nucleus::Carbon13,
                1.25,
                63.0,
            )
            .with_assignment("H1-C1"),
        )
        .with_provenance(PredictionProvenance::new("fixture").with_version("1"))
}
