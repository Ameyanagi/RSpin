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
    for x_index in 0..width {
        let mut column = Vec::with_capacity(height);
        for y_index in 0..height {
            column.push(spectrum.z[y_index * width + x_index]);
        }
        values.push(reduce(&column, mode, divisor));
    }
    derived_1d(
        spectrum,
        spectrum.x.clone(),
        values,
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
    for y_index in 0..height {
        let row_start = y_index * width;
        let row_end = row_start + width;
        values.push(reduce(&spectrum.z[row_start..row_end], mode, divisor));
    }
    derived_1d(
        spectrum,
        spectrum.y.clone(),
        values,
        ProcessingRecord::new("project_y").with_details(format!("mode={mode:?}")),
    )
}

/// Extracts the row at `y_index` as a one-dimensional spectrum over x.
///
/// # Errors
///
/// Returns an error when `y_index` is out of bounds.
pub fn slice_x_at_y_index(spectrum: &Spectrum2D, y_index: usize) -> Result<Spectrum1D> {
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
        ProcessingRecord::new("slice_x_at_y_index").with_details(format!("y_index={y_index}")),
    )
}

/// Extracts the column at `x_index` as a one-dimensional spectrum over y.
///
/// # Errors
///
/// Returns an error when `x_index` is out of bounds.
pub fn slice_y_at_x_index(spectrum: &Spectrum2D, x_index: usize) -> Result<Spectrum1D> {
    let (width, height) = spectrum.shape();
    if x_index >= width {
        return Err(RSpinError::InvalidSpectrum {
            message: format!("x index {x_index} is outside width {width}"),
        });
    }
    let mut values = Vec::with_capacity(height);
    for y_index in 0..height {
        values.push(spectrum.z[y_index * width + x_index]);
    }
    derived_1d(
        spectrum,
        spectrum.y.clone(),
        values,
        ProcessingRecord::new("slice_y_at_x_index").with_details(format!("x_index={x_index}")),
    )
}

fn reduce(values: &[f64], mode: ProjectionMode, divisor: f64) -> f64 {
    match mode {
        ProjectionMode::Sum => values.iter().sum(),
        ProjectionMode::Mean => values.iter().sum::<f64>() / divisor,
        ProjectionMode::Max => values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        ProjectionMode::Min => values.iter().copied().fold(f64::INFINITY, f64::min),
        ProjectionMode::MaxAbs => values.iter().copied().fold(0.0, |selected, candidate| {
            if candidate.abs() > selected.abs() {
                candidate
            } else {
                selected
            }
        }),
    }
}

fn divisor(value: usize) -> Result<f64> {
    let value = u32::try_from(value).map_err(|_| RSpinError::InvalidSpectrum {
        message: "2D dimension is too large to reduce".to_owned(),
    })?;
    Ok(f64::from(value))
}

fn derived_1d(
    spectrum: &Spectrum2D,
    axis: Axis,
    values: Vec<f64>,
    record: ProcessingRecord,
) -> Result<Spectrum1D> {
    let mut derived = Spectrum1D::new(axis, values, spectrum.metadata.clone())?;
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
mod tests {
    use rspin_core::{Axis, Metadata, Unit};

    use super::*;

    #[test]
    fn scales_2d_values() -> anyhow::Result<()> {
        let spectrum = demo_spectrum()?;
        let processed = Scale2D { factor: 2.0 }.apply(&spectrum)?;
        assert_eq!(processed.z, vec![2.0, -4.0, 6.0, 8.0, -10.0, 12.0]);
        assert_eq!(processed.processing[0].operation, "scale_2d");
        Ok(())
    }

    #[test]
    fn normalizes_2d_values() -> anyhow::Result<()> {
        let spectrum = demo_spectrum()?;
        let processed = Normalize2DMaxAbs.apply(&spectrum)?;
        assert_vec_close(
            &processed.z,
            &[1.0 / 6.0, -2.0 / 6.0, 3.0 / 6.0, 4.0 / 6.0, -5.0 / 6.0, 1.0],
        );
        Ok(())
    }

    #[test]
    fn projects_x_and_y() -> anyhow::Result<()> {
        let spectrum = demo_spectrum()?;
        let x_projection = project_x(&spectrum, ProjectionMode::Sum)?;
        let y_projection = project_y(&spectrum, ProjectionMode::Mean)?;
        assert_eq!(x_projection.intensities, vec![5.0, -7.0, 9.0]);
        assert_eq!(y_projection.intensities, vec![2.0 / 3.0, 5.0 / 3.0]);
        Ok(())
    }

    #[test]
    fn projects_max_abs_with_sign() -> anyhow::Result<()> {
        let spectrum = demo_spectrum()?;
        let projection = project_x(&spectrum, ProjectionMode::MaxAbs)?;
        assert_eq!(projection.intensities, vec![4.0, -5.0, 6.0]);
        Ok(())
    }

    #[test]
    fn extracts_row_and_column_slices() -> anyhow::Result<()> {
        let spectrum = demo_spectrum()?;
        let row = slice_x_at_y_index(&spectrum, 1)?;
        let column = slice_y_at_x_index(&spectrum, 1)?;
        assert_eq!(row.intensities, vec![4.0, -5.0, 6.0]);
        assert_eq!(row.x.values, spectrum.x.values);
        assert_eq!(column.intensities, vec![-2.0, -5.0]);
        assert_eq!(column.x.values, spectrum.y.values);
        Ok(())
    }

    #[test]
    fn rejects_out_of_bounds_slice() -> anyhow::Result<()> {
        let spectrum = demo_spectrum()?;
        let error = slice_y_at_x_index(&spectrum, 3).expect_err("x index should be out of bounds");
        assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));
        Ok(())
    }

    fn demo_spectrum() -> anyhow::Result<Spectrum2D> {
        Ok(Spectrum2D::new(
            Axis::linear("x", Unit::Ppm, 0.0, 2.0, 3)?,
            Axis::linear("y", Unit::Ppm, 10.0, 11.0, 2)?,
            vec![1.0, -2.0, 3.0, 4.0, -5.0, 6.0],
            Metadata::named("2d"),
        )?)
    }

    fn assert_vec_close(actual: &[f64], expected: &[f64]) {
        assert_eq!(actual.len(), expected.len());
        for (left, right) in actual.iter().zip(expected) {
            assert!((left - right).abs() < 1e-12, "{left} != {right}");
        }
    }
}
