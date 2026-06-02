//! Dimension-specific loaded bundle summaries from discovered source candidates.

use std::path::Path;

use rspin_core::Result;

use super::DiscoveredSpectrumSource;
use crate::bundle::{LoadedSourceFilter, SpectrumBundleLoader, SpectrumBundleSummary};

/// Loads selected discovered source candidates as one-dimensional spectra and returns summary counts.
///
/// # Errors
///
/// Returns an error when loading the selected one-dimensional discovered sources fails.
pub fn load_discovered_spectra_1d_summary_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
) -> Result<SpectrumBundleSummary> {
    SpectrumBundleLoader::new().read_discovered_bundle_1d_summary_relative_to(base, sources)
}

/// Loads selected discovered source candidates as one-dimensional spectra and returns summary counts.
///
/// This short alias mirrors [`load_discovered_spectra_1d_summary_relative_to`].
///
/// # Errors
///
/// Returns an error when loading the selected one-dimensional discovered sources fails.
pub fn load_discovered_spectra_1d_summary<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
) -> Result<SpectrumBundleSummary> {
    load_discovered_spectra_1d_summary_relative_to(base, sources)
}

/// Loads one-dimensional discovered source candidates matching one generic source filter and returns summary counts.
///
/// # Errors
///
/// Returns an error when loading the matching one-dimensional discovered sources fails.
pub fn load_discovered_spectra_1d_summary_by_source_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    filter: impl Into<LoadedSourceFilter>,
) -> Result<SpectrumBundleSummary> {
    SpectrumBundleLoader::new()
        .read_discovered_bundle_1d_summary_by_source_relative_to(base, sources, filter)
}

/// Loads one-dimensional discovered source candidates matching one generic source filter and returns summary counts.
///
/// This short alias mirrors [`load_discovered_spectra_1d_summary_by_source_relative_to`].
///
/// # Errors
///
/// Returns an error when loading the matching one-dimensional discovered sources fails.
pub fn load_discovered_spectra_1d_summary_by_source<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    filter: impl Into<LoadedSourceFilter>,
) -> Result<SpectrumBundleSummary> {
    load_discovered_spectra_1d_summary_by_source_relative_to(base, sources, filter)
}

/// Loads one-dimensional discovered source candidates matching any generic source filter and returns summary counts.
///
/// # Errors
///
/// Returns an error when loading the matching one-dimensional discovered sources fails.
pub fn load_discovered_spectra_1d_summary_by_sources_relative_to<'a, I, F>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    filters: I,
) -> Result<SpectrumBundleSummary>
where
    I: IntoIterator<Item = F>,
    F: Into<LoadedSourceFilter>,
{
    SpectrumBundleLoader::new()
        .read_discovered_bundle_1d_summary_by_sources_relative_to(base, sources, filters)
}

/// Loads one-dimensional discovered source candidates matching any generic source filter and returns summary counts.
///
/// This short alias mirrors [`load_discovered_spectra_1d_summary_by_sources_relative_to`].
///
/// # Errors
///
/// Returns an error when loading the matching one-dimensional discovered sources fails.
pub fn load_discovered_spectra_1d_summary_by_sources<'a, I, F>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    filters: I,
) -> Result<SpectrumBundleSummary>
where
    I: IntoIterator<Item = F>,
    F: Into<LoadedSourceFilter>,
{
    load_discovered_spectra_1d_summary_by_sources_relative_to(base, sources, filters)
}

/// Strictly loads selected discovered source candidates as one-dimensional spectra and returns summary counts.
///
/// # Errors
///
/// Returns an error when strict loading the selected one-dimensional discovered sources fails.
pub fn load_discovered_spectra_1d_summary_strict_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
) -> Result<SpectrumBundleSummary> {
    SpectrumBundleLoader::new()
        .strict()
        .read_discovered_bundle_1d_summary_relative_to(base, sources)
}

/// Strictly loads selected discovered source candidates as one-dimensional spectra and returns summary counts.
///
/// This short alias mirrors [`load_discovered_spectra_1d_summary_strict_relative_to`].
///
/// # Errors
///
/// Returns an error when strict loading the selected one-dimensional discovered sources fails.
pub fn load_discovered_spectra_1d_summary_strict<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
) -> Result<SpectrumBundleSummary> {
    load_discovered_spectra_1d_summary_strict_relative_to(base, sources)
}

/// Strictly loads one-dimensional discovered source candidates matching one generic source filter and returns summary counts.
///
/// # Errors
///
/// Returns an error when strict loading the matching one-dimensional discovered sources fails.
pub fn load_discovered_spectra_1d_summary_strict_by_source_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    filter: impl Into<LoadedSourceFilter>,
) -> Result<SpectrumBundleSummary> {
    SpectrumBundleLoader::new()
        .strict()
        .read_discovered_bundle_1d_summary_by_source_relative_to(base, sources, filter)
}

/// Strictly loads one-dimensional discovered source candidates matching one generic source filter and returns summary counts.
///
/// This short alias mirrors [`load_discovered_spectra_1d_summary_strict_by_source_relative_to`].
///
/// # Errors
///
/// Returns an error when strict loading the matching one-dimensional discovered sources fails.
pub fn load_discovered_spectra_1d_summary_strict_by_source<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    filter: impl Into<LoadedSourceFilter>,
) -> Result<SpectrumBundleSummary> {
    load_discovered_spectra_1d_summary_strict_by_source_relative_to(base, sources, filter)
}

/// Strictly loads one-dimensional discovered source candidates matching any generic source filter and returns summary counts.
///
/// # Errors
///
/// Returns an error when strict loading the matching one-dimensional discovered sources fails.
pub fn load_discovered_spectra_1d_summary_strict_by_sources_relative_to<'a, I, F>(
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
        .read_discovered_bundle_1d_summary_by_sources_relative_to(base, sources, filters)
}

/// Strictly loads one-dimensional discovered source candidates matching any generic source filter and returns summary counts.
///
/// This short alias mirrors [`load_discovered_spectra_1d_summary_strict_by_sources_relative_to`].
///
/// # Errors
///
/// Returns an error when strict loading the matching one-dimensional discovered sources fails.
pub fn load_discovered_spectra_1d_summary_strict_by_sources<'a, I, F>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    filters: I,
) -> Result<SpectrumBundleSummary>
where
    I: IntoIterator<Item = F>,
    F: Into<LoadedSourceFilter>,
{
    load_discovered_spectra_1d_summary_strict_by_sources_relative_to(base, sources, filters)
}

/// Loads selected discovered source candidates as two-dimensional spectra and returns summary counts.
///
/// # Errors
///
/// Returns an error when loading the selected two-dimensional discovered sources fails.
pub fn load_discovered_spectra_2d_summary_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
) -> Result<SpectrumBundleSummary> {
    SpectrumBundleLoader::new().read_discovered_bundle_2d_summary_relative_to(base, sources)
}

/// Loads selected discovered source candidates as two-dimensional spectra and returns summary counts.
///
/// This short alias mirrors [`load_discovered_spectra_2d_summary_relative_to`].
///
/// # Errors
///
/// Returns an error when loading the selected two-dimensional discovered sources fails.
pub fn load_discovered_spectra_2d_summary<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
) -> Result<SpectrumBundleSummary> {
    load_discovered_spectra_2d_summary_relative_to(base, sources)
}

/// Loads two-dimensional discovered source candidates matching one generic source filter and returns summary counts.
///
/// # Errors
///
/// Returns an error when loading the matching two-dimensional discovered sources fails.
pub fn load_discovered_spectra_2d_summary_by_source_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    filter: impl Into<LoadedSourceFilter>,
) -> Result<SpectrumBundleSummary> {
    SpectrumBundleLoader::new()
        .read_discovered_bundle_2d_summary_by_source_relative_to(base, sources, filter)
}

/// Loads two-dimensional discovered source candidates matching one generic source filter and returns summary counts.
///
/// This short alias mirrors [`load_discovered_spectra_2d_summary_by_source_relative_to`].
///
/// # Errors
///
/// Returns an error when loading the matching two-dimensional discovered sources fails.
pub fn load_discovered_spectra_2d_summary_by_source<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    filter: impl Into<LoadedSourceFilter>,
) -> Result<SpectrumBundleSummary> {
    load_discovered_spectra_2d_summary_by_source_relative_to(base, sources, filter)
}

/// Loads two-dimensional discovered source candidates matching any generic source filter and returns summary counts.
///
/// # Errors
///
/// Returns an error when loading the matching two-dimensional discovered sources fails.
pub fn load_discovered_spectra_2d_summary_by_sources_relative_to<'a, I, F>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    filters: I,
) -> Result<SpectrumBundleSummary>
where
    I: IntoIterator<Item = F>,
    F: Into<LoadedSourceFilter>,
{
    SpectrumBundleLoader::new()
        .read_discovered_bundle_2d_summary_by_sources_relative_to(base, sources, filters)
}

/// Loads two-dimensional discovered source candidates matching any generic source filter and returns summary counts.
///
/// This short alias mirrors [`load_discovered_spectra_2d_summary_by_sources_relative_to`].
///
/// # Errors
///
/// Returns an error when loading the matching two-dimensional discovered sources fails.
pub fn load_discovered_spectra_2d_summary_by_sources<'a, I, F>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    filters: I,
) -> Result<SpectrumBundleSummary>
where
    I: IntoIterator<Item = F>,
    F: Into<LoadedSourceFilter>,
{
    load_discovered_spectra_2d_summary_by_sources_relative_to(base, sources, filters)
}

/// Strictly loads selected discovered source candidates as two-dimensional spectra and returns summary counts.
///
/// # Errors
///
/// Returns an error when strict loading the selected two-dimensional discovered sources fails.
pub fn load_discovered_spectra_2d_summary_strict_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
) -> Result<SpectrumBundleSummary> {
    SpectrumBundleLoader::new()
        .strict()
        .read_discovered_bundle_2d_summary_relative_to(base, sources)
}

/// Strictly loads selected discovered source candidates as two-dimensional spectra and returns summary counts.
///
/// This short alias mirrors [`load_discovered_spectra_2d_summary_strict_relative_to`].
///
/// # Errors
///
/// Returns an error when strict loading the selected two-dimensional discovered sources fails.
pub fn load_discovered_spectra_2d_summary_strict<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
) -> Result<SpectrumBundleSummary> {
    load_discovered_spectra_2d_summary_strict_relative_to(base, sources)
}

/// Strictly loads two-dimensional discovered source candidates matching one generic source filter and returns summary counts.
///
/// # Errors
///
/// Returns an error when strict loading the matching two-dimensional discovered sources fails.
pub fn load_discovered_spectra_2d_summary_strict_by_source_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    filter: impl Into<LoadedSourceFilter>,
) -> Result<SpectrumBundleSummary> {
    SpectrumBundleLoader::new()
        .strict()
        .read_discovered_bundle_2d_summary_by_source_relative_to(base, sources, filter)
}

/// Strictly loads two-dimensional discovered source candidates matching one generic source filter and returns summary counts.
///
/// This short alias mirrors [`load_discovered_spectra_2d_summary_strict_by_source_relative_to`].
///
/// # Errors
///
/// Returns an error when strict loading the matching two-dimensional discovered sources fails.
pub fn load_discovered_spectra_2d_summary_strict_by_source<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    filter: impl Into<LoadedSourceFilter>,
) -> Result<SpectrumBundleSummary> {
    load_discovered_spectra_2d_summary_strict_by_source_relative_to(base, sources, filter)
}

/// Strictly loads two-dimensional discovered source candidates matching any generic source filter and returns summary counts.
///
/// # Errors
///
/// Returns an error when strict loading the matching two-dimensional discovered sources fails.
pub fn load_discovered_spectra_2d_summary_strict_by_sources_relative_to<'a, I, F>(
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
        .read_discovered_bundle_2d_summary_by_sources_relative_to(base, sources, filters)
}

/// Strictly loads two-dimensional discovered source candidates matching any generic source filter and returns summary counts.
///
/// This short alias mirrors [`load_discovered_spectra_2d_summary_strict_by_sources_relative_to`].
///
/// # Errors
///
/// Returns an error when strict loading the matching two-dimensional discovered sources fails.
pub fn load_discovered_spectra_2d_summary_strict_by_sources<'a, I, F>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    filters: I,
) -> Result<SpectrumBundleSummary>
where
    I: IntoIterator<Item = F>,
    F: Into<LoadedSourceFilter>,
{
    load_discovered_spectra_2d_summary_strict_by_sources_relative_to(base, sources, filters)
}

impl SpectrumBundleLoader {
    /// Loads selected discovered source candidates as one-dimensional spectra and returns summary counts.
    ///
    /// # Errors
    ///
    /// Returns an error when loading the selected one-dimensional discovered sources fails.
    pub fn read_discovered_bundle_1d_summary_relative_to<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    ) -> Result<SpectrumBundleSummary> {
        self.read_discovered_bundle_1d_relative_to(base, sources)
            .map(|bundle| bundle.summary())
    }

    /// Loads selected discovered source candidates as one-dimensional spectra and returns summary counts.
    ///
    /// This short alias mirrors [`Self::read_discovered_bundle_1d_summary_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading the selected one-dimensional discovered sources fails.
    pub fn read_discovered_bundle_1d_summary<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    ) -> Result<SpectrumBundleSummary> {
        self.read_discovered_bundle_1d_summary_relative_to(base, sources)
    }

    /// Loads one-dimensional discovered source candidates matching one generic source filter and returns summary counts.
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching one-dimensional discovered sources fails.
    pub fn read_discovered_bundle_1d_summary_by_source_relative_to<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        filter: impl Into<LoadedSourceFilter>,
    ) -> Result<SpectrumBundleSummary> {
        let filter = filter.into();
        self.read_discovered_bundle_1d_summary_by_sources_relative_to(base, sources, [filter])
    }

    /// Loads one-dimensional discovered source candidates matching one generic source filter and returns summary counts.
    ///
    /// This short alias mirrors [`Self::read_discovered_bundle_1d_summary_by_source_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching one-dimensional discovered sources fails.
    pub fn read_discovered_bundle_1d_summary_by_source<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        filter: impl Into<LoadedSourceFilter>,
    ) -> Result<SpectrumBundleSummary> {
        self.read_discovered_bundle_1d_summary_by_source_relative_to(base, sources, filter)
    }

    /// Loads one-dimensional discovered source candidates matching any generic source filter and returns summary counts.
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching one-dimensional discovered sources fails.
    pub fn read_discovered_bundle_1d_summary_by_sources_relative_to<'a, I, F>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        filters: I,
    ) -> Result<SpectrumBundleSummary>
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>,
    {
        self.read_discovered_bundle_1d_by_sources_relative_to(base, sources, filters)
            .map(|bundle| bundle.summary())
    }

    /// Loads one-dimensional discovered source candidates matching any generic source filter and returns summary counts.
    ///
    /// This short alias mirrors [`Self::read_discovered_bundle_1d_summary_by_sources_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching one-dimensional discovered sources fails.
    pub fn read_discovered_bundle_1d_summary_by_sources<'a, I, F>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        filters: I,
    ) -> Result<SpectrumBundleSummary>
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>,
    {
        self.read_discovered_bundle_1d_summary_by_sources_relative_to(base, sources, filters)
    }

    /// Loads selected discovered source candidates as two-dimensional spectra and returns summary counts.
    ///
    /// # Errors
    ///
    /// Returns an error when loading the selected two-dimensional discovered sources fails.
    pub fn read_discovered_bundle_2d_summary_relative_to<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    ) -> Result<SpectrumBundleSummary> {
        self.read_discovered_bundle_2d_relative_to(base, sources)
            .map(|bundle| bundle.summary())
    }

    /// Loads selected discovered source candidates as two-dimensional spectra and returns summary counts.
    ///
    /// This short alias mirrors [`Self::read_discovered_bundle_2d_summary_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading the selected two-dimensional discovered sources fails.
    pub fn read_discovered_bundle_2d_summary<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    ) -> Result<SpectrumBundleSummary> {
        self.read_discovered_bundle_2d_summary_relative_to(base, sources)
    }

    /// Loads two-dimensional discovered source candidates matching one generic source filter and returns summary counts.
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching two-dimensional discovered sources fails.
    pub fn read_discovered_bundle_2d_summary_by_source_relative_to<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        filter: impl Into<LoadedSourceFilter>,
    ) -> Result<SpectrumBundleSummary> {
        let filter = filter.into();
        self.read_discovered_bundle_2d_summary_by_sources_relative_to(base, sources, [filter])
    }

    /// Loads two-dimensional discovered source candidates matching one generic source filter and returns summary counts.
    ///
    /// This short alias mirrors [`Self::read_discovered_bundle_2d_summary_by_source_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching two-dimensional discovered sources fails.
    pub fn read_discovered_bundle_2d_summary_by_source<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        filter: impl Into<LoadedSourceFilter>,
    ) -> Result<SpectrumBundleSummary> {
        self.read_discovered_bundle_2d_summary_by_source_relative_to(base, sources, filter)
    }

    /// Loads two-dimensional discovered source candidates matching any generic source filter and returns summary counts.
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching two-dimensional discovered sources fails.
    pub fn read_discovered_bundle_2d_summary_by_sources_relative_to<'a, I, F>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        filters: I,
    ) -> Result<SpectrumBundleSummary>
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>,
    {
        self.read_discovered_bundle_2d_by_sources_relative_to(base, sources, filters)
            .map(|bundle| bundle.summary())
    }

    /// Loads two-dimensional discovered source candidates matching any generic source filter and returns summary counts.
    ///
    /// This short alias mirrors [`Self::read_discovered_bundle_2d_summary_by_sources_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching two-dimensional discovered sources fails.
    pub fn read_discovered_bundle_2d_summary_by_sources<'a, I, F>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        filters: I,
    ) -> Result<SpectrumBundleSummary>
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>,
    {
        self.read_discovered_bundle_2d_summary_by_sources_relative_to(base, sources, filters)
    }
}
