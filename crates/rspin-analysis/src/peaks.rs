//! Peak picking for one-dimensional spectra.

use serde::{Deserialize, Serialize};

use rspin_core::{RSpinError, Result, Spectrum1D};

use crate::PeakPicker;

/// Detected peak polarity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PeakPolarity {
    /// Positive local maxima.
    Positive,
    /// Negative local minima.
    Negative,
    /// Both positive maxima and negative minima.
    Both,
}

impl PeakPolarity {
    fn accepts(self, polarity: Self) -> bool {
        matches!(self, Self::Both) || self == polarity
    }
}

/// Peak-picking options for local extrema.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PeakPickOptions {
    /// Minimum absolute peak intensity.
    pub min_abs_intensity: f64,
    /// Minimum local prominence relative to adjacent points.
    pub min_prominence: f64,
    /// Polarity to detect.
    pub polarity: PeakPolarity,
}

impl Default for PeakPickOptions {
    fn default() -> Self {
        Self {
            min_abs_intensity: 0.0,
            min_prominence: 0.0,
            polarity: PeakPolarity::Positive,
        }
    }
}

impl PeakPickOptions {
    /// Creates default peak-picking options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the minimum absolute peak intensity.
    #[must_use]
    pub fn with_min_abs_intensity(mut self, min_abs_intensity: f64) -> Self {
        self.min_abs_intensity = min_abs_intensity;
        self
    }

    /// Sets the minimum local prominence.
    #[must_use]
    pub fn with_min_prominence(mut self, min_prominence: f64) -> Self {
        self.min_prominence = min_prominence;
        self
    }

    /// Sets the detected peak polarity.
    #[must_use]
    pub fn with_polarity(mut self, polarity: PeakPolarity) -> Self {
        self.polarity = polarity;
        self
    }

    fn validate(self) -> Result<()> {
        if !self.min_abs_intensity.is_finite() {
            return Err(RSpinError::NonFinite {
                field: "min_abs_intensity",
            });
        }
        if !self.min_prominence.is_finite() {
            return Err(RSpinError::NonFinite {
                field: "min_prominence",
            });
        }
        if self.min_abs_intensity < 0.0 {
            return Err(RSpinError::InvalidSpectrum {
                message: "minimum absolute intensity must be non-negative".to_owned(),
            });
        }
        if self.min_prominence < 0.0 {
            return Err(RSpinError::InvalidSpectrum {
                message: "minimum prominence must be non-negative".to_owned(),
            });
        }
        Ok(())
    }
}

/// A detected peak in a one-dimensional spectrum.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Peak {
    /// Index in the original spectrum.
    pub index: usize,
    /// X-axis coordinate.
    pub x: f64,
    /// Intensity at `x`.
    pub intensity: f64,
    /// Local prominence relative to adjacent points.
    pub prominence: f64,
    /// Peak polarity.
    pub polarity: PeakPolarity,
}

/// Local-extrema peak picker.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LocalExtremaPeakPicker {
    /// Peak-picking options.
    pub options: PeakPickOptions,
}

impl LocalExtremaPeakPicker {
    /// Creates a local-extrema peak picker with default options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets peak-picking options.
    #[must_use]
    pub fn with_options(mut self, options: PeakPickOptions) -> Self {
        self.options = options;
        self
    }
}

impl PeakPicker for LocalExtremaPeakPicker {
    fn pick(&self, spectrum: &Spectrum1D) -> Result<Vec<Peak>> {
        pick_peaks(spectrum, self.options)
    }
}

/// Picks local-extrema peaks from a one-dimensional spectrum.
///
/// # Errors
///
/// Returns an error when options are invalid.
pub fn pick_peaks(spectrum: &Spectrum1D, options: PeakPickOptions) -> Result<Vec<Peak>> {
    options.validate()?;
    if spectrum.len() < 3 {
        return Ok(Vec::new());
    }

    let peaks = spectrum
        .intensities
        .windows(3)
        .enumerate()
        .filter_map(|(window_start, values)| {
            let index = window_start + 1;
            classify_peak(values).and_then(|(polarity, prominence)| {
                let intensity = values[1];
                if options.polarity.accepts(polarity)
                    && intensity.abs() >= options.min_abs_intensity
                    && prominence >= options.min_prominence
                {
                    Some(Peak {
                        index,
                        x: spectrum.x.values[index],
                        intensity,
                        prominence,
                        polarity,
                    })
                } else {
                    None
                }
            })
        })
        .collect();

    Ok(peaks)
}

fn classify_peak(values: &[f64]) -> Option<(PeakPolarity, f64)> {
    let [left, center, right] = values else {
        return None;
    };

    if center > left && center >= right {
        Some((PeakPolarity::Positive, center - left.max(*right)))
    } else if center < left && center <= right {
        Some((PeakPolarity::Negative, left.min(*right) - center))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use rspin_core::{Axis, Metadata, Unit};

    use super::*;

    #[test]
    fn picks_positive_local_maxima() -> anyhow::Result<()> {
        let spectrum = spectrum(&[0.0, 3.0, 1.0, 4.0, 2.0])?;
        let peaks = pick_peaks(&spectrum, PeakPickOptions::default())?;
        assert_eq!(peaks.len(), 2);
        assert_eq!(peaks[0].index, 1);
        assert_close(peaks[0].x, 1.0);
        assert_close(peaks[0].prominence, 2.0);
        assert_eq!(peaks[1].index, 3);
        Ok(())
    }

    #[test]
    fn filters_by_thresholds() -> anyhow::Result<()> {
        let spectrum = spectrum(&[0.0, 3.0, 2.8, 6.0, 2.0])?;
        let peaks = pick_peaks(
            &spectrum,
            PeakPickOptions {
                min_abs_intensity: 4.0,
                min_prominence: 2.0,
                polarity: PeakPolarity::Positive,
            },
        )?;
        assert_eq!(peaks.len(), 1);
        assert_close(peaks[0].intensity, 6.0);
        Ok(())
    }

    #[test]
    fn picks_negative_peaks() -> anyhow::Result<()> {
        let spectrum = spectrum(&[0.0, -3.0, -1.0, 2.0, -5.0, 0.0])?;
        let picker = LocalExtremaPeakPicker::new()
            .with_options(PeakPickOptions::new().with_polarity(PeakPolarity::Both));
        let peaks = picker.pick(&spectrum)?;
        assert_eq!(
            peaks.iter().map(|peak| peak.polarity).collect::<Vec<_>>(),
            vec![
                PeakPolarity::Negative,
                PeakPolarity::Positive,
                PeakPolarity::Negative
            ]
        );
        Ok(())
    }

    #[test]
    fn builder_options_filter_peaks() -> anyhow::Result<()> {
        let spectrum = spectrum(&[0.0, 3.0, 2.8, 6.0, 2.0])?;
        let peaks = pick_peaks(
            &spectrum,
            PeakPickOptions::new()
                .with_min_abs_intensity(4.0)
                .with_min_prominence(2.0)
                .with_polarity(PeakPolarity::Positive),
        )?;

        assert_eq!(peaks.len(), 1);
        assert_close(peaks[0].intensity, 6.0);
        Ok(())
    }

    #[test]
    fn rejects_invalid_options() -> anyhow::Result<()> {
        let spectrum = spectrum(&[0.0, 1.0, 0.0])?;
        let error = pick_peaks(
            &spectrum,
            PeakPickOptions {
                min_abs_intensity: -1.0,
                ..PeakPickOptions::default()
            },
        )
        .expect_err("negative threshold should fail");
        assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));
        Ok(())
    }

    fn spectrum(intensities: &[f64]) -> anyhow::Result<Spectrum1D> {
        let end = f64::from(u32::try_from(intensities.len() - 1)?);
        let axis = Axis::linear("x", Unit::Ppm, 0.0, end, intensities.len())?;
        Ok(Spectrum1D::new(
            axis,
            intensities.to_vec(),
            Metadata::default(),
        )?)
    }

    fn assert_close(actual: f64, expected: f64) {
        assert!((actual - expected).abs() < 1e-12, "{actual} != {expected}");
    }
}
