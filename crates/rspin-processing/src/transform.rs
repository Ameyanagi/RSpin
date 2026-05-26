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

    /// Creates the unshifted sine-squared window (`nmrPipe -fn SP -off 0 -end 1 -pow 2`).
    #[must_use]
    pub fn sine_squared() -> Self {
        Self::new(0.0, 180.0, 2.0)
    }

    /// Creates the cosine-bell window (`nmrPipe -fn SP -off 0.5 -end 1 -pow 1`),
    /// equivalent to a Hann window.
    #[must_use]
    pub fn cosine_bell() -> Self {
        Self::new(90.0, 180.0, 1.0)
    }

    /// Creates the cosine-squared window (`nmrPipe -fn SP -off 0.5 -end 1 -pow 2`),
    /// the standard biomolecular HSQC indirect-dimension default.
    #[must_use]
    pub fn cosine_squared() -> Self {
        Self::new(90.0, 180.0, 2.0)
    }

    /// Creates a shifted-sine window with a start fraction (`off`) in
    /// `[0, 1]` and an explicit positive exponent, matching nmrPipe's
    /// `-fn SP -off <off> -end 1 -pow <exp>` convention.
    #[must_use]
    pub fn shifted_sine(offset_fraction: f64, exponent: f64) -> Self {
        Self::new(offset_fraction * 180.0, 180.0, exponent)
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

/// Applies Lorentz-to-Gauss (resolution-enhancement) apodization.
///
/// The weight at point `i` is
/// `exp(+π · lorentz_to_undo_hz · t) · exp(-(π · gauss_fwhm_hz · (t - shift · t_max))² / (4 · ln 2))`
/// with `t = i · dwell_time_s` and `t_max = (N - 1) · dwell_time_s`.
///
/// Following Ferrige & Lindon (J. Magn. Reson. 1978, 31, 337) and the
/// convention used by nmrPipe `-fn GM`, `lorentz_to_undo_hz` cancels an
/// underlying Lorentzian decay (so it should be set to the natural
/// linewidth that is to be removed) and `gauss_fwhm_hz` imposes a
/// Gaussian envelope of that FWHM. `gauss_shift` ∈ `[0, 1]` lets the
/// Gaussian peak away from `t = 0` for pseudo-echo experiments.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct LorentzToGaussApodization {
    /// Lorentzian linewidth to undo, in hertz (≥ 0).
    pub lorentz_to_undo_hz: f64,
    /// Gaussian full-width-at-half-maximum to impose, in hertz (≥ 0).
    pub gauss_fwhm_hz: f64,
    /// Position of the Gaussian peak as a fraction of the FID duration (`0..=1`).
    pub gauss_shift: f64,
    /// Dwell time in seconds.
    pub dwell_time_s: f64,
}

impl LorentzToGaussApodization {
    /// Creates a Lorentz-to-Gauss apodization step with the Gaussian
    /// peaked at the start of the FID.
    #[must_use]
    pub fn new(lorentz_to_undo_hz: f64, gauss_fwhm_hz: f64, dwell_time_s: f64) -> Self {
        Self {
            lorentz_to_undo_hz,
            gauss_fwhm_hz,
            gauss_shift: 0.0,
            dwell_time_s,
        }
    }

    /// Returns this step with a Gaussian-peak shift in `[0, 1]`.
    #[must_use]
    pub fn with_gauss_shift(mut self, gauss_shift: f64) -> Self {
        self.gauss_shift = gauss_shift;
        self
    }
}

impl ProcessingStep<Spectrum1D> for LorentzToGaussApodization {
    fn apply(&self, spectrum: &Spectrum1D) -> Result<Spectrum1D> {
        lorentz_to_gauss_apodization(
            spectrum,
            self.lorentz_to_undo_hz,
            self.gauss_fwhm_hz,
            self.gauss_shift,
            self.dwell_time_s,
        )
    }
}

/// Applies TRAF (Traficante) apodization.
///
/// `w[i] = E² / (E³ + R³)` with `E = exp(-π · LB · i · dt)` and
/// `R = exp(-π · LB · (N-1-i) · dt)`, following Traficante,
/// *Concepts Magn. Reson.* 12 (2000) 83-101. TRAF is a self-normalising
/// matched filter that preserves SNR while sharpening peaks; it is the
/// preferred default for 13C in some commercial pipelines (ACD/Labs).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct TrafApodization {
    /// Line broadening in hertz (≥ 0).
    pub line_broadening_hz: f64,
    /// Dwell time in seconds (> 0).
    pub dwell_time_s: f64,
}

impl TrafApodization {
    /// Creates a TRAF apodization step.
    #[must_use]
    pub fn new(line_broadening_hz: f64, dwell_time_s: f64) -> Self {
        Self {
            line_broadening_hz,
            dwell_time_s,
        }
    }
}

impl ProcessingStep<Spectrum1D> for TrafApodization {
    fn apply(&self, spectrum: &Spectrum1D) -> Result<Spectrum1D> {
        traf_apodization(spectrum, self.line_broadening_hz, self.dwell_time_s)
    }
}

/// Applies Bruker-style two-parameter Gaussian apodization (`procs` GMB).
///
/// `w[i] = exp(-a · i - b · i²)` with `a = π · LB · dt` and
/// `b = -a / (2 · GB · (N-1) · dt)` when `GB > 0`, else `b = 0`.
///
/// This matches the `LB`/`GB` parameter convention of Bruker's `procs`
/// file: signed `lb_hz` and the fractional Gaussian peak position
/// `gb_fraction ∈ [0, 1]`. Negative `lb_hz` combined with positive
/// `gb_fraction` yields resolution enhancement; positive `lb_hz` with
/// `gb_fraction = 0` reduces to exponential apodization.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GaussMultiplyBrukerApodization {
    /// Signed Bruker `LB` line broadening, in hertz.
    pub line_broadening_hz: f64,
    /// Bruker `GB` Gaussian peak position as a fraction of the FID, in `[0, 1]`.
    pub gauss_position_fraction: f64,
    /// Dwell time in seconds (> 0).
    pub dwell_time_s: f64,
}

impl GaussMultiplyBrukerApodization {
    /// Creates a Bruker-convention GMB apodization step.
    #[must_use]
    pub fn new(line_broadening_hz: f64, gauss_position_fraction: f64, dwell_time_s: f64) -> Self {
        Self {
            line_broadening_hz,
            gauss_position_fraction,
            dwell_time_s,
        }
    }
}

impl ProcessingStep<Spectrum1D> for GaussMultiplyBrukerApodization {
    fn apply(&self, spectrum: &Spectrum1D) -> Result<Spectrum1D> {
        gauss_multiply_bruker_apodization(
            spectrum,
            self.line_broadening_hz,
            self.gauss_position_fraction,
            self.dwell_time_s,
        )
    }
}

/// Estimates an SNR-optimal exponential apodization from a time-domain FID
/// (the "matched filter" recipe of Ernst).
///
/// # Background
///
/// The matched filter is the SNR-optimal window for a signal of known
/// shape. For an NMR FID
///
/// ```text
/// s(t) = exp(-π · LB · t) · cos(2π · ν · t) + noise(t)
/// ```
///
/// multiplying by `w(t) = s(t)` before FFT maximises the post-transform
/// peak-to-noise ratio. For NMR that reduces to multiplying the FID by
/// `exp(-π · LB · t)` — exponential apodization with the **natural
/// linewidth**. The cost is that lines broaden by a factor of two in
/// the frequency domain: you pay resolution for SNR.
///
/// # Procedure
///
/// 1. Forward-FFT the input FID with no window.
/// 2. Take the magnitude spectrum.
/// 3. Locate the strongest magnitude peak.
/// 4. Measure its FWHM in Hz (using the FFT axis directly when it is
///    already in Hz, or scaling by `metadata.frequency_mhz` when it is
///    in ppm).
/// 5. Divide that FWHM by √3 — the magnitude lineshape has
///    `FWHM = √3 · LB` while the absorption-mode lineshape has
///    `FWHM = LB`. The Ernst-optimal LB is the absorption-mode width.
/// 6. Return an [`ExponentialApodization`] step with that LB and the
///    FID's dwell time.
///
/// # When *not* to use the matched filter
///
/// - Spectra whose peaks have very different natural linewidths
///   (e.g. fast-relaxing methyls alongside slow-relaxing aromatics).
///   The estimate is dominated by the strongest peak; weaker peaks of
///   different width are mis-weighted.
/// - Resolution-critical work (couplings, dispersion analysis); the
///   ×2 linewidth penalty is exactly the wrong direction. Use
///   [`LorentzToGaussApodization`] instead.
/// - FIDs that have already been broadened upstream (e.g. Bruker
///   `procs` with `LB > 0`). Apodising again would double-broaden.
///
/// # Errors
///
/// Returns an error when the input is not a time-domain spectrum
/// (axis unit must be `Seconds`), the dwell time cannot be inferred,
/// the spectrum is too short to FFT, or the FWHM cannot be measured.
pub fn matched_filter_em(spectrum: &Spectrum1D) -> Result<ExponentialApodization> {
    if spectrum.x.unit != Unit::Seconds {
        return Err(RSpinError::InvalidSpectrum {
            message: "matched_filter_em requires a time-domain FID (axis unit = Seconds)"
                .to_owned(),
        });
    }
    let dwell =
        uniform_step(&spectrum.x.values)
            .map(f64::abs)
            .ok_or(RSpinError::InvalidSpectrum {
                message: "matched_filter_em requires a uniformly-spaced time axis".to_owned(),
            })?;
    if dwell <= 0.0 {
        return Err(RSpinError::InvalidSpectrum {
            message: "matched_filter_em requires a positive dwell time".to_owned(),
        });
    }
    if spectrum.len() < 8 {
        return Err(RSpinError::InvalidSpectrum {
            message: "matched_filter_em requires at least 8 FID points".to_owned(),
        });
    }

    let frequency = fft_1d(spectrum, FftDirection::Forward)?;
    let magnitude = magnitude_spectrum(&frequency)?;
    let mut peak_index = 0_usize;
    let mut peak_value = f64::NEG_INFINITY;
    for (index, value) in magnitude.intensities.iter().enumerate() {
        if *value > peak_value {
            peak_value = *value;
            peak_index = index;
        }
    }
    if !peak_value.is_finite() || peak_value <= 0.0 {
        return Err(RSpinError::InvalidSpectrum {
            message: "matched_filter_em could not find a positive peak".to_owned(),
        });
    }
    let half = peak_value / 2.0;
    let mut left = peak_index;
    while left > 0 && magnitude.intensities[left - 1] > half {
        left -= 1;
    }
    let mut right = peak_index;
    while right + 1 < magnitude.intensities.len() && magnitude.intensities[right + 1] > half {
        right += 1;
    }
    if right <= left {
        return Err(RSpinError::InvalidSpectrum {
            message: "matched_filter_em could not measure a peak FWHM".to_owned(),
        });
    }
    let fwhm_axis = (magnitude.x.values[right] - magnitude.x.values[left]).abs();
    let fwhm_hz = match magnitude.x.unit {
        Unit::Hertz => fwhm_axis,
        Unit::Ppm => match magnitude.metadata.frequency_mhz {
            Some(freq_mhz) if freq_mhz.is_finite() && freq_mhz.abs() > 0.0 => {
                fwhm_axis * freq_mhz.abs()
            }
            _ => {
                return Err(RSpinError::InvalidSpectrum {
                    message: "matched_filter_em needs metadata.frequency_mhz for ppm axes"
                        .to_owned(),
                });
            }
        },
        _ => {
            return Err(RSpinError::InvalidSpectrum {
                message: "matched_filter_em produced an unsupported frequency axis unit".to_owned(),
            });
        }
    };
    if !fwhm_hz.is_finite() || fwhm_hz <= 0.0 {
        return Err(RSpinError::InvalidSpectrum {
            message: "matched_filter_em produced a non-positive FWHM".to_owned(),
        });
    }
    // The magnitude spectrum of a Lorentzian decays as
    // 1/sqrt(α² + (2πΔ)²), whose FWHM in Hz equals √3·LB while the
    // absorption-mode (phased real) Lorentzian has FWHM = LB. The
    // SNR-optimal exponential decay matches LB, so divide out the √3
    // factor we picked up by measuring on the magnitude spectrum.
    let lb_hz = fwhm_hz / 3.0_f64.sqrt();
    Ok(ExponentialApodization::new(lb_hz, dwell))
}

/// Applies convolution-difference apodization
/// (Campbell, Dobson, Williams, Xavier 1973).
///
/// `w[i] = exp(-π · LB1 · i · dt) - k · exp(-π · LB2 · i · dt)`. Choosing
/// a narrow `LB1` and a broader `LB2` subtracts the broad component
/// from the spectrum and is useful for paramagnetic, solid-state, or
/// broad-line cleanup.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConvolutionDifferenceApodization {
    /// Narrow line broadening in hertz (≥ 0).
    pub narrow_line_broadening_hz: f64,
    /// Broad line broadening in hertz (≥ 0).
    pub broad_line_broadening_hz: f64,
    /// Mixing coefficient `k` for the broad component, in `[0, 1]`.
    pub mixing: f64,
    /// Dwell time in seconds (> 0).
    pub dwell_time_s: f64,
}

impl ConvolutionDifferenceApodization {
    /// Creates a convolution-difference apodization step.
    #[must_use]
    pub fn new(
        narrow_line_broadening_hz: f64,
        broad_line_broadening_hz: f64,
        mixing: f64,
        dwell_time_s: f64,
    ) -> Self {
        Self {
            narrow_line_broadening_hz,
            broad_line_broadening_hz,
            mixing,
            dwell_time_s,
        }
    }
}

impl ProcessingStep<Spectrum1D> for ConvolutionDifferenceApodization {
    fn apply(&self, spectrum: &Spectrum1D) -> Result<Spectrum1D> {
        convolution_difference_apodization(
            spectrum,
            self.narrow_line_broadening_hz,
            self.broad_line_broadening_hz,
            self.mixing,
            self.dwell_time_s,
        )
    }
}

/// Applies trapezoidal apodization.
///
/// The window ramps linearly from 0 up to 1 across the leading
/// `rise_end_fraction` of the FID, stays at 1 between
/// `rise_end_fraction` and `fall_start_fraction`, then ramps back down
/// to 0 across the trailing portion. Both parameters lie in `[0, 1]`
/// with `rise_end_fraction <= fall_start_fraction`. Setting
/// `rise_end_fraction = 0` skips the ramp-in; `fall_start_fraction = 1`
/// skips the ramp-out.
///
/// The trapezoidal window is the conventional companion to forward
/// linear prediction: the LP-extrapolated tail can be damped smoothly
/// to zero by lowering `fall_start_fraction`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct TrapezoidalApodization {
    /// Fraction of the FID where the linear ramp-up reaches 1, in `[0, 1]`.
    pub rise_end_fraction: f64,
    /// Fraction of the FID where the linear ramp-down begins, in `[0, 1]`.
    pub fall_start_fraction: f64,
}

impl TrapezoidalApodization {
    /// Creates a trapezoidal apodization step.
    #[must_use]
    pub fn new(rise_end_fraction: f64, fall_start_fraction: f64) -> Self {
        Self {
            rise_end_fraction,
            fall_start_fraction,
        }
    }

    /// Creates a half-trapezoid that only ramps down at the tail.
    #[must_use]
    pub fn fall_only(fall_start_fraction: f64) -> Self {
        Self {
            rise_end_fraction: 0.0,
            fall_start_fraction,
        }
    }
}

impl ProcessingStep<Spectrum1D> for TrapezoidalApodization {
    fn apply(&self, spectrum: &Spectrum1D) -> Result<Spectrum1D> {
        trapezoidal_apodization(spectrum, self.rise_end_fraction, self.fall_start_fraction)
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

/// Applies Lorentz-to-Gauss (resolution-enhancement) apodization.
///
/// See [`LorentzToGaussApodization`] for the math. The weight is applied
/// to both the real and imaginary channels.
///
/// # Errors
///
/// Returns an error when either broadening is negative, the dwell time is
/// not positive, the shift is outside `[0, 1]`, any parameter is non-finite,
/// or the point count is too large for checked numeric conversion.
pub fn lorentz_to_gauss_apodization(
    spectrum: &Spectrum1D,
    lorentz_to_undo_hz: f64,
    gauss_fwhm_hz: f64,
    gauss_shift: f64,
    dwell_time_s: f64,
) -> Result<Spectrum1D> {
    let weights = lorentz_to_gauss_weights(
        spectrum.len(),
        lorentz_to_undo_hz,
        gauss_fwhm_hz,
        gauss_shift,
        dwell_time_s,
        "Lorentz-to-Gauss apodization",
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
        ProcessingRecord::new("lorentz_to_gauss_apodization").with_details(format!(
            "lorentz_to_undo_hz={lorentz_to_undo_hz},gauss_fwhm_hz={gauss_fwhm_hz},gauss_shift={gauss_shift},dwell_time_s={dwell_time_s}"
        )),
    ))
}

/// Applies convolution-difference apodization to real and imaginary channels.
///
/// See [`ConvolutionDifferenceApodization`] for the math.
///
/// # Errors
///
/// Returns an error when either line broadening is negative, the mixing
/// is outside `[0, 1]`, the dwell time is not positive, any parameter
/// is non-finite, or the point count is too large for checked numeric
/// conversion.
pub fn convolution_difference_apodization(
    spectrum: &Spectrum1D,
    narrow_line_broadening_hz: f64,
    broad_line_broadening_hz: f64,
    mixing: f64,
    dwell_time_s: f64,
) -> Result<Spectrum1D> {
    let weights = convolution_difference_weights(
        spectrum.len(),
        narrow_line_broadening_hz,
        broad_line_broadening_hz,
        mixing,
        dwell_time_s,
        "convolution-difference apodization",
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
        ProcessingRecord::new("convolution_difference_apodization").with_details(format!(
            "narrow_line_broadening_hz={narrow_line_broadening_hz},broad_line_broadening_hz={broad_line_broadening_hz},mixing={mixing},dwell_time_s={dwell_time_s}"
        )),
    ))
}

/// Applies Bruker-style two-parameter Gaussian apodization.
///
/// See [`GaussMultiplyBrukerApodization`] for the math.
///
/// # Errors
///
/// Returns an error when `gauss_position_fraction` is outside `[0, 1]`,
/// dwell time is not positive, any parameter is non-finite, or the
/// point count is too large for checked numeric conversion.
pub fn gauss_multiply_bruker_apodization(
    spectrum: &Spectrum1D,
    line_broadening_hz: f64,
    gauss_position_fraction: f64,
    dwell_time_s: f64,
) -> Result<Spectrum1D> {
    let weights = gauss_multiply_bruker_weights(
        spectrum.len(),
        line_broadening_hz,
        gauss_position_fraction,
        dwell_time_s,
        "Bruker GMB apodization",
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
        ProcessingRecord::new("gauss_multiply_bruker_apodization").with_details(format!(
            "line_broadening_hz={line_broadening_hz},gauss_position_fraction={gauss_position_fraction},dwell_time_s={dwell_time_s}"
        )),
    ))
}

/// Applies TRAF apodization to real and imaginary channels.
///
/// See [`TrafApodization`] for the math.
///
/// # Errors
///
/// Returns an error when line broadening is negative, dwell time is not
/// positive, any parameter is non-finite, or the point count is too
/// large for checked numeric conversion.
pub fn traf_apodization(
    spectrum: &Spectrum1D,
    line_broadening_hz: f64,
    dwell_time_s: f64,
) -> Result<Spectrum1D> {
    let weights = traf_weights(
        spectrum.len(),
        line_broadening_hz,
        dwell_time_s,
        "TRAF apodization",
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

    Ok(
        processed.with_processing_record(ProcessingRecord::new("traf_apodization").with_details(
            format!("line_broadening_hz={line_broadening_hz},dwell_time_s={dwell_time_s}"),
        )),
    )
}

/// Applies trapezoidal apodization.
///
/// See [`TrapezoidalApodization`] for the window definition.
///
/// # Errors
///
/// Returns an error when either fraction is outside `[0, 1]`,
/// `rise_end_fraction > fall_start_fraction`, or either parameter is
/// non-finite.
pub fn trapezoidal_apodization(
    spectrum: &Spectrum1D,
    rise_end_fraction: f64,
    fall_start_fraction: f64,
) -> Result<Spectrum1D> {
    let weights = trapezoidal_weights(
        spectrum.len(),
        rise_end_fraction,
        fall_start_fraction,
        "trapezoidal apodization",
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
        ProcessingRecord::new("trapezoidal_apodization").with_details(format!(
            "rise_end_fraction={rise_end_fraction},fall_start_fraction={fall_start_fraction}"
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
    // Float-to-integer casts saturate on overflow (Rust ≥ 1.45), so
    // `samples.trunc() as usize` clamps cleanly; we still cap at `len` to
    // keep `rotate_left` in bounds.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let integer_shift = (samples.trunc() as usize).min(len);
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
    let half_f = safe_usize_to_f64(half, "spectrum index")?;
    let scale = sweep_width_hz / n_f;
    let mut hz_values = Vec::with_capacity(len);
    for index in 0..len {
        let index_f = safe_usize_to_f64(index, "spectrum index")?;
        hz_values.push((index_f - half_f) * scale);
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

pub(crate) fn complex_buffer(spectrum: &Spectrum1D) -> Vec<Complex<f64>> {
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

fn convolution_difference_weights(
    len: usize,
    narrow_line_broadening_hz: f64,
    broad_line_broadening_hz: f64,
    mixing: f64,
    dwell_time_s: f64,
    context: &'static str,
) -> Result<Vec<f64>> {
    ensure_non_negative("narrow_line_broadening_hz", narrow_line_broadening_hz)?;
    ensure_non_negative("broad_line_broadening_hz", broad_line_broadening_hz)?;
    ensure_finite("mixing", mixing)?;
    if !(0.0..=1.0).contains(&mixing) {
        return Err(RSpinError::InvalidSpectrum {
            message: "mixing must be between 0 and 1".to_owned(),
        });
    }
    ensure_positive("dwell_time_s", dwell_time_s)?;

    let narrow_scale = -PI * narrow_line_broadening_hz * dwell_time_s;
    let broad_scale = -PI * broad_line_broadening_hz * dwell_time_s;
    (0..len)
        .map(|index| {
            let index_f =
                f64::from(
                    u32::try_from(index).map_err(|_| RSpinError::InvalidSpectrum {
                        message: format!("{context} input is too large"),
                    })?,
                );
            let narrow = (narrow_scale * index_f).exp();
            let broad = (broad_scale * index_f).exp();
            Ok(narrow - mixing * broad)
        })
        .collect()
}

fn gauss_multiply_bruker_weights(
    len: usize,
    line_broadening_hz: f64,
    gauss_position_fraction: f64,
    dwell_time_s: f64,
    context: &'static str,
) -> Result<Vec<f64>> {
    ensure_finite("line_broadening_hz", line_broadening_hz)?;
    ensure_finite("gauss_position_fraction", gauss_position_fraction)?;
    if !(0.0..=1.0).contains(&gauss_position_fraction) {
        return Err(RSpinError::InvalidSpectrum {
            message: "gauss_position_fraction must be between 0 and 1".to_owned(),
        });
    }
    ensure_positive("dwell_time_s", dwell_time_s)?;

    let last_index = len.saturating_sub(1);
    let last_index_f = if last_index == 0 {
        0.0
    } else {
        f64::from(
            u32::try_from(last_index).map_err(|_| RSpinError::InvalidSpectrum {
                message: format!("{context} input is too large"),
            })?,
        )
    };
    // Bruker procs convention: F(t) = exp(-a·t - b·t²) with a = π·LB and
    // b = -a / (2·GB·AQ) where AQ = (N-1)·dt. Working in discretised
    // index form F[i] = exp(-a'·i - b'·i²) the per-step coefficients are
    // a' = π·LB·dt and b' = -a' / (2·GB·(N-1)), which gives a Gaussian
    // peak at i = GB·(N-1) for LB<0, GB>0 (the resolution-enhancement
    // case). When GB = 0 the formula reduces to plain exponential.
    let a = PI * line_broadening_hz * dwell_time_s;
    let b = if gauss_position_fraction > 0.0 && last_index_f > 0.0 {
        -a / (2.0 * gauss_position_fraction * last_index_f)
    } else {
        0.0
    };
    (0..len)
        .map(|index| {
            let index_f =
                f64::from(
                    u32::try_from(index).map_err(|_| RSpinError::InvalidSpectrum {
                        message: format!("{context} input is too large"),
                    })?,
                );
            Ok((-a * index_f - b * index_f * index_f).exp())
        })
        .collect()
}

fn traf_weights(
    len: usize,
    line_broadening_hz: f64,
    dwell_time_s: f64,
    context: &'static str,
) -> Result<Vec<f64>> {
    ensure_non_negative("line_broadening_hz", line_broadening_hz)?;
    ensure_positive("dwell_time_s", dwell_time_s)?;
    let last_index = len.saturating_sub(1);
    let last_index_f = if last_index == 0 {
        0.0
    } else {
        f64::from(
            u32::try_from(last_index).map_err(|_| RSpinError::InvalidSpectrum {
                message: format!("{context} input is too large"),
            })?,
        )
    };
    let scale = -PI * line_broadening_hz * dwell_time_s;
    (0..len)
        .map(|index| {
            let index_f =
                f64::from(
                    u32::try_from(index).map_err(|_| RSpinError::InvalidSpectrum {
                        message: format!("{context} input is too large"),
                    })?,
                );
            let e_decay = (scale * index_f).exp();
            let r_decay = (scale * (last_index_f - index_f)).exp();
            let denominator = e_decay.powi(3) + r_decay.powi(3);
            let weight = if denominator <= 0.0 {
                0.0
            } else {
                e_decay.powi(2) / denominator
            };
            Ok(weight)
        })
        .collect()
}

fn trapezoidal_weights(
    len: usize,
    rise_end_fraction: f64,
    fall_start_fraction: f64,
    context: &'static str,
) -> Result<Vec<f64>> {
    ensure_finite("rise_end_fraction", rise_end_fraction)?;
    ensure_finite("fall_start_fraction", fall_start_fraction)?;
    if !(0.0..=1.0).contains(&rise_end_fraction) {
        return Err(RSpinError::InvalidSpectrum {
            message: "rise_end_fraction must be between 0 and 1".to_owned(),
        });
    }
    if !(0.0..=1.0).contains(&fall_start_fraction) {
        return Err(RSpinError::InvalidSpectrum {
            message: "fall_start_fraction must be between 0 and 1".to_owned(),
        });
    }
    if rise_end_fraction > fall_start_fraction {
        return Err(RSpinError::InvalidSpectrum {
            message: "rise_end_fraction must not exceed fall_start_fraction".to_owned(),
        });
    }

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
            let index_f =
                f64::from(
                    u32::try_from(index).map_err(|_| RSpinError::InvalidSpectrum {
                        message: format!("{context} input is too large"),
                    })?,
                );
            let fraction = if denominator == 0.0 {
                0.0
            } else {
                index_f / denominator
            };
            let weight = if fraction < rise_end_fraction {
                if rise_end_fraction <= 0.0 {
                    1.0
                } else {
                    fraction / rise_end_fraction
                }
            } else if fraction > fall_start_fraction {
                if fall_start_fraction >= 1.0 {
                    1.0
                } else {
                    (1.0 - fraction) / (1.0 - fall_start_fraction)
                }
            } else {
                1.0
            };
            Ok(weight.max(0.0))
        })
        .collect()
}

fn lorentz_to_gauss_weights(
    len: usize,
    lorentz_to_undo_hz: f64,
    gauss_fwhm_hz: f64,
    gauss_shift: f64,
    dwell_time_s: f64,
    context: &'static str,
) -> Result<Vec<f64>> {
    ensure_non_negative("lorentz_to_undo_hz", lorentz_to_undo_hz)?;
    ensure_non_negative("gauss_fwhm_hz", gauss_fwhm_hz)?;
    ensure_finite("gauss_shift", gauss_shift)?;
    if !(0.0..=1.0).contains(&gauss_shift) {
        return Err(RSpinError::InvalidSpectrum {
            message: "gauss_shift must be between 0 and 1".to_owned(),
        });
    }
    ensure_positive("dwell_time_s", dwell_time_s)?;

    let last_index = len.saturating_sub(1);
    let last_index_f = if last_index == 0 {
        0.0
    } else {
        f64::from(
            u32::try_from(last_index).map_err(|_| RSpinError::InvalidSpectrum {
                message: format!("{context} input is too large"),
            })?,
        )
    };
    let t_max = last_index_f * dwell_time_s;
    let lorentz_scale = PI * lorentz_to_undo_hz * dwell_time_s;
    let gauss_scale = PI * gauss_fwhm_hz;
    let gauss_norm = 4.0 * LN_2;
    let center_time = gauss_shift * t_max;
    (0..len)
        .map(|index| {
            let index_f =
                f64::from(
                    u32::try_from(index).map_err(|_| RSpinError::InvalidSpectrum {
                        message: format!("{context} input is too large"),
                    })?,
                );
            let lorentz_part = lorentz_scale * index_f;
            let gauss_offset = gauss_scale * (index_f * dwell_time_s - center_time);
            let gauss_part = -(gauss_offset * gauss_offset) / gauss_norm;
            Ok((lorentz_part + gauss_part).exp())
        })
        .collect()
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
