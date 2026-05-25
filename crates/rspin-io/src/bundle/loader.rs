//! Spectrum bundle loader implementation.

mod routing;

use std::path::Path;

use rspin_core::{RSpinError, Result};

use crate::SpectrumPathReader;

use super::{
    SpectrumBundle, canonical_source_format_filter, no_data_error_at, no_data_error_in_inputs,
    selected_path_from_base, source_format_filters, source_vendor_filters,
};

/// Chainable options for loading all recognizable spectra from a path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpectrumBundleLoader {
    raw: Toggle,
    processed: Toggle,
    one_d: Toggle,
    two_d: Toggle,
    strict: Toggle,
    source_paths: Toggle,
    source_formats: Vec<String>,
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
pub(super) enum FileCandidateKind {
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

    /// Restricts loading to spectra read with one source format.
    ///
    /// Use [`LoadedSourceFormat`] for built-in names, or pass a canonical
    /// source format string for forward-compatible custom bundle data.
    #[must_use]
    pub fn only_source_format(mut self, format: impl AsRef<str>) -> Self {
        self.source_formats = vec![canonical_source_format_filter(format.as_ref())];
        self
    }

    /// Restricts loading to spectra read with any of the source formats.
    ///
    /// Passing an empty iterator clears the source-format filter.
    #[must_use]
    pub fn only_source_formats<I, F>(mut self, formats: I) -> Self
    where
        I: IntoIterator<Item = F>,
        F: AsRef<str>,
    {
        self.source_formats = source_format_filters(formats);
        self
    }

    /// Restricts loading to spectra read with one vendor-specific reader family.
    #[must_use]
    pub fn only_source_vendor(mut self, vendor: impl AsRef<str>) -> Self {
        self.source_formats = source_vendor_filters([vendor]);
        self
    }

    /// Restricts loading to spectra read with any of the vendor-specific reader families.
    ///
    /// Passing an empty iterator clears the source-format filter.
    #[must_use]
    pub fn only_source_vendors<I, V>(mut self, vendors: I) -> Self
    where
        I: IntoIterator<Item = V>,
        V: AsRef<str>,
    {
        self.source_formats = source_vendor_filters(vendors);
        self
    }

    /// Clears any source-format restriction.
    #[must_use]
    pub fn all_source_formats(mut self) -> Self {
        self.source_formats.clear();
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

    /// Loads all supported spectra from a file or directory path.
    ///
    /// This is a short alias for [`Self::read_path`] for chainable common-path
    /// workflows.
    ///
    /// # Errors
    ///
    /// Returns an error when the path is missing, strict mode rejects a
    /// candidate, or no readable bundle data is found.
    pub fn read(&self, path: impl AsRef<Path>) -> Result<SpectrumBundle> {
        self.read_path(path)
    }

    /// Loads one selected file or directory path while anchoring source paths to a base directory.
    ///
    /// Relative input paths are resolved below `base`. Absolute input paths are
    /// loaded as provided, and their source metadata is still expressed relative
    /// to `base` when possible.
    ///
    /// # Errors
    ///
    /// Returns an error when `base` is missing or is not a directory, the path
    /// is unreadable in strict mode, or no readable bundle data is found.
    pub fn read_path_relative_to(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
    ) -> Result<SpectrumBundle> {
        self.read_paths_relative_to(base, [path])
    }

    /// Loads one selected path while anchoring source paths to a base directory.
    ///
    /// This is a short alias for [`Self::read_path_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when `base` is missing or is not a directory, the path
    /// is unreadable in strict mode, or no readable bundle data is found.
    pub fn read_relative_to(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
    ) -> Result<SpectrumBundle> {
        self.read_path_relative_to(base, path)
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

    /// Loads supported spectra from multiple file or directory paths.
    ///
    /// This is a short alias for [`Self::read_paths`].
    ///
    /// # Errors
    ///
    /// Returns an error when no input paths are provided, strict mode rejects a
    /// path, or no readable bundle data is found.
    pub fn read_many<I, P>(&self, paths: I) -> Result<SpectrumBundle>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths(paths)
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

    /// Loads selected paths while anchoring source paths to a common base directory.
    ///
    /// This is a short alias for [`Self::read_paths_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when `base` is missing or is not a directory, no input
    /// paths are provided, strict mode rejects a path, or no readable bundle
    /// data is found.
    pub fn read_many_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
    ) -> Result<SpectrumBundle>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths_relative_to(base, paths)
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
            source_formats: Vec::new(),
        }
    }
}

impl SpectrumPathReader for SpectrumBundleLoader {
    type Output = SpectrumBundle;

    fn read_path(&self, path: &Path) -> Result<Self::Output> {
        SpectrumBundleLoader::read_path(self, path)
    }
}
