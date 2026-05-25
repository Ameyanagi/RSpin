use rspin_core::Spectrum1D;
use serde::{Deserialize, Serialize};

/// Scoring strategy for automatic phase correction.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutoPhaseCost {
    /// ACME-style entropy of the real-part derivative plus a negative-area penalty.
    ///
    /// Based on Chen, Marion, Le Comte, J. Magn. Reson. 158 (2002) 164.
    #[default]
    AcmeEntropy,
    /// Legacy scoring: imaginary squared plus negative-real squared.
    LegacyImagNegArea,
}

/// Options for deterministic grid-search automatic phase correction.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AutoPhaseOptions {
    /// Minimum zero-order phase searched, in degrees.
    pub zero_order_min_deg: f64,
    /// Maximum zero-order phase searched, in degrees.
    pub zero_order_max_deg: f64,
    /// Zero-order search step, in degrees.
    pub zero_order_step_deg: f64,
    /// Minimum first-order phase searched, in degrees.
    pub first_order_min_deg: f64,
    /// Maximum first-order phase searched, in degrees.
    pub first_order_max_deg: f64,
    /// First-order search step, in degrees.
    pub first_order_step_deg: f64,
    /// Pivot position as a fraction of the index range, typically in `[0, 1]`.
    pub pivot_fraction: f64,
    /// Weight for residual imaginary signal (legacy cost only).
    pub imaginary_weight: f64,
    /// Weight for negative real signal (both costs).
    pub negative_weight: f64,
    /// Scoring strategy.
    pub cost: AutoPhaseCost,
    /// Polish the best grid candidate with a Nelder-Mead simplex search.
    pub refine: bool,
}

impl Default for AutoPhaseOptions {
    fn default() -> Self {
        Self {
            zero_order_min_deg: -180.0,
            zero_order_max_deg: 180.0,
            zero_order_step_deg: 10.0,
            first_order_min_deg: -180.0,
            first_order_max_deg: 180.0,
            first_order_step_deg: 30.0,
            pivot_fraction: 0.5,
            imaginary_weight: 1.0,
            negative_weight: 1000.0,
            cost: AutoPhaseCost::AcmeEntropy,
            refine: true,
        }
    }
}

impl AutoPhaseOptions {
    /// Returns options with a zero-order search range.
    #[must_use]
    pub fn zero_order_range(mut self, min_deg: f64, max_deg: f64, step_deg: f64) -> Self {
        self.zero_order_min_deg = min_deg;
        self.zero_order_max_deg = max_deg;
        self.zero_order_step_deg = step_deg;
        self
    }

    /// Returns options with a first-order search range.
    #[must_use]
    pub fn first_order_range(mut self, min_deg: f64, max_deg: f64, step_deg: f64) -> Self {
        self.first_order_min_deg = min_deg;
        self.first_order_max_deg = max_deg;
        self.first_order_step_deg = step_deg;
        self
    }

    /// Returns options with a phase pivot fraction.
    #[must_use]
    pub fn pivot_fraction(mut self, pivot_fraction: f64) -> Self {
        self.pivot_fraction = pivot_fraction;
        self
    }

    /// Returns options with custom scoring weights.
    #[must_use]
    pub fn scoring_weights(mut self, imaginary_weight: f64, negative_weight: f64) -> Self {
        self.imaginary_weight = imaginary_weight;
        self.negative_weight = negative_weight;
        self
    }

    /// Returns options with a chosen cost variant.
    #[must_use]
    pub fn with_cost(mut self, cost: AutoPhaseCost) -> Self {
        self.cost = cost;
        self
    }

    /// Returns options with refinement enabled or disabled.
    #[must_use]
    pub fn with_refine(mut self, refine: bool) -> Self {
        self.refine = refine;
        self
    }
}

/// Automatic phase correction processing step.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AutoPhaseCorrection {
    /// Search options.
    pub options: AutoPhaseOptions,
}

impl AutoPhaseCorrection {
    /// Creates an automatic phase correction step with default options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an automatic phase correction step with explicit options.
    #[must_use]
    pub fn with_options(options: AutoPhaseOptions) -> Self {
        Self { options }
    }

    /// Returns a step with a zero-order search range.
    #[must_use]
    pub fn zero_order_range(mut self, min_deg: f64, max_deg: f64, step_deg: f64) -> Self {
        self.options = self.options.zero_order_range(min_deg, max_deg, step_deg);
        self
    }

    /// Returns a step with a first-order search range.
    #[must_use]
    pub fn first_order_range(mut self, min_deg: f64, max_deg: f64, step_deg: f64) -> Self {
        self.options = self.options.first_order_range(min_deg, max_deg, step_deg);
        self
    }

    /// Returns a step with a phase pivot fraction.
    #[must_use]
    pub fn pivot_fraction(mut self, pivot_fraction: f64) -> Self {
        self.options = self.options.pivot_fraction(pivot_fraction);
        self
    }

    /// Returns a step with custom scoring weights.
    #[must_use]
    pub fn scoring_weights(mut self, imaginary_weight: f64, negative_weight: f64) -> Self {
        self.options = self
            .options
            .scoring_weights(imaginary_weight, negative_weight);
        self
    }
}

/// Result of automatic phase correction.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AutoPhaseResult {
    /// Phased spectrum.
    pub spectrum: Spectrum1D,
    /// Selected zero-order phase in degrees.
    pub zero_order_deg: f64,
    /// Selected first-order phase in degrees.
    pub first_order_deg: f64,
    /// Final score for the selected correction.
    pub score: f64,
}
