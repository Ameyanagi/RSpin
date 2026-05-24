use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use rspin_core::{Nucleus, RSpinError, Unit};

use crate::SpectrumPathReader;

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
    assert_eq!(
        spectrum.metadata.property("agilent.procpar.sfrq"),
        Some("400.13")
    );

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

    let spectrum = AgilentFid1D.read_path(&root.join("fid"))?;

    assert_eq!(spectrum.x.values, vec![0.0, 0.002]);
    assert_eq!(spectrum.intensities, vec![0.5, 1.5]);
    assert_eq!(spectrum.imaginary, Some(vec![-0.25, -2.5]));

    remove_dir(root)?;
    Ok(())
}

#[test]
fn reads_big_endian_i32_processed_phasefile() -> anyhow::Result<()> {
    let root = synthetic_dataset("processed-phasefile")?;
    write_procpar(
        &root,
        "\
tn 2 2 4 0 0 2 1 8 1 64
1 \"H1\"
0
sfrq 1 1 1000000000 0 0 2 1 11 1 64
1 500
0
sw 1 1 5 5 5 2 1 8203 1 64
1 1000
0
rfl 1 1 1000000000 -1000000000 0 2 1 11 1 64
1 750
0
rfp 1 1 1000000000 -1000000000 0 2 1 11 1 64
1 250
0
comment 2 2 32 0 0 2 1 0 1 64
1 \"processed demo\"
0
",
    )?;
    write_phasefile(&root, EndianForTest::Big, DataForTest::I32(&[10, 20, -5]))?;

    let spectrum = AgilentProcessed1D.read_path(&root.join("datdir"))?;

    assert_eq!(spectrum.x.unit, Unit::Ppm);
    assert_eq!(spectrum.x.values, vec![1.0, 0.0, -1.0]);
    assert_eq!(spectrum.intensities, vec![10.0, 20.0, -5.0]);
    assert!(spectrum.imaginary.is_none());
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(spectrum.metadata.frequency_mhz, Some(500.0));
    assert_eq!(spectrum.metadata.name.as_deref(), Some("processed demo"));

    remove_dir(root)?;
    Ok(())
}

#[test]
fn reads_processed_phasefile_path_with_hertz_axis_fallback() -> anyhow::Result<()> {
    let root = synthetic_dataset("processed-hertz-axis")?;
    write_procpar(
        &root,
        "\
sw 1 1 5 5 5 2 1 8203 1 64
1 400
0
",
    )?;
    write_phasefile(&root, EndianForTest::Little, DataForTest::F32(&[0.5, 1.5]))?;

    let spectrum = read_agilent_processed_1d_dir(root.join("datdir/phasefile"))?;

    assert_eq!(spectrum.x.unit, Unit::Hertz);
    assert_eq!(spectrum.x.values, vec![200.0, -200.0]);
    assert_eq!(spectrum.intensities, vec![0.5, 1.5]);

    remove_dir(root)?;
    Ok(())
}

#[test]
fn reads_processed_2d_phasefile() -> anyhow::Result<()> {
    let root = synthetic_dataset("processed-2d-phasefile")?;
    write_procpar(
        &root,
        "\
acqdim 7 1 32767 0 0 2 1 0 1 64
1 2
0
tn 2 2 4 0 0 2 1 8 1 64
1 \"H1\"
0
sfrq 1 1 1000000000 0 0 2 1 11 1 64
1 500
0
sw 1 1 5 5 5 2 1 8203 1 64
1 1000
0
rfl 1 1 1000000000 -1000000000 0 2 1 11 1 64
1 750
0
rfp 1 1 1000000000 -1000000000 0 2 1 11 1 64
1 250
0
sw1 1 1 5000000 1 -1.25e-08 2 1 0 1 64
1 400
0
dfrq 1 1 1000000000 0 0 2 1 11 1 64
1 100
0
rfl1 1 1 1000000000 -1000000000 0 2 1 11 1 64
1 300
0
rfp1 1 1 1000000000 -1000000000 0 2 1 11 1 64
1 100
0
comment 2 2 32 0 0 2 1 0 1 64
1 \"processed 2d demo\"
0
",
    )?;
    write_phasefile_matrix(
        &root,
        EndianForTest::Little,
        DataForTest::I32(&[1, 2, 3, 4, 5, 6]),
        2,
        1,
    )?;

    let spectrum = AgilentProcessed2D.read_path(&root.join("datdir/phasefile"))?;

    assert_eq!(spectrum.shape(), (3, 2));
    assert_eq!(spectrum.x.unit, Unit::Ppm);
    assert_eq!(spectrum.x.values, vec![1.0, 0.0, -1.0]);
    assert_eq!(spectrum.y.unit, Unit::Ppm);
    assert_eq!(spectrum.y.values, vec![2.0, -2.0]);
    assert_eq!(spectrum.z, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    assert!(spectrum.imaginary.is_none());
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(spectrum.metadata.frequency_mhz, Some(500.0));
    assert_eq!(spectrum.metadata.name.as_deref(), Some("processed 2d demo"));

    remove_dir(root)?;
    Ok(())
}

#[test]
fn rejects_truncated_processed_phasefile() -> anyhow::Result<()> {
    let root = synthetic_dataset("processed-truncated")?;
    write_procpar(&root, "")?;
    write_phasefile(&root, EndianForTest::Big, DataForTest::I32(&[1, 2, 3]))?;
    let phasefile = root.join("datdir/phasefile");
    let mut bytes = fs::read(&phasefile)?;
    let truncated_len = bytes
        .len()
        .checked_sub(4)
        .ok_or_else(|| anyhow::anyhow!("synthetic phasefile is unexpectedly short"))?;
    bytes.truncate(truncated_len);
    fs::write(&phasefile, bytes)?;

    let error = read_agilent_processed_1d_dir(&root).expect_err("truncated phasefile should fail");
    assert!(matches!(error, RSpinError::Parse { .. }));
    assert!(error.to_string().contains("phasefile"));

    remove_dir(root)?;
    Ok(())
}

#[test]
fn rejects_1d_phasefile_for_2d_reader() -> anyhow::Result<()> {
    let root = synthetic_dataset("processed-1d-as-2d")?;
    write_procpar(&root, "")?;
    write_phasefile(&root, EndianForTest::Big, DataForTest::I32(&[1, 2, 3]))?;

    let error =
        read_agilent_processed_2d_dir(&root).expect_err("1D phasefile should not be read as 2D");
    assert!(matches!(error, RSpinError::Unsupported { .. }));

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

#[test]
fn reads_big_endian_i32_complex_2d_fid() -> anyhow::Result<()> {
    let root = synthetic_dataset("big-i32-2d")?;
    write_procpar(
        &root,
        "\
acqdim 7 1 32767 0 0 2 1 0 1 64
1 2
0
arrayelemts 1 1 9.99999984307e+17 -9.99999984307e+17 0 2 1 0 1 64
1 1
0
tn 2 2 4 0 0 2 1 8 1 64
1 \"H1\"
0
sfrq 1 1 1000000000 0 0 2 1 11 1 64
1 400.13
0
sw 1 1 5 5 5 2 1 8203 1 64
1 1000
0
sw1 1 1 5000000 1 -1.25e-08 2 1 0 1 64
1 200
0
",
    )?;
    write_fid(
        &root,
        EndianForTest::Big,
        DataForTest::I32(&[1, 2, 3, 4, -5, 6, 7, -8]),
        2,
        1,
    )?;

    let spectrum = read_agilent_fid_2d_dir(root.join("fid"))?;

    assert_eq!(spectrum.shape(), (2, 2));
    assert_eq!(spectrum.x.unit, Unit::Seconds);
    assert_eq!(spectrum.x.values, vec![0.0, 0.001]);
    assert_eq!(spectrum.y.unit, Unit::Seconds);
    assert_eq!(spectrum.y.values, vec![0.0, 0.005]);
    assert_eq!(spectrum.z, vec![1.0, 3.0, -5.0, 7.0]);
    assert_eq!(spectrum.imaginary, Some(vec![2.0, 4.0, 6.0, -8.0]));
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(spectrum.metadata.frequency_mhz, Some(400.13));

    remove_dir(root)?;
    Ok(())
}

#[test]
fn reads_arrayed_2d_fid_with_point_axis() -> anyhow::Result<()> {
    let root = synthetic_dataset("arrayed-2d")?;
    write_procpar(
        &root,
        "\
acqdim 7 1 32767 0 0 2 1 0 1 64
1 2
0
array 2 2 256 0 0 2 1 1 1 64
1 \"phase\"
0
arrayelemts 1 1 9.99999984307e+17 -9.99999984307e+17 0 2 1 0 1 64
1 2
0
sw 1 1 5 5 5 2 1 8203 1 64
1 1000
0
sw1 1 1 5000000 1 -1.25e-08 2 1 0 1 64
1 200
0
",
    )?;
    write_fid(
        &root,
        EndianForTest::Little,
        DataForTest::F32(&[0.5, -0.25, 1.5, -2.5, 3.0, 4.0, 5.0, 6.0]),
        2,
        1,
    )?;

    let spectrum = AgilentFid2D.read_path(&root)?;

    assert_eq!(spectrum.shape(), (2, 2));
    assert_eq!(spectrum.y.unit, Unit::Points);
    assert_eq!(spectrum.y.values, vec![0.0, 1.0]);
    assert_eq!(spectrum.z, vec![0.5, 1.5, 3.0, 5.0]);
    assert_eq!(spectrum.imaginary, Some(vec![-0.25, -2.5, 4.0, 6.0]));

    remove_dir(root)?;
    Ok(())
}

#[test]
fn reads_multitrace_2d_fid_block() -> anyhow::Result<()> {
    let root = synthetic_dataset("multitrace-2d")?;
    write_procpar(
        &root,
        "\
acqdim 7 1 32767 0 0 2 1 0 1 64
1 2
0
sw 1 1 5 5 5 2 1 8203 1 64
1 1000
0
",
    )?;
    write_fid(
        &root,
        EndianForTest::Big,
        DataForTest::I16(&[1, -1, 2, -2, 3, -3, 4, -4]),
        1,
        2,
    )?;

    let spectrum = read_agilent_fid_2d_dir(&root)?;

    assert_eq!(spectrum.shape(), (2, 2));
    assert_eq!(spectrum.z, vec![1.0, 2.0, 3.0, 4.0]);
    assert_eq!(spectrum.imaginary, Some(vec![-1.0, -2.0, -3.0, -4.0]));

    remove_dir(root)?;
    Ok(())
}

#[test]
fn rejects_three_dimensional_fid_for_2d_reader() -> anyhow::Result<()> {
    let root = synthetic_dataset("three-dimensional")?;
    write_procpar(
        &root,
        "\
acqdim 7 1 32767 0 0 2 1 0 1 64
1 3
0
",
    )?;
    write_fid(
        &root,
        EndianForTest::Big,
        DataForTest::I16(&[1, 2, 3, 4]),
        2,
        1,
    )?;

    let error = read_agilent_fid_2d_dir(&root).expect_err("3D Agilent FID should be unsupported");
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
    let row_count = usize::try_from(nblocks)?
        .checked_mul(usize::try_from(ntraces)?)
        .ok_or_else(|| anyhow::anyhow!("synthetic Agilent row count overflow"))?;
    let value_count = data_bytes.len() / usize::try_from(ebytes)?;
    let np_values = i32::try_from(value_count / row_count)?;
    let tbytes = np_values * ebytes;
    let trace_bytes = usize::try_from(tbytes)?;
    let block_data_len = usize::try_from(ntraces)?
        .checked_mul(trace_bytes)
        .ok_or_else(|| anyhow::anyhow!("synthetic Agilent block length overflow"))?;
    let bbytes = i32::try_from(
        BLOCK_HEADER_LEN
            .checked_add(block_data_len)
            .ok_or_else(|| anyhow::anyhow!("synthetic Agilent block byte count overflow"))?,
    )?;

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

    for block_index in 0..usize::try_from(nblocks)? {
        push_i16(&mut fid_bytes, endian, 0);
        push_i16(&mut fid_bytes, endian, status);
        push_i16(&mut fid_bytes, endian, 1);
        push_i16(&mut fid_bytes, endian, 0);
        push_i32(&mut fid_bytes, endian, i32::try_from(block_index + 1)?);
        push_f32(&mut fid_bytes, endian, 0.0);
        push_f32(&mut fid_bytes, endian, 0.0);
        push_f32(&mut fid_bytes, endian, 0.0);
        push_f32(&mut fid_bytes, endian, 0.0);
        let block_data_start = block_index
            .checked_mul(usize::try_from(ntraces)?)
            .and_then(|index| index.checked_mul(trace_bytes))
            .ok_or_else(|| anyhow::anyhow!("synthetic Agilent block offset overflow"))?;
        let block_data_end = block_data_start
            .checked_add(block_data_len)
            .ok_or_else(|| anyhow::anyhow!("synthetic Agilent block end overflow"))?;
        fid_bytes.extend(&data_bytes[block_data_start..block_data_end]);
    }

    fs::write(root.join("fid"), fid_bytes)?;
    Ok(())
}

fn write_phasefile(
    root: &Path,
    endian: EndianForTest,
    data: DataForTest<'_>,
) -> anyhow::Result<()> {
    write_phasefile_matrix(root, endian, data, 1, 1)
}

fn write_phasefile_matrix(
    root: &Path,
    endian: EndianForTest,
    data: DataForTest<'_>,
    nblocks: i32,
    ntraces: i32,
) -> anyhow::Result<()> {
    let datdir = root.join("datdir");
    fs::create_dir_all(&datdir)?;
    let (ebytes, status, data_bytes) = encode_real_data(endian, data);
    let row_count = usize::try_from(nblocks)?
        .checked_mul(usize::try_from(ntraces)?)
        .ok_or_else(|| anyhow::anyhow!("synthetic Agilent phasefile row count overflow"))?;
    let value_count = data_bytes.len() / usize::try_from(ebytes)?;
    let np_values = i32::try_from(value_count / row_count)?;
    let tbytes = np_values
        .checked_mul(ebytes)
        .ok_or_else(|| anyhow::anyhow!("synthetic Agilent phasefile trace size overflow"))?;
    let trace_bytes = usize::try_from(tbytes)?;
    let block_data_len = usize::try_from(ntraces)?
        .checked_mul(trace_bytes)
        .ok_or_else(|| anyhow::anyhow!("synthetic Agilent phasefile block length overflow"))?;
    let bbytes = i32::try_from(
        BLOCK_HEADER_LEN
            .checked_add(block_data_len)
            .ok_or_else(|| anyhow::anyhow!("synthetic Agilent phasefile block size overflow"))?,
    )?;

    let mut phasefile_bytes = Vec::new();
    push_i32(&mut phasefile_bytes, endian, nblocks);
    push_i32(&mut phasefile_bytes, endian, ntraces);
    push_i32(&mut phasefile_bytes, endian, np_values);
    push_i32(&mut phasefile_bytes, endian, ebytes);
    push_i32(&mut phasefile_bytes, endian, tbytes);
    push_i32(&mut phasefile_bytes, endian, bbytes);
    push_i16(&mut phasefile_bytes, endian, 0);
    push_i16(&mut phasefile_bytes, endian, status);
    push_i32(&mut phasefile_bytes, endian, 1);

    for block_index in 0..usize::try_from(nblocks)? {
        push_i16(&mut phasefile_bytes, endian, 0);
        push_i16(&mut phasefile_bytes, endian, status);
        push_i16(&mut phasefile_bytes, endian, 1);
        push_i16(&mut phasefile_bytes, endian, 0);
        push_i32(
            &mut phasefile_bytes,
            endian,
            i32::try_from(block_index + 1)?,
        );
        push_f32(&mut phasefile_bytes, endian, 0.0);
        push_f32(&mut phasefile_bytes, endian, 0.0);
        push_f32(&mut phasefile_bytes, endian, 0.0);
        push_f32(&mut phasefile_bytes, endian, 0.0);
        let block_data_start = block_index
            .checked_mul(usize::try_from(ntraces)?)
            .and_then(|index| index.checked_mul(trace_bytes))
            .ok_or_else(|| anyhow::anyhow!("synthetic Agilent phasefile block offset overflow"))?;
        let block_data_end = block_data_start
            .checked_add(block_data_len)
            .ok_or_else(|| anyhow::anyhow!("synthetic Agilent phasefile block end overflow"))?;
        phasefile_bytes.extend(&data_bytes[block_data_start..block_data_end]);
    }

    fs::write(datdir.join("phasefile"), phasefile_bytes)?;
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

fn remove_dir(path: PathBuf) -> anyhow::Result<()> {
    fs::remove_dir_all(path)?;
    Ok(())
}
