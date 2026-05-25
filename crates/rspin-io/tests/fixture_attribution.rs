//! Attribution checks for committed external-source fixture files.

use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

#[test]
fn redistributed_fixture_files_are_documented() -> anyhow::Result<()> {
    let testdata = manifest_dir().join("testdata");
    let top_readme = fs::read_to_string(testdata.join("README.md"))?;
    assert!(top_readme.contains("zenodo_7100132/"));
    assert!(top_readme.contains("nmrxiv/cc0/"));
    assert!(top_readme.contains("nmrxiv/cc-by-4.0/"));
    assert!(top_readme.contains("nmrml/mit/"));
    assert!(top_readme.contains("dataverse/cc0/"));
    assert!(top_readme.contains("Fixture Rules"));

    let zenodo_readme = fs::read_to_string(testdata.join("zenodo_7100132/README.md"))?;
    assert!(zenodo_readme.contains("License: MIT"));
    assert!(zenodo_readme.contains("Authors:"));
    assert_checksum_inventory_covers(&testdata.join("zenodo_7100132"), "", &zenodo_readme)?;

    let nmrxiv_readme = fs::read_to_string(testdata.join("nmrxiv/README.md"))?;
    assert!(nmrxiv_readme.contains("Creative Commons Zero v1.0 Universal"));
    assert!(nmrxiv_readme.contains("Creative Commons Attribution 4.0 International"));
    assert!(nmrxiv_readme.matches("- Authors:").count() >= 2);
    assert_checksum_inventory_covers(&testdata.join("nmrxiv"), "cc0", &nmrxiv_readme)?;
    assert_checksum_inventory_covers(&testdata.join("nmrxiv"), "cc-by-4.0", &nmrxiv_readme)?;

    let nmrml_readme = fs::read_to_string(testdata.join("nmrml/README.md"))?;
    assert!(nmrml_readme.contains("License: MIT"));
    assert!(nmrml_readme.contains("Authors:"));
    assert_checksum_inventory_covers(&testdata.join("nmrml"), "mit", &nmrml_readme)?;

    let dataverse_readme = fs::read_to_string(testdata.join("dataverse/README.md"))?;
    assert!(dataverse_readme.contains("Creative Commons Zero v1.0 Universal"));
    assert!(dataverse_readme.contains("Package license: MIT"));
    assert!(dataverse_readme.contains("Authors:"));
    assert_checksum_inventory_covers(&testdata.join("dataverse"), "cc0", &dataverse_readme)?;
    Ok(())
}

fn assert_checksum_inventory_covers(root: &Path, scope: &str, readme: &str) -> anyhow::Result<()> {
    let entries = checksum_entries(readme);
    let listed = entries
        .iter()
        .filter(|entry| path_in_scope(&entry.path, scope))
        .map(|entry| entry.path.as_str())
        .collect::<HashSet<_>>();

    let files = collect_relative_files(root, scope)?;
    assert!(
        !files.is_empty(),
        "fixture scope {scope:?} should contain committed files"
    );
    for file in &files {
        assert!(
            listed.contains(file.as_str()),
            "missing checksum entry for redistributed fixture {file}"
        );
    }

    for entry in entries
        .iter()
        .filter(|entry| path_in_scope(&entry.path, scope))
    {
        assert!(
            root.join(&entry.path).is_file(),
            "checksum entry points at missing fixture {}",
            entry.path
        );
    }

    Ok(())
}

#[derive(Debug)]
struct ChecksumEntry {
    path: String,
}

fn checksum_entries(readme: &str) -> Vec<ChecksumEntry> {
    readme
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let hash = parts.next()?;
            let path = parts.next()?;
            if parts.next().is_some() || hash.len() != 64 {
                return None;
            }
            assert!(
                hash.bytes().all(|byte| byte.is_ascii_hexdigit()),
                "checksum hash should be hex for {path}"
            );
            Some(ChecksumEntry {
                path: path.to_owned(),
            })
        })
        .collect()
}

fn collect_relative_files(root: &Path, scope: &str) -> anyhow::Result<Vec<String>> {
    let start = if scope.is_empty() {
        root.to_path_buf()
    } else {
        root.join(scope)
    };
    let mut files = Vec::new();
    collect_relative_files_into(root, &start, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_relative_files_into(
    relative_root: &Path,
    directory: &Path,
    files: &mut Vec<String>,
) -> anyhow::Result<()> {
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_relative_files_into(relative_root, &path, files)?;
        } else if path.is_file() && !is_readme(&path) {
            let relative = path.strip_prefix(relative_root)?;
            let Some(relative) = relative.to_str() else {
                anyhow::bail!("fixture path is not valid UTF-8: {}", path.display());
            };
            files.push(relative.replace('\\', "/"));
        }
    }
    Ok(())
}

fn is_readme(path: &Path) -> bool {
    path.file_name()
        .and_then(std::ffi::OsStr::to_str)
        .is_some_and(|name| name.eq_ignore_ascii_case("README.md"))
}

fn path_in_scope(path: &str, scope: &str) -> bool {
    scope.is_empty()
        || path == scope
        || path
            .strip_prefix(scope)
            .is_some_and(|suffix| suffix.starts_with('/'))
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
