//! Public facade for the `RSpin` library workspace.

pub use rspin_analysis as analysis;
pub use rspin_core as core;
pub use rspin_io as io;
pub use rspin_prediction as prediction;
pub use rspin_processing as processing;
pub use rspin_simulation as simulation;

pub use analysis::{
    AlignmentWindow, AssignedAtom, Assignment, AssignmentSet, AssignmentTarget,
    BilinearIntegrator2D, CouplingNode, DetectedMultiplet, DetectedRange, DetectedZone,
    GapMultipletDetector, Integral, Integral2D, IntegralRegion, IntegralRegion2D, Integrator,
    Integrator2D, JCoupling, JCouplingGraph, LocalExtremaPeakPicker, MatrixGeneration2DOptions,
    MatrixGenerationOptions, MultipletDetectionOptions, MultipletDetector, MultipletKind,
    OptimizedPeak, Peak, PeakAlignmentOptions, PeakAlignmentResult1D, PeakOptimizationOptions,
    PeakOptimizer, PeakPickOptions, PeakPicker, PeakPolarity, QuadraticPeakOptimizer,
    RangeDetectionOptions, RangeDetector, SignalSummary1D, SignalSummary2D, SignalSummary2DOptions,
    SignalSummaryOptions, SpectrumAlignmentShift, SpectrumMatrix1D, SpectrumMatrix2D,
    ThresholdRangeDetector, ThresholdZoneDetector, TrapezoidalIntegrator, ZoneConnectivity,
    ZoneDetectionOptions, ZoneDetector, align_spectra_by_peak, detect_multiplets, detect_ranges,
    detect_zones, deterministic_assignment_id, deterministic_j_coupling_id,
    generate_spectrum_matrix_1d, generate_spectrum_matrix_2d, integrate_region,
    integrate_region_2d, optimize_peaks_quadratic, pick_peaks, summarize_signals_1d,
    summarize_signals_2d,
};
pub use core::{
    AnnotationTarget, Atom, Axis, Bond, BondOrder, Metadata, Molecule, Nucleus, ProcessingRecord,
    RSpinError, Result, Spectrum1D, Spectrum2D, SpectrumAnnotation, Unit,
};
pub use io::{
    CsvSpectrum1D, CsvSpectrum2D, JcampDx, JsonSpectrum1D, JsonSpectrum2D, SpectrumReader,
    SpectrumWriter, read_jcamp_dx_1d, read_spectrum1d_csv, read_spectrum1d_json,
    read_spectrum2d_csv, read_spectrum2d_json, write_jcamp_dx_1d, write_spectrum1d_csv,
    write_spectrum1d_json, write_spectrum2d_csv, write_spectrum2d_json,
};
pub use prediction::{
    ElementShiftPredictor, ElementShiftRule, Experiment, PredictedCorrelation2D, PredictedSignal1D,
    PredictionLineShape, PredictionProvenance, PredictionSet, PredictionSpectrum2DOptions,
    PredictionSpectrumOptions, Predictor, StaticPrediction, predict_molecule_with_rules,
    render_prediction_1d, render_prediction_2d,
};
pub use processing::{
    Abs1D, Abs2D, AutoPhase2DOptions, AutoPhase2DResult, AutoPhaseCorrection,
    AutoPhaseCorrection2D, AutoPhaseOptions, AutoPhaseResult, BaselineFit, BaselineMethod,
    BaselineReport, ContourPoint, ContourSegment, ContourSet, Crop1D, Crop2D,
    ExponentialApodization, ExponentialApodization2D, Fft1D, Fft2D, FftDirection, Magnitude,
    Normalize2DMaxAbs, NormalizeMaxAbs, OffsetIntensity, PhaseCorrection, PhaseCorrection2D,
    ProcessSpectrum1D, ProcessSpectrum2D, ProcessingOperation1D, ProcessingOperation2D,
    ProcessingRecipe1D, ProcessingRecipe2D, ProcessingStep, ProjectionMode, Resample1D, Resample2D,
    Scale2D, ScaleIntensity, ShiftAxis, Spectrum1DPipeline, Spectrum2DPipeline, SubtractBaseline,
    ZeroFill, ZeroFill2D, abs_1d, abs_2d, apply_processing_recipe_1d,
    apply_processing_recipe_1d_until, apply_processing_recipe_2d, apply_processing_recipe_2d_until,
    auto_phase_correct, auto_phase_correct_2d, contour_segments, crop_1d, crop_2d,
    exponential_apodization, exponential_apodization_2d, extract_contours, fft_1d, fft_2d,
    fit_baseline, magnitude_spectrum, normalize_2d_max_abs, normalize_max_abs, offset_intensity,
    phase_correct, phase_correct_2d, project_x, project_y, resample_1d, resample_2d, scale_2d,
    scale_intensity, shift_axis, slice_x_at_y, slice_x_at_y_index, slice_y_at_x,
    slice_y_at_x_index, subtract_baseline, zero_fill, zero_fill_2d,
};
pub use simulation::{
    ExactSpectrumDecomposition1D, ExactSpectrumOptions, ExactSpinOptions, ExactTransition,
    ExactTransitionContribution1D, LineShape, MAX_EXACT_SPINS, ScalarCoupling, Simulator, SpinHalf,
    SpinHalfSystem, decompose_exact_spin_half_1d, exact_spin_half_transitions,
    simulate_exact_spin_half_1d,
};

/// Common imports for routine `RSpin` library workflows.
///
/// This prelude intentionally favors stable data types, IO helpers, analysis
/// primitives, processing pipeline traits, prediction types, and exact
/// simulation APIs.
pub mod prelude {
    pub use crate::{
        Abs1D, Abs2D, AlignmentWindow, AnnotationTarget, AssignedAtom, Assignment, AssignmentSet,
        AssignmentTarget, Atom, AutoPhaseOptions, Axis, BaselineMethod, BilinearIntegrator2D, Bond,
        BondOrder, Crop1D, Crop2D, CsvSpectrum1D, CsvSpectrum2D, DetectedMultiplet, DetectedRange,
        DetectedZone, ElementShiftPredictor, ElementShiftRule, ExactSpectrumOptions,
        ExactSpinOptions, ExactTransition, Experiment, FftDirection, Integral, Integral2D,
        IntegralRegion, IntegralRegion2D, JCoupling, JCouplingGraph, LineShape,
        MatrixGeneration2DOptions, MatrixGenerationOptions, Metadata, Molecule,
        MultipletDetectionOptions, MultipletKind, Nucleus, Peak, PeakAlignmentOptions,
        PeakPickOptions, PeakPolarity, PredictionLineShape, PredictionSet,
        PredictionSpectrum2DOptions, PredictionSpectrumOptions, ProcessSpectrum1D,
        ProcessSpectrum2D, ProcessingOperation1D, ProcessingOperation2D, ProcessingRecipe1D,
        ProcessingRecipe2D, ProjectionMode, RSpinError, RangeDetectionOptions, Resample1D,
        Resample2D, Result, ScalarCoupling, SignalSummary1D, SignalSummary2D,
        SignalSummary2DOptions, SignalSummaryOptions, Spectrum1D, Spectrum2D, SpectrumAnnotation,
        SpectrumMatrix1D, SpectrumMatrix2D, SpectrumReader, SpectrumWriter, SpinHalf,
        SpinHalfSystem, TrapezoidalIntegrator, Unit, ZoneConnectivity, ZoneDetectionOptions,
        abs_1d, abs_2d, align_spectra_by_peak, apply_processing_recipe_1d,
        apply_processing_recipe_1d_until, apply_processing_recipe_2d,
        apply_processing_recipe_2d_until, auto_phase_correct, auto_phase_correct_2d, crop_1d,
        crop_2d, decompose_exact_spin_half_1d, detect_multiplets, detect_ranges, detect_zones,
        exact_spin_half_transitions, extract_contours, generate_spectrum_matrix_1d,
        generate_spectrum_matrix_2d, integrate_region, integrate_region_2d, normalize_max_abs,
        pick_peaks, predict_molecule_with_rules, read_jcamp_dx_1d, read_spectrum1d_csv,
        read_spectrum1d_json, read_spectrum2d_csv, read_spectrum2d_json, render_prediction_1d,
        render_prediction_2d, resample_1d, resample_2d, scale_intensity,
        simulate_exact_spin_half_1d, slice_x_at_y, slice_y_at_x, subtract_baseline,
        summarize_signals_2d, write_jcamp_dx_1d, write_spectrum1d_csv, write_spectrum1d_json,
        write_spectrum2d_csv, write_spectrum2d_json,
    };
}

#[cfg(test)]
mod tests {
    use super::prelude::*;

    #[test]
    fn prelude_supports_common_processing_workflow() -> Result<()> {
        let spectrum = Spectrum1D::new(
            Axis::linear_ppm(0.0, 2.0, 3)?,
            vec![1.0, -2.0, 4.0],
            Metadata::new()
                .with_name("demo")
                .with_nucleus(Nucleus::Hydrogen1)
                .with_frequency_mhz(400.0),
        )?;

        let processed = spectrum
            .process()
            .crop(0.0, 1.0)
            .resample(Axis::linear_ppm(0.0, 1.0, 3)?)
            .scale(2.0)
            .absolute_value()
            .normalize_max_abs()
            .finish()?;

        assert_eq!(processed.intensities, vec![0.5, 0.25, 1.0]);
        assert_eq!(processed.processing.len(), 5);
        Ok(())
    }

    #[test]
    fn prelude_supports_common_io_and_exact_simulation() -> Result<()> {
        let spectrum = read_spectrum1d_csv("x,intensity\n1,2\n2,4\n")?;
        assert_eq!(spectrum.len(), 2);

        let molecule = Molecule::new("methane").with_atom(Atom::new("H1", "H"));
        let prediction = predict_molecule_with_rules(
            &molecule,
            &ElementShiftPredictor::new().with_rule(ElementShiftRule::new(
                "H",
                Experiment::Proton1D,
                Nucleus::Hydrogen1,
                0.9,
            )),
        )?;
        assert_eq!(prediction.signals_1d.len(), 1);

        let system = SpinHalfSystem::new().with_spin(1.0);
        let transitions = exact_spin_half_transitions(
            &system,
            &ExactSpinOptions {
                spectrometer_mhz: 400.0,
                ..ExactSpinOptions::default()
            },
        )?;

        assert_eq!(transitions.len(), 1);
        assert!((transitions[0].center_ppm - 1.0).abs() < 1.0e-12);
        Ok(())
    }
}
