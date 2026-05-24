use serde::{Deserialize, Serialize};

use crate::{
    AssignedAtom, Assignment, DetectedMultiplet, DetectedRange, DetectedZone, JCoupling,
    MultipletKind,
};

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

/// Options for assembling two-dimensional signal summaries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalSummary2DOptions {
    /// Emit zones even when no assignment is attached.
    pub include_unassigned_zones: bool,
}

impl Default for SignalSummary2DOptions {
    fn default() -> Self {
        Self {
            include_unassigned_zones: true,
        }
    }
}

impl SignalSummary2DOptions {
    /// Creates default two-dimensional signal summary options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets whether zones without assignments should be emitted.
    #[must_use]
    pub fn with_include_unassigned_zones(mut self, include_unassigned_zones: bool) -> Self {
        self.include_unassigned_zones = include_unassigned_zones;
        self
    }
}

/// A stable two-dimensional signal summary.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SignalSummary2D {
    /// Stable signal id.
    pub id: String,
    /// Source detected zone.
    pub zone: DetectedZone,
    /// Zone center along the x axis.
    pub center_x: f64,
    /// Zone center along the y axis.
    pub center_y: f64,
    /// Left x coordinate of the zone span.
    pub x_from: f64,
    /// Right x coordinate of the zone span.
    pub x_to: f64,
    /// Lower y coordinate of the zone span.
    pub y_from: f64,
    /// Upper y coordinate of the zone span.
    pub y_to: f64,
    /// Number of active points contributing to the zone.
    pub active_points: usize,
    /// Maximum absolute intensity represented by the signal.
    pub max_abs_intensity: f64,
    /// Sum of signed intensities over active points.
    pub sum_intensity: f64,
    /// Sum of absolute intensities over active points.
    pub sum_abs_intensity: f64,
    /// Assignments targeting the zone.
    pub assignments: Vec<Assignment>,
    /// Unique atoms collected from `assignments`.
    pub atoms: Vec<AssignedAtom>,
}
