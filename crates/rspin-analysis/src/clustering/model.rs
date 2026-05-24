//! Hierarchical clustering option and result types.

use serde::{Deserialize, Serialize};

use rspin_core::{RSpinError, Result};

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

    /// Cuts the dendrogram to a requested number of clusters.
    ///
    /// Cluster ids are assigned deterministically by the first source row in
    /// each active cluster.
    ///
    /// # Errors
    ///
    /// Returns an error when `cluster_count` is outside `1..=leaf_count`, or
    /// when the dendrogram is structurally invalid.
    pub fn cut_to_cluster_count(&self, cluster_count: usize) -> Result<MatrixClusterCut> {
        let leaf_count = self.leaf_count();
        if leaf_count == 0 {
            return Err(RSpinError::InvalidSpectrum {
                message: "cluster cut requires at least one row".to_owned(),
            });
        }
        if cluster_count == 0 || cluster_count > leaf_count {
            return Err(RSpinError::InvalidSpectrum {
                message: "cluster_count must be in 1..=leaf_count".to_owned(),
            });
        }
        let applied_merge_count =
            leaf_count
                .checked_sub(cluster_count)
                .ok_or_else(|| RSpinError::InvalidSpectrum {
                    message: "cluster cut count underflow".to_owned(),
                })?;
        self.cut_after_merges(applied_merge_count)
    }

    /// Cuts the dendrogram at a maximum linkage distance.
    ///
    /// Merges whose distance is less than or equal to `max_distance` are
    /// applied. Cluster ids are assigned deterministically by the first source
    /// row in each active cluster.
    ///
    /// # Errors
    ///
    /// Returns an error when the threshold is negative/non-finite, or when the
    /// dendrogram is structurally invalid.
    pub fn cut_at_distance(&self, max_distance: f64) -> Result<MatrixClusterCut> {
        if !max_distance.is_finite() {
            return Err(RSpinError::NonFinite {
                field: "cluster cut distance",
            });
        }
        if max_distance < 0.0 {
            return Err(RSpinError::InvalidSpectrum {
                message: "cluster cut distance must be non-negative".to_owned(),
            });
        }

        let mut applied_merge_count = 0;
        for merge in &self.merges {
            if !merge.distance.is_finite() {
                return Err(RSpinError::NonFinite {
                    field: "cluster merge distance",
                });
            }
            if merge.distance < 0.0 {
                return Err(RSpinError::InvalidSpectrum {
                    message: "cluster merge distance must be non-negative".to_owned(),
                });
            }
            if merge.distance > max_distance {
                break;
            }
            applied_merge_count += 1;
        }

        self.cut_after_merges(applied_merge_count)
    }

    fn cut_after_merges(&self, applied_merge_count: usize) -> Result<MatrixClusterCut> {
        let leaf_count = self.leaf_count();
        if leaf_count == 0 {
            return Err(RSpinError::InvalidSpectrum {
                message: "cluster cut requires at least one row".to_owned(),
            });
        }
        if applied_merge_count > self.merges.len() {
            return Err(RSpinError::InvalidSpectrum {
                message: "cluster cut requires more merges than available".to_owned(),
            });
        }

        let mut node_leaves = (0..leaf_count).map(|leaf| vec![leaf]).collect::<Vec<_>>();
        let mut active = vec![true; leaf_count];

        for (merge_index, merge) in self.merges.iter().take(applied_merge_count).enumerate() {
            let left_leaves = node_leaves.get(merge.left).cloned().ok_or_else(|| {
                RSpinError::InvalidSpectrum {
                    message: "cluster merge left node is out of bounds".to_owned(),
                }
            })?;
            let right_leaves = node_leaves.get(merge.right).cloned().ok_or_else(|| {
                RSpinError::InvalidSpectrum {
                    message: "cluster merge right node is out of bounds".to_owned(),
                }
            })?;
            if merge.left == merge.right {
                return Err(RSpinError::InvalidSpectrum {
                    message: "cluster merge cannot use the same child twice".to_owned(),
                });
            }
            if !node_is_active(&active, merge.left)? || !node_is_active(&active, merge.right)? {
                return Err(RSpinError::InvalidSpectrum {
                    message: "cluster merge references an inactive child".to_owned(),
                });
            }

            let size = left_leaves
                .len()
                .checked_add(right_leaves.len())
                .ok_or_else(|| RSpinError::InvalidSpectrum {
                    message: "cluster cut size overflow".to_owned(),
                })?;
            if merge.size != size {
                return Err(RSpinError::InvalidSpectrum {
                    message: "cluster merge size does not match child sizes".to_owned(),
                });
            }
            let expected_node_id =
                leaf_count
                    .checked_add(merge_index)
                    .ok_or_else(|| RSpinError::InvalidSpectrum {
                        message: "cluster cut node id overflow".to_owned(),
                    })?;
            if expected_node_id != node_leaves.len() {
                return Err(RSpinError::InvalidSpectrum {
                    message: "cluster cut node sequence is invalid".to_owned(),
                });
            }

            set_node_inactive(&mut active, merge.left)?;
            set_node_inactive(&mut active, merge.right)?;
            let mut leaves = left_leaves;
            leaves.extend(right_leaves);
            node_leaves.push(leaves);
            active.push(true);
        }

        cluster_cut_from_active_nodes(&self.row_ids, &node_leaves, &active)
    }
}

/// Deterministic flat clustering labels derived from a dendrogram cut.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatrixClusterCut {
    /// Row identifiers copied from the source matrix.
    pub row_ids: Vec<String>,
    /// Cluster id for each row, aligned with `row_ids`.
    pub cluster_ids: Vec<usize>,
    /// Number of clusters in this cut.
    pub cluster_count: usize,
}

impl MatrixClusterCut {
    /// Returns the number of source rows.
    #[must_use]
    pub fn row_count(&self) -> usize {
        self.row_ids.len()
    }

    /// Returns the cluster id for a row, or `None` when out of bounds.
    #[must_use]
    pub fn cluster_id_at(&self, row_index: usize) -> Option<usize> {
        self.cluster_ids.get(row_index).copied()
    }
}

fn node_is_active(active: &[bool], node_id: usize) -> Result<bool> {
    active
        .get(node_id)
        .copied()
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "cluster merge node activity is out of bounds".to_owned(),
        })
}

fn set_node_inactive(active: &mut [bool], node_id: usize) -> Result<()> {
    let slot = active
        .get_mut(node_id)
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "cluster merge node activity is out of bounds".to_owned(),
        })?;
    *slot = false;
    Ok(())
}

fn cluster_cut_from_active_nodes(
    row_ids: &[String],
    node_leaves: &[Vec<usize>],
    active: &[bool],
) -> Result<MatrixClusterCut> {
    let mut clusters = Vec::new();
    for (node_id, is_active) in active.iter().copied().enumerate() {
        if !is_active {
            continue;
        }
        let leaves = node_leaves
            .get(node_id)
            .ok_or_else(|| RSpinError::InvalidSpectrum {
                message: "cluster active node is out of bounds".to_owned(),
            })?;
        clusters.push((minimum_leaf(leaves)?, leaves.clone()));
    }
    clusters.sort_by_key(|(minimum_leaf, _)| *minimum_leaf);

    let mut cluster_ids = vec![None; row_ids.len()];
    for (cluster_index, (_, leaves)) in clusters.iter().enumerate() {
        for &leaf in leaves {
            let slot = cluster_ids
                .get_mut(leaf)
                .ok_or_else(|| RSpinError::InvalidSpectrum {
                    message: "cluster leaf is out of bounds".to_owned(),
                })?;
            if slot.is_some() {
                return Err(RSpinError::InvalidSpectrum {
                    message: "cluster leaf appears in more than one active cluster".to_owned(),
                });
            }
            *slot = Some(cluster_index);
        }
    }

    let mut resolved_cluster_ids = Vec::with_capacity(cluster_ids.len());
    for cluster_id in cluster_ids {
        resolved_cluster_ids.push(cluster_id.ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "cluster leaf is missing from active clusters".to_owned(),
        })?);
    }

    Ok(MatrixClusterCut {
        row_ids: row_ids.to_vec(),
        cluster_ids: resolved_cluster_ids,
        cluster_count: clusters.len(),
    })
}

fn minimum_leaf(leaves: &[usize]) -> Result<usize> {
    let mut iter = leaves.iter().copied();
    let first = iter.next().ok_or_else(|| RSpinError::InvalidSpectrum {
        message: "cluster contains no leaves".to_owned(),
    })?;
    Ok(iter.fold(first, usize::min))
}
