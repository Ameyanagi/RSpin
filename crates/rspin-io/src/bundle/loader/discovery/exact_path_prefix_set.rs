//! Exact source-path-prefix-set loading from discovered source candidates.

use std::path::Path;

use rspin_core::{Result, Spectrum1D, Spectrum2D};

use super::DiscoveredSpectrumSource;
use crate::bundle::{LoadedSource, LoadedSourceFilter, SpectrumBundleLoader};

/// Loads discovered source candidates below any source path prefix as exactly one 1D spectrum.
///
/// Prefixes are combined with logical OR. Passing an empty iterator leaves
/// source matching unrestricted.
///
/// # Errors
///
/// Returns an error when loading fails or the matching discovered sources do
/// not resolve to exactly one one-dimensional spectrum.
pub fn load_discovered_spectrum_1d_by_source_path_prefixes_relative_to<'a, I, P>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    paths: I,
) -> Result<Spectrum1D>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new()
        .read_discovered_1d_by_source_path_prefixes_relative_to(base, sources, paths)
}

/// Loads discovered source candidates below any source path prefix as exactly one 1D spectrum.
///
/// This short alias mirrors [`load_discovered_spectrum_1d_by_source_path_prefixes_relative_to`].
///
/// # Errors
///
/// Returns an error when loading fails or the matching discovered sources do
/// not resolve to exactly one one-dimensional spectrum.
pub fn load_discovered_spectrum_1d_by_source_path_prefixes<'a, I, P>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    paths: I,
) -> Result<Spectrum1D>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    load_discovered_spectrum_1d_by_source_path_prefixes_relative_to(base, sources, paths)
}

/// Loads discovered source candidates below any source path prefix as exactly one 1D spectrum with source metadata.
///
/// Prefixes are combined with logical OR. Passing an empty iterator leaves
/// source matching unrestricted.
///
/// # Errors
///
/// Returns an error when loading fails or the matching discovered sources do
/// not resolve to exactly one one-dimensional spectrum.
pub fn load_discovered_spectrum_1d_with_source_by_source_path_prefixes_relative_to<'a, I, P>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    paths: I,
) -> Result<(Spectrum1D, LoadedSource)>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new()
        .read_discovered_1d_with_source_by_source_path_prefixes_relative_to(base, sources, paths)
}

/// Loads discovered source candidates below any source path prefix as exactly one 1D spectrum with source metadata.
///
/// This short alias mirrors [`load_discovered_spectrum_1d_with_source_by_source_path_prefixes_relative_to`].
///
/// # Errors
///
/// Returns an error when loading fails or the matching discovered sources do
/// not resolve to exactly one one-dimensional spectrum.
pub fn load_discovered_spectrum_1d_with_source_by_source_path_prefixes<'a, I, P>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    paths: I,
) -> Result<(Spectrum1D, LoadedSource)>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    load_discovered_spectrum_1d_with_source_by_source_path_prefixes_relative_to(
        base, sources, paths,
    )
}

/// Loads discovered source candidates below any source path prefix as exactly one 2D spectrum.
///
/// Prefixes are combined with logical OR. Passing an empty iterator leaves
/// source matching unrestricted.
///
/// # Errors
///
/// Returns an error when loading fails or the matching discovered sources do
/// not resolve to exactly one two-dimensional spectrum.
pub fn load_discovered_spectrum_2d_by_source_path_prefixes_relative_to<'a, I, P>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    paths: I,
) -> Result<Spectrum2D>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new()
        .read_discovered_2d_by_source_path_prefixes_relative_to(base, sources, paths)
}

/// Loads discovered source candidates below any source path prefix as exactly one 2D spectrum.
///
/// This short alias mirrors [`load_discovered_spectrum_2d_by_source_path_prefixes_relative_to`].
///
/// # Errors
///
/// Returns an error when loading fails or the matching discovered sources do
/// not resolve to exactly one two-dimensional spectrum.
pub fn load_discovered_spectrum_2d_by_source_path_prefixes<'a, I, P>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    paths: I,
) -> Result<Spectrum2D>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    load_discovered_spectrum_2d_by_source_path_prefixes_relative_to(base, sources, paths)
}

/// Loads discovered source candidates below any source path prefix as exactly one 2D spectrum with source metadata.
///
/// Prefixes are combined with logical OR. Passing an empty iterator leaves
/// source matching unrestricted.
///
/// # Errors
///
/// Returns an error when loading fails or the matching discovered sources do
/// not resolve to exactly one two-dimensional spectrum.
pub fn load_discovered_spectrum_2d_with_source_by_source_path_prefixes_relative_to<'a, I, P>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    paths: I,
) -> Result<(Spectrum2D, LoadedSource)>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new()
        .read_discovered_2d_with_source_by_source_path_prefixes_relative_to(base, sources, paths)
}

/// Loads discovered source candidates below any source path prefix as exactly one 2D spectrum with source metadata.
///
/// This short alias mirrors [`load_discovered_spectrum_2d_with_source_by_source_path_prefixes_relative_to`].
///
/// # Errors
///
/// Returns an error when loading fails or the matching discovered sources do
/// not resolve to exactly one two-dimensional spectrum.
pub fn load_discovered_spectrum_2d_with_source_by_source_path_prefixes<'a, I, P>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    paths: I,
) -> Result<(Spectrum2D, LoadedSource)>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    load_discovered_spectrum_2d_with_source_by_source_path_prefixes_relative_to(
        base, sources, paths,
    )
}

impl SpectrumBundleLoader {
    /// Loads discovered source candidates below any source path prefix as exactly one 1D spectrum.
    ///
    /// Prefixes are combined with logical OR. Passing an empty iterator leaves
    /// source matching unrestricted.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the matching discovered sources
    /// do not resolve to exactly one one-dimensional spectrum.
    pub fn read_discovered_1d_by_source_path_prefixes_relative_to<'a, I, P>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        paths: I,
    ) -> Result<Spectrum1D>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_discovered_1d_by_sources_relative_to(base, sources, path_prefix_filters(paths))
    }

    /// Loads discovered source candidates below any source path prefix as exactly one 1D spectrum.
    ///
    /// This short alias mirrors [`Self::read_discovered_1d_by_source_path_prefixes_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the matching discovered sources
    /// do not resolve to exactly one one-dimensional spectrum.
    pub fn read_discovered_1d_by_source_path_prefixes<'a, I, P>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        paths: I,
    ) -> Result<Spectrum1D>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_discovered_1d_by_source_path_prefixes_relative_to(base, sources, paths)
    }

    /// Loads discovered source candidates below any source path prefix as exactly one 1D spectrum with source metadata.
    ///
    /// Prefixes are combined with logical OR. Passing an empty iterator leaves
    /// source matching unrestricted.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the matching discovered sources
    /// do not resolve to exactly one one-dimensional spectrum.
    pub fn read_discovered_1d_with_source_by_source_path_prefixes_relative_to<'a, I, P>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        paths: I,
    ) -> Result<(Spectrum1D, LoadedSource)>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_discovered_1d_with_source_by_sources_relative_to(
            base,
            sources,
            path_prefix_filters(paths),
        )
    }

    /// Loads discovered source candidates below any source path prefix as exactly one 1D spectrum with source metadata.
    ///
    /// This short alias mirrors [`Self::read_discovered_1d_with_source_by_source_path_prefixes_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the matching discovered sources
    /// do not resolve to exactly one one-dimensional spectrum.
    pub fn read_discovered_1d_with_source_by_source_path_prefixes<'a, I, P>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        paths: I,
    ) -> Result<(Spectrum1D, LoadedSource)>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_discovered_1d_with_source_by_source_path_prefixes_relative_to(
            base, sources, paths,
        )
    }

    /// Loads discovered source candidates below any source path prefix as exactly one 2D spectrum.
    ///
    /// Prefixes are combined with logical OR. Passing an empty iterator leaves
    /// source matching unrestricted.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the matching discovered sources
    /// do not resolve to exactly one two-dimensional spectrum.
    pub fn read_discovered_2d_by_source_path_prefixes_relative_to<'a, I, P>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        paths: I,
    ) -> Result<Spectrum2D>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_discovered_2d_by_sources_relative_to(base, sources, path_prefix_filters(paths))
    }

    /// Loads discovered source candidates below any source path prefix as exactly one 2D spectrum.
    ///
    /// This short alias mirrors [`Self::read_discovered_2d_by_source_path_prefixes_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the matching discovered sources
    /// do not resolve to exactly one two-dimensional spectrum.
    pub fn read_discovered_2d_by_source_path_prefixes<'a, I, P>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        paths: I,
    ) -> Result<Spectrum2D>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_discovered_2d_by_source_path_prefixes_relative_to(base, sources, paths)
    }

    /// Loads discovered source candidates below any source path prefix as exactly one 2D spectrum with source metadata.
    ///
    /// Prefixes are combined with logical OR. Passing an empty iterator leaves
    /// source matching unrestricted.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the matching discovered sources
    /// do not resolve to exactly one two-dimensional spectrum.
    pub fn read_discovered_2d_with_source_by_source_path_prefixes_relative_to<'a, I, P>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        paths: I,
    ) -> Result<(Spectrum2D, LoadedSource)>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_discovered_2d_with_source_by_sources_relative_to(
            base,
            sources,
            path_prefix_filters(paths),
        )
    }

    /// Loads discovered source candidates below any source path prefix as exactly one 2D spectrum with source metadata.
    ///
    /// This short alias mirrors [`Self::read_discovered_2d_with_source_by_source_path_prefixes_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the matching discovered sources
    /// do not resolve to exactly one two-dimensional spectrum.
    pub fn read_discovered_2d_with_source_by_source_path_prefixes<'a, I, P>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        paths: I,
    ) -> Result<(Spectrum2D, LoadedSource)>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_discovered_2d_with_source_by_source_path_prefixes_relative_to(
            base, sources, paths,
        )
    }
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
