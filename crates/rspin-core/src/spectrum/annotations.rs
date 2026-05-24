//! Annotation helpers for spectrum types.

use crate::{AnnotationTarget, RSpinError, Result, SpectrumAnnotation};

use super::{Spectrum1D, Spectrum2D};
use crate::marks::validate_annotation_collection;

impl Spectrum1D {
    /// Returns a copy with one appended annotation.
    #[must_use]
    pub fn with_annotation(mut self, annotation: SpectrumAnnotation) -> Self {
        self.annotations.push(annotation);
        self
    }

    /// Returns a copy with all annotations replaced.
    #[must_use]
    pub fn with_annotations(mut self, annotations: Vec<SpectrumAnnotation>) -> Self {
        self.annotations = annotations;
        self
    }

    /// Returns a copy with annotations removed.
    #[must_use]
    pub fn without_annotations(mut self) -> Self {
        self.annotations.clear();
        self
    }

    /// Finds an annotation by stable identifier.
    #[must_use]
    pub fn annotation(&self, id: &str) -> Option<&SpectrumAnnotation> {
        self.annotations
            .iter()
            .find(|annotation| annotation.id == id)
    }

    /// Validates annotation targets and duplicate annotation IDs.
    ///
    /// # Errors
    ///
    /// Returns an error when annotation data is invalid or duplicate annotation
    /// IDs are present.
    pub fn validate_annotations(&self) -> Result<()> {
        for annotation in &self.annotations {
            validate_1d_annotation_target(&annotation.target)?;
        }
        validate_annotation_collection(&self.annotations)
    }
}

impl Spectrum2D {
    /// Returns a copy with one appended annotation.
    #[must_use]
    pub fn with_annotation(mut self, annotation: SpectrumAnnotation) -> Self {
        self.annotations.push(annotation);
        self
    }

    /// Returns a copy with all annotations replaced.
    #[must_use]
    pub fn with_annotations(mut self, annotations: Vec<SpectrumAnnotation>) -> Self {
        self.annotations = annotations;
        self
    }

    /// Returns a copy with annotations removed.
    #[must_use]
    pub fn without_annotations(mut self) -> Self {
        self.annotations.clear();
        self
    }

    /// Finds an annotation by stable identifier.
    #[must_use]
    pub fn annotation(&self, id: &str) -> Option<&SpectrumAnnotation> {
        self.annotations
            .iter()
            .find(|annotation| annotation.id == id)
    }

    /// Validates annotation targets and duplicate annotation IDs.
    ///
    /// # Errors
    ///
    /// Returns an error when annotation data is invalid or duplicate annotation
    /// IDs are present.
    pub fn validate_annotations(&self) -> Result<()> {
        for annotation in &self.annotations {
            validate_2d_annotation_target(&annotation.target)?;
        }
        validate_annotation_collection(&self.annotations)
    }
}

fn validate_1d_annotation_target(target: &AnnotationTarget) -> Result<()> {
    if matches!(
        target,
        AnnotationTarget::Point2D { .. }
            | AnnotationTarget::Zone2D { .. }
            | AnnotationTarget::Zone2DId { .. }
    ) {
        return Err(RSpinError::InvalidMetadata {
            message: "1D spectra cannot use 2D annotation targets".to_owned(),
        });
    }
    target.validate()
}

fn validate_2d_annotation_target(target: &AnnotationTarget) -> Result<()> {
    if matches!(
        target,
        AnnotationTarget::Point1D { .. } | AnnotationTarget::Range1D { .. }
    ) {
        return Err(RSpinError::InvalidMetadata {
            message: "2D spectra cannot use 1D annotation targets".to_owned(),
        });
    }
    target.validate()
}
