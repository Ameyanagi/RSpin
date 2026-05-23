//! Prediction payload types.

use serde::{Deserialize, Serialize};

use rspin_core::{Nucleus, RSpinError, Result};

use crate::Predictor;

/// Supported experiment labels for prediction payloads.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Experiment {
    /// One-dimensional proton prediction.
    Proton1D,
    /// One-dimensional carbon-13 prediction.
    Carbon13_1D,
    /// Correlation spectroscopy.
    Cosy,
    /// Heteronuclear single quantum coherence.
    Hsqc,
    /// Heteronuclear multiple bond correlation.
    Hmbc,
    /// User-defined experiment label.
    Other(String),
}

/// Provenance for a prediction result.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PredictionProvenance {
    /// Predictor or adapter name.
    pub source: String,
    /// Optional source version.
    pub version: Option<String>,
}

/// A predicted one-dimensional signal.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PredictedSignal1D {
    /// Experiment label.
    pub experiment: Experiment,
    /// Observed nucleus.
    pub nucleus: Nucleus,
    /// Chemical shift in ppm.
    pub delta_ppm: f64,
    /// Relative signal intensity.
    pub intensity: f64,
    /// Confidence score in `[0, 1]`.
    pub confidence: Option<f64>,
    /// Assignment labels or atom identifiers.
    pub assignments: Vec<String>,
}

/// A predicted two-dimensional correlation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PredictedCorrelation2D {
    /// Experiment label.
    pub experiment: Experiment,
    /// X-axis nucleus.
    pub x_nucleus: Nucleus,
    /// Y-axis nucleus.
    pub y_nucleus: Nucleus,
    /// X-axis chemical shift in ppm.
    pub x_ppm: f64,
    /// Y-axis chemical shift in ppm.
    pub y_ppm: f64,
    /// Relative correlation intensity.
    pub intensity: f64,
    /// Confidence score in `[0, 1]`.
    pub confidence: Option<f64>,
    /// Assignment labels or atom identifiers.
    pub assignments: Vec<String>,
}

/// A backend-neutral prediction result.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PredictionSet {
    /// Optional prediction name.
    pub name: Option<String>,
    /// One-dimensional predicted signals.
    pub signals_1d: Vec<PredictedSignal1D>,
    /// Two-dimensional predicted correlations.
    pub correlations_2d: Vec<PredictedCorrelation2D>,
    /// Optional provenance.
    pub provenance: Option<PredictionProvenance>,
}

impl PredictionSet {
    /// Validates all numeric payload values.
    ///
    /// # Errors
    ///
    /// Returns an error when a numeric value is non-finite or a confidence
    /// score is outside `[0, 1]`.
    pub fn validate(&self) -> Result<()> {
        for signal in &self.signals_1d {
            require_finite("delta_ppm", signal.delta_ppm)?;
            require_finite("intensity", signal.intensity)?;
            validate_confidence(signal.confidence)?;
        }
        for correlation in &self.correlations_2d {
            require_finite("x_ppm", correlation.x_ppm)?;
            require_finite("y_ppm", correlation.y_ppm)?;
            require_finite("intensity", correlation.intensity)?;
            validate_confidence(correlation.confidence)?;
        }
        Ok(())
    }
}

/// A predictor that returns a precomputed prediction payload.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StaticPrediction {
    /// Prediction payload to return.
    pub prediction: PredictionSet,
}

impl<I> Predictor<I> for StaticPrediction {
    fn predict(&self, _input: &I) -> Result<PredictionSet> {
        self.prediction.validate()?;
        Ok(self.prediction.clone())
    }
}

fn validate_confidence(confidence: Option<f64>) -> Result<()> {
    if let Some(value) = confidence {
        require_finite("confidence", value)?;
        if !(0.0..=1.0).contains(&value) {
            return Err(RSpinError::InvalidSpectrum {
                message: "confidence must be between 0 and 1".to_owned(),
            });
        }
    }
    Ok(())
}

fn require_finite(field: &'static str, value: f64) -> Result<()> {
    if !value.is_finite() {
        return Err(RSpinError::NonFinite { field });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use rspin_core::Nucleus;

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
}
