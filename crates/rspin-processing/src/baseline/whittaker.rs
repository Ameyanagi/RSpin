//! Whittaker asymmetric least-squares baseline fitting.

use nalgebra::{DMatrix, DVector};
use rspin_core::{RSpinError, Result};

use super::BaselineReport;

pub(super) fn whittaker_asls_baseline(
    intensities: &[f64],
    lambda: f64,
    p: f64,
    max_iter: usize,
    tolerance: f64,
) -> Result<(Vec<f64>, BaselineReport)> {
    let mut weights = vec![1.0; intensities.len()];
    let mut baseline = solve_whittaker(intensities, &weights, lambda)?;
    let mut final_tolerance = f64::INFINITY;

    for iteration in 1..=max_iter {
        let previous = weights.clone();
        for ((weight, observed), fitted) in weights.iter_mut().zip(intensities).zip(&baseline) {
            *weight = if observed > fitted { p } else { 1.0 - p };
        }
        final_tolerance = relative_change(&previous, &weights);
        if final_tolerance <= tolerance {
            return Ok((
                baseline,
                BaselineReport {
                    iterations: iteration,
                    converged: true,
                    tolerance: final_tolerance,
                },
            ));
        }
        baseline = solve_whittaker(intensities, &weights, lambda)?;
    }

    Ok((
        baseline,
        BaselineReport {
            iterations: max_iter,
            converged: false,
            tolerance: final_tolerance,
        },
    ))
}

fn solve_whittaker(intensities: &[f64], weights: &[f64], lambda: f64) -> Result<Vec<f64>> {
    let len = intensities.len();
    let mut matrix = DMatrix::<f64>::zeros(len, len);
    for (index, weight) in weights.iter().copied().enumerate() {
        matrix[(index, index)] = weight;
    }
    add_second_difference_penalty(&mut matrix, lambda);

    let rhs = DVector::from_iterator(
        len,
        intensities
            .iter()
            .copied()
            .zip(weights.iter().copied())
            .map(|(observed, weight)| observed * weight),
    );
    let solution = matrix
        .lu()
        .solve(&rhs)
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "Whittaker baseline linear solve failed".to_owned(),
        })?;
    Ok(solution.iter().copied().collect())
}

fn add_second_difference_penalty(matrix: &mut DMatrix<f64>, lambda: f64) {
    let len = matrix.nrows();
    for row in 0..(len - 2) {
        let positions = [row, row + 1, row + 2];
        let coefficients = [1.0, -2.0, 1.0];
        for (left_index, left_coefficient) in positions.iter().zip(coefficients) {
            for (right_index, right_coefficient) in positions.iter().zip(coefficients) {
                matrix[(*left_index, *right_index)] +=
                    lambda * left_coefficient * right_coefficient;
            }
        }
    }
}

fn relative_change(previous: &[f64], current: &[f64]) -> f64 {
    let numerator = previous
        .iter()
        .zip(current)
        .map(|(old, new)| {
            let difference = new - old;
            difference * difference
        })
        .sum::<f64>()
        .sqrt();
    let denominator = previous
        .iter()
        .map(|value| value * value)
        .sum::<f64>()
        .sqrt();
    numerator / denominator.max(f64::EPSILON)
}
