//! Two-dimensional processing and extraction operations.

use rspin_core::{Axis, ProcessingRecord, RSpinError, Result, Spectrum1D, Spectrum2D};
use serde::{Deserialize, Serialize};

use crate::ProcessingStep;

/// Projection reduction mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Scale2D {
    /// Multiplicative factor.
    pub factor: f64,
}

impl Scale2D {
    /// Creates a two-dimensional scaling step.
    #[must_use]
    pub fn new(factor: f64) -> Self {
        Self { factor }
    }
}

impl ProcessingStep<Spectrum2D> for Scale2D {
    fn apply(&self, spectrum: &Spectrum2D) -> Result<Spectrum2D> {
        scale_2d(spectrum, self.factor)
    }
}

/// Adds a scalar offset to all real 2D intensities.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Offset2D {
    /// Additive offset.
    pub offset: f64,
}

impl Offset2D {
    /// Creates a two-dimensional real-intensity offset step.
    #[must_use]
    pub fn new(offset: f64) -> Self {
        Self { offset }
    }
}

impl ProcessingStep<Spectrum2D> for Offset2D {
    fn apply(&self, spectrum: &Spectrum2D) -> Result<Spectrum2D> {
        offset_2d(spectrum, self.offset)
    }
}

/// Normalizes 2D intensities by their maximum absolute value.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Normalize2DMaxAbs;

impl Normalize2DMaxAbs {
    /// Creates a two-dimensional max-absolute normalization step.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl ProcessingStep<Spectrum2D> for Normalize2DMaxAbs {
    fn apply(&self, spectrum: &Spectrum2D) -> Result<Spectrum2D> {
        normalize_2d_max_abs(spectrum)
    }
}

/// Normalizes 2D intensities so their bilinear volume matches a target value.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Normalize2DVolume {
    /// Desired integrated volume after normalization.
    pub target_volume: f64,
    /// Use absolute real intensities when measuring the volume.
    pub use_absolute_intensity: bool,
}

impl Normalize2DVolume {
    /// Creates signed volume normalization.
    #[must_use]
    pub fn new(target_volume: f64) -> Self {
        Self {
            target_volume,
            use_absolute_intensity: false,
        }
    }

    /// Creates absolute volume normalization.
    #[must_use]
    pub fn absolute(target_volume: f64) -> Self {
        Self {
            target_volume,
            use_absolute_intensity: true,
        }
    }

    /// Sets whether absolute real intensities are used for the volume.
    #[must_use]
    pub fn with_absolute_intensity(mut self, use_absolute_intensity: bool) -> Self {
        self.use_absolute_intensity = use_absolute_intensity;
        self
    }
}

impl ProcessingStep<Spectrum2D> for Normalize2DVolume {
    fn apply(&self, spectrum: &Spectrum2D) -> Result<Spectrum2D> {
        normalize_2d_volume(spectrum, self.target_volume, self.use_absolute_intensity)
    }
}

/// Shifts the x and y axes by constant deltas.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Shift2DAxes {
    /// Shift amount in the x-axis unit.
    pub x_delta: f64,
    /// Shift amount in the y-axis unit.
    pub y_delta: f64,
}

impl Shift2DAxes {
    /// Creates a two-dimensional axis-shift step.
    #[must_use]
    pub fn new(x_delta: f64, y_delta: f64) -> Self {
        Self { x_delta, y_delta }
    }

    /// Creates a step that shifts only the x axis.
    #[must_use]
    pub fn x(delta: f64) -> Self {
        Self::new(delta, 0.0)
    }

    /// Creates a step that shifts only the y axis.
    #[must_use]
    pub fn y(delta: f64) -> Self {
        Self::new(0.0, delta)
    }

    /// Sets the x-axis shift amount.
    #[must_use]
    pub fn with_x_delta(mut self, x_delta: f64) -> Self {
        self.x_delta = x_delta;
        self
    }

    /// Sets the y-axis shift amount.
    #[must_use]
    pub fn with_y_delta(mut self, y_delta: f64) -> Self {
        self.y_delta = y_delta;
        self
    }
}

impl ProcessingStep<Spectrum2D> for Shift2DAxes {
    fn apply(&self, spectrum: &Spectrum2D) -> Result<Spectrum2D> {
        shift_2d_axes(spectrum, self.x_delta, self.y_delta)
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
    scale_2d_values(&mut processed, factor);
    Ok(recorded_2d(
        processed,
        ProcessingRecord::new("scale_2d").with_details(format!("factor={factor}")),
    ))
}

/// Adds `offset` to all real 2D intensities.
///
/// The imaginary matrix, when present, is preserved unchanged.
///
/// # Errors
///
/// Returns an error when `offset` is not finite.
pub fn offset_2d(spectrum: &Spectrum2D, offset: f64) -> Result<Spectrum2D> {
    ensure_finite("2D intensity offset", offset)?;
    let mut processed = spectrum.clone();
    processed.z = processed
        .z
        .into_iter()
        .map(|value| value + offset)
        .collect();
    Ok(recorded_2d(
        processed,
        ProcessingRecord::new("offset_2d").with_details(format!("offset={offset}")),
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

/// Integrates real values over the x/y axes with bilinear cell averaging.
///
/// When `use_absolute_intensity` is true, each real value is converted to its
/// absolute value before integration. Axis direction does not change the sign of
/// the volume.
///
/// # Errors
///
/// Returns an error when either axis has fewer than two points, the spectrum has
/// no non-zero-area cells, or the computed volume is not finite.
pub fn spectrum_volume_2d(spectrum: &Spectrum2D, use_absolute_intensity: bool) -> Result<f64> {
    let (width, height) = spectrum.shape();
    if width < 2 || height < 2 {
        return Err(RSpinError::InvalidSpectrum {
            message: "2D volume normalization requires at least two points on each axis".to_owned(),
        });
    }

    let mut volume = 0.0;
    let mut cell_count = 0_usize;
    for y_index in 0..height - 1 {
        let dy = (spectrum.y.values[y_index + 1] - spectrum.y.values[y_index]).abs();
        if dy <= f64::EPSILON {
            continue;
        }
        for x_index in 0..width - 1 {
            let dx = (spectrum.x.values[x_index + 1] - spectrum.x.values[x_index]).abs();
            if dx <= f64::EPSILON {
                continue;
            }
            let top_left = matrix_value(spectrum, x_index, y_index, use_absolute_intensity);
            let top_right = matrix_value(spectrum, x_index + 1, y_index, use_absolute_intensity);
            let bottom_left = matrix_value(spectrum, x_index, y_index + 1, use_absolute_intensity);
            let bottom_right =
                matrix_value(spectrum, x_index + 1, y_index + 1, use_absolute_intensity);
            volume += 0.25 * (top_left + top_right + bottom_left + bottom_right) * dx * dy;
            cell_count += 1;
        }
    }

    if cell_count == 0 {
        return Err(RSpinError::InvalidSpectrum {
            message: "2D volume calculation requires at least one non-zero-area cell".to_owned(),
        });
    }
    if !volume.is_finite() {
        return Err(RSpinError::NonFinite {
            field: "spectrum volume",
        });
    }
    Ok(volume)
}

/// Normalizes real and imaginary 2D values to a target bilinear volume.
///
/// The scale factor is computed from real values and then applied to both real
/// and imaginary matrices.
///
/// # Errors
///
/// Returns an error when the target volume is not finite, the target volume is
/// zero, the absolute target is negative, or the current volume cannot be used
/// as a normalization denominator.
pub fn normalize_2d_volume(
    spectrum: &Spectrum2D,
    target_volume: f64,
    use_absolute_intensity: bool,
) -> Result<Spectrum2D> {
    ensure_finite("target volume", target_volume)?;
    if target_volume == 0.0 {
        return Err(RSpinError::InvalidSpectrum {
            message: "target volume must be non-zero".to_owned(),
        });
    }
    if use_absolute_intensity && target_volume <= 0.0 {
        return Err(RSpinError::InvalidSpectrum {
            message: "absolute volume normalization requires a positive target volume".to_owned(),
        });
    }

    let current_volume = spectrum_volume_2d(spectrum, use_absolute_intensity)?;
    if current_volume == 0.0 {
        return Err(RSpinError::InvalidSpectrum {
            message: "cannot normalize a 2D spectrum with zero integrated volume".to_owned(),
        });
    }
    let factor = target_volume / current_volume;
    if !factor.is_finite() {
        return Err(RSpinError::NonFinite {
            field: "2D volume normalization factor",
        });
    }

    let mut processed = spectrum.clone();
    scale_2d_values(&mut processed, factor);
    Ok(recorded_2d(
        processed,
        ProcessingRecord::new("normalize_2d_volume").with_details(format!(
            "target_volume={target_volume},use_absolute_intensity={use_absolute_intensity}"
        )),
    ))
}

/// Shifts x and y axis values by constant deltas.
///
/// # Errors
///
/// Returns an error when either delta is not finite.
pub fn shift_2d_axes(spectrum: &Spectrum2D, x_delta: f64, y_delta: f64) -> Result<Spectrum2D> {
    ensure_finite("x axis shift", x_delta)?;
    ensure_finite("y axis shift", y_delta)?;
    let mut processed = spectrum.clone();
    processed.x = shifted_axis(&processed.x, x_delta)?;
    processed.y = shifted_axis(&processed.y, y_delta)?;
    Ok(recorded_2d(
        processed,
        ProcessingRecord::new("shift_2d_axes")
            .with_details(format!("x_delta={x_delta},y_delta={y_delta}")),
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

fn scale_2d_values(spectrum: &mut Spectrum2D, factor: f64) {
    for value in &mut spectrum.z {
        *value *= factor;
    }
    if let Some(imaginary) = &mut spectrum.imaginary {
        for value in imaginary {
            *value *= factor;
        }
    }
}

fn shifted_axis(axis: &Axis, delta: f64) -> Result<Axis> {
    Axis::new(
        axis.label.clone(),
        axis.unit,
        axis.values.iter().map(|value| value + delta).collect(),
    )
}

fn matrix_value(
    spectrum: &Spectrum2D,
    x_index: usize,
    y_index: usize,
    use_absolute_intensity: bool,
) -> f64 {
    let value = spectrum.z[y_index * spectrum.x.len() + x_index];
    if use_absolute_intensity {
        value.abs()
    } else {
        value
    }
}

#[cfg(test)]
mod tests;
