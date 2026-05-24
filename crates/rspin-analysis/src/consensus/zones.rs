//! Consensus zone tables for two-dimensional spectra.

use std::collections::BTreeSet;

use rspin_core::{RSpinError, Result, Spectrum2D};

use crate::{DetectedZone, detect_zones};

use super::{
    ConsensusZone2D, ConsensusZoneMember2D, ConsensusZoneOptions, require_finite, row_id_2d,
    validate_spectra_2d,
};

/// Detects and groups consensus zones across two-dimensional spectra.
///
/// Zones are detected independently for each spectrum, sorted by their
/// normalized bounding boxes, then grouped when the boxes overlap or are
/// separated by no more than the configured x/y gaps.
///
/// # Errors
///
/// Returns an error when options are invalid, no spectra are provided, spectra
/// use incompatible units, or zone detection fails.
pub fn detect_consensus_zones_2d(
    spectra: &[Spectrum2D],
    options: ConsensusZoneOptions,
) -> Result<Vec<ConsensusZone2D>> {
    options.validate()?;
    validate_spectra_2d(spectra)?;

    let observations = zone_observations(spectra, options)?;
    if observations.is_empty() {
        return Ok(Vec::new());
    }

    let groups = group_observations(observations, options.max_x_gap, options.max_y_gap);
    groups
        .into_iter()
        .filter_map(|group| {
            let spectrum_count = group.spectrum_count();
            if spectrum_count >= options.min_spectrum_count {
                Some(build_consensus_zone(group))
            } else {
                None
            }
        })
        .enumerate()
        .map(|(index, result)| {
            result.map(|mut zone| {
                zone.id = format!("consensus-zone2d:{index}");
                zone
            })
        })
        .collect()
}

#[derive(Clone, Debug)]
struct ZoneObservation {
    row_id: String,
    spectrum_index: usize,
    zone: DetectedZone,
    x_from: f64,
    x_to: f64,
    y_from: f64,
    y_to: f64,
}

#[derive(Clone, Debug)]
struct ZoneGroup {
    x_from: f64,
    x_to: f64,
    y_from: f64,
    y_to: f64,
    members: Vec<ZoneObservation>,
}

impl ZoneGroup {
    fn new(observation: ZoneObservation) -> Self {
        Self {
            x_from: observation.x_from,
            x_to: observation.x_to,
            y_from: observation.y_from,
            y_to: observation.y_to,
            members: vec![observation],
        }
    }

    fn accepts(
        &self,
        observation: &ZoneObservation,
        horizontal_gap_limit: f64,
        vertical_gap_limit: f64,
    ) -> bool {
        axis_gap(self.x_from, self.x_to, observation.x_from, observation.x_to)
            <= horizontal_gap_limit
            && axis_gap(self.y_from, self.y_to, observation.y_from, observation.y_to)
                <= vertical_gap_limit
    }

    fn push(&mut self, observation: ZoneObservation) {
        self.x_from = self.x_from.min(observation.x_from);
        self.x_to = self.x_to.max(observation.x_to);
        self.y_from = self.y_from.min(observation.y_from);
        self.y_to = self.y_to.max(observation.y_to);
        self.members.push(observation);
    }

    fn spectrum_count(&self) -> usize {
        self.members
            .iter()
            .map(|member| member.spectrum_index)
            .collect::<BTreeSet<_>>()
            .len()
    }
}

fn zone_observations(
    spectra: &[Spectrum2D],
    options: ConsensusZoneOptions,
) -> Result<Vec<ZoneObservation>> {
    let mut observations = Vec::new();
    for (spectrum_index, spectrum) in spectra.iter().enumerate() {
        let row_id = row_id_2d(spectrum_index, spectrum);
        for zone in detect_zones(spectrum, options.zone_options)? {
            let (x_from, x_to) = normalized_bounds(zone.x_from, zone.x_to)?;
            let (y_from, y_to) = normalized_bounds(zone.y_from, zone.y_to)?;
            observations.push(ZoneObservation {
                row_id: row_id.clone(),
                spectrum_index,
                zone,
                x_from,
                x_to,
                y_from,
                y_to,
            });
        }
    }
    observations.sort_by(|left, right| {
        left.x_from
            .total_cmp(&right.x_from)
            .then_with(|| left.y_from.total_cmp(&right.y_from))
            .then_with(|| left.x_to.total_cmp(&right.x_to))
            .then_with(|| left.y_to.total_cmp(&right.y_to))
            .then_with(|| left.spectrum_index.cmp(&right.spectrum_index))
            .then_with(|| left.zone.id.cmp(&right.zone.id))
    });
    Ok(observations)
}

fn group_observations(
    observations: Vec<ZoneObservation>,
    horizontal_gap_limit: f64,
    vertical_gap_limit: f64,
) -> Vec<ZoneGroup> {
    let mut groups: Vec<ZoneGroup> = Vec::new();
    for observation in observations {
        if let Some(group) = groups
            .iter_mut()
            .find(|group| group.accepts(&observation, horizontal_gap_limit, vertical_gap_limit))
        {
            group.push(observation);
        } else {
            groups.push(ZoneGroup::new(observation));
        }
    }
    groups.sort_by(|left, right| {
        left.x_from
            .total_cmp(&right.x_from)
            .then_with(|| left.y_from.total_cmp(&right.y_from))
            .then_with(|| left.x_to.total_cmp(&right.x_to))
            .then_with(|| left.y_to.total_cmp(&right.y_to))
    });
    groups
}

fn build_consensus_zone(mut group: ZoneGroup) -> Result<ConsensusZone2D> {
    group.members.sort_by(|left, right| {
        left.spectrum_index
            .cmp(&right.spectrum_index)
            .then_with(|| left.zone.id.cmp(&right.zone.id))
    });
    let spectrum_count = group.spectrum_count();
    let zone_count = group.members.len();
    let total_abs_intensity = group
        .members
        .iter()
        .map(|member| member.zone.sum_abs_intensity)
        .sum::<f64>();
    let max_abs_intensity = group
        .members
        .iter()
        .map(|member| member.zone.max_abs_intensity)
        .fold(0.0_f64, f64::max);
    let (centroid_x, centroid_y) = centroid(&group.members, total_abs_intensity)?;
    let members = group
        .members
        .into_iter()
        .map(|member| ConsensusZoneMember2D {
            row_id: member.row_id,
            spectrum_index: member.spectrum_index,
            zone: member.zone,
        })
        .collect();

    Ok(ConsensusZone2D {
        id: String::new(),
        x_from: group.x_from,
        x_to: group.x_to,
        y_from: group.y_from,
        y_to: group.y_to,
        centroid_x,
        centroid_y,
        zone_count,
        spectrum_count,
        total_abs_intensity,
        max_abs_intensity,
        members,
    })
}

fn centroid(members: &[ZoneObservation], total_abs_intensity: f64) -> Result<(f64, f64)> {
    if total_abs_intensity > f64::EPSILON {
        let x = members
            .iter()
            .map(|member| member.zone.centroid_x * member.zone.sum_abs_intensity)
            .sum::<f64>()
            / total_abs_intensity;
        let y = members
            .iter()
            .map(|member| member.zone.centroid_y * member.zone.sum_abs_intensity)
            .sum::<f64>()
            / total_abs_intensity;
        return Ok((x, y));
    }
    let count = u32::try_from(members.len()).map_err(|_| RSpinError::InvalidSpectrum {
        message: "too many consensus zones to average".to_owned(),
    })?;
    let scale = 1.0 / f64::from(count);
    Ok((
        members
            .iter()
            .map(|member| member.zone.centroid_x)
            .sum::<f64>()
            * scale,
        members
            .iter()
            .map(|member| member.zone.centroid_y)
            .sum::<f64>()
            * scale,
    ))
}

fn axis_gap(left_from: f64, left_to: f64, right_from: f64, right_to: f64) -> f64 {
    if left_to < right_from {
        right_from - left_to
    } else if right_to < left_from {
        left_from - right_to
    } else {
        0.0
    }
}

fn normalized_bounds(left: f64, right: f64) -> Result<(f64, f64)> {
    require_finite("zone bound", left)?;
    require_finite("zone bound", right)?;
    if left <= right {
        Ok((left, right))
    } else {
        Ok((right, left))
    }
}
