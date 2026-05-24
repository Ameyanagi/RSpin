//! High-level spectrum analysis workflows.

use serde::{Deserialize, Serialize};

use rspin_core::{Result, Spectrum1D, Spectrum2D};

use crate::{
    AssignmentSet, DetectedMultiplet, DetectedRange, DetectedZone, JCouplingGraph,
    MultipletDetectionOptions, Peak, PeakPickOptions, RangeDetectionOptions, SignalSummary1D,
    SignalSummary2D, SignalSummary2DOptions, SignalSummaryOptions, ZoneDetectionOptions,
    detect_multiplets, detect_ranges, detect_zones, pick_peaks, summarize_signals_1d,
    summarize_signals_2d,
};

mod builder;

pub use builder::{
    AnalyzeSpectrum1D, AnalyzeSpectrum1DResult, AnalyzeSpectrum2D, AnalyzeSpectrum2DResult,
    SpectrumAnalysis1DResultWorkflow, SpectrumAnalysis1DWorkflow, SpectrumAnalysis2DResultWorkflow,
    SpectrumAnalysis2DWorkflow,
};

/// Options for the default one-dimensional analysis workflow.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SpectrumAnalysis1DOptions {
    /// Peak picking options.
    pub peak_options: PeakPickOptions,
    /// Range detection options.
    pub range_options: RangeDetectionOptions,
    /// Multiplet grouping options.
    pub multiplet_options: MultipletDetectionOptions,
    /// Signal summary options.
    pub signal_options: SignalSummaryOptions,
}

impl SpectrumAnalysis1DOptions {
    /// Creates default one-dimensional analysis options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets peak picking options.
    #[must_use]
    pub fn with_peak_options(mut self, peak_options: PeakPickOptions) -> Self {
        self.peak_options = peak_options;
        self
    }

    /// Sets range detection options.
    #[must_use]
    pub fn with_range_options(mut self, range_options: RangeDetectionOptions) -> Self {
        self.range_options = range_options;
        self
    }

    /// Sets multiplet grouping options.
    #[must_use]
    pub fn with_multiplet_options(mut self, multiplet_options: MultipletDetectionOptions) -> Self {
        self.multiplet_options = multiplet_options;
        self
    }

    /// Sets signal summary options.
    #[must_use]
    pub fn with_signal_options(mut self, signal_options: SignalSummaryOptions) -> Self {
        self.signal_options = signal_options;
        self
    }
}

/// One-dimensional analysis workflow output.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpectrumAnalysis1D {
    /// Picked peaks.
    pub peaks: Vec<Peak>,
    /// Detected threshold ranges.
    pub ranges: Vec<DetectedRange>,
    /// Multiplets grouped from picked peaks.
    pub multiplets: Vec<DetectedMultiplet>,
    /// Signal summaries assembled from ranges, multiplets, assignments, and couplings.
    pub signals: Vec<SignalSummary1D>,
}

/// Options for the default two-dimensional analysis workflow.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SpectrumAnalysis2DOptions {
    /// Zone detection options.
    pub zone_options: ZoneDetectionOptions,
    /// Signal summary options.
    pub signal_options: SignalSummary2DOptions,
}

impl SpectrumAnalysis2DOptions {
    /// Creates default two-dimensional analysis options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets zone detection options.
    #[must_use]
    pub fn with_zone_options(mut self, zone_options: ZoneDetectionOptions) -> Self {
        self.zone_options = zone_options;
        self
    }

    /// Sets signal summary options.
    #[must_use]
    pub fn with_signal_options(mut self, signal_options: SignalSummary2DOptions) -> Self {
        self.signal_options = signal_options;
        self
    }
}

/// Two-dimensional analysis workflow output.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpectrumAnalysis2D {
    /// Detected connected zones.
    pub zones: Vec<DetectedZone>,
    /// Signal summaries assembled from zones and assignments.
    pub signals: Vec<SignalSummary2D>,
}

/// Runs the default one-dimensional analysis workflow without assignments.
///
/// # Errors
///
/// Returns an error when any analysis options are invalid or when generated
/// features cannot be summarized against the input spectrum.
pub fn analyze_spectrum_1d(
    spectrum: &Spectrum1D,
    options: SpectrumAnalysis1DOptions,
) -> Result<SpectrumAnalysis1D> {
    analyze_assigned_spectrum_1d(
        spectrum,
        &AssignmentSet::default(),
        &JCouplingGraph::default(),
        options,
    )
}

/// Runs the default one-dimensional analysis workflow with assignment context.
///
/// # Errors
///
/// Returns an error when any analysis options, assignments, couplings, or
/// generated features are invalid.
pub fn analyze_assigned_spectrum_1d(
    spectrum: &Spectrum1D,
    assignments: &AssignmentSet,
    coupling_graph: &JCouplingGraph,
    options: SpectrumAnalysis1DOptions,
) -> Result<SpectrumAnalysis1D> {
    let peaks = pick_peaks(spectrum, options.peak_options)?;
    let ranges = detect_ranges(spectrum, options.range_options)?;
    let multiplets = detect_multiplets(spectrum, &peaks, options.multiplet_options)?;
    let signals = summarize_signals_1d(
        spectrum,
        &ranges,
        &multiplets,
        assignments,
        coupling_graph,
        options.signal_options,
    )?;

    Ok(SpectrumAnalysis1D {
        peaks,
        ranges,
        multiplets,
        signals,
    })
}

/// Runs the default two-dimensional analysis workflow without assignments.
///
/// # Errors
///
/// Returns an error when zone options are invalid or when generated zones cannot
/// be summarized against the input spectrum.
pub fn analyze_spectrum_2d(
    spectrum: &Spectrum2D,
    options: SpectrumAnalysis2DOptions,
) -> Result<SpectrumAnalysis2D> {
    analyze_assigned_spectrum_2d(spectrum, &AssignmentSet::default(), options)
}

/// Runs the default two-dimensional analysis workflow with assignment context.
///
/// # Errors
///
/// Returns an error when zone options, assignments, or generated zones are
/// invalid.
pub fn analyze_assigned_spectrum_2d(
    spectrum: &Spectrum2D,
    assignments: &AssignmentSet,
    options: SpectrumAnalysis2DOptions,
) -> Result<SpectrumAnalysis2D> {
    let zones = detect_zones(spectrum, options.zone_options)?;
    let signals = summarize_signals_2d(spectrum, &zones, assignments, options.signal_options)?;

    Ok(SpectrumAnalysis2D { zones, signals })
}

#[cfg(test)]
mod tests;
