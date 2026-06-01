//! Dimension-specific source-path convenience loading from discovered sources.

use std::path::Path;

use rspin_core::Result;

use super::DiscoveredSpectrumSource;
use crate::bundle::{LoadedSourceFilter, SpectrumBundle, SpectrumBundleLoader};

/// Loads discovered source candidates matching one source path as a one-dimensional bundle.
///
/// # Errors
///
/// Returns an error when loading the matching one-dimensional discovered sources fails.
pub fn load_discovered_spectra_1d_by_source_path_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    path: impl AsRef<Path>,
) -> Result<SpectrumBundle> {
    SpectrumBundleLoader::new()
        .read_discovered_bundle_1d_by_source_path_relative_to(base, sources, path)
}

/// Loads discovered source candidates matching one source path as a one-dimensional bundle.
///
/// This short alias mirrors [`load_discovered_spectra_1d_by_source_path_relative_to`].
///
/// # Errors
///
/// Returns an error when loading the matching one-dimensional discovered sources fails.
pub fn load_discovered_spectra_1d_by_source_path<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    path: impl AsRef<Path>,
) -> Result<SpectrumBundle> {
    load_discovered_spectra_1d_by_source_path_relative_to(base, sources, path)
}

/// Loads discovered source candidates below one source path prefix as a one-dimensional bundle.
///
/// # Errors
///
/// Returns an error when loading the matching one-dimensional discovered sources fails.
pub fn load_discovered_spectra_1d_by_source_path_prefix_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    path: impl AsRef<Path>,
) -> Result<SpectrumBundle> {
    SpectrumBundleLoader::new()
        .read_discovered_bundle_1d_by_source_path_prefix_relative_to(base, sources, path)
}

/// Loads discovered source candidates below one source path prefix as a one-dimensional bundle.
///
/// This short alias mirrors [`load_discovered_spectra_1d_by_source_path_prefix_relative_to`].
///
/// # Errors
///
/// Returns an error when loading the matching one-dimensional discovered sources fails.
pub fn load_discovered_spectra_1d_by_source_path_prefix<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    path: impl AsRef<Path>,
) -> Result<SpectrumBundle> {
    load_discovered_spectra_1d_by_source_path_prefix_relative_to(base, sources, path)
}

/// Strictly loads discovered source candidates matching one source path as a one-dimensional bundle.
///
/// # Errors
///
/// Returns an error when strict loading the matching one-dimensional discovered sources fails.
pub fn load_discovered_spectra_1d_strict_by_source_path_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    path: impl AsRef<Path>,
) -> Result<SpectrumBundle> {
    SpectrumBundleLoader::new()
        .strict()
        .read_discovered_bundle_1d_by_source_path_relative_to(base, sources, path)
}

/// Strictly loads discovered source candidates matching one source path as a one-dimensional bundle.
///
/// This short alias mirrors [`load_discovered_spectra_1d_strict_by_source_path_relative_to`].
///
/// # Errors
///
/// Returns an error when strict loading the matching one-dimensional discovered sources fails.
pub fn load_discovered_spectra_1d_strict_by_source_path<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    path: impl AsRef<Path>,
) -> Result<SpectrumBundle> {
    load_discovered_spectra_1d_strict_by_source_path_relative_to(base, sources, path)
}

/// Strictly loads discovered source candidates below one source path prefix as a one-dimensional bundle.
///
/// # Errors
///
/// Returns an error when strict loading the matching one-dimensional discovered sources fails.
pub fn load_discovered_spectra_1d_strict_by_source_path_prefix_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    path: impl AsRef<Path>,
) -> Result<SpectrumBundle> {
    SpectrumBundleLoader::new()
        .strict()
        .read_discovered_bundle_1d_by_source_path_prefix_relative_to(base, sources, path)
}

/// Strictly loads discovered source candidates below one source path prefix as a one-dimensional bundle.
///
/// This short alias mirrors [`load_discovered_spectra_1d_strict_by_source_path_prefix_relative_to`].
///
/// # Errors
///
/// Returns an error when strict loading the matching one-dimensional discovered sources fails.
pub fn load_discovered_spectra_1d_strict_by_source_path_prefix<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    path: impl AsRef<Path>,
) -> Result<SpectrumBundle> {
    load_discovered_spectra_1d_strict_by_source_path_prefix_relative_to(base, sources, path)
}

/// Loads discovered source candidates matching one source path as a two-dimensional bundle.
///
/// # Errors
///
/// Returns an error when loading the matching two-dimensional discovered sources fails.
pub fn load_discovered_spectra_2d_by_source_path_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    path: impl AsRef<Path>,
) -> Result<SpectrumBundle> {
    SpectrumBundleLoader::new()
        .read_discovered_bundle_2d_by_source_path_relative_to(base, sources, path)
}

/// Loads discovered source candidates matching one source path as a two-dimensional bundle.
///
/// This short alias mirrors [`load_discovered_spectra_2d_by_source_path_relative_to`].
///
/// # Errors
///
/// Returns an error when loading the matching two-dimensional discovered sources fails.
pub fn load_discovered_spectra_2d_by_source_path<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    path: impl AsRef<Path>,
) -> Result<SpectrumBundle> {
    load_discovered_spectra_2d_by_source_path_relative_to(base, sources, path)
}

/// Loads discovered source candidates below one source path prefix as a two-dimensional bundle.
///
/// # Errors
///
/// Returns an error when loading the matching two-dimensional discovered sources fails.
pub fn load_discovered_spectra_2d_by_source_path_prefix_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    path: impl AsRef<Path>,
) -> Result<SpectrumBundle> {
    SpectrumBundleLoader::new()
        .read_discovered_bundle_2d_by_source_path_prefix_relative_to(base, sources, path)
}

/// Loads discovered source candidates below one source path prefix as a two-dimensional bundle.
///
/// This short alias mirrors [`load_discovered_spectra_2d_by_source_path_prefix_relative_to`].
///
/// # Errors
///
/// Returns an error when loading the matching two-dimensional discovered sources fails.
pub fn load_discovered_spectra_2d_by_source_path_prefix<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    path: impl AsRef<Path>,
) -> Result<SpectrumBundle> {
    load_discovered_spectra_2d_by_source_path_prefix_relative_to(base, sources, path)
}

/// Strictly loads discovered source candidates matching one source path as a two-dimensional bundle.
///
/// # Errors
///
/// Returns an error when strict loading the matching two-dimensional discovered sources fails.
pub fn load_discovered_spectra_2d_strict_by_source_path_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    path: impl AsRef<Path>,
) -> Result<SpectrumBundle> {
    SpectrumBundleLoader::new()
        .strict()
        .read_discovered_bundle_2d_by_source_path_relative_to(base, sources, path)
}

/// Strictly loads discovered source candidates matching one source path as a two-dimensional bundle.
///
/// This short alias mirrors [`load_discovered_spectra_2d_strict_by_source_path_relative_to`].
///
/// # Errors
///
/// Returns an error when strict loading the matching two-dimensional discovered sources fails.
pub fn load_discovered_spectra_2d_strict_by_source_path<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    path: impl AsRef<Path>,
) -> Result<SpectrumBundle> {
    load_discovered_spectra_2d_strict_by_source_path_relative_to(base, sources, path)
}

/// Strictly loads discovered source candidates below one source path prefix as a two-dimensional bundle.
///
/// # Errors
///
/// Returns an error when strict loading the matching two-dimensional discovered sources fails.
pub fn load_discovered_spectra_2d_strict_by_source_path_prefix_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    path: impl AsRef<Path>,
) -> Result<SpectrumBundle> {
    SpectrumBundleLoader::new()
        .strict()
        .read_discovered_bundle_2d_by_source_path_prefix_relative_to(base, sources, path)
}

/// Strictly loads discovered source candidates below one source path prefix as a two-dimensional bundle.
///
/// This short alias mirrors [`load_discovered_spectra_2d_strict_by_source_path_prefix_relative_to`].
///
/// # Errors
///
/// Returns an error when strict loading the matching two-dimensional discovered sources fails.
pub fn load_discovered_spectra_2d_strict_by_source_path_prefix<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    path: impl AsRef<Path>,
) -> Result<SpectrumBundle> {
    load_discovered_spectra_2d_strict_by_source_path_prefix_relative_to(base, sources, path)
}

impl SpectrumBundleLoader {
    /// Loads discovered source candidates matching one source path as a one-dimensional bundle.
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching one-dimensional discovered sources fails.
    pub fn read_discovered_bundle_1d_by_source_path_relative_to<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        path: impl AsRef<Path>,
    ) -> Result<SpectrumBundle> {
        self.read_discovered_bundle_1d_by_source_relative_to(
            base,
            sources,
            LoadedSourceFilter::path(path),
        )
    }

    /// Loads discovered source candidates matching one source path as a one-dimensional bundle.
    ///
    /// This short alias mirrors [`Self::read_discovered_bundle_1d_by_source_path_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching one-dimensional discovered sources fails.
    pub fn read_discovered_bundle_1d_by_source_path<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        path: impl AsRef<Path>,
    ) -> Result<SpectrumBundle> {
        self.read_discovered_bundle_1d_by_source_path_relative_to(base, sources, path)
    }

    /// Loads discovered source candidates below one source path prefix as a one-dimensional bundle.
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching one-dimensional discovered sources fails.
    pub fn read_discovered_bundle_1d_by_source_path_prefix_relative_to<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        path: impl AsRef<Path>,
    ) -> Result<SpectrumBundle> {
        self.read_discovered_bundle_1d_by_source_relative_to(
            base,
            sources,
            LoadedSourceFilter::path_prefix(path),
        )
    }

    /// Loads discovered source candidates below one source path prefix as a one-dimensional bundle.
    ///
    /// This short alias mirrors [`Self::read_discovered_bundle_1d_by_source_path_prefix_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching one-dimensional discovered sources fails.
    pub fn read_discovered_bundle_1d_by_source_path_prefix<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        path: impl AsRef<Path>,
    ) -> Result<SpectrumBundle> {
        self.read_discovered_bundle_1d_by_source_path_prefix_relative_to(base, sources, path)
    }

    /// Loads discovered source candidates matching one source path as a two-dimensional bundle.
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching two-dimensional discovered sources fails.
    pub fn read_discovered_bundle_2d_by_source_path_relative_to<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        path: impl AsRef<Path>,
    ) -> Result<SpectrumBundle> {
        self.read_discovered_bundle_2d_by_source_relative_to(
            base,
            sources,
            LoadedSourceFilter::path(path),
        )
    }

    /// Loads discovered source candidates matching one source path as a two-dimensional bundle.
    ///
    /// This short alias mirrors [`Self::read_discovered_bundle_2d_by_source_path_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching two-dimensional discovered sources fails.
    pub fn read_discovered_bundle_2d_by_source_path<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        path: impl AsRef<Path>,
    ) -> Result<SpectrumBundle> {
        self.read_discovered_bundle_2d_by_source_path_relative_to(base, sources, path)
    }

    /// Loads discovered source candidates below one source path prefix as a two-dimensional bundle.
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching two-dimensional discovered sources fails.
    pub fn read_discovered_bundle_2d_by_source_path_prefix_relative_to<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        path: impl AsRef<Path>,
    ) -> Result<SpectrumBundle> {
        self.read_discovered_bundle_2d_by_source_relative_to(
            base,
            sources,
            LoadedSourceFilter::path_prefix(path),
        )
    }

    /// Loads discovered source candidates below one source path prefix as a two-dimensional bundle.
    ///
    /// This short alias mirrors [`Self::read_discovered_bundle_2d_by_source_path_prefix_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching two-dimensional discovered sources fails.
    pub fn read_discovered_bundle_2d_by_source_path_prefix<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        path: impl AsRef<Path>,
    ) -> Result<SpectrumBundle> {
        self.read_discovered_bundle_2d_by_source_path_prefix_relative_to(base, sources, path)
    }
}
