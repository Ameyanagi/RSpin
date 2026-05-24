use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use rspin_core::{Nucleus, RSpinError, Unit};

use super::*;

#[test]
fn reads_big_endian_i32_complex_fid() -> anyhow::Result<()> {
    let root = synthetic_dataset("big-i32")?;
    write_procpar(
        &root,
        "\
tn 2 2 4 0 0 2 1 8 1 64
1 \"H1\"
0
sfrq 1 1 1000000000 0 0 2 1 11 1 64
1 400.13
0
sw 1 1 5 5 5 2 1 8203 1 64
1 1000
0
solvent 2 2 6 0 0 2 1 11 1 64
1 \"CDCl3\"
0
temp 1 1 200 -150 0.1 2 1 8 1 64
1 25
0
operator 2 2 8 0 0 2 1 0 1 64
1 \"fixture user\"
0
",
    )?;
    write_fid(
        &root,
        EndianForTest::Big,
        DataForTest::I32(&[1, 2, 3, -4, -5, 6]),
        1,
        1,
    )?;

    let spectrum = read_agilent_fid_1d_dir(&root)?;

    assert_eq!(spectrum.x.unit, Unit::Seconds);
    assert_eq!(spectrum.x.values, vec![0.0, 0.001, 0.002]);
    assert_eq!(spectrum.intensities, vec![1.0, 3.0, -5.0]);
    assert_eq!(spectrum.imaginary, Some(vec![2.0, -4.0, 6.0]));
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(spectrum.metadata.frequency_mhz, Some(400.13));
    assert_eq!(spectrum.metadata.solvent.as_deref(), Some("CDCl3"));
    assert_eq!(spectrum.metadata.temperature_k, Some(298.15));
    assert_eq!(spectrum.metadata.origin.as_deref(), Some("fixture user"));

    remove_dir(root)?;
    Ok(())
}

#[test]
fn reads_little_endian_float_complex_fid() -> anyhow::Result<()> {
    let root = synthetic_dataset("little-float")?;
    write_procpar(
        &root,
        "\
sw 1 1 5 5 5 2 1 8203 1 64
1 500
0
",
    )?;
    write_fid(
        &root,
        EndianForTest::Little,
        DataForTest::F32(&[0.5, -0.25, 1.5, -2.5]),
        1,
        1,
    )?;

    let spectrum = AgilentFid1D.read_dir(root.join("fid"))?;

    assert_eq!(spectrum.x.values, vec![0.0, 0.002]);
    assert_eq!(spectrum.intensities, vec![0.5, 1.5]);
    assert_eq!(spectrum.imaginary, Some(vec![-0.25, -2.5]));

    remove_dir(root)?;
    Ok(())
}

#[test]
fn rejects_arrayed_or_multidimensional_fid() -> anyhow::Result<()> {
    let root = synthetic_dataset("arrayed")?;
    write_procpar(&root, "")?;
    write_fid(
        &root,
        EndianForTest::Big,
        DataForTest::I16(&[1, 2, 3, 4]),
        2,
        1,
    )?;

    let error =
        read_agilent_fid_1d_dir(&root).expect_err("arrayed Agilent FID should be unsupported");
    assert!(matches!(error, RSpinError::Unsupported { .. }));

    remove_dir(root)?;
    Ok(())
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

fn synthetic_dataset(name: &str) -> anyhow::Result<PathBuf> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let mut root = std::env::temp_dir();
    root.push(format!(
        "rspin-agilent-{name}-{}-{nanos}",
        std::process::id()
    ));
    fs::create_dir_all(&root)?;
    Ok(root)
}

fn write_procpar(root: &Path, text: &str) -> anyhow::Result<()> {
    fs::write(root.join("procpar"), text)?;
    Ok(())
}

fn write_fid(
    root: &Path,
    endian: EndianForTest,
    data: DataForTest<'_>,
    nblocks: i32,
    ntraces: i32,
) -> anyhow::Result<()> {
    let (ebytes, status, data_bytes) = encode_data(endian, data);
    let np_values = i32::try_from(data_bytes.len() / usize::try_from(ebytes)?)?;
    let tbytes = np_values * ebytes;
    let bbytes = tbytes + i32::try_from(BLOCK_HEADER_LEN)?;

    let mut fid_bytes = Vec::new();
    push_i32(&mut fid_bytes, endian, nblocks);
    push_i32(&mut fid_bytes, endian, ntraces);
    push_i32(&mut fid_bytes, endian, np_values);
    push_i32(&mut fid_bytes, endian, ebytes);
    push_i32(&mut fid_bytes, endian, tbytes);
    push_i32(&mut fid_bytes, endian, bbytes);
    push_i16(&mut fid_bytes, endian, 0);
    push_i16(&mut fid_bytes, endian, status);
    push_i32(&mut fid_bytes, endian, 1);

    push_i16(&mut fid_bytes, endian, 0);
    push_i16(&mut fid_bytes, endian, status);
    push_i16(&mut fid_bytes, endian, 1);
    push_i16(&mut fid_bytes, endian, 0);
    push_i32(&mut fid_bytes, endian, 1);
    push_f32(&mut fid_bytes, endian, 0.0);
    push_f32(&mut fid_bytes, endian, 0.0);
    push_f32(&mut fid_bytes, endian, 0.0);
    push_f32(&mut fid_bytes, endian, 0.0);
    fid_bytes.extend(data_bytes);

    fs::write(root.join("fid"), fid_bytes)?;
    Ok(())
}

fn encode_data(endian: EndianForTest, data: DataForTest<'_>) -> (i32, i16, Vec<u8>) {
    match data {
        DataForTest::I16(values) => {
            let mut bytes = Vec::with_capacity(values.len() * 2);
            for value in values {
                push_i16(&mut bytes, endian, *value);
            }
            (2, 0x0001 | STATUS_COMPLEX, bytes)
        }
        DataForTest::I32(values) => {
            let mut bytes = Vec::with_capacity(values.len() * 4);
            for value in values {
                push_i32(&mut bytes, endian, *value);
            }
            (4, 0x0001 | 0x0004 | STATUS_COMPLEX, bytes)
        }
        DataForTest::F32(values) => {
            let mut bytes = Vec::with_capacity(values.len() * 4);
            for value in values {
                push_f32(&mut bytes, endian, *value);
            }
            (4, 0x0001 | STATUS_FLOAT | STATUS_COMPLEX, bytes)
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

fn remove_dir(path: PathBuf) -> anyhow::Result<()> {
    fs::remove_dir_all(path)?;
    Ok(())
}
