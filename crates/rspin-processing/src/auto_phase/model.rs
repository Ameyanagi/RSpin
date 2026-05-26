use rspin_core::Spectrum1D;
use serde::{Deserialize, Serialize};

/// Top-level strategy used by [`crate::auto_phase_correct`].
///
/// `Regions` follows Zorin, Bernstein, and Cobas (Magn. Reson. Chem. 55
/// (2017) 738–746, DOI 10.1002/mrc.4586) and is the default. The legacy
/// global-cost approach (ACME or imag+neg) is still available for
/// regression tests and edge cases where the region detector cannot find
/// reliable peaks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutoPhaseStrategy {
    /// Per-region phasing followed by weighted linear regression (Zorin et
    /// al. 2017). New default.
    #[default]
    Regions,
    /// Coarse grid search over the global cost function with optional
    /// Nelder-Mead refinement.
    GlobalCost,
}

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
    /// Pivot position in the spectrum's x-axis units (ppm, Hz, etc.).
    ///
    /// When set, this overrides `pivot_fraction` by linear interpolation against
    /// the spectrum's x-axis bounds.
    pub pivot_value: Option<f64>,
    /// Optional cost-evaluation window in the spectrum's x-axis units.
    ///
    /// When set, the scoring function only sums contributions from indices whose
    /// x values fall inside `[start, end]`. Useful for restricting the search to
    /// the active spectral region and ignoring empty baseline.
    pub active_region: Option<(f64, f64)>,
    /// Weight for residual imaginary signal (legacy cost only).
    pub imaginary_weight: f64,
    /// Weight for negative real signal (both costs).
    pub negative_weight: f64,
    /// Top-level algorithm: regions (default) or global cost.
    pub strategy: AutoPhaseStrategy,
    /// Scoring strategy used by the global-cost path.
    pub cost: AutoPhaseCost,
    /// Polish the best grid candidate with a Nelder-Mead simplex search.
    pub refine: bool,
    /// Weight that penalizes large `|ph0|` and `|ph1|`.
    ///
    /// The cost adds `regularization_weight * ((ph0/180)^2 + (ph1/180)^2)`
    /// so that wrap-equivalent solutions (e.g. `ph1 = -720`) are not selected
    /// over their small-`|ph1|` equivalents when the entropy or negativity
    /// terms are nearly tied.
    pub regularization_weight: f64,
}

impl AutoPhaseStrategy {
    /// Stable lower-case token used in serde and processing-record details.
    #[must_use]
    pub fn as_token(self) -> &'static str {
        match self {
            Self::Regions => "regions",
            Self::GlobalCost => "global_cost",
        }
    }
}

impl AutoPhaseCost {
    /// Stable lower-case token used in serde and processing-record details.
    #[must_use]
    pub fn as_token(self) -> &'static str {
        match self {
            Self::AcmeEntropy => "acme_entropy",
            Self::LegacyImagNegArea => "legacy_imag_neg_area",
        }
    }
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
            pivot_value: None,
            active_region: None,
            imaginary_weight: 1.0,
            negative_weight: 1000.0,
            strategy: AutoPhaseStrategy::Regions,
            cost: AutoPhaseCost::AcmeEntropy,
            refine: true,
            regularization_weight: 0.05,
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

    /// Returns options with a chosen top-level strategy.
    #[must_use]
    pub fn with_strategy(mut self, strategy: AutoPhaseStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Returns options with refinement enabled or disabled.
    #[must_use]
    pub fn with_refine(mut self, refine: bool) -> Self {
        self.refine = refine;
        self
    }

    /// Returns options with a chosen `|ph0|`+`|ph1|` regularizer weight.
    #[must_use]
    pub fn with_regularization_weight(mut self, weight: f64) -> Self {
        self.regularization_weight = weight;
        self
    }

    /// Returns options with a pivot in the spectrum's x-axis units.
    #[must_use]
    pub fn with_pivot_value(mut self, pivot_value: f64) -> Self {
        self.pivot_value = Some(pivot_value);
        self
    }

    /// Returns options with the pivot reverted to a fraction of the index range.
    #[must_use]
    pub fn with_pivot_fraction_only(mut self, pivot_fraction: f64) -> Self {
        self.pivot_value = None;
        self.pivot_fraction = pivot_fraction;
        self
    }

    /// Returns options that score only over the supplied x-axis window.
    #[must_use]
    pub fn with_active_region(mut self, start: f64, end: f64) -> Self {
        self.active_region = Some((start, end));
        self
    }

    /// Returns options with no active-region restriction (scores the full spectrum).
    #[must_use]
    pub fn with_full_region(mut self) -> Self {
        self.active_region = None;
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
    ///
    /// **The meaning of this value depends on
    /// [`AutoPhaseOptions::strategy`] and the two are not comparable:**
    ///
    /// - [`AutoPhaseStrategy::Regions`] returns `1.0 - R²` from the
    ///   region-phase regression, so the value lies in `[0.0, 1.0]` with
    ///   `0.0` representing a perfect fit.
    /// - [`AutoPhaseStrategy::GlobalCost`] returns the raw cost-function
    ///   value (ACME entropy or the legacy imag/neg sum), which is
    ///   unbounded and scales with the spectrum length and weight
    ///   settings.
    ///
    /// Do not compare `score` across strategies to pick a "better"
    /// correction — compare the spectra themselves or pin a single
    /// strategy.
    pub score: f64,
}
