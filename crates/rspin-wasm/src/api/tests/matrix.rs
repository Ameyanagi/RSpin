use rspin_core::{Axis, Metadata, Spectrum1D, Spectrum2D, Unit};

use super::super::{
    align_spectra_by_peak_to_matrix_1d_json, from_json, generate_spectrum_matrix_1d_json,
    generate_spectrum_matrix_2d_json, to_json,
};

#[test]
fn generates_spectrum_matrix_1d_json() -> anyhow::Result<()> {
    let spectra_json = to_json(&vec![
        Spectrum1D::new(
            Axis::linear("x", Unit::Ppm, 0.0, 2.0, 3)?,
            vec![1.0, 2.0, 3.0],
            Metadata::named("a"),
        )?,
        Spectrum1D::new(
            Axis::linear("x", Unit::Ppm, 0.0, 2.0, 2)?,
            vec![10.0, 14.0],
            Metadata::named("b b"),
        )?,
    ])?;
    let matrix_json = generate_spectrum_matrix_1d_json(
        &spectra_json,
        r#"{"target_axis":null,"outside_value":0.0}"#,
    )?;
    let matrix: rspin_analysis::SpectrumMatrix1D = from_json(&matrix_json)?;

    assert_eq!(matrix.shape(), (2, 3));
    assert_eq!(matrix.row_ids, vec!["0:a", "1:b_b"]);
    assert_eq!(matrix.values, vec![1.0, 2.0, 3.0, 10.0, 12.0, 14.0]);
    Ok(())
}

#[test]
fn aligns_spectra_by_peak_to_matrix_1d_json() -> anyhow::Result<()> {
    let spectra_json = to_json(&vec![
        Spectrum1D::new(
            Axis::linear("x", Unit::Ppm, 0.0, 2.0, 3)?,
            vec![0.0, 5.0, 0.0],
            Metadata::named("ref"),
        )?,
        Spectrum1D::new(
            Axis::linear("x", Unit::Ppm, 0.5, 2.5, 3)?,
            vec![0.0, 7.0, 0.0],
            Metadata::named("shifted"),
        )?,
    ])?;
    let result_json = align_spectra_by_peak_to_matrix_1d_json(
        &spectra_json,
        "{}",
        r#"{"target_axis":null,"outside_value":0.0}"#,
    )?;
    let result: rspin_analysis::PeakAlignedMatrix1D = from_json(&result_json)?;

    assert_eq!(result.matrix.shape(), (2, 3));
    assert_eq!(result.matrix.values, vec![0.0, 5.0, 0.0, 0.0, 7.0, 0.0]);
    assert!((result.shifts[1].delta + 0.5).abs() < 1.0e-12);
    Ok(())
}

#[test]
fn generates_spectrum_matrix_2d_json() -> anyhow::Result<()> {
    let spectra_json = to_json(&vec![
        Spectrum2D::new(
            Axis::linear("x", Unit::Ppm, 0.0, 2.0, 3)?,
            Axis::linear("y", Unit::Ppm, 0.0, 1.0, 2)?,
            vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
            Metadata::named("a"),
        )?,
        Spectrum2D::new(
            Axis::linear("x", Unit::Ppm, 0.0, 2.0, 2)?,
            Axis::linear("y", Unit::Ppm, 0.0, 1.0, 2)?,
            vec![10.0, 14.0, 20.0, 24.0],
            Metadata::named("b b"),
        )?,
    ])?;
    let matrix_json = generate_spectrum_matrix_2d_json(
        &spectra_json,
        r#"{"target_x_axis":null,"target_y_axis":null,"outside_value":0.0}"#,
    )?;
    let matrix: rspin_analysis::SpectrumMatrix2D = from_json(&matrix_json)?;

    assert_eq!(matrix.shape(), (2, 2, 3));
    assert_eq!(matrix.spectrum_ids, vec!["0:a", "1:b_b"]);
    assert_eq!(
        matrix.values,
        vec![
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 10.0, 12.0, 14.0, 20.0, 22.0, 24.0
        ]
    );
    Ok(())
}
