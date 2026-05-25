//! Complex-domain one-dimensional processing.

use std::f64::consts::{LN_2, PI};

use rspin_core::{Axis, ProcessingRecord, RSpinError, Result, Spectrum1D, Unit};
use rustfft::{FftPlanner, num_complex::Complex};
use serde::{Deserialize, Serialize};

use crate::ProcessingStep;

/// Applies exponential apodization to real and imaginary channels.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExponentialApodization {
    /// Line broadening in hertz.
    pub line_broadening_hz: f64,
    /// Dwell time in seconds.
    pub dwell_time_s: f64,
}

impl ExponentialApodization {
    /// Creates an exponential apodization step.
    #[must_use]
    pub fn new(line_broadening_hz: f64, dwell_time_s: f64) -> Self {
        Self {
            line_broadening_hz,
            dwell_time_s,
        }
    }
}

impl ProcessingStep<Spectrum1D> for ExponentialApodization {
    fn apply(&self, spectrum: &Spectrum1D) -> Result<Spectrum1D> {
        exponential_apodization(spectrum, self.line_broadening_hz, self.dwell_time_s)
    }
}

/// Applies Gaussian apodization to real and imaginary channels.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GaussianApodization {
    /// Gaussian broadening full width at half maximum in hertz.
    pub gaussian_broadening_hz: f64,
    /// Dwell time in seconds.
    pub dwell_time_s: f64,
}

impl GaussianApodization {
    /// Creates a Gaussian apodization step.
    #[must_use]
    pub fn new(gaussian_broadening_hz: f64, dwell_time_s: f64) -> Self {
        Self {
            gaussian_broadening_hz,
            dwell_time_s,
        }
    }
}

impl ProcessingStep<Spectrum1D> for GaussianApodization {
    fn apply(&self, spectrum: &Spectrum1D) -> Result<Spectrum1D> {
        gaussian_apodization(spectrum, self.gaussian_broadening_hz, self.dwell_time_s)
    }
}

/// Applies sine-bell apodization to real and imaginary channels.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SineBellApodization {
    /// Start angle in degrees.
    pub start_angle_deg: f64,
    /// End angle in degrees.
    pub end_angle_deg: f64,
    /// Positive exponent applied to the sine-bell weights.
    pub exponent: f64,
}

impl SineBellApodization {
    /// Creates a sine-bell apodization step.
    #[must_use]
    pub fn new(start_angle_deg: f64, end_angle_deg: f64, exponent: f64) -> Self {
        Self {
            start_angle_deg,
            end_angle_deg,
            exponent,
        }
    }
}

impl ProcessingStep<Spectrum1D> for SineBellApodization {
    fn apply(&self, spectrum: &Spectrum1D) -> Result<Spectrum1D> {
        sine_bell_apodization(
            spectrum,
            self.start_angle_deg,
            self.end_angle_deg,
            self.exponent,
        )
    }
}

/// Converts a complex spectrum to magnitude mode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Magnitude;

impl Magnitude {
    /// Creates a magnitude-mode processing step.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl ProcessingStep<Spectrum1D> for Magnitude {
    fn apply(&self, spectrum: &Spectrum1D) -> Result<Spectrum1D> {
        magnitude_spectrum(spectrum)
    }
}

/// FFT direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FftDirection {
    /// Forward transform.
    Forward,
    /// Inverse transform normalized by `1 / len`.
    Inverse,
}

/// FFT processing step.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fft1D {
    /// Transform direction.
    pub direction: FftDirection,
}

impl Fft1D {
    /// Creates a one-dimensional FFT step.
    #[must_use]
    pub fn new(direction: FftDirection) -> Self {
        Self { direction }
    }

    /// Creates a forward FFT step.
    #[must_use]
    pub fn forward() -> Self {
        Self::new(FftDirection::Forward)
    }

    /// Creates an inverse FFT step.
    #[must_use]
    pub fn inverse() -> Self {
        Self::new(FftDirection::Inverse)
    }
}

impl ProcessingStep<Spectrum1D> for Fft1D {
    fn apply(&self, spectrum: &Spectrum1D) -> Result<Spectrum1D> {
        fft_1d(spectrum, self.direction)
    }
}

/// Manual zero- and first-order phase correction.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhaseCorrection {
    /// Zero-order phase in degrees.
    pub zero_order_deg: f64,
    /// First-order phase in degrees across the full spectrum.
    pub first_order_deg: f64,
    /// Pivot position as a fraction of the index range, typically in `[0, 1]`.
    pub pivot_fraction: f64,
}

impl Default for PhaseCorrection {
    fn default() -> Self {
        Self {
            zero_order_deg: 0.0,
            first_order_deg: 0.0,
            pivot_fraction: 0.5,
        }
    }
}

impl PhaseCorrection {
    /// Creates a no-op phase correction step.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a phase correction step from explicit zero- and first-order phases.
    #[must_use]
    pub fn from_degrees(zero_order_deg: f64, first_order_deg: f64, pivot_fraction: f64) -> Self {
        Self {
            zero_order_deg,
            first_order_deg,
            pivot_fraction,
        }
    }

    /// Returns this step with a zero-order phase.
    #[must_use]
    pub fn zero_order(mut self, zero_order_deg: f64) -> Self {
        self.zero_order_deg = zero_order_deg;
        self
    }

    /// Returns this step with a first-order phase.
    #[must_use]
    pub fn first_order(mut self, first_order_deg: f64) -> Self {
        self.first_order_deg = first_order_deg;
        self
    }

    /// Returns this step with a pivot fraction.
    #[must_use]
    pub fn pivot_fraction(mut self, pivot_fraction: f64) -> Self {
        self.pivot_fraction = pivot_fraction;
        self
    }
}

impl ProcessingStep<Spectrum1D> for PhaseCorrection {
    fn apply(&self, spectrum: &Spectrum1D) -> Result<Spectrum1D> {
        phase_correct(
            spectrum,
            self.zero_order_deg,
            self.first_order_deg,
            self.pivot_fraction,
        )
    }
}

/// Applies exponential apodization.
///
/// The multiplier at point `i` is `exp(-pi * line_broadening_hz * dwell_time_s * i)`.
///
/// # Errors
///
/// Returns an error when line broadening is negative or either parameter is
/// non-finite.
pub fn exponential_apodization(
    spectrum: &Spectrum1D,
    line_broadening_hz: f64,
    dwell_time_s: f64,
) -> Result<Spectrum1D> {
    ensure_non_negative("line_broadening_hz", line_broadening_hz)?;
    ensure_positive("dwell_time_s", dwell_time_s)?;

    let decay = (-PI * line_broadening_hz * dwell_time_s).exp();
    let mut weight = 1.0;
    let mut processed = spectrum.clone();
    for value in &mut processed.intensities {
        *value *= weight;
        weight *= decay;
    }
    if let Some(imaginary) = &mut processed.imaginary {
        weight = 1.0;
        for value in imaginary {
            *value *= weight;
            weight *= decay;
        }
    }

    Ok(processed.with_processing_record(
        ProcessingRecord::new("exponential_apodization").with_details(format!(
            "line_broadening_hz={line_broadening_hz},dwell_time_s={dwell_time_s}"
        )),
    ))
}

/// Applies Gaussian apodization.
///
/// The multiplier at point `i` is
/// `exp(-(pi * gaussian_broadening_hz * dwell_time_s * i)^2 / (4 * ln(2)))`.
/// `gaussian_broadening_hz` is interpreted as the frequency-domain full width
/// at half maximum contributed by the Gaussian window.
///
/// # Errors
///
/// Returns an error when Gaussian broadening is negative, dwell time is not
/// positive, any parameter is non-finite, or the point count is too large for
/// checked numeric conversion.
pub fn gaussian_apodization(
    spectrum: &Spectrum1D,
    gaussian_broadening_hz: f64,
    dwell_time_s: f64,
) -> Result<Spectrum1D> {
    ensure_non_negative("gaussian_broadening_hz", gaussian_broadening_hz)?;
    ensure_positive("dwell_time_s", dwell_time_s)?;

    let weights = gaussian_weights(
        spectrum.len(),
        gaussian_broadening_hz,
        dwell_time_s,
        "Gaussian apodization",
    )?;
    let mut processed = spectrum.clone();
    for (value, weight) in processed.intensities.iter_mut().zip(&weights) {
        *value *= *weight;
    }
    if let Some(imaginary) = &mut processed.imaginary {
        for (value, weight) in imaginary.iter_mut().zip(&weights) {
            *value *= *weight;
        }
    }

    Ok(processed.with_processing_record(
        ProcessingRecord::new("gaussian_apodization").with_details(format!(
            "gaussian_broadening_hz={gaussian_broadening_hz},dwell_time_s={dwell_time_s}"
        )),
    ))
}

/// Applies sine-bell apodization.
///
/// The multiplier at point `i` is `sin(theta_i)^exponent`, where `theta_i`
/// moves linearly from `start_angle_deg` to `end_angle_deg` across the spectrum.
/// Angles are constrained to `0..=180` degrees so weights remain non-negative.
///
/// # Errors
///
/// Returns an error when either angle is outside `0..=180` degrees, the
/// exponent is not positive, any parameter is non-finite, or the point count is
/// too large for checked numeric conversion.
pub fn sine_bell_apodization(
    spectrum: &Spectrum1D,
    start_angle_deg: f64,
    end_angle_deg: f64,
    exponent: f64,
) -> Result<Spectrum1D> {
    let weights = sine_bell_weights(
        spectrum.len(),
        start_angle_deg,
        end_angle_deg,
        exponent,
        "sine-bell apodization",
    )?;
    let mut processed = spectrum.clone();
    for (value, weight) in processed.intensities.iter_mut().zip(&weights) {
        *value *= *weight;
    }
    if let Some(imaginary) = &mut processed.imaginary {
        for (value, weight) in imaginary.iter_mut().zip(&weights) {
            *value *= *weight;
        }
    }

    Ok(processed.with_processing_record(
        ProcessingRecord::new("sine_bell_apodization").with_details(format!(
            "start_angle_deg={start_angle_deg},end_angle_deg={end_angle_deg},exponent={exponent}"
        )),
    ))
}

/// Converts a spectrum to magnitude mode.
///
/// # Errors
///
/// Returns an error when computed magnitude data is invalid.
pub fn magnitude_spectrum(spectrum: &Spectrum1D) -> Result<Spectrum1D> {
    let intensities = match &spectrum.imaginary {
        Some(imaginary) => spectrum
            .intensities
            .iter()
            .zip(imaginary)
            .map(|(real, imag)| real.hypot(*imag))
            .collect(),
        None => spectrum
            .intensities
            .iter()
            .map(|value| value.abs())
            .collect(),
    };

    let mut processed =
        Spectrum1D::new(spectrum.x.clone(), intensities, spectrum.metadata.clone())?;
    processed.processing.clone_from(&spectrum.processing);
    Ok(processed.with_processing_record(ProcessingRecord::new("magnitude_spectrum")))
}

/// Applies a forward or inverse FFT to a one-dimensional spectrum.
///
/// The inverse direction is normalized by `1 / len`, making
/// `inverse(forward(spectrum))` recover the original values within floating
/// point tolerance.
///
/// # Errors
///
/// Returns an error when the point count cannot be represented safely for
/// normalization.
pub fn fft_1d(spectrum: &Spectrum1D, direction: FftDirection) -> Result<Spectrum1D> {
    let len = spectrum.len();
    let mut buffer = complex_buffer(spectrum);
    let mut planner = FftPlanner::<f64>::new();
    let fft = match direction {
        FftDirection::Forward => planner.plan_fft_forward(buffer.len()),
        FftDirection::Inverse => {
            ifftshift_in_place(&mut buffer);
            planner.plan_fft_inverse(buffer.len())
        }
    };
    fft.process(&mut buffer);

    match direction {
        FftDirection::Forward => fftshift_in_place(&mut buffer),
        FftDirection::Inverse => {
            let len_u32 = u32::try_from(buffer.len()).map_err(|_| RSpinError::InvalidSpectrum {
                message: "spectrum is too large to normalize inverse FFT".to_owned(),
            })?;
            let scale = 1.0 / f64::from(len_u32);
            for value in &mut buffer {
                *value *= scale;
            }
        }
    }

    let new_axis = match direction {
        FftDirection::Forward => frequency_axis_from_time(&spectrum.x, &spectrum.metadata, len)?,
        FftDirection::Inverse => time_axis_from_frequency(&spectrum.x, &spectrum.metadata, len)?,
    };

    let intensities = buffer.iter().map(|value| value.re).collect();
    let imaginary = Some(buffer.iter().map(|value| value.im).collect());
    let mut processed =
        Spectrum1D::new_complex(new_axis, intensities, imaginary, spectrum.metadata.clone())?;
    processed.processing.clone_from(&spectrum.processing);
    Ok(processed.with_processing_record(
        ProcessingRecord::new("fft_1d").with_details(format!("direction={direction:?}")),
    ))
}

/// Removes a digital-filter group delay from a time-domain spectrum.
///
/// Circularly shifts the FID samples left by `samples.trunc()` (so the
/// early "pre-acquisition" points wrap to the end of the FID) and records
/// the operation. The fractional part of `samples` is meant to be applied
/// downstream as a frequency-domain linear phase
/// `exp(-2*pi*frac*k/N)` after FFT; this function only handles the
/// integer shift so the inverse `restore_group_delay` is a clean rotation.
///
/// `samples` must be finite and non-negative.
///
/// # Errors
///
/// Returns an error when `samples` is non-finite or negative.
pub fn remove_group_delay(spectrum: &Spectrum1D, samples: f64) -> Result<Spectrum1D> {
    if !samples.is_finite() {
        return Err(RSpinError::NonFinite {
            field: "group_delay_samples",
        });
    }
    if samples < 0.0 {
        return Err(RSpinError::InvalidSpectrum {
            message: "group delay samples must be non-negative".to_owned(),
        });
    }
    let mut processed = spectrum.clone();
    let len = processed.intensities.len();
    if len == 0 {
        return Ok(processed);
    }
    let mut integer_shift = 0_usize;
    let truncated = samples.trunc();
    if truncated > 0.0 {
        let mut counter = 0.0_f64;
        for _ in 0..len {
            if counter >= truncated {
                break;
            }
            counter += 1.0;
            integer_shift += 1;
        }
    }
    if integer_shift > 0 {
        processed.intensities.rotate_left(integer_shift);
        if let Some(imag) = processed.imaginary.as_mut() {
            imag.rotate_left(integer_shift);
        }
    }
    Ok(processed.with_processing_record(
        ProcessingRecord::new("remove_group_delay").with_details(format!("samples={samples}")),
    ))
}

/// Applies a fractional sub-sample shift as a frequency-domain linear phase.
///
/// Multiplies the complex spectrum by `exp(-2*pi*i * frac * (m - N/2) / N)`
/// for each bin `m`. This is the canonical companion to a coarse
/// integer-sample FID rotation (see [`remove_group_delay`]): a digital
/// filter group delay of `g = floor(g) + frac` samples is removed by
/// circularly shifting the FID left by `floor(g)` *and* applying this
/// frequency-domain ramp after FFT (and after `fftshift`).
///
/// Following the rustfft forward convention (`exp(-2*pi*i*k*n/N)`), the
/// sign matches what `nmrglue.bruker.rm_dig_filter` documents as
/// `exp(+2*pi*i * grpdly * k / N)` once the convention difference is
/// accounted for.
///
/// `frac` may be any finite value (typically in `(-1, 1)`); a value
/// outside that range is interpreted modulo one sample.
///
/// # Errors
///
/// Returns an error when `frac` is non-finite or the spectrum has no
/// imaginary channel (a sub-sample shift cannot be applied to a
/// real-only spectrum).
pub fn apply_subsample_shift(spectrum: &Spectrum1D, frac: f64) -> Result<Spectrum1D> {
    if !frac.is_finite() {
        return Err(RSpinError::NonFinite {
            field: "subsample_shift_frac",
        });
    }
    if spectrum.imaginary.is_none() {
        return Err(RSpinError::InvalidSpectrum {
            message: "sub-sample shift requires a complex spectrum".to_owned(),
        });
    }
    let len = spectrum.len();
    if len == 0 {
        return Ok(spectrum.clone());
    }
    let len_u32 = u32::try_from(len).map_err(|_| RSpinError::InvalidSpectrum {
        message: "spectrum too long for sub-sample shift".to_owned(),
    })?;
    let n_f = f64::from(len_u32);
    let half = if len_u32 >= 2 {
        f64::from(len_u32 / 2)
    } else {
        0.0
    };

    let mut real = spectrum.intensities.clone();
    let mut imag = match spectrum.imaginary.as_ref() {
        Some(values) => values.clone(),
        None => vec![0.0; len],
    };
    for index in 0..len {
        let m_u32 = u32::try_from(index).map_err(|_| RSpinError::InvalidSpectrum {
            message: "index too large for sub-sample shift".to_owned(),
        })?;
        let m_f = f64::from(m_u32);
        let theta = -2.0 * PI * frac * (m_f - half) / n_f;
        let cos_t = theta.cos();
        let sin_t = theta.sin();
        let re = real[index];
        let im = imag[index];
        real[index] = re * cos_t - im * sin_t;
        imag[index] = re * sin_t + im * cos_t;
    }
    let mut processed = Spectrum1D::new_complex(
        spectrum.x.clone(),
        real,
        Some(imag),
        spectrum.metadata.clone(),
    )?;
    processed.processing.clone_from(&spectrum.processing);
    Ok(processed.with_processing_record(
        ProcessingRecord::new("apply_subsample_shift").with_details(format!("frac={frac}")),
    ))
}

pub(crate) fn fftshift_in_place<T: Copy>(buffer: &mut [T]) {
    let n = buffer.len();
    if n < 2 {
        return;
    }
    buffer.rotate_left(n - n / 2);
}

pub(crate) fn ifftshift_in_place<T: Copy>(buffer: &mut [T]) {
    let n = buffer.len();
    if n < 2 {
        return;
    }
    buffer.rotate_left(n / 2);
}

fn uniform_step(values: &[f64]) -> Option<f64> {
    if values.len() < 2 {
        return None;
    }
    let step = values[1] - values[0];
    if !step.is_finite() || step.abs() <= 0.0 {
        return None;
    }
    for window in values.windows(2) {
        let local = window[1] - window[0];
        let tolerance = step.abs() * 1.0e-6;
        if (local - step).abs() > tolerance {
            return None;
        }
    }
    Some(step)
}

fn safe_usize_to_f64(value: usize, field: &'static str) -> Result<f64> {
    let value_u32 = u32::try_from(value).map_err(|_| RSpinError::InvalidSpectrum {
        message: format!("{field} too large for FFT axis labeling"),
    })?;
    Ok(f64::from(value_u32))
}

pub(crate) fn frequency_axis_from_time(
    x: &Axis,
    metadata: &rspin_core::Metadata,
    len: usize,
) -> Result<Axis> {
    if x.unit != Unit::Seconds {
        return Ok(x.clone());
    }
    let Some(dwell) = uniform_step(&x.values).map(f64::abs) else {
        return Ok(x.clone());
    };
    if dwell <= 0.0 || len == 0 {
        return Ok(x.clone());
    }
    let sweep_width_hz = 1.0 / dwell;
    let half = len / 2;
    let n_f = safe_usize_to_f64(len, "spectrum length")?;
    let mut hz_values = Vec::with_capacity(len);
    for index in 0..len {
        let index_f = safe_usize_to_f64(index, "spectrum index")?;
        let half_f = safe_usize_to_f64(half, "spectrum index")?;
        hz_values.push((index_f - half_f) * sweep_width_hz / n_f);
    }
    match metadata.frequency_mhz {
        Some(freq_mhz) if freq_mhz.is_finite() && freq_mhz.abs() > 0.0 => {
            let ppm_values: Vec<f64> = hz_values.iter().map(|hz| hz / freq_mhz).collect();
            Axis::new("chemical shift", Unit::Ppm, ppm_values)
        }
        _ => Axis::new("frequency", Unit::Hertz, hz_values),
    }
}

pub(crate) fn time_axis_from_frequency(
    x: &Axis,
    metadata: &rspin_core::Metadata,
    len: usize,
) -> Result<Axis> {
    if !matches!(x.unit, Unit::Hertz | Unit::Ppm) {
        return Ok(x.clone());
    }
    let Some(raw_step) = uniform_step(&x.values).map(f64::abs) else {
        return Ok(x.clone());
    };
    let step_hz = match x.unit {
        Unit::Hertz => raw_step,
        Unit::Ppm => match metadata.frequency_mhz {
            Some(freq_mhz) if freq_mhz.is_finite() && freq_mhz.abs() > 0.0 => {
                raw_step * freq_mhz.abs()
            }
            _ => return Ok(x.clone()),
        },
        _ => return Ok(x.clone()),
    };
    let n_f = safe_usize_to_f64(len, "spectrum length")?;
    let sweep_width_hz = step_hz * n_f;
    if sweep_width_hz <= 0.0 {
        return Ok(x.clone());
    }
    let dwell = 1.0 / sweep_width_hz;
    let mut time_values = Vec::with_capacity(len);
    for index in 0..len {
        let index_f = safe_usize_to_f64(index, "spectrum index")?;
        time_values.push(index_f * dwell);
    }
    Axis::new("time", Unit::Seconds, time_values)
}

/// Applies manual phase correction to a complex one-dimensional spectrum.
///
/// The phase at point `i` is `zero_order_deg + first_order_deg *
/// (fraction(i) - pivot_fraction)`, where `fraction(i)` spans `0..=1` across
/// the spectrum. Real-only input is treated as complex data with zero imaginary
/// values, and the output always contains an imaginary channel.
///
/// # Errors
///
/// Returns an error when phase parameters are non-finite, the pivot is outside
/// `[0, 1]`, or the point count is too large for safe conversion.
pub fn phase_correct(
    spectrum: &Spectrum1D,
    zero_order_deg: f64,
    first_order_deg: f64,
    pivot_fraction: f64,
) -> Result<Spectrum1D> {
    ensure_finite("zero_order_deg", zero_order_deg)?;
    ensure_finite("first_order_deg", first_order_deg)?;
    if !pivot_fraction.is_finite() || !(0.0..=1.0).contains(&pivot_fraction) {
        return Err(RSpinError::InvalidSpectrum {
            message: "phase pivot fraction must be finite and between 0 and 1".to_owned(),
        });
    }

    let denominator = index_denominator(spectrum.len())?;
    let mut real = Vec::with_capacity(spectrum.len());
    let mut imaginary = Vec::with_capacity(spectrum.len());
    for (index, value) in complex_buffer(spectrum).into_iter().enumerate() {
        let fraction = if denominator == 0.0 {
            0.0
        } else {
            f64::from(
                u32::try_from(index).map_err(|_| RSpinError::InvalidSpectrum {
                    message: "spectrum is too large for phase correction".to_owned(),
                })?,
            ) / denominator
        };
        let phase_rad =
            (zero_order_deg + first_order_deg * (fraction - pivot_fraction)).to_radians();
        let rotation = Complex::new(phase_rad.cos(), phase_rad.sin());
        let corrected = value * rotation;
        real.push(corrected.re);
        imaginary.push(corrected.im);
    }

    let mut processed = Spectrum1D::new_complex(
        spectrum.x.clone(),
        real,
        Some(imaginary),
        spectrum.metadata.clone(),
    )?;
    processed.processing.clone_from(&spectrum.processing);
    Ok(processed.with_processing_record(
        ProcessingRecord::new("phase_correct").with_details(format!(
            "zero_order_deg={zero_order_deg},first_order_deg={first_order_deg},pivot_fraction={pivot_fraction}"
        )),
    ))
}

fn complex_buffer(spectrum: &Spectrum1D) -> Vec<Complex<f64>> {
    match &spectrum.imaginary {
        Some(imaginary) => spectrum
            .intensities
            .iter()
            .zip(imaginary)
            .map(|(real, imag)| Complex::new(*real, *imag))
            .collect(),
        None => spectrum
            .intensities
            .iter()
            .map(|real| Complex::new(*real, 0.0))
            .collect(),
    }
}

fn index_denominator(len: usize) -> Result<f64> {
    if len <= 1 {
        return Ok(0.0);
    }
    let denominator = u32::try_from(len - 1).map_err(|_| RSpinError::InvalidSpectrum {
        message: "spectrum is too large for phase correction".to_owned(),
    })?;
    Ok(f64::from(denominator))
}

fn gaussian_weights(
    len: usize,
    gaussian_broadening_hz: f64,
    dwell_time_s: f64,
    context: &'static str,
) -> Result<Vec<f64>> {
    let scale = PI * gaussian_broadening_hz * dwell_time_s;
    let denominator = 4.0 * LN_2;
    (0..len)
        .map(|index| {
            let index =
                f64::from(
                    u32::try_from(index).map_err(|_| RSpinError::InvalidSpectrum {
                        message: format!("{context} input is too large"),
                    })?,
                );
            let scaled = scale * index;
            Ok((-(scaled * scaled) / denominator).exp())
        })
        .collect()
}

fn sine_bell_weights(
    len: usize,
    start_angle_deg: f64,
    end_angle_deg: f64,
    exponent: f64,
    context: &'static str,
) -> Result<Vec<f64>> {
    ensure_angle_degrees("start_angle_deg", start_angle_deg)?;
    ensure_angle_degrees("end_angle_deg", end_angle_deg)?;
    ensure_positive("exponent", exponent)?;
    let denominator = if len <= 1 {
        0.0
    } else {
        f64::from(
            u32::try_from(len - 1).map_err(|_| RSpinError::InvalidSpectrum {
                message: format!("{context} input is too large"),
            })?,
        )
    };

    (0..len)
        .map(|index| {
            let index =
                f64::from(
                    u32::try_from(index).map_err(|_| RSpinError::InvalidSpectrum {
                        message: format!("{context} input is too large"),
                    })?,
                );
            let fraction = if denominator == 0.0 {
                0.0
            } else {
                index / denominator
            };
            let angle = start_angle_deg + (end_angle_deg - start_angle_deg) * fraction;
            Ok(angle.to_radians().sin().max(0.0).powf(exponent))
        })
        .collect()
}

fn ensure_angle_degrees(field: &'static str, value: f64) -> Result<()> {
    ensure_finite(field, value)?;
    if !(0.0..=180.0).contains(&value) {
        return Err(RSpinError::InvalidSpectrum {
            message: format!("{field} must be between 0 and 180 degrees"),
        });
    }
    Ok(())
}

fn ensure_non_negative(field: &'static str, value: f64) -> Result<()> {
    ensure_finite(field, value)?;
    if value < 0.0 {
        return Err(RSpinError::InvalidSpectrum {
            message: format!("{field} must be non-negative"),
        });
    }
    Ok(())
}

fn ensure_positive(field: &'static str, value: f64) -> Result<()> {
    ensure_finite(field, value)?;
    if value <= 0.0 {
        return Err(RSpinError::InvalidSpectrum {
            message: format!("{field} must be positive"),
        });
    }
    Ok(())
}

fn ensure_finite(field: &'static str, value: f64) -> Result<()> {
    if !value.is_finite() {
        return Err(RSpinError::NonFinite { field });
    }
    Ok(())
}

#[cfg(test)]
mod tests;
