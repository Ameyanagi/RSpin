//! Spectrum analysis operations.

mod alignment;
mod assignments;
mod buckets;
mod couplings;
mod integration;
mod integration_2d;
mod matrix;
mod multiplets;
mod peak_optimization;
mod peaks;
mod ranges;
mod signals;
mod traits;
mod zones;

pub use alignment::{
    AlignmentWindow, PeakAlignedMatrix1D, PeakAlignmentOptions, PeakAlignmentResult1D,
    SpectrumAlignmentShift, align_spectra_by_peak, align_spectra_by_peak_to_matrix,
};
pub use assignments::{
    AssignedAtom, Assignment, AssignmentSet, AssignmentTarget, deterministic_assignment_id,
};
pub use buckets::{
    BucketMatrix1D, BucketMatrix2D, BucketOptions1D, BucketOptions2D, SpectralBucket1D,
    SpectralBucket2D, bucket_spectra_1d, bucket_spectra_2d, bucket_spectrum_1d, bucket_spectrum_2d,
};
pub use couplings::{CouplingNode, JCoupling, JCouplingGraph, deterministic_j_coupling_id};
pub use integration::{Integral, IntegralRegion, TrapezoidalIntegrator, integrate_region};
pub use integration_2d::{BilinearIntegrator2D, Integral2D, IntegralRegion2D, integrate_region_2d};
pub use matrix::{
    MatrixGeneration2DOptions, MatrixGenerationOptions, SpectrumMatrix1D, SpectrumMatrix2D,
    generate_spectrum_matrix_1d, generate_spectrum_matrix_2d,
};
pub use multiplets::{
    DetectedMultiplet, GapMultipletDetector, MultipletDetectionOptions, MultipletKind,
    detect_multiplets,
};
pub use peak_optimization::{
    OptimizedPeak, PeakOptimizationOptions, QuadraticPeakOptimizer, optimize_peaks_quadratic,
};
pub use peaks::{LocalExtremaPeakPicker, Peak, PeakPickOptions, PeakPolarity, pick_peaks};
pub use ranges::{DetectedRange, RangeDetectionOptions, ThresholdRangeDetector, detect_ranges};
pub use signals::{
    SignalSummary1D, SignalSummary2D, SignalSummary2DOptions, SignalSummaryOptions,
    summarize_signals_1d, summarize_signals_2d,
};
pub use traits::{
    Integrator, Integrator2D, MultipletDetector, PeakOptimizer, PeakPicker, RangeDetector,
    ZoneDetector,
};
pub use zones::{
    DetectedZone, ThresholdZoneDetector, ZoneConnectivity, ZoneDetectionOptions, detect_zones,
};
