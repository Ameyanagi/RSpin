use rspin_core::{Nucleus, RSpinError, Unit};

use super::super::{
    parse_agilent_arrayed_fid_1d_bytes_json, parse_agilent_arrayed_fid_2d_bytes_json,
    parse_agilent_fid_1d_bytes_json, parse_agilent_fid_2d_bytes_json,
    parse_agilent_processed_1d_bytes_json, parse_agilent_processed_2d_bytes_json,
    spectrum1d_from_json, spectrum2d_from_json,
};

const BLOCK_HEADER_LEN: usize = 28;
const STATUS_FLOAT: i16 = 0x0008;
const STATUS_COMPLEX: i16 = 0x0010;

#[test]
fn parses_agilent_fid_bytes_to_spectrum_json() -> anyhow::Result<()> {
    let one_d_json = parse_agilent_fid_1d_bytes_json(
        "\
tn 2 2 4 0 0 2 1 8 1 64
1 \"H1\"
0
sw 1 1 5 5 5 2 1 8203 1 64
1 250
0
",
        &agilent_fid_bytes(
            EndianForTest::Little,
            DataForTest::F32(&[0.5, -0.25, 1.5, -2.5]),
            1,
            1,
        )?,
    )?;
    let one_d = spectrum1d_from_json(&one_d_json)?;

    assert_eq!(one_d.x.unit, Unit::Seconds);
    assert_eq!(one_d.x.values, vec![0.0, 0.004]);
    assert_eq!(one_d.intensities, vec![0.5, 1.5]);
    assert_eq!(one_d.imaginary, Some(vec![-0.25, -2.5]));
    assert_eq!(one_d.metadata.nucleus, Some(Nucleus::Hydrogen1));

    let two_d_json = parse_agilent_fid_2d_bytes_json(
        "\
acqdim 7 1 32767 0 0 2 1 0 1 64
1 2
0
sw 1 1 5 5 5 2 1 8203 1 64
1 1000
0
sw1 1 1 5000000 1 -1.25e-08 2 1 0 1 64
1 200
0
",
        &agilent_fid_bytes(
            EndianForTest::Big,
            DataForTest::I16(&[1, -1, 2, -2, 3, -3, 4, -4]),
            2,
            1,
        )?,
    )?;
    let two_d = spectrum2d_from_json(&two_d_json)?;

    assert_eq!(two_d.shape(), (2, 2));
    assert_eq!(two_d.x.unit, Unit::Seconds);
    assert_eq!(two_d.x.values, vec![0.0, 0.001]);
    assert_eq!(two_d.y.unit, Unit::Seconds);
    assert_eq!(two_d.y.values, vec![0.0, 0.005]);
    assert_eq!(two_d.z, vec![1.0, 2.0, 3.0, 4.0]);
    assert_eq!(two_d.imaginary, Some(vec![-1.0, -2.0, -3.0, -4.0]));
    Ok(())
}

#[test]
fn parses_agilent_arrayed_fid_bytes_to_bundle_json() -> anyhow::Result<()> {
    let one_d_json = parse_agilent_arrayed_fid_1d_bytes_json(
        "\
acqdim 7 1 32767 0 0 2 1 0 1 64
1 1
0
array 2 2 256 0 0 2 1 1 1 64
1 \"delay\"
0
sw 1 1 5 5 5 2 1 8203 1 64
1 500
0
",
        &agilent_fid_bytes(
            EndianForTest::Little,
            DataForTest::F32(&[0.5, -0.25, 1.5, -2.5, 3.0, 4.0, 5.0, 6.0]),
            2,
            1,
        )?,
    )?;
    let one_d_bundle = rspin_io::read_spectrum_bundle_json(&one_d_json)?;
    let one_d_spectra = one_d_bundle.spectra_1d().collect::<Vec<_>>();
    let first_one_d = one_d_spectra
        .first()
        .ok_or_else(|| anyhow::anyhow!("missing first arrayed 1D spectrum"))?;

    assert_eq!(one_d_bundle.len(), 2);
    assert_eq!(one_d_bundle.spectra_2d().count(), 0);
    assert!(one_d_bundle.warnings().is_empty());
    assert!(
        one_d_bundle
            .spectra()
            .iter()
            .all(|entry| entry.source().format.as_str() == "agilent_fid")
    );
    assert_eq!(first_one_d.x.unit, Unit::Seconds);
    assert_eq!(first_one_d.intensities, vec![0.5, 1.5]);
    assert_eq!(first_one_d.imaginary, Some(vec![-0.25, -2.5]));
    assert_eq!(
        first_one_d.metadata.property("agilent.array.index"),
        Some("0")
    );

    let two_d_json = parse_agilent_arrayed_fid_2d_bytes_json(
        "\
acqdim 7 1 32767 0 0 2 1 0 1 64
1 2
0
array 2 2 256 0 0 2 1 1 1 64
1 \"mix\"
0
ni 7 1 32767 0 0 2 1 0 1 64
1 2
0
sw 1 1 5 5 5 2 1 8203 1 64
1 1000
0
sw1 1 1 5000000 1 -1.25e-08 2 1 0 1 64
1 200
0
",
        &agilent_fid_bytes(
            EndianForTest::Little,
            DataForTest::F32(&[
                0.5, -0.5, 1.5, -1.5, 2.5, -2.5, 3.5, -3.5, 4.5, -4.5, 5.5, -5.5, 6.5, -6.5, 7.5,
                -7.5,
            ]),
            4,
            1,
        )?,
    )?;
    let two_d_bundle = rspin_io::read_spectrum_bundle_json(&two_d_json)?;
    let two_d_spectra = two_d_bundle.spectra_2d().collect::<Vec<_>>();
    let first_two_d = two_d_spectra
        .first()
        .ok_or_else(|| anyhow::anyhow!("missing first arrayed 2D spectrum"))?;

    assert_eq!(two_d_bundle.len(), 2);
    assert_eq!(two_d_bundle.spectra_1d().count(), 0);
    assert!(two_d_bundle.warnings().is_empty());
    assert!(
        two_d_bundle
            .spectra()
            .iter()
            .all(|entry| entry.source().format.as_str() == "agilent_fid")
    );
    assert_eq!(first_two_d.shape(), (2, 2));
    assert_eq!(first_two_d.x.unit, Unit::Seconds);
    assert_eq!(first_two_d.y.unit, Unit::Points);
    assert_eq!(first_two_d.z, vec![0.5, 1.5, 2.5, 3.5]);
    assert_eq!(first_two_d.imaginary, Some(vec![-0.5, -1.5, -2.5, -3.5]));
    assert_eq!(
        first_two_d
            .metadata
            .property("agilent.array.traces_per_spectrum"),
        Some("2")
    );
    Ok(())
}

#[test]
fn parses_agilent_processed_bytes_to_spectrum_json() -> anyhow::Result<()> {
    let one_d_json = parse_agilent_processed_1d_bytes_json(
        "\
sw 1 1 5 5 5 2 1 8203 1 64
1 1000
0
rfl 1 1 1000000000 -1000000000 0 2 1 11 1 64
1 750
0
rfp 1 1 1000000000 -1000000000 0 2 1 11 1 64
1 250
0
sfrq 1 1 1000000000 0 0 2 1 11 1 64
1 500
0
",
        &agilent_phasefile_bytes(EndianForTest::Big, DataForTest::I32(&[10, 20, -5]), 1, 1)?,
    )?;
    let one_d = spectrum1d_from_json(&one_d_json)?;

    assert_eq!(one_d.x.unit, Unit::Ppm);
    assert_eq!(one_d.x.values, vec![1.0, 0.0, -1.0]);
    assert_eq!(one_d.intensities, vec![10.0, 20.0, -5.0]);

    let two_d_json = parse_agilent_processed_2d_bytes_json(
        "\
acqdim 7 1 32767 0 0 2 1 0 1 64
1 2
0
sw 1 1 5 5 5 2 1 8203 1 64
1 1000
0
sw1 1 1 5000000 1 -1.25e-08 2 1 0 1 64
1 200
0
",
        &agilent_phasefile_bytes(
            EndianForTest::Little,
            DataForTest::F32(&[1.0, 2.0, 3.0, 4.0]),
            2,
            1,
        )?,
    )?;
    let two_d = spectrum2d_from_json(&two_d_json)?;

    assert_eq!(two_d.shape(), (2, 2));
    assert_eq!(two_d.x.unit, Unit::Hertz);
    assert_eq!(two_d.x.values, vec![500.0, -500.0]);
    assert_eq!(two_d.y.unit, Unit::Hertz);
    assert_eq!(two_d.y.values, vec![100.0, -100.0]);
    assert_eq!(two_d.z, vec![1.0, 2.0, 3.0, 4.0]);
    Ok(())
}

#[test]
fn rejects_invalid_agilent_bytes_json() {
    let error = parse_agilent_fid_1d_bytes_json("", b"not fid")
        .expect_err("invalid Agilent FID bytes should fail");

    assert!(matches!(error, RSpinError::Parse { .. }));
}

#[derive(Clone, Copy)]
enum DataForTest<'a> {
    I16(&'a [i16]),
    I32(&'a [i32]),
    F32(&'a [f32]),
}

#[derive(Clone, Copy)]
enum EndianForTest {
    Big,
    Little,
}

fn agilent_fid_bytes(
    endian: EndianForTest,
    data: DataForTest<'_>,
    nblocks: i32,
    ntraces: i32,
) -> anyhow::Result<Vec<u8>> {
    let (ebytes, status, data_bytes) = encode_complex_data(endian, data);
    agilent_binary_file_bytes(endian, &data_bytes, ebytes, status, nblocks, ntraces)
}

fn agilent_phasefile_bytes(
    endian: EndianForTest,
    data: DataForTest<'_>,
    nblocks: i32,
    ntraces: i32,
) -> anyhow::Result<Vec<u8>> {
    let (ebytes, status, data_bytes) = encode_real_data(endian, data);
    agilent_binary_file_bytes(endian, &data_bytes, ebytes, status, nblocks, ntraces)
}

fn agilent_binary_file_bytes(
    endian: EndianForTest,
    data_bytes: &[u8],
    ebytes: i32,
    status: i16,
    nblocks: i32,
    ntraces: i32,
) -> anyhow::Result<Vec<u8>> {
    let nblocks_usize = usize::try_from(nblocks)?;
    let ntraces_usize = usize::try_from(ntraces)?;
    let ebytes_usize = usize::try_from(ebytes)?;
    let row_count = nblocks_usize
        .checked_mul(ntraces_usize)
        .ok_or_else(|| anyhow::anyhow!("synthetic Agilent row count overflow"))?;
    if row_count == 0 || ebytes_usize == 0 {
        anyhow::bail!("synthetic Agilent trace layout must be non-empty");
    }
    if data_bytes.len() % ebytes_usize != 0 {
        anyhow::bail!("synthetic Agilent data length must match element width");
    }
    let value_count = data_bytes.len() / ebytes_usize;
    if value_count % row_count != 0 {
        anyhow::bail!("synthetic Agilent value count must divide evenly into traces");
    }
    let np_values = i32::try_from(value_count / row_count)?;
    let tbytes = np_values
        .checked_mul(ebytes)
        .ok_or_else(|| anyhow::anyhow!("synthetic Agilent trace byte count overflow"))?;
    let trace_bytes = usize::try_from(tbytes)?;
    let block_data_len = ntraces_usize
        .checked_mul(trace_bytes)
        .ok_or_else(|| anyhow::anyhow!("synthetic Agilent block data length overflow"))?;
    let block_byte_count = i32::try_from(
        BLOCK_HEADER_LEN
            .checked_add(block_data_len)
            .ok_or_else(|| anyhow::anyhow!("synthetic Agilent block byte count overflow"))?,
    )?;

    let mut bytes = Vec::new();
    push_i32(&mut bytes, endian, nblocks);
    push_i32(&mut bytes, endian, ntraces);
    push_i32(&mut bytes, endian, np_values);
    push_i32(&mut bytes, endian, ebytes);
    push_i32(&mut bytes, endian, tbytes);
    push_i32(&mut bytes, endian, block_byte_count);
    push_i16(&mut bytes, endian, 0);
    push_i16(&mut bytes, endian, status);
    push_i32(&mut bytes, endian, 1);

    for block_index in 0..nblocks_usize {
        push_i16(&mut bytes, endian, 0);
        push_i16(&mut bytes, endian, status);
        push_i16(&mut bytes, endian, 1);
        push_i16(&mut bytes, endian, 0);
        push_i32(&mut bytes, endian, i32::try_from(block_index + 1)?);
        push_f32(&mut bytes, endian, 0.0);
        push_f32(&mut bytes, endian, 0.0);
        push_f32(&mut bytes, endian, 0.0);
        push_f32(&mut bytes, endian, 0.0);

        let data_start = block_index
            .checked_mul(ntraces_usize)
            .and_then(|index| index.checked_mul(trace_bytes))
            .ok_or_else(|| anyhow::anyhow!("synthetic Agilent block data offset overflow"))?;
        let data_end = data_start
            .checked_add(block_data_len)
            .ok_or_else(|| anyhow::anyhow!("synthetic Agilent block data end overflow"))?;
        let Some(block_data) = data_bytes.get(data_start..data_end) else {
            anyhow::bail!("synthetic Agilent block data outside payload");
        };
        bytes.extend_from_slice(block_data);
    }

    Ok(bytes)
}

fn encode_complex_data(endian: EndianForTest, data: DataForTest<'_>) -> (i32, i16, Vec<u8>) {
    let (ebytes, status, bytes) = encode_real_data(endian, data);
    (ebytes, status | STATUS_COMPLEX, bytes)
}

fn encode_real_data(endian: EndianForTest, data: DataForTest<'_>) -> (i32, i16, Vec<u8>) {
    match data {
        DataForTest::I16(values) => {
            let mut bytes = Vec::with_capacity(values.len() * 2);
            for value in values {
                push_i16(&mut bytes, endian, *value);
            }
            (2, 0x0001, bytes)
        }
        DataForTest::I32(values) => {
            let mut bytes = Vec::with_capacity(values.len() * 4);
            for value in values {
                push_i32(&mut bytes, endian, *value);
            }
            (4, 0x0001 | 0x0004, bytes)
        }
        DataForTest::F32(values) => {
            let mut bytes = Vec::with_capacity(values.len() * 4);
            for value in values {
                push_f32(&mut bytes, endian, *value);
            }
            (4, 0x0001 | STATUS_FLOAT, bytes)
        }
    }
}

fn push_i16(bytes: &mut Vec<u8>, endian: EndianForTest, value: i16) {
    match endian {
        EndianForTest::Big => bytes.extend(value.to_be_bytes()),
        EndianForTest::Little => bytes.extend(value.to_le_bytes()),
    }
}

fn push_i32(bytes: &mut Vec<u8>, endian: EndianForTest, value: i32) {
    match endian {
        EndianForTest::Big => bytes.extend(value.to_be_bytes()),
        EndianForTest::Little => bytes.extend(value.to_le_bytes()),
    }
}

fn push_f32(bytes: &mut Vec<u8>, endian: EndianForTest, value: f32) {
    match endian {
        EndianForTest::Big => bytes.extend(value.to_be_bytes()),
        EndianForTest::Little => bytes.extend(value.to_le_bytes()),
    }
}
