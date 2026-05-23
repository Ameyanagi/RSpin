//! First-order one-dimensional simulation.

use nalgebra::DVector;
use serde::{Deserialize, Serialize};

use rspin_core::{Axis, Metadata, RSpinError, Result, Spectrum1D, Unit};

use crate::{LineShape, Simulator};

/// A transition in a simulated peak list.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Transition {
    /// Transition center in ppm.
    pub center_ppm: f64,
    /// Transition frequency in Hz relative to the transmitter reference.
    pub frequency_hz: f64,
    /// Relative or absolute transition intensity.
    pub intensity: f64,
}

/// A group of equivalent spin-1/2 neighbors.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct CouplingGroup {
    /// Scalar coupling constant in Hz.
    pub j_hz: f64,
    /// Number of equivalent spin-1/2 neighbors.
    pub equivalent_spins: u32,
}

/// A first-order multiplet model.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FirstOrderMultiplet {
    /// Multiplet center in ppm.
    pub center_ppm: f64,
    /// Integrated multiplet area.
    pub area: f64,
    /// Coupling groups.
    pub couplings: Vec<CouplingGroup>,
}

/// Peak-list generation options.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct FirstOrderOptions {
    /// Spectrometer frequency in MHz.
    pub spectrometer_mhz: f64,
    /// Merge transitions this close in Hz.
    pub merge_tolerance_hz: f64,
}

impl Default for FirstOrderOptions {
    fn default() -> Self {
        Self {
            spectrometer_mhz: 400.0,
            merge_tolerance_hz: 1.0e-9,
        }
    }
}

/// Dense one-dimensional simulation options.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SimulationOptions {
    /// Left axis bound in ppm.
    pub from_ppm: f64,
    /// Right axis bound in ppm.
    pub to_ppm: f64,
    /// Number of output points.
    pub points: usize,
    /// Full width at half maximum in Hz.
    pub line_width_hz: f64,
    /// Spectrometer frequency in MHz.
    pub spectrometer_mhz: f64,
    /// Line shape.
    pub line_shape: LineShape,
}

impl Default for SimulationOptions {
    fn default() -> Self {
        Self {
            from_ppm: -1.0,
            to_ppm: 12.0,
            points: 16_384,
            line_width_hz: 1.0,
            spectrometer_mhz: 400.0,
            line_shape: LineShape::Lorentzian,
        }
    }
}

impl Simulator<FirstOrderMultiplet> for SimulationOptions {
    type Output = Spectrum1D;

    fn simulate(&self, model: &FirstOrderMultiplet) -> Result<Self::Output> {
        simulate_multiplet_1d(model, *self)
    }
}

/// Generates first-order transitions for a multiplet.
///
/// # Errors
///
/// Returns an error when model or options contain invalid numeric values.
pub fn multiplet_transitions(
    multiplet: &FirstOrderMultiplet,
    options: FirstOrderOptions,
) -> Result<Vec<Transition>> {
    validate_multiplet(multiplet)?;
    validate_first_order_options(options)?;

    let mut lines = vec![(0.0_f64, multiplet.area)];
    for coupling in &multiplet.couplings {
        validate_coupling(*coupling)?;
        lines = split_lines(&lines, *coupling);
    }

    let mut transitions = lines
        .into_iter()
        .map(|(offset_hz, intensity)| {
            let center_ppm = multiplet.center_ppm + offset_hz / options.spectrometer_mhz;
            Transition {
                center_ppm,
                frequency_hz: center_ppm * options.spectrometer_mhz,
                intensity,
            }
        })
        .collect::<Vec<_>>();
    transitions.sort_by(|left, right| left.frequency_hz.total_cmp(&right.frequency_hz));

    Ok(merge_transitions(
        transitions,
        options.merge_tolerance_hz,
        options.spectrometer_mhz,
    ))
}

/// Simulates a dense one-dimensional spectrum for a first-order multiplet.
///
/// # Errors
///
/// Returns an error when model or options contain invalid numeric values.
pub fn simulate_multiplet_1d(
    multiplet: &FirstOrderMultiplet,
    options: SimulationOptions,
) -> Result<Spectrum1D> {
    validate_simulation_options(options)?;
    let transitions = multiplet_transitions(
        multiplet,
        FirstOrderOptions {
            spectrometer_mhz: options.spectrometer_mhz,
            ..FirstOrderOptions::default()
        },
    )?;
    let axis = Axis::linear(
        "chemical shift",
        Unit::Ppm,
        options.from_ppm,
        options.to_ppm,
        options.points,
    )?;
    let intensities = synthesize(&axis.values, &transitions, options);

    let metadata = Metadata {
        name: Some("simulated first-order multiplet".to_owned()),
        frequency_mhz: Some(options.spectrometer_mhz),
        ..Metadata::default()
    };

    Spectrum1D::new(axis, intensities, metadata)
}

fn split_lines(lines: &[(f64, f64)], coupling: CouplingGroup) -> Vec<(f64, f64)> {
    let denominator = 2.0_f64.powf(f64::from(coupling.equivalent_spins));
    let half = f64::from(coupling.equivalent_spins) / 2.0;
    let target_len = usize::try_from(coupling.equivalent_spins)
        .map_or(lines.len(), |spins| lines.len() * (spins + 1));
    let mut split = Vec::with_capacity(target_len);

    for &(offset_hz, intensity) in lines {
        for index in 0..=coupling.equivalent_spins {
            let coefficient = f64::from(binomial(coupling.equivalent_spins, index));
            let group_offset = (f64::from(index) - half) * coupling.j_hz;
            split.push((
                offset_hz + group_offset,
                intensity * coefficient / denominator,
            ));
        }
    }

    split
}

fn binomial(n: u32, k: u32) -> u32 {
    let k = k.min(n - k);
    (0..k).fold(1_u32, |accumulator, index| {
        accumulator * (n - index) / (index + 1)
    })
}

fn merge_transitions(
    transitions: Vec<Transition>,
    tolerance_hz: f64,
    spectrometer_mhz: f64,
) -> Vec<Transition> {
    transitions
        .into_iter()
        .fold(Vec::new(), |mut merged, transition| {
            if let Some(last) = merged.last_mut() {
                let distance = (last.frequency_hz - transition.frequency_hz).abs();
                if distance <= tolerance_hz {
                    let total = last.intensity + transition.intensity;
                    last.frequency_hz = (last.frequency_hz * last.intensity
                        + transition.frequency_hz * transition.intensity)
                        / total;
                    last.center_ppm = last.frequency_hz / spectrometer_mhz;
                    last.intensity = total;
                    return merged;
                }
            }
            merged.push(transition);
            merged
        })
}

fn synthesize(axis: &[f64], transitions: &[Transition], options: SimulationOptions) -> Vec<f64> {
    let mut values = DVector::from_element(axis.len(), 0.0);
    for transition in transitions {
        for (index, x_ppm) in axis.iter().copied().enumerate() {
            values[index] += options.line_shape.value(
                x_ppm,
                transition.center_ppm,
                options.line_width_hz,
                options.spectrometer_mhz,
                transition.intensity,
            );
        }
    }
    values.as_slice().to_vec()
}

fn validate_multiplet(multiplet: &FirstOrderMultiplet) -> Result<()> {
    require_finite("center_ppm", multiplet.center_ppm)?;
    require_finite("area", multiplet.area)?;
    if multiplet.area <= 0.0 {
        return Err(RSpinError::InvalidSpectrum {
            message: "multiplet area must be positive".to_owned(),
        });
    }
    Ok(())
}

fn validate_coupling(coupling: CouplingGroup) -> Result<()> {
    require_finite("j_hz", coupling.j_hz)?;
    if coupling.equivalent_spins == 0 {
        return Err(RSpinError::InvalidSpectrum {
            message: "coupling group must contain at least one spin".to_owned(),
        });
    }
    Ok(())
}

fn validate_first_order_options(options: FirstOrderOptions) -> Result<()> {
    require_positive("spectrometer_mhz", options.spectrometer_mhz)?;
    require_finite("merge_tolerance_hz", options.merge_tolerance_hz)?;
    if options.merge_tolerance_hz < 0.0 {
        return Err(RSpinError::InvalidSpectrum {
            message: "merge tolerance must be non-negative".to_owned(),
        });
    }
    Ok(())
}

fn validate_simulation_options(options: SimulationOptions) -> Result<()> {
    require_finite("from_ppm", options.from_ppm)?;
    require_finite("to_ppm", options.to_ppm)?;
    require_positive("line_width_hz", options.line_width_hz)?;
    require_positive("spectrometer_mhz", options.spectrometer_mhz)?;
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
mod tests {
    use super::*;

    #[test]
    fn creates_reference_like_doublet() -> anyhow::Result<()> {
        let multiplet = FirstOrderMultiplet {
            center_ppm: 7.0,
            area: 1.0,
            couplings: vec![CouplingGroup {
                j_hz: 8.0,
                equivalent_spins: 1,
            }],
        };
        let transitions = multiplet_transitions(&multiplet, FirstOrderOptions::default())?;

        assert_eq!(transitions.len(), 2);
        assert_close(transitions[0].center_ppm, 6.99);
        assert_close(transitions[0].frequency_hz, 2796.0);
        assert_close(transitions[0].intensity, 0.5);
        assert_close(transitions[1].center_ppm, 7.01);
        assert_close(transitions[1].frequency_hz, 2804.0);
        assert_close(transitions[1].intensity, 0.5);
        Ok(())
    }

    #[test]
    fn creates_binomial_triplet() -> anyhow::Result<()> {
        let multiplet = FirstOrderMultiplet {
            center_ppm: 1.0,
            area: 2.0,
            couplings: vec![CouplingGroup {
                j_hz: 10.0,
                equivalent_spins: 2,
            }],
        };
        let transitions = multiplet_transitions(&multiplet, FirstOrderOptions::default())?;
        let intensities = transitions
            .iter()
            .map(|transition| transition.intensity)
            .collect::<Vec<_>>();

        assert_eq!(transitions.len(), 3);
        assert_close(intensities[0], 0.5);
        assert_close(intensities[1], 1.0);
        assert_close(intensities[2], 0.5);
        Ok(())
    }

    #[test]
    fn merges_overlapping_transitions() -> anyhow::Result<()> {
        let multiplet = FirstOrderMultiplet {
            center_ppm: 1.0,
            area: 1.0,
            couplings: vec![
                CouplingGroup {
                    j_hz: 8.0,
                    equivalent_spins: 1,
                },
                CouplingGroup {
                    j_hz: 8.0,
                    equivalent_spins: 1,
                },
            ],
        };
        let transitions = multiplet_transitions(&multiplet, FirstOrderOptions::default())?;
        assert_eq!(transitions.len(), 3);
        assert_close(transitions[1].intensity, 0.5);
        Ok(())
    }

    #[test]
    fn simulates_dense_spectrum() -> anyhow::Result<()> {
        let multiplet = FirstOrderMultiplet {
            center_ppm: 7.0,
            area: 1.0,
            couplings: vec![CouplingGroup {
                j_hz: 8.0,
                equivalent_spins: 1,
            }],
        };
        let spectrum = simulate_multiplet_1d(
            &multiplet,
            SimulationOptions {
                from_ppm: 6.95,
                to_ppm: 7.05,
                points: 101,
                line_width_hz: 1.0,
                spectrometer_mhz: 400.0,
                line_shape: LineShape::Lorentzian,
            },
        )?;

        assert_eq!(spectrum.len(), 101);
        assert_eq!(spectrum.metadata.frequency_mhz, Some(400.0));
        assert!(spectrum.intensities.iter().all(|value| *value >= 0.0));
        Ok(())
    }

    #[test]
    fn rejects_invalid_line_width() {
        let multiplet = FirstOrderMultiplet {
            center_ppm: 1.0,
            area: 1.0,
            couplings: Vec::new(),
        };
        let error = simulate_multiplet_1d(
            &multiplet,
            SimulationOptions {
                line_width_hz: 0.0,
                ..SimulationOptions::default()
            },
        )
        .expect_err("zero line width should fail");
        assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));
    }

    fn assert_close(actual: f64, expected: f64) {
        assert!((actual - expected).abs() < 1e-12, "{actual} != {expected}");
    }
}
