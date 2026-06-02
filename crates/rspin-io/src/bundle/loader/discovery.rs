//! Source discovery for the spectrum bundle loader.

mod dimension;
mod dimension_path;
mod exact;
mod exact_path;
mod exact_path_prefix_set;
mod loaded_summary;
mod model;
mod selected;
mod selection;
mod source_path;
mod strict;
mod summary;
mod summary_path;

use std::path::Path;

use rspin_core::{RSpinError, Result};

use crate::{Spectrum1DPathFormat, Spectrum2DPathFormat};

use super::SpectrumBundleLoader;
use crate::bundle::{
    LoadWarning, LoadedSourceFilter, SpectrumBundle, collect_tree, file_candidate_kind,
    format_from_file, is_agilent_arrayed_1d_fid_path, is_agilent_arrayed_2d_fid_path,
    is_agilent_fid_dir, is_agilent_processed_dir, is_bruker_fid_dir, is_bruker_processed_1d_dir,
    is_bruker_processed_2d_dir, is_bruker_ser_dir, is_nmredata_file, is_standalone_spectrum_file,
    no_data_error_in_inputs, selected_path_from_base,
};
pub use dimension::{
    load_discovered_spectra_1d, load_discovered_spectra_1d_by_source,
    load_discovered_spectra_1d_by_source_relative_to, load_discovered_spectra_1d_by_sources,
    load_discovered_spectra_1d_by_sources_relative_to, load_discovered_spectra_1d_relative_to,
    load_discovered_spectra_1d_strict, load_discovered_spectra_1d_strict_by_source,
    load_discovered_spectra_1d_strict_by_source_relative_to,
    load_discovered_spectra_1d_strict_by_sources,
    load_discovered_spectra_1d_strict_by_sources_relative_to,
    load_discovered_spectra_1d_strict_relative_to, load_discovered_spectra_2d,
    load_discovered_spectra_2d_by_source, load_discovered_spectra_2d_by_source_relative_to,
    load_discovered_spectra_2d_by_sources, load_discovered_spectra_2d_by_sources_relative_to,
    load_discovered_spectra_2d_relative_to, load_discovered_spectra_2d_strict,
    load_discovered_spectra_2d_strict_by_source,
    load_discovered_spectra_2d_strict_by_source_relative_to,
    load_discovered_spectra_2d_strict_by_sources,
    load_discovered_spectra_2d_strict_by_sources_relative_to,
    load_discovered_spectra_2d_strict_relative_to,
};
pub use dimension_path::{
    load_discovered_spectra_1d_by_source_path, load_discovered_spectra_1d_by_source_path_prefix,
    load_discovered_spectra_1d_by_source_path_prefix_relative_to,
    load_discovered_spectra_1d_by_source_path_relative_to,
    load_discovered_spectra_1d_strict_by_source_path,
    load_discovered_spectra_1d_strict_by_source_path_prefix,
    load_discovered_spectra_1d_strict_by_source_path_prefix_relative_to,
    load_discovered_spectra_1d_strict_by_source_path_relative_to,
    load_discovered_spectra_2d_by_source_path, load_discovered_spectra_2d_by_source_path_prefix,
    load_discovered_spectra_2d_by_source_path_prefix_relative_to,
    load_discovered_spectra_2d_by_source_path_relative_to,
    load_discovered_spectra_2d_strict_by_source_path,
    load_discovered_spectra_2d_strict_by_source_path_prefix,
    load_discovered_spectra_2d_strict_by_source_path_prefix_relative_to,
    load_discovered_spectra_2d_strict_by_source_path_relative_to,
};
pub use exact::{
    load_discovered_spectrum_1d, load_discovered_spectrum_1d_by_source,
    load_discovered_spectrum_1d_by_source_relative_to, load_discovered_spectrum_1d_by_sources,
    load_discovered_spectrum_1d_by_sources_relative_to, load_discovered_spectrum_1d_relative_to,
    load_discovered_spectrum_1d_with_source, load_discovered_spectrum_1d_with_source_by_source,
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
    load_discovered_spectrum_2d_with_source_relative_to,
};
pub use exact_path::{
    load_discovered_spectrum_1d_by_source_path, load_discovered_spectrum_1d_by_source_path_prefix,
    load_discovered_spectrum_1d_by_source_path_prefix_relative_to,
    load_discovered_spectrum_1d_by_source_path_relative_to,
    load_discovered_spectrum_1d_with_source_by_source_path,
    load_discovered_spectrum_1d_with_source_by_source_path_prefix,
    load_discovered_spectrum_1d_with_source_by_source_path_prefix_relative_to,
    load_discovered_spectrum_1d_with_source_by_source_path_relative_to,
    load_discovered_spectrum_2d_by_source_path, load_discovered_spectrum_2d_by_source_path_prefix,
    load_discovered_spectrum_2d_by_source_path_prefix_relative_to,
    load_discovered_spectrum_2d_by_source_path_relative_to,
    load_discovered_spectrum_2d_with_source_by_source_path,
    load_discovered_spectrum_2d_with_source_by_source_path_prefix,
    load_discovered_spectrum_2d_with_source_by_source_path_prefix_relative_to,
    load_discovered_spectrum_2d_with_source_by_source_path_relative_to,
};
pub use exact_path_prefix_set::{
    load_discovered_spectrum_1d_by_source_path_prefixes,
    load_discovered_spectrum_1d_by_source_path_prefixes_relative_to,
    load_discovered_spectrum_1d_with_source_by_source_path_prefixes,
    load_discovered_spectrum_1d_with_source_by_source_path_prefixes_relative_to,
    load_discovered_spectrum_2d_by_source_path_prefixes,
    load_discovered_spectrum_2d_by_source_path_prefixes_relative_to,
    load_discovered_spectrum_2d_with_source_by_source_path_prefixes,
    load_discovered_spectrum_2d_with_source_by_source_path_prefixes_relative_to,
};
pub use loaded_summary::{
    load_discovered_spectra_summary, load_discovered_spectra_summary_by_source,
    load_discovered_spectra_summary_by_source_relative_to,
    load_discovered_spectra_summary_by_sources,
    load_discovered_spectra_summary_by_sources_relative_to,
    load_discovered_spectra_summary_relative_to, load_discovered_spectra_summary_strict,
    load_discovered_spectra_summary_strict_by_source,
    load_discovered_spectra_summary_strict_by_source_relative_to,
    load_discovered_spectra_summary_strict_by_sources,
    load_discovered_spectra_summary_strict_by_sources_relative_to,
    load_discovered_spectra_summary_strict_relative_to,
};
pub use model::{DiscoveredSpectrumDimension, DiscoveredSpectrumSource};
pub use selected::SelectedDiscoveredSpectrumSourcesExt;
pub use selection::{
    DiscoveredSpectrumSourcesExt, select_discovered_spectra_1d,
    select_discovered_spectra_1d_by_source, select_discovered_spectra_1d_by_source_path,
    select_discovered_spectra_1d_by_source_path_prefix, select_discovered_spectra_1d_by_sources,
    select_discovered_spectra_2d, select_discovered_spectra_2d_by_source,
    select_discovered_spectra_2d_by_source_path,
    select_discovered_spectra_2d_by_source_path_prefix, select_discovered_spectra_2d_by_sources,
    select_discovered_spectra_by_dimension, select_discovered_spectra_by_dimension_and_source,
    select_discovered_spectra_by_dimension_and_sources, select_discovered_spectra_by_source,
    select_discovered_spectra_by_source_path, select_discovered_spectra_by_source_path_prefix,
    select_discovered_spectra_by_sources,
};
pub use source_path::{
    load_discovered_spectra_by_source_path, load_discovered_spectra_by_source_path_prefix,
    load_discovered_spectra_by_source_path_prefix_relative_to,
    load_discovered_spectra_by_source_path_prefixes,
    load_discovered_spectra_by_source_path_prefixes_relative_to,
    load_discovered_spectra_by_source_path_relative_to,
};
pub use strict::{
    load_discovered_spectra_strict, load_discovered_spectra_strict_by_source,
    load_discovered_spectra_strict_by_source_path,
    load_discovered_spectra_strict_by_source_path_prefix,
    load_discovered_spectra_strict_by_source_path_prefix_relative_to,
    load_discovered_spectra_strict_by_source_path_relative_to,
    load_discovered_spectra_strict_by_source_relative_to,
    load_discovered_spectra_strict_by_sources,
    load_discovered_spectra_strict_by_sources_relative_to,
    load_discovered_spectra_strict_relative_to,
};
pub use summary::{
    DiscoveredSpectrumDimensionCount, DiscoveredSpectrumPathCount, DiscoveredSpectrumSummary,
    summarize_discovered_spectra,
};
pub use summary_path::{
    load_discovered_spectra_summary_by_source_path,
    load_discovered_spectra_summary_by_source_path_prefix,
    load_discovered_spectra_summary_by_source_path_prefix_relative_to,
    load_discovered_spectra_summary_by_source_path_prefixes,
    load_discovered_spectra_summary_by_source_path_prefixes_relative_to,
    load_discovered_spectra_summary_by_source_path_relative_to,
    load_discovered_spectra_summary_strict_by_source_path,
    load_discovered_spectra_summary_strict_by_source_path_prefix,
    load_discovered_spectra_summary_strict_by_source_path_prefix_relative_to,
    load_discovered_spectra_summary_strict_by_source_path_prefixes,
    load_discovered_spectra_summary_strict_by_source_path_prefixes_relative_to,
    load_discovered_spectra_summary_strict_by_source_path_relative_to,
};

/// Loads selected discovered source candidates relative to a common base directory.
///
/// This is a convenience wrapper for the common preflight workflow: discover
/// candidates, let the caller select some of them, then load only those exact
/// source path and format pairs.
///
/// # Errors
///
/// Returns an error when `base` is missing or not a directory, no discovered
/// sources are provided, a selected source does not include a tracked source
/// path, strict loading rejects a selected source, or no selected source can be
/// read.
pub fn load_discovered_spectra_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
) -> Result<SpectrumBundle> {
    SpectrumBundleLoader::new().read_discovered_relative_to(base, sources)
}

/// Loads selected discovered source candidates relative to a common base directory.
///
/// This short alias mirrors [`load_discovered_spectra_relative_to`].
///
/// # Errors
///
/// Returns an error when loading the selected discovered sources fails.
pub fn load_discovered_spectra<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
) -> Result<SpectrumBundle> {
    load_discovered_spectra_relative_to(base, sources)
}

/// Loads selected discovered source candidates matching one generic source filter.
///
/// This is a convenience wrapper for filtering discovered candidates before
/// loading them with [`load_discovered_spectra_relative_to`].
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_by_source_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    filter: impl Into<LoadedSourceFilter>,
) -> Result<SpectrumBundle> {
    SpectrumBundleLoader::new().read_discovered_by_source_relative_to(base, sources, filter)
}

/// Loads selected discovered source candidates matching one generic source filter.
///
/// This short alias mirrors [`load_discovered_spectra_by_source_relative_to`].
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_by_source<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    filter: impl Into<LoadedSourceFilter>,
) -> Result<SpectrumBundle> {
    load_discovered_spectra_by_source_relative_to(base, sources, filter)
}

/// Loads selected discovered source candidates matching any generic source filter.
///
/// Filters are combined with logical OR. Passing an empty iterator loads all
/// provided discovered sources.
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_by_sources_relative_to<'a, I, F>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    filters: I,
) -> Result<SpectrumBundle>
where
    I: IntoIterator<Item = F>,
    F: Into<LoadedSourceFilter>,
{
    SpectrumBundleLoader::new().read_discovered_by_sources_relative_to(base, sources, filters)
}

/// Loads selected discovered source candidates matching any generic source filter.
///
/// This short alias mirrors [`load_discovered_spectra_by_sources_relative_to`].
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_by_sources<'a, I, F>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    filters: I,
) -> Result<SpectrumBundle>
where
    I: IntoIterator<Item = F>,
    F: Into<LoadedSourceFilter>,
{
    load_discovered_spectra_by_sources_relative_to(base, sources, filters)
}

impl SpectrumBundleLoader {
    /// Discovers source candidates below a file or directory without loading full spectra.
    ///
    /// Discovery respects source-format, vendor, data-kind, path, dimension,
    /// and raw/processed loader filters. Empty results are allowed so callers
    /// can use this as a lightweight preflight step.
    ///
    /// # Errors
    ///
    /// Returns an error when the input path is missing or a directory cannot be read.
    pub fn discover_path(&self, path: impl AsRef<Path>) -> Result<Vec<DiscoveredSpectrumSource>> {
        let root = path.as_ref();
        if !root.exists() {
            return Err(RSpinError::Parse {
                format: "spectrum source discovery",
                message: format!("{} does not exist", root.display()),
            });
        }

        let mut sources = Vec::new();
        self.discover_existing_path_into(root, root, &mut sources)?;
        Ok(sources)
    }

    /// Discovers source candidates below a file or directory.
    ///
    /// This is a short alias for [`Self::discover_path`].
    ///
    /// # Errors
    ///
    /// Returns an error when the input path is missing or a directory cannot be read.
    pub fn discover(&self, path: impl AsRef<Path>) -> Result<Vec<DiscoveredSpectrumSource>> {
        self.discover_path(path)
    }

    /// Discovers and summarizes source candidates below a file or directory.
    ///
    /// Discovery respects the same filters as [`Self::discover_path`].
    ///
    /// # Errors
    ///
    /// Returns an error when the input path is missing or a directory cannot be read.
    pub fn discover_summary_path(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<DiscoveredSpectrumSummary> {
        let sources = self.discover_path(path)?;
        Ok(DiscoveredSpectrumSummary::new(&sources))
    }

    /// Discovers and summarizes source candidates below a file or directory.
    ///
    /// This is a short alias for [`Self::discover_summary_path`].
    ///
    /// # Errors
    ///
    /// Returns an error when the input path is missing or a directory cannot be read.
    pub fn discover_summary(&self, path: impl AsRef<Path>) -> Result<DiscoveredSpectrumSummary> {
        self.discover_summary_path(path)
    }

    /// Discovers one-dimensional source candidates below a file or directory.
    ///
    /// This preserves the loader's other source filters.
    ///
    /// # Errors
    ///
    /// Returns an error when the input path is missing or a directory cannot be read.
    pub fn discover_1d_path(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<Vec<DiscoveredSpectrumSource>> {
        let sources = self.discover_path(path)?;
        Ok(discovered_dimension_subset(
            sources,
            DiscoveredSpectrumDimension::OneD,
        ))
    }

    /// Discovers one-dimensional source candidates below a file or directory.
    ///
    /// This is a short alias for [`Self::discover_1d_path`].
    ///
    /// # Errors
    ///
    /// Returns an error when the input path is missing or a directory cannot be read.
    pub fn discover_1d(&self, path: impl AsRef<Path>) -> Result<Vec<DiscoveredSpectrumSource>> {
        self.discover_1d_path(path)
    }

    /// Discovers two-dimensional source candidates below a file or directory.
    ///
    /// This preserves the loader's other source filters.
    ///
    /// # Errors
    ///
    /// Returns an error when the input path is missing or a directory cannot be read.
    pub fn discover_2d_path(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<Vec<DiscoveredSpectrumSource>> {
        let sources = self.discover_path(path)?;
        Ok(discovered_dimension_subset(
            sources,
            DiscoveredSpectrumDimension::TwoD,
        ))
    }

    /// Discovers two-dimensional source candidates below a file or directory.
    ///
    /// This is a short alias for [`Self::discover_2d_path`].
    ///
    /// # Errors
    ///
    /// Returns an error when the input path is missing or a directory cannot be read.
    pub fn discover_2d(&self, path: impl AsRef<Path>) -> Result<Vec<DiscoveredSpectrumSource>> {
        self.discover_2d_path(path)
    }

    /// Discovers and summarizes one-dimensional source candidates below a file or directory.
    ///
    /// # Errors
    ///
    /// Returns an error when the input path is missing or a directory cannot be read.
    pub fn discover_1d_summary_path(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<DiscoveredSpectrumSummary> {
        let sources = self.discover_1d_path(path)?;
        Ok(DiscoveredSpectrumSummary::new(&sources))
    }

    /// Discovers and summarizes one-dimensional source candidates below a file or directory.
    ///
    /// This is a short alias for [`Self::discover_1d_summary_path`].
    ///
    /// # Errors
    ///
    /// Returns an error when the input path is missing or a directory cannot be read.
    pub fn discover_1d_summary(&self, path: impl AsRef<Path>) -> Result<DiscoveredSpectrumSummary> {
        self.discover_1d_summary_path(path)
    }

    /// Discovers and summarizes two-dimensional source candidates below a file or directory.
    ///
    /// # Errors
    ///
    /// Returns an error when the input path is missing or a directory cannot be read.
    pub fn discover_2d_summary_path(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<DiscoveredSpectrumSummary> {
        let sources = self.discover_2d_path(path)?;
        Ok(DiscoveredSpectrumSummary::new(&sources))
    }

    /// Discovers and summarizes two-dimensional source candidates below a file or directory.
    ///
    /// This is a short alias for [`Self::discover_2d_summary_path`].
    ///
    /// # Errors
    ///
    /// Returns an error when the input path is missing or a directory cannot be read.
    pub fn discover_2d_summary(&self, path: impl AsRef<Path>) -> Result<DiscoveredSpectrumSummary> {
        self.discover_2d_summary_path(path)
    }

    /// Discovers one selected path while anchoring source paths to a base directory.
    ///
    /// # Errors
    ///
    /// Returns an error when the base directory or selected path is missing.
    pub fn discover_path_relative_to(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
    ) -> Result<Vec<DiscoveredSpectrumSource>> {
        self.discover_paths_relative_to(base, [path])
    }

    /// Discovers one selected path while anchoring source paths to a base directory.
    ///
    /// This is a short alias for [`Self::discover_path_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when the base directory or selected path is missing.
    pub fn discover_relative_to(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
    ) -> Result<Vec<DiscoveredSpectrumSource>> {
        self.discover_path_relative_to(base, path)
    }

    /// Discovers and summarizes one selected path while anchoring source paths to a base directory.
    ///
    /// # Errors
    ///
    /// Returns an error when the base directory or selected path is missing.
    pub fn discover_summary_path_relative_to(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
    ) -> Result<DiscoveredSpectrumSummary> {
        let sources = self.discover_path_relative_to(base, path)?;
        Ok(DiscoveredSpectrumSummary::new(&sources))
    }

    /// Discovers and summarizes one selected path while anchoring source paths to a base directory.
    ///
    /// This is a short alias for [`Self::discover_summary_path_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when the base directory or selected path is missing.
    pub fn discover_summary_relative_to(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
    ) -> Result<DiscoveredSpectrumSummary> {
        self.discover_summary_path_relative_to(base, path)
    }

    /// Discovers one-dimensional source candidates from one selected path relative to a base directory.
    ///
    /// # Errors
    ///
    /// Returns an error when the base directory or selected path is missing.
    pub fn discover_1d_relative_to(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
    ) -> Result<Vec<DiscoveredSpectrumSource>> {
        let sources = self.discover_path_relative_to(base, path)?;
        Ok(discovered_dimension_subset(
            sources,
            DiscoveredSpectrumDimension::OneD,
        ))
    }

    /// Discovers two-dimensional source candidates from one selected path relative to a base directory.
    ///
    /// # Errors
    ///
    /// Returns an error when the base directory or selected path is missing.
    pub fn discover_2d_relative_to(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
    ) -> Result<Vec<DiscoveredSpectrumSource>> {
        let sources = self.discover_path_relative_to(base, path)?;
        Ok(discovered_dimension_subset(
            sources,
            DiscoveredSpectrumDimension::TwoD,
        ))
    }

    /// Discovers and summarizes one-dimensional source candidates from one selected path relative to a base directory.
    ///
    /// # Errors
    ///
    /// Returns an error when the base directory or selected path is missing.
    pub fn discover_1d_summary_relative_to(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
    ) -> Result<DiscoveredSpectrumSummary> {
        let sources = self.discover_1d_relative_to(base, path)?;
        Ok(DiscoveredSpectrumSummary::new(&sources))
    }

    /// Discovers and summarizes two-dimensional source candidates from one selected path relative to a base directory.
    ///
    /// # Errors
    ///
    /// Returns an error when the base directory or selected path is missing.
    pub fn discover_2d_summary_relative_to(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
    ) -> Result<DiscoveredSpectrumSummary> {
        let sources = self.discover_2d_relative_to(base, path)?;
        Ok(DiscoveredSpectrumSummary::new(&sources))
    }

    /// Discovers source candidates below multiple file or directory paths.
    ///
    /// # Errors
    ///
    /// Returns an error when no paths are provided or an input path is missing.
    pub fn discover_paths<I, P>(&self, paths: I) -> Result<Vec<DiscoveredSpectrumSource>>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        let mut sources = Vec::new();
        let mut saw_input = false;

        for path in paths {
            saw_input = true;
            let path = path.as_ref();
            if !path.exists() {
                return Err(RSpinError::Parse {
                    format: "spectrum source discovery",
                    message: format!("{} does not exist", path.display()),
                });
            }
            self.discover_existing_path_into(path, path, &mut sources)?;
        }

        if !saw_input {
            return Err(no_discovery_inputs_error());
        }

        Ok(sources)
    }

    /// Discovers source candidates below multiple file or directory paths.
    ///
    /// This is a short alias for [`Self::discover_paths`].
    ///
    /// # Errors
    ///
    /// Returns an error when no paths are provided or an input path is missing.
    pub fn discover_many<I, P>(&self, paths: I) -> Result<Vec<DiscoveredSpectrumSource>>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.discover_paths(paths)
    }

    /// Discovers and summarizes source candidates below multiple file or directory paths.
    ///
    /// # Errors
    ///
    /// Returns an error when no paths are provided or an input path is missing.
    pub fn discover_summary_paths<I, P>(&self, paths: I) -> Result<DiscoveredSpectrumSummary>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        let sources = self.discover_paths(paths)?;
        Ok(DiscoveredSpectrumSummary::new(&sources))
    }

    /// Discovers and summarizes source candidates below multiple file or directory paths.
    ///
    /// This is a short alias for [`Self::discover_summary_paths`].
    ///
    /// # Errors
    ///
    /// Returns an error when no paths are provided or an input path is missing.
    pub fn discover_summary_many<I, P>(&self, paths: I) -> Result<DiscoveredSpectrumSummary>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.discover_summary_paths(paths)
    }

    /// Discovers one-dimensional source candidates below multiple file or directory paths.
    ///
    /// # Errors
    ///
    /// Returns an error when no paths are provided or an input path is missing.
    pub fn discover_1d_many<I, P>(&self, paths: I) -> Result<Vec<DiscoveredSpectrumSource>>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        let sources = self.discover_paths(paths)?;
        Ok(discovered_dimension_subset(
            sources,
            DiscoveredSpectrumDimension::OneD,
        ))
    }

    /// Discovers two-dimensional source candidates below multiple file or directory paths.
    ///
    /// # Errors
    ///
    /// Returns an error when no paths are provided or an input path is missing.
    pub fn discover_2d_many<I, P>(&self, paths: I) -> Result<Vec<DiscoveredSpectrumSource>>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        let sources = self.discover_paths(paths)?;
        Ok(discovered_dimension_subset(
            sources,
            DiscoveredSpectrumDimension::TwoD,
        ))
    }

    /// Discovers and summarizes one-dimensional source candidates below multiple file or directory paths.
    ///
    /// # Errors
    ///
    /// Returns an error when no paths are provided or an input path is missing.
    pub fn discover_1d_summary_many<I, P>(&self, paths: I) -> Result<DiscoveredSpectrumSummary>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        let sources = self.discover_1d_many(paths)?;
        Ok(DiscoveredSpectrumSummary::new(&sources))
    }

    /// Discovers and summarizes two-dimensional source candidates below multiple file or directory paths.
    ///
    /// # Errors
    ///
    /// Returns an error when no paths are provided or an input path is missing.
    pub fn discover_2d_summary_many<I, P>(&self, paths: I) -> Result<DiscoveredSpectrumSummary>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        let sources = self.discover_2d_many(paths)?;
        Ok(DiscoveredSpectrumSummary::new(&sources))
    }

    /// Discovers selected paths while anchoring source paths to a base directory.
    ///
    /// # Errors
    ///
    /// Returns an error when the base directory is missing, no selected paths
    /// are provided, or a selected path is missing.
    pub fn discover_paths_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
    ) -> Result<Vec<DiscoveredSpectrumSource>>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        let base = base.as_ref();
        if !base.exists() {
            return Err(RSpinError::Parse {
                format: "spectrum source discovery",
                message: format!("{} does not exist", base.display()),
            });
        }
        if !base.is_dir() {
            return Err(RSpinError::Parse {
                format: "spectrum source discovery",
                message: format!("{} is not a directory", base.display()),
            });
        }

        let mut sources = Vec::new();
        let mut saw_input = false;
        for path in paths {
            saw_input = true;
            let path = selected_path_from_base(base, path.as_ref());
            if !path.exists() {
                return Err(RSpinError::Parse {
                    format: "spectrum source discovery",
                    message: format!("{} does not exist", path.display()),
                });
            }
            self.discover_existing_path_into(base, &path, &mut sources)?;
        }

        if !saw_input {
            return Err(no_discovery_inputs_error());
        }

        Ok(sources)
    }

    /// Discovers selected paths while anchoring source paths to a base directory.
    ///
    /// This is a short alias for [`Self::discover_paths_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when the base directory is missing, no selected paths
    /// are provided, or a selected path is missing.
    pub fn discover_many_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
    ) -> Result<Vec<DiscoveredSpectrumSource>>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.discover_paths_relative_to(base, paths)
    }

    /// Discovers and summarizes selected paths while anchoring source paths to a base directory.
    ///
    /// # Errors
    ///
    /// Returns an error when the base directory is missing, no selected paths
    /// are provided, or a selected path is missing.
    pub fn discover_summary_paths_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
    ) -> Result<DiscoveredSpectrumSummary>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        let sources = self.discover_paths_relative_to(base, paths)?;
        Ok(DiscoveredSpectrumSummary::new(&sources))
    }

    /// Discovers and summarizes selected paths while anchoring source paths to a base directory.
    ///
    /// This is a short alias for [`Self::discover_summary_paths_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when the base directory is missing, no selected paths
    /// are provided, or a selected path is missing.
    pub fn discover_summary_many_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
    ) -> Result<DiscoveredSpectrumSummary>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.discover_summary_paths_relative_to(base, paths)
    }

    /// Discovers one-dimensional source candidates from selected paths relative to a base directory.
    ///
    /// # Errors
    ///
    /// Returns an error when the base directory is missing, no selected paths
    /// are provided, or a selected path is missing.
    pub fn discover_1d_many_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
    ) -> Result<Vec<DiscoveredSpectrumSource>>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        let sources = self.discover_paths_relative_to(base, paths)?;
        Ok(discovered_dimension_subset(
            sources,
            DiscoveredSpectrumDimension::OneD,
        ))
    }

    /// Discovers two-dimensional source candidates from selected paths relative to a base directory.
    ///
    /// # Errors
    ///
    /// Returns an error when the base directory is missing, no selected paths
    /// are provided, or a selected path is missing.
    pub fn discover_2d_many_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
    ) -> Result<Vec<DiscoveredSpectrumSource>>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        let sources = self.discover_paths_relative_to(base, paths)?;
        Ok(discovered_dimension_subset(
            sources,
            DiscoveredSpectrumDimension::TwoD,
        ))
    }

    /// Discovers and summarizes one-dimensional source candidates from selected paths relative to a base directory.
    ///
    /// # Errors
    ///
    /// Returns an error when the base directory is missing, no selected paths
    /// are provided, or a selected path is missing.
    pub fn discover_1d_summary_many_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
    ) -> Result<DiscoveredSpectrumSummary>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        let sources = self.discover_1d_many_relative_to(base, paths)?;
        Ok(DiscoveredSpectrumSummary::new(&sources))
    }

    /// Discovers and summarizes two-dimensional source candidates from selected paths relative to a base directory.
    ///
    /// # Errors
    ///
    /// Returns an error when the base directory is missing, no selected paths
    /// are provided, or a selected path is missing.
    pub fn discover_2d_summary_many_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
    ) -> Result<DiscoveredSpectrumSummary>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        let sources = self.discover_2d_many_relative_to(base, paths)?;
        Ok(DiscoveredSpectrumSummary::new(&sources))
    }

    /// Loads selected discovered source candidates relative to a common base directory.
    ///
    /// Each discovered source must have source-path tracking enabled. The
    /// loader reads each selected source with both its discovered path and
    /// discovered source format as filters, preserving any other options already
    /// configured on this reader.
    ///
    /// # Errors
    ///
    /// Returns an error when `base` is missing or not a directory, no discovered
    /// sources are provided, a selected source does not include a tracked source
    /// path, strict loading rejects a selected source, or no selected source can
    /// be read.
    pub fn read_discovered_relative_to<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    ) -> Result<SpectrumBundle> {
        let base = base.as_ref();
        validate_discovered_base(base)?;

        let mut bundle = SpectrumBundle::new();
        let mut saw_source = false;

        for source in sources {
            saw_source = true;
            let source_path = source.path().ok_or_else(discovered_source_path_error)?;
            let selected_path = selected_path_from_base(base, source_path);
            let loader = self.loader_for_discovered_source(source_path, source);

            match loader.read_path_relative_to(base, source_path) {
                Ok(selected_bundle) => bundle.extend_bundle(selected_bundle),
                Err(error) if self.strict.is_enabled() => return Err(error),
                Err(error) => bundle.push_warning(LoadWarning::new(
                    self.source_path_for_metadata(base, &selected_path),
                    error.to_string(),
                )),
            }
        }

        if !saw_source {
            return Err(no_discovered_sources_error());
        }

        if bundle.has_data() {
            Ok(bundle)
        } else {
            Err(no_data_error_in_inputs(&bundle))
        }
    }

    /// Loads selected discovered source candidates relative to a common base directory.
    ///
    /// This short alias mirrors [`Self::read_discovered_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading the selected discovered sources fails.
    pub fn read_discovered<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    ) -> Result<SpectrumBundle> {
        self.read_discovered_relative_to(base, sources)
    }

    /// Loads discovered source candidates matching one generic source filter.
    ///
    /// This is useful after a preflight discovery step when caller state stores
    /// a runtime source filter rather than explicit selected candidate indices.
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_by_source_relative_to<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        filter: impl Into<LoadedSourceFilter>,
    ) -> Result<SpectrumBundle> {
        let filter = filter.into();
        self.read_discovered_by_sources_relative_to(base, sources, [filter])
    }

    /// Loads discovered source candidates matching one generic source filter.
    ///
    /// This short alias mirrors [`Self::read_discovered_by_source_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_by_source<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        filter: impl Into<LoadedSourceFilter>,
    ) -> Result<SpectrumBundle> {
        self.read_discovered_by_source_relative_to(base, sources, filter)
    }

    /// Loads discovered source candidates matching any generic source filter.
    ///
    /// Filters are combined with logical OR. Passing an empty iterator loads all
    /// provided discovered sources.
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_by_sources_relative_to<'a, I, F>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        filters: I,
    ) -> Result<SpectrumBundle>
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>,
    {
        let selected = selection::select_discovered_source_refs(sources, filters);
        self.read_discovered_relative_to(base, selected)
    }

    /// Loads discovered source candidates matching any generic source filter.
    ///
    /// This short alias mirrors [`Self::read_discovered_by_sources_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_by_sources<'a, I, F>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        filters: I,
    ) -> Result<SpectrumBundle>
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>,
    {
        self.read_discovered_by_sources_relative_to(base, sources, filters)
    }

    fn discover_existing_path_into(
        &self,
        source_root: &Path,
        path: &Path,
        sources: &mut Vec<DiscoveredSpectrumSource>,
    ) -> Result<()> {
        if path.is_dir() {
            self.discover_directory(source_root, path, sources)
        } else {
            self.discover_file_candidate(source_root, path, sources);
            Ok(())
        }
    }

    fn discover_directory(
        &self,
        source_root: &Path,
        directory: &Path,
        sources: &mut Vec<DiscoveredSpectrumSource>,
    ) -> Result<()> {
        let tree = collect_tree(directory)?;
        for directory in &tree.directories {
            self.discover_directory_candidate(source_root, directory, sources);
        }
        for file in &tree.files {
            if is_nmredata_file(file) {
                continue;
            }
            if is_standalone_spectrum_file(file) {
                self.discover_file_candidate(source_root, file, sources);
            }
        }
        Ok(())
    }

    fn discover_directory_candidate(
        &self,
        root: &Path,
        directory: &Path,
        sources: &mut Vec<DiscoveredSpectrumSource>,
    ) {
        if is_bruker_ser_dir(directory) {
            self.push_discovered_source_if_allowed(
                sources,
                root,
                directory,
                "bruker_ser",
                DiscoveredSpectrumDimension::TwoD,
            );
        }
        if is_bruker_fid_dir(directory) {
            self.push_discovered_source_if_allowed(
                sources,
                root,
                directory,
                "bruker_fid",
                DiscoveredSpectrumDimension::OneD,
            );
        }
        if is_bruker_processed_2d_dir(directory) {
            self.push_discovered_source_if_allowed(
                sources,
                root,
                directory,
                "bruker_processed",
                DiscoveredSpectrumDimension::TwoD,
            );
        }
        if is_bruker_processed_1d_dir(directory) {
            self.push_discovered_source_if_allowed(
                sources,
                root,
                directory,
                "bruker_processed",
                DiscoveredSpectrumDimension::OneD,
            );
        }
        if is_agilent_fid_dir(directory) {
            self.discover_agilent_fid_path(root, directory, sources);
        }
        if is_agilent_processed_dir(directory) {
            self.discover_detected_path(root, directory, Some("agilent_processed"), sources);
        }
    }

    fn discover_file_candidate(
        &self,
        root: &Path,
        file: &Path,
        sources: &mut Vec<DiscoveredSpectrumSource>,
    ) {
        if is_nmredata_file(file) || !self.allows_file_candidate_kind(file_candidate_kind(file)) {
            return;
        }

        let format = format_from_file(file);
        if format == "auto" {
            self.discover_detected_path(root, file, None, sources);
        } else {
            self.discover_detected_path(root, file, Some(format), sources);
        }
    }

    fn discover_agilent_fid_path(
        &self,
        root: &Path,
        path: &Path,
        sources: &mut Vec<DiscoveredSpectrumSource>,
    ) {
        if is_agilent_arrayed_2d_fid_path(path) {
            self.push_discovered_source_if_allowed(
                sources,
                root,
                path,
                "agilent_fid",
                DiscoveredSpectrumDimension::TwoD,
            );
            return;
        }

        if is_agilent_arrayed_1d_fid_path(path) {
            self.push_discovered_source_if_allowed(
                sources,
                root,
                path,
                "agilent_fid",
                DiscoveredSpectrumDimension::OneD,
            );
            return;
        }

        self.discover_detected_path(root, path, Some("agilent_fid"), sources);
    }

    fn discover_detected_path(
        &self,
        root: &Path,
        path: &Path,
        fallback_format: Option<&'static str>,
        sources: &mut Vec<DiscoveredSpectrumSource>,
    ) {
        let mut pushed = false;

        if let Ok(format) = crate::detect_spectrum1d_path_format(path) {
            pushed = self.push_discovered_source_if_allowed(
                sources,
                root,
                path,
                path_format_1d(format, fallback_format),
                DiscoveredSpectrumDimension::OneD,
            ) || pushed;
        }

        if let Ok(format) = crate::detect_spectrum2d_path_format(path) {
            pushed = self.push_discovered_source_if_allowed(
                sources,
                root,
                path,
                path_format_2d(format, fallback_format),
                DiscoveredSpectrumDimension::TwoD,
            ) || pushed;
        }

        if !pushed && let Some(format) = fallback_format {
            self.push_discovered_source_if_allowed(
                sources,
                root,
                path,
                format,
                DiscoveredSpectrumDimension::Unknown,
            );
        }
    }

    fn push_discovered_source_if_allowed(
        &self,
        sources: &mut Vec<DiscoveredSpectrumSource>,
        root: &Path,
        path: &Path,
        format: &'static str,
        dimension: DiscoveredSpectrumDimension,
    ) -> bool {
        if !self.allows_discovered_dimension(dimension) || !self.allows_source(root, path, format) {
            return false;
        }

        sources.push(DiscoveredSpectrumSource::new(
            self.source_path_for_metadata(root, path),
            format,
            dimension,
        ));
        true
    }

    fn allows_discovered_dimension(&self, dimension: DiscoveredSpectrumDimension) -> bool {
        match dimension {
            DiscoveredSpectrumDimension::OneD => self.one_d.is_enabled(),
            DiscoveredSpectrumDimension::TwoD => self.two_d.is_enabled(),
            DiscoveredSpectrumDimension::Unknown => {
                self.one_d.is_enabled() || self.two_d.is_enabled()
            }
        }
    }

    fn loader_for_discovered_source(
        &self,
        source_path: &Path,
        source: &DiscoveredSpectrumSource,
    ) -> Self {
        let loader = self
            .clone()
            .source_path(source_path)
            .source_format(source.format());
        match source.dimension() {
            DiscoveredSpectrumDimension::OneD => loader.with_2d(false),
            DiscoveredSpectrumDimension::TwoD => loader.with_1d(false),
            DiscoveredSpectrumDimension::Unknown => loader,
        }
    }
}

fn validate_discovered_base(base: &Path) -> Result<()> {
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
    Ok(())
}

fn discovered_source_path_error() -> RSpinError {
    RSpinError::Parse {
        format: "spectrum bundle",
        message:
            "discovered source is missing a tracked source path; discover with source paths enabled"
                .to_owned(),
    }
}

fn no_discovered_sources_error() -> RSpinError {
    RSpinError::Parse {
        format: "spectrum bundle",
        message: "no discovered sources provided".to_owned(),
    }
}

fn discovered_dimension_subset(
    sources: Vec<DiscoveredSpectrumSource>,
    dimension: DiscoveredSpectrumDimension,
) -> Vec<DiscoveredSpectrumSource> {
    sources
        .into_iter()
        .filter(|source| source.dimension() == dimension)
        .collect()
}

fn path_format_1d(
    detected: Spectrum1DPathFormat,
    fallback_format: Option<&'static str>,
) -> &'static str {
    if let Some(format) = fallback_format {
        format
    } else {
        detected.as_str()
    }
}

fn path_format_2d(
    detected: Spectrum2DPathFormat,
    fallback_format: Option<&'static str>,
) -> &'static str {
    if let Some(format) = fallback_format {
        format
    } else {
        detected.as_str()
    }
}

fn no_discovery_inputs_error() -> RSpinError {
    RSpinError::Parse {
        format: "spectrum source discovery",
        message: "no input paths provided".to_owned(),
    }
}
