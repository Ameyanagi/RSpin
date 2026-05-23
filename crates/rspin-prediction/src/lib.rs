//! Prediction traits and adapter types.

mod model;
mod traits;

pub use model::{
    Experiment, PredictedCorrelation2D, PredictedSignal1D, PredictionProvenance, PredictionSet,
    StaticPrediction,
};
pub use traits::Predictor;
