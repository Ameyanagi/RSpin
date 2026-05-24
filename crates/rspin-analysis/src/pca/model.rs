//! PCA option and result types.

use serde::{Deserialize, Serialize};

use rspin_core::{RSpinError, Result};

/// Column scaling applied before principal component analysis.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatrixScaling {
    /// Do not scale columns.
    #[default]
    None,
    /// Divide each column by its sample standard deviation.
    UnitVariance,
    /// Divide each column by the square root of its sample standard deviation.
    Pareto,
}

/// Options for principal component analysis.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatrixPcaOptions {
    /// Requested number of principal components.
    pub component_count: usize,
    /// Whether to mean-center columns before decomposition.
    pub center: bool,
    /// Column scaling applied after centering.
    pub scaling: MatrixScaling,
}

impl Default for MatrixPcaOptions {
    fn default() -> Self {
        Self {
            component_count: 2,
            center: true,
            scaling: MatrixScaling::None,
        }
    }
}

impl MatrixPcaOptions {
    /// Creates default PCA options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the requested number of components.
    #[must_use]
    pub fn with_component_count(mut self, component_count: usize) -> Self {
        self.component_count = component_count;
        self
    }

    /// Enables or disables mean centering.
    #[must_use]
    pub fn with_centering(mut self, center: bool) -> Self {
        self.center = center;
        self
    }

    /// Disables mean centering.
    #[must_use]
    pub fn without_centering(mut self) -> Self {
        self.center = false;
        self
    }

    /// Sets column scaling.
    #[must_use]
    pub fn with_scaling(mut self, scaling: MatrixScaling) -> Self {
        self.scaling = scaling;
        self
    }

    pub(super) fn validate(self) -> Result<()> {
        if self.component_count == 0 {
            return Err(RSpinError::InvalidSpectrum {
                message: "PCA component_count must be positive".to_owned(),
            });
        }
        Ok(())
    }
}

/// Principal component analysis result.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MatrixPcaResult {
    /// Row identifiers copied from the source matrix.
    pub row_ids: Vec<String>,
    /// Number of source variables/columns.
    pub column_count: usize,
    /// Number of returned components.
    pub component_count: usize,
    /// Column means used during preprocessing. Zero when centering is disabled.
    pub means: Vec<f64>,
    /// Column divisors used during preprocessing.
    pub scales: Vec<f64>,
    /// Row-major score matrix: `row_ids.len() * component_count`.
    pub scores: Vec<f64>,
    /// Component-major loading matrix: `component_count * column_count`.
    pub loadings: Vec<f64>,
    /// Eigenvalue for each returned component.
    pub explained_variance: Vec<f64>,
    /// Fraction of total variance explained by each returned component.
    pub explained_variance_ratio: Vec<f64>,
}

impl MatrixPcaResult {
    /// Returns the score matrix shape as `(rows, components)`.
    #[must_use]
    pub fn score_shape(&self) -> (usize, usize) {
        (self.row_ids.len(), self.component_count)
    }

    /// Returns the loading matrix shape as `(components, columns)`.
    #[must_use]
    pub fn loading_shape(&self) -> (usize, usize) {
        (self.component_count, self.column_count)
    }

    /// Returns one score value, or `None` when out of bounds.
    #[must_use]
    pub fn score_at(&self, row_index: usize, component_index: usize) -> Option<f64> {
        let (rows, components) = self.score_shape();
        if row_index >= rows || component_index >= components {
            return None;
        }
        self.scores
            .get(
                row_index
                    .checked_mul(components)?
                    .checked_add(component_index)?,
            )
            .copied()
    }

    /// Returns one loading value, or `None` when out of bounds.
    #[must_use]
    pub fn loading_at(&self, component_index: usize, column_index: usize) -> Option<f64> {
        let (components, columns) = self.loading_shape();
        if component_index >= components || column_index >= columns {
            return None;
        }
        self.loadings
            .get(
                component_index
                    .checked_mul(columns)?
                    .checked_add(column_index)?,
            )
            .copied()
    }
}
