//! Unified spectrum bundle loading.

use std::{
    fs,
    path::{Path, PathBuf},
};

use rspin_core::{Molecule, RSpinError, Result};

use crate::agilent::is_agilent_arrayed_2d_series_array;
use crate::{
    NmreDataRecord, Spectrum1DPathFormat, Spectrum2DPathFormat, inspect_agilent_binary_file,
    inspect_agilent_procpar,
};

mod exact;
mod loader;
mod model;
mod selectors;
mod source_format;
pub use exact::{
    load_spectrum_1d, load_spectrum_1d_by_source_format, load_spectrum_1d_by_source_vendor,
    load_spectrum_1d_many, load_spectrum_1d_many_relative_to, load_spectrum_1d_many_with_source,
    load_spectrum_1d_many_with_source_relative_to, load_spectrum_1d_paths,
    load_spectrum_1d_paths_relative_to, load_spectrum_1d_paths_with_source,
    load_spectrum_1d_paths_with_source_relative_to, load_spectrum_1d_relative_to,
    load_spectrum_1d_with_source, load_spectrum_1d_with_source_by_source_format,
    load_spectrum_1d_with_source_by_source_vendor, load_spectrum_1d_with_source_relative_to,
    load_spectrum_2d, load_spectrum_2d_by_source_format, load_spectrum_2d_by_source_vendor,
    load_spectrum_2d_many, load_spectrum_2d_many_relative_to, load_spectrum_2d_many_with_source,
    load_spectrum_2d_many_with_source_relative_to, load_spectrum_2d_paths,
    load_spectrum_2d_paths_relative_to, load_spectrum_2d_paths_with_source,
    load_spectrum_2d_paths_with_source_relative_to, load_spectrum_2d_relative_to,
    load_spectrum_2d_with_source, load_spectrum_2d_with_source_by_source_format,
    load_spectrum_2d_with_source_by_source_vendor, load_spectrum_2d_with_source_relative_to,
};
use loader::FileCandidateKind;
pub use loader::SpectrumBundleLoader;
pub use model::{
    LoadWarning, LoadedSource, LoadedSpectrum, SourceFormatCount, SourceVendorCount,
    SpectrumBundle, SpectrumBundleSummary,
};
pub use source_format::{
    LoadedSourceFormat, LoadedSourceVendor, parse_loaded_source_format, parse_loaded_source_vendor,
};

/// High-level reader for supported `RSpin` spectrum inputs.
pub type RSpinReader = SpectrumBundleLoader;

/// Loads all supported spectrum bundle data from a file or directory path.
///
/// # Errors
///
/// Returns an error when the path is missing or no readable bundle data is found.
pub fn load_spectra(path: impl AsRef<Path>) -> Result<SpectrumBundle> {
    SpectrumBundleLoader::new().read_path(path)
}

/// Loads one selected path while anchoring source paths to a common base directory.
///
/// Relative input paths are resolved below `base`; absolute input paths are loaded
/// as provided.
///
/// # Errors
///
/// Returns an error when `base` is missing or is not a directory, the path is
/// unreadable in strict mode, or no readable bundle data is found.
pub fn load_spectra_relative_to(
    base: impl AsRef<Path>,
    path: impl AsRef<Path>,
) -> Result<SpectrumBundle> {
    SpectrumBundleLoader::new().read_path_relative_to(base, path)
}

/// Loads supported spectra from multiple file or directory paths.
///
/// # Errors
///
/// Returns an error when no input paths are provided or no readable bundle data
/// is found.
pub fn load_spectra_many<I, P>(paths: I) -> Result<SpectrumBundle>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_paths(paths)
}

/// Loads selected paths while anchoring source paths to a common base directory.
///
/// Relative input paths are resolved below `base`; absolute input paths are loaded
/// as provided.
///
/// # Errors
///
/// Returns an error when `base` is missing or is not a directory, no input paths
/// are provided, or no readable bundle data is found.
pub fn load_spectra_many_relative_to<I, P>(
    base: impl AsRef<Path>,
    paths: I,
) -> Result<SpectrumBundle>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_paths_relative_to(base, paths)
}

fn spectrum_dimension_counts<'a>(
    spectra: impl IntoIterator<Item = &'a LoadedSpectrum>,
) -> (usize, usize) {
    spectra.into_iter().fold((0, 0), |(one_d, two_d), entry| {
        if entry.is_1d() {
            (one_d + 1, two_d)
        } else {
            (one_d, two_d + 1)
        }
    })
}

fn only_error_from_counts(expected: &'static str, one_d: usize, two_d: usize) -> RSpinError {
    RSpinError::Parse {
        format: "spectrum bundle",
        message: format!(
            "expected exactly one {expected} spectrum, found {one_d} one-dimensional and {two_d} two-dimensional spectra"
        ),
    }
}

fn source_format_filters<I, F>(formats: I) -> Vec<String>
where
    I: IntoIterator<Item = F>,
    F: AsRef<str>,
{
    let mut filters = Vec::new();
    for format in formats {
        push_unique_filter(&mut filters, format.as_ref());
    }
    filters
}

fn source_format_matches(actual: &str, requested: &str) -> bool {
    let actual = actual.trim();
    let requested = requested.trim();
    match (
        LoadedSourceFormat::parse(actual),
        LoadedSourceFormat::parse(requested),
    ) {
        (Ok(actual), Ok(requested)) => actual == requested,
        _ => actual == requested,
    }
}

fn source_format_count_name(format: &str) -> &str {
    match LoadedSourceFormat::parse(format) {
        Ok(format) => format.as_str(),
        Err(_) => format.trim(),
    }
}

fn source_format_candidate_kind(format: &str) -> FileCandidateKind {
    match LoadedSourceFormat::parse(format) {
        Ok(
            LoadedSourceFormat::BrukerFid
            | LoadedSourceFormat::BrukerSer
            | LoadedSourceFormat::AgilentFid,
        ) => FileCandidateKind::Raw,
        Ok(LoadedSourceFormat::BrukerProcessed | LoadedSourceFormat::AgilentProcessed) => {
            FileCandidateKind::Processed
        }
        Ok(_) | Err(_) => FileCandidateKind::Other,
    }
}

fn source_vendor_counts_from_format_counts(
    format_counts: &[SourceFormatCount],
) -> Vec<SourceVendorCount> {
    let mut counts = Vec::new();
    for format_count in format_counts {
        let Some(vendor) = format_count.vendor() else {
            continue;
        };
        push_source_vendor_count(&mut counts, vendor, format_count.count());
    }
    counts
}

fn push_source_vendor_count(
    counts: &mut Vec<SourceVendorCount>,
    vendor: LoadedSourceVendor,
    increment: usize,
) {
    match counts
        .iter_mut()
        .find(|count| count.vendor_kind() == Some(vendor))
    {
        Some(count) => count.count += increment,
        None => counts.push(SourceVendorCount::new(vendor.as_str(), increment)),
    }
}

fn source_vendor_filters<I, V>(vendors: I) -> Vec<String>
where
    I: IntoIterator<Item = V>,
    V: AsRef<str>,
{
    let mut filters = Vec::new();
    for vendor in vendors {
        match LoadedSourceVendor::parse(vendor.as_ref()) {
            Ok(vendor) => {
                for format in vendor.source_formats() {
                    push_unique_filter(&mut filters, format.as_str());
                }
            }
            Err(_) => push_invalid_vendor_filter(&mut filters, vendor.as_ref()),
        }
    }
    filters
}

fn canonical_source_format_filter(format: &str) -> String {
    match LoadedSourceFormat::parse(format) {
        Ok(format) => format.as_str().to_owned(),
        Err(_) => format.trim().to_owned(),
    }
}

fn push_unique_filter(filters: &mut Vec<String>, format: &str) {
    let format = canonical_source_format_filter(format);
    if !filters.iter().any(|existing| existing == &format) {
        filters.push(format);
    }
}

fn push_invalid_vendor_filter(filters: &mut Vec<String>, vendor: &str) {
    let vendor = vendor.trim();
    let filter = format!("source_vendor:{vendor}");
    if !filters.iter().any(|existing| existing == &filter) {
        filters.push(filter);
    }
}

fn no_data_error_at(path: &Path, bundle: &SpectrumBundle) -> RSpinError {
    no_data_error(
        format!("no readable bundle data found at {}", path.display()),
        bundle,
    )
}

fn no_data_error_in_inputs(bundle: &SpectrumBundle) -> RSpinError {
    no_data_error(
        "no readable bundle data found in input paths".to_owned(),
        bundle,
    )
}

fn no_data_error(mut message: String, bundle: &SpectrumBundle) -> RSpinError {
    if let Some(warning) = bundle.warnings.first() {
        message.push_str("; first warning");
        if let Some(path) = warning.path.as_ref() {
            message.push_str(" at ");
            message.push_str(&path.display().to_string());
        }
        message.push_str(": ");
        message.push_str(&warning.message);
        if bundle.warnings.len() > 1 {
            message.push_str("; ");
            message.push_str(&bundle.warnings.len().to_string());
            message.push_str(" total warnings");
        }
    }

    RSpinError::Parse {
        format: "spectrum bundle",
        message,
    }
}

fn fallback_message(
    first_error: Option<&RSpinError>,
    second_error: Option<&RSpinError>,
    bundle_error: &RSpinError,
) -> String {
    let mut parts = Vec::new();
    if let Some(error) = first_error {
        parts.push(error.to_string());
    }
    if let Some(error) = second_error {
        parts.push(format!("two-dimensional fallback: {error}"));
    }
    parts.push(format!("bundle fallback: {bundle_error}"));
    parts.join("; ")
}

fn disabled_dimension_error(path: &Path, dimension: &'static str) -> RSpinError {
    RSpinError::Parse {
        format: "spectrum bundle",
        message: disabled_dimension_message(path, dimension),
    }
}

fn disabled_candidate_message(path: &Path, candidate: &'static str) -> String {
    format!("{candidate} candidates are disabled for {}", path.display())
}

fn disabled_dimension_message(path: &Path, dimension: &'static str) -> String {
    format!(
        "{dimension} spectrum candidates are disabled for {}",
        path.display()
    )
}

#[derive(Default)]
struct PathTree {
    directories: Vec<PathBuf>,
    files: Vec<PathBuf>,
}

fn collect_tree(root: &Path) -> Result<PathTree> {
    let mut tree = PathTree {
        directories: vec![root.to_path_buf()],
        files: Vec::new(),
    };
    collect_children(root, &mut tree)?;
    tree.directories.sort();
    tree.files.sort();
    Ok(tree)
}

fn collect_children(path: &Path, tree: &mut PathTree) -> Result<()> {
    let entries = fs::read_dir(path).map_err(|error| RSpinError::Parse {
        format: "spectrum bundle",
        message: format!("failed to read directory {}: {error}", path.display()),
    })?;

    for entry in entries {
        let entry = entry.map_err(|error| RSpinError::Parse {
            format: "spectrum bundle",
            message: format!(
                "failed to read directory entry below {}: {error}",
                path.display()
            ),
        })?;
        let path = entry.path();
        if path.is_dir() {
            tree.directories.push(path.clone());
            collect_children(&path, tree)?;
        } else if path.is_file() {
            tree.files.push(path);
        }
    }
    Ok(())
}

fn is_bruker_fid_dir(path: &Path) -> bool {
    path.join("fid").is_file() && path.join("acqus").is_file()
}

fn is_bruker_ser_dir(path: &Path) -> bool {
    path.join("ser").is_file() && path.join("acqus").is_file() && path.join("acqu2s").is_file()
}

fn is_bruker_processed_1d_dir(path: &Path) -> bool {
    path.join("procs").is_file() && path.join("1r").is_file()
}

fn is_bruker_processed_2d_dir(path: &Path) -> bool {
    path.join("procs").is_file() && path.join("proc2s").is_file() && path.join("2rr").is_file()
}

fn is_agilent_fid_dir(path: &Path) -> bool {
    path.join("fid").is_file() && path.join("procpar").is_file()
}

fn is_agilent_arrayed_1d_fid_path(path: &Path) -> bool {
    let Some(dataset) = agilent_fid_dataset_dir(path) else {
        return false;
    };

    let Ok(procpar) = fs::read_to_string(dataset.join("procpar")) else {
        return false;
    };
    let Ok(procpar_info) = inspect_agilent_procpar(&procpar) else {
        return false;
    };
    if !matches!(procpar_info.acquisition_dimension, Some(0 | 1)) {
        return false;
    }

    inspect_agilent_binary_file(dataset.join("fid")).is_ok_and(|info| info.trace_count > 1)
}

fn is_agilent_arrayed_2d_fid_path(path: &Path) -> bool {
    let Some(dataset) = agilent_fid_dataset_dir(path) else {
        return false;
    };

    let Ok(procpar) = fs::read_to_string(dataset.join("procpar")) else {
        return false;
    };
    let Ok(procpar_info) = inspect_agilent_procpar(&procpar) else {
        return false;
    };
    if !matches!(procpar_info.acquisition_dimension, Some(2)) {
        return false;
    }
    if procpar_info
        .array_parameter
        .as_deref()
        .is_none_or(|value| !is_agilent_arrayed_2d_series_array(value))
    {
        return false;
    }

    inspect_agilent_binary_file(dataset.join("fid")).is_ok_and(|info| info.trace_count > 1)
}

fn agilent_fid_dataset_dir(path: &Path) -> Option<PathBuf> {
    let dataset = if path.is_file() {
        let file_name = path.file_name().and_then(std::ffi::OsStr::to_str)?;
        if !file_name.eq_ignore_ascii_case("fid") {
            return None;
        }
        path.parent()?.to_path_buf()
    } else {
        path.to_path_buf()
    };

    (dataset.join("fid").is_file() && dataset.join("procpar").is_file()).then_some(dataset)
}

fn is_agilent_processed_dir(path: &Path) -> bool {
    path.join("procpar").is_file()
        && (path.join("phasefile").is_file() || path.join("datdir").join("phasefile").is_file())
}

fn is_standalone_spectrum_file(path: &Path) -> bool {
    path.extension()
        .and_then(std::ffi::OsStr::to_str)
        .is_some_and(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "jdf" | "jdx" | "dx" | "jcamp" | "nmrml" | "xml" | "json" | "csv"
            )
        })
}

fn is_nmredata_file(path: &Path) -> bool {
    path.extension()
        .and_then(std::ffi::OsStr::to_str)
        .is_some_and(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "sdf" | "sd" | "nmredata"
            )
        })
}

fn is_json_file(path: &Path) -> bool {
    path.extension()
        .and_then(std::ffi::OsStr::to_str)
        .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
}

fn file_candidate_kind(path: &Path) -> FileCandidateKind {
    match path
        .file_name()
        .and_then(std::ffi::OsStr::to_str)
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("fid" | "ser") => FileCandidateKind::Raw,
        Some("phasefile" | "1r" | "1i" | "2rr" | "2ri" | "2ir" | "2ii") => {
            FileCandidateKind::Processed
        }
        _ => FileCandidateKind::Other,
    }
}

fn selected_path_candidate_kind(path: &Path) -> FileCandidateKind {
    match file_candidate_kind(path) {
        FileCandidateKind::Raw => FileCandidateKind::Raw,
        FileCandidateKind::Processed => FileCandidateKind::Processed,
        FileCandidateKind::Other if path_looks_raw(path) => FileCandidateKind::Raw,
        FileCandidateKind::Other if path_looks_processed(path) => FileCandidateKind::Processed,
        FileCandidateKind::Other => FileCandidateKind::Other,
    }
}

fn path_looks_raw(path: &Path) -> bool {
    crate::detect_spectrum1d_path_format(path).is_ok_and(|format| {
        matches!(
            format,
            Spectrum1DPathFormat::BrukerFid | Spectrum1DPathFormat::AgilentFid
        )
    }) || crate::detect_spectrum2d_path_format(path).is_ok_and(|format| {
        matches!(
            format,
            Spectrum2DPathFormat::BrukerSer | Spectrum2DPathFormat::AgilentFid
        )
    })
}

fn path_looks_processed(path: &Path) -> bool {
    crate::detect_spectrum1d_path_format(path).is_ok_and(|format| {
        matches!(
            format,
            Spectrum1DPathFormat::BrukerProcessed | Spectrum1DPathFormat::AgilentProcessed
        )
    }) || crate::detect_spectrum2d_path_format(path).is_ok_and(|format| {
        matches!(
            format,
            Spectrum2DPathFormat::BrukerProcessed | Spectrum2DPathFormat::AgilentProcessed
        )
    })
}

fn format_from_file(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .map(str::to_ascii_lowercase)
    {
        Some(extension) if extension == "jdf" => "jeol_jdf",
        Some(extension) if matches!(extension.as_str(), "jdx" | "dx" | "jcamp") => "jcamp_dx",
        Some(extension) if matches!(extension.as_str(), "nmrml" | "xml") => "nmrml",
        Some(extension) if extension == "json" => "json",
        Some(extension) if extension == "csv" => "csv",
        _ => "auto",
    }
}

fn is_agilent_format(format: &str) -> bool {
    matches!(format, "agilent_fid" | "agilent_processed")
}

fn source_format_1d(path: &Path, fallback: &'static str) -> &'static str {
    if fallback != "auto" {
        return fallback;
    }

    match crate::detect_spectrum1d_path_format(path) {
        Ok(format) => format.as_str(),
        Err(_) => fallback,
    }
}

fn source_format_2d(path: &Path, fallback: &'static str) -> &'static str {
    if fallback != "auto" {
        return fallback;
    }

    match crate::detect_spectrum2d_path_format(path) {
        Ok(format) => format.as_str(),
        Err(_) => fallback,
    }
}

fn selected_path_from_base(base: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    }
}

fn relative_source_path(root: &Path, path: &Path) -> Option<PathBuf> {
    if root.is_file() {
        return file_name_path(path);
    }

    match path.strip_prefix(root) {
        Ok(relative) if !relative.as_os_str().is_empty() => Some(relative.to_path_buf()),
        _ => file_name_path(path),
    }
}

fn file_name_path(path: &Path) -> Option<PathBuf> {
    path.file_name().map(PathBuf::from)
}

fn clear_bundle_source_paths(bundle: &mut SpectrumBundle) {
    for loaded in &mut bundle.spectra {
        match loaded {
            LoadedSpectrum::OneD { source, .. } | LoadedSpectrum::TwoD { source, .. } => {
                source.path = None;
            }
        }
    }
    for warning in &mut bundle.warnings {
        warning.path = None;
    }
}

fn prefix_bundle_source_paths(bundle: &mut SpectrumBundle, container_path: &Path) {
    for loaded in &mut bundle.spectra {
        match loaded {
            LoadedSpectrum::OneD { source, .. } | LoadedSpectrum::TwoD { source, .. } => {
                source.path = Some(nested_source_path(container_path, source.path.take()));
            }
        }
    }
    for warning in &mut bundle.warnings {
        warning.path = Some(nested_source_path(container_path, warning.path.take()));
    }
}

fn nested_source_path(container_path: &Path, source_path: Option<PathBuf>) -> PathBuf {
    match source_path {
        Some(path) if !path.as_os_str().is_empty() => container_path.join(path),
        Some(_) | None => container_path.to_path_buf(),
    }
}

fn nmredata_record_molecule(
    root: &Path,
    path: &Path,
    record_index: usize,
    record: &NmreDataRecord,
) -> Option<Molecule> {
    let formula = record.formula.as_ref()?;
    let id = nmredata_molecule_id(root, path, record_index);

    match Molecule::from_formula(id.clone(), formula.clone()) {
        Ok(molecule) => Some(molecule),
        Err(_) => Some(Molecule::new(id).with_formula(formula.clone())),
    }
}

fn nmredata_molecule_id(root: &Path, path: &Path, record_index: usize) -> String {
    let source = match relative_source_path(root, path) {
        Some(path) => path.to_string_lossy().replace('\\', "/"),
        None => "record".to_owned(),
    };
    format!("nmredata:{source}:{}", record_index + 1)
}
