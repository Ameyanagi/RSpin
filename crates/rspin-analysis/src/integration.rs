//! One-dimensional integration.

use serde::{Deserialize, Serialize};

use rspin_core::{RSpinError, Result, Spectrum1D};

use crate::{Integrator, ranges::DetectedRange};

/// Inclusive integration region.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct IntegralRegion {
    /// Start coordinate.
    pub from: f64,
    /// End coordinate.
    pub to: f64,
}

impl IntegralRegion {
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
}

/// Integrated area over a one-dimensional region.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Integral {
    /// Requested region.
    pub region: IntegralRegion,
    /// Trapezoidal area.
    pub area: f64,
    /// Number of spectrum segments contributing to the area.
    pub segments: usize,
}

/// Trapezoidal integrator.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrapezoidalIntegrator;

impl Integrator for TrapezoidalIntegrator {
    fn integrate(&self, spectrum: &Spectrum1D, region: IntegralRegion) -> Result<Integral> {
        integrate_region(spectrum, region)
    }
}

/// Integrates a spectrum over `region` using linear interpolation at boundaries.
///
/// # Errors
///
/// Returns an error when the region is invalid or the spectrum has too few
/// points.
pub fn integrate_region(spectrum: &Spectrum1D, region: IntegralRegion) -> Result<Integral> {
    let (region_min, region_max) = region.bounds()?;
    if spectrum.len() < 2 {
        return Err(RSpinError::InvalidSpectrum {
            message: "integration requires at least two points".to_owned(),
        });
    }

    let mut area = 0.0;
    let mut segments = 0;
    for ((&x0, &x1), (&y0, &y1)) in spectrum
        .x
        .values
        .iter()
        .zip(spectrum.x.values.iter().skip(1))
        .zip(
            spectrum
                .intensities
                .iter()
                .zip(spectrum.intensities.iter().skip(1)),
        )
    {
        if (x0 - x1).abs() <= f64::EPSILON {
            continue;
        }
        let segment_min = x0.min(x1);
        let segment_max = x0.max(x1);
        let overlap_min = segment_min.max(region_min);
        let overlap_max = segment_max.min(region_max);
        if overlap_min >= overlap_max {
            continue;
        }

        let y_at_min = interpolate(x0, y0, x1, y1, overlap_min);
        let y_at_max = interpolate(x0, y0, x1, y1, overlap_max);
        area += 0.5 * (y_at_min + y_at_max) * (overlap_max - overlap_min);
        segments += 1;
    }

    Ok(Integral {
        region,
        area,
        segments,
    })
}

/// Integrates a spectrum over multiple regions in input order.
///
/// # Errors
///
/// Returns the first integration error produced by any region.
pub fn integrate_regions(
    spectrum: &Spectrum1D,
    regions: &[IntegralRegion],
) -> Result<Vec<Integral>> {
    regions
        .iter()
        .copied()
        .map(|region| integrate_region(spectrum, region))
        .collect()
}

/// Integrates a spectrum over detected range bounds in input order.
///
/// # Errors
///
/// Returns the first integration error produced by any detected range.
pub fn integrate_ranges(spectrum: &Spectrum1D, ranges: &[DetectedRange]) -> Result<Vec<Integral>> {
    ranges
        .iter()
        .map(|range| {
            integrate_region(
                spectrum,
                IntegralRegion {
                    from: range.from,
                    to: range.to,
                },
            )
        })
        .collect()
}

fn interpolate(x0: f64, y0: f64, x1: f64, y1: f64, x: f64) -> f64 {
    let fraction = (x - x0) / (x1 - x0);
    y0 + fraction * (y1 - y0)
}

#[cfg(test)]
mod tests {
    use rspin_core::{Axis, Metadata, Unit};

    use super::*;

    #[test]
    fn integrates_full_region() -> anyhow::Result<()> {
        let spectrum = spectrum(&[0.0, 1.0, 2.0], 0.0, 2.0)?;
        let integral = integrate_region(&spectrum, IntegralRegion { from: 0.0, to: 2.0 })?;
        assert_close(integral.area, 2.0);
        assert_eq!(integral.segments, 2);
        Ok(())
    }

    #[test]
    fn integrates_partial_region_with_interpolation() -> anyhow::Result<()> {
        let spectrum = spectrum(&[0.0, 2.0, 2.0], 0.0, 2.0)?;
        let integral =
            TrapezoidalIntegrator.integrate(&spectrum, IntegralRegion { from: 0.5, to: 1.5 })?;
        assert_close(integral.area, 1.75);
        assert_eq!(integral.segments, 2);
        Ok(())
    }

    #[test]
    fn handles_descending_axis_and_reversed_region() -> anyhow::Result<()> {
        let spectrum = spectrum(&[0.0, 1.0, 2.0], 2.0, 0.0)?;
        let integral = integrate_region(&spectrum, IntegralRegion { from: 1.5, to: 0.5 })?;
        assert_close(integral.area, 1.0);
        assert_eq!(integral.segments, 2);
        Ok(())
    }

    #[test]
    fn returns_zero_outside_spectrum_domain() -> anyhow::Result<()> {
        let spectrum = spectrum(&[1.0, 1.0, 1.0], 0.0, 2.0)?;
        let integral = integrate_region(&spectrum, IntegralRegion { from: 3.0, to: 4.0 })?;
        assert_close(integral.area, 0.0);
        assert_eq!(integral.segments, 0);
        Ok(())
    }

    #[test]
    fn integrates_multiple_regions_in_order() -> anyhow::Result<()> {
        let spectrum = spectrum(&[0.0, 1.0, 2.0], 0.0, 2.0)?;
        let integrals = integrate_regions(
            &spectrum,
            &[
                IntegralRegion { from: 0.0, to: 1.0 },
                IntegralRegion { from: 1.0, to: 2.0 },
            ],
        )?;

        assert_eq!(integrals.len(), 2);
        assert_close(integrals[0].area, 0.5);
        assert_close(integrals[1].area, 1.5);
        assert_eq!(integrals[0].region, IntegralRegion { from: 0.0, to: 1.0 });
        assert_eq!(integrals[1].region, IntegralRegion { from: 1.0, to: 2.0 });
        Ok(())
    }

    #[test]
    fn integrates_detected_ranges_in_order() -> anyhow::Result<()> {
        let spectrum = spectrum(&[0.0, 2.0, 0.0, 4.0, 0.0], 0.0, 4.0)?;
        let ranges = [
            detected_range(0, 2, 0.0, 2.0),
            detected_range(2, 4, 2.0, 4.0),
        ];
        let integrals = integrate_ranges(&spectrum, &ranges)?;

        assert_eq!(integrals.len(), 2);
        assert_close(integrals[0].area, 2.0);
        assert_close(integrals[1].area, 4.0);
        assert_eq!(integrals[0].region, IntegralRegion { from: 0.0, to: 2.0 });
        assert_eq!(integrals[1].region, IntegralRegion { from: 2.0, to: 4.0 });
        Ok(())
    }

    fn detected_range(start_index: usize, end_index: usize, from: f64, to: f64) -> DetectedRange {
        DetectedRange {
            start_index,
            end_index,
            from,
            to,
            active_points: end_index - start_index + 1,
            max_abs_intensity: 0.0,
            area: 0.0,
        }
    }

    fn spectrum(intensities: &[f64], start: f64, end: f64) -> anyhow::Result<Spectrum1D> {
        let axis = Axis::linear("x", Unit::Ppm, start, end, intensities.len())?;
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
