//! Unified spectrum bundle loading.

use std::{
    fs,
    path::{Path, PathBuf},
};

use rspin_core::{Molecule, RSpinError, Result, Spectrum1D, Spectrum2D};
use serde::{Deserialize, Serialize};

use crate::{
    NmreDataRecord, read_agilent_fid_1d_dir, read_agilent_fid_2d_dir,
    read_agilent_processed_1d_dir, read_agilent_processed_2d_dir, read_bruker_fid_1d_dir,
    read_bruker_processed_1d_dir, read_bruker_processed_2d_dir, read_bruker_ser_2d_dir,
    read_nmredata_records_file, read_spectrum_bundle_json_file, read_spectrum1d_path,
    read_spectrum2d_path,
};

/// High-level reader for supported `RSpin` spectrum inputs.
pub type RSpinReader = SpectrumBundleLoader;

/// Source metadata for a loaded spectrum.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoadedSource {
    /// Relative source path when source tracking is enabled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
    /// Reader format used for this source.
    pub format: String,
}

impl LoadedSource {
    /// Creates source metadata.
    #[must_use]
    pub fn new(path: Option<PathBuf>, format: impl Into<String>) -> Self {
        Self {
            path,
            format: format.into(),
        }
    }
}

/// A loaded one- or two-dimensional spectrum plus source metadata.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "dimension", rename_all = "snake_case")]
pub enum LoadedSpectrum {
    /// One-dimensional spectrum.
    OneD {
        /// Spectrum payload.
        spectrum: Spectrum1D,
        /// Source metadata.
        source: LoadedSource,
    },
    /// Two-dimensional spectrum.
    TwoD {
        /// Spectrum payload.
        spectrum: Spectrum2D,
        /// Source metadata.
        source: LoadedSource,
    },
}

impl LoadedSpectrum {
    /// Returns true when this entry contains a one-dimensional spectrum.
    #[must_use]
    pub fn is_1d(&self) -> bool {
        matches!(self, Self::OneD { .. })
    }

    /// Returns true when this entry contains a two-dimensional spectrum.
    #[must_use]
    pub fn is_2d(&self) -> bool {
        matches!(self, Self::TwoD { .. })
    }

    /// Returns the source metadata.
    #[must_use]
    pub fn source(&self) -> &LoadedSource {
        match self {
            Self::OneD { source, .. } | Self::TwoD { source, .. } => source,
        }
    }

    /// Returns the one-dimensional spectrum, if present.
    #[must_use]
    pub fn as_1d(&self) -> Option<&Spectrum1D> {
        match self {
            Self::OneD { spectrum, .. } => Some(spectrum),
            Self::TwoD { .. } => None,
        }
    }

    /// Returns the two-dimensional spectrum, if present.
    #[must_use]
    pub fn as_2d(&self) -> Option<&Spectrum2D> {
        match self {
            Self::TwoD { spectrum, .. } => Some(spectrum),
            Self::OneD { .. } => None,
        }
    }

    /// Consumes this entry and returns the one-dimensional spectrum, if present.
    #[must_use]
    pub fn into_1d(self) -> Option<Spectrum1D> {
        match self {
            Self::OneD { spectrum, .. } => Some(spectrum),
            Self::TwoD { .. } => None,
        }
    }

    /// Consumes this entry and returns the two-dimensional spectrum, if present.
    #[must_use]
    pub fn into_2d(self) -> Option<Spectrum2D> {
        match self {
            Self::TwoD { spectrum, .. } => Some(spectrum),
            Self::OneD { .. } => None,
        }
    }
}

/// Non-fatal load warning emitted by the bundle loader.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoadWarning {
    /// Relative source path when source tracking is enabled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
    /// Human-readable warning message.
    pub message: String,
}

impl LoadWarning {
    fn new(path: Option<PathBuf>, message: impl Into<String>) -> Self {
        Self {
            path,
            message: message.into(),
        }
    }
}

/// Loaded spectra, molecules, and non-fatal warnings.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SpectrumBundle {
    spectra: Vec<LoadedSpectrum>,
    molecules: Vec<Molecule>,
    warnings: Vec<LoadWarning>,
}

impl SpectrumBundle {
    /// Creates an empty bundle.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns all loaded spectrum entries.
    #[must_use]
    pub fn spectra(&self) -> &[LoadedSpectrum] {
        &self.spectra
    }

    /// Returns all loaded molecules.
    #[must_use]
    pub fn molecules(&self) -> &[Molecule] {
        &self.molecules
    }

    /// Returns non-fatal loader warnings.
    #[must_use]
    pub fn warnings(&self) -> &[LoadWarning] {
        &self.warnings
    }

    /// Returns an iterator over one-dimensional spectra.
    pub fn spectra_1d(&self) -> impl Iterator<Item = &Spectrum1D> {
        self.spectra.iter().filter_map(LoadedSpectrum::as_1d)
    }

    /// Returns an iterator over two-dimensional spectra.
    pub fn spectra_2d(&self) -> impl Iterator<Item = &Spectrum2D> {
        self.spectra.iter().filter_map(LoadedSpectrum::as_2d)
    }

    /// Returns an iterator over one-dimensional spectra and their sources.
    pub fn loaded_1d(&self) -> impl Iterator<Item = (&Spectrum1D, &LoadedSource)> {
        self.spectra.iter().filter_map(|entry| match entry {
            LoadedSpectrum::OneD { spectrum, source } => Some((spectrum, source)),
            LoadedSpectrum::TwoD { .. } => None,
        })
    }

    /// Returns an iterator over two-dimensional spectra and their sources.
    pub fn loaded_2d(&self) -> impl Iterator<Item = (&Spectrum2D, &LoadedSource)> {
        self.spectra.iter().filter_map(|entry| match entry {
            LoadedSpectrum::TwoD { spectrum, source } => Some((spectrum, source)),
            LoadedSpectrum::OneD { .. } => None,
        })
    }

    /// Returns the only loaded one-dimensional spectrum.
    ///
    /// # Errors
    ///
    /// Returns an error unless the bundle contains exactly one spectrum and it
    /// is one-dimensional.
    pub fn only_1d(&self) -> Result<&Spectrum1D> {
        match self.spectra.as_slice() {
            [LoadedSpectrum::OneD { spectrum, .. }] => Ok(spectrum),
            _ => Err(self.only_error("one-dimensional")),
        }
    }

    /// Returns the only loaded two-dimensional spectrum.
    ///
    /// # Errors
    ///
    /// Returns an error unless the bundle contains exactly one spectrum and it
    /// is two-dimensional.
    pub fn only_2d(&self) -> Result<&Spectrum2D> {
        match self.spectra.as_slice() {
            [LoadedSpectrum::TwoD { spectrum, .. }] => Ok(spectrum),
            _ => Err(self.only_error("two-dimensional")),
        }
    }

    /// Consumes the bundle and returns the only loaded one-dimensional spectrum.
    ///
    /// # Errors
    ///
    /// Returns an error unless the bundle contains exactly one spectrum and it
    /// is one-dimensional.
    pub fn into_only_1d(self) -> Result<Spectrum1D> {
        let (one_d, two_d) = spectrum_dimension_counts(self.spectra.iter());
        if self.spectra.len() != 1 {
            return Err(only_error_from_counts("one-dimensional", one_d, two_d));
        }

        match self.spectra.into_iter().next() {
            Some(LoadedSpectrum::OneD { spectrum, .. }) => Ok(spectrum),
            Some(LoadedSpectrum::TwoD { .. }) | None => {
                Err(only_error_from_counts("one-dimensional", one_d, two_d))
            }
        }
    }

    /// Consumes the bundle and returns the only loaded two-dimensional spectrum.
    ///
    /// # Errors
    ///
    /// Returns an error unless the bundle contains exactly one spectrum and it
    /// is two-dimensional.
    pub fn into_only_2d(self) -> Result<Spectrum2D> {
        let (one_d, two_d) = spectrum_dimension_counts(self.spectra.iter());
        if self.spectra.len() != 1 {
            return Err(only_error_from_counts("two-dimensional", one_d, two_d));
        }

        match self.spectra.into_iter().next() {
            Some(LoadedSpectrum::TwoD { spectrum, .. }) => Ok(spectrum),
            Some(LoadedSpectrum::OneD { .. }) | None => {
                Err(only_error_from_counts("two-dimensional", one_d, two_d))
            }
        }
    }

    /// Consumes the bundle and returns all loaded pieces.
    #[must_use]
    pub fn into_parts(self) -> (Vec<LoadedSpectrum>, Vec<Molecule>, Vec<LoadWarning>) {
        (self.spectra, self.molecules, self.warnings)
    }

    /// Returns the number of loaded spectra.
    #[must_use]
    pub fn len(&self) -> usize {
        self.spectra.len()
    }

    /// Returns true when no spectra or molecules were loaded.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        !self.has_data()
    }

    fn push_1d(&mut self, spectrum: Spectrum1D, source: LoadedSource) {
        self.spectra.push(LoadedSpectrum::OneD { spectrum, source });
    }

    fn push_2d(&mut self, spectrum: Spectrum2D, source: LoadedSource) {
        self.spectra.push(LoadedSpectrum::TwoD { spectrum, source });
    }

    fn push_molecule(&mut self, molecule: Molecule) {
        self.molecules.push(molecule);
    }

    fn extend_bundle(&mut self, bundle: SpectrumBundle) {
        self.spectra.extend(bundle.spectra);
        self.molecules.extend(bundle.molecules);
        self.warnings.extend(bundle.warnings);
    }

    fn push_warning(&mut self, warning: LoadWarning) {
        self.warnings.push(warning);
    }

    fn has_data(&self) -> bool {
        !self.spectra.is_empty() || !self.molecules.is_empty()
    }

    fn only_error(&self, expected: &'static str) -> RSpinError {
        let (one_d, two_d) = spectrum_dimension_counts(self.spectra.iter());
        only_error_from_counts(expected, one_d, two_d)
    }
}

/// Chainable options for loading all recognizable spectra from a path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpectrumBundleLoader {
    raw: Toggle,
    processed: Toggle,
    strict: Toggle,
    source_paths: Toggle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Toggle {
    Enabled,
    Disabled,
}

impl Toggle {
    fn from_bool(enabled: bool) -> Self {
        if enabled {
            Self::Enabled
        } else {
            Self::Disabled
        }
    }

    fn is_enabled(self) -> bool {
        matches!(self, Self::Enabled)
    }
}

impl SpectrumBundleLoader {
    /// Creates a loader with raw and processed spectra enabled.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Enables or disables raw spectrum candidates.
    #[must_use]
    pub fn with_raw(mut self, enabled: bool) -> Self {
        self.raw = Toggle::from_bool(enabled);
        self
    }

    /// Enables or disables processed spectrum candidates.
    #[must_use]
    pub fn with_processed(mut self, enabled: bool) -> Self {
        self.processed = Toggle::from_bool(enabled);
        self
    }

    /// Enables strict mode. In strict mode, the first failed candidate aborts loading.
    #[must_use]
    pub fn with_strict(mut self, enabled: bool) -> Self {
        self.strict = Toggle::from_bool(enabled);
        self
    }

    /// Enables relative source paths in spectra and warnings.
    #[must_use]
    pub fn with_source_paths(mut self, enabled: bool) -> Self {
        self.source_paths = Toggle::from_bool(enabled);
        self
    }

    /// Loads all supported spectra from a file or directory path.
    ///
    /// # Errors
    ///
    /// Returns an error when the path is missing, strict mode rejects a
    /// candidate, or no readable bundle data is found.
    pub fn read_path(&self, path: impl AsRef<Path>) -> Result<SpectrumBundle> {
        let root = path.as_ref();
        if !root.exists() {
            return Err(RSpinError::Parse {
                format: "spectrum bundle",
                message: format!("{} does not exist", root.display()),
            });
        }

        let mut bundle = SpectrumBundle::new();
        self.read_existing_path_into(root, &mut bundle)?;

        if bundle.has_data() {
            Ok(bundle)
        } else {
            Err(RSpinError::Parse {
                format: "spectrum bundle",
                message: format!("no readable bundle data found at {}", root.display()),
            })
        }
    }

    /// Loads supported spectra from multiple file or directory paths.
    ///
    /// Non-strict mode records unreadable input paths as warnings and continues
    /// loading later paths. Strict mode aborts at the first unreadable path.
    ///
    /// # Errors
    ///
    /// Returns an error when no input paths are provided, strict mode rejects a
    /// path, or no readable bundle data is found.
    pub fn read_paths<I, P>(&self, paths: I) -> Result<SpectrumBundle>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        let mut bundle = SpectrumBundle::new();
        let mut saw_input = false;

        for path in paths {
            saw_input = true;
            let path = path.as_ref();
            if !path.exists() {
                self.handle_error(
                    &mut bundle,
                    path,
                    path,
                    RSpinError::Parse {
                        format: "spectrum bundle",
                        message: format!("{} does not exist", path.display()),
                    },
                )?;
                continue;
            }

            let data_before = bundle.spectra.len() + bundle.molecules.len();
            let warnings_before = bundle.warnings.len();
            self.read_existing_path_into(path, &mut bundle)?;
            let data_after = bundle.spectra.len() + bundle.molecules.len();
            let warnings_after = bundle.warnings.len();
            if data_after == data_before && warnings_after == warnings_before {
                self.handle_error_message(
                    &mut bundle,
                    path,
                    path,
                    format!("no readable bundle data found at {}", path.display()),
                )?;
            }
        }

        if !saw_input {
            return Err(RSpinError::Parse {
                format: "spectrum bundle",
                message: "no input paths provided".to_owned(),
            });
        }

        if bundle.has_data() {
            Ok(bundle)
        } else {
            Err(RSpinError::Parse {
                format: "spectrum bundle",
                message: "no readable bundle data found in input paths".to_owned(),
            })
        }
    }

    /// Loads exactly one one-dimensional spectrum from a file or directory path.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the path does not resolve to
    /// exactly one one-dimensional spectrum.
    pub fn read_1d(&self, path: impl AsRef<Path>) -> Result<Spectrum1D> {
        self.read_path(path)?.into_only_1d()
    }

    /// Loads exactly one two-dimensional spectrum from a file or directory path.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the path does not resolve to
    /// exactly one two-dimensional spectrum.
    pub fn read_2d(&self, path: impl AsRef<Path>) -> Result<Spectrum2D> {
        self.read_path(path)?.into_only_2d()
    }

    fn read_existing_path_into(&self, root: &Path, bundle: &mut SpectrumBundle) -> Result<()> {
        if root.is_dir() {
            self.read_directory(root, bundle)
        } else {
            self.read_file_candidate(root, root, bundle)
        }
    }

    fn read_directory(&self, root: &Path, bundle: &mut SpectrumBundle) -> Result<()> {
        let tree = collect_tree(root)?;
        for directory in &tree.directories {
            self.read_directory_candidate(root, directory, bundle)?;
        }
        for file in &tree.files {
            if is_nmredata_file(file) {
                self.read_nmredata_candidate(root, file, bundle)?;
            } else if is_standalone_spectrum_file(file) {
                self.read_file_candidate(root, file, bundle)?;
            }
        }
        Ok(())
    }

    fn read_directory_candidate(
        &self,
        root: &Path,
        directory: &Path,
        bundle: &mut SpectrumBundle,
    ) -> Result<()> {
        if self.raw.is_enabled() && is_bruker_ser_dir(directory) {
            self.add_2d_result(
                bundle,
                root,
                directory,
                "bruker_ser",
                read_bruker_ser_2d_dir(directory),
            )?;
        }
        if self.raw.is_enabled() && is_bruker_fid_dir(directory) {
            self.add_1d_result(
                bundle,
                root,
                directory,
                "bruker_fid",
                read_bruker_fid_1d_dir(directory),
            )?;
        }
        if self.processed.is_enabled() && is_bruker_processed_2d_dir(directory) {
            self.add_2d_result(
                bundle,
                root,
                directory,
                "bruker_processed",
                read_bruker_processed_2d_dir(directory),
            )?;
        }
        if self.processed.is_enabled() && is_bruker_processed_1d_dir(directory) {
            self.add_1d_result(
                bundle,
                root,
                directory,
                "bruker_processed",
                read_bruker_processed_1d_dir(directory),
            )?;
        }
        if self.raw.is_enabled() && is_agilent_fid_dir(directory) {
            self.add_1d_or_2d_result(
                bundle,
                root,
                directory,
                "agilent_fid",
                || read_agilent_fid_1d_dir(directory),
                || read_agilent_fid_2d_dir(directory),
            )?;
        }
        if self.processed.is_enabled() && is_agilent_processed_dir(directory) {
            self.add_1d_or_2d_result(
                bundle,
                root,
                directory,
                "agilent_processed",
                || read_agilent_processed_1d_dir(directory),
                || read_agilent_processed_2d_dir(directory),
            )?;
        }
        Ok(())
    }

    fn read_file_candidate(
        &self,
        root: &Path,
        file: &Path,
        bundle: &mut SpectrumBundle,
    ) -> Result<()> {
        if is_nmredata_file(file) {
            return self.read_nmredata_candidate(root, file, bundle);
        }
        if is_json_file(file) {
            return self.add_1d_or_2d_or_bundle_result(
                bundle,
                root,
                file,
                || read_spectrum1d_path(file),
                || read_spectrum2d_path(file),
                || read_spectrum_bundle_json_file(file),
            );
        }

        self.add_1d_or_2d_result(
            bundle,
            root,
            file,
            format_from_file(file),
            || read_spectrum1d_path(file),
            || read_spectrum2d_path(file),
        )
    }

    fn add_1d_or_2d_or_bundle_result(
        &self,
        bundle: &mut SpectrumBundle,
        root: &Path,
        path: &Path,
        read_1d: impl FnOnce() -> Result<Spectrum1D>,
        read_2d: impl FnOnce() -> Result<Spectrum2D>,
        read_bundle: impl FnOnce() -> Result<SpectrumBundle>,
    ) -> Result<()> {
        let format = format_from_file(path);
        match read_1d() {
            Ok(spectrum) => {
                bundle.push_1d(spectrum, self.loaded_source(root, path, format));
                Ok(())
            }
            Err(first_error) => match read_2d() {
                Ok(spectrum) => {
                    bundle.push_2d(spectrum, self.loaded_source(root, path, format));
                    Ok(())
                }
                Err(second_error) => match read_bundle() {
                    Ok(loaded) => {
                        bundle.extend_bundle(loaded);
                        Ok(())
                    }
                    Err(third_error) => {
                        let message = format!(
                            "{first_error}; two-dimensional fallback: {second_error}; bundle fallback: {third_error}"
                        );
                        self.handle_error_message(bundle, root, path, message)
                    }
                },
            },
        }
    }

    fn read_nmredata_candidate(
        &self,
        root: &Path,
        file: &Path,
        bundle: &mut SpectrumBundle,
    ) -> Result<()> {
        match read_nmredata_records_file(file) {
            Ok(records) => {
                for (record_index, record) in records.iter().enumerate() {
                    if let Some(molecule) =
                        nmredata_record_molecule(root, file, record_index, record)
                    {
                        bundle.push_molecule(molecule);
                    }
                }
                Ok(())
            }
            Err(error) => self.handle_error(bundle, root, file, error),
        }
    }

    fn add_1d_result(
        &self,
        bundle: &mut SpectrumBundle,
        root: &Path,
        path: &Path,
        format: &'static str,
        result: Result<Spectrum1D>,
    ) -> Result<()> {
        match result {
            Ok(spectrum) => {
                bundle.push_1d(spectrum, self.loaded_source(root, path, format));
                Ok(())
            }
            Err(error) => self.handle_error(bundle, root, path, error),
        }
    }

    fn add_2d_result(
        &self,
        bundle: &mut SpectrumBundle,
        root: &Path,
        path: &Path,
        format: &'static str,
        result: Result<Spectrum2D>,
    ) -> Result<()> {
        match result {
            Ok(spectrum) => {
                bundle.push_2d(spectrum, self.loaded_source(root, path, format));
                Ok(())
            }
            Err(error) => self.handle_error(bundle, root, path, error),
        }
    }

    fn add_1d_or_2d_result(
        &self,
        bundle: &mut SpectrumBundle,
        root: &Path,
        path: &Path,
        format: &'static str,
        read_1d: impl FnOnce() -> Result<Spectrum1D>,
        read_2d: impl FnOnce() -> Result<Spectrum2D>,
    ) -> Result<()> {
        match read_1d() {
            Ok(spectrum) => {
                bundle.push_1d(spectrum, self.loaded_source(root, path, format));
                Ok(())
            }
            Err(first_error) => match read_2d() {
                Ok(spectrum) => {
                    bundle.push_2d(spectrum, self.loaded_source(root, path, format));
                    Ok(())
                }
                Err(second_error) => {
                    let message =
                        format!("{first_error}; two-dimensional fallback: {second_error}");
                    self.handle_error_message(bundle, root, path, message)
                }
            },
        }
    }

    fn handle_error(
        &self,
        bundle: &mut SpectrumBundle,
        root: &Path,
        path: &Path,
        error: RSpinError,
    ) -> Result<()> {
        if self.strict.is_enabled() {
            Err(error)
        } else {
            self.handle_error_message(bundle, root, path, error.to_string())
        }
    }

    fn handle_error_message(
        &self,
        bundle: &mut SpectrumBundle,
        root: &Path,
        path: &Path,
        message: String,
    ) -> Result<()> {
        if self.strict.is_enabled() {
            Err(RSpinError::Parse {
                format: "spectrum bundle",
                message,
            })
        } else {
            bundle.push_warning(LoadWarning::new(self.source_path(root, path), message));
            Ok(())
        }
    }

    fn loaded_source(&self, root: &Path, path: &Path, format: impl Into<String>) -> LoadedSource {
        LoadedSource::new(self.source_path(root, path), format)
    }

    fn source_path(&self, root: &Path, path: &Path) -> Option<PathBuf> {
        if !self.source_paths.is_enabled() {
            return None;
        }
        relative_source_path(root, path)
    }
}

impl Default for SpectrumBundleLoader {
    fn default() -> Self {
        Self {
            raw: Toggle::Enabled,
            processed: Toggle::Enabled,
            strict: Toggle::Disabled,
            source_paths: Toggle::Enabled,
        }
    }
}

/// Loads all supported spectrum bundle data from a file or directory path.
///
/// # Errors
///
/// Returns an error when the path is missing or no readable bundle data is found.
pub fn load_spectra(path: impl AsRef<Path>) -> Result<SpectrumBundle> {
    SpectrumBundleLoader::new().read_path(path)
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

/// Loads exactly one one-dimensional spectrum from a file or directory path.
///
/// # Errors
///
/// Returns an error when loading fails or the path does not resolve to exactly
/// one one-dimensional spectrum.
pub fn load_spectrum_1d(path: impl AsRef<Path>) -> Result<Spectrum1D> {
    SpectrumBundleLoader::new().read_1d(path)
}

/// Loads exactly one two-dimensional spectrum from a file or directory path.
///
/// # Errors
///
/// Returns an error when loading fails or the path does not resolve to exactly
/// one two-dimensional spectrum.
pub fn load_spectrum_2d(path: impl AsRef<Path>) -> Result<Spectrum2D> {
    SpectrumBundleLoader::new().read_2d(path)
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
