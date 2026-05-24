//! Two-dimensional resampling.

use rspin_core::{Axis, ProcessingRecord, RSpinError, Result, Spectrum2D};
use serde::{Deserialize, Serialize};

use crate::ProcessingStep;

/// Bilinearly resamples a two-dimensional spectrum onto target axes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Resample2D {
    /// Target x axis.
    pub target_x: Axis,
    /// Target y axis.
    pub target_y: Axis,
    /// Value used outside the source axis domain.
    pub outside_value: f64,
}

impl Resample2D {
    /// Creates a resampling step with zero outside the source domain.
    #[must_use]
    pub fn new(target_x: Axis, target_y: Axis) -> Self {
        Self {
            target_x,
            target_y,
            outside_value: 0.0,
        }
    }

    /// Sets the value used outside the source axis domain.
    #[must_use]
    pub fn with_outside_value(mut self, outside_value: f64) -> Self {
        self.outside_value = outside_value;
        self
    }
}

impl ProcessingStep<Spectrum2D> for Resample2D {
    fn apply(&self, spectrum: &Spectrum2D) -> Result<Spectrum2D> {
        resample_2d(
            spectrum,
            self.target_x.clone(),
            self.target_y.clone(),
            self.outside_value,
        )
    }
}

/// Bilinearly resamples real and imaginary matrices onto `target_x` and `target_y`.
///
/// Source axes may be ascending or descending, but each must be strictly
/// monotonic when it contains more than one point. Coordinates outside either
/// source axis domain receive `outside_value`.
///
/// # Errors
///
/// Returns an error when `outside_value` is non-finite or a source axis is not
/// strictly monotonic.
pub fn resample_2d(
    spectrum: &Spectrum2D,
    target_x: Axis,
    target_y: Axis,
    outside_value: f64,
) -> Result<Spectrum2D> {
    ensure_finite("outside_value", outside_value)?;
    validate_source_axis(&spectrum.x, "x source axis")?;
    validate_source_axis(&spectrum.y, "y source axis")?;

    let target_len =
        target_x
            .len()
            .checked_mul(target_y.len())
            .ok_or_else(|| RSpinError::InvalidSpectrum {
                message: "2D resampling target size overflow".to_owned(),
            })?;
    let mut z = Vec::with_capacity(target_len);
    let mut imaginary = spectrum
        .imaginary
        .as_ref()
        .map(|_| Vec::with_capacity(target_len));

    let x_segments = target_x
        .values
        .iter()
        .copied()
        .map(|x| axis_interpolation(&spectrum.x.values, x))
        .collect::<Vec<_>>();
    let y_segments = target_y
        .values
        .iter()
        .copied()
        .map(|y| axis_interpolation(&spectrum.y.values, y))
        .collect::<Vec<_>>();
    let (width, _) = spectrum.shape();

    for y_segment in &y_segments {
        for x_segment in &x_segments {
            z.push(sample_grid(
                &spectrum.z,
                width,
                *x_segment,
                *y_segment,
                outside_value,
            ));
            if let (Some(source), Some(target)) = (&spectrum.imaginary, &mut imaginary) {
                target.push(sample_grid(
                    source,
                    width,
                    *x_segment,
                    *y_segment,
                    outside_value,
                ));
            }
        }
    }

    let mut resampled =
        Spectrum2D::new_complex(target_x, target_y, z, imaginary, spectrum.metadata.clone())?;
    resampled.processing.clone_from(&spectrum.processing);
    let (target_width, target_height) = resampled.shape();
    let record = ProcessingRecord::new("resample_2d").with_details(format!(
        "target_width={target_width},target_height={target_height},outside_value={outside_value}"
    ));
    Ok(resampled.with_processing_record(record))
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum AxisInterpolation {
    Single(usize),
    Between {
        start: usize,
        end: usize,
        fraction: f64,
    },
}

fn axis_interpolation(axis: &[f64], coordinate: f64) -> Option<AxisInterpolation> {
    if axis.len() == 1 {
        if (axis[0] - coordinate).abs() <= f64::EPSILON {
            return Some(AxisInterpolation::Single(0));
        }
        return None;
    }

    for (index, pair) in axis.windows(2).enumerate() {
        let start = pair[0];
        let end = pair[1];
        if point_in_segment(coordinate, start, end) {
            let fraction = (coordinate - start) / (end - start);
            return Some(AxisInterpolation::Between {
                start: index,
                end: index + 1,
                fraction,
            });
        }
    }
    None
}

fn sample_grid(
    values: &[f64],
    width: usize,
    x: Option<AxisInterpolation>,
    y: Option<AxisInterpolation>,
    outside_value: f64,
) -> f64 {
    let Some(x) = x else {
        return outside_value;
    };
    let Some(y) = y else {
        return outside_value;
    };
    match (x, y) {
        (AxisInterpolation::Single(x_index), AxisInterpolation::Single(y_index)) => {
            values[y_index * width + x_index]
        }
        (
            AxisInterpolation::Between {
                start,
                end,
                fraction,
            },
            AxisInterpolation::Single(y_index),
        ) => interpolate(
            values[y_index * width + start],
            values[y_index * width + end],
            fraction,
        ),
        (
            AxisInterpolation::Single(x_index),
            AxisInterpolation::Between {
                start,
                end,
                fraction,
            },
        ) => interpolate(
            values[start * width + x_index],
            values[end * width + x_index],
            fraction,
        ),
        (
            AxisInterpolation::Between {
                start: x_start,
                end: x_end,
                fraction: x_fraction,
            },
            AxisInterpolation::Between {
                start: y_start,
                end: y_end,
                fraction: y_fraction,
            },
        ) => {
            let top = interpolate(
                values[y_start * width + x_start],
                values[y_start * width + x_end],
                x_fraction,
            );
            let bottom = interpolate(
                values[y_end * width + x_start],
                values[y_end * width + x_end],
                x_fraction,
            );
            interpolate(top, bottom, y_fraction)
        }
    }
}

fn interpolate(start: f64, end: f64, fraction: f64) -> f64 {
    start + fraction * (end - start)
}

fn point_in_segment(x: f64, start: f64, end: f64) -> bool {
    if start <= end {
        x >= start && x <= end
    } else {
        x <= start && x >= end
    }
}

fn validate_source_axis(axis: &Axis, field: &'static str) -> Result<()> {
    if axis.len() > 1 && !is_strictly_monotonic(&axis.values) {
        return Err(RSpinError::InvalidAxis {
            message: format!("{field} must be strictly monotonic for resampling"),
        });
    }
    Ok(())
}

fn is_strictly_monotonic(values: &[f64]) -> bool {
    let ascending = values.windows(2).all(|pair| pair[0] < pair[1]);
    let descending = values.windows(2).all(|pair| pair[0] > pair[1]);
    ascending || descending
}

fn ensure_finite(field: &'static str, value: f64) -> Result<()> {
    if !value.is_finite() {
        return Err(RSpinError::NonFinite { field });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use rspin_core::{Metadata, Unit};

    use super::*;

    #[test]
    fn resamples_real_grid_bilinearly() -> anyhow::Result<()> {
        let spectrum = Spectrum2D::new(
            Axis::linear("x", Unit::Ppm, 0.0, 2.0, 3)?,
            Axis::linear("y", Unit::Ppm, 0.0, 2.0, 3)?,
            vec![0.0, 10.0, 20.0, 100.0, 110.0, 120.0, 200.0, 210.0, 220.0],
            Metadata::named("grid"),
        )?;
        let resampled = Resample2D::new(
            Axis::linear("x", Unit::Ppm, 0.0, 2.0, 5)?,
            Axis::linear("y", Unit::Ppm, 0.0, 2.0, 5)?,
        )
        .apply(&spectrum)?;

        assert_eq!(resampled.shape(), (5, 5));
        assert_close(resampled.value_at(1, 1), Some(55.0));
        assert_close(resampled.value_at(3, 2), Some(115.0));
        assert_eq!(resampled.processing[0].operation, "resample_2d");
        Ok(())
    }

    #[test]
    fn resamples_complex_descending_grid_and_outside_values() -> anyhow::Result<()> {
        let spectrum = Spectrum2D::new_complex(
            Axis::new("x", Unit::Ppm, vec![2.0, 1.0, 0.0])?,
            Axis::new("y", Unit::Ppm, vec![2.0, 0.0])?,
            vec![20.0, 10.0, 0.0, 220.0, 210.0, 200.0],
            Some(vec![2.0, 1.0, 0.0, 22.0, 21.0, 20.0]),
            Metadata::default(),
        )?;
        let resampled = resample_2d(
            &spectrum,
            Axis::new("x", Unit::Ppm, vec![3.0, 1.5, 0.5])?,
            Axis::new("y", Unit::Ppm, vec![3.0, 1.0])?,
            -1.0,
        )?;

        assert_eq!(resampled.shape(), (3, 2));
        assert_vec_close(&resampled.z, &[-1.0, -1.0, -1.0, -1.0, 115.0, 105.0]);
        assert_option_vec_close(
            resampled.imaginary.as_deref(),
            &[-1.0, -1.0, -1.0, -1.0, 11.5, 10.5],
        );
        Ok(())
    }

    #[test]
    fn rejects_non_monotonic_source_axis() -> anyhow::Result<()> {
        let spectrum = Spectrum2D::new(
            Axis::new("x", Unit::Ppm, vec![0.0, 2.0, 1.0])?,
            Axis::linear("y", Unit::Ppm, 0.0, 1.0, 2)?,
            vec![0.0, 2.0, 1.0, 10.0, 12.0, 11.0],
            Metadata::default(),
        )?;
        let error = resample_2d(
            &spectrum,
            Axis::linear("x", Unit::Ppm, 0.0, 1.0, 2)?,
            Axis::linear("y", Unit::Ppm, 0.0, 1.0, 2)?,
            0.0,
        )
        .expect_err("non-monotonic source axis should fail");
        assert!(matches!(error, RSpinError::InvalidAxis { .. }));
        Ok(())
    }

    fn assert_close(actual: Option<f64>, expected: Option<f64>) {
        match (actual, expected) {
            (Some(actual), Some(expected)) => {
                assert!(
                    (actual - expected).abs() < 1.0e-12,
                    "{actual} != {expected}"
                );
            }
            _ => assert_eq!(actual, expected),
        }
    }

    fn assert_option_vec_close(actual: Option<&[f64]>, expected: &[f64]) {
        match actual {
            Some(values) => assert_vec_close(values, expected),
            None => panic!("expected imaginary values"),
        }
    }

    fn assert_vec_close(actual: &[f64], expected: &[f64]) {
        assert_eq!(actual.len(), expected.len());
        for (left, right) in actual.iter().zip(expected) {
            assert!((left - right).abs() < 1.0e-12, "{left} != {right}");
        }
    }
}
