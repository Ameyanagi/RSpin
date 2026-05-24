use rspin_core::{Atom, Bond, Molecule, Nucleus, RSpinError};

use super::*;

#[test]
fn predicts_molecule_signals_from_element_rules() -> anyhow::Result<()> {
    let molecule = Molecule::new("ethanol")
        .with_name("ethanol")
        .with_atom(Atom::new("H1", "H").with_label("H-a"))
        .with_atom(Atom::new("C1", "C"))
        .with_atom(Atom::new("O1", "O"))
        .with_bond(Bond::new("C1", "O1"));
    let predictor = ElementShiftPredictor::new()
        .with_rule(
            ElementShiftRule::new("H", Experiment::Proton1D, Nucleus::Hydrogen1, 1.25)
                .with_intensity(1.0)
                .with_confidence(0.75),
        )
        .with_rule(ElementShiftRule::new(
            "C",
            Experiment::Carbon13_1D,
            Nucleus::Carbon13,
            63.0,
        ));

    let prediction = predictor.predict_molecule(&molecule)?;

    assert_eq!(prediction.name, Some("ethanol".to_owned()));
    assert_eq!(prediction.signals_1d.len(), 2);
    assert!((prediction.signals_1d[0].delta_ppm - 1.25).abs() < 1.0e-12);
    assert_eq!(prediction.signals_1d[0].assignments, vec!["H-a".to_owned()]);
    assert!((prediction.signals_1d[1].delta_ppm - 63.0).abs() < 1.0e-12);
    assert_eq!(prediction.signals_1d[1].assignments, vec!["C1".to_owned()]);
    assert!(prediction.correlations_2d.is_empty());
    assert_eq!(
        prediction
            .provenance
            .as_ref()
            .map(|item| item.source.as_str()),
        Some("rspin-element-shift-rules")
    );
    Ok(())
}

#[test]
fn predicts_bond_correlations_from_shift_rules() -> anyhow::Result<()> {
    let molecule = Molecule::new("methanol")
        .with_atom(Atom::new("C1", "C"))
        .with_atom(Atom::new("H1", "H").with_label("H-a"))
        .with_bond(Bond::new("C1", "H1"));
    let predictor = ElementShiftPredictor::new()
        .with_rule(ElementShiftRule::new(
            "C",
            Experiment::Carbon13_1D,
            Nucleus::Carbon13,
            50.0,
        ))
        .with_rule(
            ElementShiftRule::new("H", Experiment::Proton1D, Nucleus::Hydrogen1, 3.2)
                .with_confidence(0.9),
        )
        .with_correlation_rule(
            BondCorrelationRule::new(Experiment::Hsqc, Nucleus::Hydrogen1, Nucleus::Carbon13)
                .with_intensity(0.8)
                .with_confidence(0.75),
        );

    let prediction = predictor.predict_molecule(&molecule)?;

    assert_eq!(prediction.signals_1d.len(), 2);
    assert_eq!(prediction.correlations_2d.len(), 1);
    let correlation = &prediction.correlations_2d[0];
    assert_eq!(correlation.experiment, Experiment::Hsqc);
    assert_eq!(correlation.x_nucleus, Nucleus::Hydrogen1);
    assert_eq!(correlation.y_nucleus, Nucleus::Carbon13);
    assert!((correlation.x_ppm - 3.2).abs() < 1.0e-12);
    assert!((correlation.y_ppm - 50.0).abs() < 1.0e-12);
    assert!((correlation.intensity - 0.8).abs() < 1.0e-12);
    assert_eq!(correlation.confidence, Some(0.75));
    assert_eq!(correlation.assignments, vec!["H-a-C1".to_owned()]);
    Ok(())
}

#[test]
fn can_emit_reverse_homonuclear_bond_correlations() -> anyhow::Result<()> {
    let molecule = Molecule::new("h2")
        .with_atom(Atom::new("H1", "H"))
        .with_atom(Atom::new("H2", "H"))
        .with_bond(Bond::new("H1", "H2"));
    let predictor = ElementShiftPredictor::new()
        .with_rule(ElementShiftRule::new(
            "H",
            Experiment::Proton1D,
            Nucleus::Hydrogen1,
            1.0,
        ))
        .with_correlation_rule(
            BondCorrelationRule::new(Experiment::Cosy, Nucleus::Hydrogen1, Nucleus::Hydrogen1)
                .with_reverse()
                .without_reverse()
                .with_reverse(),
        );

    let prediction = predictor.predict_molecule(&molecule)?;

    assert_eq!(prediction.correlations_2d.len(), 2);
    assert_eq!(
        prediction.correlations_2d[0].assignments,
        vec!["H1-H2".to_owned()]
    );
    assert_eq!(
        prediction.correlations_2d[1].assignments,
        vec!["H2-H1".to_owned()]
    );
    Ok(())
}

#[test]
fn supports_chainable_predictor_options_and_trait_api() -> anyhow::Result<()> {
    let molecule = Molecule::new("labeled")
        .with_atom(Atom::new("C13", "c").with_isotope(13))
        .with_atom(Atom::new("C12", "C").with_isotope(12));
    let predictor = ElementShiftPredictor::new()
        .with_name("custom")
        .with_rule(
            ElementShiftRule::new("C", Experiment::Carbon13_1D, Nucleus::Carbon13, 120.0)
                .with_isotope(13)
                .without_isotope()
                .with_isotope(13)
                .without_confidence(),
        )
        .with_correlation_rule(BondCorrelationRule::new(
            Experiment::Hsqc,
            Nucleus::Hydrogen1,
            Nucleus::Carbon13,
        ))
        .without_correlation_rules()
        .with_correlation_rules(Vec::new())
        .with_provenance(PredictionProvenance::new("test-rules"));

    let prediction = Predictor::predict(&predictor, &molecule)?;

    assert_eq!(prediction.name, Some("custom".to_owned()));
    assert_eq!(prediction.signals_1d.len(), 1);
    assert_eq!(prediction.signals_1d[0].assignments, vec!["C13".to_owned()]);
    assert_eq!(
        prediction
            .provenance
            .as_ref()
            .map(|item| item.source.as_str()),
        Some("test-rules")
    );
    Ok(())
}

#[test]
fn free_function_delegates_to_rule_predictor() -> anyhow::Result<()> {
    let molecule = Molecule::new("one").with_atom(Atom::new("H1", "H"));
    let predictor = ElementShiftPredictor::new().with_rule(ElementShiftRule::new(
        "H",
        Experiment::Proton1D,
        Nucleus::Hydrogen1,
        0.9,
    ));

    let prediction = predict_molecule_with_rules(&molecule, &predictor)?;

    assert_eq!(prediction.signals_1d.len(), 1);
    assert_eq!(prediction.signals_1d[0].assignments, vec!["H1".to_owned()]);
    Ok(())
}

#[test]
fn predicts_formula_signals_from_element_rules() -> anyhow::Result<()> {
    let predictor = ElementShiftPredictor::new()
        .with_rule(ElementShiftRule::new(
            "H",
            Experiment::Proton1D,
            Nucleus::Hydrogen1,
            1.1,
        ))
        .with_rule(ElementShiftRule::new(
            "C",
            Experiment::Carbon13_1D,
            Nucleus::Carbon13,
            30.0,
        ));

    let prediction = predict_formula_with_rules("ethanol", "C2H6O", &predictor)?;

    assert_eq!(prediction.name, Some("ethanol".to_owned()));
    assert_eq!(prediction.signals_1d.len(), 8);
    assert_eq!(prediction.signals_1d[0].assignments, vec!["C1".to_owned()]);
    assert_eq!(prediction.signals_1d[1].assignments, vec!["C2".to_owned()]);
    assert_eq!(prediction.signals_1d[7].assignments, vec!["H6".to_owned()]);
    Ok(())
}

#[test]
fn rejects_invalid_rules_and_molecules() {
    let empty_element = ElementShiftRule::new("", Experiment::Proton1D, Nucleus::Hydrogen1, 1.0);
    let non_finite_shift =
        ElementShiftRule::new("H", Experiment::Proton1D, Nucleus::Hydrogen1, f64::NAN);
    let invalid_confidence =
        ElementShiftRule::new("H", Experiment::Proton1D, Nucleus::Hydrogen1, 1.0)
            .with_confidence(1.1);
    let invalid_correlation_confidence =
        BondCorrelationRule::new(Experiment::Hsqc, Nucleus::Hydrogen1, Nucleus::Carbon13)
            .with_confidence(1.1);
    let non_finite_correlation_intensity =
        BondCorrelationRule::new(Experiment::Hsqc, Nucleus::Hydrogen1, Nucleus::Carbon13)
            .with_intensity(f64::NAN)
            .without_confidence();
    let invalid_molecule = Molecule::new("m")
        .with_atom(Atom::new("H1", "H"))
        .with_bond(Bond::new("H1", "missing"));

    assert!(matches!(
        ElementShiftPredictor::new()
            .with_rule(empty_element)
            .validate(),
        Err(RSpinError::InvalidSpectrum { .. })
    ));
    assert!(matches!(
        ElementShiftPredictor::new()
            .with_rule(non_finite_shift)
            .validate(),
        Err(RSpinError::NonFinite { .. })
    ));
    assert!(matches!(
        ElementShiftPredictor::new()
            .with_rule(invalid_confidence)
            .validate(),
        Err(RSpinError::InvalidSpectrum { .. })
    ));
    assert!(matches!(
        ElementShiftPredictor::new()
            .with_correlation_rule(invalid_correlation_confidence)
            .validate(),
        Err(RSpinError::InvalidSpectrum { .. })
    ));
    assert!(matches!(
        ElementShiftPredictor::new()
            .with_correlation_rule(non_finite_correlation_intensity)
            .validate(),
        Err(RSpinError::NonFinite { .. })
    ));
    assert!(matches!(
        ElementShiftPredictor::new().predict_molecule(&invalid_molecule),
        Err(RSpinError::InvalidMetadata { .. })
    ));
}
