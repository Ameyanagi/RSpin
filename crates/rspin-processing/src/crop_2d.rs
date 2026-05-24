//! Two-dimensional spectral windowing.

use rspin_core::{Axis, ProcessingRecord, RSpinError, Result, Spectrum2D};
use serde::{Deserialize, Serialize};

use crate::ProcessingStep;

/// Keeps 2D points whose x and y coordinates fall inside inclusive windows.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Crop2D {
    /// First requested x coordinate.
    pub x_from: f64,
    /// Second requested x coordinate.
    pub x_to: f64,
    /// First requested y coordinate.
    pub y_from: f64,
    /// Second requested y coordinate.
    pub y_to: f64,
}

impl Crop2D {
    /// Creates a two-dimensional crop step.
    #[must_use]
    pub fn new(x_from: f64, x_to: f64, y_from: f64, y_to: f64) -> Self {
        Self {
            x_from,
            x_to,
            y_from,
            y_to,
        }
    }
}

impl ProcessingStep<Spectrum2D> for Crop2D {
    fn apply(&self, spectrum: &Spectrum2D) -> Result<Spectrum2D> {
        crop_2d(spectrum, self.x_from, self.x_to, self.y_from, self.y_to)
    }
}

/// Keeps points inside inclusive x and y coordinate windows.
///
/// The output preserves original axis order, so descending axes remain descending.
/// Window bounds may be supplied in either order.
///
/// # Errors
///
/// Returns an error when any bound is non-finite or either window selects no points.
pub fn crop_2d(
    spectrum: &Spectrum2D,
    x_from: f64,
    x_to: f64,
    y_from: f64,
    y_to: f64,
) -> Result<Spectrum2D> {
    let x_indices = selected_axis_indices(&spectrum.x.values, x_from, x_to, "x crop window")?;
    let y_indices = selected_axis_indices(&spectrum.y.values, y_from, y_to, "y crop window")?;
    let (width, _) = spectrum.shape();

    let mut z = Vec::with_capacity(x_indices.len() * y_indices.len());
    let mut imaginary = spectrum
        .imaginary
        .as_ref()
        .map(|_| Vec::with_capacity(x_indices.len() * y_indices.len()));
    for y_index in &y_indices {
        for x_index in &x_indices {
            let source_index = y_index * width + x_index;
            z.push(spectrum.z[source_index]);
            if let (Some(source), Some(target)) = (&spectrum.imaginary, &mut imaginary) {
                target.push(source[source_index]);
            }
        }
    }

    let x_values = x_indices
        .iter()
        .map(|index| spectrum.x.values[*index])
        .collect();
    let y_values = y_indices
        .iter()
        .map(|index| spectrum.y.values[*index])
        .collect();
    let mut cropped = Spectrum2D::new_complex(
        Axis::new(spectrum.x.label.clone(), spectrum.x.unit, x_values)?,
        Axis::new(spectrum.y.label.clone(), spectrum.y.unit, y_values)?,
        z,
        imaginary,
        spectrum.metadata.clone(),
    )?;
    cropped.processing.clone_from(&spectrum.processing);
    Ok(
        cropped.with_processing_record(ProcessingRecord::new("crop_2d").with_details(format!(
            "x_from={x_from},x_to={x_to},y_from={y_from},y_to={y_to}"
        ))),
    )
}

fn selected_axis_indices(
    values: &[f64],
    from: f64,
    to: f64,
    operation: &'static str,
) -> Result<Vec<usize>> {
    ensure_finite("crop from", from)?;
    ensure_finite("crop to", to)?;
    let lower = from.min(to);
    let upper = from.max(to);
    let indices = values
        .iter()
        .enumerate()
        .filter_map(|(index, value)| {
            if *value >= lower && *value <= upper {
                Some(index)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    if indices.is_empty() {
        return Err(RSpinError::InvalidSpectrum {
            message: format!("{operation} selects no points"),
        });
    }
    Ok(indices)
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
    fn crops_2d_windows_and_preserves_complex_data() -> anyhow::Result<()> {
        let spectrum = demo_complex_spectrum()?;
        let cropped = crop_2d(&spectrum, 1.0, 2.0, 10.5, 11.0)?;

        assert_eq!(cropped.shape(), (2, 1));
        assert_eq!(cropped.x.values, vec![1.0, 2.0]);
        assert_eq!(cropped.y.values, vec![11.0]);
        assert_eq!(cropped.z, vec![-5.0, 6.0]);
        assert_eq!(require_imaginary(&cropped)?, &[50.0, 60.0]);
        assert_eq!(cropped.processing[0].operation, "crop_2d");
        Ok(())
    }

    #[test]
    fn crops_2d_descending_axes() -> anyhow::Result<()> {
        let spectrum = Spectrum2D::new(
            Axis::linear("x", Unit::Ppm, 2.0, 0.0, 3)?,
            Axis::linear("y", Unit::Ppm, 11.0, 10.0, 2)?,
            vec![3.0, 2.0, 1.0, 6.0, 5.0, 4.0],
            Metadata::named("descending"),
        )?;
        let cropped = Crop2D {
            x_from: 0.0,
            x_to: 1.0,
            y_from: 10.0,
            y_to: 11.0,
        }
        .apply(&spectrum)?;

        assert_eq!(cropped.shape(), (2, 2));
        assert_eq!(cropped.x.values, vec![1.0, 0.0]);
        assert_eq!(cropped.y.values, vec![11.0, 10.0]);
        assert_eq!(cropped.z, vec![2.0, 1.0, 5.0, 4.0]);
        Ok(())
    }

    #[test]
    fn rejects_empty_2d_crop_window() -> anyhow::Result<()> {
        let spectrum = demo_complex_spectrum()?;
        let error =
            crop_2d(&spectrum, 20.0, 21.0, 10.0, 11.0).expect_err("empty x crop should fail");
        assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));
        Ok(())
    }

    fn demo_complex_spectrum() -> anyhow::Result<Spectrum2D> {
        Ok(Spectrum2D::new_complex(
            Axis::linear("x", Unit::Ppm, 0.0, 2.0, 3)?,
            Axis::linear("y", Unit::Ppm, 10.0, 11.0, 2)?,
            vec![1.0, -2.0, 3.0, 4.0, -5.0, 6.0],
            Some(vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0]),
            Metadata::named("2d"),
        )?)
    }

    fn require_imaginary(spectrum: &Spectrum2D) -> anyhow::Result<&[f64]> {
        match &spectrum.imaginary {
            Some(imaginary) => Ok(imaginary),
            None => anyhow::bail!("missing imaginary channel"),
        }
    }
}
