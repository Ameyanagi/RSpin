use rspin_core::Spectrum2D;

use crate::PhaseCorrection2D;

/// Options for deterministic grid-search two-dimensional automatic phase correction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AutoPhase2DOptions {
    /// Minimum x zero-order phase searched, in degrees.
    pub x_zero_order_min_deg: f64,
    /// Maximum x zero-order phase searched, in degrees.
    pub x_zero_order_max_deg: f64,
    /// X zero-order search step, in degrees.
    pub x_zero_order_step_deg: f64,
    /// Minimum x first-order phase searched, in degrees.
    pub x_first_order_min_deg: f64,
    /// Maximum x first-order phase searched, in degrees.
    pub x_first_order_max_deg: f64,
    /// X first-order search step, in degrees.
    pub x_first_order_step_deg: f64,
    /// X pivot position as a fraction of the index range.
    pub x_pivot_fraction: f64,
    /// Minimum y zero-order phase searched, in degrees.
    pub y_zero_order_min_deg: f64,
    /// Maximum y zero-order phase searched, in degrees.
    pub y_zero_order_max_deg: f64,
    /// Y zero-order search step, in degrees.
    pub y_zero_order_step_deg: f64,
    /// Minimum y first-order phase searched, in degrees.
    pub y_first_order_min_deg: f64,
    /// Maximum y first-order phase searched, in degrees.
    pub y_first_order_max_deg: f64,
    /// Y first-order search step, in degrees.
    pub y_first_order_step_deg: f64,
    /// Y pivot position as a fraction of the index range.
    pub y_pivot_fraction: f64,
    /// Weight for residual imaginary signal.
    pub imaginary_weight: f64,
    /// Weight for negative real signal.
    pub negative_weight: f64,
}

impl Default for AutoPhase2DOptions {
    fn default() -> Self {
        Self {
            x_zero_order_min_deg: -180.0,
            x_zero_order_max_deg: 180.0,
            x_zero_order_step_deg: 5.0,
            x_first_order_min_deg: 0.0,
            x_first_order_max_deg: 0.0,
            x_first_order_step_deg: 5.0,
            x_pivot_fraction: 0.5,
            y_zero_order_min_deg: 0.0,
            y_zero_order_max_deg: 0.0,
            y_zero_order_step_deg: 5.0,
            y_first_order_min_deg: 0.0,
            y_first_order_max_deg: 0.0,
            y_first_order_step_deg: 5.0,
            y_pivot_fraction: 0.5,
            imaginary_weight: 1.0,
            negative_weight: 4.0,
        }
    }
}

impl AutoPhase2DOptions {
    /// Returns options with an x zero-order search range.
    #[must_use]
    pub fn x_zero_order_range(mut self, min_deg: f64, max_deg: f64, step_deg: f64) -> Self {
        self.x_zero_order_min_deg = min_deg;
        self.x_zero_order_max_deg = max_deg;
        self.x_zero_order_step_deg = step_deg;
        self
    }

    /// Returns options with an x first-order search range.
    #[must_use]
    pub fn x_first_order_range(mut self, min_deg: f64, max_deg: f64, step_deg: f64) -> Self {
        self.x_first_order_min_deg = min_deg;
        self.x_first_order_max_deg = max_deg;
        self.x_first_order_step_deg = step_deg;
        self
    }

    /// Returns options with an x phase pivot fraction.
    #[must_use]
    pub fn x_pivot_fraction(mut self, pivot_fraction: f64) -> Self {
        self.x_pivot_fraction = pivot_fraction;
        self
    }

    /// Returns options with a y zero-order search range.
    #[must_use]
    pub fn y_zero_order_range(mut self, min_deg: f64, max_deg: f64, step_deg: f64) -> Self {
        self.y_zero_order_min_deg = min_deg;
        self.y_zero_order_max_deg = max_deg;
        self.y_zero_order_step_deg = step_deg;
        self
    }

    /// Returns options with a y first-order search range.
    #[must_use]
    pub fn y_first_order_range(mut self, min_deg: f64, max_deg: f64, step_deg: f64) -> Self {
        self.y_first_order_min_deg = min_deg;
        self.y_first_order_max_deg = max_deg;
        self.y_first_order_step_deg = step_deg;
        self
    }

    /// Returns options with a y phase pivot fraction.
    #[must_use]
    pub fn y_pivot_fraction(mut self, pivot_fraction: f64) -> Self {
        self.y_pivot_fraction = pivot_fraction;
        self
    }

    /// Returns options with custom scoring weights.
    #[must_use]
    pub fn scoring_weights(mut self, imaginary_weight: f64, negative_weight: f64) -> Self {
        self.imaginary_weight = imaginary_weight;
        self.negative_weight = negative_weight;
        self
    }
}

/// Automatic two-dimensional phase correction processing step.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AutoPhaseCorrection2D {
    /// Search options.
    pub options: AutoPhase2DOptions,
}

impl AutoPhaseCorrection2D {
    /// Creates an automatic 2D phase correction step with default options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an automatic 2D phase correction step with explicit options.
    #[must_use]
    pub fn with_options(options: AutoPhase2DOptions) -> Self {
        Self { options }
    }

    /// Returns a step with an x zero-order search range.
    #[must_use]
    pub fn x_zero_order_range(mut self, min_deg: f64, max_deg: f64, step_deg: f64) -> Self {
        self.options = self.options.x_zero_order_range(min_deg, max_deg, step_deg);
        self
    }

    /// Returns a step with an x first-order search range.
    #[must_use]
    pub fn x_first_order_range(mut self, min_deg: f64, max_deg: f64, step_deg: f64) -> Self {
        self.options = self.options.x_first_order_range(min_deg, max_deg, step_deg);
        self
    }

    /// Returns a step with an x phase pivot fraction.
    #[must_use]
    pub fn x_pivot_fraction(mut self, pivot_fraction: f64) -> Self {
        self.options = self.options.x_pivot_fraction(pivot_fraction);
        self
    }

    /// Returns a step with a y zero-order search range.
    #[must_use]
    pub fn y_zero_order_range(mut self, min_deg: f64, max_deg: f64, step_deg: f64) -> Self {
        self.options = self.options.y_zero_order_range(min_deg, max_deg, step_deg);
        self
    }

    /// Returns a step with a y first-order search range.
    #[must_use]
    pub fn y_first_order_range(mut self, min_deg: f64, max_deg: f64, step_deg: f64) -> Self {
        self.options = self.options.y_first_order_range(min_deg, max_deg, step_deg);
        self
    }

    /// Returns a step with a y phase pivot fraction.
    #[must_use]
    pub fn y_pivot_fraction(mut self, pivot_fraction: f64) -> Self {
        self.options = self.options.y_pivot_fraction(pivot_fraction);
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

/// Result of automatic two-dimensional phase correction.
#[derive(Clone, Debug, PartialEq)]
pub struct AutoPhase2DResult {
    /// Phased spectrum.
    pub spectrum: Spectrum2D,
    /// Selected phase correction.
    pub correction: PhaseCorrection2D,
    /// Final score for the selected correction.
    pub score: f64,
}
