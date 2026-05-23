//! Signal processing operations.

mod baseline;
mod one_d;
mod traits;
mod transform;
mod two_d;

pub use baseline::{
    BaselineFit, BaselineMethod, BaselineReport, SubtractBaseline, fit_baseline, subtract_baseline,
};
pub use one_d::{
    NormalizeMaxAbs, OffsetIntensity, ScaleIntensity, ShiftAxis, ZeroFill, normalize_max_abs,
    offset_intensity, scale_intensity, shift_axis, zero_fill,
};
pub use traits::ProcessingStep;
pub use transform::{
    ExponentialApodization, Fft1D, FftDirection, Magnitude, PhaseCorrection,
    exponential_apodization, fft_1d, magnitude_spectrum, phase_correct,
};
pub use two_d::{
    Normalize2DMaxAbs, ProjectionMode, Scale2D, normalize_2d_max_abs, project_x, project_y,
    scale_2d, slice_x_at_y_index, slice_y_at_x_index,
};
