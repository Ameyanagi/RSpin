//! One-dimensional range detection.

use serde::{Deserialize, Serialize};

use rspin_core::{RSpinError, Result, Spectrum1D};

use crate::{IntegralRegion, RangeDetector, integrate_region};

/// Options for threshold-based range detection.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct RangeDetectionOptions {
    /// Minimum absolute intensity for a point to be considered active.
    pub threshold_abs: f64,
    /// Minimum number of active points in a detected range.
    pub min_active_points: usize,
    /// Number of inactive points allowed inside a range before it is split.
    pub merge_gap_points: usize,
}

impl Default for RangeDetectionOptions {
    fn default() -> Self {
        Self {
            threshold_abs: 0.0,
            min_active_points: 1,
            merge_gap_points: 0,
        }
    }
}

impl RangeDetectionOptions {
    /// Creates default range-detection options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the minimum absolute intensity for active points.
    #[must_use]
    pub fn with_threshold_abs(mut self, threshold_abs: f64) -> Self {
        self.threshold_abs = threshold_abs;
        self
    }

    /// Sets the minimum number of active points in a detected range.
    #[must_use]
    pub fn with_min_active_points(mut self, min_active_points: usize) -> Self {
        self.min_active_points = min_active_points;
        self
    }

    /// Sets the number of inactive points allowed inside a range.
    #[must_use]
    pub fn with_merge_gap_points(mut self, merge_gap_points: usize) -> Self {
        self.merge_gap_points = merge_gap_points;
        self
    }

    fn validate(self) -> Result<()> {
        if !self.threshold_abs.is_finite() {
            return Err(RSpinError::NonFinite {
                field: "threshold_abs",
            });
        }
        if self.threshold_abs < 0.0 {
            return Err(RSpinError::InvalidSpectrum {
                message: "range threshold must be non-negative".to_owned(),
            });
        }
        if self.min_active_points == 0 {
            return Err(RSpinError::InvalidSpectrum {
                message: "minimum active points must be positive".to_owned(),
            });
        }
        Ok(())
    }
}

/// A detected one-dimensional range.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DetectedRange {
    /// First index included in the range span.
    pub start_index: usize,
    /// Last index included in the range span.
    pub end_index: usize,
    /// Range start coordinate.
    pub from: f64,
    /// Range end coordinate.
    pub to: f64,
    /// Number of active points contributing to the range.
    pub active_points: usize,
    /// Maximum absolute intensity inside the range span.
    pub max_abs_intensity: f64,
    /// Trapezoidal area over the range span.
    pub area: f64,
}

/// Threshold-based range detector.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ThresholdRangeDetector {
    /// Detection options.
    pub options: RangeDetectionOptions,
}

impl ThresholdRangeDetector {
    /// Creates a threshold range detector with default options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets range-detection options.
    #[must_use]
    pub fn with_options(mut self, options: RangeDetectionOptions) -> Self {
        self.options = options;
        self
    }
}

impl RangeDetector for ThresholdRangeDetector {
    fn detect(&self, spectrum: &Spectrum1D) -> Result<Vec<DetectedRange>> {
        detect_ranges(spectrum, self.options)
    }
}

#[derive(Clone, Copy, Debug)]
struct ActiveSpan {
    start_index: usize,
    last_active_index: usize,
    active_points: usize,
    gap_points: usize,
}

/// Detects contiguous ranges whose absolute intensity crosses `threshold_abs`.
///
/// # Errors
///
/// Returns an error when options are invalid or area calculation fails.
pub fn detect_ranges(
    spectrum: &Spectrum1D,
    options: RangeDetectionOptions,
) -> Result<Vec<DetectedRange>> {
    options.validate()?;
    let mut ranges = Vec::new();
    let mut active_span: Option<ActiveSpan> = None;

    for (index, intensity) in spectrum.intensities.iter().copied().enumerate() {
        let is_active = intensity.abs() >= options.threshold_abs;
        active_span = match (active_span, is_active) {
            (None, false) => None,
            (None, true) => Some(ActiveSpan {
                start_index: index,
                last_active_index: index,
                active_points: 1,
                gap_points: 0,
            }),
            (Some(mut span), true) => {
                span.last_active_index = index;
                span.active_points += 1;
                span.gap_points = 0;
                Some(span)
            }
            (Some(mut span), false) => {
                span.gap_points += 1;
                if span.gap_points > options.merge_gap_points {
                    maybe_push_range(&mut ranges, spectrum, span, options.min_active_points)?;
                    None
                } else {
                    Some(span)
                }
            }
        };
    }

    if let Some(span) = active_span {
        maybe_push_range(&mut ranges, spectrum, span, options.min_active_points)?;
    }

    Ok(ranges)
}

fn maybe_push_range(
    ranges: &mut Vec<DetectedRange>,
    spectrum: &Spectrum1D,
    span: ActiveSpan,
    min_active_points: usize,
) -> Result<()> {
    if span.active_points < min_active_points {
        return Ok(());
    }
    let start_index = span.start_index;
    let end_index = span.last_active_index;
    let from = spectrum.x.values[start_index];
    let to = spectrum.x.values[end_index];
    let max_abs_intensity = spectrum.intensities[start_index..=end_index]
        .iter()
        .map(|value| value.abs())
        .fold(0.0_f64, f64::max);
    let area = integrate_region(spectrum, IntegralRegion { from, to })?.area;
    ranges.push(DetectedRange {
        start_index,
        end_index,
        from,
        to,
        active_points: span.active_points,
        max_abs_intensity,
        area,
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use rspin_core::{Axis, Metadata, Unit};

    use super::*;

    #[test]
    fn detects_threshold_ranges() -> anyhow::Result<()> {
        let spectrum = spectrum(&[0.0, 2.0, 3.0, 0.2, -4.0, -5.0, 0.0])?;
        let ranges = detect_ranges(
            &spectrum,
            RangeDetectionOptions {
                threshold_abs: 1.0,
                min_active_points: 1,
                merge_gap_points: 0,
            },
        )?;
        assert_eq!(ranges.len(), 2);
        assert_eq!(ranges[0].start_index, 1);
        assert_eq!(ranges[0].end_index, 2);
        assert_eq!(ranges[0].active_points, 2);
        assert_close(ranges[0].max_abs_intensity, 3.0);
        assert_eq!(ranges[1].start_index, 4);
        assert_eq!(ranges[1].end_index, 5);
        Ok(())
    }

    #[test]
    fn merges_small_gaps() -> anyhow::Result<()> {
        let spectrum = spectrum(&[0.0, 2.0, 0.1, 3.0, 0.0])?;
        let detector = ThresholdRangeDetector::new().with_options(
            RangeDetectionOptions::new()
                .with_threshold_abs(1.0)
                .with_min_active_points(2)
                .with_merge_gap_points(1),
        );
        let ranges = detector.detect(&spectrum)?;
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].start_index, 1);
        assert_eq!(ranges[0].end_index, 3);
        assert_eq!(ranges[0].active_points, 2);
        Ok(())
    }

    #[test]
    fn filters_short_ranges() -> anyhow::Result<()> {
        let spectrum = spectrum(&[0.0, 2.0, 0.0, 3.0, 4.0])?;
        let ranges = detect_ranges(
            &spectrum,
            RangeDetectionOptions {
                threshold_abs: 1.0,
                min_active_points: 2,
                merge_gap_points: 0,
            },
        )?;
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].start_index, 3);
        Ok(())
    }

    #[test]
    fn rejects_invalid_options() -> anyhow::Result<()> {
        let spectrum = spectrum(&[0.0, 1.0, 0.0])?;
        let error = detect_ranges(
            &spectrum,
            RangeDetectionOptions {
                threshold_abs: -1.0,
                ..RangeDetectionOptions::default()
            },
        )
        .expect_err("negative threshold should fail");
        assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));
        Ok(())
    }

    fn spectrum(intensities: &[f64]) -> anyhow::Result<Spectrum1D> {
        let end = f64::from(u32::try_from(intensities.len() - 1)?);
        Ok(Spectrum1D::new(
            Axis::linear("x", Unit::Ppm, 0.0, end, intensities.len())?,
            intensities.to_vec(),
            Metadata::default(),
        )?)
    }

    fn assert_close(actual: f64, expected: f64) {
        assert!((actual - expected).abs() < 1e-12, "{actual} != {expected}");
    }
}
