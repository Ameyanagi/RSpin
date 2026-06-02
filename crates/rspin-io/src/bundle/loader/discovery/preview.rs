//! User-facing preview and scan aliases for source discovery.

use std::path::Path;

use rspin_core::Result;

use super::{DiscoveredSpectrumSource, DiscoveredSpectrumSummary};
use crate::bundle::SpectrumBundleLoader;

/// Discovers source candidates below a file or directory without loading spectra.
///
/// This is a user-facing alias for `discover_spectra`.
///
/// # Errors
///
/// Returns an error when the input path is missing or a directory cannot be read.
pub fn scan_spectra(path: impl AsRef<Path>) -> Result<Vec<DiscoveredSpectrumSource>> {
    SpectrumBundleLoader::new().discover_path(path)
}

/// Discovers and summarizes source candidates below a file or directory.
///
/// This is a user-facing alias for `discover_spectra_summary`.
///
/// # Errors
///
/// Returns an error when the input path is missing or a directory cannot be read.
pub fn preview_spectra(path: impl AsRef<Path>) -> Result<DiscoveredSpectrumSummary> {
    SpectrumBundleLoader::new().discover_summary_path(path)
}

/// Discovers source candidates from one selected path relative to a base directory.
///
/// This is a user-facing alias for `discover_spectra_relative_to`.
///
/// # Errors
///
/// Returns an error when the base directory or selected path is missing.
pub fn scan_spectra_relative_to(
    base: impl AsRef<Path>,
    path: impl AsRef<Path>,
) -> Result<Vec<DiscoveredSpectrumSource>> {
    SpectrumBundleLoader::new().discover_path_relative_to(base, path)
}

/// Discovers and summarizes one selected path relative to a base directory.
///
/// This is a user-facing alias for `discover_spectra_summary_relative_to`.
///
/// # Errors
///
/// Returns an error when the base directory or selected path is missing.
pub fn preview_spectra_relative_to(
    base: impl AsRef<Path>,
    path: impl AsRef<Path>,
) -> Result<DiscoveredSpectrumSummary> {
    SpectrumBundleLoader::new().discover_summary_path_relative_to(base, path)
}

/// Discovers source candidates below multiple file or directory paths.
///
/// This is a user-facing alias for `discover_spectra_many`.
///
/// # Errors
///
/// Returns an error when no paths are provided or an input path is missing.
pub fn scan_spectra_many<I, P>(paths: I) -> Result<Vec<DiscoveredSpectrumSource>>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().discover_paths(paths)
}

/// Discovers and summarizes source candidates below multiple file or directory paths.
///
/// This is a user-facing alias for `discover_spectra_many_summary`.
///
/// # Errors
///
/// Returns an error when no paths are provided or an input path is missing.
pub fn preview_spectra_many<I, P>(paths: I) -> Result<DiscoveredSpectrumSummary>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().discover_summary_paths(paths)
}

/// Discovers source candidates from selected paths relative to a base directory.
///
/// This is a user-facing alias for `discover_spectra_many_relative_to`.
///
/// # Errors
///
/// Returns an error when the base directory is missing, no selected paths are
/// provided, or a selected path is missing.
pub fn scan_spectra_many_relative_to<I, P>(
    base: impl AsRef<Path>,
    paths: I,
) -> Result<Vec<DiscoveredSpectrumSource>>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().discover_paths_relative_to(base, paths)
}

/// Discovers and summarizes selected paths relative to a base directory.
///
/// This is a user-facing alias for `discover_spectra_many_summary_relative_to`.
///
/// # Errors
///
/// Returns an error when the base directory is missing, no selected paths are
/// provided, or a selected path is missing.
pub fn preview_spectra_many_relative_to<I, P>(
    base: impl AsRef<Path>,
    paths: I,
) -> Result<DiscoveredSpectrumSummary>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().discover_summary_paths_relative_to(base, paths)
}

impl SpectrumBundleLoader {
    /// Discovers source candidates below a file or directory.
    ///
    /// This is a user-facing alias for [`Self::discover_path`].
    ///
    /// # Errors
    ///
    /// Returns an error when the input path is missing or a directory cannot be read.
    pub fn scan(&self, path: impl AsRef<Path>) -> Result<Vec<DiscoveredSpectrumSource>> {
        self.discover_path(path)
    }

    /// Discovers and summarizes source candidates below a file or directory.
    ///
    /// This is a user-facing alias for [`Self::discover_summary_path`].
    ///
    /// # Errors
    ///
    /// Returns an error when the input path is missing or a directory cannot be read.
    pub fn preview(&self, path: impl AsRef<Path>) -> Result<DiscoveredSpectrumSummary> {
        self.discover_summary_path(path)
    }

    /// Discovers one-dimensional source candidates below a file or directory.
    ///
    /// This is a user-facing alias for [`Self::discover_1d_path`].
    ///
    /// # Errors
    ///
    /// Returns an error when the input path is missing or a directory cannot be read.
    pub fn scan_1d(&self, path: impl AsRef<Path>) -> Result<Vec<DiscoveredSpectrumSource>> {
        self.discover_1d_path(path)
    }

    /// Discovers two-dimensional source candidates below a file or directory.
    ///
    /// This is a user-facing alias for [`Self::discover_2d_path`].
    ///
    /// # Errors
    ///
    /// Returns an error when the input path is missing or a directory cannot be read.
    pub fn scan_2d(&self, path: impl AsRef<Path>) -> Result<Vec<DiscoveredSpectrumSource>> {
        self.discover_2d_path(path)
    }

    /// Discovers and summarizes one-dimensional source candidates below a file or directory.
    ///
    /// This is a user-facing alias for [`Self::discover_1d_summary_path`].
    ///
    /// # Errors
    ///
    /// Returns an error when the input path is missing or a directory cannot be read.
    pub fn preview_1d(&self, path: impl AsRef<Path>) -> Result<DiscoveredSpectrumSummary> {
        self.discover_1d_summary_path(path)
    }

    /// Discovers and summarizes two-dimensional source candidates below a file or directory.
    ///
    /// This is a user-facing alias for [`Self::discover_2d_summary_path`].
    ///
    /// # Errors
    ///
    /// Returns an error when the input path is missing or a directory cannot be read.
    pub fn preview_2d(&self, path: impl AsRef<Path>) -> Result<DiscoveredSpectrumSummary> {
        self.discover_2d_summary_path(path)
    }

    /// Discovers one selected path while anchoring source paths to a base directory.
    ///
    /// This is a user-facing alias for [`Self::discover_path_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when the base directory or selected path is missing.
    pub fn scan_relative_to(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
    ) -> Result<Vec<DiscoveredSpectrumSource>> {
        self.discover_path_relative_to(base, path)
    }

    /// Discovers and summarizes one selected path while anchoring source paths to a base directory.
    ///
    /// This is a user-facing alias for [`Self::discover_summary_path_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when the base directory or selected path is missing.
    pub fn preview_relative_to(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
    ) -> Result<DiscoveredSpectrumSummary> {
        self.discover_summary_path_relative_to(base, path)
    }

    /// Discovers one-dimensional source candidates from one selected path relative to a base directory.
    ///
    /// This is a user-facing alias for [`Self::discover_1d_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when the base directory or selected path is missing.
    pub fn scan_1d_relative_to(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
    ) -> Result<Vec<DiscoveredSpectrumSource>> {
        self.discover_1d_relative_to(base, path)
    }

    /// Discovers two-dimensional source candidates from one selected path relative to a base directory.
    ///
    /// This is a user-facing alias for [`Self::discover_2d_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when the base directory or selected path is missing.
    pub fn scan_2d_relative_to(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
    ) -> Result<Vec<DiscoveredSpectrumSource>> {
        self.discover_2d_relative_to(base, path)
    }

    /// Discovers and summarizes one-dimensional source candidates from one selected path relative to a base directory.
    ///
    /// This is a user-facing alias for [`Self::discover_1d_summary_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when the base directory or selected path is missing.
    pub fn preview_1d_relative_to(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
    ) -> Result<DiscoveredSpectrumSummary> {
        self.discover_1d_summary_relative_to(base, path)
    }

    /// Discovers and summarizes two-dimensional source candidates from one selected path relative to a base directory.
    ///
    /// This is a user-facing alias for [`Self::discover_2d_summary_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when the base directory or selected path is missing.
    pub fn preview_2d_relative_to(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
    ) -> Result<DiscoveredSpectrumSummary> {
        self.discover_2d_summary_relative_to(base, path)
    }

    /// Discovers source candidates below multiple file or directory paths.
    ///
    /// This is a user-facing alias for [`Self::discover_paths`].
    ///
    /// # Errors
    ///
    /// Returns an error when no paths are provided or an input path is missing.
    pub fn scan_many<I, P>(&self, paths: I) -> Result<Vec<DiscoveredSpectrumSource>>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.discover_paths(paths)
    }

    /// Discovers and summarizes source candidates below multiple file or directory paths.
    ///
    /// This is a user-facing alias for [`Self::discover_summary_paths`].
    ///
    /// # Errors
    ///
    /// Returns an error when no paths are provided or an input path is missing.
    pub fn preview_many<I, P>(&self, paths: I) -> Result<DiscoveredSpectrumSummary>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.discover_summary_paths(paths)
    }

    /// Discovers selected paths while anchoring source paths to a base directory.
    ///
    /// This is a user-facing alias for [`Self::discover_paths_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when the base directory is missing, no selected paths
    /// are provided, or a selected path is missing.
    pub fn scan_many_relative_to<I, P>(
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
    /// This is a user-facing alias for [`Self::discover_summary_paths_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when the base directory is missing, no selected paths
    /// are provided, or a selected path is missing.
    pub fn preview_many_relative_to<I, P>(
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
}
