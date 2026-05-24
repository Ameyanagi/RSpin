//! Spectrum and axis data structures.

use serde::{Deserialize, Serialize};

use crate::{Metadata, RSpinError, Result, Unit};

/// A numeric axis.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Axis {
    /// Human-readable axis label.
    pub label: String,
    /// Axis unit.
    pub unit: Unit,
    /// Axis values.
    pub values: Vec<f64>,
}

impl Axis {
    /// Creates an axis from explicit values.
    ///
    /// # Errors
    ///
    /// Returns an error when the axis is empty or contains non-finite values.
    pub fn new(label: impl Into<String>, unit: Unit, values: Vec<f64>) -> Result<Self> {
        if values.is_empty() {
            return Err(RSpinError::InvalidAxis {
                message: "axis must contain at least one point".to_owned(),
            });
        }
        if !values.iter().all(|value| value.is_finite()) {
            return Err(RSpinError::NonFinite { field: "axis" });
        }
        Ok(Self {
            label: label.into(),
            unit,
            values,
        })
    }

    /// Creates a linearly spaced axis.
    ///
    /// # Errors
    ///
    /// Returns an error when `points` is zero or the bounds are not finite.
    pub fn linear(
        label: impl Into<String>,
        unit: Unit,
        start: f64,
        end: f64,
        points: usize,
    ) -> Result<Self> {
        if points == 0 {
            return Err(RSpinError::InvalidAxis {
                message: "linear axis must contain at least one point".to_owned(),
            });
        }
        if !start.is_finite() || !end.is_finite() {
            return Err(RSpinError::NonFinite {
                field: "axis bounds",
            });
        }

        let values = if points == 1 {
            vec![start]
        } else {
            let segments = u32::try_from(points - 1).map_err(|_| RSpinError::InvalidAxis {
                message: "linear axis point count is too large".to_owned(),
            })?;
            let step = (end - start) / f64::from(segments);
            let mut values = Vec::with_capacity(points);
            let mut value = start;
            for _ in 0..points {
                values.push(value);
                value += step;
            }
            if let Some(last) = values.last_mut() {
                *last = end;
            }
            values
        };

        Self::new(label, unit, values)
    }

    /// Returns the number of axis points.
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns true when the axis has no values.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

/// A processing step recorded on a spectrum.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProcessingRecord {
    /// Stable operation name.
    pub operation: String,
    /// Human-readable details.
    pub details: Option<String>,
}

impl ProcessingRecord {
    /// Creates a processing record with no details.
    #[must_use]
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            details: None,
        }
    }

    /// Adds details to the record.
    #[must_use]
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}

/// A one-dimensional spectrum.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Spectrum1D {
    /// X axis.
    pub x: Axis,
    /// Real-valued intensities.
    pub intensities: Vec<f64>,
    /// Optional imaginary component.
    pub imaginary: Option<Vec<f64>>,
    /// Spectrum metadata.
    pub metadata: Metadata,
    /// Applied processing records.
    pub processing: Vec<ProcessingRecord>,
}

impl Spectrum1D {
    /// Creates a one-dimensional spectrum.
    ///
    /// # Errors
    ///
    /// Returns an error when axis and data lengths differ or data is non-finite.
    pub fn new(x: Axis, intensities: Vec<f64>, metadata: Metadata) -> Result<Self> {
        Self::new_complex(x, intensities, None, metadata)
    }

    /// Creates a one-dimensional spectrum with an optional imaginary channel.
    ///
    /// # Errors
    ///
    /// Returns an error when axis and data lengths differ or data is non-finite.
    pub fn new_complex(
        x: Axis,
        intensities: Vec<f64>,
        imaginary: Option<Vec<f64>>,
        metadata: Metadata,
    ) -> Result<Self> {
        validate_vector("intensities", &intensities)?;
        if x.len() != intensities.len() {
            return Err(RSpinError::InvalidSpectrum {
                message: format!(
                    "x axis has {} points but intensities have {} points",
                    x.len(),
                    intensities.len()
                ),
            });
        }
        if let Some(imaginary_values) = imaginary.as_deref() {
            validate_vector("imaginary", imaginary_values)?;
            if imaginary_values.len() != x.len() {
                return Err(RSpinError::InvalidSpectrum {
                    message: format!(
                        "x axis has {} points but imaginary data has {} points",
                        x.len(),
                        imaginary_values.len()
                    ),
                });
            }
        }

        Ok(Self {
            x,
            intensities,
            imaginary,
            metadata,
            processing: Vec::new(),
        })
    }

    /// Returns the number of points.
    #[must_use]
    pub fn len(&self) -> usize {
        self.intensities.len()
    }

    /// Returns true when the spectrum has no points.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.intensities.is_empty()
    }

    /// Iterates over `(x, intensity)` pairs.
    pub fn points(&self) -> impl Iterator<Item = (f64, f64)> + '_ {
        self.x
            .values
            .iter()
            .copied()
            .zip(self.intensities.iter().copied())
    }

    /// Returns a copy with one appended processing record.
    #[must_use]
    pub fn with_processing_record(mut self, record: ProcessingRecord) -> Self {
        self.processing.push(record);
        self
    }
}

/// A two-dimensional spectrum with row-major `z` data.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Spectrum2D {
    /// X axis.
    pub x: Axis,
    /// Y axis.
    pub y: Axis,
    /// Row-major intensity matrix with `y.len() * x.len()` values.
    pub z: Vec<f64>,
    /// Optional row-major imaginary component.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub imaginary: Option<Vec<f64>>,
    /// Spectrum metadata.
    pub metadata: Metadata,
    /// Applied processing records.
    pub processing: Vec<ProcessingRecord>,
}

impl Spectrum2D {
    /// Creates a two-dimensional spectrum.
    ///
    /// # Errors
    ///
    /// Returns an error when matrix length is not `x.len() * y.len()` or data is non-finite.
    pub fn new(x: Axis, y: Axis, z: Vec<f64>, metadata: Metadata) -> Result<Self> {
        Self::new_complex(x, y, z, None, metadata)
    }

    /// Creates a two-dimensional spectrum with an optional imaginary channel.
    ///
    /// # Errors
    ///
    /// Returns an error when matrix length is not `x.len() * y.len()` or data is non-finite.
    pub fn new_complex(
        x: Axis,
        y: Axis,
        z: Vec<f64>,
        imaginary: Option<Vec<f64>>,
        metadata: Metadata,
    ) -> Result<Self> {
        validate_vector("z", &z)?;
        let expected = x
            .len()
            .checked_mul(y.len())
            .ok_or_else(|| RSpinError::InvalidSpectrum {
                message: "2D axis size overflow".to_owned(),
            })?;
        if z.len() != expected {
            return Err(RSpinError::InvalidSpectrum {
                message: format!("matrix has {} values but axes require {expected}", z.len()),
            });
        }
        if let Some(imaginary_values) = imaginary.as_deref() {
            validate_vector("imaginary", imaginary_values)?;
            if imaginary_values.len() != expected {
                return Err(RSpinError::InvalidSpectrum {
                    message: format!(
                        "imaginary matrix has {} values but axes require {expected}",
                        imaginary_values.len()
                    ),
                });
            }
        }
        Ok(Self {
            x,
            y,
            z,
            imaginary,
            metadata,
            processing: Vec::new(),
        })
    }

    /// Returns the `(x, y)` shape.
    #[must_use]
    pub fn shape(&self) -> (usize, usize) {
        (self.x.len(), self.y.len())
    }

    /// Gets a matrix value by x/y index.
    #[must_use]
    pub fn value_at(&self, x_index: usize, y_index: usize) -> Option<f64> {
        let (width, height) = self.shape();
        if x_index >= width || y_index >= height {
            return None;
        }
        self.z.get(y_index * width + x_index).copied()
    }

    /// Gets an imaginary matrix value by x/y index.
    #[must_use]
    pub fn imaginary_at(&self, x_index: usize, y_index: usize) -> Option<f64> {
        let (width, height) = self.shape();
        if x_index >= width || y_index >= height {
            return None;
        }
        self.imaginary
            .as_ref()
            .and_then(|values| values.get(y_index * width + x_index).copied())
    }

    /// Returns a copy with one appended processing record.
    #[must_use]
    pub fn with_processing_record(mut self, record: ProcessingRecord) -> Self {
        self.processing.push(record);
        self
    }
}

fn validate_vector(field: &'static str, values: &[f64]) -> Result<()> {
    if !values.iter().all(|value| value.is_finite()) {
        return Err(RSpinError::NonFinite { field });
    }
    Ok(())
}

#[cfg(test)]
mod tests;
