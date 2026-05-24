//! Bond-based two-dimensional correlation rules.

use rspin_core::{Atom, Molecule, Nucleus, RSpinError, Result};
use serde::{Deserialize, Serialize};

use crate::{Experiment, PredictedCorrelation2D, PredictedSignal1D};

/// Rule mapping bonded atoms with predicted shifts to two-dimensional correlations.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BondCorrelationRule {
    /// Experiment label for generated correlations.
    pub experiment: Experiment,
    /// X-axis nucleus selected from one endpoint's predicted signals.
    pub x_nucleus: Nucleus,
    /// Y-axis nucleus selected from the bonded endpoint's predicted signals.
    pub y_nucleus: Nucleus,
    /// Relative correlation intensity multiplier.
    pub intensity: f64,
    /// Optional rule confidence in `[0, 1]`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
    /// Include the mirrored x/y correlation for each matched bond.
    #[serde(default)]
    pub include_reverse: bool,
}

impl BondCorrelationRule {
    /// Creates a one-bond correlation rule with unit intensity.
    #[must_use]
    pub fn new(experiment: Experiment, x_nucleus: Nucleus, y_nucleus: Nucleus) -> Self {
        Self {
            experiment,
            x_nucleus,
            y_nucleus,
            intensity: 1.0,
            confidence: None,
            include_reverse: false,
        }
    }

    /// Sets the relative correlation intensity multiplier.
    #[must_use]
    pub fn with_intensity(mut self, intensity: f64) -> Self {
        self.intensity = intensity;
        self
    }

    /// Sets the confidence score.
    #[must_use]
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = Some(confidence);
        self
    }

    /// Clears the confidence score.
    #[must_use]
    pub fn without_confidence(mut self) -> Self {
        self.confidence = None;
        self
    }

    /// Adds a mirrored x/y correlation for each matched bond.
    #[must_use]
    pub fn with_reverse(mut self) -> Self {
        self.include_reverse = true;
        self
    }

    /// Disables mirrored x/y correlations.
    #[must_use]
    pub fn without_reverse(mut self) -> Self {
        self.include_reverse = false;
        self
    }

    /// Validates the rule.
    ///
    /// # Errors
    ///
    /// Returns an error when numeric values are non-finite or confidence is
    /// outside `[0, 1]`.
    pub fn validate(&self) -> Result<()> {
        require_finite("correlation intensity", self.intensity)?;
        validate_confidence(self.confidence)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct AtomSignal {
    pub atom_id: String,
    pub signal: PredictedSignal1D,
}

pub(super) fn correlations_for_molecule(
    molecule: &Molecule,
    atom_signals: &[AtomSignal],
    rules: &[BondCorrelationRule],
) -> Result<Vec<PredictedCorrelation2D>> {
    let mut correlations = Vec::new();
    for bond in &molecule.bonds {
        let from_atom = molecule_atom(molecule, &bond.from_atom_id)?;
        let to_atom = molecule_atom(molecule, &bond.to_atom_id)?;
        for rule in rules {
            append_bond_correlations(&mut correlations, rule, from_atom, to_atom, atom_signals);
        }
    }
    Ok(correlations)
}

fn append_bond_correlations(
    correlations: &mut Vec<PredictedCorrelation2D>,
    rule: &BondCorrelationRule,
    from_atom: &Atom,
    to_atom: &Atom,
    atom_signals: &[AtomSignal],
) {
    append_oriented_correlations(correlations, rule, from_atom, to_atom, atom_signals);
    if rule.x_nucleus != rule.y_nucleus {
        append_oriented_correlations(correlations, rule, to_atom, from_atom, atom_signals);
    }
}

fn append_oriented_correlations(
    correlations: &mut Vec<PredictedCorrelation2D>,
    rule: &BondCorrelationRule,
    x_atom: &Atom,
    y_atom: &Atom,
    atom_signals: &[AtomSignal],
) {
    for x_signal in atom_signals_for_nucleus(atom_signals, &x_atom.id, &rule.x_nucleus) {
        for y_signal in atom_signals_for_nucleus(atom_signals, &y_atom.id, &rule.y_nucleus) {
            correlations.push(correlation_from_signals(rule, x_signal, y_signal, false));
            if rule.include_reverse {
                correlations.push(correlation_from_signals(rule, x_signal, y_signal, true));
            }
        }
    }
}

fn atom_signals_for_nucleus<'a>(
    atom_signals: &'a [AtomSignal],
    atom_id: &str,
    nucleus: &Nucleus,
) -> impl Iterator<Item = &'a AtomSignal> {
    atom_signals
        .iter()
        .filter(move |signal| signal.atom_id == atom_id && signal.signal.nucleus == *nucleus)
}

fn correlation_from_signals(
    rule: &BondCorrelationRule,
    x_signal: &AtomSignal,
    y_signal: &AtomSignal,
    reverse: bool,
) -> PredictedCorrelation2D {
    let (x_nucleus, y_nucleus, x_delta_ppm, y_delta_ppm, x_assignment, y_assignment) = if reverse {
        (
            rule.y_nucleus.clone(),
            rule.x_nucleus.clone(),
            y_signal.signal.delta_ppm,
            x_signal.signal.delta_ppm,
            signal_assignment(y_signal),
            signal_assignment(x_signal),
        )
    } else {
        (
            rule.x_nucleus.clone(),
            rule.y_nucleus.clone(),
            x_signal.signal.delta_ppm,
            y_signal.signal.delta_ppm,
            signal_assignment(x_signal),
            signal_assignment(y_signal),
        )
    };
    let mut correlation = PredictedCorrelation2D::new(
        rule.experiment.clone(),
        x_nucleus,
        y_nucleus,
        x_delta_ppm,
        y_delta_ppm,
    )
    .with_intensity(rule.intensity * x_signal.signal.intensity * y_signal.signal.intensity)
    .with_assignments([format!("{x_assignment}-{y_assignment}")]);

    if let Some(confidence) = combined_confidence(
        rule.confidence,
        x_signal.signal.confidence,
        y_signal.signal.confidence,
    ) {
        correlation = correlation.with_confidence(confidence);
    }
    correlation
}

fn signal_assignment(signal: &AtomSignal) -> String {
    match signal.signal.assignments.first() {
        Some(assignment) => assignment.clone(),
        None => signal.atom_id.clone(),
    }
}

fn combined_confidence(
    rule_confidence: Option<f64>,
    x_confidence: Option<f64>,
    y_confidence: Option<f64>,
) -> Option<f64> {
    let mut combined = None;
    for confidence in [rule_confidence, x_confidence, y_confidence]
        .into_iter()
        .flatten()
    {
        combined = Some(match combined {
            Some(current) if current <= confidence => current,
            Some(_) | None => confidence,
        });
    }
    combined
}

fn molecule_atom<'a>(molecule: &'a Molecule, atom_id: &str) -> Result<&'a Atom> {
    match molecule.atom(atom_id) {
        Some(atom) => Ok(atom),
        None => Err(RSpinError::InvalidMetadata {
            message: format!("bond references unknown atom {atom_id}"),
        }),
    }
}

fn require_finite(field: &'static str, value: f64) -> Result<()> {
    if value.is_finite() {
        return Ok(());
    }
    Err(RSpinError::NonFinite { field })
}

fn validate_confidence(confidence: Option<f64>) -> Result<()> {
    if let Some(value) = confidence {
        require_finite("confidence", value)?;
        if !(0.0..=1.0).contains(&value) {
            return Err(RSpinError::InvalidSpectrum {
                message: "confidence must be between 0 and 1".to_owned(),
            });
        }
    }
    Ok(())
}
