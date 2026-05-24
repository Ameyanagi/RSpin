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
        match (self.assignments, self.coupling_graph) {
            (Some(assignments), Some(coupling_graph)) => analyze_assigned_spectrum_1d(
                self.spectrum,
                assignments,
                coupling_graph,
                self.options,
            ),
            (Some(assignments), None) => {
                let coupling_graph = JCouplingGraph::default();
                analyze_assigned_spectrum_1d(
                    self.spectrum,
                    assignments,
                    &coupling_graph,
                    self.options,
                )
            }
            (None, Some(coupling_graph)) => {
                let assignments = AssignmentSet::default();
                analyze_assigned_spectrum_1d(
                    self.spectrum,
                    &assignments,
                    coupling_graph,
                    self.options,
                )
            }
            (None, None) => analyze_spectrum_1d(self.spectrum, self.options),
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
        if let Some(assignments) = self.assignments {
            analyze_assigned_spectrum_2d(self.spectrum, assignments, self.options)
        } else {
            analyze_spectrum_2d(self.spectrum, self.options)
        }
    }
}
