//! Signal processing operations.

mod abs_1d;
mod abs_2d;
mod apodization_2d;
mod auto_phase;
mod auto_phase_2d;
mod baseline;
#[cfg(test)]
mod builder_tests;
mod contours;
mod crop_1d;
mod crop_2d;
mod one_d;
mod pipeline;
mod pipeline_2d;
mod recipe_1d;
mod recipe_2d;
mod resample_1d;
mod resample_2d;
#[cfg(test)]
mod serde_tests;
mod traits;
mod transform;
mod transform_2d;
mod two_d;
mod zero_fill_2d;

pub use abs_1d::{Abs1D, abs_1d};
pub use abs_2d::{Abs2D, abs_2d};
pub use apodization_2d::{
    ExponentialApodization2D, GaussianApodization2D, LorentzToGaussApodization2D,
    SineBellApodization2D, TrafApodization2D, TrapezoidalApodization2D, exponential_apodization_2d,
    gaussian_apodization_2d, lorentz_to_gauss_apodization_2d, sine_bell_apodization_2d,
    traf_apodization_2d, trapezoidal_apodization_2d,
};
pub use auto_phase::{
    AutoPhaseCorrection, AutoPhaseCost, AutoPhaseOptions, AutoPhaseResult, AutoPhaseStrategy,
    RegionsOptions, RegionsResult, auto_phase_correct, auto_phase_correct_regions,
    auto_phase_correct_with_peaks, peak_based_phase_estimate,
};
pub use auto_phase_2d::{
    AutoPhase2DOptions, AutoPhase2DResult, AutoPhaseCorrection2D, auto_phase_correct_2d,
};
pub use baseline::{
    BaselineFit, BaselineMethod, BaselineReport, SubtractBaseline, fit_baseline, subtract_baseline,
};
pub use contours::{ContourPoint, ContourSegment, ContourSet, contour_segments, extract_contours};
pub use crop_1d::{Crop1D, crop_1d};
pub use crop_2d::{Crop2D, crop_2d};
pub use one_d::{
    NormalizeArea, NormalizeMaxAbs, OffsetIntensity, ScaleIntensity, ShiftAxis, ZeroFill,
    normalize_area, normalize_max_abs, offset_intensity, scale_intensity, shift_axis,
    spectrum_area, zero_fill,
};
pub use pipeline::{ProcessSpectrum1D, Spectrum1DPipeline};
pub use pipeline_2d::{ProcessSpectrum2D, Spectrum2DPipeline};
pub use recipe_1d::{
    ProcessingOperation1D, ProcessingRecipe1D, apply_processing_recipe_1d,
    apply_processing_recipe_1d_until,
};
pub use recipe_2d::{
    ProcessingOperation2D, ProcessingRecipe2D, apply_processing_recipe_2d,
    apply_processing_recipe_2d_until,
};
pub use resample_1d::{Resample1D, resample_1d};
pub use resample_2d::{Resample2D, resample_2d};
pub use traits::ProcessingStep;
pub use transform::{
    ConvolutionDifferenceApodization, ExponentialApodization, Fft1D, FftDirection, FirstPointScale,
    GaussMultiplyBrukerApodization, GaussianApodization, LorentzToGaussApodization, Magnitude,
    PhaseCorrection, SineBellApodization, TrafApodization, TrapezoidalApodization,
    convolution_difference_apodization, exponential_apodization, fft_1d, first_point_scale,
    gauss_multiply_bruker_apodization, gaussian_apodization, lorentz_to_gauss_apodization,
    magnitude_spectrum, matched_filter_em, phase_correct, sine_bell_apodization, traf_apodization,
    trapezoidal_apodization,
};
pub use transform::{SubsampleShift, apply_subsample_shift, remove_group_delay};
pub use transform_2d::{Fft2D, PhaseCorrection2D, fft_2d, phase_correct_2d};
pub use two_d::{
    Normalize2DMaxAbs, Normalize2DVolume, Offset2D, ProjectionMode, Scale2D, Shift2DAxes,
    normalize_2d_max_abs, normalize_2d_volume, offset_2d, project_x, project_y, scale_2d,
    shift_2d_axes, slice_x_at_y, slice_x_at_y_index, slice_y_at_x, slice_y_at_x_index,
    spectrum_volume_2d,
};
pub use zero_fill_2d::{ZeroFill2D, zero_fill_2d};
