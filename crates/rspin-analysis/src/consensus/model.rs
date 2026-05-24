//! Consensus peak option and result types.

use serde::{Deserialize, Serialize};

use rspin_core::{RSpinError, Result};

use crate::{DetectedRange, Peak, PeakPickOptions, RangeDetectionOptions};

/// Options for building a consensus peak table across one-dimensional spectra.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConsensusPeakOptions {
    /// Maximum peak coordinate span inside one consensus group.
    pub max_shift: f64,
    /// Minimum number of spectra represented by a reported group.
    pub min_spectrum_count: usize,
    /// Peak picking options applied to each input spectrum.
    pub peak_options: PeakPickOptions,
}

impl Default for ConsensusPeakOptions {
    fn default() -> Self {
        Self {
            max_shift: 0.03,
            min_spectrum_count: 1,
            peak_options: PeakPickOptions::default(),
        }
    }
}

impl ConsensusPeakOptions {
    /// Creates default consensus peak options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the maximum coordinate span for one consensus group.
    #[must_use]
    pub fn with_max_shift(mut self, max_shift: f64) -> Self {
        self.max_shift = max_shift;
        self
    }

    /// Sets the minimum number of spectra represented by a reported group.
    #[must_use]
    pub fn with_min_spectrum_count(mut self, min_spectrum_count: usize) -> Self {
        self.min_spectrum_count = min_spectrum_count;
        self
    }

    /// Sets the peak picking options applied to each spectrum.
    #[must_use]
    pub fn with_peak_options(mut self, peak_options: PeakPickOptions) -> Self {
        self.peak_options = peak_options;
        self
    }

    pub(super) fn validate(self) -> Result<()> {
        if !self.max_shift.is_finite() {
            return Err(RSpinError::NonFinite { field: "max_shift" });
        }
        if self.max_shift < 0.0 {
            return Err(RSpinError::InvalidSpectrum {
                message: "consensus peak max_shift must be non-negative".to_owned(),
            });
        }
        if self.min_spectrum_count == 0 {
            return Err(RSpinError::InvalidSpectrum {
                message: "minimum consensus spectrum count must be positive".to_owned(),
            });
        }
        Ok(())
    }
}

/// One peak observation contributing to a consensus peak.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConsensusPeakMember1D {
    /// Deterministic row identifier for the source spectrum.
    pub row_id: String,
    /// Input spectrum index.
    pub spectrum_index: usize,
    /// Peak picked from that spectrum.
    pub peak: Peak,
}

/// One consensus peak group across one-dimensional spectra.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConsensusPeak1D {
    /// Deterministic consensus peak id.
    pub id: String,
    /// Intensity-weighted center coordinate.
    pub center_x: f64,
    /// Lowest peak coordinate in the group.
    pub from_x: f64,
    /// Highest peak coordinate in the group.
    pub to_x: f64,
    /// Number of peak observations in the group.
    pub peak_count: usize,
    /// Number of spectra represented in the group.
    pub spectrum_count: usize,
    /// Sum of absolute peak intensities.
    pub total_abs_intensity: f64,
    /// Peak observations in input spectrum order.
    pub members: Vec<ConsensusPeakMember1D>,
}

/// Options for building a consensus range table across one-dimensional spectra.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConsensusRangeOptions {
    /// Maximum coordinate gap allowed between grouped range spans.
    pub max_gap: f64,
    /// Minimum number of spectra represented by a reported group.
    pub min_spectrum_count: usize,
    /// Range detection options applied to each input spectrum.
    pub range_options: RangeDetectionOptions,
}

impl Default for ConsensusRangeOptions {
    fn default() -> Self {
        Self {
            max_gap: 0.03,
            min_spectrum_count: 1,
            range_options: RangeDetectionOptions::default(),
        }
    }
}

impl ConsensusRangeOptions {
    /// Creates default consensus range options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the maximum coordinate gap allowed between grouped ranges.
    #[must_use]
    pub fn with_max_gap(mut self, max_gap: f64) -> Self {
        self.max_gap = max_gap;
        self
    }

    /// Sets the minimum number of spectra represented by a reported group.
    #[must_use]
    pub fn with_min_spectrum_count(mut self, min_spectrum_count: usize) -> Self {
        self.min_spectrum_count = min_spectrum_count;
        self
    }

    /// Sets the range detection options applied to each spectrum.
    #[must_use]
    pub fn with_range_options(mut self, range_options: RangeDetectionOptions) -> Self {
        self.range_options = range_options;
        self
    }

    pub(super) fn validate(self) -> Result<()> {
        if !self.max_gap.is_finite() {
            return Err(RSpinError::NonFinite { field: "max_gap" });
        }
        if self.max_gap < 0.0 {
            return Err(RSpinError::InvalidSpectrum {
                message: "consensus range max_gap must be non-negative".to_owned(),
            });
        }
        if self.min_spectrum_count == 0 {
            return Err(RSpinError::InvalidSpectrum {
                message: "minimum consensus spectrum count must be positive".to_owned(),
            });
        }
        Ok(())
    }
}

/// One range observation contributing to a consensus range.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConsensusRangeMember1D {
    /// Deterministic row identifier for the source spectrum.
    pub row_id: String,
    /// Input spectrum index.
    pub spectrum_index: usize,
    /// Range detected from that spectrum.
    pub range: DetectedRange,
}

/// One consensus range group across one-dimensional spectra.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConsensusRange1D {
    /// Deterministic consensus range id.
    pub id: String,
    /// Lowest coordinate covered by the group.
    pub from: f64,
    /// Highest coordinate covered by the group.
    pub to: f64,
    /// Absolute-area-weighted center coordinate.
    pub center_x: f64,
    /// Number of range observations in the group.
    pub range_count: usize,
    /// Number of spectra represented in the group.
    pub spectrum_count: usize,
    /// Sum of absolute range areas.
    pub total_abs_area: f64,
    /// Maximum absolute intensity across member ranges.
    pub max_abs_intensity: f64,
    /// Range observations in input spectrum order.
    pub members: Vec<ConsensusRangeMember1D>,
}
