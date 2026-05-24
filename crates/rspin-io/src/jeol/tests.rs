use rspin_core::{Nucleus, RSpinError, Result, Unit};

use super::{JeolJdf1D, JeolJdf2D, read_jeol_jdf_1d_bytes, read_jeol_jdf_2d_bytes};

#[test]
fn reads_synthetic_complex_1d_jdf() -> Result<()> {
    let bytes = synthetic_complex_jdf()?;

    let spectrum = JeolJdf1D.read_bytes(&bytes)?;

    assert_eq!(spectrum.len(), 4);
    assert_eq!(spectrum.x.unit, Unit::Seconds);
    assert_eq!(spectrum.x.values, vec![0.0, 0.25, 0.5, 0.75]);
    assert_eq!(spectrum.intensities, vec![1.0, 2.0, 3.0, 4.0]);
    assert_eq!(spectrum.imaginary, Some(vec![0.1, 0.2, 0.3, 0.4]));
    assert_eq!(spectrum.metadata.name.as_deref(), Some("sample"));
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(spectrum.metadata.solvent.as_deref(), Some("DMSO-D6"));
    assert_eq!(spectrum.metadata.origin.as_deref(), Some("JEOL"));
    assert_close(spectrum.metadata.frequency_mhz, Some(400.0));
    assert_close(spectrum.metadata.temperature_k, Some(298.15));
    Ok(())
}

#[test]
fn reads_synthetic_complex_2d_jdf() -> Result<()> {
    let bytes = synthetic_complex_2d_jdf()?;

    let spectrum = JeolJdf2D.read_bytes(&bytes)?;

    assert_eq!(spectrum.shape(), (3, 2));
    assert_eq!(spectrum.x.unit, Unit::Seconds);
    assert_eq!(spectrum.x.values, vec![0.0, 0.1, 0.2]);
    assert_eq!(spectrum.y.unit, Unit::Seconds);
    assert_eq!(spectrum.y.values, vec![0.0, 0.5]);
    assert_eq!(spectrum.z, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    assert_eq!(spectrum.imaginary, Some(vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6]));
    assert_eq!(spectrum.metadata.name.as_deref(), Some("sample"));
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(spectrum.metadata.origin.as_deref(), Some("JEOL"));
    Ok(())
}

#[test]
fn rejects_unknown_signature() {
    let error = read_jeol_jdf_1d_bytes(b"not jdf").expect_err("invalid signature should fail");

    assert!(matches!(error, RSpinError::Parse { .. }));
}

#[test]
fn rejects_multidimensional_jdf_for_1d_reader() -> Result<()> {
    let mut bytes = synthetic_complex_jdf()?;
    bytes[12] = 2;

    let error = read_jeol_jdf_1d_bytes(&bytes).expect_err("2D JDF should be unsupported");

    assert!(matches!(error, RSpinError::Unsupported { .. }));
    Ok(())
}

#[test]
fn rejects_one_dimensional_jdf_for_2d_reader() -> Result<()> {
    let bytes = synthetic_complex_jdf()?;

    let error = read_jeol_jdf_2d_bytes(&bytes).expect_err("1D JDF should be unsupported");

    assert!(matches!(error, RSpinError::Unsupported { .. }));
    Ok(())
}

#[test]
fn rejects_unsupported_jdf_major_version() -> Result<()> {
    let mut bytes = synthetic_complex_jdf()?;
    bytes[9] = 2;

    let error =
        read_jeol_jdf_1d_bytes(&bytes).expect_err("unsupported JDF major version should fail");

    assert!(matches!(error, RSpinError::Parse { .. }));
    Ok(())
}

fn synthetic_complex_jdf() -> Result<Vec<u8>> {
    let real: [f64; 4] = [1.0, 2.0, 3.0, 4.0];
    let imaginary: [f64; 4] = [0.1, 0.2, 0.3, 0.4];
    let mut bytes = Vec::new();

    bytes.extend_from_slice(b"JEOL.NMR");
    bytes.push(1);
    bytes.push(1);
    push_be_u16(&mut bytes, 2);
    bytes.push(1);
    bytes.push(0x80);
    bytes.push(1);
    bytes.push(25);
    bytes.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
    bytes.extend_from_slice(&[3, 0, 0, 0, 0, 0, 0, 0]);
    push_unit_array(&mut bytes, 28);
    push_padded(&mut bytes, "synthetic", 124, 0);
    bytes.extend_from_slice(&[0; 4]);
    push_be_u32_array(&mut bytes, &[4, 0, 0, 0, 0, 0, 0, 0]);
    push_be_u32_array(&mut bytes, &[0; 8]);
    push_be_u32_array(&mut bytes, &[3, 0, 0, 0, 0, 0, 0, 0]);
    push_be_f64_array(&mut bytes, &[0.0; 8]);
    push_be_f64_array(&mut bytes, &[0.75, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    bytes.extend_from_slice(&[0; 8]);
    push_padded(&mut bytes, "", 16, 0);
    push_padded(&mut bytes, "", 128, 0);
    push_padded(&mut bytes, "", 128, 0);
    push_padded(&mut bytes, "", 128, 0);
    bytes.extend_from_slice(&[0; 8 * 32]);
    bytes.extend_from_slice(&[0; 8 * 8]);
    bytes.extend_from_slice(&[0; 8 * 8]);
    bytes.extend_from_slice(&[0; 8]);
    bytes.extend_from_slice(&[0; 4]);
    bytes.extend_from_slice(&[0; 8]);

    let param_start_pos = bytes.len();
    push_be_u32(&mut bytes, 0);
    let param_length_pos = bytes.len();
    push_be_u32(&mut bytes, 0);
    bytes.extend_from_slice(&[0; 8 * 4]);
    bytes.extend_from_slice(&[0; 8 * 4]);
    let data_start_pos = bytes.len();
    push_be_u32(&mut bytes, 0);

    let param_start = bytes.len();
    let params = parameter_section()?;
    bytes.extend_from_slice(&params);
    let data_start = bytes.len();
    for value in real {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    for value in imaginary {
        bytes.extend_from_slice(&value.to_le_bytes());
    }

    write_be_u32_at(&mut bytes, param_start_pos, usize_to_u32(param_start)?)?;
    write_be_u32_at(&mut bytes, param_length_pos, usize_to_u32(params.len())?)?;
    write_be_u32_at(&mut bytes, data_start_pos, usize_to_u32(data_start)?)?;
    Ok(bytes)
}

fn synthetic_complex_2d_jdf() -> Result<Vec<u8>> {
    let real: [f64; 6] = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    let imaginary: [f64; 6] = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6];
    let mut bytes = Vec::new();

    bytes.extend_from_slice(b"JEOL.NMR");
    bytes.push(1);
    bytes.push(1);
    push_be_u16(&mut bytes, 2);
    bytes.push(2);
    bytes.push(0xC0);
    bytes.push(2);
    bytes.push(25);
    bytes.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
    bytes.extend_from_slice(&[4, 4, 0, 0, 0, 0, 0, 0]);
    push_unit_array_with(&mut bytes, &[28, 28]);
    push_padded(&mut bytes, "synthetic 2d", 124, 0);
    bytes.extend_from_slice(&[0; 4]);
    push_be_u32_array(&mut bytes, &[3, 2, 1, 1, 1, 1, 1, 1]);
    push_be_u32_array(&mut bytes, &[0; 8]);
    push_be_u32_array(&mut bytes, &[2, 1, 0, 0, 0, 0, 0, 0]);
    push_be_f64_array(&mut bytes, &[0.0; 8]);
    push_be_f64_array(&mut bytes, &[0.2, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    bytes.extend_from_slice(&[0; 8]);
    push_padded(&mut bytes, "", 16, 0);
    push_padded(&mut bytes, "", 128, 0);
    push_padded(&mut bytes, "", 128, 0);
    push_padded(&mut bytes, "", 128, 0);
    bytes.extend_from_slice(&[0; 8 * 32]);
    bytes.extend_from_slice(&[0; 8 * 8]);
    bytes.extend_from_slice(&[0; 8 * 8]);
    bytes.extend_from_slice(&[0; 8]);
    bytes.extend_from_slice(&[0; 4]);
    bytes.extend_from_slice(&[0; 8]);

    let param_start_pos = bytes.len();
    push_be_u32(&mut bytes, 0);
    let param_length_pos = bytes.len();
    push_be_u32(&mut bytes, 0);
    bytes.extend_from_slice(&[0; 8 * 4]);
    bytes.extend_from_slice(&[0; 8 * 4]);
    let data_start_pos = bytes.len();
    push_be_u32(&mut bytes, 0);

    let param_start = bytes.len();
    let params = parameter_section()?;
    bytes.extend_from_slice(&params);
    let data_start = bytes.len();
    for value in real {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    for value in imaginary {
        bytes.extend_from_slice(&value.to_le_bytes());
    }

    write_be_u32_at(&mut bytes, param_start_pos, usize_to_u32(param_start)?)?;
    write_be_u32_at(&mut bytes, param_length_pos, usize_to_u32(params.len())?)?;
    write_be_u32_at(&mut bytes, data_start_pos, usize_to_u32(data_start)?)?;
    Ok(bytes)
}

fn parameter_section() -> Result<Vec<u8>> {
    let records = vec![
        string_parameter("x_domain", "1H", 0),
        float_parameter("x_freq", 400_000_000.0, 13),
        string_parameter("solvent", "DMSO-D6", 0),
        float_parameter("temp_get", 25.0, 4),
        string_parameter("sample_id", "sample", 0),
    ];
    let mut bytes = Vec::new();
    push_le_u32(&mut bytes, 64);
    push_le_u32(&mut bytes, 0);
    push_le_u32(&mut bytes, usize_to_u32(records.len().saturating_sub(1))?);
    let total =
        16usize
            .checked_add(records.len().checked_mul(64).ok_or_else(|| {
                RSpinError::InvalidSpectrum {
                    message: "test parameter size overflow".to_owned(),
                }
            })?)
            .ok_or_else(|| RSpinError::InvalidSpectrum {
                message: "test parameter size overflow".to_owned(),
            })?;
    push_le_u32(&mut bytes, usize_to_u32(total)?);
    for record in records {
        bytes.extend_from_slice(&record);
    }
    Ok(bytes)
}

fn string_parameter(name: &str, value: &str, base_unit: u8) -> [u8; 64] {
    let mut record = parameter_record(name, base_unit);
    push_padded_at(&mut record, 16, value, 16);
    record[32..36].copy_from_slice(&0_i32.to_le_bytes());
    record
}

fn float_parameter(name: &str, value: f64, base_unit: u8) -> [u8; 64] {
    let mut record = parameter_record(name, base_unit);
    record[16..24].copy_from_slice(&value.to_le_bytes());
    record[32..36].copy_from_slice(&2_i32.to_le_bytes());
    record
}

fn parameter_record(name: &str, base_unit: u8) -> [u8; 64] {
    let mut record = [0u8; 64];
    record[6] = 0;
    record[7] = base_unit;
    push_padded_at(&mut record, 36, name, 28);
    record
}

fn push_unit_array(bytes: &mut Vec<u8>, first_base_unit: u8) {
    push_unit_array_with(bytes, &[first_base_unit]);
}

fn push_unit_array_with(bytes: &mut Vec<u8>, base_units: &[u8]) {
    bytes.push(0);
    bytes.push(unit_base_at(base_units, 0));
    for index in 1..8 {
        bytes.push(0);
        bytes.push(unit_base_at(base_units, index));
    }
}

fn unit_base_at(base_units: &[u8], index: usize) -> u8 {
    match base_units.get(index) {
        Some(value) => *value,
        None => 0,
    }
}

fn push_padded(bytes: &mut Vec<u8>, value: &str, len: usize, pad: u8) {
    let raw = value.as_bytes();
    for index in 0..len {
        bytes.push(if index < raw.len() { raw[index] } else { pad });
    }
}

fn push_padded_at(bytes: &mut [u8], offset: usize, value: &str, len: usize) {
    let raw = value.as_bytes();
    for index in 0..len {
        bytes[offset + index] = if index < raw.len() { raw[index] } else { b' ' };
    }
}

fn push_be_u16(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

fn push_be_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

fn push_le_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_be_u32_array(bytes: &mut Vec<u8>, values: &[u32; 8]) {
    for value in values {
        push_be_u32(bytes, *value);
    }
}

fn push_be_f64_array(bytes: &mut Vec<u8>, values: &[f64; 8]) {
    for value in values {
        bytes.extend_from_slice(&value.to_be_bytes());
    }
}

fn write_be_u32_at(bytes: &mut [u8], offset: usize, value: u32) -> Result<()> {
    let end = offset
        .checked_add(4)
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "test offset overflow".to_owned(),
        })?;
    let Some(slice) = bytes.get_mut(offset..end) else {
        return Err(RSpinError::InvalidSpectrum {
            message: "test offset outside buffer".to_owned(),
        });
    };
    slice.copy_from_slice(&value.to_be_bytes());
    Ok(())
}

fn usize_to_u32(value: usize) -> Result<u32> {
    u32::try_from(value).map_err(|_| RSpinError::InvalidSpectrum {
        message: "test value too large".to_owned(),
    })
}

fn assert_close(actual: Option<f64>, expected: Option<f64>) {
    match (actual, expected) {
        (Some(left), Some(right)) => assert!((left - right).abs() < 1e-12, "{left} != {right}"),
        (left, right) => assert_eq!(left, right),
    }
}
