//! Spectrum bundle loader implementation.

mod discovery;
mod routing;

use std::path::{Path, PathBuf};

use rspin_core::{RSpinError, Result};

use crate::SpectrumPathReader;

use super::source_filter::source_filters;
use super::{
    LoadedSourceDataKind, LoadedSourceDataKindInfo, LoadedSourceFilter, LoadedSourceFormatInfo,
    LoadedSourceVendorInfo, SpectrumBundle, canonical_source_format_filter, no_data_error_at,
    no_data_error_in_inputs, selected_path_from_base, source_format_filters, source_vendor_filters,
    supported_bundle_source_data_kinds, supported_bundle_source_formats,
    supported_bundle_source_vendors,
};
pub use discovery::{
    DiscoveredSpectrumDimension, DiscoveredSpectrumDimensionCount, DiscoveredSpectrumPathCount,
    DiscoveredSpectrumSource, DiscoveredSpectrumSummary, load_discovered_spectra,
    load_discovered_spectra_by_source, load_discovered_spectra_by_source_relative_to,
    load_discovered_spectra_by_sources, load_discovered_spectra_by_sources_relative_to,
    load_discovered_spectra_relative_to, load_discovered_spectra_strict,
    load_discovered_spectra_strict_by_source, load_discovered_spectra_strict_by_source_relative_to,
    load_discovered_spectra_strict_by_sources,
    load_discovered_spectra_strict_by_sources_relative_to,
    load_discovered_spectra_strict_relative_to, load_discovered_spectrum_1d,
    load_discovered_spectrum_1d_by_source, load_discovered_spectrum_1d_by_source_relative_to,
    load_discovered_spectrum_1d_by_sources, load_discovered_spectrum_1d_by_sources_relative_to,
    load_discovered_spectrum_1d_relative_to, load_discovered_spectrum_1d_with_source,
    load_discovered_spectrum_1d_with_source_by_source,
    load_discovered_spectrum_1d_with_source_by_source_relative_to,
    load_discovered_spectrum_1d_with_source_by_sources,
    load_discovered_spectrum_1d_with_source_by_sources_relative_to,
    load_discovered_spectrum_1d_with_source_relative_to, load_discovered_spectrum_2d,
    load_discovered_spectrum_2d_by_source, load_discovered_spectrum_2d_by_source_relative_to,
    load_discovered_spectrum_2d_by_sources, load_discovered_spectrum_2d_by_sources_relative_to,
    load_discovered_spectrum_2d_relative_to, load_discovered_spectrum_2d_with_source,
    load_discovered_spectrum_2d_with_source_by_source,
    load_discovered_spectrum_2d_with_source_by_source_relative_to,
    load_discovered_spectrum_2d_with_source_by_sources,
    load_discovered_spectrum_2d_with_source_by_sources_relative_to,
    load_discovered_spectrum_2d_with_source_relative_to, select_discovered_spectra_by_source,
    select_discovered_spectra_by_sources, summarize_discovered_spectra,
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
    source_path_filters: Vec<PathBuf>,
    source_path_prefix_filters: Vec<PathBuf>,
    source_data_kind_filters: Vec<LoadedSourceDataKind>,
    source_filters: Vec<LoadedSourceFilter>,
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

    /// Returns discovery metadata for supported built-in source formats.
    #[must_use]
    pub fn supported_source_formats() -> Vec<LoadedSourceFormatInfo> {
        supported_bundle_source_formats()
    }

    /// Returns discovery metadata for supported built-in source vendor families.
    #[must_use]
    pub fn supported_source_vendors() -> Vec<LoadedSourceVendorInfo> {
        supported_bundle_source_vendors()
    }

    /// Returns discovery metadata for supported source data kinds.
    #[must_use]
    pub fn supported_source_data_kinds() -> Vec<LoadedSourceDataKindInfo> {
        supported_bundle_source_data_kinds()
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

    /// Restricts loading to spectra read with one source format.
    ///
    /// This is a short chainable alias for [`Self::only_source_format`].
    #[must_use]
    pub fn source_format(self, format: impl AsRef<str>) -> Self {
        self.only_source_format(format)
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

    /// Restricts loading to spectra read with any of the source formats.
    ///
    /// This is a short chainable alias for [`Self::only_source_formats`].
    #[must_use]
    pub fn source_formats<I, F>(self, formats: I) -> Self
    where
        I: IntoIterator<Item = F>,
        F: AsRef<str>,
    {
        self.only_source_formats(formats)
    }

    /// Restricts loading to spectra read with one vendor-specific reader family.
    #[must_use]
    pub fn only_source_vendor(mut self, vendor: impl AsRef<str>) -> Self {
        self.source_formats = source_vendor_filters([vendor]);
        self
    }

    /// Restricts loading to spectra read with one vendor-specific reader family.
    ///
    /// This is a short chainable alias for [`Self::only_source_vendor`].
    #[must_use]
    pub fn source_vendor(self, vendor: impl AsRef<str>) -> Self {
        self.only_source_vendor(vendor)
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

    /// Restricts loading to spectra read with any vendor-specific reader families.
    ///
    /// This is a short chainable alias for [`Self::only_source_vendors`].
    #[must_use]
    pub fn source_vendors<I, V>(self, vendors: I) -> Self
    where
        I: IntoIterator<Item = V>,
        V: AsRef<str>,
    {
        self.only_source_vendors(vendors)
    }

    /// Restricts loading to spectra with one raw/processed source data kind.
    ///
    /// Use this when callers want vendor raw acquisition data, vendor processed
    /// data, or open/custom data without matching specific vendor formats.
    #[must_use]
    pub fn only_source_data_kind(self, data_kind: LoadedSourceDataKind) -> Self {
        self.only_source_data_kinds([data_kind])
    }

    /// Restricts loading to spectra with one raw/processed source data kind.
    ///
    /// This is a short chainable alias for [`Self::only_source_data_kind`].
    #[must_use]
    pub fn source_data_kind(self, data_kind: LoadedSourceDataKind) -> Self {
        self.only_source_data_kind(data_kind)
    }

    /// Restricts loading to spectra with any of the raw/processed source data kinds.
    ///
    /// Passing an empty iterator leaves source loading unrestricted.
    #[must_use]
    pub fn only_source_data_kinds<I>(mut self, data_kinds: I) -> Self
    where
        I: IntoIterator<Item = LoadedSourceDataKind>,
    {
        self.source_data_kind_filters = source_data_kind_filters(data_kinds);
        self
    }

    /// Restricts loading to spectra with any raw/processed source data kinds.
    ///
    /// This is a short chainable alias for [`Self::only_source_data_kinds`].
    #[must_use]
    pub fn source_data_kinds<I>(self, data_kinds: I) -> Self
    where
        I: IntoIterator<Item = LoadedSourceDataKind>,
    {
        self.only_source_data_kinds(data_kinds)
    }

    /// Restricts loading to vendor raw acquisition data.
    #[must_use]
    pub fn only_raw_sources(self) -> Self {
        self.only_source_data_kind(LoadedSourceDataKind::Raw)
    }

    /// Restricts loading to vendor raw acquisition data.
    ///
    /// This is a short chainable alias for [`Self::only_raw_sources`].
    #[must_use]
    pub fn raw_sources(self) -> Self {
        self.only_raw_sources()
    }

    /// Restricts loading to vendor processed data.
    #[must_use]
    pub fn only_processed_sources(self) -> Self {
        self.only_source_data_kind(LoadedSourceDataKind::Processed)
    }

    /// Restricts loading to vendor processed data.
    ///
    /// This is a short chainable alias for [`Self::only_processed_sources`].
    #[must_use]
    pub fn processed_sources(self) -> Self {
        self.only_processed_sources()
    }

    /// Restricts loading to open exchange or custom data without raw/processed classification.
    #[must_use]
    pub fn only_other_sources(self) -> Self {
        self.only_source_data_kind(LoadedSourceDataKind::Other)
    }

    /// Restricts loading to open exchange or custom data without raw/processed classification.
    ///
    /// This is a short chainable alias for [`Self::only_other_sources`].
    #[must_use]
    pub fn other_sources(self) -> Self {
        self.only_other_sources()
    }

    /// Restricts loading to spectra read from one tracked source path.
    ///
    /// Source paths are matched after anchoring to the selected loader root, so
    /// pass the same relative path that appears in `LoadedSource`.
    #[must_use]
    pub fn only_source_path(mut self, path: impl AsRef<Path>) -> Self {
        self.source_path_filters = vec![path.as_ref().to_path_buf()];
        self
    }

    /// Restricts loading to spectra read from one tracked source path.
    ///
    /// This is a short chainable alias for [`Self::only_source_path`].
    #[must_use]
    pub fn source_path(self, path: impl AsRef<Path>) -> Self {
        self.only_source_path(path)
    }

    /// Restricts loading to spectra read from any of the tracked source paths.
    ///
    /// Passing an empty iterator clears the source-path filter. Filters are
    /// evaluated before source paths are optionally hidden with
    /// [`Self::without_source_paths`].
    #[must_use]
    pub fn only_source_paths<I, P>(mut self, paths: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.source_path_filters = source_path_filters(paths);
        self
    }

    /// Restricts loading to spectra read from any tracked source paths.
    ///
    /// This is a short chainable alias for [`Self::only_source_paths`].
    #[must_use]
    pub fn source_paths<I, P>(self, paths: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.only_source_paths(paths)
    }

    /// Restricts loading with one generic source filter.
    ///
    /// Prefer the specific `only_source_format`, `only_source_vendor`, or
    /// `only_source_path` helpers when the filter kind is known statically.
    #[must_use]
    pub fn only_source(self, filter: impl Into<LoadedSourceFilter>) -> Self {
        match filter.into() {
            LoadedSourceFilter::Format { format } => self.only_source_format(format),
            LoadedSourceFilter::Vendor { vendor } => self.only_source_vendor(vendor),
            LoadedSourceFilter::DataKind { data_kind } => self.only_source_data_kind(data_kind),
            LoadedSourceFilter::Path { path } => self.only_source_path(path),
            LoadedSourceFilter::PathPrefix { path } => self.only_source_path_prefix(path),
        }
    }

    /// Restricts loading with one generic source filter.
    ///
    /// This is a short chainable alias for [`Self::only_source`].
    #[must_use]
    pub fn source(self, filter: impl Into<LoadedSourceFilter>) -> Self {
        self.only_source(filter)
    }

    /// Restricts loading to spectra matching any generic source filter.
    ///
    /// This is useful when a caller accepts runtime filter choices from
    /// configuration. Filters are combined with logical OR. This replaces
    /// source-format and source-path restrictions; passing an empty iterator
    /// leaves source loading unrestricted.
    #[must_use]
    pub fn only_sources<I, F>(mut self, filters: I) -> Self
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>,
    {
        self.source_filters = source_filters(filters);
        self.source_formats.clear();
        self.source_path_filters.clear();
        self.source_path_prefix_filters.clear();
        self.source_data_kind_filters.clear();
        self
    }

    /// Restricts loading to spectra matching any generic source filter.
    ///
    /// This is a short chainable alias for [`Self::only_sources`].
    #[must_use]
    pub fn sources<I, F>(self, filters: I) -> Self
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>,
    {
        self.only_sources(filters)
    }

    /// Restricts loading to spectra whose tracked source path starts with a prefix.
    ///
    /// Use this for directory-shaped source paths after loading relative to a
    /// base directory. Exact path matching remains available with
    /// [`Self::only_source_path`].
    #[must_use]
    pub fn only_source_path_prefix(mut self, path: impl AsRef<Path>) -> Self {
        self.source_path_prefix_filters = vec![path.as_ref().to_path_buf()];
        self
    }

    /// Restricts loading to spectra whose tracked source path starts with a prefix.
    ///
    /// This is a short chainable alias for [`Self::only_source_path_prefix`].
    #[must_use]
    pub fn source_path_prefix(self, path: impl AsRef<Path>) -> Self {
        self.only_source_path_prefix(path)
    }

    /// Restricts loading to spectra whose tracked source path starts with any prefix.
    ///
    /// Passing an empty iterator leaves source loading unrestricted.
    #[must_use]
    pub fn only_source_path_prefixes<I, P>(mut self, paths: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.source_path_prefix_filters = source_path_filters(paths);
        self
    }

    /// Restricts loading to spectra whose tracked source path starts with any prefix.
    ///
    /// This is a short chainable alias for [`Self::only_source_path_prefixes`].
    #[must_use]
    pub fn source_path_prefixes<I, P>(self, paths: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.only_source_path_prefixes(paths)
    }

    /// Clears all source-format, source-path, and generic source filters.
    #[must_use]
    pub fn all_sources(mut self) -> Self {
        self.source_formats.clear();
        self.source_path_filters.clear();
        self.source_path_prefix_filters.clear();
        self.source_data_kind_filters.clear();
        self.source_filters.clear();
        self
    }

    /// Clears any source-format restriction.
    #[must_use]
    pub fn all_source_formats(mut self) -> Self {
        self.source_formats.clear();
        self.source_filters.retain(|filter| {
            !matches!(
                filter,
                LoadedSourceFilter::Format { .. } | LoadedSourceFilter::Vendor { .. }
            )
        });
        self
    }

    /// Clears any source-path restriction.
    #[must_use]
    pub fn all_source_paths(mut self) -> Self {
        self.source_path_filters.clear();
        self.source_path_prefix_filters.clear();
        self.source_filters.retain(|filter| {
            !matches!(
                filter,
                LoadedSourceFilter::Path { .. } | LoadedSourceFilter::PathPrefix { .. }
            )
        });
        self
    }

    /// Clears any generic source-data-kind restriction.
    ///
    /// Source data kind filters are represented as generic source filters, so
    /// this removes data-kind generic filters while leaving source-format,
    /// source-vendor, and source-path restrictions unchanged.
    #[must_use]
    pub fn all_source_data_kinds(mut self) -> Self {
        self.source_data_kind_filters.clear();
        self.source_filters
            .retain(|filter| !matches!(filter, LoadedSourceFilter::DataKind { .. }));
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
                if self.selected_path_is_filtered_out(path, path) {
                    continue;
                }
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
                if self.selected_path_is_filtered_out(base, &path) {
                    continue;
                }
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
            source_path_filters: Vec::new(),
            source_path_prefix_filters: Vec::new(),
            source_data_kind_filters: Vec::new(),
            source_filters: Vec::new(),
        }
    }
}

impl SpectrumPathReader for SpectrumBundleLoader {
    type Output = SpectrumBundle;

    fn read_path(&self, path: &Path) -> Result<Self::Output> {
        SpectrumBundleLoader::read_path(self, path)
    }
}

fn source_path_filters<I, P>(paths: I) -> Vec<PathBuf>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    let mut filters = Vec::new();
    for path in paths {
        let path = path.as_ref().to_path_buf();
        if !filters.iter().any(|existing| existing == &path) {
            filters.push(path);
        }
    }
    filters
}

fn source_data_kind_filters<I>(data_kinds: I) -> Vec<LoadedSourceDataKind>
where
    I: IntoIterator<Item = LoadedSourceDataKind>,
{
    let mut filters = Vec::new();
    for data_kind in data_kinds {
        if !filters.iter().any(|existing| existing == &data_kind) {
            filters.push(data_kind);
        }
    }
    filters
}
