//! Two-dimensional zone detection.

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use rspin_core::{RSpinError, Result, Spectrum2D};

use crate::ZoneDetector;

/// Neighborhood connectivity used when grouping active 2D points.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZoneConnectivity {
    /// Connect points through horizontal and vertical neighbors.
    #[default]
    Four,
    /// Connect points through horizontal, vertical, and diagonal neighbors.
    Eight,
}

/// Options for threshold-based 2D zone detection.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ZoneDetectionOptions {
    /// Minimum absolute intensity for a point to be considered active.
    pub threshold_abs: f64,
    /// Minimum number of active points in a detected zone.
    pub min_active_points: usize,
    /// Neighbor connectivity for grouping active points.
    pub connectivity: ZoneConnectivity,
}

impl Default for ZoneDetectionOptions {
    fn default() -> Self {
        Self {
            threshold_abs: 0.0,
            min_active_points: 1,
            connectivity: ZoneConnectivity::Four,
        }
    }
}

impl ZoneDetectionOptions {
    fn validate(self) -> Result<()> {
        if !self.threshold_abs.is_finite() {
            return Err(RSpinError::NonFinite {
                field: "threshold_abs",
            });
        }
        if self.threshold_abs < 0.0 {
            return Err(RSpinError::InvalidSpectrum {
                message: "zone threshold must be non-negative".to_owned(),
            });
        }
        if self.min_active_points == 0 {
            return Err(RSpinError::InvalidSpectrum {
                message: "minimum active points must be positive".to_owned(),
            });
        }
        Ok(())
    }
}

/// A detected connected zone in a two-dimensional spectrum.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DetectedZone {
    /// Deterministic identifier derived from index bounds.
    pub id: String,
    /// First x index included in the zone bounding box.
    pub x_start_index: usize,
    /// Last x index included in the zone bounding box.
    pub x_end_index: usize,
    /// First y index included in the zone bounding box.
    pub y_start_index: usize,
    /// Last y index included in the zone bounding box.
    pub y_end_index: usize,
    /// Coordinate at `x_start_index`.
    pub x_from: f64,
    /// Coordinate at `x_end_index`.
    pub x_to: f64,
    /// Coordinate at `y_start_index`.
    pub y_from: f64,
    /// Coordinate at `y_end_index`.
    pub y_to: f64,
    /// Absolute-intensity weighted x centroid.
    pub centroid_x: f64,
    /// Absolute-intensity weighted y centroid.
    pub centroid_y: f64,
    /// Number of active points contributing to the zone.
    pub active_points: usize,
    /// Maximum absolute intensity inside the zone.
    pub max_abs_intensity: f64,
    /// Sum of signed intensities over active points.
    pub sum_intensity: f64,
    /// Sum of absolute intensities over active points.
    pub sum_abs_intensity: f64,
}

/// Threshold-based 2D zone detector.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ThresholdZoneDetector {
    /// Detection options.
    pub options: ZoneDetectionOptions,
}

impl ZoneDetector for ThresholdZoneDetector {
    fn detect(&self, spectrum: &Spectrum2D) -> Result<Vec<DetectedZone>> {
        detect_zones(spectrum, self.options)
    }
}

#[derive(Clone, Debug)]
struct ZoneAccumulator {
    min_x: usize,
    max_x: usize,
    min_y: usize,
    max_y: usize,
    active_points: usize,
    max_abs_intensity: f64,
    sum_intensity: f64,
    sum_abs_intensity: f64,
    weighted_x_sum: f64,
    weighted_y_sum: f64,
}

impl ZoneAccumulator {
    fn new(x_index: usize, y_index: usize, intensity: f64, spectrum: &Spectrum2D) -> Self {
        let abs_intensity = intensity.abs();
        Self {
            min_x: x_index,
            max_x: x_index,
            min_y: y_index,
            max_y: y_index,
            active_points: 1,
            max_abs_intensity: abs_intensity,
            sum_intensity: intensity,
            sum_abs_intensity: abs_intensity,
            weighted_x_sum: spectrum.x.values[x_index] * abs_intensity,
            weighted_y_sum: spectrum.y.values[y_index] * abs_intensity,
        }
    }

    fn add(&mut self, x_index: usize, y_index: usize, intensity: f64, spectrum: &Spectrum2D) {
        let abs_intensity = intensity.abs();
        self.min_x = self.min_x.min(x_index);
        self.max_x = self.max_x.max(x_index);
        self.min_y = self.min_y.min(y_index);
        self.max_y = self.max_y.max(y_index);
        self.active_points += 1;
        self.max_abs_intensity = self.max_abs_intensity.max(abs_intensity);
        self.sum_intensity += intensity;
        self.sum_abs_intensity += abs_intensity;
        self.weighted_x_sum += spectrum.x.values[x_index] * abs_intensity;
        self.weighted_y_sum += spectrum.y.values[y_index] * abs_intensity;
    }

    fn into_zone(self, spectrum: &Spectrum2D) -> DetectedZone {
        let x_from = spectrum.x.values[self.min_x];
        let x_to = spectrum.x.values[self.max_x];
        let y_from = spectrum.y.values[self.min_y];
        let y_to = spectrum.y.values[self.max_y];
        let (centroid_x, centroid_y) = if self.sum_abs_intensity > 0.0 {
            (
                self.weighted_x_sum / self.sum_abs_intensity,
                self.weighted_y_sum / self.sum_abs_intensity,
            )
        } else {
            ((x_from + x_to) * 0.5, (y_from + y_to) * 0.5)
        };

        DetectedZone {
            id: format!(
                "zone:x{}-{}:y{}-{}",
                self.min_x, self.max_x, self.min_y, self.max_y
            ),
            x_start_index: self.min_x,
            x_end_index: self.max_x,
            y_start_index: self.min_y,
            y_end_index: self.max_y,
            x_from,
            x_to,
            y_from,
            y_to,
            centroid_x,
            centroid_y,
            active_points: self.active_points,
            max_abs_intensity: self.max_abs_intensity,
            sum_intensity: self.sum_intensity,
            sum_abs_intensity: self.sum_abs_intensity,
        }
    }
}

/// Detects connected 2D zones whose absolute intensity crosses `threshold_abs`.
///
/// # Errors
///
/// Returns an error when options are invalid.
pub fn detect_zones(
    spectrum: &Spectrum2D,
    options: ZoneDetectionOptions,
) -> Result<Vec<DetectedZone>> {
    options.validate()?;
    let (width, height) = spectrum.shape();
    let mut visited = vec![false; spectrum.z.len()];
    let mut zones = Vec::new();

    for y_index in 0..height {
        for x_index in 0..width {
            let index = matrix_index(width, x_index, y_index);
            if visited[index] {
                continue;
            }
            visited[index] = true;
            let intensity = spectrum.z[index];
            if !is_active(intensity, options.threshold_abs) {
                continue;
            }

            let zone = collect_zone(
                spectrum,
                &mut visited,
                x_index,
                y_index,
                options.threshold_abs,
                options.connectivity,
            );
            if zone.active_points >= options.min_active_points {
                zones.push(zone.into_zone(spectrum));
            }
        }
    }

    Ok(zones)
}

fn collect_zone(
    spectrum: &Spectrum2D,
    visited: &mut [bool],
    start_x: usize,
    start_y: usize,
    threshold_abs: f64,
    connectivity: ZoneConnectivity,
) -> ZoneAccumulator {
    let (width, height) = spectrum.shape();
    let start_index = matrix_index(width, start_x, start_y);
    let mut accumulator = ZoneAccumulator::new(start_x, start_y, spectrum.z[start_index], spectrum);
    let mut queue = VecDeque::from([(start_x, start_y)]);

    while let Some((x_index, y_index)) = queue.pop_front() {
        for (neighbor_x, neighbor_y) in neighbors(width, height, x_index, y_index, connectivity) {
            let neighbor_index = matrix_index(width, neighbor_x, neighbor_y);
            if visited[neighbor_index] {
                continue;
            }
            visited[neighbor_index] = true;
            let intensity = spectrum.z[neighbor_index];
            if is_active(intensity, threshold_abs) {
                accumulator.add(neighbor_x, neighbor_y, intensity, spectrum);
                queue.push_back((neighbor_x, neighbor_y));
            }
        }
    }

    accumulator
}

fn neighbors(
    width: usize,
    height: usize,
    x_index: usize,
    y_index: usize,
    connectivity: ZoneConnectivity,
) -> Vec<(usize, usize)> {
    let mut positions = Vec::with_capacity(match connectivity {
        ZoneConnectivity::Four => 4,
        ZoneConnectivity::Eight => 8,
    });

    if x_index > 0 {
        positions.push((x_index - 1, y_index));
    }
    if x_index + 1 < width {
        positions.push((x_index + 1, y_index));
    }
    if y_index > 0 {
        positions.push((x_index, y_index - 1));
    }
    if y_index + 1 < height {
        positions.push((x_index, y_index + 1));
    }

    if connectivity == ZoneConnectivity::Eight {
        if x_index > 0 && y_index > 0 {
            positions.push((x_index - 1, y_index - 1));
        }
        if x_index + 1 < width && y_index > 0 {
            positions.push((x_index + 1, y_index - 1));
        }
        if x_index > 0 && y_index + 1 < height {
            positions.push((x_index - 1, y_index + 1));
        }
        if x_index + 1 < width && y_index + 1 < height {
            positions.push((x_index + 1, y_index + 1));
        }
    }

    positions
}

fn matrix_index(width: usize, x_index: usize, y_index: usize) -> usize {
    y_index * width + x_index
}

fn is_active(intensity: f64, threshold_abs: f64) -> bool {
    intensity.abs() >= threshold_abs
}

#[cfg(test)]
mod tests;
