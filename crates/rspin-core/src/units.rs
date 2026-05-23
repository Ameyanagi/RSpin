//! Unit labels for spectral axes and values.

use serde::{Deserialize, Serialize};

/// Units used by axes and scalar values.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Unit {
    /// Chemical shift in parts per million.
    Ppm,
    /// Frequency in hertz.
    Hertz,
    /// Time in seconds.
    Seconds,
    /// Unitless point index.
    Points,
    /// Arbitrary intensity unit.
    Arbitrary,
}
