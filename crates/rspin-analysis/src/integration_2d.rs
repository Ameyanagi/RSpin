//! Two-dimensional integration.

use serde::{Deserialize, Serialize};

use rspin_core::{RSpinError, Result, Spectrum2D};

use crate::Integrator2D;

/// Inclusive rectangular integration region.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct IntegralRegion2D {
    /// Start x coordinate.
    pub x_from: f64,
    /// End x coordinate.
    pub x_to: f64,
    /// Start y coordinate.
    pub y_from: f64,
    /// End y coordinate.
    pub y_to: f64,
}

impl IntegralRegion2D {
    fn bounds(self) -> Result<((f64, f64), (f64, f64))> {
        require_finite("x_from", self.x_from)?;
        require_finite("x_to", self.x_to)?;
        require_finite("y_from", self.y_from)?;
        require_finite("y_to", self.y_to)?;
        Ok((
            ordered_bounds(self.x_from, self.x_to),
            ordered_bounds(self.y_from, self.y_to),
        ))
    }
}

/// Integrated volume over a two-dimensional region.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Integral2D {
    /// Requested region.
    pub region: IntegralRegion2D,
    /// Bilinear surface integral.
    pub volume: f64,
    /// Number of grid cells contributing to the integral.
    pub cells: usize,
}

/// Integrator that treats each matrix cell as a bilinear surface.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BilinearIntegrator2D;

impl Integrator2D for BilinearIntegrator2D {
    fn integrate(&self, spectrum: &Spectrum2D, region: IntegralRegion2D) -> Result<Integral2D> {
        integrate_region_2d(spectrum, region)
    }
}

/// Integrates a two-dimensional spectrum over `region`.
///
/// Each grid cell is treated as a bilinear surface. Partial boundary cells are
/// integrated by evaluating the bilinear surface at the four overlap corners,
/// which is exact for bilinear interpolation.
///
/// # Errors
///
/// Returns an error when the region is invalid or the spectrum is smaller than
/// one cell in either dimension.
pub fn integrate_region_2d(spectrum: &Spectrum2D, region: IntegralRegion2D) -> Result<Integral2D> {
    let ((x_min, x_max), (y_min, y_max)) = region.bounds()?;
    let (width, height) = spectrum.shape();
    if width < 2 || height < 2 {
        return Err(RSpinError::InvalidSpectrum {
            message: "2D integration requires at least one grid cell".to_owned(),
        });
    }

    let mut volume = 0.0;
    let mut cells = 0;
    for y_index in 0..(height - 1) {
        let y0 = spectrum.y.values[y_index];
        let y1 = spectrum.y.values[y_index + 1];
        if (y0 - y1).abs() <= f64::EPSILON {
            continue;
        }
        let (row_lower, row_upper) = ordered_bounds(y0, y1);
        let row_start = row_lower.max(y_min);
        let row_end = row_upper.min(y_max);
        if row_start >= row_end {
            continue;
        }

        for x_index in 0..(width - 1) {
            let x0 = spectrum.x.values[x_index];
            let x1 = spectrum.x.values[x_index + 1];
            if (x0 - x1).abs() <= f64::EPSILON {
                continue;
            }
            let (column_lower, column_upper) = ordered_bounds(x0, x1);
            let column_start = column_lower.max(x_min);
            let column_end = column_upper.min(x_max);
            if column_start >= column_end {
                continue;
            }

            let cell = Cell {
                x_index,
                y_index,
                x0,
                x1,
                y0,
                y1,
            };
            let z00 = bilinear_value(&spectrum.z, width, cell, column_start, row_start);
            let z10 = bilinear_value(&spectrum.z, width, cell, column_end, row_start);
            let z01 = bilinear_value(&spectrum.z, width, cell, column_start, row_end);
            let z11 = bilinear_value(&spectrum.z, width, cell, column_end, row_end);
            let overlap_area = (column_end - column_start) * (row_end - row_start);
            volume += overlap_area * 0.25 * (z00 + z10 + z01 + z11);
            cells += 1;
        }
    }

    Ok(Integral2D {
        region,
        volume,
        cells,
    })
}

#[derive(Clone, Copy)]
struct Cell {
    x_index: usize,
    y_index: usize,
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
}

fn bilinear_value(values: &[f64], width: usize, cell: Cell, x: f64, y: f64) -> f64 {
    let x_fraction = (x - cell.x0) / (cell.x1 - cell.x0);
    let y_fraction = (y - cell.y0) / (cell.y1 - cell.y0);
    let lower_left = values[cell.y_index * width + cell.x_index];
    let lower_right = values[cell.y_index * width + cell.x_index + 1];
    let upper_left = values[(cell.y_index + 1) * width + cell.x_index];
    let upper_right = values[(cell.y_index + 1) * width + cell.x_index + 1];
    let lower = interpolate(lower_left, lower_right, x_fraction);
    let upper = interpolate(upper_left, upper_right, x_fraction);
    interpolate(lower, upper, y_fraction)
}

fn interpolate(start: f64, end: f64, fraction: f64) -> f64 {
    start + fraction * (end - start)
}

fn ordered_bounds(a: f64, b: f64) -> (f64, f64) {
    if a <= b { (a, b) } else { (b, a) }
}

fn require_finite(field: &'static str, value: f64) -> Result<()> {
    if !value.is_finite() {
        return Err(RSpinError::NonFinite { field });
    }
    Ok(())
}

#[cfg(test)]
mod tests;
