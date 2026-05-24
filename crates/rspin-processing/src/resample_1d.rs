//! One-dimensional resampling.

use rspin_core::{Axis, ProcessingRecord, RSpinError, Result, Spectrum1D};
use serde::{Deserialize, Serialize};

use crate::ProcessingStep;

/// Linearly resamples a one-dimensional spectrum onto a target axis.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Resample1D {
    /// Target x axis.
    pub target_axis: Axis,
    /// Value used outside the source axis domain.
    pub outside_value: f64,
}

impl Resample1D {
    /// Creates a resampling step with zero outside the source domain.
    #[must_use]
    pub fn new(target_axis: Axis) -> Self {
        Self {
            target_axis,
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

impl ProcessingStep<Spectrum1D> for Resample1D {
    fn apply(&self, spectrum: &Spectrum1D) -> Result<Spectrum1D> {
        resample_1d(spectrum, self.target_axis.clone(), self.outside_value)
    }
}

/// Linearly resamples real and imaginary channels onto `target_axis`.
///
/// Source axes may be ascending or descending, but must be strictly monotonic.
/// Coordinates outside the source domain receive `outside_value`.
///
/// # Errors
///
/// Returns an error when `outside_value` is non-finite or the source axis is
/// not strictly monotonic.
pub fn resample_1d(
    spectrum: &Spectrum1D,
    target_axis: Axis,
    outside_value: f64,
) -> Result<Spectrum1D> {
    ensure_finite("outside_value", outside_value)?;
    validate_source_axis(&spectrum.x)?;

    let intensities = target_axis
        .values
        .iter()
        .copied()
        .map(|x| interpolate_channel(&spectrum.x.values, &spectrum.intensities, x, outside_value))
        .collect::<Vec<_>>();
    let imaginary = spectrum.imaginary.as_ref().map(|imaginary| {
        target_axis
            .values
            .iter()
            .copied()
            .map(|x| interpolate_channel(&spectrum.x.values, imaginary, x, outside_value))
            .collect::<Vec<_>>()
    });

    let mut resampled = Spectrum1D::new_complex(
        target_axis,
        intensities,
        imaginary,
        spectrum.metadata.clone(),
    )?;
    resampled.processing.clone_from(&spectrum.processing);
    let target_len = resampled.len();
    Ok(
        resampled.with_processing_record(ProcessingRecord::new("resample_1d").with_details(
            format!("target_len={target_len},outside_value={outside_value}"),
        )),
    )
}

fn interpolate_channel(axis: &[f64], values: &[f64], x: f64, outside_value: f64) -> f64 {
    if axis.len() == 1 {
        if (axis[0] - x).abs() <= f64::EPSILON {
            return values[0];
        }
        return outside_value;
    }

    for (index, pair) in axis.windows(2).enumerate() {
        let x0 = pair[0];
        let x1 = pair[1];
        if point_in_segment(x, x0, x1) {
            if (x1 - x0).abs() <= f64::EPSILON {
                return values[index];
            }
            let fraction = (x - x0) / (x1 - x0);
            return values[index] + fraction * (values[index + 1] - values[index]);
        }
    }

    outside_value
}

fn point_in_segment(x: f64, x0: f64, x1: f64) -> bool {
    if x0 <= x1 {
        x >= x0 && x <= x1
    } else {
        x <= x0 && x >= x1
    }
}

fn validate_source_axis(axis: &Axis) -> Result<()> {
    if axis.len() > 1 && !is_strictly_monotonic(&axis.values) {
        return Err(RSpinError::InvalidAxis {
            message: "source axis must be strictly monotonic for resampling".to_owned(),
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
    fn resamples_complex_spectrum_to_explicit_axis() -> anyhow::Result<()> {
        let spectrum = Spectrum1D::new_complex(
            Axis::linear("shift", Unit::Ppm, 0.0, 2.0, 3)?,
            vec![0.0, 10.0, 20.0],
            Some(vec![0.0, -10.0, -20.0]),
            Metadata::named("source"),
        )?;
        let target_axis = Axis::linear("shift", Unit::Ppm, 0.0, 2.0, 5)?;
        let resampled = Resample1D::new(target_axis).apply(&spectrum)?;

        assert_eq!(resampled.x.values, vec![0.0, 0.5, 1.0, 1.5, 2.0]);
        assert_vec_close(&resampled.intensities, &[0.0, 5.0, 10.0, 15.0, 20.0]);
        assert_option_vec_close(
            resampled.imaginary.as_deref(),
            &[0.0, -5.0, -10.0, -15.0, -20.0],
        );
        assert_eq!(resampled.processing[0].operation, "resample_1d");
        Ok(())
    }

    #[test]
    fn supports_descending_source_and_outside_value() -> anyhow::Result<()> {
        let spectrum = Spectrum1D::new(
            Axis::new("shift", Unit::Ppm, vec![2.0, 1.0, 0.0])?,
            vec![20.0, 10.0, 0.0],
            Metadata::default(),
        )?;
        let resampled = resample_1d(
            &spectrum,
            Axis::new("shift", Unit::Ppm, vec![3.0, 1.5, 0.5, -1.0])?,
            -1.0,
        )?;

        assert_vec_close(&resampled.intensities, &[-1.0, 15.0, 5.0, -1.0]);
        Ok(())
    }

    #[test]
    fn rejects_non_monotonic_source_axis() -> anyhow::Result<()> {
        let spectrum = Spectrum1D::new(
            Axis::new("shift", Unit::Ppm, vec![0.0, 2.0, 1.0])?,
            vec![0.0, 2.0, 1.0],
            Metadata::default(),
        )?;
        let error = resample_1d(
            &spectrum,
            Axis::linear("shift", Unit::Ppm, 0.0, 1.0, 2)?,
            0.0,
        )
        .expect_err("non-monotonic source axis should fail");
        assert!(matches!(error, RSpinError::InvalidAxis { .. }));
        Ok(())
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
