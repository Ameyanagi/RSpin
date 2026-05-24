//! Polynomial baseline fitting.

use nalgebra::{DMatrix, DVector};
use rspin_core::{RSpinError, Result};

pub(super) fn polynomial_baseline(
    x_values: &[f64],
    intensities: &[f64],
    degree: usize,
) -> Result<Vec<f64>> {
    if degree == 0 {
        return Ok(vec![mean(intensities)?; intensities.len()]);
    }

    let normalized_x = normalize_x(x_values)?;
    let design = vandermonde(&normalized_x, degree);
    let rhs = DVector::from_column_slice(intensities);
    let normal_matrix = design.transpose() * &design;
    let normal_rhs = design.transpose() * rhs;
    let coefficients =
        normal_matrix
            .lu()
            .solve(&normal_rhs)
            .ok_or_else(|| RSpinError::InvalidSpectrum {
                message: "polynomial baseline fit failed".to_owned(),
            })?;

    Ok(normalized_x
        .iter()
        .map(|x| evaluate_polynomial(&coefficients, *x))
        .collect())
}

fn normalize_x(x_values: &[f64]) -> Result<Vec<f64>> {
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    for value in x_values {
        min = min.min(*value);
        max = max.max(*value);
    }

    let span = max - min;
    if span == 0.0 {
        return Err(RSpinError::InvalidSpectrum {
            message: "polynomial baseline requires a non-degenerate x axis".to_owned(),
        });
    }
    let midpoint = (max + min) * 0.5;
    let half_span = span * 0.5;
    Ok(x_values
        .iter()
        .map(|value| (value - midpoint) / half_span)
        .collect())
}

fn vandermonde(x_values: &[f64], degree: usize) -> DMatrix<f64> {
    let mut matrix = DMatrix::zeros(x_values.len(), degree + 1);
    for (row, x) in x_values.iter().copied().enumerate() {
        let mut power = 1.0;
        for column in 0..=degree {
            matrix[(row, column)] = power;
            power *= x;
        }
    }
    matrix
}

fn evaluate_polynomial(coefficients: &DVector<f64>, x: f64) -> f64 {
    let mut power = 1.0;
    let mut value = 0.0;
    for coefficient in coefficients {
        value += *coefficient * power;
        power *= x;
    }
    value
}

fn mean(values: &[f64]) -> Result<f64> {
    let sum = values.iter().sum::<f64>();
    let len = u32::try_from(values.len()).map_err(|_| RSpinError::InvalidSpectrum {
        message: "polynomial baseline input is too large".to_owned(),
    })?;
    Ok(sum / f64::from(len))
}
