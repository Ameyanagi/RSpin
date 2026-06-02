//! Source-path convenience summary loading from discovered source candidates.

use std::path::Path;

use rspin_core::Result;

use super::DiscoveredSpectrumSource;
use crate::bundle::{LoadedSourceFilter, SpectrumBundleLoader, SpectrumBundleSummary};

/// Loads discovered source candidates matching one source path and returns summary counts.
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_by_source_path_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    path: impl AsRef<Path>,
) -> Result<SpectrumBundleSummary> {
    SpectrumBundleLoader::new()
        .read_discovered_summary_by_source_path_relative_to(base, sources, path)
}

/// Loads discovered source candidates matching one source path and returns summary counts.
///
/// This short alias mirrors [`load_discovered_spectra_summary_by_source_path_relative_to`].
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_by_source_path<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    path: impl AsRef<Path>,
) -> Result<SpectrumBundleSummary> {
    load_discovered_spectra_summary_by_source_path_relative_to(base, sources, path)
}

/// Loads discovered source candidates matching any source path and returns summary counts.
///
/// Paths are combined with logical OR. Passing an empty iterator leaves source
/// matching unrestricted.
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_by_source_paths_relative_to<'a, I, P>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    paths: I,
) -> Result<SpectrumBundleSummary>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new()
        .read_discovered_summary_by_source_paths_relative_to(base, sources, paths)
}

/// Loads discovered source candidates matching any source path and returns summary counts.
///
/// This short alias mirrors [`load_discovered_spectra_summary_by_source_paths_relative_to`].
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_by_source_paths<'a, I, P>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    paths: I,
) -> Result<SpectrumBundleSummary>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    load_discovered_spectra_summary_by_source_paths_relative_to(base, sources, paths)
}

/// Loads discovered source candidates below one source path prefix and returns summary counts.
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_by_source_path_prefix_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    path: impl AsRef<Path>,
) -> Result<SpectrumBundleSummary> {
    SpectrumBundleLoader::new()
        .read_discovered_summary_by_source_path_prefix_relative_to(base, sources, path)
}

/// Loads discovered source candidates below one source path prefix and returns summary counts.
///
/// This short alias mirrors [`load_discovered_spectra_summary_by_source_path_prefix_relative_to`].
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_by_source_path_prefix<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    path: impl AsRef<Path>,
) -> Result<SpectrumBundleSummary> {
    load_discovered_spectra_summary_by_source_path_prefix_relative_to(base, sources, path)
}

/// Loads discovered source candidates below any source path prefix and returns summary counts.
///
/// Prefixes are combined with logical OR. Passing an empty iterator leaves
/// source matching unrestricted.
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_by_source_path_prefixes_relative_to<'a, I, P>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    paths: I,
) -> Result<SpectrumBundleSummary>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new()
        .read_discovered_summary_by_source_path_prefixes_relative_to(base, sources, paths)
}

/// Loads discovered source candidates below any source path prefix and returns summary counts.
///
/// This short alias mirrors [`load_discovered_spectra_summary_by_source_path_prefixes_relative_to`].
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_by_source_path_prefixes<'a, I, P>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    paths: I,
) -> Result<SpectrumBundleSummary>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    load_discovered_spectra_summary_by_source_path_prefixes_relative_to(base, sources, paths)
}

/// Strictly loads discovered source candidates matching one source path and returns summary counts.
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_strict_by_source_path_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    path: impl AsRef<Path>,
) -> Result<SpectrumBundleSummary> {
    SpectrumBundleLoader::new()
        .strict()
        .read_discovered_summary_by_source_path_relative_to(base, sources, path)
}

/// Strictly loads discovered source candidates matching one source path and returns summary counts.
///
/// This short alias mirrors [`load_discovered_spectra_summary_strict_by_source_path_relative_to`].
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_strict_by_source_path<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    path: impl AsRef<Path>,
) -> Result<SpectrumBundleSummary> {
    load_discovered_spectra_summary_strict_by_source_path_relative_to(base, sources, path)
}

/// Strictly loads discovered source candidates matching any source path and returns summary counts.
///
/// Paths are combined with logical OR. Passing an empty iterator leaves source
/// matching unrestricted.
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_strict_by_source_paths_relative_to<'a, I, P>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    paths: I,
) -> Result<SpectrumBundleSummary>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new()
        .strict()
        .read_discovered_summary_by_source_paths_relative_to(base, sources, paths)
}

/// Strictly loads discovered source candidates matching any source path and returns summary counts.
///
/// This short alias mirrors [`load_discovered_spectra_summary_strict_by_source_paths_relative_to`].
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_strict_by_source_paths<'a, I, P>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    paths: I,
) -> Result<SpectrumBundleSummary>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    load_discovered_spectra_summary_strict_by_source_paths_relative_to(base, sources, paths)
}

/// Strictly loads discovered source candidates below one source path prefix and returns summary counts.
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_strict_by_source_path_prefix_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    path: impl AsRef<Path>,
) -> Result<SpectrumBundleSummary> {
    SpectrumBundleLoader::new()
        .strict()
        .read_discovered_summary_by_source_path_prefix_relative_to(base, sources, path)
}

/// Strictly loads discovered source candidates below one source path prefix and returns summary counts.
///
/// This short alias mirrors [`load_discovered_spectra_summary_strict_by_source_path_prefix_relative_to`].
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_strict_by_source_path_prefix<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    path: impl AsRef<Path>,
) -> Result<SpectrumBundleSummary> {
    load_discovered_spectra_summary_strict_by_source_path_prefix_relative_to(base, sources, path)
}

/// Strictly loads discovered source candidates below any source path prefix and returns summary counts.
///
/// Prefixes are combined with logical OR. Passing an empty iterator leaves
/// source matching unrestricted.
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_strict_by_source_path_prefixes_relative_to<'a, I, P>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    paths: I,
) -> Result<SpectrumBundleSummary>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new()
        .strict()
        .read_discovered_summary_by_source_path_prefixes_relative_to(base, sources, paths)
}

/// Strictly loads discovered source candidates below any source path prefix and returns summary counts.
///
/// This short alias mirrors [`load_discovered_spectra_summary_strict_by_source_path_prefixes_relative_to`].
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_strict_by_source_path_prefixes<'a, I, P>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    paths: I,
) -> Result<SpectrumBundleSummary>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    load_discovered_spectra_summary_strict_by_source_path_prefixes_relative_to(base, sources, paths)
}

impl SpectrumBundleLoader {
    /// Loads discovered source candidates matching one source path and returns summary counts.
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_summary_by_source_path_relative_to<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        path: impl AsRef<Path>,
    ) -> Result<SpectrumBundleSummary> {
        self.read_discovered_summary_by_source_relative_to(
            base,
            sources,
            LoadedSourceFilter::path(path),
        )
    }

    /// Loads discovered source candidates matching one source path and returns summary counts.
    ///
    /// This short alias mirrors [`Self::read_discovered_summary_by_source_path_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_summary_by_source_path<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        path: impl AsRef<Path>,
    ) -> Result<SpectrumBundleSummary> {
        self.read_discovered_summary_by_source_path_relative_to(base, sources, path)
    }

    /// Loads discovered source candidates matching any source path and returns summary counts.
    ///
    /// Paths are combined with logical OR. Passing an empty iterator leaves
    /// source matching unrestricted.
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_summary_by_source_paths_relative_to<'a, I, P>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        paths: I,
    ) -> Result<SpectrumBundleSummary>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_discovered_summary_by_sources_relative_to(base, sources, path_filters(paths))
    }

    /// Loads discovered source candidates matching any source path and returns summary counts.
    ///
    /// This short alias mirrors [`Self::read_discovered_summary_by_source_paths_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_summary_by_source_paths<'a, I, P>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        paths: I,
    ) -> Result<SpectrumBundleSummary>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_discovered_summary_by_source_paths_relative_to(base, sources, paths)
    }

    /// Loads discovered source candidates below one source path prefix and returns summary counts.
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_summary_by_source_path_prefix_relative_to<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        path: impl AsRef<Path>,
    ) -> Result<SpectrumBundleSummary> {
        self.read_discovered_summary_by_source_relative_to(
            base,
            sources,
            LoadedSourceFilter::path_prefix(path),
        )
    }

    /// Loads discovered source candidates below one source path prefix and returns summary counts.
    ///
    /// This short alias mirrors [`Self::read_discovered_summary_by_source_path_prefix_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_summary_by_source_path_prefix<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        path: impl AsRef<Path>,
    ) -> Result<SpectrumBundleSummary> {
        self.read_discovered_summary_by_source_path_prefix_relative_to(base, sources, path)
    }

    /// Loads discovered source candidates below any source path prefix and returns summary counts.
    ///
    /// Prefixes are combined with logical OR. Passing an empty iterator leaves
    /// source matching unrestricted.
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_summary_by_source_path_prefixes_relative_to<'a, I, P>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        paths: I,
    ) -> Result<SpectrumBundleSummary>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_discovered_summary_by_sources_relative_to(
            base,
            sources,
            path_prefix_filters(paths),
        )
    }

    /// Loads discovered source candidates below any source path prefix and returns summary counts.
    ///
    /// This short alias mirrors [`Self::read_discovered_summary_by_source_path_prefixes_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_summary_by_source_path_prefixes<'a, I, P>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        paths: I,
    ) -> Result<SpectrumBundleSummary>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_discovered_summary_by_source_path_prefixes_relative_to(base, sources, paths)
    }
}

fn path_filters<I, P>(paths: I) -> Vec<LoadedSourceFilter>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    let mut filters = Vec::new();
    for path in paths {
        filters.push(LoadedSourceFilter::path(path));
    }
    filters
}

fn path_prefix_filters<I, P>(paths: I) -> Vec<LoadedSourceFilter>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    let mut filters = Vec::new();
    for path in paths {
        filters.push(LoadedSourceFilter::path_prefix(path));
    }
    filters
}
