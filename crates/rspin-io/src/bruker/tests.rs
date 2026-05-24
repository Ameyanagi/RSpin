use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use rspin_core::{Nucleus, RSpinError, Unit};

use crate::SpectrumPathReader;

use super::*;

mod processed;
mod raw_cases;

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
    fs::write(processed.join("1r"), i32_bytes(values, byte_order))?;
    Ok(())
}

fn write_processed_1d_imaginary(
    root: &Path,
    values: &[i32],
    byte_order: ByteOrder,
) -> anyhow::Result<()> {
    fs::write(root.join("pdata/1/1i"), i32_bytes(values, byte_order))?;
    Ok(())
}

fn write_processed_2d_dir(
    root: &Path,
    direct_parameters: &str,
    indirect_parameters: &str,
    real: &[i32],
    imaginary: Option<&[i32]>,
    byte_order: ByteOrder,
) -> anyhow::Result<()> {
    let processed = root.join("pdata/1");
    write_text(&processed.join("procs"), direct_parameters)?;
    write_text(&processed.join("proc2s"), indirect_parameters)?;
    fs::write(processed.join("2rr"), i32_bytes(real, byte_order))?;
    if let Some(values) = imaginary {
        fs::write(processed.join("2ri"), i32_bytes(values, byte_order))?;
    }
    Ok(())
}

fn write_raw_fid(root: &Path, values: &[i32], byte_order: ByteOrder) -> anyhow::Result<()> {
    fs::write(root.join("fid"), i32_bytes(values, byte_order))?;
    Ok(())
}

fn write_raw_ser(root: &Path, rows: &[Vec<i32>], byte_order: ByteOrder) -> anyhow::Result<()> {
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
    fs::write(root.join("ser"), bytes)?;
    Ok(())
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

#[test]
fn inspects_bruker_parameter_file_versions() -> anyhow::Result<()> {
    let info = inspect_bruker_parameter_file(
        "\
##TITLE= Bruker Parameters
##JCAMPDX= 5.00
##DATATYPE= Parameter Values
##ORIGIN= fixture
##OWNER= local
##$TD= 4
",
    )?;

    let version = info
        .jcamp_dx_version
        .as_ref()
        .ok_or_else(|| RSpinError::Parse {
            format: "Bruker",
            message: "missing inspected Bruker JCAMPDX version".to_owned(),
        })?;
    assert_eq!(version.raw, "5.00");
    assert_eq!(version.major, 5);
    assert_eq!(info.data_type.as_deref(), Some("Parameter Values"));
    assert_eq!(info.origin.as_deref(), Some("fixture"));
    assert_eq!(info.owner.as_deref(), Some("local"));
    assert!(info.is_supported_by_current_readers());
    Ok(())
}

#[test]
fn preserves_future_bruker_parameter_versions_for_routing() -> anyhow::Result<()> {
    let info = inspect_bruker_parameter_file("##JCAMPDX= 6.00\n##$TD= 4\n")?;

    assert!(!info.is_supported_by_current_readers());
    let error = info
        .validate_supported_by_current_readers()
        .expect_err("future Bruker parameter version should be rejected");
    assert!(matches!(error, RSpinError::Unsupported { .. }));
    Ok(())
}

#[test]
fn rejects_malformed_bruker_parameter_versions() {
    let error = inspect_bruker_parameter_file("##JCAMPDX= release\n")
        .expect_err("malformed Bruker parameter version should fail");
    assert!(matches!(error, RSpinError::Parse { .. }));
}
