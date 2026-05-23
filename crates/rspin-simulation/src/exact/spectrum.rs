//! Dense spectrum rendering for exact spin-1/2 transitions.

use rspin_core::{Axis, Metadata, ProcessingRecord, RSpinError, Result, Spectrum1D, Unit};
use serde::{Deserialize, Serialize};

use crate::{LineShape, Simulator};

use super::{ExactSpinOptions, ExactTransition, SpinHalfSystem, exact_spin_half_transitions};

/// Dense one-dimensional rendering options for exact spin-1/2 simulations.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExactSpectrumOptions {
    /// Left axis bound in ppm.
    pub from_ppm: f64,
    /// Right axis bound in ppm.
    pub to_ppm: f64,
    /// Number of output points.
    pub points: usize,
    /// Integrated spectrum area.
    pub area: f64,
    /// Full width at half maximum in Hz.
    pub line_width_hz: f64,
    /// Line shape used after exact transition generation.
    pub line_shape: LineShape,
    /// Exact transition generation options.
    pub transition_options: ExactSpinOptions,
}

impl Default for ExactSpectrumOptions {
    fn default() -> Self {
        Self {
            from_ppm: -1.0,
            to_ppm: 12.0,
            points: 16_384,
            area: 1.0,
            line_width_hz: 1.0,
            line_shape: LineShape::Lorentzian,
            transition_options: ExactSpinOptions::default(),
        }
    }
}

impl Simulator<SpinHalfSystem> for ExactSpectrumOptions {
    type Output = Spectrum1D;

    fn simulate(&self, model: &SpinHalfSystem) -> Result<Self::Output> {
        simulate_exact_spin_half_1d(model, self)
    }
}

/// Simulates a dense one-dimensional spectrum from exact spin-1/2 transitions.
///
/// Exact Hamiltonian transition intensities are normalized before rendering so
/// [`ExactSpectrumOptions::area`] controls the integrated area of the rendered
/// transition set.
///
/// # Errors
///
/// Returns an error when the spin system, exact transition options, or rendering
/// options are invalid.
pub fn simulate_exact_spin_half_1d(
    system: &SpinHalfSystem,
    options: &ExactSpectrumOptions,
) -> Result<Spectrum1D> {
    validate_options(options)?;
    let transitions = exact_spin_half_transitions(system, &options.transition_options)?;
    let axis = Axis::linear(
        "chemical shift",
        Unit::Ppm,
        options.from_ppm,
        options.to_ppm,
        options.points,
    )?;
    let intensities = synthesize(&axis.values, &transitions, options);
    let metadata = Metadata {
        name: Some("simulated exact spin-1/2 spectrum".to_owned()),
        frequency_mhz: Some(options.transition_options.spectrometer_mhz),
        ..Metadata::default()
    };

    Spectrum1D::new(axis, intensities, metadata).map(|spectrum| {
        spectrum.with_processing_record(
            ProcessingRecord::new("simulate_exact_spin_half_1d").with_details(format!(
                "{} transitions rendered with {:?}",
                transitions.len(),
                options.line_shape
            )),
        )
    })
}

fn synthesize(
    axis: &[f64],
    transitions: &[ExactTransition],
    options: &ExactSpectrumOptions,
) -> Vec<f64> {
    let total_intensity = transitions
        .iter()
        .map(|transition| transition.intensity)
        .sum::<f64>();
    let mut values = vec![0.0; axis.len()];
    if total_intensity <= 0.0 || !total_intensity.is_finite() {
        return values;
    }

    for transition in transitions {
        let area = options.area * transition.intensity / total_intensity;
        for (index, x_ppm) in axis.iter().copied().enumerate() {
            values[index] += options.line_shape.value(
                x_ppm,
                transition.center_ppm,
                options.line_width_hz,
                options.transition_options.spectrometer_mhz,
                area,
            );
        }
    }

    values
}

fn validate_options(options: &ExactSpectrumOptions) -> Result<()> {
    require_finite("from_ppm", options.from_ppm)?;
    require_finite("to_ppm", options.to_ppm)?;
    require_positive("area", options.area)?;
    require_positive("line_width_hz", options.line_width_hz)?;
    if options.points == 0 {
        return Err(RSpinError::InvalidSpectrum {
            message: "simulation point count must be positive".to_owned(),
        });
    }
    Ok(())
}

fn require_finite(field: &'static str, value: f64) -> Result<()> {
    if !value.is_finite() {
        return Err(RSpinError::NonFinite { field });
    }
    Ok(())
}

fn require_positive(field: &'static str, value: f64) -> Result<()> {
    require_finite(field, value)?;
    if value <= 0.0 {
        return Err(RSpinError::InvalidSpectrum {
            message: format!("{field} must be positive"),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests;
