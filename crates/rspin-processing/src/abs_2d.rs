//! Two-dimensional absolute-value processing.

use rspin_core::{ProcessingRecord, Result, Spectrum2D};
use serde::{Deserialize, Serialize};

use crate::ProcessingStep;

/// Applies component-wise absolute value to two-dimensional data.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Abs2D;

impl ProcessingStep<Spectrum2D> for Abs2D {
    fn apply(&self, spectrum: &Spectrum2D) -> Result<Spectrum2D> {
        abs_2d(spectrum)
    }
}

/// Applies absolute value to real and imaginary matrices independently.
///
/// # Errors
///
/// Returns an error when the resulting spectrum cannot be constructed.
pub fn abs_2d(spectrum: &Spectrum2D) -> Result<Spectrum2D> {
    let z = spectrum
        .z
        .iter()
        .map(|value| value.abs())
        .collect::<Vec<_>>();
    let imaginary = spectrum
        .imaginary
        .as_ref()
        .map(|values| values.iter().map(|value| value.abs()).collect::<Vec<_>>());

    let mut processed = Spectrum2D::new_complex(
        spectrum.x.clone(),
        spectrum.y.clone(),
        z,
        imaginary,
        spectrum.metadata.clone(),
    )?;
    processed.processing.clone_from(&spectrum.processing);
    Ok(processed.with_processing_record(ProcessingRecord::new("abs_2d")))
}

#[cfg(test)]
mod tests {
    use rspin_core::{Axis, Metadata, Unit};

    use super::*;

    #[test]
    fn applies_component_absolute_value() -> anyhow::Result<()> {
        let spectrum = Spectrum2D::new_complex(
            Axis::linear("x", Unit::Ppm, 0.0, 1.0, 2)?,
            Axis::linear("y", Unit::Ppm, 0.0, 1.0, 2)?,
            vec![-1.0, 2.0, -3.0, 4.0],
            Some(vec![0.5, -1.5, 2.5, -3.5]),
            Metadata::default(),
        )?;
        let processed = Abs2D.apply(&spectrum)?;

        assert_eq!(processed.z, vec![1.0, 2.0, 3.0, 4.0]);
        assert_eq!(processed.imaginary, Some(vec![0.5, 1.5, 2.5, 3.5]));
        assert_eq!(processed.processing[0].operation, "abs_2d");
        Ok(())
    }
}
