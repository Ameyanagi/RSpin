//! Spectrum alignment utilities.

use serde::{Deserialize, Serialize};

use rspin_core::{Axis, ProcessingRecord, RSpinError, Result, Spectrum1D};

use crate::{
    MatrixGenerationOptions, Peak, PeakPickOptions, SpectrumMatrix1D, generate_spectrum_matrix_1d,
    pick_peaks,
};

mod two_d;

pub use two_d::{
    Spectrum2DAlignmentShift, ZoneAlignedMatrix2D, ZoneAlignmentOptions, ZoneAlignmentResult2D,
    align_spectra_by_zone, align_spectra_by_zone_to_matrix,
};

/// Optional coordinate window used for alignment peak selection.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct AlignmentWindow {
    /// Window start coordinate.
    pub from: f64,
    /// Window end coordinate.
    pub to: f64,
}

impl AlignmentWindow {
    /// Creates an alignment search window.
    #[must_use]
    pub fn new(from: f64, to: f64) -> Self {
        Self { from, to }
    }

    fn bounds(self) -> Result<(f64, f64)> {
        if !self.from.is_finite() {
            return Err(RSpinError::NonFinite { field: "from" });
        }
        if !self.to.is_finite() {
            return Err(RSpinError::NonFinite { field: "to" });
        }
        if self.from <= self.to {
            Ok((self.from, self.to))
        } else {
            Ok((self.to, self.from))
        }
    }

    fn contains(self, x: f64) -> Result<bool> {
        let (min, max) = self.bounds()?;
        Ok(x >= min && x <= max)
    }
}

/// Options for peak-based one-dimensional alignment.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PeakAlignmentOptions {
    /// Target coordinate. When omitted, the first spectrum's selected peak is used.
    pub target_x: Option<f64>,
    /// Optional window for selecting alignment peaks.
    pub search_window: Option<AlignmentWindow>,
    /// Peak picking options used before selecting the strongest peak.
    pub peak_options: PeakPickOptions,
}

impl PeakAlignmentOptions {
    /// Creates default peak alignment options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the target coordinate for selected alignment peaks.
    #[must_use]
    pub fn with_target_x(mut self, target_x: f64) -> Self {
        self.target_x = Some(target_x);
        self
    }

    /// Uses the first spectrum's selected peak as the target coordinate.
    #[must_use]
    pub fn without_target_x(mut self) -> Self {
        self.target_x = None;
        self
    }

    /// Sets a search window for selecting alignment peaks.
    #[must_use]
    pub fn with_search_window(mut self, window: AlignmentWindow) -> Self {
        self.search_window = Some(window);
        self
    }

    /// Clears the search window.
    #[must_use]
    pub fn without_search_window(mut self) -> Self {
        self.search_window = None;
        self
    }

    /// Sets peak-picking options used to choose alignment peaks.
    #[must_use]
    pub fn with_peak_options(mut self, peak_options: PeakPickOptions) -> Self {
        self.peak_options = peak_options;
        self
    }

    fn validate(self) -> Result<()> {
        if let Some(target_x) = self.target_x {
            if !target_x.is_finite() {
                return Err(RSpinError::NonFinite { field: "target_x" });
            }
        }
        if let Some(window) = self.search_window {
            window.bounds()?;
        }
        Ok(())
    }
}

/// Alignment metadata for one spectrum.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpectrumAlignmentShift {
    /// Deterministic row identifier.
    pub row_id: String,
    /// Selected peak index in the original spectrum.
    pub peak_index: usize,
    /// Selected peak coordinate before alignment.
    pub observed_x: f64,
    /// Target coordinate after alignment.
    pub target_x: f64,
    /// Axis shift applied to the spectrum.
    pub delta: f64,
}

/// Peak-based alignment output.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PeakAlignmentResult1D {
    /// Aligned spectra in input order.
    pub spectra: Vec<Spectrum1D>,
    /// Per-spectrum shift metadata in input order.
    pub shifts: Vec<SpectrumAlignmentShift>,
}

/// Peak-aligned matrix output for one-dimensional spectra.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PeakAlignedMatrix1D {
    /// Matrix generated from peak-aligned spectra.
    pub matrix: SpectrumMatrix1D,
    /// Per-spectrum shift metadata in input order.
    pub shifts: Vec<SpectrumAlignmentShift>,
}

/// Aligns spectra by shifting each x axis so its selected peak lands on `target_x`.
///
/// When `target_x` is omitted, the selected peak from the first spectrum is
/// used as the target. The selected peak is the highest absolute-intensity peak
/// after applying [`PeakPickOptions`] and the optional [`AlignmentWindow`].
///
/// # Errors
///
/// Returns an error when no spectra are provided, no alignment peak is found,
/// or options are invalid.
pub fn align_spectra_by_peak(
    spectra: &[Spectrum1D],
    options: PeakAlignmentOptions,
) -> Result<PeakAlignmentResult1D> {
    options.validate()?;
    if spectra.is_empty() {
        return Err(RSpinError::InvalidSpectrum {
            message: "alignment requires at least one spectrum".to_owned(),
        });
    }

    let selected_peaks = spectra
        .iter()
        .map(|spectrum| select_alignment_peak(spectrum, options))
        .collect::<Result<Vec<_>>>()?;
    let target_x = match options.target_x {
        Some(value) => value,
        None => selected_peaks[0].x,
    };

    let mut aligned = Vec::with_capacity(spectra.len());
    let mut shifts = Vec::with_capacity(spectra.len());
    for (index, (spectrum, peak)) in spectra.iter().zip(selected_peaks).enumerate() {
        let delta = target_x - peak.x;
        aligned.push(shift_spectrum_axis(spectrum, delta)?);
        shifts.push(SpectrumAlignmentShift {
            row_id: row_id(index, spectrum),
            peak_index: peak.index,
            observed_x: peak.x,
            target_x,
            delta,
        });
    }

    Ok(PeakAlignmentResult1D {
        spectra: aligned,
        shifts,
    })
}

/// Aligns spectra by peak and generates a common one-dimensional matrix.
///
/// This combines peak-based axis shifting with matrix generation for
/// multi-spectrum workflows. Matrix generation runs on the aligned spectra, so
/// its default target axis is the first aligned spectrum axis.
///
/// # Errors
///
/// Returns an error when alignment fails or matrix generation options are
/// invalid.
pub fn align_spectra_by_peak_to_matrix(
    spectra: &[Spectrum1D],
    alignment_options: PeakAlignmentOptions,
    matrix_options: MatrixGenerationOptions,
) -> Result<PeakAlignedMatrix1D> {
    let alignment = align_spectra_by_peak(spectra, alignment_options)?;
    let matrix = generate_spectrum_matrix_1d(&alignment.spectra, matrix_options)?;
    Ok(PeakAlignedMatrix1D {
        matrix,
        shifts: alignment.shifts,
    })
}

fn select_alignment_peak(spectrum: &Spectrum1D, options: PeakAlignmentOptions) -> Result<Peak> {
    let mut peaks = pick_peaks(spectrum, options.peak_options)?;
    if let Some(window) = options.search_window {
        peaks = peaks
            .into_iter()
            .filter_map(|peak| match window.contains(peak.x) {
                Ok(true) => Some(Ok(peak)),
                Ok(false) => None,
                Err(error) => Some(Err(error)),
            })
            .collect::<Result<Vec<_>>>()?;
    }

    peaks
        .into_iter()
        .max_by(|left, right| {
            left.intensity
                .abs()
                .total_cmp(&right.intensity.abs())
                .then_with(|| right.index.cmp(&left.index))
        })
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "no peak found for spectrum alignment".to_owned(),
        })
}

fn shift_spectrum_axis(spectrum: &Spectrum1D, delta: f64) -> Result<Spectrum1D> {
    let values = spectrum
        .x
        .values
        .iter()
        .map(|value| value + delta)
        .collect();
    let mut shifted = Spectrum1D::new_complex(
        Axis::new(spectrum.x.label.clone(), spectrum.x.unit, values)?,
        spectrum.intensities.clone(),
        spectrum.imaginary.clone(),
        spectrum.metadata.clone(),
    )?;
    shifted.processing.clone_from(&spectrum.processing);
    Ok(shifted.with_processing_record(
        ProcessingRecord::new("align_spectrum_by_peak").with_details(format!("delta={delta}")),
    ))
}

fn row_id(index: usize, spectrum: &Spectrum1D) -> String {
    match spectrum.metadata.name.as_deref() {
        Some(name) if !name.trim().is_empty() => format!("{index}:{}", sanitize_id_token(name)),
        _ => format!("spectrum-{index}"),
    }
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
