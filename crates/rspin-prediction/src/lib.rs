//! Prediction traits and adapter types.

mod model;
mod spectrum;
mod traits;

pub use model::{
    Experiment, PredictedCorrelation2D, PredictedSignal1D, PredictionProvenance, PredictionSet,
    StaticPrediction,
};
pub use spectrum::{PredictionLineShape, PredictionSpectrumOptions, render_prediction_1d};
pub use traits::Predictor;
