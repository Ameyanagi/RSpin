//! One-dimensional multiplet detection from picked peaks.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use rspin_core::{RSpinError, Result, Spectrum1D};

use crate::{MultipletDetector, Peak, PeakPolarity};

/// Options for grouping one-dimensional peaks into multiplets.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct MultipletDetectionOptions {
    /// Maximum adjacent peak gap inside one multiplet, in ppm.
    pub max_peak_gap_ppm: f64,
    /// Minimum number of peaks required for a reported group.
    pub min_peak_count: usize,
    /// Whether isolated peaks are reported as singlets.
    pub include_singlets: bool,
    /// Optional spectrometer frequency in MHz for J estimates.
    ///
    /// When omitted, `spectrum.metadata.frequency_mhz` is used if available.
    pub spectrometer_mhz: Option<f64>,
}

impl Default for MultipletDetectionOptions {
    fn default() -> Self {
        Self {
            max_peak_gap_ppm: 0.05,
            min_peak_count: 1,
            include_singlets: true,
            spectrometer_mhz: None,
        }
    }
}

impl MultipletDetectionOptions {
    /// Creates default multiplet-detection options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the maximum adjacent peak gap inside one multiplet, in ppm.
    #[must_use]
    pub fn with_max_peak_gap_ppm(mut self, max_peak_gap_ppm: f64) -> Self {
        self.max_peak_gap_ppm = max_peak_gap_ppm;
        self
    }

    /// Sets the minimum number of peaks required for a reported group.
    #[must_use]
    pub fn with_min_peak_count(mut self, min_peak_count: usize) -> Self {
        self.min_peak_count = min_peak_count;
        self
    }

    /// Sets whether isolated peaks are reported as singlets.
    #[must_use]
    pub fn with_singlets(mut self, include_singlets: bool) -> Self {
        self.include_singlets = include_singlets;
        self
    }

    /// Sets the spectrometer frequency in MHz used for J estimates.
    #[must_use]
    pub fn with_spectrometer_mhz(mut self, spectrometer_mhz: f64) -> Self {
        self.spectrometer_mhz = Some(spectrometer_mhz);
        self
    }

    /// Uses spectrum metadata for spectrometer frequency when available.
    #[must_use]
    pub fn without_spectrometer_mhz(mut self) -> Self {
        self.spectrometer_mhz = None;
        self
    }

    fn validate(self) -> Result<()> {
        require_finite("max_peak_gap_ppm", self.max_peak_gap_ppm)?;
        if self.max_peak_gap_ppm < 0.0 {
            return Err(RSpinError::InvalidSpectrum {
                message: "maximum peak gap must be non-negative".to_owned(),
            });
        }
        if self.min_peak_count == 0 {
            return Err(RSpinError::InvalidSpectrum {
                message: "minimum peak count must be positive".to_owned(),
            });
        }
        if let Some(spectrometer_mhz) = self.spectrometer_mhz {
            require_positive("spectrometer_mhz", spectrometer_mhz)?;
        }
        Ok(())
    }
}

/// Multiplet class inferred from the number of grouped lines.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MultipletKind {
    /// One line.
    Singlet,
    /// Two lines.
    Doublet,
    /// Three lines.
    Triplet,
    /// Four lines.
    Quartet,
    /// Any other grouped line count.
    Multiplet,
}

impl MultipletKind {
    fn from_peak_count(peak_count: usize) -> Self {
        match peak_count {
            1 => Self::Singlet,
            2 => Self::Doublet,
            3 => Self::Triplet,
            4 => Self::Quartet,
            _ => Self::Multiplet,
        }
    }
}

/// A detected one-dimensional multiplet.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DetectedMultiplet {
    /// Stable deterministic id derived from grouped peak indices.
    pub id: String,
    /// Multiplet kind inferred from peak count.
    pub kind: MultipletKind,
    /// Peaks included in ascending ppm order.
    pub peaks: Vec<Peak>,
    /// Intensity-weighted center in ppm.
    pub center_ppm: f64,
    /// Lowest ppm coordinate in the group.
    pub from_ppm: f64,
    /// Highest ppm coordinate in the group.
    pub to_ppm: f64,
    /// Sum of absolute peak intensities.
    pub total_abs_intensity: f64,
    /// Adjacent peak spacings in ppm.
    pub spacings_ppm: Vec<f64>,
    /// Mean adjacent spacing converted to Hz when a frequency is available.
    pub estimated_j_hz: Option<f64>,
}

/// Gap-based multiplet detector.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GapMultipletDetector {
    /// Detection options.
    pub options: MultipletDetectionOptions,
}

impl GapMultipletDetector {
    /// Creates a gap-based multiplet detector with default options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets multiplet-detection options.
    #[must_use]
    pub fn with_options(mut self, options: MultipletDetectionOptions) -> Self {
        self.options = options;
        self
    }
}

impl MultipletDetector for GapMultipletDetector {
    fn detect(&self, spectrum: &Spectrum1D, peaks: &[Peak]) -> Result<Vec<DetectedMultiplet>> {
        detect_multiplets(spectrum, peaks, self.options)
    }
}

/// Groups picked peaks into one-dimensional multiplets.
///
/// Peaks are sorted by ppm before grouping. A new group starts when the adjacent
/// gap exceeds [`MultipletDetectionOptions::max_peak_gap_ppm`] or when adjacent
/// peak polarities are incompatible.
///
/// # Errors
///
/// Returns an error when options are invalid or peak data cannot be mapped onto
/// the source spectrum.
pub fn detect_multiplets(
    spectrum: &Spectrum1D,
    peaks: &[Peak],
    options: MultipletDetectionOptions,
) -> Result<Vec<DetectedMultiplet>> {
    options.validate()?;
    let spectrometer_mhz = spectrometer_frequency(spectrum, options.spectrometer_mhz)?;
    let mut sorted_peaks = validate_and_sort_peaks(spectrum, peaks)?;
    if sorted_peaks.is_empty() {
        return Ok(Vec::new());
    }

    let mut groups = Vec::new();
    let mut current = Vec::new();
    for peak in sorted_peaks.drain(..) {
        if should_start_new_group(current.last(), &peak, options.max_peak_gap_ppm) {
            maybe_push_group(&mut groups, current, options, spectrometer_mhz)?;
            current = Vec::new();
        }
        current.push(peak);
    }
    maybe_push_group(&mut groups, current, options, spectrometer_mhz)?;

    Ok(groups)
}

fn validate_and_sort_peaks(spectrum: &Spectrum1D, peaks: &[Peak]) -> Result<Vec<Peak>> {
    let mut seen_indices = BTreeSet::new();
    for peak in peaks {
        if peak.index >= spectrum.len() {
            return Err(RSpinError::InvalidSpectrum {
                message: "peak index is outside the spectrum".to_owned(),
            });
        }
        if !seen_indices.insert(peak.index) {
            return Err(RSpinError::InvalidSpectrum {
                message: "duplicate peak index".to_owned(),
            });
        }
        require_finite("peak x", peak.x)?;
        require_finite("peak intensity", peak.intensity)?;
        require_finite("peak prominence", peak.prominence)?;
    }

    let mut sorted = peaks.to_vec();
    sorted.sort_by(|left, right| {
        left.x
            .total_cmp(&right.x)
            .then_with(|| left.index.cmp(&right.index))
    });
    Ok(sorted)
}

fn spectrometer_frequency(spectrum: &Spectrum1D, configured: Option<f64>) -> Result<Option<f64>> {
    let spectrometer_mhz = configured.or(spectrum.metadata.frequency_mhz);
    if let Some(value) = spectrometer_mhz {
        require_positive("spectrometer_mhz", value)?;
    }
    Ok(spectrometer_mhz)
}

fn should_start_new_group(previous: Option<&Peak>, peak: &Peak, max_peak_gap_ppm: f64) -> bool {
    let Some(previous) = previous else {
        return false;
    };

    (peak.x - previous.x).abs() > max_peak_gap_ppm
        || !polarities_compatible(previous.polarity, peak.polarity)
}

fn polarities_compatible(left: PeakPolarity, right: PeakPolarity) -> bool {
    left == right || matches!(left, PeakPolarity::Both) || matches!(right, PeakPolarity::Both)
}

fn maybe_push_group(
    groups: &mut Vec<DetectedMultiplet>,
    peaks: Vec<Peak>,
    options: MultipletDetectionOptions,
    spectrometer_mhz: Option<f64>,
) -> Result<()> {
    if peaks.is_empty() {
        return Ok(());
    }
    if peaks.len() < options.min_peak_count {
        return Ok(());
    }
    if peaks.len() == 1 && !options.include_singlets {
        return Ok(());
    }

    groups.push(build_multiplet(peaks, spectrometer_mhz)?);
    Ok(())
}

fn build_multiplet(peaks: Vec<Peak>, spectrometer_mhz: Option<f64>) -> Result<DetectedMultiplet> {
    let from_ppm = peaks[0].x;
    let to_ppm = peaks[peaks.len() - 1].x;
    let total_abs_intensity = peaks.iter().map(|peak| peak.intensity.abs()).sum::<f64>();
    let center_ppm = weighted_center(&peaks, total_abs_intensity)?;
    let spacings_ppm = peak_spacings(&peaks);
    let estimated_j_hz = mean_spacing_ppm(&spacings_ppm)?
        .and_then(|spacing| spectrometer_mhz.map(|frequency| spacing * frequency));
    let id = multiplet_id(&peaks);
    let kind = MultipletKind::from_peak_count(peaks.len());

    Ok(DetectedMultiplet {
        id,
        kind,
        peaks,
        center_ppm,
        from_ppm,
        to_ppm,
        total_abs_intensity,
        spacings_ppm,
        estimated_j_hz,
    })
}

fn weighted_center(peaks: &[Peak], total_abs_intensity: f64) -> Result<f64> {
    if total_abs_intensity > 0.0 {
        return Ok(peaks
            .iter()
            .map(|peak| peak.x * peak.intensity.abs())
            .sum::<f64>()
            / total_abs_intensity);
    }

    let count = f64::from(
        u32::try_from(peaks.len()).map_err(|_| RSpinError::InvalidSpectrum {
            message: "too many peaks to average".to_owned(),
        })?,
    );
    Ok(peaks.iter().map(|peak| peak.x).sum::<f64>() / count)
}

fn peak_spacings(peaks: &[Peak]) -> Vec<f64> {
    peaks.windows(2).map(|pair| pair[1].x - pair[0].x).collect()
}

fn mean_spacing_ppm(spacings_ppm: &[f64]) -> Result<Option<f64>> {
    if spacings_ppm.is_empty() {
        return Ok(None);
    }

    let count =
        f64::from(
            u32::try_from(spacings_ppm.len()).map_err(|_| RSpinError::InvalidSpectrum {
                message: "too many spacings to average".to_owned(),
            })?,
        );
    Ok(Some(spacings_ppm.iter().sum::<f64>() / count))
}

fn multiplet_id(peaks: &[Peak]) -> String {
    let mut first_index = peaks[0].index;
    let mut last_index = peaks[0].index;
    for peak in peaks {
        first_index = first_index.min(peak.index);
        last_index = last_index.max(peak.index);
    }
    format!("multiplet1d:{first_index}-{last_index}")
}

fn require_finite(field: &'static str, value: f64) -> Result<()> {
    if !value.is_finite() {
        return Err(RSpinError::NonFinite { field });
    }
    Ok(())
}

fn require_positive(field: &'static str, value: f64) -> Result<()> {
    require_finite(field, value)?;
    if value <= 0.0 {
        return Err(RSpinError::InvalidSpectrum {
            message: format!("{field} must be positive"),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests;
