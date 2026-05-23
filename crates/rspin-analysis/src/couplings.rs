//! J-coupling graph data for assignment and analysis workflows.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use rspin_core::{Nucleus, RSpinError, Result};

/// A node in a J-coupling graph.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CouplingNode {
    /// Stable node identifier.
    pub id: String,
    /// Optional display label.
    pub label: Option<String>,
    /// Node nucleus.
    pub nucleus: Nucleus,
}

impl CouplingNode {
    /// Creates a coupling node with no display label.
    #[must_use]
    pub fn new(id: impl Into<String>, nucleus: Nucleus) -> Self {
        Self {
            id: id.into(),
            label: None,
            nucleus,
        }
    }

    /// Returns a copy with a display label.
    #[must_use]
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    fn validate(&self) -> Result<()> {
        validate_non_empty("coupling node id", &self.id)?;
        if let Some(label) = &self.label {
            validate_non_empty("coupling node label", label)?;
        }
        if let Nucleus::Other(label) = &self.nucleus {
            validate_non_empty("nucleus label", label)?;
        }
        Ok(())
    }
}

/// A scalar J coupling between two graph nodes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct JCoupling {
    /// Stable coupling id.
    pub id: String,
    /// First endpoint node id.
    pub node_a: String,
    /// Second endpoint node id.
    pub node_b: String,
    /// Coupling constant in Hz.
    pub j_hz: f64,
    /// Optional confidence in `[0, 1]`.
    pub confidence: Option<f64>,
    /// Optional provenance or source label.
    pub source: Option<String>,
}

impl JCoupling {
    /// Creates a coupling with a caller-provided id.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        node_a: impl Into<String>,
        node_b: impl Into<String>,
        j_hz: f64,
    ) -> Self {
        Self {
            id: id.into(),
            node_a: node_a.into(),
            node_b: node_b.into(),
            j_hz,
            confidence: None,
            source: None,
        }
    }

    /// Creates a coupling with a deterministic id derived from endpoint ids.
    ///
    /// # Errors
    ///
    /// Returns an error when endpoint ids or `j_hz` are invalid.
    pub fn deterministic(
        node_a: impl Into<String>,
        node_b: impl Into<String>,
        j_hz: f64,
    ) -> Result<Self> {
        let node_a = node_a.into();
        let node_b = node_b.into();
        let id = deterministic_j_coupling_id(&node_a, &node_b)?;
        let coupling = Self::new(id, node_a, node_b, j_hz);
        coupling.validate()?;
        Ok(coupling)
    }

    /// Returns a copy with confidence metadata.
    ///
    /// # Errors
    ///
    /// Returns an error when confidence is not finite or outside `[0, 1]`.
    pub fn with_confidence(mut self, confidence: f64) -> Result<Self> {
        validate_confidence(confidence)?;
        self.confidence = Some(confidence);
        Ok(self)
    }

    /// Returns a copy with source metadata.
    #[must_use]
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    fn validate(&self) -> Result<()> {
        validate_non_empty("coupling id", &self.id)?;
        validate_non_empty("coupling node_a", &self.node_a)?;
        validate_non_empty("coupling node_b", &self.node_b)?;
        if self.node_a == self.node_b {
            return Err(RSpinError::InvalidAssignment {
                message: "coupling endpoints must be different".to_owned(),
            });
        }
        ensure_finite("j_hz", self.j_hz)?;
        if let Some(confidence) = self.confidence {
            validate_confidence(confidence)?;
        }
        if let Some(source) = &self.source {
            validate_non_empty("coupling source", source)?;
        }
        Ok(())
    }
}

/// A validated graph of assigned resonances and scalar J couplings.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct JCouplingGraph {
    /// Graph nodes in insertion order.
    pub nodes: Vec<CouplingNode>,
    /// Couplings in insertion order.
    pub couplings: Vec<JCoupling>,
}

impl JCouplingGraph {
    /// Creates a validated J-coupling graph.
    ///
    /// # Errors
    ///
    /// Returns an error when nodes or couplings are invalid.
    pub fn new(nodes: Vec<CouplingNode>, couplings: Vec<JCoupling>) -> Result<Self> {
        let graph = Self { nodes, couplings };
        graph.validate()?;
        Ok(graph)
    }

    /// Validates graph consistency.
    ///
    /// # Errors
    ///
    /// Returns an error when ids are empty or duplicated, couplings reference
    /// missing nodes, endpoint pairs are duplicated, or metadata is invalid.
    pub fn validate(&self) -> Result<()> {
        let node_ids = validate_nodes(&self.nodes)?;
        validate_couplings(&self.couplings, &node_ids)
    }

    /// Returns true when the graph has no nodes and no couplings.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty() && self.couplings.is_empty()
    }

    /// Returns couplings connected to `node_id`.
    #[must_use]
    pub fn couplings_for_node(&self, node_id: &str) -> Vec<&JCoupling> {
        self.couplings
            .iter()
            .filter(|coupling| coupling.node_a == node_id || coupling.node_b == node_id)
            .collect()
    }
}

/// Builds a deterministic J-coupling id from two endpoint node ids.
///
/// # Errors
///
/// Returns an error when either endpoint id is empty or both endpoints are the
/// same.
pub fn deterministic_j_coupling_id(node_a: &str, node_b: &str) -> Result<String> {
    validate_non_empty("coupling node_a", node_a)?;
    validate_non_empty("coupling node_b", node_b)?;
    if node_a == node_b {
        return Err(RSpinError::InvalidAssignment {
            message: "coupling endpoints must be different".to_owned(),
        });
    }

    let (left, right) = ordered_pair(node_a, node_b);
    Ok(format!(
        "j:{}-{}",
        sanitize_id_token(left),
        sanitize_id_token(right)
    ))
}

fn validate_nodes(nodes: &[CouplingNode]) -> Result<BTreeSet<&str>> {
    let mut ids = BTreeSet::new();
    for node in nodes {
        node.validate()?;
        if !ids.insert(node.id.as_str()) {
            return Err(RSpinError::InvalidAssignment {
                message: format!("duplicate coupling node id '{}'", node.id),
            });
        }
    }
    Ok(ids)
}

fn validate_couplings(couplings: &[JCoupling], node_ids: &BTreeSet<&str>) -> Result<()> {
    let mut ids = BTreeSet::new();
    let mut pairs = BTreeSet::new();
    for coupling in couplings {
        coupling.validate()?;
        if !ids.insert(coupling.id.as_str()) {
            return Err(RSpinError::InvalidAssignment {
                message: format!("duplicate coupling id '{}'", coupling.id),
            });
        }
        if !node_ids.contains(coupling.node_a.as_str()) {
            return Err(RSpinError::InvalidAssignment {
                message: format!("coupling references missing node '{}'", coupling.node_a),
            });
        }
        if !node_ids.contains(coupling.node_b.as_str()) {
            return Err(RSpinError::InvalidAssignment {
                message: format!("coupling references missing node '{}'", coupling.node_b),
            });
        }
        let pair = stable_pair(&coupling.node_a, &coupling.node_b);
        if !pairs.insert(pair) {
            return Err(RSpinError::InvalidAssignment {
                message: "duplicate coupling endpoint pair".to_owned(),
            });
        }
    }
    Ok(())
}

fn stable_pair(left: &str, right: &str) -> (String, String) {
    let (left, right) = ordered_pair(left, right);
    (left.to_owned(), right.to_owned())
}

fn ordered_pair<'a>(left: &'a str, right: &'a str) -> (&'a str, &'a str) {
    if left <= right {
        (left, right)
    } else {
        (right, left)
    }
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(RSpinError::InvalidAssignment {
            message: format!("{field} must not be empty"),
        });
    }
    Ok(())
}

fn ensure_finite(field: &'static str, value: f64) -> Result<()> {
    if !value.is_finite() {
        return Err(RSpinError::NonFinite { field });
    }
    Ok(())
}

fn validate_confidence(confidence: f64) -> Result<()> {
    if !confidence.is_finite() || !(0.0..=1.0).contains(&confidence) {
        return Err(RSpinError::InvalidAssignment {
            message: "coupling confidence must be finite and between 0 and 1".to_owned(),
        });
    }
    Ok(())
}

fn sanitize_id_token(value: &str) -> String {
    value
        .trim()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.') {
                character
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests;
