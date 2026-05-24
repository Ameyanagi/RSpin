//! Chainable analysis workflow builders.

use rspin_core::{Result, Spectrum1D, Spectrum2D};

use crate::{
    AssignmentSet, JCouplingGraph, MultipletDetectionOptions, PeakPickOptions,
    RangeDetectionOptions, SignalSummary2DOptions, SignalSummaryOptions, SpectrumAnalysis1D,
    SpectrumAnalysis1DOptions, SpectrumAnalysis2D, SpectrumAnalysis2DOptions, ZoneDetectionOptions,
    analyze_assigned_spectrum_1d, analyze_assigned_spectrum_2d, analyze_spectrum_1d,
    analyze_spectrum_2d,
};

/// Extension trait for chainable one-dimensional spectrum analysis.
pub trait AnalyzeSpectrum1D {
    /// Creates a borrowed analysis workflow builder.
    #[must_use]
    fn analyze(&self) -> SpectrumAnalysis1DWorkflow<'_>;
}

impl AnalyzeSpectrum1D for Spectrum1D {
    fn analyze(&self) -> SpectrumAnalysis1DWorkflow<'_> {
        SpectrumAnalysis1DWorkflow::new(self)
    }
}

/// Extension trait for chainable one-dimensional analysis from fallible inputs.
pub trait AnalyzeSpectrum1DResult {
    /// Creates an owned analysis workflow builder from an existing result.
    #[must_use]
    fn analyze(self) -> SpectrumAnalysis1DResultWorkflow<'static, 'static>;
}

impl AnalyzeSpectrum1DResult for Result<Spectrum1D> {
    fn analyze(self) -> SpectrumAnalysis1DResultWorkflow<'static, 'static> {
        SpectrumAnalysis1DResultWorkflow::from_result(self)
    }
}

/// Extension trait for chainable two-dimensional spectrum analysis.
pub trait AnalyzeSpectrum2D {
    /// Creates a borrowed analysis workflow builder.
    #[must_use]
    fn analyze(&self) -> SpectrumAnalysis2DWorkflow<'_>;
}

impl AnalyzeSpectrum2D for Spectrum2D {
    fn analyze(&self) -> SpectrumAnalysis2DWorkflow<'_> {
        SpectrumAnalysis2DWorkflow::new(self)
    }
}

/// Extension trait for chainable two-dimensional analysis from fallible inputs.
pub trait AnalyzeSpectrum2DResult {
    /// Creates an owned analysis workflow builder from an existing result.
    #[must_use]
    fn analyze(self) -> SpectrumAnalysis2DResultWorkflow<'static>;
}

impl AnalyzeSpectrum2DResult for Result<Spectrum2D> {
    fn analyze(self) -> SpectrumAnalysis2DResultWorkflow<'static> {
        SpectrumAnalysis2DResultWorkflow::from_result(self)
    }
}

/// Borrowed builder for one-dimensional analysis workflows.
#[derive(Clone, Copy, Debug)]
pub struct SpectrumAnalysis1DWorkflow<'a> {
    spectrum: &'a Spectrum1D,
    options: SpectrumAnalysis1DOptions,
    assignments: Option<&'a AssignmentSet>,
    coupling_graph: Option<&'a JCouplingGraph>,
}

impl<'a> SpectrumAnalysis1DWorkflow<'a> {
    /// Creates a one-dimensional analysis workflow for `spectrum`.
    #[must_use]
    pub fn new(spectrum: &'a Spectrum1D) -> Self {
        Self {
            spectrum,
            options: SpectrumAnalysis1DOptions::default(),
            assignments: None,
            coupling_graph: None,
        }
    }

    /// Replaces all workflow options.
    #[must_use]
    pub fn with_options(mut self, options: SpectrumAnalysis1DOptions) -> Self {
        self.options = options;
        self
    }

    /// Sets peak picking options.
    #[must_use]
    pub fn with_peak_options(mut self, peak_options: PeakPickOptions) -> Self {
        self.options.peak_options = peak_options;
        self
    }

    /// Sets range detection options.
    #[must_use]
    pub fn with_range_options(mut self, range_options: RangeDetectionOptions) -> Self {
        self.options.range_options = range_options;
        self
    }

    /// Sets multiplet grouping options.
    #[must_use]
    pub fn with_multiplet_options(mut self, multiplet_options: MultipletDetectionOptions) -> Self {
        self.options.multiplet_options = multiplet_options;
        self
    }

    /// Sets signal summary options.
    #[must_use]
    pub fn with_signal_options(mut self, signal_options: SignalSummaryOptions) -> Self {
        self.options.signal_options = signal_options;
        self
    }

    /// Adds assignment context used when assembling signal summaries.
    #[must_use]
    pub fn with_assignments(mut self, assignments: &'a AssignmentSet) -> Self {
        self.assignments = Some(assignments);
        self
    }

    /// Clears assignment context.
    #[must_use]
    pub fn without_assignments(mut self) -> Self {
        self.assignments = None;
        self
    }

    /// Adds J-coupling graph context used when assembling signal summaries.
    #[must_use]
    pub fn with_coupling_graph(mut self, coupling_graph: &'a JCouplingGraph) -> Self {
        self.coupling_graph = Some(coupling_graph);
        self
    }

    /// Clears J-coupling graph context.
    #[must_use]
    pub fn without_coupling_graph(mut self) -> Self {
        self.coupling_graph = None;
        self
    }

    /// Runs the configured analysis workflow.
    ///
    /// # Errors
    ///
    /// Returns an error when options, assignment context, coupling context, or
    /// generated features are invalid.
    pub fn run(self) -> Result<SpectrumAnalysis1D> {
        run_1d_analysis(
            self.spectrum,
            self.options,
            self.assignments,
            self.coupling_graph,
        )
    }
}

/// Owned builder for one-dimensional analysis from fallible inputs.
#[derive(Debug)]
pub struct SpectrumAnalysis1DResultWorkflow<'a, 'c> {
    spectrum: Result<Spectrum1D>,
    options: SpectrumAnalysis1DOptions,
    assignments: Option<&'a AssignmentSet>,
    coupling_graph: Option<&'c JCouplingGraph>,
}

impl<'a, 'c> SpectrumAnalysis1DResultWorkflow<'a, 'c> {
    /// Creates a one-dimensional analysis workflow from an existing result.
    #[must_use]
    pub fn from_result(spectrum: Result<Spectrum1D>) -> Self {
        Self {
            spectrum,
            options: SpectrumAnalysis1DOptions::default(),
            assignments: None,
            coupling_graph: None,
        }
    }

    /// Replaces all workflow options.
    #[must_use]
    pub fn with_options(mut self, options: SpectrumAnalysis1DOptions) -> Self {
        self.options = options;
        self
    }

    /// Sets peak picking options.
    #[must_use]
    pub fn with_peak_options(mut self, peak_options: PeakPickOptions) -> Self {
        self.options.peak_options = peak_options;
        self
    }

    /// Sets range detection options.
    #[must_use]
    pub fn with_range_options(mut self, range_options: RangeDetectionOptions) -> Self {
        self.options.range_options = range_options;
        self
    }

    /// Sets multiplet grouping options.
    #[must_use]
    pub fn with_multiplet_options(mut self, multiplet_options: MultipletDetectionOptions) -> Self {
        self.options.multiplet_options = multiplet_options;
        self
    }

    /// Sets signal summary options.
    #[must_use]
    pub fn with_signal_options(mut self, signal_options: SignalSummaryOptions) -> Self {
        self.options.signal_options = signal_options;
        self
    }

    /// Adds assignment context used when assembling signal summaries.
    #[must_use]
    pub fn with_assignments<'b>(
        self,
        assignments: &'b AssignmentSet,
    ) -> SpectrumAnalysis1DResultWorkflow<'b, 'c> {
        SpectrumAnalysis1DResultWorkflow {
            spectrum: self.spectrum,
            options: self.options,
            assignments: Some(assignments),
            coupling_graph: self.coupling_graph,
        }
    }

    /// Adds J-coupling graph context used when assembling signal summaries.
    #[must_use]
    pub fn with_coupling_graph<'b>(
        self,
        coupling_graph: &'b JCouplingGraph,
    ) -> SpectrumAnalysis1DResultWorkflow<'a, 'b> {
        SpectrumAnalysis1DResultWorkflow {
            spectrum: self.spectrum,
            options: self.options,
            assignments: self.assignments,
            coupling_graph: Some(coupling_graph),
        }
    }

    /// Runs the configured analysis workflow.
    ///
    /// # Errors
    ///
    /// Returns the initial spectrum error, or an error from analysis.
    pub fn run(self) -> Result<SpectrumAnalysis1D> {
        let spectrum = self.spectrum?;
        run_1d_analysis(
            &spectrum,
            self.options,
            self.assignments,
            self.coupling_graph,
        )
    }
}

impl<'a, 'c> SpectrumAnalysis1DResultWorkflow<'a, 'c> {
    /// Clears assignment context.
    #[must_use]
    pub fn without_assignments(self) -> SpectrumAnalysis1DResultWorkflow<'static, 'c> {
        SpectrumAnalysis1DResultWorkflow {
            spectrum: self.spectrum,
            options: self.options,
            assignments: None,
            coupling_graph: self.coupling_graph,
        }
    }

    /// Clears J-coupling graph context.
    #[must_use]
    pub fn without_coupling_graph(self) -> SpectrumAnalysis1DResultWorkflow<'a, 'static> {
        SpectrumAnalysis1DResultWorkflow {
            spectrum: self.spectrum,
            options: self.options,
            assignments: self.assignments,
            coupling_graph: None,
        }
    }
}

/// Borrowed builder for two-dimensional analysis workflows.
#[derive(Clone, Copy, Debug)]
pub struct SpectrumAnalysis2DWorkflow<'a> {
    spectrum: &'a Spectrum2D,
    options: SpectrumAnalysis2DOptions,
    assignments: Option<&'a AssignmentSet>,
}

impl<'a> SpectrumAnalysis2DWorkflow<'a> {
    /// Creates a two-dimensional analysis workflow for `spectrum`.
    #[must_use]
    pub fn new(spectrum: &'a Spectrum2D) -> Self {
        Self {
            spectrum,
            options: SpectrumAnalysis2DOptions::default(),
            assignments: None,
        }
    }

    /// Replaces all workflow options.
    #[must_use]
    pub fn with_options(mut self, options: SpectrumAnalysis2DOptions) -> Self {
        self.options = options;
        self
    }

    /// Sets zone detection options.
    #[must_use]
    pub fn with_zone_options(mut self, zone_options: ZoneDetectionOptions) -> Self {
        self.options.zone_options = zone_options;
        self
    }

    /// Sets signal summary options.
    #[must_use]
    pub fn with_signal_options(mut self, signal_options: SignalSummary2DOptions) -> Self {
        self.options.signal_options = signal_options;
        self
    }

    /// Adds assignment context used when assembling signal summaries.
    #[must_use]
    pub fn with_assignments(mut self, assignments: &'a AssignmentSet) -> Self {
        self.assignments = Some(assignments);
        self
    }

    /// Clears assignment context.
    #[must_use]
    pub fn without_assignments(mut self) -> Self {
        self.assignments = None;
        self
    }

    /// Runs the configured analysis workflow.
    ///
    /// # Errors
    ///
    /// Returns an error when options, assignment context, or generated zones
    /// are invalid.
    pub fn run(self) -> Result<SpectrumAnalysis2D> {
        run_2d_analysis(self.spectrum, self.options, self.assignments)
    }
}

/// Owned builder for two-dimensional analysis from fallible inputs.
#[derive(Debug)]
pub struct SpectrumAnalysis2DResultWorkflow<'a> {
    spectrum: Result<Spectrum2D>,
    options: SpectrumAnalysis2DOptions,
    assignments: Option<&'a AssignmentSet>,
}

impl SpectrumAnalysis2DResultWorkflow<'_> {
    /// Creates a two-dimensional analysis workflow from an existing result.
    #[must_use]
    pub fn from_result(spectrum: Result<Spectrum2D>) -> Self {
        Self {
            spectrum,
            options: SpectrumAnalysis2DOptions::default(),
            assignments: None,
        }
    }

    /// Replaces all workflow options.
    #[must_use]
    pub fn with_options(mut self, options: SpectrumAnalysis2DOptions) -> Self {
        self.options = options;
        self
    }

    /// Sets zone detection options.
    #[must_use]
    pub fn with_zone_options(mut self, zone_options: ZoneDetectionOptions) -> Self {
        self.options.zone_options = zone_options;
        self
    }

    /// Sets signal summary options.
    #[must_use]
    pub fn with_signal_options(mut self, signal_options: SignalSummary2DOptions) -> Self {
        self.options.signal_options = signal_options;
        self
    }

    /// Adds assignment context used when assembling signal summaries.
    #[must_use]
    pub fn with_assignments(
        self,
        assignments: &AssignmentSet,
    ) -> SpectrumAnalysis2DResultWorkflow<'_> {
        SpectrumAnalysis2DResultWorkflow {
            spectrum: self.spectrum,
            options: self.options,
            assignments: Some(assignments),
        }
    }

    /// Clears assignment context.
    #[must_use]
    pub fn without_assignments(self) -> SpectrumAnalysis2DResultWorkflow<'static> {
        SpectrumAnalysis2DResultWorkflow {
            spectrum: self.spectrum,
            options: self.options,
            assignments: None,
        }
    }

    /// Runs the configured analysis workflow.
    ///
    /// # Errors
    ///
    /// Returns the initial spectrum error, or an error from analysis.
    pub fn run(self) -> Result<SpectrumAnalysis2D> {
        let spectrum = self.spectrum?;
        run_2d_analysis(&spectrum, self.options, self.assignments)
    }
}

fn run_1d_analysis(
    spectrum: &Spectrum1D,
    options: SpectrumAnalysis1DOptions,
    assignments: Option<&AssignmentSet>,
    coupling_graph: Option<&JCouplingGraph>,
) -> Result<SpectrumAnalysis1D> {
    match (assignments, coupling_graph) {
        (Some(assignments), Some(coupling_graph)) => {
            analyze_assigned_spectrum_1d(spectrum, assignments, coupling_graph, options)
        }
        (Some(assignments), None) => {
            let coupling_graph = JCouplingGraph::default();
            analyze_assigned_spectrum_1d(spectrum, assignments, &coupling_graph, options)
        }
        (None, Some(coupling_graph)) => {
            let assignments = AssignmentSet::default();
            analyze_assigned_spectrum_1d(spectrum, &assignments, coupling_graph, options)
        }
        (None, None) => analyze_spectrum_1d(spectrum, options),
    }
}

fn run_2d_analysis(
    spectrum: &Spectrum2D,
    options: SpectrumAnalysis2DOptions,
    assignments: Option<&AssignmentSet>,
) -> Result<SpectrumAnalysis2D> {
    if let Some(assignments) = assignments {
        analyze_assigned_spectrum_2d(spectrum, assignments, options)
    } else {
        analyze_spectrum_2d(spectrum, options)
    }
}
