use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use rspin_core::{Nucleus, RSpinError, Unit};

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
