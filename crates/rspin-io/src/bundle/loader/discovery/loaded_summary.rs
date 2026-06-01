//! Loaded bundle summary helpers for discovered source candidates.

use std::path::Path;

use rspin_core::Result;

use super::DiscoveredSpectrumSource;
use crate::bundle::{LoadedSourceFilter, SpectrumBundleLoader, SpectrumBundleSummary};

/// Loads selected discovered source candidates and returns bundle summary counts.
///
/// # Errors
///
/// Returns an error when loading the selected discovered sources fails.
pub fn load_discovered_spectra_summary_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
) -> Result<SpectrumBundleSummary> {
    SpectrumBundleLoader::new().read_discovered_summary_relative_to(base, sources)
}

/// Loads selected discovered source candidates and returns bundle summary counts.
///
/// This short alias mirrors [`load_discovered_spectra_summary_relative_to`].
///
/// # Errors
///
/// Returns an error when loading the selected discovered sources fails.
pub fn load_discovered_spectra_summary<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
) -> Result<SpectrumBundleSummary> {
    load_discovered_spectra_summary_relative_to(base, sources)
}

/// Loads discovered source candidates matching one generic source filter and returns summary counts.
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_by_source_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    filter: impl Into<LoadedSourceFilter>,
) -> Result<SpectrumBundleSummary> {
    SpectrumBundleLoader::new().read_discovered_summary_by_source_relative_to(base, sources, filter)
}

/// Loads discovered source candidates matching one generic source filter and returns summary counts.
///
/// This short alias mirrors [`load_discovered_spectra_summary_by_source_relative_to`].
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_by_source<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    filter: impl Into<LoadedSourceFilter>,
) -> Result<SpectrumBundleSummary> {
    load_discovered_spectra_summary_by_source_relative_to(base, sources, filter)
}

/// Loads discovered source candidates matching any generic source filter and returns summary counts.
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_by_sources_relative_to<'a, I, F>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    filters: I,
) -> Result<SpectrumBundleSummary>
where
    I: IntoIterator<Item = F>,
    F: Into<LoadedSourceFilter>,
{
    SpectrumBundleLoader::new()
        .read_discovered_summary_by_sources_relative_to(base, sources, filters)
}

/// Loads discovered source candidates matching any generic source filter and returns summary counts.
///
/// This short alias mirrors [`load_discovered_spectra_summary_by_sources_relative_to`].
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_by_sources<'a, I, F>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    filters: I,
) -> Result<SpectrumBundleSummary>
where
    I: IntoIterator<Item = F>,
    F: Into<LoadedSourceFilter>,
{
    load_discovered_spectra_summary_by_sources_relative_to(base, sources, filters)
}

/// Strictly loads selected discovered source candidates and returns bundle summary counts.
///
/// # Errors
///
/// Returns an error when strict loading the selected discovered sources fails.
pub fn load_discovered_spectra_summary_strict_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
) -> Result<SpectrumBundleSummary> {
    SpectrumBundleLoader::new()
        .strict()
        .read_discovered_summary_relative_to(base, sources)
}

/// Strictly loads selected discovered source candidates and returns bundle summary counts.
///
/// This short alias mirrors [`load_discovered_spectra_summary_strict_relative_to`].
///
/// # Errors
///
/// Returns an error when strict loading the selected discovered sources fails.
pub fn load_discovered_spectra_summary_strict<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
) -> Result<SpectrumBundleSummary> {
    load_discovered_spectra_summary_strict_relative_to(base, sources)
}

/// Strictly loads discovered source candidates matching one generic source filter and returns summary counts.
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_strict_by_source_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    filter: impl Into<LoadedSourceFilter>,
) -> Result<SpectrumBundleSummary> {
    SpectrumBundleLoader::new()
        .strict()
        .read_discovered_summary_by_source_relative_to(base, sources, filter)
}

/// Strictly loads discovered source candidates matching one generic source filter and returns summary counts.
///
/// This short alias mirrors [`load_discovered_spectra_summary_strict_by_source_relative_to`].
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_strict_by_source<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    filter: impl Into<LoadedSourceFilter>,
) -> Result<SpectrumBundleSummary> {
    load_discovered_spectra_summary_strict_by_source_relative_to(base, sources, filter)
}

/// Strictly loads discovered source candidates matching any generic source filter and returns summary counts.
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_strict_by_sources_relative_to<'a, I, F>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    filters: I,
) -> Result<SpectrumBundleSummary>
where
    I: IntoIterator<Item = F>,
    F: Into<LoadedSourceFilter>,
{
    SpectrumBundleLoader::new()
        .strict()
        .read_discovered_summary_by_sources_relative_to(base, sources, filters)
}

/// Strictly loads discovered source candidates matching any generic source filter and returns summary counts.
///
/// This short alias mirrors [`load_discovered_spectra_summary_strict_by_sources_relative_to`].
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_strict_by_sources<'a, I, F>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    filters: I,
) -> Result<SpectrumBundleSummary>
where
    I: IntoIterator<Item = F>,
    F: Into<LoadedSourceFilter>,
{
    load_discovered_spectra_summary_strict_by_sources_relative_to(base, sources, filters)
}

impl SpectrumBundleLoader {
    /// Loads selected discovered source candidates and returns bundle summary counts.
    ///
    /// This follows the same loading, filtering, strict-mode, and warning
    /// behavior as [`Self::read_discovered_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when [`Self::read_discovered_relative_to`] would return an error.
    pub fn read_discovered_summary_relative_to<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    ) -> Result<SpectrumBundleSummary> {
        self.read_discovered_relative_to(base, sources)
            .map(|bundle| bundle.summary())
    }

    /// Loads selected discovered source candidates and returns bundle summary counts.
    ///
    /// This short alias mirrors [`Self::read_discovered_summary_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading the selected discovered sources fails.
    pub fn read_discovered_summary<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    ) -> Result<SpectrumBundleSummary> {
        self.read_discovered_summary_relative_to(base, sources)
    }

    /// Loads discovered source candidates matching one generic source filter and returns summary counts.
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_summary_by_source_relative_to<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        filter: impl Into<LoadedSourceFilter>,
    ) -> Result<SpectrumBundleSummary> {
        let filter = filter.into();
        self.read_discovered_summary_by_sources_relative_to(base, sources, [filter])
    }

    /// Loads discovered source candidates matching one generic source filter and returns summary counts.
    ///
    /// This short alias mirrors [`Self::read_discovered_summary_by_source_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_summary_by_source<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        filter: impl Into<LoadedSourceFilter>,
    ) -> Result<SpectrumBundleSummary> {
        self.read_discovered_summary_by_source_relative_to(base, sources, filter)
    }

    /// Loads discovered source candidates matching any generic source filter and returns summary counts.
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_summary_by_sources_relative_to<'a, I, F>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        filters: I,
    ) -> Result<SpectrumBundleSummary>
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>,
    {
        self.read_discovered_by_sources_relative_to(base, sources, filters)
            .map(|bundle| bundle.summary())
    }

    /// Loads discovered source candidates matching any generic source filter and returns summary counts.
    ///
    /// This short alias mirrors [`Self::read_discovered_summary_by_sources_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_summary_by_sources<'a, I, F>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        filters: I,
    ) -> Result<SpectrumBundleSummary>
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>,
    {
        self.read_discovered_summary_by_sources_relative_to(base, sources, filters)
    }
}
