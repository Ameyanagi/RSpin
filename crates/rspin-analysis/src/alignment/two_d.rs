//! Two-dimensional spectrum alignment utilities.

use serde::{Deserialize, Serialize};

use rspin_core::{Axis, ProcessingRecord, RSpinError, Result, Spectrum2D};

use crate::{
    DetectedZone, MatrixGeneration2DOptions, SpectrumMatrix2D, ZoneDetectionOptions, detect_zones,
    generate_spectrum_matrix_2d,
};

use super::{AlignmentWindow, sanitize_id_token};

/// Options for zone-based two-dimensional alignment.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ZoneAlignmentOptions {
    /// Target x coordinate. When omitted, the first spectrum's selected zone is used.
    pub target_x: Option<f64>,
    /// Target y coordinate. When omitted, the first spectrum's selected zone is used.
    pub target_y: Option<f64>,
    /// Optional x-coordinate window for selecting alignment zones.
    pub x_window: Option<AlignmentWindow>,
    /// Optional y-coordinate window for selecting alignment zones.
    pub y_window: Option<AlignmentWindow>,
    /// Zone detection options used before selecting the strongest zone.
    pub zone_options: ZoneDetectionOptions,
}

impl Default for ZoneAlignmentOptions {
    fn default() -> Self {
        Self {
            target_x: None,
            target_y: None,
            x_window: None,
            y_window: None,
            zone_options: ZoneDetectionOptions::new().with_threshold_abs(f64::EPSILON),
        }
    }
}

impl ZoneAlignmentOptions {
    /// Creates default zone alignment options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets both target coordinates for selected alignment zones.
    #[must_use]
    pub fn with_target(mut self, x: f64, y: f64) -> Self {
        self.target_x = Some(x);
        self.target_y = Some(y);
        self
    }

    /// Sets the target x coordinate for selected alignment zones.
    #[must_use]
    pub fn with_target_x(mut self, x: f64) -> Self {
        self.target_x = Some(x);
        self
    }

    /// Sets the target y coordinate for selected alignment zones.
    #[must_use]
    pub fn with_target_y(mut self, y: f64) -> Self {
        self.target_y = Some(y);
        self
    }

    /// Uses the first spectrum's selected zone as the target coordinate.
    #[must_use]
    pub fn without_target(mut self) -> Self {
        self.target_x = None;
        self.target_y = None;
        self
    }

    /// Sets an x-coordinate window for selecting alignment zones.
    #[must_use]
    pub fn with_x_window(mut self, window: AlignmentWindow) -> Self {
        self.x_window = Some(window);
        self
    }

    /// Sets a y-coordinate window for selecting alignment zones.
    #[must_use]
    pub fn with_y_window(mut self, window: AlignmentWindow) -> Self {
        self.y_window = Some(window);
        self
    }

    /// Sets x/y coordinate windows for selecting alignment zones.
    #[must_use]
    pub fn with_windows(mut self, x_window: AlignmentWindow, y_window: AlignmentWindow) -> Self {
        self.x_window = Some(x_window);
        self.y_window = Some(y_window);
        self
    }

    /// Clears x/y alignment windows.
    #[must_use]
    pub fn without_windows(mut self) -> Self {
        self.x_window = None;
        self.y_window = None;
        self
    }

    /// Sets zone detection options used to choose alignment zones.
    #[must_use]
    pub fn with_zone_options(mut self, zone_options: ZoneDetectionOptions) -> Self {
        self.zone_options = zone_options;
        self
    }

    fn validate(self) -> Result<()> {
        if let Some(target) = self.target_x {
            ensure_finite("target_x", target)?;
        }
        if let Some(target) = self.target_y {
            ensure_finite("target_y", target)?;
        }
        if let Some(window) = self.x_window {
            window.bounds()?;
        }
        if let Some(window) = self.y_window {
            window.bounds()?;
        }
        Ok(())
    }
}

/// Alignment metadata for one two-dimensional spectrum.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Spectrum2DAlignmentShift {
    /// Deterministic row identifier.
    pub row_id: String,
    /// Selected zone identifier in the original spectrum.
    pub zone_id: String,
    /// Selected zone x centroid before alignment.
    pub observed_x: f64,
    /// Selected zone y centroid before alignment.
    pub observed_y: f64,
    /// Target x coordinate after alignment.
    pub target_x: f64,
    /// Target y coordinate after alignment.
    pub target_y: f64,
    /// X-axis shift applied to the spectrum.
    pub delta_x: f64,
    /// Y-axis shift applied to the spectrum.
    pub delta_y: f64,
}

/// Zone-based alignment output for two-dimensional spectra.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ZoneAlignmentResult2D {
    /// Aligned spectra in input order.
    pub spectra: Vec<Spectrum2D>,
    /// Per-spectrum shift metadata in input order.
    pub shifts: Vec<Spectrum2DAlignmentShift>,
}

/// Zone-aligned matrix output for two-dimensional spectra.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ZoneAlignedMatrix2D {
    /// Matrix generated from zone-aligned spectra.
    pub matrix: SpectrumMatrix2D,
    /// Per-spectrum shift metadata in input order.
    pub shifts: Vec<Spectrum2DAlignmentShift>,
}

/// Aligns spectra by shifting x/y axes so selected zone centroids land on target coordinates.
///
/// When either target coordinate is omitted, the corresponding selected-zone
/// centroid from the first spectrum is used. The selected zone is the zone with
/// the largest summed absolute intensity after applying [`ZoneDetectionOptions`]
/// and optional x/y centroid windows.
///
/// # Errors
///
/// Returns an error when no spectra are provided, no alignment zone is found,
/// or options are invalid.
pub fn align_spectra_by_zone(
    spectra: &[Spectrum2D],
    options: ZoneAlignmentOptions,
) -> Result<ZoneAlignmentResult2D> {
    options.validate()?;
    if spectra.is_empty() {
        return Err(RSpinError::InvalidSpectrum {
            message: "2D zone alignment requires at least one spectrum".to_owned(),
        });
    }

    let selected_zones = spectra
        .iter()
        .map(|spectrum| select_alignment_zone(spectrum, options))
        .collect::<Result<Vec<_>>>()?;
    let target_x = match options.target_x {
        Some(value) => value,
        None => selected_zones[0].centroid_x,
    };
    let target_y = match options.target_y {
        Some(value) => value,
        None => selected_zones[0].centroid_y,
    };

    let mut aligned = Vec::with_capacity(spectra.len());
    let mut shifts = Vec::with_capacity(spectra.len());
    for (index, (spectrum, zone)) in spectra.iter().zip(selected_zones).enumerate() {
        let delta_x = target_x - zone.centroid_x;
        let delta_y = target_y - zone.centroid_y;
        aligned.push(shift_spectrum_axes(spectrum, delta_x, delta_y)?);
        shifts.push(Spectrum2DAlignmentShift {
            row_id: row_id(index, spectrum),
            zone_id: zone.id,
            observed_x: zone.centroid_x,
            observed_y: zone.centroid_y,
            target_x,
            target_y,
            delta_x,
            delta_y,
        });
    }

    Ok(ZoneAlignmentResult2D {
        spectra: aligned,
        shifts,
    })
}

/// Aligns spectra by zone and generates a common two-dimensional matrix.
///
/// Matrix generation runs on the aligned spectra, so its default target axes
/// are the first aligned spectrum axes.
///
/// # Errors
///
/// Returns an error when alignment fails or matrix generation options are
/// invalid.
pub fn align_spectra_by_zone_to_matrix(
    spectra: &[Spectrum2D],
    alignment_options: ZoneAlignmentOptions,
    matrix_options: MatrixGeneration2DOptions,
) -> Result<ZoneAlignedMatrix2D> {
    let alignment = align_spectra_by_zone(spectra, alignment_options)?;
    let matrix = generate_spectrum_matrix_2d(&alignment.spectra, matrix_options)?;
    Ok(ZoneAlignedMatrix2D {
        matrix,
        shifts: alignment.shifts,
    })
}

fn select_alignment_zone(
    spectrum: &Spectrum2D,
    options: ZoneAlignmentOptions,
) -> Result<DetectedZone> {
    detect_zones(spectrum, options.zone_options)?
        .into_iter()
        .filter_map(|zone| match zone_is_in_windows(&zone, options) {
            Ok(true) => Some(Ok(zone)),
            Ok(false) => None,
            Err(error) => Some(Err(error)),
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .max_by(|left, right| {
            left.sum_abs_intensity
                .total_cmp(&right.sum_abs_intensity)
                .then_with(|| left.max_abs_intensity.total_cmp(&right.max_abs_intensity))
                .then_with(|| right.id.cmp(&left.id))
        })
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "no zone found for 2D spectrum alignment".to_owned(),
        })
}

fn zone_is_in_windows(zone: &DetectedZone, options: ZoneAlignmentOptions) -> Result<bool> {
    let x_matches = match options.x_window {
        Some(window) => window.contains(zone.centroid_x)?,
        None => true,
    };
    let y_matches = match options.y_window {
        Some(window) => window.contains(zone.centroid_y)?,
        None => true,
    };
    Ok(x_matches && y_matches)
}

fn shift_spectrum_axes(spectrum: &Spectrum2D, delta_x: f64, delta_y: f64) -> Result<Spectrum2D> {
    ensure_finite("delta_x", delta_x)?;
    ensure_finite("delta_y", delta_y)?;
    let x_values = spectrum
        .x
        .values
        .iter()
        .map(|value| value + delta_x)
        .collect();
    let y_values = spectrum
        .y
        .values
        .iter()
        .map(|value| value + delta_y)
        .collect();
    let mut shifted = Spectrum2D::new_complex(
        Axis::new(spectrum.x.label.clone(), spectrum.x.unit, x_values)?,
        Axis::new(spectrum.y.label.clone(), spectrum.y.unit, y_values)?,
        spectrum.z.clone(),
        spectrum.imaginary.clone(),
        spectrum.metadata.clone(),
    )?;
    shifted.processing.clone_from(&spectrum.processing);
    shifted.annotations.clone_from(&spectrum.annotations);
    Ok(shifted.with_processing_record(
        ProcessingRecord::new("align_spectrum_by_zone")
            .with_details(format!("delta_x={delta_x},delta_y={delta_y}")),
    ))
}

fn row_id(index: usize, spectrum: &Spectrum2D) -> String {
    match spectrum.metadata.name.as_deref() {
        Some(name) if !name.trim().is_empty() => format!("{index}:{}", sanitize_id_token(name)),
        _ => format!("spectrum-{index}"),
    }
}

fn ensure_finite(field: &'static str, value: f64) -> Result<()> {
    if !value.is_finite() {
        return Err(RSpinError::NonFinite { field });
    }
    Ok(())
}

#[cfg(test)]
mod tests;
