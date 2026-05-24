//! Consensus peak tables for multi-spectrum analysis.

use std::collections::BTreeSet;

use rspin_core::{RSpinError, Result, Spectrum1D, Spectrum2D};

use crate::{Peak, PeakPolarity, pick_peaks};

mod model;
mod ranges;
mod zones;

pub use model::{
    ConsensusPeak1D, ConsensusPeakMember1D, ConsensusPeakOptions, ConsensusRange1D,
    ConsensusRangeMember1D, ConsensusRangeOptions, ConsensusZone2D, ConsensusZoneMember2D,
    ConsensusZoneOptions,
};
pub use ranges::detect_consensus_ranges_1d;
pub use zones::detect_consensus_zones_2d;

/// Detects and groups consensus peaks across one-dimensional spectra.
///
/// Peaks are picked independently for each spectrum, sorted by coordinate, then
/// grouped when the coordinate span stays within
/// [`ConsensusPeakOptions::max_shift`] and peak polarities are compatible.
///
/// # Errors
///
/// Returns an error when options are invalid, no spectra are provided, spectra
/// use incompatible x units, or peak picking fails.
pub fn detect_consensus_peaks_1d(
    spectra: &[Spectrum1D],
    options: ConsensusPeakOptions,
) -> Result<Vec<ConsensusPeak1D>> {
    options.validate()?;
    validate_spectra(spectra)?;

    let observations = peak_observations(spectra, options)?;
    if observations.is_empty() {
        return Ok(Vec::new());
    }

    let groups = group_observations(observations, options.max_shift);
    groups
        .into_iter()
        .filter_map(|group| {
            let spectrum_count = group.spectrum_count();
            if spectrum_count >= options.min_spectrum_count {
                Some(build_consensus_peak(group))
            } else {
                None
            }
        })
        .enumerate()
        .map(|(index, result)| {
            result.map(|mut peak| {
                peak.id = format!("consensus1d:{index}");
                peak
            })
        })
        .collect()
}

#[derive(Clone, Debug)]
struct PeakObservation {
    row_id: String,
    spectrum_index: usize,
    peak: Peak,
}

#[derive(Clone, Debug)]
struct ObservationGroup {
    polarity: PeakPolarity,
    from_x: f64,
    members: Vec<PeakObservation>,
}

impl ObservationGroup {
    fn new(observation: PeakObservation) -> Self {
        Self {
            polarity: observation.peak.polarity,
            from_x: observation.peak.x,
            members: vec![observation],
        }
    }

    fn accepts(&self, observation: &PeakObservation, max_shift: f64) -> bool {
        polarities_compatible(self.polarity, observation.peak.polarity)
            && observation.peak.x - self.from_x <= max_shift
    }

    fn push(&mut self, observation: PeakObservation) {
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

fn validate_spectra(spectra: &[Spectrum1D]) -> Result<()> {
    let first = spectra.first().ok_or_else(|| RSpinError::InvalidSpectrum {
        message: "consensus peak detection requires at least one spectrum".to_owned(),
    })?;
    for spectrum in spectra {
        if spectrum.x.unit != first.x.unit {
            return Err(RSpinError::InvalidSpectrum {
                message: "consensus peak spectra must use the same x unit".to_owned(),
            });
        }
    }
    Ok(())
}

fn validate_spectra_2d(spectra: &[Spectrum2D]) -> Result<()> {
    let first = spectra.first().ok_or_else(|| RSpinError::InvalidSpectrum {
        message: "consensus zone detection requires at least one spectrum".to_owned(),
    })?;
    for spectrum in spectra {
        if spectrum.x.unit != first.x.unit {
            return Err(RSpinError::InvalidSpectrum {
                message: "consensus zone spectra must use the same x unit".to_owned(),
            });
        }
        if spectrum.y.unit != first.y.unit {
            return Err(RSpinError::InvalidSpectrum {
                message: "consensus zone spectra must use the same y unit".to_owned(),
            });
        }
    }
    Ok(())
}

fn peak_observations(
    spectra: &[Spectrum1D],
    options: ConsensusPeakOptions,
) -> Result<Vec<PeakObservation>> {
    let mut observations = Vec::new();
    for (spectrum_index, spectrum) in spectra.iter().enumerate() {
        let row_id = row_id(spectrum_index, spectrum);
        for peak in pick_peaks(spectrum, options.peak_options)? {
            validate_peak(&peak)?;
            observations.push(PeakObservation {
                row_id: row_id.clone(),
                spectrum_index,
                peak,
            });
        }
    }
    observations.sort_by(|left, right| {
        left.peak
            .x
            .total_cmp(&right.peak.x)
            .then_with(|| left.spectrum_index.cmp(&right.spectrum_index))
            .then_with(|| left.peak.index.cmp(&right.peak.index))
    });
    Ok(observations)
}

fn group_observations(observations: Vec<PeakObservation>, max_shift: f64) -> Vec<ObservationGroup> {
    let mut groups = Vec::new();
    let mut current = None;

    for observation in observations {
        current = match current {
            None => Some(ObservationGroup::new(observation)),
            Some(mut group) if group.accepts(&observation, max_shift) => {
                group.push(observation);
                Some(group)
            }
            Some(group) => {
                groups.push(group);
                Some(ObservationGroup::new(observation))
            }
        };
    }

    if let Some(group) = current {
        groups.push(group);
    }
    groups
}

fn build_consensus_peak(mut group: ObservationGroup) -> Result<ConsensusPeak1D> {
    group.members.sort_by(|left, right| {
        left.spectrum_index
            .cmp(&right.spectrum_index)
            .then_with(|| left.peak.index.cmp(&right.peak.index))
    });
    let spectrum_count = group.spectrum_count();
    let peak_count = group.members.len();
    let total_abs_intensity = group
        .members
        .iter()
        .map(|member| member.peak.intensity.abs())
        .sum::<f64>();
    let from_x = group
        .members
        .iter()
        .map(|member| member.peak.x)
        .fold(f64::INFINITY, f64::min);
    let to_x = group
        .members
        .iter()
        .map(|member| member.peak.x)
        .fold(f64::NEG_INFINITY, f64::max);
    let center_x = center_x(&group.members, total_abs_intensity)?;
    let members = group
        .members
        .into_iter()
        .map(|member| ConsensusPeakMember1D {
            row_id: member.row_id,
            spectrum_index: member.spectrum_index,
            peak: member.peak,
        })
        .collect();

    Ok(ConsensusPeak1D {
        id: String::new(),
        center_x,
        from_x,
        to_x,
        peak_count,
        spectrum_count,
        total_abs_intensity,
        members,
    })
}

fn center_x(members: &[PeakObservation], total_abs_intensity: f64) -> Result<f64> {
    if total_abs_intensity > f64::EPSILON {
        return Ok(members
            .iter()
            .map(|member| member.peak.x * member.peak.intensity.abs())
            .sum::<f64>()
            / total_abs_intensity);
    }
    let count = u32::try_from(members.len()).map_err(|_| RSpinError::InvalidSpectrum {
        message: "too many consensus peaks to average".to_owned(),
    })?;
    Ok(members.iter().map(|member| member.peak.x).sum::<f64>() / f64::from(count))
}

fn validate_peak(peak: &Peak) -> Result<()> {
    require_finite("peak x", peak.x)?;
    require_finite("peak intensity", peak.intensity)?;
    require_finite("peak prominence", peak.prominence)
}

fn require_finite(field: &'static str, value: f64) -> Result<()> {
    if !value.is_finite() {
        return Err(RSpinError::NonFinite { field });
    }
    Ok(())
}

fn polarities_compatible(left: PeakPolarity, right: PeakPolarity) -> bool {
    left == right || matches!(left, PeakPolarity::Both) || matches!(right, PeakPolarity::Both)
}

fn row_id(index: usize, spectrum: &Spectrum1D) -> String {
    match spectrum.metadata.name.as_deref() {
        Some(name) if !name.trim().is_empty() => format!("{index}:{}", sanitize_id_token(name)),
        _ => format!("spectrum-{index}"),
    }
}

fn row_id_2d(index: usize, spectrum: &Spectrum2D) -> String {
    match spectrum.metadata.name.as_deref() {
        Some(name) if !name.trim().is_empty() => format!("{index}:{}", sanitize_id_token(name)),
        _ => format!("spectrum-{index}"),
    }
}

fn sanitize_id_token(value: &str) -> String {
    value
        .trim()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.') {
                character
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests;
