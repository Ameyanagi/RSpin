use serde::{Deserialize, Serialize};

use crate::{AssignedAtom, Assignment, DetectedMultiplet, DetectedRange, JCoupling, MultipletKind};

/// Options for assembling one-dimensional signal summaries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalSummaryOptions {
    /// Emit ranges even when no multiplet is attached.
    pub include_empty_ranges: bool,
    /// Emit multiplets that are not covered by a detected range.
    pub include_orphan_multiplets: bool,
}

impl Default for SignalSummaryOptions {
    fn default() -> Self {
        Self {
            include_empty_ranges: true,
            include_orphan_multiplets: true,
        }
    }
}

/// A stable one-dimensional signal summary.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SignalSummary1D {
    /// Stable signal id.
    pub id: String,
    /// Left coordinate of the signal span in ppm.
    pub from_ppm: f64,
    /// Right coordinate of the signal span in ppm.
    pub to_ppm: f64,
    /// Intensity-weighted center when multiplets are available, otherwise span midpoint.
    pub center_ppm: f64,
    /// Source detected range, if the signal is range-backed.
    pub range: Option<DetectedRange>,
    /// Multiplets assigned to this signal.
    pub multiplets: Vec<DetectedMultiplet>,
    /// Multiplet classes present in `multiplets`.
    pub multiplet_kinds: Vec<MultipletKind>,
    /// Estimated J values from attached multiplets.
    pub estimated_j_hz: Vec<f64>,
    /// Total number of peaks across attached multiplets.
    pub peak_count: usize,
    /// Trapezoidal range area when the signal is range-backed.
    pub area: Option<f64>,
    /// Maximum absolute intensity represented by the signal.
    pub max_abs_intensity: f64,
    /// Assignments targeting the range or attached peaks.
    pub assignments: Vec<Assignment>,
    /// Unique atoms collected from `assignments`.
    pub atoms: Vec<AssignedAtom>,
    /// J couplings connected to assigned atom ids.
    pub couplings: Vec<JCoupling>,
}
