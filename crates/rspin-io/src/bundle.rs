//! Unified spectrum bundle loading.

use std::{
    fs,
    path::{Path, PathBuf},
};

use rspin_core::{Molecule, RSpinError, Result, Spectrum1D, Spectrum2D};
use serde::{Deserialize, Serialize};

use crate::agilent::is_agilent_arrayed_2d_series_array;
use crate::{
    NmreDataRecord, Spectrum1DPathFormat, Spectrum2DPathFormat, SpectrumPathReader,
    inspect_agilent_binary_file, inspect_agilent_procpar, read_agilent_arrayed_fid_1d_dir,
    read_agilent_arrayed_fid_2d_dir, read_agilent_fid_1d_dir, read_agilent_fid_2d_dir,
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
    /// Creates a non-fatal load warning.
    #[must_use]
    pub fn new(path: Option<PathBuf>, message: impl Into<String>) -> Self {
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

    /// Adds a one-dimensional spectrum with source metadata and returns the bundle.
    #[must_use]
    pub fn with_1d(mut self, spectrum: Spectrum1D, source: LoadedSource) -> Self {
        self.push_1d(spectrum, source);
        self
    }

    /// Adds a two-dimensional spectrum with source metadata and returns the bundle.
    #[must_use]
    pub fn with_2d(mut self, spectrum: Spectrum2D, source: LoadedSource) -> Self {
        self.push_2d(spectrum, source);
        self
    }

    /// Adds molecule metadata and returns the bundle.
    #[must_use]
    pub fn with_molecule(mut self, molecule: Molecule) -> Self {
        self.push_molecule(molecule);
        self
    }

    /// Adds a non-fatal load warning and returns the bundle.
    #[must_use]
    pub fn with_warning(mut self, warning: LoadWarning) -> Self {
        self.push_warning(warning);
        self
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

    /// Returns a loaded spectrum by its source path, if present.
    #[must_use]
    pub fn loaded_by_source_path(&self, path: impl AsRef<Path>) -> Option<&LoadedSpectrum> {
        let path = path.as_ref();
        self.spectra
            .iter()
            .find(|entry| entry.source().path.as_deref() == Some(path))
    }

    /// Returns a one-dimensional spectrum and source by source path, if present.
    #[must_use]
    pub fn loaded_1d_by_source_path(
        &self,
        path: impl AsRef<Path>,
    ) -> Option<(&Spectrum1D, &LoadedSource)> {
        let path = path.as_ref();
        self.loaded_1d()
            .find(|(_, source)| source.path.as_deref() == Some(path))
    }

    /// Returns a two-dimensional spectrum and source by source path, if present.
    #[must_use]
    pub fn loaded_2d_by_source_path(
        &self,
        path: impl AsRef<Path>,
    ) -> Option<(&Spectrum2D, &LoadedSource)> {
        let path = path.as_ref();
        self.loaded_2d()
            .find(|(_, source)| source.path.as_deref() == Some(path))
    }

    /// Returns warnings associated with a source path.
    pub fn warnings_for_source_path(
        &self,
        path: impl AsRef<Path>,
    ) -> impl Iterator<Item = &LoadWarning> + '_ {
        let path = path.as_ref().to_path_buf();
        self.warnings
            .iter()
            .filter(move |warning| warning.path.as_deref() == Some(path.as_path()))
    }

    /// Consumes the bundle and returns loaded one-dimensional spectra with sources.
    #[must_use]
    pub fn into_loaded_1d(self) -> Vec<(Spectrum1D, LoadedSource)> {
        self.spectra
            .into_iter()
            .filter_map(|entry| match entry {
                LoadedSpectrum::OneD { spectrum, source } => Some((spectrum, source)),
                LoadedSpectrum::TwoD { .. } => None,
            })
            .collect()
    }

    /// Consumes the bundle and returns loaded two-dimensional spectra with sources.
    #[must_use]
    pub fn into_loaded_2d(self) -> Vec<(Spectrum2D, LoadedSource)> {
        self.spectra
            .into_iter()
            .filter_map(|entry| match entry {
                LoadedSpectrum::TwoD { spectrum, source } => Some((spectrum, source)),
                LoadedSpectrum::OneD { .. } => None,
            })
            .collect()
    }

    /// Consumes the bundle and returns one-dimensional spectra without source metadata.
    #[must_use]
    pub fn into_spectra_1d(self) -> Vec<Spectrum1D> {
        self.into_loaded_1d()
            .into_iter()
            .map(|(spectrum, _)| spectrum)
            .collect()
    }

    /// Consumes the bundle and returns two-dimensional spectra without source metadata.
    #[must_use]
    pub fn into_spectra_2d(self) -> Vec<Spectrum2D> {
        self.into_loaded_2d()
            .into_iter()
            .map(|(spectrum, _)| spectrum)
            .collect()
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

    /// Returns the number of one-dimensional spectra in the bundle.
    #[must_use]
    pub fn len_1d(&self) -> usize {
        self.spectra.iter().filter(|entry| entry.is_1d()).count()
    }

    /// Returns the number of two-dimensional spectra in the bundle.
    #[must_use]
    pub fn len_2d(&self) -> usize {
        self.spectra.iter().filter(|entry| entry.is_2d()).count()
    }

    /// Returns the number of molecule metadata entries in the bundle.
    #[must_use]
    pub fn molecule_count(&self) -> usize {
        self.molecules.len()
    }

    /// Returns the number of non-fatal loader warnings in the bundle.
    #[must_use]
    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }

    /// Returns true when the bundle contains non-fatal loader warnings.
    #[must_use]
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Returns true when no spectra or molecules were loaded.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        !self.has_data()
    }

    /// Adds a one-dimensional spectrum with source metadata.
    pub fn push_1d(&mut self, spectrum: Spectrum1D, source: LoadedSource) {
        self.spectra.push(LoadedSpectrum::OneD { spectrum, source });
    }

    /// Adds a two-dimensional spectrum with source metadata.
    pub fn push_2d(&mut self, spectrum: Spectrum2D, source: LoadedSource) {
        self.spectra.push(LoadedSpectrum::TwoD { spectrum, source });
    }

    /// Adds molecule metadata.
    pub fn push_molecule(&mut self, molecule: Molecule) {
        self.molecules.push(molecule);
    }

    fn extend_bundle(&mut self, bundle: SpectrumBundle) {
        self.spectra.extend(bundle.spectra);
        self.molecules.extend(bundle.molecules);
        self.warnings.extend(bundle.warnings);
    }

    /// Adds a non-fatal load warning.
    pub fn push_warning(&mut self, warning: LoadWarning) {
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
    one_d: Toggle,
    two_d: Toggle,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FileCandidateKind {
    Raw,
    Processed,
    Other,
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

    /// Enables or disables one-dimensional spectrum candidates.
    #[must_use]
    pub fn with_1d(mut self, enabled: bool) -> Self {
        self.one_d = Toggle::from_bool(enabled);
        self
    }

    /// Enables or disables two-dimensional spectrum candidates.
    #[must_use]
    pub fn with_2d(mut self, enabled: bool) -> Self {
        self.two_d = Toggle::from_bool(enabled);
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

    /// Enables raw candidates and disables processed candidates.
    #[must_use]
    pub fn raw_only(mut self) -> Self {
        self.raw = Toggle::Enabled;
        self.processed = Toggle::Disabled;
        self
    }

    /// Enables processed candidates and disables raw candidates.
    #[must_use]
    pub fn processed_only(mut self) -> Self {
        self.raw = Toggle::Disabled;
        self.processed = Toggle::Enabled;
        self
    }

    /// Enables one-dimensional candidates and disables two-dimensional candidates.
    #[must_use]
    pub fn one_d_only(mut self) -> Self {
        self.one_d = Toggle::Enabled;
        self.two_d = Toggle::Disabled;
        self
    }

    /// Enables two-dimensional candidates and disables one-dimensional candidates.
    #[must_use]
    pub fn two_d_only(mut self) -> Self {
        self.one_d = Toggle::Disabled;
        self.two_d = Toggle::Enabled;
        self
    }

    /// Enables strict mode.
    #[must_use]
    pub fn strict(mut self) -> Self {
        self.strict = Toggle::Enabled;
        self
    }

    /// Disables source paths in loaded spectra and warnings.
    #[must_use]
    pub fn without_source_paths(mut self) -> Self {
        self.source_paths = Toggle::Disabled;
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
        self.read_existing_path_into(root, root, &mut bundle)?;
        self.add_selected_path_disabled_warning(root, &mut bundle)?;

        if bundle.has_data() {
            Ok(bundle)
        } else {
            Err(no_data_error_at(root, &bundle))
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
            self.read_existing_path_into(path, path, &mut bundle)?;
            let data_after = bundle.spectra.len() + bundle.molecules.len();
            let warnings_after = bundle.warnings.len();
            if data_after == data_before && warnings_after == warnings_before {
                let message = match self.disabled_selected_path_message(path) {
                    Some(message) => message,
                    None => format!("no readable bundle data found at {}", path.display()),
                };
                self.handle_error_message(&mut bundle, path, path, message)?;
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
            Err(no_data_error_in_inputs(&bundle))
        }
    }

    /// Loads selected paths while anchoring source paths to a common base directory.
    ///
    /// Relative input paths are resolved below `base`. Absolute input paths are
    /// loaded as provided, and their source metadata is still expressed relative
    /// to `base` when possible.
    ///
    /// # Errors
    ///
    /// Returns an error when `base` is missing or is not a directory, no input
    /// paths are provided, strict mode rejects a path, or no readable bundle
    /// data is found.
    pub fn read_paths_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
    ) -> Result<SpectrumBundle>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        let base = base.as_ref();
        if !base.exists() {
            return Err(RSpinError::Parse {
                format: "spectrum bundle",
                message: format!("{} does not exist", base.display()),
            });
        }
        if !base.is_dir() {
            return Err(RSpinError::Parse {
                format: "spectrum bundle",
                message: format!("{} is not a directory", base.display()),
            });
        }

        let mut bundle = SpectrumBundle::new();
        let mut saw_input = false;

        for path in paths {
            saw_input = true;
            let path = selected_path_from_base(base, path.as_ref());
            if !path.exists() {
                self.handle_error(
                    &mut bundle,
                    base,
                    &path,
                    RSpinError::Parse {
                        format: "spectrum bundle",
                        message: format!("{} does not exist", path.display()),
                    },
                )?;
                continue;
            }

            let data_before = bundle.spectra.len() + bundle.molecules.len();
            let warnings_before = bundle.warnings.len();
            self.read_existing_path_into(base, &path, &mut bundle)?;
            let data_after = bundle.spectra.len() + bundle.molecules.len();
            let warnings_after = bundle.warnings.len();
            if data_after == data_before && warnings_after == warnings_before {
                let message = match self.disabled_selected_path_message(&path) {
                    Some(message) => message,
                    None => format!("no readable bundle data found at {}", path.display()),
                };
                self.handle_error_message(&mut bundle, base, &path, message)?;
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
            Err(no_data_error_in_inputs(&bundle))
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

    fn read_existing_path_into(
        &self,
        source_root: &Path,
        path: &Path,
        bundle: &mut SpectrumBundle,
    ) -> Result<()> {
        if path.is_dir() {
            self.read_directory(source_root, path, bundle)
        } else {
            self.read_file_candidate(source_root, path, bundle)
        }
    }

    fn read_directory(
        &self,
        source_root: &Path,
        directory: &Path,
        bundle: &mut SpectrumBundle,
    ) -> Result<()> {
        let tree = collect_tree(directory)?;
        for directory in &tree.directories {
            self.read_directory_candidate(source_root, directory, bundle)?;
        }
        for file in &tree.files {
            if is_nmredata_file(file) {
                self.read_nmredata_candidate(source_root, file, bundle)?;
            } else if is_standalone_spectrum_file(file) {
                self.read_file_candidate(source_root, file, bundle)?;
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
        if self.raw.is_enabled() && self.two_d.is_enabled() && is_bruker_ser_dir(directory) {
            self.add_2d_result(
                bundle,
                root,
                directory,
                "bruker_ser",
                read_bruker_ser_2d_dir(directory),
            )?;
        }
        if self.raw.is_enabled() && self.one_d.is_enabled() && is_bruker_fid_dir(directory) {
            self.add_1d_result(
                bundle,
                root,
                directory,
                "bruker_fid",
                read_bruker_fid_1d_dir(directory),
            )?;
        }
        if self.processed.is_enabled()
            && self.two_d.is_enabled()
            && is_bruker_processed_2d_dir(directory)
        {
            self.add_2d_result(
                bundle,
                root,
                directory,
                "bruker_processed",
                read_bruker_processed_2d_dir(directory),
            )?;
        }
        if self.processed.is_enabled()
            && self.one_d.is_enabled()
            && is_bruker_processed_1d_dir(directory)
        {
            self.add_1d_result(
                bundle,
                root,
                directory,
                "bruker_processed",
                read_bruker_processed_1d_dir(directory),
            )?;
        }
        if self.raw.is_enabled() && is_agilent_fid_dir(directory) {
            if is_agilent_arrayed_2d_fid_path(directory) {
                if self.two_d.is_enabled() {
                    self.add_2d_results(
                        bundle,
                        root,
                        directory,
                        "agilent_fid",
                        read_agilent_arrayed_fid_2d_dir(directory),
                    )?;
                }
                return Ok(());
            }
            if is_agilent_arrayed_1d_fid_path(directory) {
                if self.one_d.is_enabled() {
                    self.add_1d_results(
                        bundle,
                        root,
                        directory,
                        "agilent_fid",
                        read_agilent_arrayed_fid_1d_dir(directory),
                    )?;
                }
                return Ok(());
            }
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
        match file_candidate_kind(file) {
            FileCandidateKind::Raw if !self.raw.is_enabled() => {
                return self.handle_error_message(
                    bundle,
                    root,
                    file,
                    format!(
                        "raw spectrum candidates are disabled for {}",
                        file.display()
                    ),
                );
            }
            FileCandidateKind::Processed if !self.processed.is_enabled() => {
                return self.handle_error_message(
                    bundle,
                    root,
                    file,
                    format!(
                        "processed spectrum candidates are disabled for {}",
                        file.display()
                    ),
                );
            }
            FileCandidateKind::Raw | FileCandidateKind::Processed | FileCandidateKind::Other => {}
        }

        if is_nmredata_file(file) {
            return self.read_nmredata_candidate(root, file, bundle);
        }
        if self.raw.is_enabled() && is_agilent_arrayed_1d_fid_path(file) {
            if !self.one_d.is_enabled() {
                return self.handle_error_message(
                    bundle,
                    root,
                    file,
                    disabled_dimension_message(file, "one-dimensional"),
                );
            }
            return self.add_1d_results(
                bundle,
                root,
                file,
                "agilent_fid",
                read_agilent_arrayed_fid_1d_dir(file),
            );
        }
        if self.raw.is_enabled() && is_agilent_arrayed_2d_fid_path(file) {
            if !self.two_d.is_enabled() {
                return self.handle_error_message(
                    bundle,
                    root,
                    file,
                    disabled_dimension_message(file, "two-dimensional"),
                );
            }
            return self.add_2d_results(
                bundle,
                root,
                file,
                "agilent_fid",
                read_agilent_arrayed_fid_2d_dir(file),
            );
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
        if let Some(message) = self.disabled_dimension_file_message(file) {
            return self.handle_error_message(bundle, root, file, message);
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
                if self.one_d.is_enabled() {
                    bundle.push_1d(
                        spectrum,
                        self.loaded_source(root, path, source_format_1d(path, format)),
                    );
                    return Ok(());
                }

                let first_error = disabled_dimension_error(path, "one-dimensional");
                if self.two_d.is_enabled() {
                    return match read_2d() {
                        Ok(spectrum) => {
                            bundle.push_2d(
                                spectrum,
                                self.loaded_source(root, path, source_format_2d(path, format)),
                            );
                            Ok(())
                        }
                        Err(second_error) => self.add_bundle_or_warning(
                            bundle,
                            root,
                            path,
                            read_bundle,
                            Some(&first_error),
                            Some(&second_error),
                        ),
                    };
                }

                self.add_bundle_or_warning(
                    bundle,
                    root,
                    path,
                    read_bundle,
                    Some(&first_error),
                    None,
                )
            }
            Err(first_error) => {
                if self.two_d.is_enabled() {
                    return match read_2d() {
                        Ok(spectrum) => {
                            bundle.push_2d(
                                spectrum,
                                self.loaded_source(root, path, source_format_2d(path, format)),
                            );
                            Ok(())
                        }
                        Err(second_error) => self.add_bundle_or_warning(
                            bundle,
                            root,
                            path,
                            read_bundle,
                            Some(&first_error),
                            Some(&second_error),
                        ),
                    };
                }
                let second_error = match read_2d() {
                    Ok(_) => disabled_dimension_error(path, "two-dimensional"),
                    Err(error) => error,
                };
                self.add_bundle_or_warning(
                    bundle,
                    root,
                    path,
                    read_bundle,
                    Some(&first_error),
                    Some(&second_error),
                )
            }
        }
    }

    fn add_bundle_or_warning(
        &self,
        bundle: &mut SpectrumBundle,
        root: &Path,
        path: &Path,
        read_bundle: impl FnOnce() -> Result<SpectrumBundle>,
        first_error: Option<&RSpinError>,
        second_error: Option<&RSpinError>,
    ) -> Result<()> {
        match read_bundle() {
            Ok(loaded) => {
                let loaded = self.bundle_with_source_context(root, path, loaded);
                bundle.extend_bundle(loaded);
                Ok(())
            }
            Err(third_error) => {
                let message = fallback_message(first_error, second_error, &third_error);
                self.handle_error_message(bundle, root, path, message)
            }
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
        if !self.one_d.is_enabled() {
            return Ok(());
        }
        match result {
            Ok(spectrum) => {
                bundle.push_1d(spectrum, self.loaded_source(root, path, format));
                Ok(())
            }
            Err(error) => self.handle_error(bundle, root, path, error),
        }
    }

    fn add_1d_results(
        &self,
        bundle: &mut SpectrumBundle,
        root: &Path,
        path: &Path,
        format: &'static str,
        result: Result<Vec<Spectrum1D>>,
    ) -> Result<()> {
        if !self.one_d.is_enabled() {
            return Ok(());
        }
        match result {
            Ok(spectra) => {
                for spectrum in spectra {
                    bundle.push_1d(spectrum, self.loaded_source(root, path, format));
                }
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
        if !self.two_d.is_enabled() {
            return Ok(());
        }
        match result {
            Ok(spectrum) => {
                bundle.push_2d(spectrum, self.loaded_source(root, path, format));
                Ok(())
            }
            Err(error) => self.handle_error(bundle, root, path, error),
        }
    }

    fn add_2d_results(
        &self,
        bundle: &mut SpectrumBundle,
        root: &Path,
        path: &Path,
        format: &'static str,
        result: Result<Vec<Spectrum2D>>,
    ) -> Result<()> {
        if !self.two_d.is_enabled() {
            return Ok(());
        }
        match result {
            Ok(spectra) => {
                for spectrum in spectra {
                    bundle.push_2d(spectrum, self.loaded_source(root, path, format));
                }
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
        if is_agilent_format(format) {
            if let Some(message) = self.disabled_dimension_file_message(path) {
                if root == path {
                    return self.handle_error_message(bundle, root, path, message);
                }
                return Ok(());
            }
        }

        if self.one_d.is_enabled() {
            match read_1d() {
                Ok(spectrum) => {
                    bundle.push_1d(
                        spectrum,
                        self.loaded_source(root, path, source_format_1d(path, format)),
                    );
                    return Ok(());
                }
                Err(first_error) if !self.two_d.is_enabled() => {
                    return self.handle_error(bundle, root, path, first_error);
                }
                Err(first_error) => {
                    return match read_2d() {
                        Ok(spectrum) => {
                            bundle.push_2d(
                                spectrum,
                                self.loaded_source(root, path, source_format_2d(path, format)),
                            );
                            Ok(())
                        }
                        Err(second_error) => {
                            let message =
                                format!("{first_error}; two-dimensional fallback: {second_error}");
                            self.handle_error_message(bundle, root, path, message)
                        }
                    };
                }
            }
        }

        if self.two_d.is_enabled() {
            match read_2d() {
                Ok(spectrum) => {
                    bundle.push_2d(
                        spectrum,
                        self.loaded_source(root, path, source_format_2d(path, format)),
                    );
                    Ok(())
                }
                Err(error) => self.handle_error(bundle, root, path, error),
            }
        } else {
            Ok(())
        }
    }

    fn disabled_dimension_file_message(&self, path: &Path) -> Option<String> {
        let supports_1d = crate::detect_spectrum1d_path_format(path).is_ok();
        let supports_2d = crate::detect_spectrum2d_path_format(path).is_ok();

        if !self.one_d.is_enabled() && supports_1d && (!self.two_d.is_enabled() || !supports_2d) {
            return Some(disabled_dimension_message(path, "one-dimensional"));
        }
        if !self.two_d.is_enabled() && supports_2d && (!self.one_d.is_enabled() || !supports_1d) {
            return Some(disabled_dimension_message(path, "two-dimensional"));
        }
        None
    }

    fn add_selected_path_disabled_warning(
        &self,
        root: &Path,
        bundle: &mut SpectrumBundle,
    ) -> Result<()> {
        if bundle.has_data() || !bundle.warnings.is_empty() {
            return Ok(());
        }

        if let Some(message) = self.disabled_selected_path_message(root) {
            self.handle_error_message(bundle, root, root, message)?;
        }
        Ok(())
    }

    fn disabled_selected_path_message(&self, path: &Path) -> Option<String> {
        match selected_path_candidate_kind(path) {
            FileCandidateKind::Raw if !self.raw.is_enabled() => {
                return Some(disabled_candidate_message(path, "raw spectrum"));
            }
            FileCandidateKind::Processed if !self.processed.is_enabled() => {
                return Some(disabled_candidate_message(path, "processed spectrum"));
            }
            FileCandidateKind::Raw | FileCandidateKind::Processed | FileCandidateKind::Other => {}
        }

        if is_agilent_arrayed_1d_fid_path(path) && !self.one_d.is_enabled() {
            return Some(disabled_dimension_message(path, "one-dimensional"));
        }
        if is_agilent_arrayed_2d_fid_path(path) && !self.two_d.is_enabled() {
            return Some(disabled_dimension_message(path, "two-dimensional"));
        }

        self.disabled_dimension_file_message(path)
    }

    fn filter_bundle_dimensions(&self, bundle: &mut SpectrumBundle) {
        let include_1d = self.one_d.is_enabled();
        let include_2d = self.two_d.is_enabled();
        bundle.spectra.retain(|entry| match entry {
            LoadedSpectrum::OneD { .. } => include_1d,
            LoadedSpectrum::TwoD { .. } => include_2d,
        });
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

    fn bundle_with_source_context(
        &self,
        root: &Path,
        path: &Path,
        mut bundle: SpectrumBundle,
    ) -> SpectrumBundle {
        self.filter_bundle_dimensions(&mut bundle);
        if !self.source_paths.is_enabled() {
            clear_bundle_source_paths(&mut bundle);
            return bundle;
        }

        if root == path {
            return bundle;
        }

        let Some(container_path) = self.source_path(root, path) else {
            return bundle;
        };
        prefix_bundle_source_paths(&mut bundle, &container_path);
        bundle
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
            one_d: Toggle::Enabled,
            two_d: Toggle::Enabled,
            strict: Toggle::Disabled,
            source_paths: Toggle::Enabled,
        }
    }
}

impl SpectrumPathReader for SpectrumBundleLoader {
    type Output = SpectrumBundle;

    fn read_path(&self, path: &Path) -> Result<Self::Output> {
        SpectrumBundleLoader::read_path(self, path)
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
