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
