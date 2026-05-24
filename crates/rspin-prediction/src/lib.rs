//! Prediction traits and adapter types.

mod model;
mod rules;
mod spectrum;
mod spectrum_2d;
mod traits;
mod workflow;

pub use model::{
    Experiment, PredictedCorrelation2D, PredictedSignal1D, PredictionProvenance, PredictionSet,
    StaticPrediction,
};
pub use rules::{
    BondCorrelationRule, ElementShiftPredictor, ElementShiftRule, predict_formula_with_rules,
    predict_molecule_with_rules,
};
pub use spectrum::{PredictionLineShape, PredictionSpectrumOptions, render_prediction_1d};
pub use spectrum_2d::{PredictionSpectrum2DOptions, render_prediction_2d};
pub use traits::Predictor;
pub use workflow::{
    PredictionSpectrum1DResultWorkflow, PredictionSpectrum1DWorkflow,
    PredictionSpectrum2DResultWorkflow, PredictionSpectrum2DWorkflow, RenderPrediction1D,
    RenderPrediction1DResult, RenderPrediction2D, RenderPrediction2DResult,
};
