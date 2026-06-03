//! One-dimensional correction and cleanup operations.

use rspin_core::{ProcessingRecord, RSpinError, Result, Spectrum1D};
use serde::{Deserialize, Serialize};

use crate::ProcessingStep;

/// Strategy for estimating or providing a constant DC offset.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DcOffsetMethod {
    /// Estimate the offset from the first `count` points.
    MeanFirstPoints {
        /// Number of leading points used for the estimate.
        count: usize,
    },
    /// Estimate the offset from the last `count` points.
    MeanLastPoints {
        /// Number of trailing points used for the estimate.
        count: usize,
    },
    /// Estimate the offset from the first and last `count` points.
    MeanEdges {
        /// Number of points used at each edge.
        count: usize,
    },
    /// Use explicit real and optional imaginary offsets.
    Explicit {
        /// Real-channel offset to subtract.
        real_offset: f64,
        /// Imaginary-channel offset to subtract when present.
        imaginary_offset: Option<f64>,
    },
}

impl Default for DcOffsetMethod {
    fn default() -> Self {
        Self::MeanLastPoints { count: 32 }
    }
}

/// Estimated DC offset values.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DcOffsetReport {
    /// Method used for the estimate or explicit value.
    pub method: DcOffsetMethod,
    /// Real-channel offset that should be subtracted.
    pub real_offset: f64,
    /// Imaginary-channel offset that should be subtracted, when applicable.
    pub imaginary_offset: Option<f64>,
    /// Number of points used for estimated offsets.
    pub sample_count: usize,
}

/// DC offset correction processing step.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct DcOffsetCorrection {
    /// Offset method.
    pub method: DcOffsetMethod,
}

impl DcOffsetCorrection {
    /// Creates a DC offset correction step.
    #[must_use]
    pub fn new(method: DcOffsetMethod) -> Self {
        Self { method }
    }
}

impl ProcessingStep<Spectrum1D> for DcOffsetCorrection {
    fn apply(&self, spectrum: &Spectrum1D) -> Result<Spectrum1D> {
        correct_dc_offset(spectrum, self.method)
    }
}

/// Fill strategy for explicit region suppression.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuppressionFill {
    /// Replace selected points with zero.
    Zero,
    /// Linearly interpolate between neighboring points.
    #[default]
    LinearInterpolate,
    /// Replace selected points with the mean of neighboring edge points.
    MeanEdges,
}

/// Report for an explicit suppression operation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SuppressionReport {
    /// First requested x coordinate.
    pub from: f64,
    /// Second requested x coordinate.
    pub to: f64,
    /// Fill strategy used inside the selected range.
    pub fill: SuppressionFill,
    /// Number of suppressed points.
    pub suppressed_points: usize,
}

/// Explicit region-suppression processing step.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SuppressRegion1D {
    /// First requested x coordinate.
    pub from: f64,
    /// Second requested x coordinate.
    pub to: f64,
    /// Fill strategy used inside the selected range.
    pub fill: SuppressionFill,
}

impl SuppressRegion1D {
    /// Creates a region-suppression step.
    #[must_use]
    pub fn new(from: f64, to: f64, fill: SuppressionFill) -> Self {
        Self { from, to, fill }
    }
}

impl ProcessingStep<Spectrum1D> for SuppressRegion1D {
    fn apply(&self, spectrum: &Spectrum1D) -> Result<Spectrum1D> {
        suppress_region_1d(spectrum, self.from, self.to, self.fill)
    }
}

/// Estimates the DC offset for a one-dimensional spectrum.
///
/// # Errors
///
/// Returns an error when the method is invalid for the spectrum length or an
/// explicit offset is non-finite.
pub fn estimate_dc_offset(spectrum: &Spectrum1D, method: DcOffsetMethod) -> Result<DcOffsetReport> {
    match method {
        DcOffsetMethod::Explicit {
            real_offset,
            imaginary_offset,
        } => explicit_dc_offset(real_offset, imaginary_offset, method),
        DcOffsetMethod::MeanFirstPoints { count } => {
            let range = edge_range(spectrum.len(), count, Edge::First)?;
            estimated_dc_offset(spectrum, method, range)
        }
        DcOffsetMethod::MeanLastPoints { count } => {
            let range = edge_range(spectrum.len(), count, Edge::Last)?;
            estimated_dc_offset(spectrum, method, range)
        }
        DcOffsetMethod::MeanEdges { count } => {
            validate_count(count, spectrum.len(), "DC offset edge point count")?;
            let total = count
                .checked_mul(2)
                .ok_or_else(|| RSpinError::InvalidSpectrum {
                    message: "DC offset edge point count is too large".to_owned(),
                })?;
            if total > spectrum.len() {
                return Err(RSpinError::InvalidSpectrum {
                    message: "DC offset edge estimate requires non-overlapping edges".to_owned(),
                });
            }
            estimated_dc_offset_edges(spectrum, method, count)
        }
    }
}

/// Corrects a one-dimensional spectrum by subtracting a constant DC offset.
///
/// # Errors
///
/// Returns an error when offset estimation fails.
pub fn correct_dc_offset(spectrum: &Spectrum1D, method: DcOffsetMethod) -> Result<Spectrum1D> {
    let (spectrum, _) = correct_dc_offset_with_report(spectrum, method)?;
    Ok(spectrum)
}

/// Corrects a spectrum and returns the offset report used for correction.
///
/// # Errors
///
/// Returns an error when offset estimation fails.
pub fn correct_dc_offset_with_report(
    spectrum: &Spectrum1D,
    method: DcOffsetMethod,
) -> Result<(Spectrum1D, DcOffsetReport)> {
    let report = estimate_dc_offset(spectrum, method)?;
    let mut processed = spectrum.clone();
    for value in &mut processed.intensities {
        *value -= report.real_offset;
    }
    if let (Some(offset), Some(imaginary)) = (report.imaginary_offset, &mut processed.imaginary) {
        for value in imaginary {
            *value -= offset;
        }
    }
    Ok((
        recorded(
            processed,
            ProcessingRecord::new("correct_dc_offset").with_details(format!(
                "real_offset={},imaginary_offset={:?},sample_count={}",
                report.real_offset, report.imaginary_offset, report.sample_count
            )),
        ),
        report,
    ))
}

/// Suppresses an explicit x-axis region using the requested fill strategy.
///
/// # Errors
///
/// Returns an error when the requested range is invalid, selects no points, or
/// the fill strategy needs unavailable edge points.
pub fn suppress_region_1d(
    spectrum: &Spectrum1D,
    from: f64,
    to: f64,
    fill: SuppressionFill,
) -> Result<Spectrum1D> {
    let (spectrum, _) = suppress_region_1d_with_report(spectrum, from, to, fill)?;
    Ok(spectrum)
}

/// Suppresses an explicit x-axis region and returns an operation report.
///
/// # Errors
///
/// Returns an error when the requested range is invalid, selects no points, or
/// the fill strategy needs unavailable edge points.
pub fn suppress_region_1d_with_report(
    spectrum: &Spectrum1D,
    from: f64,
    to: f64,
    fill: SuppressionFill,
) -> Result<(Spectrum1D, SuppressionReport)> {
    validate_range(from, to)?;
    let selected = selected_indices(&spectrum.x.values, from, to);
    if selected.is_empty() {
        return Err(RSpinError::InvalidSpectrum {
            message: "suppression range selected no points".to_owned(),
        });
    }
    let mut processed = spectrum.clone();
    let first = selected[0];
    let last = selected[selected.len() - 1];
    match fill {
        SuppressionFill::Zero => fill_zero(&mut processed, &selected),
        SuppressionFill::LinearInterpolate => {
            fill_linear_interpolate(&mut processed, first, last, &selected)?;
        }
        SuppressionFill::MeanEdges => fill_mean_edges(&mut processed, first, last, &selected)?,
    }
    let report = SuppressionReport {
        from,
        to,
        fill,
        suppressed_points: selected.len(),
    };
    Ok((
        recorded(
            processed,
            ProcessingRecord::new("suppress_region_1d").with_details(format!(
                "from={from},to={to},fill={fill:?},suppressed_points={}",
                report.suppressed_points
            )),
        ),
        report,
    ))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Edge {
    First,
    Last,
}

fn explicit_dc_offset(
    real_offset: f64,
    imaginary_offset: Option<f64>,
    method: DcOffsetMethod,
) -> Result<DcOffsetReport> {
    ensure_finite("real_offset", real_offset)?;
    if let Some(offset) = imaginary_offset {
        ensure_finite("imaginary_offset", offset)?;
    }
    Ok(DcOffsetReport {
        method,
        real_offset,
        imaginary_offset,
        sample_count: 0,
    })
}

fn estimated_dc_offset(
    spectrum: &Spectrum1D,
    method: DcOffsetMethod,
    range: std::ops::Range<usize>,
) -> Result<DcOffsetReport> {
    let sample_count = range.len();
    let real_offset = mean(&spectrum.intensities[range.clone()], "real DC offset")?;
    let imaginary_offset = spectrum
        .imaginary
        .as_deref()
        .map(|imaginary| mean(&imaginary[range], "imaginary DC offset"))
        .transpose()?;
    Ok(DcOffsetReport {
        method,
        real_offset,
        imaginary_offset,
        sample_count,
    })
}

fn estimated_dc_offset_edges(
    spectrum: &Spectrum1D,
    method: DcOffsetMethod,
    count: usize,
) -> Result<DcOffsetReport> {
    let sample_count = count
        .checked_mul(2)
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "DC offset edge point count is too large".to_owned(),
        })?;
    let real_offset = mean_edges(&spectrum.intensities, count, "real DC offset")?;
    let imaginary_offset = spectrum
        .imaginary
        .as_deref()
        .map(|imaginary| mean_edges(imaginary, count, "imaginary DC offset"))
        .transpose()?;
    Ok(DcOffsetReport {
        method,
        real_offset,
        imaginary_offset,
        sample_count,
    })
}

fn edge_range(len: usize, count: usize, edge: Edge) -> Result<std::ops::Range<usize>> {
    validate_count(count, len, "DC offset point count")?;
    Ok(match edge {
        Edge::First => 0..count,
        Edge::Last => len - count..len,
    })
}

fn validate_count(count: usize, len: usize, label: &'static str) -> Result<()> {
    if count == 0 {
        return Err(RSpinError::InvalidSpectrum {
            message: format!("{label} must be positive"),
        });
    }
    if count > len {
        return Err(RSpinError::InvalidSpectrum {
            message: format!("{label} exceeds spectrum length"),
        });
    }
    Ok(())
}

fn mean(values: &[f64], field: &'static str) -> Result<f64> {
    let count = f64_count(values.len(), field)?;
    let value = values.iter().sum::<f64>() / count;
    ensure_finite(field, value)?;
    Ok(value)
}

fn mean_edges(values: &[f64], count: usize, field: &'static str) -> Result<f64> {
    let denominator = f64_count(
        count
            .checked_mul(2)
            .ok_or_else(|| RSpinError::InvalidSpectrum {
                message: "edge point count is too large".to_owned(),
            })?,
        field,
    )?;
    let head = values.iter().take(count).sum::<f64>();
    let tail = values.iter().rev().take(count).sum::<f64>();
    let value = (head + tail) / denominator;
    ensure_finite(field, value)?;
    Ok(value)
}

fn selected_indices(values: &[f64], from: f64, to: f64) -> Vec<usize> {
    let lower = from.min(to);
    let upper = from.max(to);
    values
        .iter()
        .enumerate()
        .filter_map(|(index, value)| {
            if *value >= lower && *value <= upper {
                Some(index)
            } else {
                None
            }
        })
        .collect()
}

fn fill_zero(spectrum: &mut Spectrum1D, selected: &[usize]) {
    for index in selected {
        spectrum.intensities[*index] = 0.0;
        if let Some(imaginary) = &mut spectrum.imaginary {
            imaginary[*index] = 0.0;
        }
    }
}

fn fill_linear_interpolate(
    spectrum: &mut Spectrum1D,
    first: usize,
    last: usize,
    selected: &[usize],
) -> Result<()> {
    let (left_index, right_index) = neighboring_edge_indices(spectrum.len(), first, last)?;
    let left_x = spectrum.x.values[left_index];
    let right_x = spectrum.x.values[right_index];
    let denominator = right_x - left_x;
    if denominator.abs() <= f64::EPSILON || !denominator.is_finite() {
        return Err(RSpinError::InvalidAxis {
            message: "linear suppression interpolation requires distinct edge x values".to_owned(),
        });
    }
    let left_real = spectrum.intensities[left_index];
    let right_real = spectrum.intensities[right_index];
    let imaginary_edges = spectrum
        .imaginary
        .as_ref()
        .map(|imaginary| (imaginary[left_index], imaginary[right_index]));

    for index in selected {
        let fraction = (spectrum.x.values[*index] - left_x) / denominator;
        spectrum.intensities[*index] = lerp(left_real, right_real, fraction);
        if let (Some((left_imag, right_imag)), Some(imaginary)) =
            (imaginary_edges, &mut spectrum.imaginary)
        {
            imaginary[*index] = lerp(left_imag, right_imag, fraction);
        }
    }
    Ok(())
}

fn fill_mean_edges(
    spectrum: &mut Spectrum1D,
    first: usize,
    last: usize,
    selected: &[usize],
) -> Result<()> {
    let (left_index, right_index) = neighboring_edge_indices(spectrum.len(), first, last)?;
    let real = 0.5 * (spectrum.intensities[left_index] + spectrum.intensities[right_index]);
    let imaginary_edge = spectrum
        .imaginary
        .as_ref()
        .map(|imaginary| 0.5 * (imaginary[left_index] + imaginary[right_index]));
    for index in selected {
        spectrum.intensities[*index] = real;
        if let (Some(value), Some(imaginary)) = (imaginary_edge, &mut spectrum.imaginary) {
            imaginary[*index] = value;
        }
    }
    Ok(())
}

fn neighboring_edge_indices(len: usize, first: usize, last: usize) -> Result<(usize, usize)> {
    if first == 0 || last + 1 >= len {
        return Err(RSpinError::InvalidSpectrum {
            message: "suppression fill requires one unsuppressed point on each side".to_owned(),
        });
    }
    Ok((first - 1, last + 1))
}

fn lerp(left: f64, right: f64, fraction: f64) -> f64 {
    left * (1.0 - fraction) + right * fraction
}

fn validate_range(from: f64, to: f64) -> Result<()> {
    ensure_finite("suppression range start", from)?;
    ensure_finite("suppression range end", to)
}

fn ensure_finite(field: &'static str, value: f64) -> Result<()> {
    if !value.is_finite() {
        return Err(RSpinError::NonFinite { field });
    }
    Ok(())
}

fn f64_count(count: usize, field: &'static str) -> Result<f64> {
    let count = u32::try_from(count).map_err(|_| RSpinError::InvalidSpectrum {
        message: format!("{field} sample count is too large"),
    })?;
    Ok(f64::from(count))
}

fn recorded(spectrum: Spectrum1D, record: ProcessingRecord) -> Spectrum1D {
    spectrum.with_processing_record(record)
}

#[cfg(test)]
mod tests {
    use rspin_core::{Axis, Metadata, Unit};

    use super::*;

    #[test]
    fn estimates_and_corrects_real_dc_offset() -> anyhow::Result<()> {
        let spectrum = demo_spectrum(&[5.0, 6.0, 7.0, 5.0])?;
        let (processed, report) =
            correct_dc_offset_with_report(&spectrum, DcOffsetMethod::MeanEdges { count: 1 })?;

        assert_eq!(report.sample_count, 2);
        assert_close(report.real_offset, 5.0);
        assert_eq!(processed.intensities, vec![0.0, 1.0, 2.0, 0.0]);
        assert_eq!(processed.processing[0].operation, "correct_dc_offset");
        Ok(())
    }

    #[test]
    fn corrects_complex_explicit_dc_offset() -> anyhow::Result<()> {
        let spectrum = Spectrum1D::new_complex(
            Axis::linear("time", Unit::Seconds, 0.0, 0.3, 4)?,
            vec![2.0, 3.0, 4.0, 5.0],
            Some(vec![1.5, 2.5, 3.5, 4.5]),
            Metadata::default(),
        )?;
        let processed = correct_dc_offset(
            &spectrum,
            DcOffsetMethod::Explicit {
                real_offset: 2.0,
                imaginary_offset: Some(1.5),
            },
        )?;

        assert_eq!(processed.intensities, vec![0.0, 1.0, 2.0, 3.0]);
        assert_eq!(processed.imaginary, Some(vec![0.0, 1.0, 2.0, 3.0]));
        Ok(())
    }

    #[test]
    fn rejects_overlapping_edge_dc_estimate() -> anyhow::Result<()> {
        let spectrum = demo_spectrum(&[1.0, 2.0, 3.0])?;
        let error = estimate_dc_offset(&spectrum, DcOffsetMethod::MeanEdges { count: 2 })
            .expect_err("overlapping edges should fail");

        assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));
        Ok(())
    }

    #[test]
    fn suppresses_region_by_linear_interpolation() -> anyhow::Result<()> {
        let spectrum = demo_spectrum(&[0.0, 1.0, 100.0, 100.0, 4.0])?;
        let (processed, report) = suppress_region_1d_with_report(
            &spectrum,
            2.0,
            3.0,
            SuppressionFill::LinearInterpolate,
        )?;

        assert_eq!(report.suppressed_points, 2);
        assert_eq!(processed.intensities, vec![0.0, 1.0, 2.0, 3.0, 4.0]);
        assert_eq!(processed.processing[0].operation, "suppress_region_1d");
        Ok(())
    }

    #[test]
    fn suppresses_region_by_mean_edges() -> anyhow::Result<()> {
        let spectrum = demo_spectrum(&[0.0, 2.0, 100.0, 100.0, 6.0])?;
        let processed = suppress_region_1d(&spectrum, 2.0, 3.0, SuppressionFill::MeanEdges)?;

        assert_eq!(processed.intensities, vec![0.0, 2.0, 4.0, 4.0, 6.0]);
        Ok(())
    }

    #[test]
    fn rejects_suppression_without_edge_points() -> anyhow::Result<()> {
        let spectrum = demo_spectrum(&[100.0, 100.0, 3.0])?;
        let error = suppress_region_1d(&spectrum, 0.0, 1.0, SuppressionFill::LinearInterpolate)
            .expect_err("suppression at an edge should fail");

        assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));
        Ok(())
    }

    fn demo_spectrum(values: &[f64]) -> anyhow::Result<Spectrum1D> {
        let last = u32::try_from(values.len() - 1)?;
        Ok(Spectrum1D::new(
            Axis::linear("shift", Unit::Ppm, 0.0, f64::from(last), values.len())?,
            values.to_vec(),
            Metadata::default(),
        )?)
    }

    fn assert_close(left: f64, right: f64) {
        assert!((left - right).abs() < 1.0e-12, "left={left}, right={right}");
    }
}
