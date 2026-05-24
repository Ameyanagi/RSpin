use rspin_core::{Nucleus, RSpinError, Unit};

use super::super::{
    parse_spectrum_1d_bytes_as_json, parse_spectrum_2d_bytes_as_json, spectrum1d_from_json,
    spectrum2d_from_json,
};

#[test]
fn parses_explicit_one_dimensional_bytes_to_json() -> anyhow::Result<()> {
    let json = parse_spectrum_1d_bytes_as_json(
        &i32_bytes(&[2, -4, 6], ByteOrder::Little),
        "bruker 1r",
        Some(
            "\
##$SI= 3
##$BYTORDP= 0
##$DTYPP= 0
##$NC_proc= -1
##$OFFSET= 10
##$SW_p= 3000
##$SF= 500
##$AXNUC= <1H>
",
        ),
    )?;
    let spectrum = spectrum1d_from_json(&json)?;

    assert_eq!(spectrum.x.unit, Unit::Ppm);
    assert_eq!(spectrum.x.values, vec![10.0, 7.0, 4.0]);
    assert_eq!(spectrum.intensities, vec![4.0, -8.0, 12.0]);
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(spectrum.metadata.frequency_mhz, Some(500.0));
    Ok(())
}

#[test]
fn parses_explicit_two_dimensional_bytes_to_json() -> anyhow::Result<()> {
    let json = parse_spectrum_2d_bytes_as_json(
        &raw_ser_bytes(&[vec![1, 2, 3, 4], vec![5, 6, 7, 8]], ByteOrder::Big),
        "ser",
        Some(
            "\
##$TD= 4
##$BYTORDA= 1
##$DTYPA= 0
##$NC= -1
##$SW_h= 1000
##$NUC1= <1H>
##$SFO1= 400.25
##$PULPROG= <hsqc>
",
        ),
        Some(
            "\
##$TD= 2
##$SW_h= 200
",
        ),
    )?;
    let spectrum = spectrum2d_from_json(&json)?;

    assert_eq!(spectrum.shape(), (2, 2));
    assert_eq!(spectrum.x.unit, Unit::Seconds);
    assert_eq!(spectrum.y.unit, Unit::Seconds);
    assert_eq!(spectrum.x.values, vec![0.0, 0.001]);
    assert_eq!(spectrum.y.values, vec![0.0, 0.005]);
    assert_eq!(spectrum.z, vec![2.0, 6.0, 10.0, 14.0]);
    assert_eq!(spectrum.imaginary, Some(vec![4.0, 8.0, 12.0, 16.0]));
    assert_eq!(spectrum.metadata.name.as_deref(), Some("hsqc"));
    Ok(())
}

#[test]
fn rejects_explicit_bytes_without_required_parameters() {
    let error = parse_spectrum_2d_bytes_as_json(b"not ser", "bruker_ser", Some("##$TD= 4\n"), None)
        .expect_err("missing indirect byte parameters should fail");

    assert!(matches!(error, RSpinError::Parse { .. }));
    assert!(error.to_string().contains("acqu2s"));
}

#[derive(Clone, Copy)]
enum ByteOrder {
    Little,
    Big,
}

fn raw_ser_bytes(rows: &[Vec<i32>], byte_order: ByteOrder) -> Vec<u8> {
    let mut bytes = Vec::new();
    for row in rows {
        for value in row {
            bytes.extend_from_slice(&match byte_order {
                ByteOrder::Little => value.to_le_bytes(),
                ByteOrder::Big => value.to_be_bytes(),
            });
        }
        let padded_words = 256usize.saturating_sub(row.len());
        bytes.extend(std::iter::repeat_n(0, padded_words * 4));
    }
    bytes
}

fn i32_bytes(values: &[i32], byte_order: ByteOrder) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| match byte_order {
            ByteOrder::Little => value.to_le_bytes(),
            ByteOrder::Big => value.to_be_bytes(),
        })
        .collect::<Vec<_>>()
}
