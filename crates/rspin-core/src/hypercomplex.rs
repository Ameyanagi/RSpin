//! Hypercomplex two-dimensional container for phase-sensitive processing.

// The four hypercomplex quadrants (rr/ri/ir/ii) are intrinsically similar
// names; renaming them hurts clarity.
#![allow(clippy::similar_names)]

use serde::{Deserialize, Serialize};

use crate::{Axis, Metadata, ProcessingRecord, RSpinError, Result, Spectrum2D};

/// Row-major hypercomplex 2D data with four real quadrant planes.
///
/// Each plane has length `x.len() * y.len()` in row-major order
/// (`index = y_index * x.len() + x_index`). The first letter of each quadrant
/// names the indirect (y) channel and the second the direct (x) channel, where
/// `R` is the real/cosine component and `I` the imaginary/sine component.
///
/// This is a transient container used while a phase-sensitive 2D experiment is
/// processed: it can hold either time-domain or frequency-domain data along
/// each dimension, with the four quadrants carried together so the indirect
/// dimension can be complex-transformed and phased.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HyperComplex2D {
    /// Direct (x) axis.
    pub x: Axis,
    /// Indirect (y) axis.
    pub y: Axis,
    /// Real-indirect, real-direct quadrant.
    pub rr: Vec<f64>,
    /// Real-indirect, imaginary-direct quadrant.
    pub ri: Vec<f64>,
    /// Imaginary-indirect, real-direct quadrant.
    pub ir: Vec<f64>,
    /// Imaginary-indirect, imaginary-direct quadrant.
    pub ii: Vec<f64>,
    /// Spectrum metadata.
    pub metadata: Metadata,
    /// Applied processing records.
    pub processing: Vec<ProcessingRecord>,
}

impl HyperComplex2D {
    /// Creates a hypercomplex 2D container from four equal-length quadrants.
    ///
    /// # Errors
    ///
    /// Returns an error when any plane length differs from `x.len() * y.len()`
    /// or contains a non-finite value.
    pub fn new(
        x: Axis,
        y: Axis,
        rr: Vec<f64>,
        ri: Vec<f64>,
        ir: Vec<f64>,
        ii: Vec<f64>,
        metadata: Metadata,
    ) -> Result<Self> {
        metadata.validate()?;
        let expected = x
            .len()
            .checked_mul(y.len())
            .ok_or_else(|| RSpinError::InvalidSpectrum {
                message: "hypercomplex 2D axis size overflow".to_owned(),
            })?;
        for (name, plane) in [("rr", &rr), ("ri", &ri), ("ir", &ir), ("ii", &ii)] {
            if plane.len() != expected {
                return Err(RSpinError::InvalidSpectrum {
                    message: format!(
                        "hypercomplex plane {name} has {} values but axes require {expected}",
                        plane.len()
                    ),
                });
            }
            if !plane.iter().all(|value| value.is_finite()) {
                return Err(RSpinError::NonFinite {
                    field: "hypercomplex",
                });
            }
        }
        Ok(Self {
            x,
            y,
            rr,
            ri,
            ir,
            ii,
            metadata,
            processing: Vec::new(),
        })
    }

    /// Returns the `(x, y)` shape.
    #[must_use]
    pub fn shape(&self) -> (usize, usize) {
        (self.x.len(), self.y.len())
    }

    fn plane_at(plane: &[f64], width: usize, height: usize, x: usize, y: usize) -> Option<f64> {
        if x >= width || y >= height {
            return None;
        }
        plane.get(y * width + x).copied()
    }

    /// Gets the RR quadrant value by x/y index.
    #[must_use]
    pub fn rr_at(&self, x: usize, y: usize) -> Option<f64> {
        let (width, height) = self.shape();
        Self::plane_at(&self.rr, width, height, x, y)
    }

    /// Gets the RI quadrant value by x/y index.
    #[must_use]
    pub fn ri_at(&self, x: usize, y: usize) -> Option<f64> {
        let (width, height) = self.shape();
        Self::plane_at(&self.ri, width, height, x, y)
    }

    /// Gets the IR quadrant value by x/y index.
    #[must_use]
    pub fn ir_at(&self, x: usize, y: usize) -> Option<f64> {
        let (width, height) = self.shape();
        Self::plane_at(&self.ir, width, height, x, y)
    }

    /// Gets the II quadrant value by x/y index.
    #[must_use]
    pub fn ii_at(&self, x: usize, y: usize) -> Option<f64> {
        let (width, height) = self.shape();
        Self::plane_at(&self.ii, width, height, x, y)
    }

    /// Returns a copy with one appended processing record.
    #[must_use]
    pub fn with_processing_record(mut self, record: ProcessingRecord) -> Self {
        self.processing.push(record);
        self
    }

    /// Downgrades to a displayable [`Spectrum2D`].
    ///
    /// The doubly-real `rr` quadrant becomes the real channel (`2rr` analog)
    /// and `ri` becomes the companion imaginary channel for direct-dimension
    /// re-phasing.
    ///
    /// # Errors
    ///
    /// Returns an error when the resulting spectrum fails validation.
    pub fn into_spectrum_2d(self) -> Result<Spectrum2D> {
        let mut spectrum =
            Spectrum2D::new_complex(self.x, self.y, self.rr, Some(self.ri), self.metadata)?;
        spectrum.processing.clone_from(&self.processing);
        Ok(spectrum)
    }

    /// Downgrades to a real [`Spectrum2D`] holding the hypercomplex modulus
    /// `sqrt(rr^2 + ri^2 + ir^2 + ii^2)`.
    ///
    /// This combines all four quadrants, so it is phase-insensitive: a
    /// frequency-domain spectrum is displayed cleanly without needing a perfect
    /// direct/indirect phase correction (the dispersive energy in the `ir`/`ii`
    /// quadrants is folded into the magnitude instead of appearing as ridges).
    ///
    /// # Errors
    ///
    /// Returns an error when the resulting spectrum fails validation.
    pub fn into_magnitude_spectrum_2d(self) -> Result<Spectrum2D> {
        let magnitude: Vec<f64> = self
            .rr
            .iter()
            .zip(&self.ri)
            .zip(self.ir.iter().zip(&self.ii))
            .map(|((rr, ri), (ir, ii))| (rr * rr + ri * ri + ir * ir + ii * ii).sqrt())
            .collect();
        let mut spectrum = Spectrum2D::new(self.x, self.y, magnitude, self.metadata)?;
        spectrum.processing.clone_from(&self.processing);
        Ok(spectrum)
    }
}
