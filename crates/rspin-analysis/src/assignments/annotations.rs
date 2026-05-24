//! Assignment to spectrum annotation conversion.

use rspin_core::{AnnotationTarget, Result, Spectrum1D, Spectrum2D, SpectrumAnnotation};

use super::{AssignedAtom, Assignment, AssignmentSet, AssignmentTarget};

impl AssignmentTarget {
    /// Converts this assignment target into a spectrum annotation target.
    #[must_use]
    pub fn to_annotation_target(&self) -> AnnotationTarget {
        match self {
            Self::Peak1D { index, x } => AnnotationTarget::point_1d(*index, *x),
            Self::Range1D { from, to, .. } => AnnotationTarget::range_1d(*from, *to),
            Self::Zone2D { id } => AnnotationTarget::zone_2d_id(id.clone()),
        }
    }
}

impl Assignment {
    /// Converts this assignment into a spectrum annotation.
    ///
    /// # Errors
    ///
    /// Returns an error when the assignment or generated annotation is invalid.
    pub fn to_annotation(&self) -> Result<SpectrumAnnotation> {
        self.validate()?;
        let annotation =
            SpectrumAnnotation::new(self.id.clone(), self.target.to_annotation_target())
                .with_label(assignment_label(&self.atoms));
        annotation.validate()?;
        Ok(annotation)
    }
}

impl AssignmentSet {
    /// Converts all assignments into spectrum annotations.
    ///
    /// # Errors
    ///
    /// Returns an error when the set contains invalid or duplicate assignments,
    /// or when any generated annotation is invalid.
    pub fn to_annotations(&self) -> Result<Vec<SpectrumAnnotation>> {
        self.validate()?;
        self.assignments
            .iter()
            .map(Assignment::to_annotation)
            .collect()
    }

    /// Returns a one-dimensional spectrum with assignment annotations appended.
    ///
    /// # Errors
    ///
    /// Returns an error when assignments are invalid or include two-dimensional
    /// targets that cannot be attached to a one-dimensional spectrum.
    pub fn annotate_spectrum_1d(&self, spectrum: Spectrum1D) -> Result<Spectrum1D> {
        let annotated = self
            .to_annotations()?
            .into_iter()
            .fold(spectrum, Spectrum1D::with_annotation);
        annotated.validate_annotations()?;
        Ok(annotated)
    }

    /// Returns a two-dimensional spectrum with assignment annotations appended.
    ///
    /// # Errors
    ///
    /// Returns an error when assignments are invalid or include one-dimensional
    /// targets that cannot be attached to a two-dimensional spectrum.
    pub fn annotate_spectrum_2d(&self, spectrum: Spectrum2D) -> Result<Spectrum2D> {
        let annotated = self
            .to_annotations()?
            .into_iter()
            .fold(spectrum, Spectrum2D::with_annotation);
        annotated.validate_annotations()?;
        Ok(annotated)
    }
}

fn assignment_label(atoms: &[AssignedAtom]) -> String {
    atoms
        .iter()
        .map(|atom| match atom.label.as_deref() {
            Some(label) => label,
            None => atom.id.as_str(),
        })
        .collect::<Vec<_>>()
        .join(",")
}
