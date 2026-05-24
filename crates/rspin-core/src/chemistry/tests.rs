use super::*;

#[test]
fn builds_and_validates_molecule() -> Result<()> {
    let molecule = Molecule::new("ethanol")
        .with_name("ethanol")
        .with_formula("C2H6O")
        .with_atom(Atom::new("C1", "C").with_position_3d(0.0, 0.0, 0.0))
        .with_atom(Atom::new("C2", "C").with_position_3d(1.5, 0.0, 0.0))
        .with_bond(Bond::new("C1", "C2").with_order(BondOrder::Single));

    molecule.validate()?;
    assert_eq!(
        molecule.atom("C2").map(|atom| atom.element.as_str()),
        Some("C")
    );
    Ok(())
}

#[test]
fn expands_simple_molecular_formulas() -> Result<()> {
    let molecule = Molecule::from_formula("ethanol", "C2H5OH")?.with_name("ethanol");

    assert_eq!(molecule.formula.as_deref(), Some("C2H5OH"));
    assert_eq!(molecule.atoms.len(), 9);
    assert_eq!(molecule.atoms[0], Atom::new("C1", "C"));
    assert_eq!(molecule.atoms[1], Atom::new("C2", "C"));
    assert_eq!(molecule.atoms[7], Atom::new("O1", "O"));
    assert_eq!(molecule.atoms[8], Atom::new("H6", "H"));
    molecule.validate()
}

#[test]
fn rejects_invalid_molecular_formulas() {
    for formula in ["", "2H", "H0", "C(OH)2"] {
        assert!(matches!(
            atoms_from_formula(formula),
            Err(RSpinError::InvalidMetadata { .. })
        ));
    }
}

#[test]
fn rejects_invalid_molecule_data() {
    let duplicate_atoms = Molecule::new("m")
        .with_atom(Atom::new("C1", "C"))
        .with_atom(Atom::new("C1", "C"));
    let unknown_bond = Molecule::new("m")
        .with_atom(Atom::new("C1", "C"))
        .with_bond(Bond::new("C1", "C2"));
    let self_bond = Molecule::new("m")
        .with_atom(Atom::new("C1", "C"))
        .with_bond(Bond::new("C1", "C1"));

    assert!(matches!(
        duplicate_atoms.validate(),
        Err(RSpinError::InvalidMetadata { .. })
    ));
    assert!(matches!(
        unknown_bond.validate(),
        Err(RSpinError::InvalidMetadata { .. })
    ));
    assert!(matches!(
        self_bond.validate(),
        Err(RSpinError::InvalidMetadata { .. })
    ));
    assert!(matches!(
        Atom::new("C1", "C")
            .with_position_2d(f64::NAN, 0.0)
            .validate(),
        Err(RSpinError::NonFinite { .. })
    ));
}
