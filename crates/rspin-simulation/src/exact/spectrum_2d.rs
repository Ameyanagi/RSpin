//! Dense two-dimensional correlation rendering from exact spin-1/2 transitions.

use std::collections::BTreeSet;

use rspin_core::{Axis, Metadata, ProcessingRecord, RSpinError, Result, Spectrum2D, Unit};

use super::{
    ExactSpinOptions, ExactTransition, SpinHalfSystem, exact_spin_half_transitions,
    validate_options as validate_transition_options, validate_spin_count, validate_system,
};

mod model;

pub use model::{
    ExactSpectrum2DOptions, ExactSpectrumDecomposition2D, ExactSpinPair,
    ExactTransitionContribution2D,
};

/// Simulates a dense two-dimensional correlation map from exact transition sets.
///
/// Each directed spin pair is rendered as the outer product of the x spin's
/// exact transition set and the y spin's exact transition set. Empty
/// [`ExactSpectrum2DOptions::spin_pairs`] uses the system's scalar couplings as
/// directed pairs.
///
/// # Errors
///
/// Returns an error when the spin system, exact transition options, spin pairs,
/// or rendering options are invalid.
pub fn simulate_exact_spin_half_2d(
    system: &SpinHalfSystem,
    options: &ExactSpectrum2DOptions,
) -> Result<Spectrum2D> {
    decompose_exact_spin_half_2d(system, options).map(|result| result.spectrum)
}

/// Simulates a dense two-dimensional correlation map with per-pair contributions.
///
/// # Errors
///
/// Returns an error when the spin system, exact transition options, spin pairs,
/// or rendering options are invalid.
pub fn decompose_exact_spin_half_2d(
    system: &SpinHalfSystem,
    options: &ExactSpectrum2DOptions,
) -> Result<ExactSpectrumDecomposition2D> {
    validate_render_options(options)?;
    validate_system(system)?;
    validate_transition_options(&options.transition_options)?;
    validate_spin_count(system.spins.len(), options.transition_options.max_spins)?;
    let spin_pairs = resolved_spin_pairs(system, options);
    validate_spin_pairs(system.spins.len(), &spin_pairs)?;

    let x_axis = Axis::linear(
        "chemical shift x",
        Unit::Ppm,
        options.x_from_ppm,
        options.x_to_ppm,
        options.x_points,
    )?;
    let y_axis = Axis::linear(
        "chemical shift y",
        Unit::Ppm,
        options.y_from_ppm,
        options.y_to_ppm,
        options.y_points,
    )?;
    let contributions =
        render_contributions(system, &x_axis.values, &y_axis.values, &spin_pairs, options)?;
    let z = sum_contributions(x_axis.len(), y_axis.len(), &contributions);
    let metadata = Metadata {
        name: Some("simulated exact spin-1/2 correlation spectrum".to_owned()),
        frequency_mhz: Some(options.transition_options.spectrometer_mhz),
        ..Metadata::default()
    };

    let spectrum = Spectrum2D::new(x_axis, y_axis, z, metadata).map(|spectrum| {
        spectrum.with_processing_record(
            ProcessingRecord::new("simulate_exact_spin_half_2d").with_details(format!(
                "{} correlations rendered with {:?}",
                contributions.len(),
                options.line_shape
            )),
        )
    })?;

    Ok(ExactSpectrumDecomposition2D {
        spectrum,
        contributions,
    })
}

fn resolved_spin_pairs(
    system: &SpinHalfSystem,
    options: &ExactSpectrum2DOptions,
) -> Vec<ExactSpinPair> {
    if options.spin_pairs.is_empty() {
        return system
            .couplings
            .iter()
            .map(|coupling| ExactSpinPair::new(coupling.spin_a, coupling.spin_b))
            .collect();
    }
    options.spin_pairs.clone()
}

fn render_contributions(
    system: &SpinHalfSystem,
    x_axis: &[f64],
    y_axis: &[f64],
    spin_pairs: &[ExactSpinPair],
    options: &ExactSpectrum2DOptions,
) -> Result<Vec<ExactTransitionContribution2D>> {
    let seeds = transition_pair_seeds(system, spin_pairs, &options.transition_options)?;
    let total_volume = seeds.iter().map(TransitionPairSeed::volume).sum::<f64>();
    if total_volume <= 0.0 || !total_volume.is_finite() {
        return Ok(Vec::new());
    }

    seeds
        .into_iter()
        .map(|seed| {
            let volume = options.volume * seed.volume() / total_volume;
            Ok(ExactTransitionContribution2D {
                x_spin: seed.pair.x_spin,
                y_spin: seed.pair.y_spin,
                x_transition: seed.x_transition,
                y_transition: seed.y_transition,
                volume,
                z: render_transition_pair(x_axis, y_axis, &seed, volume, options),
            })
        })
        .collect()
}

fn transition_pair_seeds(
    system: &SpinHalfSystem,
    spin_pairs: &[ExactSpinPair],
    transition_options: &ExactSpinOptions,
) -> Result<Vec<TransitionPairSeed>> {
    let mut seeds = Vec::new();
    for pair in spin_pairs {
        let x_transitions = exact_spin_half_transitions(
            system,
            &options_for_spin(transition_options, pair.x_spin),
        )?;
        let y_transitions = exact_spin_half_transitions(
            system,
            &options_for_spin(transition_options, pair.y_spin),
        )?;
        for x_transition in x_transitions {
            for y_transition in &y_transitions {
                let seed = TransitionPairSeed {
                    pair: *pair,
                    x_transition,
                    y_transition: *y_transition,
                };
                if seed.volume() > 0.0 && seed.volume().is_finite() {
                    seeds.push(seed);
                }
            }
        }
    }
    Ok(seeds)
}

fn options_for_spin(options: &ExactSpinOptions, spin: usize) -> ExactSpinOptions {
    ExactSpinOptions {
        detected_spins: vec![spin],
        ..options.clone()
    }
}

fn render_transition_pair(
    x_axis: &[f64],
    y_axis: &[f64],
    seed: &TransitionPairSeed,
    volume: f64,
    options: &ExactSpectrum2DOptions,
) -> Vec<f64> {
    let x_profile = axis_profile(x_axis, seed.x_transition, options.x_line_width_hz, options);
    let y_profile = axis_profile(y_axis, seed.y_transition, options.y_line_width_hz, options);

    let mut z = vec![0.0; x_axis.len() * y_axis.len()];
    for (y_index, y_value) in y_profile.iter().copied().enumerate() {
        let row_offset = y_index * x_axis.len();
        for (x_index, x_value) in x_profile.iter().copied().enumerate() {
            z[row_offset + x_index] = volume * x_value * y_value;
        }
    }
    z
}

fn axis_profile(
    axis: &[f64],
    transition: ExactTransition,
    line_width_hz: f64,
    options: &ExactSpectrum2DOptions,
) -> Vec<f64> {
    axis.iter()
        .copied()
        .map(|ppm| {
            options.line_shape.value(
                ppm,
                transition.center_ppm,
                line_width_hz,
                options.transition_options.spectrometer_mhz,
                1.0,
            )
        })
        .collect()
}

fn sum_contributions(
    x_points: usize,
    y_points: usize,
    contributions: &[ExactTransitionContribution2D],
) -> Vec<f64> {
    let mut z = vec![0.0; x_points * y_points];
    for contribution in contributions {
        for (index, value) in contribution.z.iter().copied().enumerate() {
            z[index] += value;
        }
    }
    z
}

fn validate_render_options(options: &ExactSpectrum2DOptions) -> Result<()> {
    require_finite("x_from_ppm", options.x_from_ppm)?;
    require_finite("x_to_ppm", options.x_to_ppm)?;
    require_finite("y_from_ppm", options.y_from_ppm)?;
    require_finite("y_to_ppm", options.y_to_ppm)?;
    require_positive("volume", options.volume)?;
    require_positive("x_line_width_hz", options.x_line_width_hz)?;
    require_positive("y_line_width_hz", options.y_line_width_hz)?;
    if options.x_points == 0 || options.y_points == 0 {
        return Err(RSpinError::InvalidSpectrum {
            message: "2D simulation point counts must be positive".to_owned(),
        });
    }
    Ok(())
}

fn validate_spin_pairs(spin_count: usize, spin_pairs: &[ExactSpinPair]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for pair in spin_pairs {
        if pair.x_spin >= spin_count || pair.y_spin >= spin_count {
            return Err(RSpinError::InvalidSpectrum {
                message: "2D simulation spin pair references a spin outside the system".to_owned(),
            });
        }
        if pair.x_spin == pair.y_spin {
            return Err(RSpinError::InvalidSpectrum {
                message: "2D simulation spin pair must reference two different spins".to_owned(),
            });
        }
        if !seen.insert(*pair) {
            return Err(RSpinError::InvalidSpectrum {
                message: "duplicate 2D simulation spin pair".to_owned(),
            });
        }
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

#[derive(Clone, Copy, Debug, PartialEq)]
struct TransitionPairSeed {
    pair: ExactSpinPair,
    x_transition: ExactTransition,
    y_transition: ExactTransition,
}

impl TransitionPairSeed {
    fn volume(&self) -> f64 {
        self.x_transition.intensity * self.y_transition.intensity
    }
}

#[cfg(test)]
mod tests;
