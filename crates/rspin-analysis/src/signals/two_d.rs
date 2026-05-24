//! Two-dimensional signal summaries assembled from zones and assignments.

use std::collections::BTreeSet;

use rspin_core::{RSpinError, Result, Spectrum2D};

use crate::{
    Assignment, AssignmentSet, AssignmentTarget, DetectedZone, SignalSummary2D,
    SignalSummary2DOptions,
};

use super::{collect_atoms, push_assignments_for_target, require_finite, validate_non_empty};

/// Assembles stable two-dimensional signal summaries from zones and assignments.
///
/// # Errors
///
/// Returns an error when zones do not map onto `spectrum`, when zone ids are
/// duplicated, or when assignments are invalid.
pub fn summarize_signals_2d(
    spectrum: &Spectrum2D,
    zones: &[DetectedZone],
    assignments: &AssignmentSet,
    options: SignalSummary2DOptions,
) -> Result<Vec<SignalSummary2D>> {
    validate_zones(spectrum, zones)?;
    assignments.validate()?;

    let mut summaries = Vec::new();
    for zone in zones {
        let assignments = collect_zone_assignments(zone, assignments);
        if assignments.is_empty() && !options.include_unassigned_zones {
            continue;
        }
        let atoms = collect_atoms(&assignments);
        summaries.push(SignalSummary2D {
            id: format!("signal2d:{}", zone.id),
            zone: zone.clone(),
            center_x: zone.centroid_x,
            center_y: zone.centroid_y,
            x_from: zone.x_from,
            x_to: zone.x_to,
            y_from: zone.y_from,
            y_to: zone.y_to,
            active_points: zone.active_points,
            max_abs_intensity: zone.max_abs_intensity,
            sum_intensity: zone.sum_intensity,
            sum_abs_intensity: zone.sum_abs_intensity,
            assignments,
            atoms,
        });
    }

    Ok(summaries)
}

fn validate_zones(spectrum: &Spectrum2D, zones: &[DetectedZone]) -> Result<()> {
    let (width, height) = spectrum.shape();
    let mut ids = BTreeSet::new();
    for zone in zones {
        validate_non_empty("zone id", &zone.id)?;
        if !ids.insert(zone.id.as_str()) {
            return Err(RSpinError::InvalidSpectrum {
                message: format!("duplicate zone id '{}'", zone.id),
            });
        }
        if zone.x_start_index > zone.x_end_index || zone.y_start_index > zone.y_end_index {
            return Err(RSpinError::InvalidSpectrum {
                message: "zone start index must not exceed end index".to_owned(),
            });
        }
        if zone.x_end_index >= width || zone.y_end_index >= height {
            return Err(RSpinError::InvalidSpectrum {
                message: "zone index is outside the spectrum".to_owned(),
            });
        }
        if zone.active_points == 0 {
            return Err(RSpinError::InvalidSpectrum {
                message: "zone active points must be positive".to_owned(),
            });
        }
        require_finite("zone x_from", zone.x_from)?;
        require_finite("zone x_to", zone.x_to)?;
        require_finite("zone y_from", zone.y_from)?;
        require_finite("zone y_to", zone.y_to)?;
        require_finite("zone centroid_x", zone.centroid_x)?;
        require_finite("zone centroid_y", zone.centroid_y)?;
        require_finite("zone max_abs_intensity", zone.max_abs_intensity)?;
        require_finite("zone sum_intensity", zone.sum_intensity)?;
        require_finite("zone sum_abs_intensity", zone.sum_abs_intensity)?;
    }
    Ok(())
}

fn collect_zone_assignments(zone: &DetectedZone, assignments: &AssignmentSet) -> Vec<Assignment> {
    let target = AssignmentTarget::zone_2d(zone);
    let mut seen = BTreeSet::new();
    let mut collected = Vec::new();
    push_assignments_for_target(assignments, &target, &mut seen, &mut collected);
    collected
}
