use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use rspin_core::{Nucleus, RSpinError, Unit};

use super::*;

#[test]
fn reads_processed_1d_dataset_root() -> anyhow::Result<()> {
    let root = synthetic_dataset("root")?;
    write_text(
        &root.join("acqus"),
        "\
##$NUC1= <1H>
##$SFO1= 400.13
##$SOLVENT= <CDCl3>
##$TE= 298.15
##$OWNER= <local fixture>
",
    )?;
    write_processed_dir(
        &root,
        "\
##$SI= 4
##$BYTORDP= 0
##$DTYPP= 0
##$NC_proc= 0
##$OFFSET= 10
##$SW_p= 4000
##$SF= 400
##$AXNUC= <1H>
",
        &[100, -50, 25, 0],
        ByteOrder::Little,
    )?;
    write_text(&root.join("pdata/1/title"), "ethyl acetate\n")?;

    let spectrum = read_bruker_processed_1d_dir(&root)?;

    assert_eq!(spectrum.metadata.name.as_deref(), Some("ethyl acetate"));
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(spectrum.metadata.frequency_mhz, Some(400.0));
    assert_eq!(spectrum.metadata.solvent.as_deref(), Some("CDCl3"));
    assert_eq!(spectrum.metadata.temperature_k, Some(298.15));
    assert_eq!(spectrum.metadata.origin.as_deref(), Some("local fixture"));
    assert_eq!(spectrum.x.unit, Unit::Ppm);
    assert_close(spectrum.x.values[0], 10.0);
    assert_close(spectrum.x.values[1], 20.0 / 3.0);
    assert_close(spectrum.x.values[2], 10.0 / 3.0);
    assert_close(spectrum.x.values[3], 0.0);
    assert_eq!(spectrum.intensities, vec![100.0, -50.0, 25.0, 0.0]);

    remove_dir(root)?;
    Ok(())
}

#[test]
fn reads_processed_directory_with_scaling_and_big_endian_data() -> anyhow::Result<()> {
    let root = synthetic_dataset("processed")?;
    write_processed_dir(
        &root,
        "\
##$SI= 3
##$BYTORDP= 1
##$DTYPP= 0
##$NC_proc= -1
",
        &[2, -4, 6],
        ByteOrder::Big,
    )?;

    let spectrum = BrukerProcessed1D.read_dir(root.join("pdata/1"))?;

    assert_eq!(spectrum.x.unit, Unit::Points);
    assert_eq!(spectrum.x.values, vec![0.0, 1.0, 2.0]);
    assert_eq!(spectrum.intensities, vec![4.0, -8.0, 12.0]);

    remove_dir(root)?;
    Ok(())
}

#[test]
fn rejects_unsupported_processed_data_type() -> anyhow::Result<()> {
    let root = synthetic_dataset("unsupported")?;
    write_processed_dir(
        &root,
        "\
##$SI= 1
##$DTYPP= 2
",
        &[1],
        ByteOrder::Little,
    )?;

    let error = read_bruker_processed_1d_dir(&root)
        .expect_err("unsupported processed data type should fail");
    assert!(matches!(error, RSpinError::Unsupported { .. }));

    remove_dir(root)?;
    Ok(())
}

#[test]
fn reads_raw_1d_fid_dataset_root() -> anyhow::Result<()> {
    let root = synthetic_dataset("raw")?;
    write_text(
        &root.join("acqus"),
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
##$OWNER= <raw fixture>
##$PULPROG= <zg>
",
    )?;
    write_raw_fid(&root, &[1, -2, 3, -4], ByteOrder::Big)?;

    let spectrum = read_bruker_fid_1d_dir(&root)?;

    assert_eq!(spectrum.len(), 2);
    assert_eq!(spectrum.x.unit, Unit::Seconds);
    assert_eq!(spectrum.x.values, vec![0.0, 0.001]);
    assert_eq!(spectrum.intensities, vec![2.0, 6.0]);
    assert_eq!(spectrum.imaginary, Some(vec![-4.0, -8.0]));
    assert_eq!(spectrum.metadata.name.as_deref(), Some("zg"));
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Carbon13));
    assert_eq!(spectrum.metadata.frequency_mhz, Some(125.5));
    assert_eq!(spectrum.metadata.solvent.as_deref(), Some("CDCl3"));
    assert_eq!(spectrum.metadata.temperature_k, Some(300.0));
    assert_eq!(spectrum.metadata.origin.as_deref(), Some("raw fixture"));

    remove_dir(root)?;
    Ok(())
}

#[test]
fn reads_raw_1d_fid_file_path() -> anyhow::Result<()> {
    let root = synthetic_dataset("raw-file")?;
    write_text(
        &root.join("acqus"),
        "\
##$TD= 2
##$BYTORDA= 0
##$DTYPA= 0
",
    )?;
    write_raw_fid(&root, &[5, -7], ByteOrder::Little)?;

    let spectrum = BrukerFid1D.read_dir(root.join("fid"))?;

    assert_eq!(spectrum.x.unit, Unit::Points);
    assert_eq!(spectrum.intensities, vec![5.0]);
    assert_eq!(spectrum.imaginary, Some(vec![-7.0]));

    remove_dir(root)?;
    Ok(())
}

#[test]
fn rejects_unsupported_raw_data_type() -> anyhow::Result<()> {
    let root = synthetic_dataset("raw-unsupported")?;
    write_text(
        &root.join("acqus"),
        "\
##$TD= 2
##$DTYPA= 2
",
    )?;
    write_raw_fid(&root, &[1, 2], ByteOrder::Little)?;

    let error = read_bruker_fid_1d_dir(&root).expect_err("unsupported raw data type should fail");
    assert!(matches!(error, RSpinError::Unsupported { .. }));

    remove_dir(root)?;
    Ok(())
}

#[derive(Clone, Copy)]
enum ByteOrder {
    Little,
    Big,
}

fn synthetic_dataset(name: &str) -> anyhow::Result<PathBuf> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let mut root = std::env::temp_dir();
    root.push(format!(
        "rspin-bruker-{name}-{}-{nanos}",
        std::process::id()
    ));
    fs::create_dir_all(root.join("pdata/1"))?;
    Ok(root)
}

fn write_processed_dir(
    root: &Path,
    procs: &str,
    values: &[i32],
    byte_order: ByteOrder,
) -> anyhow::Result<()> {
    let processed = root.join("pdata/1");
    write_text(&processed.join("procs"), procs)?;
    let bytes = values
        .iter()
        .flat_map(|value| match byte_order {
            ByteOrder::Little => value.to_le_bytes(),
            ByteOrder::Big => value.to_be_bytes(),
        })
        .collect::<Vec<_>>();
    fs::write(processed.join("1r"), bytes)?;
    Ok(())
}

fn write_raw_fid(root: &Path, values: &[i32], byte_order: ByteOrder) -> anyhow::Result<()> {
    let bytes = values
        .iter()
        .flat_map(|value| match byte_order {
            ByteOrder::Little => value.to_le_bytes(),
            ByteOrder::Big => value.to_be_bytes(),
        })
        .collect::<Vec<_>>();
    fs::write(root.join("fid"), bytes)?;
    Ok(())
}

fn write_text(path: &Path, text: &str) -> anyhow::Result<()> {
    fs::write(path, text)?;
    Ok(())
}

fn remove_dir(path: PathBuf) -> anyhow::Result<()> {
    fs::remove_dir_all(path)?;
    Ok(())
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1.0e-12,
        "{actual} != {expected}"
    );
}
