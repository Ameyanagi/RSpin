//! One-dimensional spectrum referencing.

use rspin_core::{Axis, ProcessingRecord, RSpinError, Result, Spectrum1D};
use serde::{Deserialize, Serialize};

use crate::{Peak, PeakPickOptions, PeakPolarity, pick_peaks};

/// Options for peak-based axis referencing.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReferencePeakOptions {
    /// Desired x coordinate for the reference peak.
    pub target_x: f64,
    /// Half-width of the search window around [`Self::target_x`].
    pub search_window: f64,
    /// Peak polarity to consider.
    pub polarity: PeakPolarity,
    /// Minimum absolute peak intensity.
    pub min_abs_intensity: f64,
    /// Minimum local peak prominence.
    pub min_prominence: f64,
}

impl ReferencePeakOptions {
    /// Creates reference options for a target x coordinate.
    #[must_use]
    pub fn new(target_x: f64) -> Self {
        Self {
            target_x,
            search_window: 0.1,
            polarity: PeakPolarity::Positive,
            min_abs_intensity: 0.0,
            min_prominence: 0.0,
        }
    }

    /// Sets the half-width of the search window.
    #[must_use]
    pub fn with_search_window(mut self, search_window: f64) -> Self {
        self.search_window = search_window;
        self
    }

    /// Sets the accepted peak polarity.
    #[must_use]
    pub fn with_polarity(mut self, polarity: PeakPolarity) -> Self {
        self.polarity = polarity;
        self
    }

    /// Sets the minimum absolute peak intensity.
    #[must_use]
    pub fn with_min_abs_intensity(mut self, min_abs_intensity: f64) -> Self {
        self.min_abs_intensity = min_abs_intensity;
        self
    }

    /// Sets the minimum local peak prominence.
    #[must_use]
    pub fn with_min_prominence(mut self, min_prominence: f64) -> Self {
        self.min_prominence = min_prominence;
        self
    }

    fn validate(self) -> Result<()> {
        ensure_finite("reference target_x", self.target_x)?;
        ensure_finite("reference search_window", self.search_window)?;
        ensure_finite("reference min_abs_intensity", self.min_abs_intensity)?;
        ensure_finite("reference min_prominence", self.min_prominence)?;
        if self.search_window <= 0.0 {
            return Err(RSpinError::InvalidSpectrum {
                message: "reference search_window must be positive".to_owned(),
            });
        }
        if self.min_abs_intensity < 0.0 {
            return Err(RSpinError::InvalidSpectrum {
                message: "reference min_abs_intensity must be non-negative".to_owned(),
            });
        }
        if self.min_prominence < 0.0 {
            return Err(RSpinError::InvalidSpectrum {
                message: "reference min_prominence must be non-negative".to_owned(),
            });
        }
        Ok(())
    }
}

impl Default for ReferencePeakOptions {
    fn default() -> Self {
        Self::new(0.0)
    }
}

/// Result of peak-based spectrum referencing.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReferenceResult {
    /// Referenced spectrum.
    pub spectrum: Spectrum1D,
    /// Peak selected for referencing in the original spectrum.
    pub peak: Peak,
    /// Observed x coordinate before referencing.
    pub observed_x: f64,
    /// Desired x coordinate after referencing.
    pub target_x: f64,
    /// Axis shift applied to every x value.
    pub delta: f64,
}

/// References a spectrum by shifting the x axis so the selected peak lands on
/// the requested target coordinate.
///
/// # Errors
///
/// Returns an error when options are invalid, no matching peak is found, or the
/// shifted axis would be invalid.
pub fn reference_spectrum_1d(
    spectrum: &Spectrum1D,
    options: ReferencePeakOptions,
) -> Result<ReferenceResult> {
    options.validate()?;
    let peak = select_reference_peak(spectrum, options)?;
    let delta = options.target_x - peak.x;
    ensure_finite("reference axis delta", delta)?;
    let shifted_values = spectrum
        .x
        .values
        .iter()
        .map(|value| value + delta)
        .collect::<Vec<_>>();
    let mut referenced = spectrum.clone();
    referenced.x = Axis::new(
        referenced.x.label.clone(),
        referenced.x.unit,
        shifted_values,
    )?;
    referenced = referenced.with_processing_record(
        ProcessingRecord::new("reference_peak").with_details(format!(
            "observed_x={},target_x={},delta={delta}",
            peak.x, options.target_x
        )),
    );
    Ok(ReferenceResult {
        spectrum: referenced,
        observed_x: peak.x,
        target_x: options.target_x,
        delta,
        peak,
    })
}

fn select_reference_peak(spectrum: &Spectrum1D, options: ReferencePeakOptions) -> Result<Peak> {
    let peaks = pick_peaks(
        spectrum,
        PeakPickOptions::new()
            .with_min_abs_intensity(options.min_abs_intensity)
            .with_min_prominence(options.min_prominence)
            .with_polarity(options.polarity),
    )?;
    let lower = options.target_x - options.search_window;
    let upper = options.target_x + options.search_window;
    peaks
        .into_iter()
        .filter(|peak| peak.x >= lower && peak.x <= upper)
        .max_by(compare_reference_candidates)
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "reference search found no matching peak".to_owned(),
        })
}

fn compare_reference_candidates(left: &Peak, right: &Peak) -> std::cmp::Ordering {
    left.prominence
        .total_cmp(&right.prominence)
        .then_with(|| left.intensity.abs().total_cmp(&right.intensity.abs()))
}

fn ensure_finite(field: &'static str, value: f64) -> Result<()> {
    if !value.is_finite() {
        return Err(RSpinError::NonFinite { field });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use rspin_core::{Axis, Metadata, Unit};

    use super::*;

    #[test]
    fn references_spectrum_to_target_peak() -> anyhow::Result<()> {
        let spectrum = Spectrum1D::new(
            Axis::linear("shift", Unit::Ppm, 0.0, 4.0, 5)?,
            vec![0.0, 1.0, 8.0, 1.0, 0.0],
            Metadata::default(),
        )?;
        let result = reference_spectrum_1d(
            &spectrum,
            ReferencePeakOptions::new(2.1).with_search_window(0.2),
        )?;

        assert_close(result.observed_x, 2.0);
        assert_close(result.delta, 0.1);
        assert_close(result.spectrum.x.values[2], 2.1);
        assert_eq!(result.spectrum.processing[0].operation, "reference_peak");
        Ok(())
    }

    #[test]
    fn rejects_reference_when_no_peak_matches() -> anyhow::Result<()> {
        let spectrum = Spectrum1D::new(
            Axis::linear("shift", Unit::Ppm, 0.0, 4.0, 5)?,
            vec![0.0, 1.0, 8.0, 1.0, 0.0],
            Metadata::default(),
        )?;
        let error = reference_spectrum_1d(
            &spectrum,
            ReferencePeakOptions::new(0.0).with_search_window(0.2),
        )
        .expect_err("missing reference peak should fail");

        assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));
        Ok(())
    }

    fn assert_close(left: f64, right: f64) {
        assert!((left - right).abs() < 1.0e-12, "left={left}, right={right}");
    }
}
