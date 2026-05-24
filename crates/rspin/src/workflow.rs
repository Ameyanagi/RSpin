//! Facade-level workflow bridges across lower-level crates.

use rspin_analysis::{SpectrumAnalysis1DResultWorkflow, SpectrumAnalysis2DResultWorkflow};
use rspin_processing::{Spectrum1DPipeline, Spectrum2DPipeline};

/// Extension trait for analyzing the output of a one-dimensional processing pipeline.
pub trait AnalyzeProcessedSpectrum1D {
    /// Finishes the processing pipeline and starts a chainable analysis workflow.
    #[must_use]
    fn analyze(self) -> SpectrumAnalysis1DResultWorkflow<'static, 'static>;
}

impl AnalyzeProcessedSpectrum1D for Spectrum1DPipeline {
    fn analyze(self) -> SpectrumAnalysis1DResultWorkflow<'static, 'static> {
        SpectrumAnalysis1DResultWorkflow::from_result(self.finish())
    }
}

/// Extension trait for analyzing the output of a two-dimensional processing pipeline.
pub trait AnalyzeProcessedSpectrum2D {
    /// Finishes the processing pipeline and starts a chainable analysis workflow.
    #[must_use]
    fn analyze(self) -> SpectrumAnalysis2DResultWorkflow<'static>;
}

impl AnalyzeProcessedSpectrum2D for Spectrum2DPipeline {
    fn analyze(self) -> SpectrumAnalysis2DResultWorkflow<'static> {
        SpectrumAnalysis2DResultWorkflow::from_result(self.finish())
    }
}
