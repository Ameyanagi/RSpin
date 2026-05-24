//! One-dimensional processing operations.

use rspin_core::{Axis, ProcessingRecord, RSpinError, Result, Spectrum1D};
use serde::{Deserialize, Serialize};

use crate::ProcessingStep;

/// Multiplies all intensities by a scalar.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScaleIntensity {
    /// Multiplicative factor.
    pub factor: f64,
}

impl ProcessingStep<Spectrum1D> for ScaleIntensity {
    fn apply(&self, spectrum: &Spectrum1D) -> Result<Spectrum1D> {
        scale_intensity(spectrum, self.factor)
    }
}

/// Adds a scalar offset to all intensities.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct OffsetIntensity {
    /// Additive offset.
    pub offset: f64,
}

impl ProcessingStep<Spectrum1D> for OffsetIntensity {
    fn apply(&self, spectrum: &Spectrum1D) -> Result<Spectrum1D> {
        offset_intensity(spectrum, self.offset)
    }
}

/// Normalizes intensities by their maximum absolute value.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizeMaxAbs;

impl ProcessingStep<Spectrum1D> for NormalizeMaxAbs {
    fn apply(&self, spectrum: &Spectrum1D) -> Result<Spectrum1D> {
        normalize_max_abs(spectrum)
    }
}

/// Normalizes intensities so their trapezoidal area matches a target value.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct NormalizeArea {
    /// Desired integrated area after normalization.
    pub target_area: f64,
    /// Use absolute real intensities when measuring the area.
    pub use_absolute_intensity: bool,
}

impl NormalizeArea {
    /// Creates signed area normalization.
    #[must_use]
    pub fn new(target_area: f64) -> Self {
        Self {
            target_area,
            use_absolute_intensity: false,
        }
    }

    /// Creates absolute area normalization.
    #[must_use]
    pub fn absolute(target_area: f64) -> Self {
        Self {
            target_area,
            use_absolute_intensity: true,
        }
    }

    /// Sets whether absolute real intensities are used for the area.
    #[must_use]
    pub fn with_absolute_intensity(mut self, use_absolute_intensity: bool) -> Self {
        self.use_absolute_intensity = use_absolute_intensity;
        self
    }
}

impl ProcessingStep<Spectrum1D> for NormalizeArea {
    fn apply(&self, spectrum: &Spectrum1D) -> Result<Spectrum1D> {
        normalize_area(spectrum, self.target_area, self.use_absolute_intensity)
    }
}

/// Shifts the x axis by a constant delta.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShiftAxis {
    /// Shift amount in the x-axis unit.
    pub delta: f64,
}

impl ProcessingStep<Spectrum1D> for ShiftAxis {
    fn apply(&self, spectrum: &Spectrum1D) -> Result<Spectrum1D> {
        shift_axis(spectrum, self.delta)
    }
}

/// Extends a one-dimensional spectrum with trailing zeroes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZeroFill {
    /// Desired total point count.
    pub target_len: usize,
}

impl ProcessingStep<Spectrum1D> for ZeroFill {
    fn apply(&self, spectrum: &Spectrum1D) -> Result<Spectrum1D> {
        zero_fill(spectrum, self.target_len)
    }
}

/// Multiplies all intensities by `factor`.
///
/// # Errors
///
/// Returns an error when `factor` is not finite.
pub fn scale_intensity(spectrum: &Spectrum1D, factor: f64) -> Result<Spectrum1D> {
    ensure_finite("scale factor", factor)?;
    let mut processed = spectrum.clone();
    processed.intensities = processed
        .intensities
        .into_iter()
        .map(|value| value * factor)
        .collect();
    if let Some(imaginary) = processed.imaginary.take() {
        processed.imaginary = Some(imaginary.into_iter().map(|value| value * factor).collect());
    }
    Ok(recorded(
        processed,
        ProcessingRecord::new("scale_intensity").with_details(format!("factor={factor}")),
    ))
}

/// Adds `offset` to all real intensities.
///
/// # Errors
///
/// Returns an error when `offset` is not finite.
pub fn offset_intensity(spectrum: &Spectrum1D, offset: f64) -> Result<Spectrum1D> {
    ensure_finite("intensity offset", offset)?;
    let mut processed = spectrum.clone();
    processed.intensities = processed
        .intensities
        .into_iter()
        .map(|value| value + offset)
        .collect();
    Ok(recorded(
        processed,
        ProcessingRecord::new("offset_intensity").with_details(format!("offset={offset}")),
    ))
}

/// Normalizes all intensities by the maximum absolute real intensity.
///
/// # Errors
///
/// Returns an error when all intensities are zero.
pub fn normalize_max_abs(spectrum: &Spectrum1D) -> Result<Spectrum1D> {
    let max_abs = spectrum
        .intensities
        .iter()
        .map(|value| value.abs())
        .fold(0.0_f64, f64::max);
    if max_abs == 0.0 {
        return Err(RSpinError::InvalidSpectrum {
            message: "cannot normalize an all-zero spectrum".to_owned(),
        });
    }
    let mut processed = scale_intensity(spectrum, 1.0 / max_abs)?;
    processed.processing.pop();
    Ok(recorded(
        processed,
        ProcessingRecord::new("normalize_max_abs"),
    ))
}

/// Integrates real intensities over the x axis with the trapezoidal rule.
///
/// When `use_absolute_intensity` is true, each real intensity is converted to
/// its absolute value before integration. Axis direction does not change the
/// sign of the area.
///
/// # Errors
///
/// Returns an error when the spectrum has fewer than two points, has no
/// non-zero-width intervals, or the computed area is not finite.
pub fn spectrum_area(spectrum: &Spectrum1D, use_absolute_intensity: bool) -> Result<f64> {
    if spectrum.len() < 2 {
        return Err(RSpinError::InvalidSpectrum {
            message: "area normalization requires at least two points".to_owned(),
        });
    }

    let mut area = 0.0;
    let mut interval_count = 0_usize;
    for (x_pair, y_pair) in spectrum
        .x
        .values
        .windows(2)
        .zip(spectrum.intensities.windows(2))
    {
        let width = (x_pair[1] - x_pair[0]).abs();
        if width <= f64::EPSILON {
            continue;
        }
        let left = area_intensity(y_pair[0], use_absolute_intensity);
        let right = area_intensity(y_pair[1], use_absolute_intensity);
        area += 0.5 * (left + right) * width;
        interval_count += 1;
    }

    if interval_count == 0 {
        return Err(RSpinError::InvalidSpectrum {
            message: "area calculation requires at least one non-zero-width interval".to_owned(),
        });
    }
    if !area.is_finite() {
        return Err(RSpinError::NonFinite {
            field: "spectrum area",
        });
    }
    Ok(area)
}

/// Normalizes real and imaginary intensities to a target trapezoidal area.
///
/// The scale factor is computed from real intensities and then applied to both
/// real and imaginary channels.
///
/// # Errors
///
/// Returns an error when the target area is not finite, the target area is
/// zero, the absolute target is negative, or the current area cannot be used as
/// a normalization denominator.
pub fn normalize_area(
    spectrum: &Spectrum1D,
    target_area: f64,
    use_absolute_intensity: bool,
) -> Result<Spectrum1D> {
    ensure_finite("target area", target_area)?;
    if target_area == 0.0 {
        return Err(RSpinError::InvalidSpectrum {
            message: "target area must be non-zero".to_owned(),
        });
    }
    if use_absolute_intensity && target_area <= 0.0 {
        return Err(RSpinError::InvalidSpectrum {
            message: "absolute area normalization requires a positive target area".to_owned(),
        });
    }

    let current_area = spectrum_area(spectrum, use_absolute_intensity)?;
    if current_area == 0.0 {
        return Err(RSpinError::InvalidSpectrum {
            message: "cannot normalize a spectrum with zero integrated area".to_owned(),
        });
    }
    let factor = target_area / current_area;
    if !factor.is_finite() {
        return Err(RSpinError::NonFinite {
            field: "area normalization factor",
        });
    }

    let mut processed = spectrum.clone();
    scale_values(&mut processed, factor);
    Ok(recorded(
        processed,
        ProcessingRecord::new("normalize_area").with_details(format!(
            "target_area={target_area},use_absolute_intensity={use_absolute_intensity}"
        )),
    ))
}

/// Shifts the x-axis values by `delta`.
///
/// # Errors
///
/// Returns an error when `delta` is not finite.
pub fn shift_axis(spectrum: &Spectrum1D, delta: f64) -> Result<Spectrum1D> {
    ensure_finite("axis shift", delta)?;
    let mut processed = spectrum.clone();
    let values = processed
        .x
        .values
        .iter()
        .map(|value| value + delta)
        .collect();
    processed.x = Axis::new(processed.x.label, processed.x.unit, values)?;
    Ok(recorded(
        processed,
        ProcessingRecord::new("shift_axis").with_details(format!("delta={delta}")),
    ))
}

/// Extends a spectrum to `target_len` points by appending zeroes.
///
/// Axis values are extended using the final observed spacing, or `1.0` when
/// only one point exists.
///
/// # Errors
///
/// Returns an error when `target_len` is smaller than the current length.
pub fn zero_fill(spectrum: &Spectrum1D, target_len: usize) -> Result<Spectrum1D> {
    if target_len < spectrum.len() {
        return Err(RSpinError::InvalidSpectrum {
            message: format!(
                "zero-fill target {target_len} is smaller than current length {}",
                spectrum.len()
            ),
        });
    }
    if target_len == spectrum.len() {
        return Ok(recorded(
            spectrum.clone(),
            ProcessingRecord::new("zero_fill"),
        ));
    }

    let mut processed = spectrum.clone();
    let step = axis_step(&processed.x);
    let mut next_x = processed
        .x
        .values
        .last()
        .copied()
        .ok_or_else(|| RSpinError::InvalidAxis {
            message: "missing x axis values".to_owned(),
        })?
        + step;

    processed.x.values.reserve(target_len - processed.len());
    processed.intensities.reserve(target_len - processed.len());
    while processed.len() < target_len {
        processed.x.values.push(next_x);
        processed.intensities.push(0.0);
        if let Some(imaginary) = &mut processed.imaginary {
            imaginary.push(0.0);
        }
        next_x += step;
    }

    Ok(recorded(
        processed,
        ProcessingRecord::new("zero_fill").with_details(format!("target_len={target_len}")),
    ))
}

fn ensure_finite(field: &'static str, value: f64) -> Result<()> {
    if !value.is_finite() {
        return Err(RSpinError::NonFinite { field });
    }
    Ok(())
}

fn scale_values(spectrum: &mut Spectrum1D, factor: f64) {
    for value in &mut spectrum.intensities {
        *value *= factor;
    }
    if let Some(imaginary) = &mut spectrum.imaginary {
        for value in imaginary {
            *value *= factor;
        }
    }
}

fn area_intensity(value: f64, use_absolute_intensity: bool) -> f64 {
    if use_absolute_intensity {
        value.abs()
    } else {
        value
    }
}

fn axis_step(axis: &Axis) -> f64 {
    let values = &axis.values;
    match values.as_slice() {
        [.., previous, last] => last - previous,
        [_] | [] => 1.0,
    }
}

fn recorded(spectrum: Spectrum1D, record: ProcessingRecord) -> Spectrum1D {
    spectrum.with_processing_record(record)
}

#[cfg(test)]
mod tests {
    use rspin_core::{Metadata, Unit};

    use super::*;

    #[test]
    fn scales_intensities() -> anyhow::Result<()> {
        let spectrum = demo_spectrum()?;
        let processed = scale_intensity(&spectrum, 2.0)?;
        assert_eq!(processed.intensities, vec![2.0, -4.0, 8.0]);
        assert_eq!(processed.processing[0].operation, "scale_intensity");
        Ok(())
    }

    #[test]
    fn offsets_intensities() -> anyhow::Result<()> {
        let spectrum = demo_spectrum()?;
        let processed = OffsetIntensity { offset: 1.0 }.apply(&spectrum)?;
        assert_eq!(processed.intensities, vec![2.0, -1.0, 5.0]);
        Ok(())
    }

    #[test]
    fn normalizes_by_max_abs() -> anyhow::Result<()> {
        let spectrum = demo_spectrum()?;
        let processed = NormalizeMaxAbs.apply(&spectrum)?;
        assert_eq!(processed.intensities, vec![0.25, -0.5, 1.0]);
        assert_eq!(processed.processing[0].operation, "normalize_max_abs");
        Ok(())
    }

    #[test]
    fn normalizes_by_signed_area() -> anyhow::Result<()> {
        let spectrum = Spectrum1D::new(
            Axis::linear("shift", Unit::Ppm, 0.0, 2.0, 3)?,
            vec![1.0, 1.0, 1.0],
            Metadata::default(),
        )?;
        let processed = NormalizeArea::new(4.0).apply(&spectrum)?;

        assert!((spectrum_area(&processed, false)? - 4.0).abs() < 1.0e-12);
        assert_eq!(processed.intensities, vec![2.0, 2.0, 2.0]);
        assert_eq!(processed.processing[0].operation, "normalize_area");
        assert_eq!(
            processed.processing[0].details.as_deref(),
            Some("target_area=4,use_absolute_intensity=false")
        );
        Ok(())
    }

    #[test]
    fn normalizes_by_absolute_area_and_scales_imaginary_channel() -> anyhow::Result<()> {
        let spectrum = Spectrum1D::new_complex(
            Axis::linear("shift", Unit::Ppm, 0.0, 2.0, 3)?,
            vec![-1.0, 1.0, -1.0],
            Some(vec![0.5, -0.5, 1.0]),
            Metadata::default(),
        )?;
        let processed = NormalizeArea::absolute(4.0).apply(&spectrum)?;

        assert!((spectrum_area(&processed, true)? - 4.0).abs() < 1.0e-12);
        assert_eq!(processed.intensities, vec![-2.0, 2.0, -2.0]);
        assert_eq!(processed.imaginary.as_deref(), Some(&[1.0, -1.0, 2.0][..]));
        Ok(())
    }

    #[test]
    fn rejects_unusable_area_normalization_inputs() -> anyhow::Result<()> {
        let spectrum = Spectrum1D::new(
            Axis::linear("shift", Unit::Ppm, 0.0, 2.0, 3)?,
            vec![1.0, 0.0, -1.0],
            Metadata::default(),
        )?;
        let zero_area_error =
            normalize_area(&spectrum, 1.0, false).expect_err("zero signed area should fail");
        assert!(matches!(
            zero_area_error,
            RSpinError::InvalidSpectrum { .. }
        ));

        let target_error =
            normalize_area(&spectrum, 0.0, true).expect_err("zero target area should fail");
        assert!(matches!(target_error, RSpinError::InvalidSpectrum { .. }));

        let short = Spectrum1D::new(
            Axis::linear("shift", Unit::Ppm, 0.0, 0.0, 1)?,
            vec![1.0],
            Metadata::default(),
        )?;
        let short_error = spectrum_area(&short, false).expect_err("short spectrum should fail");
        assert!(matches!(short_error, RSpinError::InvalidSpectrum { .. }));
        Ok(())
    }

    #[test]
    fn shifts_axis() -> anyhow::Result<()> {
        let spectrum = demo_spectrum()?;
        let processed = ShiftAxis { delta: -0.5 }.apply(&spectrum)?;
        assert_eq!(processed.x.values, vec![0.5, 1.5, 2.5]);
        Ok(())
    }

    #[test]
    fn zero_fills_trailing_points() -> anyhow::Result<()> {
        let spectrum = demo_spectrum()?;
        let processed = ZeroFill { target_len: 5 }.apply(&spectrum)?;
        assert_eq!(processed.x.values, vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        assert_eq!(processed.intensities, vec![1.0, -2.0, 4.0, 0.0, 0.0]);
        Ok(())
    }

    #[test]
    fn rejects_invalid_zero_fill_target() -> anyhow::Result<()> {
        let spectrum = demo_spectrum()?;
        let error = zero_fill(&spectrum, 2).expect_err("short target should fail");
        assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));
        Ok(())
    }

    fn demo_spectrum() -> anyhow::Result<Spectrum1D> {
        let x = Axis::linear("shift", Unit::Ppm, 1.0, 3.0, 3)?;
        Ok(Spectrum1D::new(
            x,
            vec![1.0, -2.0, 4.0],
            Metadata::default(),
        )?)
    }
}
