//! Public facade for the `RSpin` library workspace.

pub use rspin_analysis as analysis;
pub use rspin_core as core;
pub use rspin_io as io;
pub use rspin_prediction as prediction;
pub use rspin_processing as processing;
pub use rspin_simulation as simulation;

pub use analysis::{
    AlignmentWindow, AssignedAtom, Assignment, AssignmentSet, AssignmentTarget,
    BilinearIntegrator2D, BucketMatrix1D, BucketMatrix2D, BucketOptions1D, BucketOptions2D,
    ClusterMerge, ConsensusPeak1D, ConsensusPeakMember1D, ConsensusPeakOptions, ConsensusRange1D,
    ConsensusRangeMember1D, ConsensusRangeOptions, CouplingNode, DetectedMultiplet, DetectedRange,
    DetectedZone, GapMultipletDetector, Integral, Integral2D, IntegralRegion, IntegralRegion2D,
    Integrator, Integrator2D, JCoupling, JCouplingGraph, LocalExtremaPeakPicker, MatrixClusterCut,
    MatrixClusterMetric, MatrixClusterResult, MatrixClusteringOptions, MatrixGeneration2DOptions,
    MatrixGenerationOptions, MatrixLinkage, MatrixPairwiseMetric, MatrixPairwiseOptions,
    MatrixPairwiseResult, MatrixPcaOptions, MatrixPcaResult, MatrixScaling,
    MultipletDetectionOptions, MultipletDetector, MultipletKind, OptimizedPeak, Peak,
    PeakAlignedMatrix1D, PeakAlignmentOptions, PeakAlignmentResult1D, PeakOptimizationOptions,
    PeakOptimizer, PeakPickOptions, PeakPicker, PeakPolarity, QuadraticPeakOptimizer,
    RangeDetectionOptions, RangeDetector, SignalSummary1D, SignalSummary2D, SignalSummary2DOptions,
    SignalSummaryOptions, SpectralBucket1D, SpectralBucket2D, SpectrumAlignmentShift,
    SpectrumMatrix1D, SpectrumMatrix2D, ThresholdRangeDetector, ThresholdZoneDetector,
    TrapezoidalIntegrator, ZoneConnectivity, ZoneDetectionOptions, ZoneDetector,
    align_spectra_by_peak, align_spectra_by_peak_to_matrix, bucket_spectra_1d, bucket_spectra_2d,
    bucket_spectrum_1d, bucket_spectrum_2d, cluster_bucket_matrix_1d, cluster_bucket_matrix_2d,
    cluster_matrix, cluster_spectrum_matrix_1d, cluster_spectrum_matrix_2d,
    detect_consensus_peaks_1d, detect_consensus_ranges_1d, detect_multiplets, detect_ranges,
    detect_zones, deterministic_assignment_id, deterministic_j_coupling_id,
    generate_spectrum_matrix_1d, generate_spectrum_matrix_2d, integrate_region,
    integrate_region_2d, optimize_peaks_quadratic, pairwise_bucket_matrix_1d,
    pairwise_bucket_matrix_2d, pairwise_matrix, pairwise_spectrum_matrix_1d,
    pairwise_spectrum_matrix_2d, pca_bucket_matrix_1d, pca_bucket_matrix_2d, pca_matrix,
    pca_spectrum_matrix_1d, pca_spectrum_matrix_2d, pick_peaks, summarize_signals_1d,
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
    BondCorrelationRule, ElementShiftPredictor, ElementShiftRule, Experiment,
    PredictedCorrelation2D, PredictedSignal1D, PredictionLineShape, PredictionProvenance,
    PredictionSet, PredictionSpectrum2DOptions, PredictionSpectrumOptions, Predictor,
    StaticPrediction, predict_molecule_with_rules, render_prediction_1d, render_prediction_2d,
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
    ExactSpectrum2DOptions, ExactSpectrumDecomposition1D, ExactSpectrumDecomposition2D,
    ExactSpectrumOptions, ExactSpinOptions, ExactSpinPair, ExactTransition,
    ExactTransitionContribution1D, ExactTransitionContribution2D, LineShape, MAX_EXACT_SPINS,
    ScalarCoupling, Simulator, SpinHalf, SpinHalfSystem, decompose_exact_spin_half_1d,
    decompose_exact_spin_half_2d, exact_spin_half_transitions, simulate_exact_spin_half_1d,
    simulate_exact_spin_half_2d,
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
        BondCorrelationRule, BondOrder, BucketMatrix1D, BucketMatrix2D, BucketOptions1D,
        BucketOptions2D, ClusterMerge, ConsensusPeak1D, ConsensusPeakMember1D,
        ConsensusPeakOptions, ConsensusRange1D, ConsensusRangeMember1D, ConsensusRangeOptions,
        Crop1D, Crop2D, CsvSpectrum1D, CsvSpectrum2D, DetectedMultiplet, DetectedRange,
        DetectedZone, ElementShiftPredictor, ElementShiftRule, ExactSpectrum2DOptions,
        ExactSpectrumOptions, ExactSpinOptions, ExactSpinPair, ExactTransition, Experiment,
        FftDirection, Integral, Integral2D, IntegralRegion, IntegralRegion2D, JCoupling,
        JCouplingGraph, LineShape, MatrixClusterCut, MatrixClusterMetric, MatrixClusterResult,
        MatrixClusteringOptions, MatrixGeneration2DOptions, MatrixGenerationOptions, MatrixLinkage,
        MatrixPairwiseMetric, MatrixPairwiseOptions, MatrixPairwiseResult, MatrixPcaOptions,
        MatrixPcaResult, MatrixScaling, Metadata, Molecule, MultipletDetectionOptions,
        MultipletKind, Nucleus, Peak, PeakAlignedMatrix1D, PeakAlignmentOptions, PeakPickOptions,
        PeakPolarity, PredictionLineShape, PredictionSet, PredictionSpectrum2DOptions,
        PredictionSpectrumOptions, ProcessSpectrum1D, ProcessSpectrum2D, ProcessingOperation1D,
        ProcessingOperation2D, ProcessingRecipe1D, ProcessingRecipe2D, ProjectionMode, RSpinError,
        RangeDetectionOptions, Resample1D, Resample2D, Result, ScalarCoupling, SignalSummary1D,
        SignalSummary2D, SignalSummary2DOptions, SignalSummaryOptions, SpectralBucket1D,
        SpectralBucket2D, Spectrum1D, Spectrum2D, SpectrumAnnotation, SpectrumMatrix1D,
        SpectrumMatrix2D, SpectrumReader, SpectrumWriter, SpinHalf, SpinHalfSystem,
        TrapezoidalIntegrator, Unit, ZoneConnectivity, ZoneDetectionOptions, abs_1d, abs_2d,
        align_spectra_by_peak, align_spectra_by_peak_to_matrix, apply_processing_recipe_1d,
        apply_processing_recipe_1d_until, apply_processing_recipe_2d,
        apply_processing_recipe_2d_until, auto_phase_correct, auto_phase_correct_2d,
        bucket_spectra_1d, bucket_spectra_2d, bucket_spectrum_1d, bucket_spectrum_2d,
        cluster_bucket_matrix_1d, cluster_bucket_matrix_2d, cluster_matrix,
        cluster_spectrum_matrix_1d, cluster_spectrum_matrix_2d, crop_1d, crop_2d,
        decompose_exact_spin_half_1d, decompose_exact_spin_half_2d, detect_consensus_peaks_1d,
        detect_consensus_ranges_1d, detect_multiplets, detect_ranges, detect_zones,
        exact_spin_half_transitions, extract_contours, generate_spectrum_matrix_1d,
        generate_spectrum_matrix_2d, integrate_region, integrate_region_2d, normalize_max_abs,
        pairwise_bucket_matrix_1d, pairwise_bucket_matrix_2d, pairwise_matrix,
        pairwise_spectrum_matrix_1d, pairwise_spectrum_matrix_2d, pca_bucket_matrix_1d,
        pca_bucket_matrix_2d, pca_matrix, pca_spectrum_matrix_1d, pca_spectrum_matrix_2d,
        pick_peaks, predict_molecule_with_rules, read_jcamp_dx_1d, read_spectrum1d_csv,
        read_spectrum1d_json, read_spectrum2d_csv, read_spectrum2d_json, render_prediction_1d,
        render_prediction_2d, resample_1d, resample_2d, scale_intensity,
        simulate_exact_spin_half_1d, simulate_exact_spin_half_2d, slice_x_at_y, slice_y_at_x,
        subtract_baseline, summarize_signals_2d, write_jcamp_dx_1d, write_spectrum1d_csv,
        write_spectrum1d_json, write_spectrum2d_csv, write_spectrum2d_json,
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

        let aligned = align_spectra_by_peak_to_matrix(
            &[
                Spectrum1D::new(
                    Axis::linear_ppm(0.0, 2.0, 3)?,
                    vec![0.0, 5.0, 0.0],
                    Metadata::named("ref"),
                )?,
                Spectrum1D::new(
                    Axis::linear_ppm(0.5, 2.5, 3)?,
                    vec![0.0, 7.0, 0.0],
                    Metadata::named("shifted"),
                )?,
            ],
            PeakAlignmentOptions::new(),
            MatrixGenerationOptions::new(),
        )?;
        assert_eq!(aligned.matrix.shape(), (2, 3));

        let buckets = bucket_spectrum_1d(
            &Spectrum1D::new(
                Axis::linear_ppm(0.0, 2.0, 3)?,
                vec![1.0, 1.0, 1.0],
                Metadata::named("bucketed"),
            )?,
            BucketOptions1D::new(0.0, 2.0, 2),
        )?;
        assert_eq!(buckets.len(), 2);

        let buckets_2d = bucket_spectrum_2d(
            &Spectrum2D::new(
                Axis::linear_ppm(0.0, 2.0, 3)?,
                Axis::linear_ppm(0.0, 2.0, 3)?,
                vec![1.0; 9],
                Metadata::named("bucketed-2d"),
            )?,
            BucketOptions2D::new(0.0, 2.0, 0.0, 2.0, 2, 2),
        )?;
        assert_eq!(buckets_2d.len(), 4);

        let pca = pca_matrix(
            &["a".to_owned(), "b".to_owned(), "c".to_owned()],
            &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
            3,
            2,
            MatrixPcaOptions::new()
                .with_component_count(1)
                .with_scaling(MatrixScaling::None),
        )?;
        assert_eq!(pca.score_shape(), (3, 1));

        let pairwise = pairwise_matrix(
            &["a".to_owned(), "b".to_owned()],
            &[3.0, 4.0, 0.0, 0.0],
            2,
            2,
            MatrixPairwiseOptions::new().with_metric(MatrixPairwiseMetric::EuclideanDistance),
        )?;
        let pairwise_value =
            pairwise
                .value_at(0, 1)
                .ok_or_else(|| RSpinError::InvalidSpectrum {
                    message: "missing pairwise value".to_owned(),
                })?;
        assert!((pairwise_value - 5.0).abs() < 1.0e-12);

        let clusters = cluster_matrix(
            &["a".to_owned(), "b".to_owned(), "c".to_owned()],
            &[0.0, 2.0, 5.0],
            3,
            1,
            MatrixClusteringOptions::new().with_linkage(MatrixLinkage::Single),
        )?;
        assert_eq!(clusters.merges.len(), 2);
        let cluster_cut = clusters.cut_to_cluster_count(2)?;
        assert_eq!(cluster_cut.cluster_ids, vec![0, 0, 1]);

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

    #[test]
    fn prelude_supports_prediction_bond_correlations() -> Result<()> {
        let molecule = Molecule::new("methanol")
            .with_atom(Atom::new("H1", "H"))
            .with_atom(Atom::new("C1", "C"))
            .with_bond(Bond::new("C1", "H1"));
        let prediction = predict_molecule_with_rules(
            &molecule,
            &ElementShiftPredictor::new()
                .with_rule(ElementShiftRule::new(
                    "H",
                    Experiment::Proton1D,
                    Nucleus::Hydrogen1,
                    0.9,
                ))
                .with_rule(ElementShiftRule::new(
                    "C",
                    Experiment::Carbon13_1D,
                    Nucleus::Carbon13,
                    50.0,
                ))
                .with_correlation_rule(BondCorrelationRule::new(
                    Experiment::Hsqc,
                    Nucleus::Hydrogen1,
                    Nucleus::Carbon13,
                )),
        )?;

        assert_eq!(prediction.signals_1d.len(), 2);
        assert_eq!(prediction.correlations_2d.len(), 1);
        Ok(())
    }

    #[test]
    fn prelude_supports_exact_2d_simulation() -> Result<()> {
        let system = SpinHalfSystem::new().with_spin(1.0).with_spin(2.0);
        let spectrum = simulate_exact_spin_half_2d(
            &system,
            &ExactSpectrum2DOptions::new()
                .with_x_ppm_range(0.95, 1.05)
                .with_y_ppm_range(1.95, 2.05)
                .with_points(5, 5)
                .with_spin_pair(0, 1),
        )?;

        assert_eq!(spectrum.shape(), (5, 5));
        assert!(spectrum.z[12] > spectrum.z[0]);
        Ok(())
    }

    #[test]
    fn prelude_supports_consensus_workflows() -> Result<()> {
        let consensus = detect_consensus_peaks_1d(
            &[
                Spectrum1D::new(
                    Axis::linear_ppm(0.0, 2.0, 3)?,
                    vec![0.0, 5.0, 0.0],
                    Metadata::named("a"),
                )?,
                Spectrum1D::new(
                    Axis::linear_ppm(0.02, 2.02, 3)?,
                    vec![0.0, 4.0, 0.0],
                    Metadata::named("b"),
                )?,
            ],
            ConsensusPeakOptions::new()
                .with_max_shift(0.05)
                .with_min_spectrum_count(2),
        )?;

        assert_eq!(consensus.len(), 1);
        assert_eq!(consensus[0].spectrum_count, 2);

        let consensus_ranges = detect_consensus_ranges_1d(
            &[
                Spectrum1D::new(
                    Axis::linear_ppm(0.0, 3.0, 4)?,
                    vec![0.0, 2.0, 3.0, 0.0],
                    Metadata::named("a"),
                )?,
                Spectrum1D::new(
                    Axis::linear_ppm(0.02, 3.02, 4)?,
                    vec![0.0, 4.0, 5.0, 0.0],
                    Metadata::named("b"),
                )?,
            ],
            ConsensusRangeOptions::new()
                .with_max_gap(0.05)
                .with_min_spectrum_count(2)
                .with_range_options(RangeDetectionOptions::new().with_threshold_abs(1.0)),
        )?;

        assert_eq!(consensus_ranges.len(), 1);
        assert_eq!(consensus_ranges[0].spectrum_count, 2);
        Ok(())
    }
}
