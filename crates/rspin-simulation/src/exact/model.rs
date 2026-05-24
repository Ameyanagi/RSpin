//! Exact spin-1/2 simulation data model.

use rspin_core::Result;
use serde::{Deserialize, Serialize};

use crate::Simulator;

use super::exact_spin_half_transitions;

/// A spin-1/2 nucleus in an exact spin system.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpinHalf {
    /// Chemical shift in ppm relative to the transmitter reference.
    pub shift_ppm: f64,
}

impl SpinHalf {
    /// Creates a spin-1/2 nucleus with a chemical shift in ppm.
    #[must_use]
    pub const fn new(shift_ppm: f64) -> Self {
        Self { shift_ppm }
    }
}

/// An isotropic scalar coupling between two spin-1/2 nuclei.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScalarCoupling {
    /// Zero-based index of the first spin.
    pub spin_a: usize,
    /// Zero-based index of the second spin.
    pub spin_b: usize,
    /// Scalar coupling constant in Hz.
    pub j_hz: f64,
}

impl ScalarCoupling {
    /// Creates an isotropic scalar coupling between two zero-based spin indices.
    #[must_use]
    pub const fn new(spin_a: usize, spin_b: usize, j_hz: f64) -> Self {
        Self {
            spin_a,
            spin_b,
            j_hz,
        }
    }
}

/// A spin-1/2 system for exact transition simulation.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SpinHalfSystem {
    /// Spin definitions.
    pub spins: Vec<SpinHalf>,
    /// Scalar couplings between spins.
    pub couplings: Vec<ScalarCoupling>,
}

impl SpinHalfSystem {
    /// Creates an empty spin system.
    ///
    /// Add at least one spin before simulation.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an uncoupled spin system from chemical shifts in ppm.
    #[must_use]
    pub fn from_shifts(shifts_ppm: impl IntoIterator<Item = f64>) -> Self {
        Self {
            spins: shifts_ppm.into_iter().map(SpinHalf::new).collect(),
            couplings: Vec::new(),
        }
    }

    /// Returns a copy of the system with one additional spin.
    #[must_use]
    pub fn with_spin(mut self, shift_ppm: f64) -> Self {
        self.spins.push(SpinHalf::new(shift_ppm));
        self
    }

    /// Returns a copy of the system with one additional scalar coupling.
    #[must_use]
    pub fn with_coupling(mut self, spin_a: usize, spin_b: usize, j_hz: f64) -> Self {
        self.couplings
            .push(ScalarCoupling::new(spin_a, spin_b, j_hz));
        self
    }
}

/// Options for exact spin-1/2 transition simulation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExactSpinOptions {
    /// Spectrometer frequency in MHz.
    pub spectrometer_mhz: f64,
    /// Discard transitions with intensity at or below this threshold.
    pub intensity_threshold: f64,
    /// Merge transitions this close in Hz.
    pub frequency_tolerance_hz: f64,
    /// Per-call spin-count limit, capped by [`super::MAX_EXACT_SPINS`].
    pub max_spins: usize,
    /// Spin indices included in the transverse detection operator.
    ///
    /// An empty list detects all spins in the system.
    #[serde(default)]
    pub detected_spins: Vec<usize>,
}

impl Default for ExactSpinOptions {
    fn default() -> Self {
        Self {
            spectrometer_mhz: 400.0,
            intensity_threshold: 1.0e-12,
            frequency_tolerance_hz: 1.0e-9,
            max_spins: 10,
            detected_spins: Vec::new(),
        }
    }
}

impl ExactSpinOptions {
    /// Creates default exact transition simulation options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the spectrometer frequency in MHz.
    #[must_use]
    pub fn with_spectrometer_mhz(mut self, spectrometer_mhz: f64) -> Self {
        self.spectrometer_mhz = spectrometer_mhz;
        self
    }

    /// Sets the transition intensity threshold.
    #[must_use]
    pub fn with_intensity_threshold(mut self, intensity_threshold: f64) -> Self {
        self.intensity_threshold = intensity_threshold;
        self
    }

    /// Sets the merge tolerance for transition frequencies in Hz.
    #[must_use]
    pub fn with_frequency_tolerance_hz(mut self, frequency_tolerance_hz: f64) -> Self {
        self.frequency_tolerance_hz = frequency_tolerance_hz;
        self
    }

    /// Sets the per-call exact solver spin-count limit.
    #[must_use]
    pub fn with_max_spins(mut self, max_spins: usize) -> Self {
        self.max_spins = max_spins;
        self
    }

    /// Adds one zero-based spin index to the detection operator.
    #[must_use]
    pub fn with_detected_spin(mut self, spin_index: usize) -> Self {
        self.detected_spins.push(spin_index);
        self
    }

    /// Replaces the detected spin list.
    ///
    /// An empty list detects all spins.
    #[must_use]
    pub fn with_detected_spins(mut self, detected_spins: impl IntoIterator<Item = usize>) -> Self {
        self.detected_spins = detected_spins.into_iter().collect();
        self
    }
}

impl Simulator<SpinHalfSystem> for ExactSpinOptions {
    type Output = Vec<ExactTransition>;

    fn simulate(&self, model: &SpinHalfSystem) -> Result<Self::Output> {
        exact_spin_half_transitions(model, self)
    }
}

/// An observable exact transition line.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExactTransition {
    /// Absolute transition frequency in Hz.
    pub frequency_hz: f64,
    /// Signed transition offset in Hz relative to the transmitter reference.
    pub offset_hz: f64,
    /// Signed transition position in ppm.
    pub center_ppm: f64,
    /// Relative transition intensity.
    pub intensity: f64,
    /// Number of eigenstate transitions merged into this line.
    pub contribution_count: u32,
}
