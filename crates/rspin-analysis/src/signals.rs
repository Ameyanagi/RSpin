//! One-dimensional signal summaries assembled from analysis primitives.

use std::collections::BTreeSet;

use rspin_core::{RSpinError, Result, Spectrum1D};

use crate::{
    AssignedAtom, Assignment, AssignmentSet, AssignmentTarget, DetectedMultiplet, DetectedRange,
    JCoupling, JCouplingGraph, Peak,
};

mod model;

pub use model::{SignalSummary1D, SignalSummaryOptions};

/// Assembles stable one-dimensional signal summaries from ranges, multiplets,
/// assignments, and a J-coupling graph.
///
/// # Errors
///
/// Returns an error when ranges or multiplets do not map onto `spectrum`, when
/// assignments or coupling graph data are invalid, or when summary coordinates
/// cannot be derived.
pub fn summarize_signals_1d(
    spectrum: &Spectrum1D,
    ranges: &[DetectedRange],
    multiplets: &[DetectedMultiplet],
    assignments: &AssignmentSet,
    coupling_graph: &JCouplingGraph,
    options: SignalSummaryOptions,
) -> Result<Vec<SignalSummary1D>> {
    validate_ranges(spectrum, ranges)?;
    validate_multiplets(spectrum, multiplets)?;
    assignments.validate()?;
    coupling_graph.validate()?;

    let mut summaries = Vec::new();
    let mut used_multiplets = BTreeSet::new();
    for range in ranges {
        let indexed = multiplets_in_range(range, multiplets);
        if indexed.is_empty() && !options.include_empty_ranges {
            continue;
        }

        let attached = indexed
            .iter()
            .map(|(index, multiplet)| {
                used_multiplets.insert(*index);
                (*multiplet).clone()
            })
            .collect::<Vec<_>>();
        summaries.push(build_signal(
            range_signal_id(range),
            Some(range.clone()),
            attached,
            assignments,
            coupling_graph,
        )?);
    }

    if options.include_orphan_multiplets {
        for (index, multiplet) in multiplets.iter().enumerate() {
            if used_multiplets.contains(&index) {
                continue;
            }
            summaries.push(build_signal(
                orphan_signal_id(multiplet),
                None,
                vec![multiplet.clone()],
                assignments,
                coupling_graph,
            )?);
        }
    }

    Ok(summaries)
}

fn validate_ranges(spectrum: &Spectrum1D, ranges: &[DetectedRange]) -> Result<()> {
    for range in ranges {
        if range.start_index > range.end_index {
            return Err(RSpinError::InvalidSpectrum {
                message: "range start index must not exceed end index".to_owned(),
            });
        }
        if range.end_index >= spectrum.len() {
            return Err(RSpinError::InvalidSpectrum {
                message: "range end index is outside the spectrum".to_owned(),
            });
        }
        require_finite("range from", range.from)?;
        require_finite("range to", range.to)?;
        require_finite("range max_abs_intensity", range.max_abs_intensity)?;
        require_finite("range area", range.area)?;
    }
    Ok(())
}

fn validate_multiplets(spectrum: &Spectrum1D, multiplets: &[DetectedMultiplet]) -> Result<()> {
    let mut ids = BTreeSet::new();
    for multiplet in multiplets {
        validate_non_empty("multiplet id", &multiplet.id)?;
        if !ids.insert(multiplet.id.as_str()) {
            return Err(RSpinError::InvalidSpectrum {
                message: format!("duplicate multiplet id '{}'", multiplet.id),
            });
        }
        require_finite("multiplet center_ppm", multiplet.center_ppm)?;
        require_finite("multiplet from_ppm", multiplet.from_ppm)?;
        require_finite("multiplet to_ppm", multiplet.to_ppm)?;
        require_finite(
            "multiplet total_abs_intensity",
            multiplet.total_abs_intensity,
        )?;
        if let Some(j_hz) = multiplet.estimated_j_hz {
            require_finite("multiplet estimated_j_hz", j_hz)?;
        }
        for spacing in &multiplet.spacings_ppm {
            require_finite("multiplet spacing", *spacing)?;
        }
        for peak in &multiplet.peaks {
            validate_peak(spectrum, peak)?;
        }
    }
    Ok(())
}

fn validate_peak(spectrum: &Spectrum1D, peak: &Peak) -> Result<()> {
    if peak.index >= spectrum.len() {
        return Err(RSpinError::InvalidSpectrum {
            message: "multiplet peak index is outside the spectrum".to_owned(),
        });
    }
    require_finite("peak x", peak.x)?;
    require_finite("peak intensity", peak.intensity)?;
    require_finite("peak prominence", peak.prominence)
}

fn multiplets_in_range<'a>(
    range: &DetectedRange,
    multiplets: &'a [DetectedMultiplet],
) -> Vec<(usize, &'a DetectedMultiplet)> {
    multiplets
        .iter()
        .enumerate()
        .filter(|(_, multiplet)| range_contains_multiplet(range, multiplet))
        .collect()
}

fn range_contains_multiplet(range: &DetectedRange, multiplet: &DetectedMultiplet) -> bool {
    multiplet
        .peaks
        .iter()
        .any(|peak| peak.index >= range.start_index && peak.index <= range.end_index)
        || coordinate_inside(multiplet.center_ppm, range.from, range.to)
}

fn coordinate_inside(value: f64, left: f64, right: f64) -> bool {
    let min = left.min(right);
    let max = left.max(right);
    value >= min && value <= max
}

fn build_signal(
    id: String,
    range: Option<DetectedRange>,
    multiplets: Vec<DetectedMultiplet>,
    assignments: &AssignmentSet,
    coupling_graph: &JCouplingGraph,
) -> Result<SignalSummary1D> {
    let (from_ppm, to_ppm) = signal_span(range.as_ref(), &multiplets)?;
    let center_ppm = signal_center(range.as_ref(), &multiplets, from_ppm, to_ppm)?;
    let multiplet_kinds = multiplets
        .iter()
        .map(|multiplet| multiplet.kind)
        .collect::<Vec<_>>();
    let estimated_j_hz = multiplets
        .iter()
        .filter_map(|multiplet| multiplet.estimated_j_hz)
        .collect::<Vec<_>>();
    let peak_count = multiplets
        .iter()
        .map(|multiplet| multiplet.peaks.len())
        .sum();
    let area = range.as_ref().map(|range| range.area);
    let max_abs_intensity = max_abs_intensity(range.as_ref(), &multiplets);
    let assignments = collect_assignments(range.as_ref(), &multiplets, assignments);
    let atoms = collect_atoms(&assignments);
    let couplings = collect_couplings(&atoms, coupling_graph);

    Ok(SignalSummary1D {
        id,
        from_ppm,
        to_ppm,
        center_ppm,
        range,
        multiplets,
        multiplet_kinds,
        estimated_j_hz,
        peak_count,
        area,
        max_abs_intensity,
        assignments,
        atoms,
        couplings,
    })
}

fn signal_span(
    range: Option<&DetectedRange>,
    multiplets: &[DetectedMultiplet],
) -> Result<(f64, f64)> {
    if let Some(range) = range {
        return Ok((range.from, range.to));
    }

    let mut values = multiplets
        .iter()
        .flat_map(|multiplet| [multiplet.from_ppm, multiplet.to_ppm]);
    let Some(first) = values.next() else {
        return Err(RSpinError::InvalidSpectrum {
            message: "signal requires a range or at least one multiplet".to_owned(),
        });
    };
    let (min, max) = values.fold((first, first), |(min, max), value| {
        (min.min(value), max.max(value))
    });
    Ok((min, max))
}

fn signal_center(
    range: Option<&DetectedRange>,
    multiplets: &[DetectedMultiplet],
    from_ppm: f64,
    to_ppm: f64,
) -> Result<f64> {
    let total_weight = multiplets
        .iter()
        .map(|multiplet| multiplet.total_abs_intensity)
        .sum::<f64>();
    if total_weight > 0.0 {
        return Ok(multiplets
            .iter()
            .map(|multiplet| multiplet.center_ppm * multiplet.total_abs_intensity)
            .sum::<f64>()
            / total_weight);
    }
    if let Some(range) = range {
        return Ok(f64::midpoint(range.from, range.to));
    }
    if multiplets.is_empty() {
        return Err(RSpinError::InvalidSpectrum {
            message: "signal requires a range or at least one multiplet".to_owned(),
        });
    }
    Ok(f64::midpoint(from_ppm, to_ppm))
}

fn max_abs_intensity(range: Option<&DetectedRange>, multiplets: &[DetectedMultiplet]) -> f64 {
    let range_max = range.map_or(0.0, |range| range.max_abs_intensity);
    multiplets
        .iter()
        .flat_map(|multiplet| multiplet.peaks.iter())
        .map(|peak| peak.intensity.abs())
        .fold(range_max, f64::max)
}

fn collect_assignments(
    range: Option<&DetectedRange>,
    multiplets: &[DetectedMultiplet],
    assignments: &AssignmentSet,
) -> Vec<Assignment> {
    let mut seen = BTreeSet::new();
    let mut collected = Vec::new();
    if let Some(range) = range {
        let target = AssignmentTarget::Range1D {
            start_index: range.start_index,
            end_index: range.end_index,
            from: range.from,
            to: range.to,
        };
        push_assignments_for_target(assignments, &target, &mut seen, &mut collected);
    }
    for peak in multiplets
        .iter()
        .flat_map(|multiplet| multiplet.peaks.iter())
    {
        let target = AssignmentTarget::Peak1D {
            index: peak.index,
            x: peak.x,
        };
        push_assignments_for_target(assignments, &target, &mut seen, &mut collected);
    }
    collected
}

fn push_assignments_for_target(
    assignments: &AssignmentSet,
    target: &AssignmentTarget,
    seen: &mut BTreeSet<String>,
    collected: &mut Vec<Assignment>,
) {
    for assignment in assignments.for_target(target) {
        if seen.insert(assignment.id.clone()) {
            collected.push(assignment.clone());
        }
    }
}

fn collect_atoms(assignments: &[Assignment]) -> Vec<AssignedAtom> {
    let mut seen = BTreeSet::new();
    let mut atoms = Vec::new();
    for atom in assignments
        .iter()
        .flat_map(|assignment| assignment.atoms.iter())
    {
        if seen.insert(atom.id.clone()) {
            atoms.push(atom.clone());
        }
    }
    atoms
}

fn collect_couplings(atoms: &[AssignedAtom], graph: &JCouplingGraph) -> Vec<JCoupling> {
    let atom_ids = atoms
        .iter()
        .map(|atom| atom.id.as_str())
        .collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    let mut couplings = Vec::new();
    for coupling in &graph.couplings {
        if (atom_ids.contains(coupling.node_a.as_str())
            || atom_ids.contains(coupling.node_b.as_str()))
            && seen.insert(coupling.id.clone())
        {
            couplings.push(coupling.clone());
        }
    }
    couplings
}

fn range_signal_id(range: &DetectedRange) -> String {
    format!("signal1d:range:{}-{}", range.start_index, range.end_index)
}

fn orphan_signal_id(multiplet: &DetectedMultiplet) -> String {
    format!("signal1d:{}", multiplet.id)
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(RSpinError::InvalidSpectrum {
            message: format!("{field} must not be empty"),
        });
    }
    Ok(())
}

fn require_finite(field: &'static str, value: f64) -> Result<()> {
    if !value.is_finite() {
        return Err(RSpinError::NonFinite { field });
    }
    Ok(())
}

#[cfg(test)]
mod tests;
