//! One-dimensional absolute-value processing.

use rspin_core::{ProcessingRecord, Result, Spectrum1D};
use serde::{Deserialize, Serialize};

use crate::ProcessingStep;

/// Applies component-wise absolute value to one-dimensional data.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Abs1D;

impl Abs1D {
    /// Creates an absolute-value processing step.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl ProcessingStep<Spectrum1D> for Abs1D {
    fn apply(&self, spectrum: &Spectrum1D) -> Result<Spectrum1D> {
        abs_1d(spectrum)
    }
}

/// Applies absolute value to real and imaginary channels independently.
///
/// This preserves the imaginary channel when present. Use
/// [`crate::magnitude_spectrum`] when complex magnitude mode is required.
///
/// # Errors
///
/// Returns an error when the resulting spectrum cannot be constructed.
pub fn abs_1d(spectrum: &Spectrum1D) -> Result<Spectrum1D> {
    let intensities = spectrum
        .intensities
        .iter()
        .map(|value| value.abs())
        .collect::<Vec<_>>();
    let imaginary = spectrum
        .imaginary
        .as_ref()
        .map(|values| values.iter().map(|value| value.abs()).collect::<Vec<_>>());

    let mut processed = Spectrum1D::new_complex(
        spectrum.x.clone(),
        intensities,
        imaginary,
        spectrum.metadata.clone(),
    )?;
    processed.processing.clone_from(&spectrum.processing);
    Ok(processed.with_processing_record(ProcessingRecord::new("abs_1d")))
}

#[cfg(test)]
mod tests {
    use rspin_core::{Axis, Metadata, Unit};

    use super::*;

    #[test]
    fn applies_component_absolute_value() -> anyhow::Result<()> {
        let spectrum = Spectrum1D::new_complex(
            Axis::linear("x", Unit::Ppm, 0.0, 2.0, 3)?,
            vec![-1.0, 2.0, -3.0],
            Some(vec![-0.5, 1.5, -2.5]),
            Metadata::default(),
        )?;
        let processed = Abs1D.apply(&spectrum)?;

        assert_eq!(processed.intensities, vec![1.0, 2.0, 3.0]);
        assert_eq!(processed.imaginary, Some(vec![0.5, 1.5, 2.5]));
        assert_eq!(processed.processing[0].operation, "abs_1d");
        Ok(())
    }
}
