//! Public model types for exact two-dimensional correlation rendering.

use rspin_core::{Result, Spectrum2D};
use serde::{Deserialize, Serialize};

use crate::{LineShape, Simulator};

use super::{
    super::{ExactSpinOptions, ExactTransition, SpinHalfSystem},
    simulate_exact_spin_half_2d,
};

/// Directed spin pair used when rendering exact two-dimensional correlations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ExactSpinPair {
    /// Spin used for the x-axis transition set.
    pub x_spin: usize,
    /// Spin used for the y-axis transition set.
    pub y_spin: usize,
}

impl ExactSpinPair {
    /// Creates a directed x/y spin pair.
    #[must_use]
    pub const fn new(x_spin: usize, y_spin: usize) -> Self {
        Self { x_spin, y_spin }
    }
}

/// Dense two-dimensional rendering options for exact spin-1/2 correlations.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExactSpectrum2DOptions {
    /// Left x-axis bound in ppm.
    pub x_from_ppm: f64,
    /// Right x-axis bound in ppm.
    pub x_to_ppm: f64,
    /// Number of x-axis output points.
    pub x_points: usize,
    /// Lower y-axis bound in ppm.
    pub y_from_ppm: f64,
    /// Upper y-axis bound in ppm.
    pub y_to_ppm: f64,
    /// Number of y-axis output points.
    pub y_points: usize,
    /// Integrated correlation volume.
    pub volume: f64,
    /// X-axis full width at half maximum in Hz.
    pub x_line_width_hz: f64,
    /// Y-axis full width at half maximum in Hz.
    pub y_line_width_hz: f64,
    /// Line shape used in both dimensions.
    pub line_shape: LineShape,
    /// Exact transition generation options.
    pub transition_options: ExactSpinOptions,
    /// Directed spin pairs to render.
    ///
    /// When empty, scalar couplings in the spin system provide the directed
    /// pairs in input order.
    #[serde(default)]
    pub spin_pairs: Vec<ExactSpinPair>,
}

impl Default for ExactSpectrum2DOptions {
    fn default() -> Self {
        Self {
            x_from_ppm: -1.0,
            x_to_ppm: 12.0,
            x_points: 512,
            y_from_ppm: -1.0,
            y_to_ppm: 12.0,
            y_points: 512,
            volume: 1.0,
            x_line_width_hz: 1.0,
            y_line_width_hz: 1.0,
            line_shape: LineShape::Lorentzian,
            transition_options: ExactSpinOptions::default(),
            spin_pairs: Vec::new(),
        }
    }
}

impl ExactSpectrum2DOptions {
    /// Creates default exact 2D correlation rendering options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the output x-axis ppm range.
    #[must_use]
    pub fn with_x_ppm_range(mut self, from_ppm: f64, to_ppm: f64) -> Self {
        self.x_from_ppm = from_ppm;
        self.x_to_ppm = to_ppm;
        self
    }

    /// Sets the output y-axis ppm range.
    #[must_use]
    pub fn with_y_ppm_range(mut self, from_ppm: f64, to_ppm: f64) -> Self {
        self.y_from_ppm = from_ppm;
        self.y_to_ppm = to_ppm;
        self
    }

    /// Sets both output point counts.
    #[must_use]
    pub fn with_points(mut self, x_points: usize, y_points: usize) -> Self {
        self.x_points = x_points;
        self.y_points = y_points;
        self
    }

    /// Sets the integrated correlation volume.
    #[must_use]
    pub fn with_volume(mut self, volume: f64) -> Self {
        self.volume = volume;
        self
    }

    /// Sets the x-axis full width at half maximum in Hz.
    #[must_use]
    pub fn with_x_line_width_hz(mut self, line_width_hz: f64) -> Self {
        self.x_line_width_hz = line_width_hz;
        self
    }

    /// Sets the y-axis full width at half maximum in Hz.
    #[must_use]
    pub fn with_y_line_width_hz(mut self, line_width_hz: f64) -> Self {
        self.y_line_width_hz = line_width_hz;
        self
    }

    /// Sets the rendered line shape.
    #[must_use]
    pub fn with_line_shape(mut self, line_shape: LineShape) -> Self {
        self.line_shape = line_shape;
        self
    }

    /// Sets the exact transition generation options.
    #[must_use]
    pub fn with_transition_options(mut self, transition_options: ExactSpinOptions) -> Self {
        self.transition_options = transition_options;
        self
    }

    /// Adds one directed spin pair.
    #[must_use]
    pub fn with_spin_pair(mut self, x_spin: usize, y_spin: usize) -> Self {
        self.spin_pairs.push(ExactSpinPair::new(x_spin, y_spin));
        self
    }

    /// Replaces all directed spin pairs.
    #[must_use]
    pub fn with_spin_pairs(mut self, spin_pairs: impl IntoIterator<Item = ExactSpinPair>) -> Self {
        self.spin_pairs = spin_pairs.into_iter().collect();
        self
    }

    /// Uses scalar couplings from the spin system as directed spin pairs.
    #[must_use]
    pub fn without_spin_pairs(mut self) -> Self {
        self.spin_pairs.clear();
        self
    }
}

impl Simulator<SpinHalfSystem> for ExactSpectrum2DOptions {
    type Output = Spectrum2D;

    fn simulate(&self, model: &SpinHalfSystem) -> Result<Self::Output> {
        simulate_exact_spin_half_2d(model, self)
    }
}

/// One rendered contribution from an exact transition pair.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExactTransitionContribution2D {
    /// Spin used for the x-axis transition.
    pub x_spin: usize,
    /// Spin used for the y-axis transition.
    pub y_spin: usize,
    /// Exact transition rendered on the x axis.
    pub x_transition: ExactTransition,
    /// Exact transition rendered on the y axis.
    pub y_transition: ExactTransition,
    /// Integrated contribution volume.
    pub volume: f64,
    /// Contribution intensities on the shared row-major output grid.
    pub z: Vec<f64>,
}

/// Dense exact 2D correlation spectrum plus per-correlation contributions.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExactSpectrumDecomposition2D {
    /// Total simulated spectrum.
    pub spectrum: Spectrum2D,
    /// Per-correlation rendered intensities on `spectrum.x` and `spectrum.y`.
    pub contributions: Vec<ExactTransitionContribution2D>,
}
