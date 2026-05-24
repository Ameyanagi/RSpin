use rspin_core::{Nucleus, RSpinError, Unit};

use super::super::{
    parse_bruker_fid_1d_bytes_json, parse_bruker_processed_1d_bytes_json,
    parse_bruker_processed_2d_bytes_json, parse_bruker_ser_2d_bytes_json, spectrum1d_from_json,
    spectrum2d_from_json,
};

#[test]
fn parses_bruker_1d_bytes_to_spectrum_json() -> anyhow::Result<()> {
    let processed_json = parse_bruker_processed_1d_bytes_json(
        "\
##$SI= 3
##$BYTORDP= 1
##$DTYPP= 0
##$NC_proc= -1
##$OFFSET= 10
##$SW_p= 3000
##$SF= 500
##$AXNUC= <1H>
",
        &i32_bytes(&[2, -4, 6], ByteOrder::Big),
    )?;
    let processed = spectrum1d_from_json(&processed_json)?;

    assert_eq!(processed.x.unit, Unit::Ppm);
    assert_eq!(processed.x.values, vec![10.0, 7.0, 4.0]);
    assert_eq!(processed.intensities, vec![4.0, -8.0, 12.0]);
    assert_eq!(processed.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(processed.metadata.frequency_mhz, Some(500.0));

    let fid_json = parse_bruker_fid_1d_bytes_json(
        "\
##$TD= 4
##$BYTORDA= 1
##$DTYPA= 0
##$NC= -1
##$SW_h= 1000
##$NUC1= <13C>
##$SFO1= 125.5
##$SOLVENT= <CDCl3>
##$TE= 300
##$OWNER= <wasm raw bytes fixture>
##$PULPROG= <zg>
",
        &i32_bytes(&[1, -2, 3, -4], ByteOrder::Big),
    )?;
    let fid = spectrum1d_from_json(&fid_json)?;

    assert_eq!(fid.x.unit, Unit::Seconds);
    assert_eq!(fid.x.values, vec![0.0, 0.001]);
    assert_eq!(fid.intensities, vec![2.0, 6.0]);
    assert_eq!(fid.imaginary, Some(vec![-4.0, -8.0]));
    assert_eq!(fid.metadata.name.as_deref(), Some("zg"));
    assert_eq!(fid.metadata.nucleus, Some(Nucleus::Carbon13));
    assert_eq!(fid.metadata.frequency_mhz, Some(125.5));
    assert_eq!(fid.metadata.solvent.as_deref(), Some("CDCl3"));
    assert_eq!(fid.metadata.temperature_k, Some(300.0));
    assert_eq!(
        fid.metadata.origin.as_deref(),
        Some("wasm raw bytes fixture")
    );
    Ok(())
}

#[test]
fn parses_bruker_2d_bytes_to_spectrum_json() -> anyhow::Result<()> {
    let processed_json = parse_bruker_processed_2d_bytes_json(
        "\
##$SI= 2
##$BYTORDP= 0
##$DTYPP= 0
##$NC_proc= -1
##$OFFSET= 10
##$SW_p= 2000
##$SF= 500
##$AXNUC= <1H>
",
        "\
##$SI= 2
##$OFFSET= 120
##$SW_p= 2000
##$SF= 100
##$AXNUC= <13C>
",
        &i32_bytes(&[1, 2, 3, 4], ByteOrder::Little),
    )?;
    let processed = spectrum2d_from_json(&processed_json)?;

    assert_eq!(processed.shape(), (2, 2));
    assert_eq!(processed.x.unit, Unit::Ppm);
    assert_eq!(processed.x.values, vec![10.0, 6.0]);
    assert_eq!(processed.y.unit, Unit::Ppm);
    assert_eq!(processed.y.values, vec![120.0, 100.0]);
    assert_eq!(processed.z, vec![2.0, 4.0, 6.0, 8.0]);
    assert_eq!(processed.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(processed.metadata.frequency_mhz, Some(500.0));

    let ser_json = parse_bruker_ser_2d_bytes_json(
        "\
##$TD= 4
##$BYTORDA= 1
##$DTYPA= 0
##$NC= -1
##$SW_h= 1000
##$NUC1= <1H>
##$SFO1= 400.25
##$SOLVENT= <D2O>
##$TE= 299
##$OWNER= <wasm ser bytes fixture>
##$PULPROG= <hsqc>
",
        "\
##$TD= 2
##$SW_h= 200
##$FnMODE= 0
",
        &raw_ser_bytes(&[vec![1, 2, 3, 4], vec![5, 6, 7, 8]], ByteOrder::Big),
    )?;
    let ser = spectrum2d_from_json(&ser_json)?;

    assert_eq!(ser.shape(), (2, 2));
    assert_eq!(ser.x.unit, Unit::Seconds);
    assert_eq!(ser.y.unit, Unit::Seconds);
    assert_eq!(ser.x.values, vec![0.0, 0.001]);
    assert_eq!(ser.y.values, vec![0.0, 0.005]);
    assert_eq!(ser.z, vec![2.0, 6.0, 10.0, 14.0]);
    assert_eq!(ser.imaginary, Some(vec![4.0, 8.0, 12.0, 16.0]));
    assert_eq!(ser.metadata.name.as_deref(), Some("hsqc"));
    assert_eq!(ser.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(ser.metadata.frequency_mhz, Some(400.25));
    assert_eq!(ser.metadata.solvent.as_deref(), Some("D2O"));
    assert_eq!(ser.metadata.temperature_k, Some(299.0));
    assert_eq!(
        ser.metadata.origin.as_deref(),
        Some("wasm ser bytes fixture")
    );
    Ok(())
}

#[test]
fn rejects_invalid_bruker_bytes_json() {
    let error = parse_bruker_fid_1d_bytes_json("", b"not fid")
        .expect_err("invalid Bruker FID bytes should fail");

    assert!(matches!(error, RSpinError::Parse { .. }));
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
