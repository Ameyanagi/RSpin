//! Complex-domain two-dimensional transforms.

use rspin_core::{ProcessingRecord, RSpinError, Result, Spectrum2D};
use rustfft::{FftPlanner, num_complex::Complex};

use crate::{FftDirection, ProcessingStep};

/// Two-dimensional FFT processing step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fft2D {
    /// Transform direction.
    pub direction: FftDirection,
}

impl ProcessingStep<Spectrum2D> for Fft2D {
    fn apply(&self, spectrum: &Spectrum2D) -> Result<Spectrum2D> {
        fft_2d(spectrum, self.direction)
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

#[cfg(test)]
mod tests;
