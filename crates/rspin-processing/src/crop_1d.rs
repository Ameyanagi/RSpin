//! One-dimensional spectral windowing.

use rspin_core::{Axis, ProcessingRecord, RSpinError, Result, Spectrum1D};
use serde::{Deserialize, Serialize};

use crate::ProcessingStep;

/// Keeps points whose x coordinates fall inside an inclusive window.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Crop1D {
    /// First requested x coordinate.
    pub from: f64,
    /// Second requested x coordinate.
    pub to: f64,
}

impl Crop1D {
    /// Creates a one-dimensional crop step.
    #[must_use]
    pub fn new(from: f64, to: f64) -> Self {
        Self { from, to }
    }
}

impl ProcessingStep<Spectrum1D> for Crop1D {
    fn apply(&self, spectrum: &Spectrum1D) -> Result<Spectrum1D> {
        crop_1d(spectrum, self.from, self.to)
    }
}

/// Keeps points whose x coordinates fall inside the inclusive `[from, to]` window.
///
/// The output preserves the original axis order, so descending ppm axes remain
/// descending. `from` and `to` may be supplied in either order.
///
/// # Errors
///
/// Returns an error when either bound is non-finite or the window selects no points.
pub fn crop_1d(spectrum: &Spectrum1D, from: f64, to: f64) -> Result<Spectrum1D> {
    let indices = selected_axis_indices(&spectrum.x.values, from, to, "crop window")?;
    let x_values = indices
        .iter()
        .map(|index| spectrum.x.values[*index])
        .collect();
    let intensities = indices
        .iter()
        .map(|index| spectrum.intensities[*index])
        .collect();
    let imaginary = spectrum.imaginary.as_ref().map(|values| {
        indices
            .iter()
            .map(|index| values[*index])
            .collect::<Vec<_>>()
    });

    let mut cropped = Spectrum1D::new_complex(
        Axis::new(spectrum.x.label.clone(), spectrum.x.unit, x_values)?,
        intensities,
        imaginary,
        spectrum.metadata.clone(),
    )?;
    cropped.processing.clone_from(&spectrum.processing);
    Ok(cropped.with_processing_record(
        ProcessingRecord::new("crop_1d").with_details(format!("from={from},to={to}")),
    ))
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
    fn crops_ascending_and_descending_spectra() -> anyhow::Result<()> {
        let spectrum = Spectrum1D::new_complex(
            Axis::linear("shift", Unit::Ppm, 0.0, 4.0, 5)?,
            vec![0.0, 1.0, 2.0, 3.0, 4.0],
            Some(vec![10.0, 11.0, 12.0, 13.0, 14.0]),
            Metadata::default(),
        )?;
        let cropped = Crop1D { from: 1.0, to: 3.0 }.apply(&spectrum)?;

        assert_eq!(cropped.x.values, vec![1.0, 2.0, 3.0]);
        assert_eq!(cropped.intensities, vec![1.0, 2.0, 3.0]);
        assert_eq!(cropped.imaginary, Some(vec![11.0, 12.0, 13.0]));
        assert_eq!(cropped.processing[0].operation, "crop_1d");

        let descending = Spectrum1D::new(
            Axis::linear("shift", Unit::Ppm, 4.0, 0.0, 5)?,
            vec![4.0, 3.0, 2.0, 1.0, 0.0],
            Metadata::default(),
        )?;
        let cropped = crop_1d(&descending, 1.0, 3.0)?;
        assert_eq!(cropped.x.values, vec![3.0, 2.0, 1.0]);
        assert_eq!(cropped.intensities, vec![3.0, 2.0, 1.0]);
        Ok(())
    }

    #[test]
    fn rejects_empty_crop_window() -> anyhow::Result<()> {
        let spectrum = Spectrum1D::new(
            Axis::linear("shift", Unit::Ppm, 1.0, 3.0, 3)?,
            vec![1.0, -2.0, 4.0],
            Metadata::default(),
        )?;
        let error = crop_1d(&spectrum, 10.0, 11.0).expect_err("empty crop should fail");
        assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));
        Ok(())
    }
}
