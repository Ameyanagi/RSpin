//! Consensus range tables for one-dimensional spectra.

use std::collections::BTreeSet;

use rspin_core::{RSpinError, Result, Spectrum1D};

use crate::detect_ranges;

use super::{
    ConsensusRange1D, ConsensusRangeMember1D, ConsensusRangeOptions, require_finite, row_id,
    validate_spectra,
};

/// Detects and groups consensus ranges across one-dimensional spectra.
///
/// Ranges are detected independently for each spectrum, sorted by their
/// normalized coordinate span, then grouped when spans overlap or are separated
/// by no more than [`ConsensusRangeOptions::max_gap`].
///
/// # Errors
///
/// Returns an error when options are invalid, no spectra are provided, spectra
/// use incompatible x units, or range detection fails.
pub fn detect_consensus_ranges_1d(
    spectra: &[Spectrum1D],
    options: ConsensusRangeOptions,
) -> Result<Vec<ConsensusRange1D>> {
    options.validate()?;
    validate_spectra(spectra)?;

    let observations = range_observations(spectra, options)?;
    if observations.is_empty() {
        return Ok(Vec::new());
    }

    let groups = group_observations(observations, options.max_gap);
    groups
        .into_iter()
        .filter_map(|group| {
            let spectrum_count = group.spectrum_count();
            if spectrum_count >= options.min_spectrum_count {
                Some(build_consensus_range(group))
            } else {
                None
            }
        })
        .enumerate()
        .map(|(index, result)| {
            result.map(|mut range| {
                range.id = format!("consensus-range1d:{index}");
                range
            })
        })
        .collect()
}

#[derive(Clone, Debug)]
struct RangeObservation {
    row_id: String,
    spectrum_index: usize,
    range: crate::DetectedRange,
    from: f64,
    to: f64,
}

#[derive(Clone, Debug)]
struct RangeGroup {
    from: f64,
    to: f64,
    members: Vec<RangeObservation>,
}

impl RangeGroup {
    fn new(observation: RangeObservation) -> Self {
        Self {
            from: observation.from,
            to: observation.to,
            members: vec![observation],
        }
    }

    fn accepts(&self, observation: &RangeObservation, max_gap: f64) -> bool {
        observation.from <= self.to + max_gap
    }

    fn push(&mut self, observation: RangeObservation) {
        self.from = self.from.min(observation.from);
        self.to = self.to.max(observation.to);
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

fn range_observations(
    spectra: &[Spectrum1D],
    options: ConsensusRangeOptions,
) -> Result<Vec<RangeObservation>> {
    let mut observations = Vec::new();
    for (spectrum_index, spectrum) in spectra.iter().enumerate() {
        let row_id = row_id(spectrum_index, spectrum);
        for range in detect_ranges(spectrum, options.range_options)? {
            let (from, to) = normalized_bounds(range.from, range.to)?;
            observations.push(RangeObservation {
                row_id: row_id.clone(),
                spectrum_index,
                range,
                from,
                to,
            });
        }
    }
    observations.sort_by(|left, right| {
        left.from
            .total_cmp(&right.from)
            .then_with(|| left.to.total_cmp(&right.to))
            .then_with(|| left.spectrum_index.cmp(&right.spectrum_index))
            .then_with(|| left.range.start_index.cmp(&right.range.start_index))
    });
    Ok(observations)
}

fn group_observations(observations: Vec<RangeObservation>, max_gap: f64) -> Vec<RangeGroup> {
    let mut groups = Vec::new();
    let mut current = None;

    for observation in observations {
        current = match current {
            None => Some(RangeGroup::new(observation)),
            Some(mut group) if group.accepts(&observation, max_gap) => {
                group.push(observation);
                Some(group)
            }
            Some(group) => {
                groups.push(group);
                Some(RangeGroup::new(observation))
            }
        };
    }

    if let Some(group) = current {
        groups.push(group);
    }
    groups
}

fn build_consensus_range(mut group: RangeGroup) -> Result<ConsensusRange1D> {
    group.members.sort_by(|left, right| {
        left.spectrum_index
            .cmp(&right.spectrum_index)
            .then_with(|| left.range.start_index.cmp(&right.range.start_index))
    });
    let spectrum_count = group.spectrum_count();
    let range_count = group.members.len();
    let total_abs_area = group
        .members
        .iter()
        .map(|member| member.range.area.abs())
        .sum::<f64>();
    let max_abs_intensity = group
        .members
        .iter()
        .map(|member| member.range.max_abs_intensity)
        .fold(0.0_f64, f64::max);
    let center_x = center_x(&group.members, total_abs_area)?;
    let members = group
        .members
        .into_iter()
        .map(|member| ConsensusRangeMember1D {
            row_id: member.row_id,
            spectrum_index: member.spectrum_index,
            range: member.range,
        })
        .collect();

    Ok(ConsensusRange1D {
        id: String::new(),
        from: group.from,
        to: group.to,
        center_x,
        range_count,
        spectrum_count,
        total_abs_area,
        max_abs_intensity,
        members,
    })
}

fn center_x(members: &[RangeObservation], total_abs_area: f64) -> Result<f64> {
    if total_abs_area > f64::EPSILON {
        return Ok(members
            .iter()
            .map(|member| midpoint(member.from, member.to) * member.range.area.abs())
            .sum::<f64>()
            / total_abs_area);
    }
    let count = u32::try_from(members.len()).map_err(|_| RSpinError::InvalidSpectrum {
        message: "too many consensus ranges to average".to_owned(),
    })?;
    Ok(members
        .iter()
        .map(|member| midpoint(member.from, member.to))
        .sum::<f64>()
        / f64::from(count))
}

fn midpoint(from: f64, to: f64) -> f64 {
    (from + to) * 0.5
}

fn normalized_bounds(left: f64, right: f64) -> Result<(f64, f64)> {
    require_finite("range from", left)?;
    require_finite("range to", right)?;
    if left <= right {
        Ok((left, right))
    } else {
        Ok((right, left))
    }
}
