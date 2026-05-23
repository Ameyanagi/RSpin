//! Two-dimensional zero filling.

use rspin_core::{Axis, ProcessingRecord, RSpinError, Result, Spectrum2D};

use crate::ProcessingStep;

/// Extends a two-dimensional spectrum with trailing zeroes in each dimension.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZeroFill2D {
    /// Desired x-axis point count.
    pub target_x_len: usize,
    /// Desired y-axis point count.
    pub target_y_len: usize,
}

impl ProcessingStep<Spectrum2D> for ZeroFill2D {
    fn apply(&self, spectrum: &Spectrum2D) -> Result<Spectrum2D> {
        zero_fill_2d(spectrum, self.target_x_len, self.target_y_len)
    }
}

/// Extends a 2D spectrum to `target_x_len * target_y_len` points by padding zeroes.
///
/// Existing row-major values are kept at the same x/y indices. New x columns
/// and y rows are appended, and axis values are extended using the final
/// observed spacing in each dimension, or `1.0` when an axis has one point.
///
/// # Errors
///
/// Returns an error when either target length is smaller than the current
/// dimension or the target matrix size overflows.
pub fn zero_fill_2d(
    spectrum: &Spectrum2D,
    target_width: usize,
    target_height: usize,
) -> Result<Spectrum2D> {
    let (width, height) = spectrum.shape();
    if target_width < width {
        return Err(RSpinError::InvalidSpectrum {
            message: format!(
                "2D zero-fill x target {target_width} is smaller than current width {width}"
            ),
        });
    }
    if target_height < height {
        return Err(RSpinError::InvalidSpectrum {
            message: format!(
                "2D zero-fill y target {target_height} is smaller than current height {height}"
            ),
        });
    }
    let target_len =
        target_width
            .checked_mul(target_height)
            .ok_or_else(|| RSpinError::InvalidSpectrum {
                message: "2D zero-fill target size overflow".to_owned(),
            })?;

    let mut z = vec![0.0; target_len];
    for y_index in 0..height {
        let source_start = y_index * width;
        let target_start = y_index * target_width;
        let source_end = source_start + width;
        let target_end = target_start + width;
        z[target_start..target_end].copy_from_slice(&spectrum.z[source_start..source_end]);
    }

    let x = extend_axis(&spectrum.x, target_width)?;
    let y = extend_axis(&spectrum.y, target_height)?;
    let mut processed = Spectrum2D::new(x, y, z, spectrum.metadata.clone())?;
    processed.processing.clone_from(&spectrum.processing);
    Ok(
        processed.with_processing_record(ProcessingRecord::new("zero_fill_2d").with_details(
            format!("target_x_len={target_width},target_y_len={target_height}"),
        )),
    )
}

fn extend_axis(axis: &Axis, target_len: usize) -> Result<Axis> {
    if target_len == axis.len() {
        return Ok(axis.clone());
    }
    let mut values = axis.values.clone();
    let step = axis_step(axis);
    let mut next = axis_last(axis)? + step;
    values.reserve(target_len - axis.len());
    while values.len() < target_len {
        values.push(next);
        next += step;
    }
    Axis::new(axis.label.clone(), axis.unit, values)
}

fn axis_last(axis: &Axis) -> Result<f64> {
    axis.values
        .last()
        .copied()
        .ok_or_else(|| RSpinError::InvalidAxis {
            message: "missing axis values".to_owned(),
        })
}

fn axis_step(axis: &Axis) -> f64 {
    let values = &axis.values;
    match values.as_slice() {
        [.., previous, last] => last - previous,
        [_] | [] => 1.0,
    }
}

#[cfg(test)]
mod tests;
