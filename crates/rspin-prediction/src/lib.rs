//! Prediction traits and adapter types.

mod model;
mod rules;
mod spectrum;
mod spectrum_2d;
mod traits;

pub use model::{
    Experiment, PredictedCorrelation2D, PredictedSignal1D, PredictionProvenance, PredictionSet,
    StaticPrediction,
};
pub use rules::{
    BondCorrelationRule, ElementShiftPredictor, ElementShiftRule, predict_molecule_with_rules,
};
pub use spectrum::{PredictionLineShape, PredictionSpectrumOptions, render_prediction_1d};
pub use spectrum_2d::{PredictionSpectrum2DOptions, render_prediction_2d};
pub use traits::Predictor;
