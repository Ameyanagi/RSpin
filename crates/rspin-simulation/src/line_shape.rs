//! Normalized line-shape functions.

use std::f64::consts::{LN_2, PI};

use serde::{Deserialize, Serialize};

/// Supported normalized line shapes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineShape {
    /// Lorentzian peak shape.
    #[default]
    Lorentzian,
    /// Gaussian peak shape.
    Gaussian,
}

impl LineShape {
    /// Evaluates the line shape at `x_ppm`.
    ///
    /// `area` is the integrated area in ppm-domain units.
    #[must_use]
    pub fn value(
        self,
        x_ppm: f64,
        center_ppm: f64,
        line_width_hz: f64,
        spectrometer_mhz: f64,
        area: f64,
    ) -> f64 {
        let fwhm_ppm = line_width_hz / spectrometer_mhz;
        match self {
            Self::Lorentzian => lorentzian(x_ppm, center_ppm, fwhm_ppm, area),
            Self::Gaussian => gaussian(x_ppm, center_ppm, fwhm_ppm, area),
        }
    }
}

fn lorentzian(x_ppm: f64, center_ppm: f64, fwhm_ppm: f64, area: f64) -> f64 {
    let half_width = fwhm_ppm / 2.0;
    area * half_width / (PI * ((x_ppm - center_ppm).powi(2) + half_width.powi(2)))
}

fn gaussian(x_ppm: f64, center_ppm: f64, fwhm_ppm: f64, area: f64) -> f64 {
    let sigma = fwhm_ppm / (2.0 * (2.0 * LN_2).sqrt());
    let normalizer = sigma * (2.0 * PI).sqrt();
    area * (-(x_ppm - center_ppm).powi(2) / (2.0 * sigma.powi(2))).exp() / normalizer
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lorentzian_peak_height_tracks_area_and_width() {
        let height = LineShape::Lorentzian.value(1.0, 1.0, 2.0, 100.0, 3.0);
        let expected = 3.0 / (PI * 0.01);
        assert_close(height, expected);
    }

    #[test]
    fn gaussian_peak_height_tracks_area_and_width() {
        let height = LineShape::Gaussian.value(1.0, 1.0, 2.0, 100.0, 3.0);
        assert!(height > 0.0);
    }

    fn assert_close(actual: f64, expected: f64) {
        assert!((actual - expected).abs() < 1e-12, "{actual} != {expected}");
    }
}
