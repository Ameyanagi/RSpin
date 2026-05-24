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

impl PredictionProvenance {
    /// Creates provenance with a source label.
    #[must_use]
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            version: None,
        }
    }

    /// Sets the source version.
    #[must_use]
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }
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

impl PredictedSignal1D {
    /// Creates a one-dimensional predicted signal with unit intensity.
    #[must_use]
    pub fn new(experiment: Experiment, nucleus: Nucleus, delta_ppm: f64) -> Self {
        Self {
            experiment,
            nucleus,
            delta_ppm,
            intensity: 1.0,
            confidence: None,
            assignments: Vec::new(),
        }
    }

    /// Sets the relative signal intensity.
    #[must_use]
    pub fn with_intensity(mut self, intensity: f64) -> Self {
        self.intensity = intensity;
        self
    }

    /// Sets the confidence score.
    #[must_use]
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = Some(confidence);
        self
    }

    /// Clears the confidence score.
    #[must_use]
    pub fn without_confidence(mut self) -> Self {
        self.confidence = None;
        self
    }

    /// Adds one assignment label or atom identifier.
    #[must_use]
    pub fn with_assignment(mut self, assignment: impl Into<String>) -> Self {
        self.assignments.push(assignment.into());
        self
    }

    /// Replaces the assignment labels or atom identifiers.
    #[must_use]
    pub fn with_assignments(
        mut self,
        assignments: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.assignments = assignments.into_iter().map(Into::into).collect();
        self
    }
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

impl PredictedCorrelation2D {
    /// Creates a two-dimensional predicted correlation with unit intensity.
    #[must_use]
    pub fn new(
        experiment: Experiment,
        x_nucleus: Nucleus,
        y_nucleus: Nucleus,
        x_ppm: f64,
        y_ppm: f64,
    ) -> Self {
        Self {
            experiment,
            x_nucleus,
            y_nucleus,
            x_ppm,
            y_ppm,
            intensity: 1.0,
            confidence: None,
            assignments: Vec::new(),
        }
    }

    /// Sets the relative correlation intensity.
    #[must_use]
    pub fn with_intensity(mut self, intensity: f64) -> Self {
        self.intensity = intensity;
        self
    }

    /// Sets the confidence score.
    #[must_use]
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = Some(confidence);
        self
    }

    /// Clears the confidence score.
    #[must_use]
    pub fn without_confidence(mut self) -> Self {
        self.confidence = None;
        self
    }

    /// Adds one assignment label or atom identifier.
    #[must_use]
    pub fn with_assignment(mut self, assignment: impl Into<String>) -> Self {
        self.assignments.push(assignment.into());
        self
    }

    /// Replaces the assignment labels or atom identifiers.
    #[must_use]
    pub fn with_assignments(
        mut self,
        assignments: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.assignments = assignments.into_iter().map(Into::into).collect();
        self
    }
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
    /// Creates an empty prediction set.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the prediction name.
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Clears the prediction name.
    #[must_use]
    pub fn without_name(mut self) -> Self {
        self.name = None;
        self
    }

    /// Adds a one-dimensional predicted signal.
    #[must_use]
    pub fn with_signal_1d(mut self, signal: PredictedSignal1D) -> Self {
        self.signals_1d.push(signal);
        self
    }

    /// Adds a two-dimensional predicted correlation.
    #[must_use]
    pub fn with_correlation_2d(mut self, correlation: PredictedCorrelation2D) -> Self {
        self.correlations_2d.push(correlation);
        self
    }

    /// Sets prediction provenance.
    #[must_use]
    pub fn with_provenance(mut self, provenance: PredictionProvenance) -> Self {
        self.provenance = Some(provenance);
        self
    }

    /// Clears prediction provenance.
    #[must_use]
    pub fn without_provenance(mut self) -> Self {
        self.provenance = None;
        self
    }

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

impl StaticPrediction {
    /// Creates a predictor that returns a precomputed prediction payload.
    #[must_use]
    pub fn new(prediction: PredictionSet) -> Self {
        Self { prediction }
    }
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
mod tests;
