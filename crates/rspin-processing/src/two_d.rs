//! Two-dimensional processing and extraction operations.

use rspin_core::{Axis, ProcessingRecord, RSpinError, Result, Spectrum1D, Spectrum2D};

use crate::ProcessingStep;

/// Projection reduction mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectionMode {
    /// Sum all values along the collapsed axis.
    Sum,
    /// Average all values along the collapsed axis.
    Mean,
    /// Maximum value along the collapsed axis.
    Max,
    /// Minimum value along the collapsed axis.
    Min,
    /// Value with maximum absolute magnitude along the collapsed axis.
    MaxAbs,
}

/// Multiplies all 2D intensities by a scalar.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Scale2D {
    /// Multiplicative factor.
    pub factor: f64,
}

impl ProcessingStep<Spectrum2D> for Scale2D {
    fn apply(&self, spectrum: &Spectrum2D) -> Result<Spectrum2D> {
        scale_2d(spectrum, self.factor)
    }
}

/// Normalizes 2D intensities by their maximum absolute value.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Normalize2DMaxAbs;

impl ProcessingStep<Spectrum2D> for Normalize2DMaxAbs {
    fn apply(&self, spectrum: &Spectrum2D) -> Result<Spectrum2D> {
        normalize_2d_max_abs(spectrum)
    }
}

/// Multiplies all 2D intensities by `factor`.
///
/// # Errors
///
/// Returns an error when `factor` is not finite.
pub fn scale_2d(spectrum: &Spectrum2D, factor: f64) -> Result<Spectrum2D> {
    ensure_finite("scale factor", factor)?;
    let mut processed = spectrum.clone();
    processed.z = processed
        .z
        .into_iter()
        .map(|value| value * factor)
        .collect();
    if let Some(imaginary) = processed.imaginary.take() {
        processed.imaginary = Some(imaginary.into_iter().map(|value| value * factor).collect());
    }
    Ok(recorded_2d(
        processed,
        ProcessingRecord::new("scale_2d").with_details(format!("factor={factor}")),
    ))
}

/// Normalizes 2D intensities by the maximum absolute value.
///
/// # Errors
///
/// Returns an error when every value is zero.
pub fn normalize_2d_max_abs(spectrum: &Spectrum2D) -> Result<Spectrum2D> {
    let max_abs = spectrum
        .z
        .iter()
        .map(|value| value.abs())
        .fold(0.0_f64, f64::max);
    if max_abs == 0.0 {
        return Err(RSpinError::InvalidSpectrum {
            message: "cannot normalize an all-zero 2D spectrum".to_owned(),
        });
    }
    let mut processed = scale_2d(spectrum, 1.0 / max_abs)?;
    let _ = processed.processing.pop();
    Ok(recorded_2d(
        processed,
        ProcessingRecord::new("normalize_2d_max_abs"),
    ))
}

/// Projects a 2D spectrum onto the x axis.
///
/// # Errors
///
/// Returns an error when dimensions cannot be reduced safely.
pub fn project_x(spectrum: &Spectrum2D, mode: ProjectionMode) -> Result<Spectrum1D> {
    let (width, height) = spectrum.shape();
    let divisor = divisor(height)?;
    let mut values = Vec::with_capacity(width);
    let mut imaginary_values = spectrum
        .imaginary
        .as_ref()
        .map(|_| Vec::with_capacity(width));
    for x_index in 0..width {
        let mut column = Vec::with_capacity(height);
        let mut imaginary_column = spectrum
            .imaginary
            .as_ref()
            .map(|_| Vec::with_capacity(height));
        for y_index in 0..height {
            column.push(spectrum.z[y_index * width + x_index]);
            if let (Some(source), Some(target)) = (&spectrum.imaginary, &mut imaginary_column) {
                target.push(source[y_index * width + x_index]);
            }
        }
        let reduced = reduce_complex(&column, imaginary_column.as_deref(), mode, divisor);
        values.push(reduced.real);
        if let (Some(value), Some(target)) = (reduced.imaginary, &mut imaginary_values) {
            target.push(value);
        }
    }
    derived_1d(
        spectrum,
        spectrum.x.clone(),
        values,
        imaginary_values,
        ProcessingRecord::new("project_x").with_details(format!("mode={mode:?}")),
    )
}

/// Projects a 2D spectrum onto the y axis.
///
/// # Errors
///
/// Returns an error when dimensions cannot be reduced safely.
pub fn project_y(spectrum: &Spectrum2D, mode: ProjectionMode) -> Result<Spectrum1D> {
    let (width, height) = spectrum.shape();
    let divisor = divisor(width)?;
    let mut values = Vec::with_capacity(height);
    let mut imaginary_values = spectrum
        .imaginary
        .as_ref()
        .map(|_| Vec::with_capacity(height));
    for y_index in 0..height {
        let row_start = y_index * width;
        let row_end = row_start + width;
        let imaginary_row = spectrum
            .imaginary
            .as_ref()
            .map(|imaginary| &imaginary[row_start..row_end]);
        let reduced = reduce_complex(
            &spectrum.z[row_start..row_end],
            imaginary_row,
            mode,
            divisor,
        );
        values.push(reduced.real);
        if let (Some(value), Some(target)) = (reduced.imaginary, &mut imaginary_values) {
            target.push(value);
        }
    }
    derived_1d(
        spectrum,
        spectrum.y.clone(),
        values,
        imaginary_values,
        ProcessingRecord::new("project_y").with_details(format!("mode={mode:?}")),
    )
}

/// Extracts the row at `y_index` as a one-dimensional spectrum over x.
///
/// # Errors
///
/// Returns an error when `y_index` is out of bounds.
pub fn slice_x_at_y_index(spectrum: &Spectrum2D, y_index: usize) -> Result<Spectrum1D> {
    slice_x_at_y_index_with_record(
        spectrum,
        y_index,
        ProcessingRecord::new("slice_x_at_y_index").with_details(format!("y_index={y_index}")),
    )
}

/// Extracts the row nearest `y` as a one-dimensional spectrum over x.
///
/// # Errors
///
/// Returns an error when `y` is not finite.
pub fn slice_x_at_y(spectrum: &Spectrum2D, y: f64) -> Result<Spectrum1D> {
    let y_index = nearest_axis_index(&spectrum.y.values, y, "y coordinate")?;
    let selected_y = spectrum.y.values[y_index];
    slice_x_at_y_index_with_record(
        spectrum,
        y_index,
        ProcessingRecord::new("slice_x_at_y")
            .with_details(format!("y={y},y_index={y_index},selected_y={selected_y}")),
    )
}

fn slice_x_at_y_index_with_record(
    spectrum: &Spectrum2D,
    y_index: usize,
    record: ProcessingRecord,
) -> Result<Spectrum1D> {
    let (width, height) = spectrum.shape();
    if y_index >= height {
        return Err(RSpinError::InvalidSpectrum {
            message: format!("y index {y_index} is outside height {height}"),
        });
    }
    let row_start = y_index * width;
    let row_end = row_start + width;
    derived_1d(
        spectrum,
        spectrum.x.clone(),
        spectrum.z[row_start..row_end].to_vec(),
        spectrum
            .imaginary
            .as_ref()
            .map(|imaginary| imaginary[row_start..row_end].to_vec()),
        record,
    )
}

/// Extracts the column at `x_index` as a one-dimensional spectrum over y.
///
/// # Errors
///
/// Returns an error when `x_index` is out of bounds.
pub fn slice_y_at_x_index(spectrum: &Spectrum2D, x_index: usize) -> Result<Spectrum1D> {
    slice_y_at_x_index_with_record(
        spectrum,
        x_index,
        ProcessingRecord::new("slice_y_at_x_index").with_details(format!("x_index={x_index}")),
    )
}

/// Extracts the column nearest `x` as a one-dimensional spectrum over y.
///
/// # Errors
///
/// Returns an error when `x` is not finite.
pub fn slice_y_at_x(spectrum: &Spectrum2D, x: f64) -> Result<Spectrum1D> {
    let x_index = nearest_axis_index(&spectrum.x.values, x, "x coordinate")?;
    let selected_x = spectrum.x.values[x_index];
    slice_y_at_x_index_with_record(
        spectrum,
        x_index,
        ProcessingRecord::new("slice_y_at_x")
            .with_details(format!("x={x},x_index={x_index},selected_x={selected_x}")),
    )
}

fn slice_y_at_x_index_with_record(
    spectrum: &Spectrum2D,
    x_index: usize,
    record: ProcessingRecord,
) -> Result<Spectrum1D> {
    let (width, height) = spectrum.shape();
    if x_index >= width {
        return Err(RSpinError::InvalidSpectrum {
            message: format!("x index {x_index} is outside width {width}"),
        });
    }
    let mut values = Vec::with_capacity(height);
    let mut imaginary_values = spectrum
        .imaginary
        .as_ref()
        .map(|_| Vec::with_capacity(height));
    for y_index in 0..height {
        values.push(spectrum.z[y_index * width + x_index]);
        if let (Some(source), Some(target)) = (&spectrum.imaginary, &mut imaginary_values) {
            target.push(source[y_index * width + x_index]);
        }
    }
    derived_1d(
        spectrum,
        spectrum.y.clone(),
        values,
        imaginary_values,
        record,
    )
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ReducedComplex {
    real: f64,
    imaginary: Option<f64>,
}

fn reduce_complex(
    values: &[f64],
    imaginary: Option<&[f64]>,
    mode: ProjectionMode,
    divisor: f64,
) -> ReducedComplex {
    match mode {
        ProjectionMode::Sum => ReducedComplex {
            real: values.iter().sum(),
            imaginary: imaginary.map(|values| values.iter().sum()),
        },
        ProjectionMode::Mean => ReducedComplex {
            real: values.iter().sum::<f64>() / divisor,
            imaginary: imaginary.map(|values| values.iter().sum::<f64>() / divisor),
        },
        ProjectionMode::Max | ProjectionMode::Min | ProjectionMode::MaxAbs => {
            let index = selected_index(values, mode);
            ReducedComplex {
                real: values[index],
                imaginary: imaginary.map(|values| values[index]),
            }
        }
    }
}

fn selected_index(values: &[f64], mode: ProjectionMode) -> usize {
    let mut selected = 0;
    for (index, value) in values.iter().copied().enumerate().skip(1) {
        let current = values[selected];
        let better = match mode {
            ProjectionMode::Max => value > current,
            ProjectionMode::Min => value < current,
            ProjectionMode::MaxAbs => value.abs() > current.abs(),
            ProjectionMode::Sum | ProjectionMode::Mean => false,
        };
        if better {
            selected = index;
        }
    }
    selected
}

fn divisor(value: usize) -> Result<f64> {
    let value = u32::try_from(value).map_err(|_| RSpinError::InvalidSpectrum {
        message: "2D dimension is too large to reduce".to_owned(),
    })?;
    Ok(f64::from(value))
}

fn nearest_axis_index(values: &[f64], coordinate: f64, field: &'static str) -> Result<usize> {
    ensure_finite(field, coordinate)?;
    let first = match values.first() {
        Some(value) => *value,
        None => {
            return Err(RSpinError::InvalidAxis {
                message: "axis must contain at least one point".to_owned(),
            });
        }
    };
    let mut nearest_index = 0;
    let mut nearest_distance = (first - coordinate).abs();
    for (index, value) in values.iter().copied().enumerate().skip(1) {
        let distance = (value - coordinate).abs();
        if distance < nearest_distance {
            nearest_index = index;
            nearest_distance = distance;
        }
    }
    Ok(nearest_index)
}

fn derived_1d(
    spectrum: &Spectrum2D,
    axis: Axis,
    values: Vec<f64>,
    imaginary: Option<Vec<f64>>,
    record: ProcessingRecord,
) -> Result<Spectrum1D> {
    let mut derived = Spectrum1D::new_complex(axis, values, imaginary, spectrum.metadata.clone())?;
    derived.processing.clone_from(&spectrum.processing);
    Ok(derived.with_processing_record(record))
}

fn recorded_2d(spectrum: Spectrum2D, record: ProcessingRecord) -> Spectrum2D {
    spectrum.with_processing_record(record)
}

fn ensure_finite(field: &'static str, value: f64) -> Result<()> {
    if !value.is_finite() {
        return Err(RSpinError::NonFinite { field });
    }
    Ok(())
}

#[cfg(test)]
mod tests;
