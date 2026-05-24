//! Hierarchical clustering option and result types.

use serde::{Deserialize, Serialize};

/// Distance metric used for hierarchical clustering.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatrixClusterMetric {
    /// Euclidean distance between row vectors.
    #[default]
    EuclideanDistance,
    /// Manhattan distance between row vectors.
    ManhattanDistance,
    /// Pearson distance, computed as `1 - correlation`.
    PearsonDistance,
    /// Cosine distance, computed as `1 - cosine_similarity`.
    CosineDistance,
}

/// Linkage strategy used to compare clusters.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatrixLinkage {
    /// Minimum pairwise member distance.
    #[default]
    Single,
    /// Maximum pairwise member distance.
    Complete,
    /// Average pairwise member distance.
    Average,
}

/// Options for agglomerative hierarchical clustering.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatrixClusteringOptions {
    /// Distance metric used between rows.
    pub metric: MatrixClusterMetric,
    /// Linkage strategy used between clusters.
    pub linkage: MatrixLinkage,
}

impl MatrixClusteringOptions {
    /// Creates default clustering options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the row distance metric.
    #[must_use]
    pub fn with_metric(mut self, metric: MatrixClusterMetric) -> Self {
        self.metric = metric;
        self
    }

    /// Sets the linkage strategy.
    #[must_use]
    pub fn with_linkage(mut self, linkage: MatrixLinkage) -> Self {
        self.linkage = linkage;
        self
    }
}

/// One dendrogram merge.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClusterMerge {
    /// Left child node id. Leaf ids are row indices; internal ids start at row count.
    pub left: usize,
    /// Right child node id. Leaf ids are row indices; internal ids start at row count.
    pub right: usize,
    /// Linkage distance at this merge.
    pub distance: f64,
    /// Number of leaves in the merged cluster.
    pub size: usize,
}

/// Hierarchical clustering dendrogram for matrix rows.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MatrixClusterResult {
    /// Row identifiers copied from the source matrix.
    pub row_ids: Vec<String>,
    /// Distance metric used for clustering.
    pub metric: MatrixClusterMetric,
    /// Linkage strategy used for clustering.
    pub linkage: MatrixLinkage,
    /// Merges in agglomerative order.
    pub merges: Vec<ClusterMerge>,
}

impl MatrixClusterResult {
    /// Returns the number of original rows/leaves.
    #[must_use]
    pub fn leaf_count(&self) -> usize {
        self.row_ids.len()
    }

    /// Returns the root node id, or `None` when the dendrogram is empty.
    #[must_use]
    pub fn root_node_id(&self) -> Option<usize> {
        if self.row_ids.len() < 2 {
            return None;
        }
        self.row_ids
            .len()
            .checked_add(self.merges.len())?
            .checked_sub(1)
    }
}
