//! Two-dimensional contour extraction.

use rspin_core::{RSpinError, Result, Spectrum2D};
use serde::{Deserialize, Serialize};

/// Point on an extracted contour.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContourPoint {
    /// X coordinate.
    pub x: f64,
    /// Y coordinate.
    pub y: f64,
}

/// Line segment belonging to one contour level.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContourSegment {
    /// Contour level.
    pub level: f64,
    /// Start point.
    pub start: ContourPoint,
    /// End point.
    pub end: ContourPoint,
    /// X cell index that produced this segment.
    pub cell_x_index: usize,
    /// Y cell index that produced this segment.
    pub cell_y_index: usize,
}

/// Extracted contour segments for one level.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContourSet {
    /// Contour level.
    pub level: f64,
    /// Extracted line segments.
    pub segments: Vec<ContourSegment>,
}

/// Extracts contour segments for multiple levels using marching squares.
///
/// # Errors
///
/// Returns an error when the spectrum is smaller than one cell or any level is
/// non-finite.
pub fn extract_contours(spectrum: &Spectrum2D, levels: &[f64]) -> Result<Vec<ContourSet>> {
    validate_shape(spectrum)?;
    levels
        .iter()
        .copied()
        .map(|level| {
            Ok(ContourSet {
                level,
                segments: contour_segments(spectrum, level)?,
            })
        })
        .collect()
}

/// Extracts contour segments for a single level using marching squares.
///
/// Ambiguous four-intersection cells are split deterministically by edge order.
///
/// # Errors
///
/// Returns an error when the spectrum is smaller than one cell or `level` is
/// non-finite.
pub fn contour_segments(spectrum: &Spectrum2D, level: f64) -> Result<Vec<ContourSegment>> {
    validate_shape(spectrum)?;
    if !level.is_finite() {
        return Err(RSpinError::NonFinite {
            field: "contour level",
        });
    }

    let (width, height) = spectrum.shape();
    let mut segments = Vec::new();
    for y_index in 0..(height - 1) {
        for x_index in 0..(width - 1) {
            let intersections = cell_intersections(spectrum, level, x_index, y_index);
            push_cell_segments(&mut segments, level, x_index, y_index, &intersections);
        }
    }
    Ok(segments)
}

fn cell_intersections(
    spectrum: &Spectrum2D,
    level: f64,
    x_index: usize,
    y_index: usize,
) -> Vec<ContourPoint> {
    let (width, _) = spectrum.shape();
    let x0 = spectrum.x.values[x_index];
    let x1 = spectrum.x.values[x_index + 1];
    let y0 = spectrum.y.values[y_index];
    let y1 = spectrum.y.values[y_index + 1];
    let z00 = spectrum.z[matrix_index(width, x_index, y_index)];
    let z10 = spectrum.z[matrix_index(width, x_index + 1, y_index)];
    let z11 = spectrum.z[matrix_index(width, x_index + 1, y_index + 1)];
    let z01 = spectrum.z[matrix_index(width, x_index, y_index + 1)];

    let edges = [
        ((x0, y0, z00), (x1, y0, z10)),
        ((x1, y0, z10), (x1, y1, z11)),
        ((x1, y1, z11), (x0, y1, z01)),
        ((x0, y1, z01), (x0, y0, z00)),
    ];
    let mut points = Vec::with_capacity(4);
    for (start, end) in edges {
        if let Some(point) = edge_intersection(start, end, level) {
            push_unique(&mut points, point);
        }
    }
    points
}

fn edge_intersection(
    start: (f64, f64, f64),
    end: (f64, f64, f64),
    level: f64,
) -> Option<ContourPoint> {
    let (x0, y0, z0) = start;
    let (x1, y1, z1) = end;
    let d0 = z0 - level;
    let d1 = z1 - level;
    if d0 == 0.0 && d1 == 0.0 {
        return None;
    }
    if d0 == 0.0 {
        return Some(ContourPoint { x: x0, y: y0 });
    }
    if d1 == 0.0 {
        return Some(ContourPoint { x: x1, y: y1 });
    }
    if d0.is_sign_positive() == d1.is_sign_positive() {
        return None;
    }

    let fraction = (level - z0) / (z1 - z0);
    Some(ContourPoint {
        x: x0 + fraction * (x1 - x0),
        y: y0 + fraction * (y1 - y0),
    })
}

fn push_cell_segments(
    segments: &mut Vec<ContourSegment>,
    level: f64,
    x_index: usize,
    y_index: usize,
    points: &[ContourPoint],
) {
    match points {
        [start, end] => segments.push(segment(level, x_index, y_index, *start, *end)),
        [first, second, third, fourth] => {
            segments.push(segment(level, x_index, y_index, *first, *second));
            segments.push(segment(level, x_index, y_index, *third, *fourth));
        }
        _ => {}
    }
}

fn segment(
    level: f64,
    x_index: usize,
    y_index: usize,
    start: ContourPoint,
    end: ContourPoint,
) -> ContourSegment {
    ContourSegment {
        level,
        start,
        end,
        cell_x_index: x_index,
        cell_y_index: y_index,
    }
}

fn push_unique(points: &mut Vec<ContourPoint>, point: ContourPoint) {
    if !points.contains(&point) {
        points.push(point);
    }
}

fn validate_shape(spectrum: &Spectrum2D) -> Result<()> {
    let (width, height) = spectrum.shape();
    if width < 2 || height < 2 {
        return Err(RSpinError::InvalidSpectrum {
            message: "contour extraction requires at least one 2D cell".to_owned(),
        });
    }
    Ok(())
}

fn matrix_index(width: usize, x_index: usize, y_index: usize) -> usize {
    y_index * width + x_index
}

#[cfg(test)]
mod tests;
