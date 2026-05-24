//! Complex-domain two-dimensional transforms.

use rspin_core::{ProcessingRecord, RSpinError, Result, Spectrum2D};
use rustfft::{FftPlanner, num_complex::Complex};
use serde::{Deserialize, Serialize};

use crate::{FftDirection, ProcessingStep};

/// Two-dimensional FFT processing step.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fft2D {
    /// Transform direction.
    pub direction: FftDirection,
}

impl Fft2D {
    /// Creates a two-dimensional FFT step.
    #[must_use]
    pub fn new(direction: FftDirection) -> Self {
        Self { direction }
    }

    /// Creates a forward 2D FFT step.
    #[must_use]
    pub fn forward() -> Self {
        Self::new(FftDirection::Forward)
    }

    /// Creates an inverse 2D FFT step.
    #[must_use]
    pub fn inverse() -> Self {
        Self::new(FftDirection::Inverse)
    }
}

impl ProcessingStep<Spectrum2D> for Fft2D {
    fn apply(&self, spectrum: &Spectrum2D) -> Result<Spectrum2D> {
        fft_2d(spectrum, self.direction)
    }
}

/// Manual x/y phase correction for a two-dimensional spectrum.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PhaseCorrection2D {
    /// X-dimension zero-order phase in degrees.
    pub x_zero_order_deg: f64,
    /// X-dimension first-order phase in degrees.
    pub x_first_order_deg: f64,
    /// X-dimension pivot position as a fraction of the index range.
    pub x_pivot_fraction: f64,
    /// Y-dimension zero-order phase in degrees.
    pub y_zero_order_deg: f64,
    /// Y-dimension first-order phase in degrees.
    pub y_first_order_deg: f64,
    /// Y-dimension pivot position as a fraction of the index range.
    pub y_pivot_fraction: f64,
}

impl Default for PhaseCorrection2D {
    fn default() -> Self {
        Self {
            x_zero_order_deg: 0.0,
            x_first_order_deg: 0.0,
            x_pivot_fraction: 0.5,
            y_zero_order_deg: 0.0,
            y_first_order_deg: 0.0,
            y_pivot_fraction: 0.5,
        }
    }
}

impl PhaseCorrection2D {
    /// Creates a no-op two-dimensional phase correction.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a correction with x-dimension phase parameters.
    #[must_use]
    pub fn x_phase(
        mut self,
        zero_order_deg: f64,
        first_order_deg: f64,
        pivot_fraction: f64,
    ) -> Self {
        self.x_zero_order_deg = zero_order_deg;
        self.x_first_order_deg = first_order_deg;
        self.x_pivot_fraction = pivot_fraction;
        self
    }

    /// Returns a correction with y-dimension phase parameters.
    #[must_use]
    pub fn y_phase(
        mut self,
        zero_order_deg: f64,
        first_order_deg: f64,
        pivot_fraction: f64,
    ) -> Self {
        self.y_zero_order_deg = zero_order_deg;
        self.y_first_order_deg = first_order_deg;
        self.y_pivot_fraction = pivot_fraction;
        self
    }

    fn validate(self) -> Result<()> {
        ensure_finite("x_zero_order_deg", self.x_zero_order_deg)?;
        ensure_finite("x_first_order_deg", self.x_first_order_deg)?;
        ensure_finite("y_zero_order_deg", self.y_zero_order_deg)?;
        ensure_finite("y_first_order_deg", self.y_first_order_deg)?;
        ensure_pivot("x_pivot_fraction", self.x_pivot_fraction)?;
        ensure_pivot("y_pivot_fraction", self.y_pivot_fraction)
    }
}

impl ProcessingStep<Spectrum2D> for PhaseCorrection2D {
    fn apply(&self, spectrum: &Spectrum2D) -> Result<Spectrum2D> {
        phase_correct_2d(spectrum, *self)
    }
}

/// Applies a separable two-dimensional FFT.
///
/// The transform is applied along x for every row, then along y for every
/// column. The inverse direction is normalized by `1 / (x.len() * y.len())`,
/// making `inverse(forward(spectrum))` recover the original values within
/// floating-point tolerance.
///
/// # Errors
///
/// Returns an error when the point count cannot be represented safely for
/// inverse normalization.
pub fn fft_2d(spectrum: &Spectrum2D, direction: FftDirection) -> Result<Spectrum2D> {
    let (width, height) = spectrum.shape();
    let mut buffer = complex_buffer(spectrum);
    let mut planner = FftPlanner::<f64>::new();

    let row_fft = match direction {
        FftDirection::Forward => planner.plan_fft_forward(width),
        FftDirection::Inverse => planner.plan_fft_inverse(width),
    };
    for row in buffer.chunks_exact_mut(width) {
        row_fft.process(row);
    }

    let column_fft = match direction {
        FftDirection::Forward => planner.plan_fft_forward(height),
        FftDirection::Inverse => planner.plan_fft_inverse(height),
    };
    for x_index in 0..width {
        let mut column = Vec::with_capacity(height);
        for y_index in 0..height {
            column.push(buffer[y_index * width + x_index]);
        }
        column_fft.process(&mut column);
        for (y_index, value) in column.into_iter().enumerate() {
            buffer[y_index * width + x_index] = value;
        }
    }

    if direction == FftDirection::Inverse {
        let len = u32::try_from(buffer.len()).map_err(|_| RSpinError::InvalidSpectrum {
            message: "2D spectrum is too large to normalize inverse FFT".to_owned(),
        })?;
        let scale = 1.0 / f64::from(len);
        for value in &mut buffer {
            *value *= scale;
        }
    }

    let z = buffer.iter().map(|value| value.re).collect();
    let imaginary = Some(buffer.iter().map(|value| value.im).collect());
    let mut processed = Spectrum2D::new_complex(
        spectrum.x.clone(),
        spectrum.y.clone(),
        z,
        imaginary,
        spectrum.metadata.clone(),
    )?;
    processed.processing.clone_from(&spectrum.processing);
    Ok(processed.with_processing_record(
        ProcessingRecord::new("fft_2d").with_details(format!("direction={direction:?}")),
    ))
}

/// Applies manual separable x/y phase correction to a two-dimensional spectrum.
///
/// The phase at point `(x, y)` is the x-dimension phase term plus the
/// y-dimension phase term. Each dimension term is `zero_order_deg +
/// first_order_deg * (fraction(index) - pivot_fraction)`, where
/// `fraction(index)` spans `0..=1` across that dimension. Real-only input is
/// treated as complex data with zero imaginary values, and the output always
/// contains an imaginary channel.
///
/// # Errors
///
/// Returns an error when phase parameters are non-finite, pivots are outside
/// `[0, 1]`, or a dimension is too large for safe conversion.
pub fn phase_correct_2d(
    spectrum: &Spectrum2D,
    correction: PhaseCorrection2D,
) -> Result<Spectrum2D> {
    correction.validate()?;
    let (width, height) = spectrum.shape();
    let x_denominator = index_denominator(width, "x phase correction")?;
    let y_denominator = index_denominator(height, "y phase correction")?;

    let mut z = Vec::with_capacity(spectrum.z.len());
    let mut imaginary = Vec::with_capacity(spectrum.z.len());
    for (index, value) in complex_buffer(spectrum).into_iter().enumerate() {
        let x_index = index % width;
        let y_index = index / width;
        let x_fraction = index_fraction(x_index, x_denominator, "x phase correction")?;
        let y_fraction = index_fraction(y_index, y_denominator, "y phase correction")?;
        let phase_deg = correction.x_zero_order_deg
            + correction.x_first_order_deg * (x_fraction - correction.x_pivot_fraction)
            + correction.y_zero_order_deg
            + correction.y_first_order_deg * (y_fraction - correction.y_pivot_fraction);
        let phase_rad = phase_deg.to_radians();
        let rotation = Complex::new(phase_rad.cos(), phase_rad.sin());
        let corrected = value * rotation;
        z.push(corrected.re);
        imaginary.push(corrected.im);
    }

    let mut processed = Spectrum2D::new_complex(
        spectrum.x.clone(),
        spectrum.y.clone(),
        z,
        Some(imaginary),
        spectrum.metadata.clone(),
    )?;
    processed.processing.clone_from(&spectrum.processing);
    Ok(processed.with_processing_record(
        ProcessingRecord::new("phase_correct_2d").with_details(format!(
            "x_zero_order_deg={},x_first_order_deg={},x_pivot_fraction={},y_zero_order_deg={},y_first_order_deg={},y_pivot_fraction={}",
            correction.x_zero_order_deg,
            correction.x_first_order_deg,
            correction.x_pivot_fraction,
            correction.y_zero_order_deg,
            correction.y_first_order_deg,
            correction.y_pivot_fraction,
        )),
    ))
}

fn complex_buffer(spectrum: &Spectrum2D) -> Vec<Complex<f64>> {
    match &spectrum.imaginary {
        Some(imaginary) => spectrum
            .z
            .iter()
            .zip(imaginary)
            .map(|(real, imag)| Complex::new(*real, *imag))
            .collect(),
        None => spectrum
            .z
            .iter()
            .map(|real| Complex::new(*real, 0.0))
            .collect(),
    }
}

fn index_denominator(len: usize, context: &'static str) -> Result<f64> {
    if len <= 1 {
        return Ok(0.0);
    }
    let denominator = u32::try_from(len - 1).map_err(|_| RSpinError::InvalidSpectrum {
        message: format!("2D spectrum is too large for {context}"),
    })?;
    Ok(f64::from(denominator))
}

fn index_fraction(index: usize, denominator: f64, context: &'static str) -> Result<f64> {
    if denominator == 0.0 {
        return Ok(0.0);
    }
    let index = u32::try_from(index).map_err(|_| RSpinError::InvalidSpectrum {
        message: format!("2D spectrum is too large for {context}"),
    })?;
    Ok(f64::from(index) / denominator)
}

fn ensure_pivot(field: &'static str, value: f64) -> Result<()> {
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        return Err(RSpinError::InvalidSpectrum {
            message: format!("{field} must be finite and between 0 and 1"),
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
