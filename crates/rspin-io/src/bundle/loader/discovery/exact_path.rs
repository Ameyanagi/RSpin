//! Exact source-path convenience loading from discovered source candidates.

use std::path::Path;

use rspin_core::{Result, Spectrum1D, Spectrum2D};

use super::DiscoveredSpectrumSource;
use crate::bundle::{LoadedSource, LoadedSourceFilter, SpectrumBundleLoader};

/// Loads discovered source candidates matching one source path as exactly one 1D spectrum.
///
/// # Errors
///
/// Returns an error when loading fails or the matching discovered sources do
/// not resolve to exactly one one-dimensional spectrum.
pub fn load_discovered_spectrum_1d_by_source_path_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    path: impl AsRef<Path>,
) -> Result<Spectrum1D> {
    SpectrumBundleLoader::new().read_discovered_1d_by_source_path_relative_to(base, sources, path)
}

/// Loads discovered source candidates matching one source path as exactly one 1D spectrum.
///
/// This short alias mirrors [`load_discovered_spectrum_1d_by_source_path_relative_to`].
///
/// # Errors
///
/// Returns an error when loading fails or the matching discovered sources do
/// not resolve to exactly one one-dimensional spectrum.
pub fn load_discovered_spectrum_1d_by_source_path<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    path: impl AsRef<Path>,
) -> Result<Spectrum1D> {
    load_discovered_spectrum_1d_by_source_path_relative_to(base, sources, path)
}

/// Loads discovered source candidates below one source path prefix as exactly one 1D spectrum.
///
/// # Errors
///
/// Returns an error when loading fails or the matching discovered sources do
/// not resolve to exactly one one-dimensional spectrum.
pub fn load_discovered_spectrum_1d_by_source_path_prefix_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    path: impl AsRef<Path>,
) -> Result<Spectrum1D> {
    SpectrumBundleLoader::new()
        .read_discovered_1d_by_source_path_prefix_relative_to(base, sources, path)
}

/// Loads discovered source candidates below one source path prefix as exactly one 1D spectrum.
///
/// This short alias mirrors [`load_discovered_spectrum_1d_by_source_path_prefix_relative_to`].
///
/// # Errors
///
/// Returns an error when loading fails or the matching discovered sources do
/// not resolve to exactly one one-dimensional spectrum.
pub fn load_discovered_spectrum_1d_by_source_path_prefix<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    path: impl AsRef<Path>,
) -> Result<Spectrum1D> {
    load_discovered_spectrum_1d_by_source_path_prefix_relative_to(base, sources, path)
}

/// Loads discovered source candidates matching one source path as exactly one 1D spectrum with source metadata.
///
/// # Errors
///
/// Returns an error when loading fails or the matching discovered sources do
/// not resolve to exactly one one-dimensional spectrum.
pub fn load_discovered_spectrum_1d_with_source_by_source_path_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    path: impl AsRef<Path>,
) -> Result<(Spectrum1D, LoadedSource)> {
    SpectrumBundleLoader::new()
        .read_discovered_1d_with_source_by_source_path_relative_to(base, sources, path)
}

/// Loads discovered source candidates matching one source path as exactly one 1D spectrum with source metadata.
///
/// This short alias mirrors [`load_discovered_spectrum_1d_with_source_by_source_path_relative_to`].
///
/// # Errors
///
/// Returns an error when loading fails or the matching discovered sources do
/// not resolve to exactly one one-dimensional spectrum.
pub fn load_discovered_spectrum_1d_with_source_by_source_path<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    path: impl AsRef<Path>,
) -> Result<(Spectrum1D, LoadedSource)> {
    load_discovered_spectrum_1d_with_source_by_source_path_relative_to(base, sources, path)
}

/// Loads discovered source candidates below one source path prefix as exactly one 1D spectrum with source metadata.
///
/// # Errors
///
/// Returns an error when loading fails or the matching discovered sources do
/// not resolve to exactly one one-dimensional spectrum.
pub fn load_discovered_spectrum_1d_with_source_by_source_path_prefix_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    path: impl AsRef<Path>,
) -> Result<(Spectrum1D, LoadedSource)> {
    SpectrumBundleLoader::new()
        .read_discovered_1d_with_source_by_source_path_prefix_relative_to(base, sources, path)
}

/// Loads discovered source candidates below one source path prefix as exactly one 1D spectrum with source metadata.
///
/// This short alias mirrors [`load_discovered_spectrum_1d_with_source_by_source_path_prefix_relative_to`].
///
/// # Errors
///
/// Returns an error when loading fails or the matching discovered sources do
/// not resolve to exactly one one-dimensional spectrum.
pub fn load_discovered_spectrum_1d_with_source_by_source_path_prefix<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    path: impl AsRef<Path>,
) -> Result<(Spectrum1D, LoadedSource)> {
    load_discovered_spectrum_1d_with_source_by_source_path_prefix_relative_to(base, sources, path)
}

/// Loads discovered source candidates matching one source path as exactly one 2D spectrum.
///
/// # Errors
///
/// Returns an error when loading fails or the matching discovered sources do
/// not resolve to exactly one two-dimensional spectrum.
pub fn load_discovered_spectrum_2d_by_source_path_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    path: impl AsRef<Path>,
) -> Result<Spectrum2D> {
    SpectrumBundleLoader::new().read_discovered_2d_by_source_path_relative_to(base, sources, path)
}

/// Loads discovered source candidates matching one source path as exactly one 2D spectrum.
///
/// This short alias mirrors [`load_discovered_spectrum_2d_by_source_path_relative_to`].
///
/// # Errors
///
/// Returns an error when loading fails or the matching discovered sources do
/// not resolve to exactly one two-dimensional spectrum.
pub fn load_discovered_spectrum_2d_by_source_path<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    path: impl AsRef<Path>,
) -> Result<Spectrum2D> {
    load_discovered_spectrum_2d_by_source_path_relative_to(base, sources, path)
}

/// Loads discovered source candidates below one source path prefix as exactly one 2D spectrum.
///
/// # Errors
///
/// Returns an error when loading fails or the matching discovered sources do
/// not resolve to exactly one two-dimensional spectrum.
pub fn load_discovered_spectrum_2d_by_source_path_prefix_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    path: impl AsRef<Path>,
) -> Result<Spectrum2D> {
    SpectrumBundleLoader::new()
        .read_discovered_2d_by_source_path_prefix_relative_to(base, sources, path)
}

/// Loads discovered source candidates below one source path prefix as exactly one 2D spectrum.
///
/// This short alias mirrors [`load_discovered_spectrum_2d_by_source_path_prefix_relative_to`].
///
/// # Errors
///
/// Returns an error when loading fails or the matching discovered sources do
/// not resolve to exactly one two-dimensional spectrum.
pub fn load_discovered_spectrum_2d_by_source_path_prefix<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    path: impl AsRef<Path>,
) -> Result<Spectrum2D> {
    load_discovered_spectrum_2d_by_source_path_prefix_relative_to(base, sources, path)
}

/// Loads discovered source candidates matching one source path as exactly one 2D spectrum with source metadata.
///
/// # Errors
///
/// Returns an error when loading fails or the matching discovered sources do
/// not resolve to exactly one two-dimensional spectrum.
pub fn load_discovered_spectrum_2d_with_source_by_source_path_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    path: impl AsRef<Path>,
) -> Result<(Spectrum2D, LoadedSource)> {
    SpectrumBundleLoader::new()
        .read_discovered_2d_with_source_by_source_path_relative_to(base, sources, path)
}

/// Loads discovered source candidates matching one source path as exactly one 2D spectrum with source metadata.
///
/// This short alias mirrors [`load_discovered_spectrum_2d_with_source_by_source_path_relative_to`].
///
/// # Errors
///
/// Returns an error when loading fails or the matching discovered sources do
/// not resolve to exactly one two-dimensional spectrum.
pub fn load_discovered_spectrum_2d_with_source_by_source_path<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    path: impl AsRef<Path>,
) -> Result<(Spectrum2D, LoadedSource)> {
    load_discovered_spectrum_2d_with_source_by_source_path_relative_to(base, sources, path)
}

/// Loads discovered source candidates below one source path prefix as exactly one 2D spectrum with source metadata.
///
/// # Errors
///
/// Returns an error when loading fails or the matching discovered sources do
/// not resolve to exactly one two-dimensional spectrum.
pub fn load_discovered_spectrum_2d_with_source_by_source_path_prefix_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    path: impl AsRef<Path>,
) -> Result<(Spectrum2D, LoadedSource)> {
    SpectrumBundleLoader::new()
        .read_discovered_2d_with_source_by_source_path_prefix_relative_to(base, sources, path)
}

/// Loads discovered source candidates below one source path prefix as exactly one 2D spectrum with source metadata.
///
/// This short alias mirrors [`load_discovered_spectrum_2d_with_source_by_source_path_prefix_relative_to`].
///
/// # Errors
///
/// Returns an error when loading fails or the matching discovered sources do
/// not resolve to exactly one two-dimensional spectrum.
pub fn load_discovered_spectrum_2d_with_source_by_source_path_prefix<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    path: impl AsRef<Path>,
) -> Result<(Spectrum2D, LoadedSource)> {
    load_discovered_spectrum_2d_with_source_by_source_path_prefix_relative_to(base, sources, path)
}

impl SpectrumBundleLoader {
    /// Loads discovered source candidates matching one source path as exactly one 1D spectrum.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the matching discovered sources
    /// do not resolve to exactly one one-dimensional spectrum.
    pub fn read_discovered_1d_by_source_path_relative_to<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        path: impl AsRef<Path>,
    ) -> Result<Spectrum1D> {
        self.read_discovered_1d_by_source_relative_to(base, sources, LoadedSourceFilter::path(path))
    }

    /// Loads discovered source candidates matching one source path as exactly one 1D spectrum.
    ///
    /// This short alias mirrors [`Self::read_discovered_1d_by_source_path_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the matching discovered sources
    /// do not resolve to exactly one one-dimensional spectrum.
    pub fn read_discovered_1d_by_source_path<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        path: impl AsRef<Path>,
    ) -> Result<Spectrum1D> {
        self.read_discovered_1d_by_source_path_relative_to(base, sources, path)
    }

    /// Loads discovered source candidates below one source path prefix as exactly one 1D spectrum.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the matching discovered sources
    /// do not resolve to exactly one one-dimensional spectrum.
    pub fn read_discovered_1d_by_source_path_prefix_relative_to<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        path: impl AsRef<Path>,
    ) -> Result<Spectrum1D> {
        self.read_discovered_1d_by_source_relative_to(
            base,
            sources,
            LoadedSourceFilter::path_prefix(path),
        )
    }

    /// Loads discovered source candidates below one source path prefix as exactly one 1D spectrum.
    ///
    /// This short alias mirrors [`Self::read_discovered_1d_by_source_path_prefix_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the matching discovered sources
    /// do not resolve to exactly one one-dimensional spectrum.
    pub fn read_discovered_1d_by_source_path_prefix<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        path: impl AsRef<Path>,
    ) -> Result<Spectrum1D> {
        self.read_discovered_1d_by_source_path_prefix_relative_to(base, sources, path)
    }

    /// Loads discovered source candidates matching one source path as exactly one 1D spectrum with source metadata.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the matching discovered sources
    /// do not resolve to exactly one one-dimensional spectrum.
    pub fn read_discovered_1d_with_source_by_source_path_relative_to<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        path: impl AsRef<Path>,
    ) -> Result<(Spectrum1D, LoadedSource)> {
        self.read_discovered_1d_with_source_by_source_relative_to(
            base,
            sources,
            LoadedSourceFilter::path(path),
        )
    }

    /// Loads discovered source candidates matching one source path as exactly one 1D spectrum with source metadata.
    ///
    /// This short alias mirrors [`Self::read_discovered_1d_with_source_by_source_path_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the matching discovered sources
    /// do not resolve to exactly one one-dimensional spectrum.
    pub fn read_discovered_1d_with_source_by_source_path<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        path: impl AsRef<Path>,
    ) -> Result<(Spectrum1D, LoadedSource)> {
        self.read_discovered_1d_with_source_by_source_path_relative_to(base, sources, path)
    }

    /// Loads discovered source candidates below one source path prefix as exactly one 1D spectrum with source metadata.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the matching discovered sources
    /// do not resolve to exactly one one-dimensional spectrum.
    pub fn read_discovered_1d_with_source_by_source_path_prefix_relative_to<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        path: impl AsRef<Path>,
    ) -> Result<(Spectrum1D, LoadedSource)> {
        self.read_discovered_1d_with_source_by_source_relative_to(
            base,
            sources,
            LoadedSourceFilter::path_prefix(path),
        )
    }

    /// Loads discovered source candidates below one source path prefix as exactly one 1D spectrum with source metadata.
    ///
    /// This short alias mirrors [`Self::read_discovered_1d_with_source_by_source_path_prefix_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the matching discovered sources
    /// do not resolve to exactly one one-dimensional spectrum.
    pub fn read_discovered_1d_with_source_by_source_path_prefix<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        path: impl AsRef<Path>,
    ) -> Result<(Spectrum1D, LoadedSource)> {
        self.read_discovered_1d_with_source_by_source_path_prefix_relative_to(base, sources, path)
    }

    /// Loads discovered source candidates matching one source path as exactly one 2D spectrum.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the matching discovered sources
    /// do not resolve to exactly one two-dimensional spectrum.
    pub fn read_discovered_2d_by_source_path_relative_to<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        path: impl AsRef<Path>,
    ) -> Result<Spectrum2D> {
        self.read_discovered_2d_by_source_relative_to(base, sources, LoadedSourceFilter::path(path))
    }

    /// Loads discovered source candidates matching one source path as exactly one 2D spectrum.
    ///
    /// This short alias mirrors [`Self::read_discovered_2d_by_source_path_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the matching discovered sources
    /// do not resolve to exactly one two-dimensional spectrum.
    pub fn read_discovered_2d_by_source_path<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        path: impl AsRef<Path>,
    ) -> Result<Spectrum2D> {
        self.read_discovered_2d_by_source_path_relative_to(base, sources, path)
    }

    /// Loads discovered source candidates below one source path prefix as exactly one 2D spectrum.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the matching discovered sources
    /// do not resolve to exactly one two-dimensional spectrum.
    pub fn read_discovered_2d_by_source_path_prefix_relative_to<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        path: impl AsRef<Path>,
    ) -> Result<Spectrum2D> {
        self.read_discovered_2d_by_source_relative_to(
            base,
            sources,
            LoadedSourceFilter::path_prefix(path),
        )
    }

    /// Loads discovered source candidates below one source path prefix as exactly one 2D spectrum.
    ///
    /// This short alias mirrors [`Self::read_discovered_2d_by_source_path_prefix_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the matching discovered sources
    /// do not resolve to exactly one two-dimensional spectrum.
    pub fn read_discovered_2d_by_source_path_prefix<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        path: impl AsRef<Path>,
    ) -> Result<Spectrum2D> {
        self.read_discovered_2d_by_source_path_prefix_relative_to(base, sources, path)
    }

    /// Loads discovered source candidates matching one source path as exactly one 2D spectrum with source metadata.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the matching discovered sources
    /// do not resolve to exactly one two-dimensional spectrum.
    pub fn read_discovered_2d_with_source_by_source_path_relative_to<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        path: impl AsRef<Path>,
    ) -> Result<(Spectrum2D, LoadedSource)> {
        self.read_discovered_2d_with_source_by_source_relative_to(
            base,
            sources,
            LoadedSourceFilter::path(path),
        )
    }

    /// Loads discovered source candidates matching one source path as exactly one 2D spectrum with source metadata.
    ///
    /// This short alias mirrors [`Self::read_discovered_2d_with_source_by_source_path_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the matching discovered sources
    /// do not resolve to exactly one two-dimensional spectrum.
    pub fn read_discovered_2d_with_source_by_source_path<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        path: impl AsRef<Path>,
    ) -> Result<(Spectrum2D, LoadedSource)> {
        self.read_discovered_2d_with_source_by_source_path_relative_to(base, sources, path)
    }

    /// Loads discovered source candidates below one source path prefix as exactly one 2D spectrum with source metadata.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the matching discovered sources
    /// do not resolve to exactly one two-dimensional spectrum.
    pub fn read_discovered_2d_with_source_by_source_path_prefix_relative_to<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        path: impl AsRef<Path>,
    ) -> Result<(Spectrum2D, LoadedSource)> {
        self.read_discovered_2d_with_source_by_source_relative_to(
            base,
            sources,
            LoadedSourceFilter::path_prefix(path),
        )
    }

    /// Loads discovered source candidates below one source path prefix as exactly one 2D spectrum with source metadata.
    ///
    /// This short alias mirrors [`Self::read_discovered_2d_with_source_by_source_path_prefix_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the matching discovered sources
    /// do not resolve to exactly one two-dimensional spectrum.
    pub fn read_discovered_2d_with_source_by_source_path_prefix<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        path: impl AsRef<Path>,
    ) -> Result<(Spectrum2D, LoadedSource)> {
        self.read_discovered_2d_with_source_by_source_path_prefix_relative_to(base, sources, path)
    }
}
