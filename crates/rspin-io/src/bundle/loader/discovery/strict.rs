//! Strict convenience loading from discovered source candidates.

use std::path::Path;

use rspin_core::Result;

use super::DiscoveredSpectrumSource;
use crate::bundle::{LoadedSourceFilter, SpectrumBundle, SpectrumBundleLoader};

/// Strictly loads selected discovered source candidates relative to a common base directory.
///
/// # Errors
///
/// Returns an error when `base` is missing or not a directory, no discovered
/// sources are provided, a selected source does not include a tracked source
/// path, a selected source cannot be read, or no selected source can be read.
pub fn load_discovered_spectra_strict_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
) -> Result<SpectrumBundle> {
    SpectrumBundleLoader::new()
        .strict()
        .read_discovered_relative_to(base, sources)
}

/// Strictly loads selected discovered source candidates relative to a common base directory.
///
/// This short alias mirrors [`load_discovered_spectra_strict_relative_to`].
///
/// # Errors
///
/// Returns an error when strict loading the selected discovered sources fails.
pub fn load_discovered_spectra_strict<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
) -> Result<SpectrumBundle> {
    load_discovered_spectra_strict_relative_to(base, sources)
}

/// Strictly loads discovered source candidates matching one generic source filter.
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_strict_by_source_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    filter: impl Into<LoadedSourceFilter>,
) -> Result<SpectrumBundle> {
    SpectrumBundleLoader::new()
        .strict()
        .read_discovered_by_source_relative_to(base, sources, filter)
}

/// Strictly loads discovered source candidates matching one generic source filter.
///
/// This short alias mirrors [`load_discovered_spectra_strict_by_source_relative_to`].
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_strict_by_source<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    filter: impl Into<LoadedSourceFilter>,
) -> Result<SpectrumBundle> {
    load_discovered_spectra_strict_by_source_relative_to(base, sources, filter)
}

/// Strictly loads discovered source candidates matching one source path.
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_strict_by_source_path_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    path: impl AsRef<Path>,
) -> Result<SpectrumBundle> {
    SpectrumBundleLoader::new()
        .strict()
        .read_discovered_by_source_path_relative_to(base, sources, path)
}

/// Strictly loads discovered source candidates matching one source path.
///
/// This short alias mirrors [`load_discovered_spectra_strict_by_source_path_relative_to`].
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_strict_by_source_path<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    path: impl AsRef<Path>,
) -> Result<SpectrumBundle> {
    load_discovered_spectra_strict_by_source_path_relative_to(base, sources, path)
}

/// Strictly loads discovered source candidates below one source path prefix.
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_strict_by_source_path_prefix_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    path: impl AsRef<Path>,
) -> Result<SpectrumBundle> {
    SpectrumBundleLoader::new()
        .strict()
        .read_discovered_by_source_path_prefix_relative_to(base, sources, path)
}

/// Strictly loads discovered source candidates below one source path prefix.
///
/// This short alias mirrors [`load_discovered_spectra_strict_by_source_path_prefix_relative_to`].
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_strict_by_source_path_prefix<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    path: impl AsRef<Path>,
) -> Result<SpectrumBundle> {
    load_discovered_spectra_strict_by_source_path_prefix_relative_to(base, sources, path)
}

/// Strictly loads discovered source candidates matching any generic source filter.
///
/// Filters are combined with logical OR. Passing an empty iterator strictly
/// loads all provided discovered sources.
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_strict_by_sources_relative_to<'a, I, F>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    filters: I,
) -> Result<SpectrumBundle>
where
    I: IntoIterator<Item = F>,
    F: Into<LoadedSourceFilter>,
{
    SpectrumBundleLoader::new()
        .strict()
        .read_discovered_by_sources_relative_to(base, sources, filters)
}

/// Strictly loads discovered source candidates matching any generic source filter.
///
/// This short alias mirrors [`load_discovered_spectra_strict_by_sources_relative_to`].
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_strict_by_sources<'a, I, F>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    filters: I,
) -> Result<SpectrumBundle>
where
    I: IntoIterator<Item = F>,
    F: Into<LoadedSourceFilter>,
{
    load_discovered_spectra_strict_by_sources_relative_to(base, sources, filters)
}
