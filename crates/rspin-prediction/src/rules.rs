//! Rule-based pure Rust prediction.

use serde::{Deserialize, Serialize};

use rspin_core::{Atom, Molecule, Nucleus, RSpinError, Result};

use crate::{Experiment, PredictedSignal1D, PredictionProvenance, PredictionSet, Predictor};

mod correlations;

pub use correlations::BondCorrelationRule;
use correlations::{AtomSignal, correlations_for_molecule};

/// Rule mapping molecule atoms to one-dimensional predicted signals.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ElementShiftRule {
    /// Element symbol matched against atoms, such as `H` or `C`.
    pub element: String,
    /// Optional atom isotope selector. When omitted, all isotopes match.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub isotope: Option<u16>,
    /// Experiment label for generated signals.
    pub experiment: Experiment,
    /// Observed nucleus for generated signals.
    pub nucleus: Nucleus,
    /// Predicted chemical shift in ppm.
    pub delta_ppm: f64,
    /// Relative signal intensity assigned to each matching atom.
    pub intensity: f64,
    /// Optional rule confidence in `[0, 1]`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
}

impl ElementShiftRule {
    /// Creates a rule with unit intensity.
    #[must_use]
    pub fn new(
        element: impl Into<String>,
        experiment: Experiment,
        nucleus: Nucleus,
        delta_ppm: f64,
    ) -> Self {
        Self {
            element: element.into(),
            isotope: None,
            experiment,
            nucleus,
            delta_ppm,
            intensity: 1.0,
            confidence: None,
        }
    }

    /// Restricts the rule to atoms with the selected isotope.
    #[must_use]
    pub fn with_isotope(mut self, isotope: u16) -> Self {
        self.isotope = Some(isotope);
        self
    }

    /// Clears the isotope selector.
    #[must_use]
    pub fn without_isotope(mut self) -> Self {
        self.isotope = None;
        self
    }

    /// Sets the relative intensity.
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

    /// Returns true when this rule applies to `atom`.
    #[must_use]
    pub fn matches_atom(&self, atom: &Atom) -> bool {
        if !self.element.eq_ignore_ascii_case(&atom.element) {
            return false;
        }
        match self.isotope {
            Some(isotope) => atom.isotope == Some(isotope),
            None => true,
        }
    }

    /// Validates the rule.
    ///
    /// # Errors
    ///
    /// Returns an error when required labels are empty or numeric values are
    /// non-finite or outside their supported range.
    pub fn validate(&self) -> Result<()> {
        ensure_non_empty("element", &self.element)?;
        if self.isotope == Some(0) {
            return invalid_prediction("isotope selector must be positive");
        }
        require_finite("delta_ppm", self.delta_ppm)?;
        require_finite("intensity", self.intensity)?;
        validate_confidence(self.confidence)
    }

    fn signal_for_atom(&self, atom: &Atom) -> PredictedSignal1D {
        PredictedSignal1D::new(
            self.experiment.clone(),
            self.nucleus.clone(),
            self.delta_ppm,
        )
        .with_intensity(self.intensity)
        .with_assignments([assignment_for_atom(atom)])
        .with_optional_confidence(self.confidence)
    }
}

/// A table-driven predictor for molecule atom lists.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ElementShiftPredictor {
    /// Optional prediction name override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Element rules applied to each atom in stable molecule order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<ElementShiftRule>,
    /// Bond correlation rules applied after atom shifts are predicted.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub correlation_rules: Vec<BondCorrelationRule>,
    /// Provenance attached to generated prediction payloads.
    #[serde(default = "default_provenance")]
    pub provenance: PredictionProvenance,
}

impl ElementShiftPredictor {
    /// Creates an empty rule-based predictor.
    #[must_use]
    pub fn new() -> Self {
        Self {
            name: None,
            rules: Vec::new(),
            correlation_rules: Vec::new(),
            provenance: default_provenance(),
        }
    }

    /// Sets a prediction name override.
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Clears the prediction name override.
    #[must_use]
    pub fn without_name(mut self) -> Self {
        self.name = None;
        self
    }

    /// Appends one element shift rule.
    #[must_use]
    pub fn with_rule(mut self, rule: ElementShiftRule) -> Self {
        self.rules.push(rule);
        self
    }

    /// Replaces all element shift rules.
    #[must_use]
    pub fn with_rules(mut self, rules: Vec<ElementShiftRule>) -> Self {
        self.rules = rules;
        self
    }

    /// Clears all rules.
    #[must_use]
    pub fn without_rules(mut self) -> Self {
        self.rules.clear();
        self
    }

    /// Appends one bond correlation rule.
    #[must_use]
    pub fn with_correlation_rule(mut self, rule: BondCorrelationRule) -> Self {
        self.correlation_rules.push(rule);
        self
    }

    /// Replaces all bond correlation rules.
    #[must_use]
    pub fn with_correlation_rules(mut self, rules: Vec<BondCorrelationRule>) -> Self {
        self.correlation_rules = rules;
        self
    }

    /// Clears all bond correlation rules.
    #[must_use]
    pub fn without_correlation_rules(mut self) -> Self {
        self.correlation_rules.clear();
        self
    }

    /// Sets prediction provenance.
    #[must_use]
    pub fn with_provenance(mut self, provenance: PredictionProvenance) -> Self {
        self.provenance = provenance;
        self
    }

    /// Predicts signals for a molecule using this predictor's rule table.
    ///
    /// # Errors
    ///
    /// Returns an error when the molecule, rule table, or generated prediction
    /// payload is invalid.
    pub fn predict_molecule(&self, molecule: &Molecule) -> Result<PredictionSet> {
        molecule.validate()?;
        self.validate()?;

        let atom_signals = self.atom_signals_for_molecule(molecule);
        let correlations =
            correlations_for_molecule(molecule, &atom_signals, &self.correlation_rules)?;
        let signals = atom_signals
            .into_iter()
            .map(|atom_signal| atom_signal.signal)
            .collect();
        let mut prediction = PredictionSet::new()
            .with_provenance(self.provenance.clone())
            .with_signals_1d(signals)
            .with_correlations_2d(correlations);

        prediction.name = Some(self.prediction_name(molecule));
        prediction.validate()?;
        Ok(prediction)
    }

    /// Predicts signals for a formula-expanded molecule.
    ///
    /// # Errors
    ///
    /// Returns an error when the formula, rule table, or generated prediction
    /// payload is invalid.
    pub fn predict_formula(
        &self,
        molecule_id: impl Into<String>,
        formula: impl Into<String>,
    ) -> Result<PredictionSet> {
        let molecule = Molecule::from_formula(molecule_id, formula)?;
        self.predict_molecule(&molecule)
    }

    /// Validates the rule table.
    ///
    /// # Errors
    ///
    /// Returns an error when any rule is invalid.
    pub fn validate(&self) -> Result<()> {
        for rule in &self.rules {
            rule.validate()?;
        }
        for rule in &self.correlation_rules {
            rule.validate()?;
        }
        Ok(())
    }

    fn atom_signals_for_molecule(&self, molecule: &Molecule) -> Vec<AtomSignal> {
        molecule
            .atoms
            .iter()
            .flat_map(|atom| {
                self.rules
                    .iter()
                    .filter(|rule| rule.matches_atom(atom))
                    .map(|rule| AtomSignal {
                        atom_id: atom.id.clone(),
                        signal: rule.signal_for_atom(atom),
                    })
            })
            .collect()
    }

    fn prediction_name(&self, molecule: &Molecule) -> String {
        match &self.name {
            Some(name) => name.clone(),
            None => match &molecule.name {
                Some(name) => name.clone(),
                None => molecule.id.clone(),
            },
        }
    }
}

impl Default for ElementShiftPredictor {
    fn default() -> Self {
        Self::new()
    }
}

impl Predictor<Molecule> for ElementShiftPredictor {
    fn predict(&self, input: &Molecule) -> Result<PredictionSet> {
        self.predict_molecule(input)
    }
}

/// Predicts molecule signals with an element shift rule table.
///
/// # Errors
///
/// Returns an error when the molecule, rule table, or generated prediction
/// payload is invalid.
pub fn predict_molecule_with_rules(
    molecule: &Molecule,
    predictor: &ElementShiftPredictor,
) -> Result<PredictionSet> {
    predictor.predict_molecule(molecule)
}

/// Predicts formula-expanded molecule signals with an element shift rule table.
///
/// # Errors
///
/// Returns an error when the formula, rule table, or generated prediction
/// payload is invalid.
pub fn predict_formula_with_rules(
    molecule_id: impl Into<String>,
    formula: impl Into<String>,
    predictor: &ElementShiftPredictor,
) -> Result<PredictionSet> {
    predictor.predict_formula(molecule_id, formula)
}

trait OptionalConfidence {
    fn with_optional_confidence(self, confidence: Option<f64>) -> Self;
}

impl OptionalConfidence for PredictedSignal1D {
    fn with_optional_confidence(self, confidence: Option<f64>) -> Self {
        match confidence {
            Some(value) => self.with_confidence(value),
            None => self,
        }
    }
}

fn assignment_for_atom(atom: &Atom) -> String {
    match atom.label.as_ref().filter(|label| !label.trim().is_empty()) {
        Some(label) => label.clone(),
        None => atom.id.clone(),
    }
}

fn default_provenance() -> PredictionProvenance {
    PredictionProvenance::new("rspin-element-shift-rules").with_version(env!("CARGO_PKG_VERSION"))
}

fn ensure_non_empty(field: &'static str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return invalid_prediction(format!("{field} must not be empty"));
    }
    Ok(())
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
            return invalid_prediction("confidence must be between 0 and 1");
        }
    }
    Ok(())
}

fn invalid_prediction(message: impl Into<String>) -> Result<()> {
    Err(RSpinError::InvalidSpectrum {
        message: message.into(),
    })
}

#[cfg(test)]
mod tests;
