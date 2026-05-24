use rspin_prediction::PredictionSet;

use super::super::{
    from_json, predict_formula_with_element_rules_json, predict_molecule_with_element_rules_json,
    render_prediction_1d_json, render_prediction_2d_json, spectrum1d_from_json,
    spectrum2d_from_json, validate_prediction_json,
};

#[test]
fn validates_prediction_json() -> anyhow::Result<()> {
    let json = validate_prediction_json(
        r#"{"name":"demo","signals_1d":[{"experiment":"Proton1D","nucleus":"Hydrogen1","delta_ppm":1.0,"intensity":1.0,"confidence":0.9,"assignments":[]}],"correlations_2d":[],"provenance":null}"#,
    )?;
    let prediction: PredictionSet = from_json(&json)?;
    assert_eq!(prediction.signals_1d.len(), 1);
    Ok(())
}

#[test]
fn predicts_molecule_with_element_rules_json() -> anyhow::Result<()> {
    let json = predict_molecule_with_element_rules_json(
        r#"{"id":"ethanol","name":"ethanol","atoms":[{"id":"H1","element":"H","label":"H-a"},{"id":"C1","element":"C"},{"id":"O1","element":"O"}],"bonds":[{"from_atom_id":"C1","to_atom_id":"H1","order":"single"}]}"#,
        r#"{"rules":[{"element":"H","experiment":"Proton1D","nucleus":"Hydrogen1","delta_ppm":1.25,"intensity":1.0,"confidence":0.8},{"element":"C","experiment":"Carbon13_1D","nucleus":"Carbon13","delta_ppm":63.0,"intensity":1.0}],"correlation_rules":[{"experiment":"Hsqc","x_nucleus":"Hydrogen1","y_nucleus":"Carbon13","intensity":0.5,"confidence":0.7}]}"#,
    )?;
    let prediction: PredictionSet = from_json(&json)?;

    assert_eq!(prediction.name, Some("ethanol".to_owned()));
    assert_eq!(prediction.signals_1d.len(), 2);
    assert_eq!(prediction.correlations_2d.len(), 1);
    assert_eq!(prediction.signals_1d[0].assignments, vec!["H-a".to_owned()]);
    assert_eq!(prediction.signals_1d[1].assignments, vec!["C1".to_owned()]);
    assert_eq!(
        prediction.correlations_2d[0].assignments,
        vec!["H-a-C1".to_owned()]
    );
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
fn predicts_formula_with_element_rules_json() -> anyhow::Result<()> {
    let json = predict_formula_with_element_rules_json(
        "ethanol",
        "C2H6O",
        r#"{"rules":[{"element":"H","experiment":"Proton1D","nucleus":"Hydrogen1","delta_ppm":1.1,"intensity":1.0},{"element":"C","experiment":"Carbon13_1D","nucleus":"Carbon13","delta_ppm":30.0,"intensity":1.0}]}"#,
    )?;
    let prediction: PredictionSet = from_json(&json)?;

    assert_eq!(prediction.name, Some("ethanol".to_owned()));
    assert_eq!(prediction.signals_1d.len(), 8);
    assert_eq!(prediction.signals_1d[0].assignments, vec!["C1".to_owned()]);
    assert_eq!(prediction.signals_1d[7].assignments, vec!["H6".to_owned()]);
    Ok(())
}

#[test]
fn renders_prediction_json() -> anyhow::Result<()> {
    let spectrum_json = render_prediction_1d_json(
        r#"{"name":"demo","signals_1d":[{"experiment":"Proton1D","nucleus":"Hydrogen1","delta_ppm":1.0,"intensity":1.0,"confidence":0.9,"assignments":["H1"]}],"correlations_2d":[],"provenance":{"source":"fixture","version":null}}"#,
        r#"{"experiment":"Proton1D","nucleus":"Hydrogen1","from_ppm":0.99,"to_ppm":1.01,"points":3,"spectrometer_mhz":400.0,"line_width_hz":1.0,"line_shape":"PseudoVoigt","area_scale":1.0}"#,
    )?;
    let spectrum = spectrum1d_from_json(&spectrum_json)?;

    assert_eq!(spectrum.len(), 3);
    assert_eq!(spectrum.metadata.frequency_mhz, Some(400.0));
    assert!(spectrum.intensities[1] > spectrum.intensities[0]);
    Ok(())
}

#[test]
fn renders_prediction_2d_json() -> anyhow::Result<()> {
    let spectrum_json = render_prediction_2d_json(
        r#"{"name":"demo","signals_1d":[],"correlations_2d":[{"experiment":"Hsqc","x_nucleus":"Hydrogen1","y_nucleus":"Carbon13","x_ppm":1.0,"y_ppm":20.0,"intensity":1.0,"confidence":0.9,"assignments":["H1-C1"]}],"provenance":{"source":"fixture","version":null}}"#,
        r#"{"experiment":"Hsqc","x_nucleus":"Hydrogen1","y_nucleus":"Carbon13","x_from_ppm":0.99,"x_to_ppm":1.01,"x_points":3,"y_from_ppm":19.9,"y_to_ppm":20.1,"y_points":3,"x_spectrometer_mhz":400.0,"y_spectrometer_mhz":100.0,"x_line_width_hz":1.0,"y_line_width_hz":4.0,"line_shape":"PseudoVoigt","volume_scale":1.0}"#,
    )?;
    let spectrum = spectrum2d_from_json(&spectrum_json)?;

    assert_eq!(spectrum.shape(), (3, 3));
    assert_eq!(spectrum.metadata.origin, Some("fixture".to_owned()));
    let Some(center) = spectrum.value_at(1, 1) else {
        panic!("center point should exist");
    };
    let Some(edge) = spectrum.value_at(0, 0) else {
        panic!("edge point should exist");
    };
    assert!(center > edge);
    Ok(())
}
