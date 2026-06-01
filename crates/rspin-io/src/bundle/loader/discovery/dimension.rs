//! Dimension-specific bundle loading from discovered source candidates.

use std::path::Path;

use rspin_core::{RSpinError, Result};

use super::{DiscoveredSpectrumDimension, DiscoveredSpectrumSource, selection};
use crate::bundle::{LoadedSourceFilter, SpectrumBundle, SpectrumBundleLoader};

/// Loads selected discovered source candidates as a one-dimensional spectrum bundle.
///
/// # Errors
///
/// Returns an error when loading fails, no discovered sources are provided, no
/// one-dimensional candidate is selected, or no one-dimensional bundle data is found.
pub fn load_discovered_spectra_1d_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
) -> Result<SpectrumBundle> {
    SpectrumBundleLoader::new().read_discovered_bundle_1d_relative_to(base, sources)
}

/// Loads selected discovered source candidates as a one-dimensional spectrum bundle.
///
/// This short alias mirrors [`load_discovered_spectra_1d_relative_to`].
///
/// # Errors
///
/// Returns an error when loading the selected discovered sources fails.
pub fn load_discovered_spectra_1d<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
) -> Result<SpectrumBundle> {
    load_discovered_spectra_1d_relative_to(base, sources)
}

/// Loads discovered source candidates matching one generic source filter as a one-dimensional bundle.
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_1d_by_source_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    filter: impl Into<LoadedSourceFilter>,
) -> Result<SpectrumBundle> {
    SpectrumBundleLoader::new()
        .read_discovered_bundle_1d_by_source_relative_to(base, sources, filter)
}

/// Loads discovered source candidates matching one generic source filter as a one-dimensional bundle.
///
/// This short alias mirrors [`load_discovered_spectra_1d_by_source_relative_to`].
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_1d_by_source<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    filter: impl Into<LoadedSourceFilter>,
) -> Result<SpectrumBundle> {
    load_discovered_spectra_1d_by_source_relative_to(base, sources, filter)
}

/// Loads discovered source candidates matching any generic source filter as a one-dimensional bundle.
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_1d_by_sources_relative_to<'a, I, F>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    filters: I,
) -> Result<SpectrumBundle>
where
    I: IntoIterator<Item = F>,
    F: Into<LoadedSourceFilter>,
{
    SpectrumBundleLoader::new()
        .read_discovered_bundle_1d_by_sources_relative_to(base, sources, filters)
}

/// Loads discovered source candidates matching any generic source filter as a one-dimensional bundle.
///
/// This short alias mirrors [`load_discovered_spectra_1d_by_sources_relative_to`].
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_1d_by_sources<'a, I, F>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    filters: I,
) -> Result<SpectrumBundle>
where
    I: IntoIterator<Item = F>,
    F: Into<LoadedSourceFilter>,
{
    load_discovered_spectra_1d_by_sources_relative_to(base, sources, filters)
}

/// Strictly loads selected discovered source candidates as a one-dimensional spectrum bundle.
///
/// # Errors
///
/// Returns an error when strict loading the selected discovered sources fails.
pub fn load_discovered_spectra_1d_strict_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
) -> Result<SpectrumBundle> {
    SpectrumBundleLoader::new()
        .strict()
        .read_discovered_bundle_1d_relative_to(base, sources)
}

/// Strictly loads selected discovered source candidates as a one-dimensional spectrum bundle.
///
/// This short alias mirrors [`load_discovered_spectra_1d_strict_relative_to`].
///
/// # Errors
///
/// Returns an error when strict loading the selected discovered sources fails.
pub fn load_discovered_spectra_1d_strict<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
) -> Result<SpectrumBundle> {
    load_discovered_spectra_1d_strict_relative_to(base, sources)
}

/// Strictly loads discovered source candidates matching one generic source filter as a one-dimensional bundle.
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_1d_strict_by_source_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    filter: impl Into<LoadedSourceFilter>,
) -> Result<SpectrumBundle> {
    SpectrumBundleLoader::new()
        .strict()
        .read_discovered_bundle_1d_by_source_relative_to(base, sources, filter)
}

/// Strictly loads discovered source candidates matching one generic source filter as a one-dimensional bundle.
///
/// This short alias mirrors [`load_discovered_spectra_1d_strict_by_source_relative_to`].
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_1d_strict_by_source<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    filter: impl Into<LoadedSourceFilter>,
) -> Result<SpectrumBundle> {
    load_discovered_spectra_1d_strict_by_source_relative_to(base, sources, filter)
}

/// Strictly loads discovered source candidates matching any generic source filter as a one-dimensional bundle.
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_1d_strict_by_sources_relative_to<'a, I, F>(
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
        .read_discovered_bundle_1d_by_sources_relative_to(base, sources, filters)
}

/// Strictly loads discovered source candidates matching any generic source filter as a one-dimensional bundle.
///
/// This short alias mirrors [`load_discovered_spectra_1d_strict_by_sources_relative_to`].
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_1d_strict_by_sources<'a, I, F>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    filters: I,
) -> Result<SpectrumBundle>
where
    I: IntoIterator<Item = F>,
    F: Into<LoadedSourceFilter>,
{
    load_discovered_spectra_1d_strict_by_sources_relative_to(base, sources, filters)
}

/// Loads selected discovered source candidates as a two-dimensional spectrum bundle.
///
/// # Errors
///
/// Returns an error when loading fails, no discovered sources are provided, no
/// two-dimensional candidate is selected, or no two-dimensional bundle data is found.
pub fn load_discovered_spectra_2d_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
) -> Result<SpectrumBundle> {
    SpectrumBundleLoader::new().read_discovered_bundle_2d_relative_to(base, sources)
}

/// Loads selected discovered source candidates as a two-dimensional spectrum bundle.
///
/// This short alias mirrors [`load_discovered_spectra_2d_relative_to`].
///
/// # Errors
///
/// Returns an error when loading the selected discovered sources fails.
pub fn load_discovered_spectra_2d<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
) -> Result<SpectrumBundle> {
    load_discovered_spectra_2d_relative_to(base, sources)
}

/// Loads discovered source candidates matching one generic source filter as a two-dimensional bundle.
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_2d_by_source_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    filter: impl Into<LoadedSourceFilter>,
) -> Result<SpectrumBundle> {
    SpectrumBundleLoader::new()
        .read_discovered_bundle_2d_by_source_relative_to(base, sources, filter)
}

/// Loads discovered source candidates matching one generic source filter as a two-dimensional bundle.
///
/// This short alias mirrors [`load_discovered_spectra_2d_by_source_relative_to`].
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_2d_by_source<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    filter: impl Into<LoadedSourceFilter>,
) -> Result<SpectrumBundle> {
    load_discovered_spectra_2d_by_source_relative_to(base, sources, filter)
}

/// Loads discovered source candidates matching any generic source filter as a two-dimensional bundle.
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_2d_by_sources_relative_to<'a, I, F>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    filters: I,
) -> Result<SpectrumBundle>
where
    I: IntoIterator<Item = F>,
    F: Into<LoadedSourceFilter>,
{
    SpectrumBundleLoader::new()
        .read_discovered_bundle_2d_by_sources_relative_to(base, sources, filters)
}

/// Loads discovered source candidates matching any generic source filter as a two-dimensional bundle.
///
/// This short alias mirrors [`load_discovered_spectra_2d_by_sources_relative_to`].
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_2d_by_sources<'a, I, F>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    filters: I,
) -> Result<SpectrumBundle>
where
    I: IntoIterator<Item = F>,
    F: Into<LoadedSourceFilter>,
{
    load_discovered_spectra_2d_by_sources_relative_to(base, sources, filters)
}

/// Strictly loads selected discovered source candidates as a two-dimensional spectrum bundle.
///
/// # Errors
///
/// Returns an error when strict loading the selected discovered sources fails.
pub fn load_discovered_spectra_2d_strict_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
) -> Result<SpectrumBundle> {
    SpectrumBundleLoader::new()
        .strict()
        .read_discovered_bundle_2d_relative_to(base, sources)
}

/// Strictly loads selected discovered source candidates as a two-dimensional spectrum bundle.
///
/// This short alias mirrors [`load_discovered_spectra_2d_strict_relative_to`].
///
/// # Errors
///
/// Returns an error when strict loading the selected discovered sources fails.
pub fn load_discovered_spectra_2d_strict<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
) -> Result<SpectrumBundle> {
    load_discovered_spectra_2d_strict_relative_to(base, sources)
}

/// Strictly loads discovered source candidates matching one generic source filter as a two-dimensional bundle.
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_2d_strict_by_source_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    filter: impl Into<LoadedSourceFilter>,
) -> Result<SpectrumBundle> {
    SpectrumBundleLoader::new()
        .strict()
        .read_discovered_bundle_2d_by_source_relative_to(base, sources, filter)
}

/// Strictly loads discovered source candidates matching one generic source filter as a two-dimensional bundle.
///
/// This short alias mirrors [`load_discovered_spectra_2d_strict_by_source_relative_to`].
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_2d_strict_by_source<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    filter: impl Into<LoadedSourceFilter>,
) -> Result<SpectrumBundle> {
    load_discovered_spectra_2d_strict_by_source_relative_to(base, sources, filter)
}

/// Strictly loads discovered source candidates matching any generic source filter as a two-dimensional bundle.
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_2d_strict_by_sources_relative_to<'a, I, F>(
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
        .read_discovered_bundle_2d_by_sources_relative_to(base, sources, filters)
}

/// Strictly loads discovered source candidates matching any generic source filter as a two-dimensional bundle.
///
/// This short alias mirrors [`load_discovered_spectra_2d_strict_by_sources_relative_to`].
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_2d_strict_by_sources<'a, I, F>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    filters: I,
) -> Result<SpectrumBundle>
where
    I: IntoIterator<Item = F>,
    F: Into<LoadedSourceFilter>,
{
    load_discovered_spectra_2d_strict_by_sources_relative_to(base, sources, filters)
}

impl SpectrumBundleLoader {
    /// Loads selected discovered source candidates as a one-dimensional spectrum bundle.
    ///
    /// Known two-dimensional candidates are skipped before loading. Unknown
    /// dimension candidates are still attempted with one-dimensional loading enabled.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails, no discovered sources are provided,
    /// no one-dimensional candidate is selected, or no one-dimensional data is found.
    pub fn read_discovered_bundle_1d_relative_to<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    ) -> Result<SpectrumBundle> {
        let sources = discovered_dimension_sources(sources, DiscoveredSpectrumDimension::OneD)?;
        self.clone()
            .one_d_only()
            .read_discovered_relative_to(base, sources)
    }

    /// Loads selected discovered source candidates as a one-dimensional spectrum bundle.
    ///
    /// This short alias mirrors [`Self::read_discovered_bundle_1d_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading the selected discovered sources fails.
    pub fn read_discovered_bundle_1d<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    ) -> Result<SpectrumBundle> {
        self.read_discovered_bundle_1d_relative_to(base, sources)
    }

    /// Loads discovered source candidates matching one generic source filter as a one-dimensional bundle.
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_bundle_1d_by_source_relative_to<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        filter: impl Into<LoadedSourceFilter>,
    ) -> Result<SpectrumBundle> {
        let filter = filter.into();
        self.read_discovered_bundle_1d_by_sources_relative_to(base, sources, [filter])
    }

    /// Loads discovered source candidates matching one generic source filter as a one-dimensional bundle.
    ///
    /// This short alias mirrors [`Self::read_discovered_bundle_1d_by_source_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_bundle_1d_by_source<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        filter: impl Into<LoadedSourceFilter>,
    ) -> Result<SpectrumBundle> {
        self.read_discovered_bundle_1d_by_source_relative_to(base, sources, filter)
    }

    /// Loads discovered source candidates matching any generic source filter as a one-dimensional bundle.
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_bundle_1d_by_sources_relative_to<'a, I, F>(
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
        self.read_discovered_bundle_1d_relative_to(base, selected)
    }

    /// Loads discovered source candidates matching any generic source filter as a one-dimensional bundle.
    ///
    /// This short alias mirrors [`Self::read_discovered_bundle_1d_by_sources_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_bundle_1d_by_sources<'a, I, F>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        filters: I,
    ) -> Result<SpectrumBundle>
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>,
    {
        self.read_discovered_bundle_1d_by_sources_relative_to(base, sources, filters)
    }

    /// Loads selected discovered source candidates as a two-dimensional spectrum bundle.
    ///
    /// Known one-dimensional candidates are skipped before loading. Unknown
    /// dimension candidates are still attempted with two-dimensional loading enabled.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails, no discovered sources are provided,
    /// no two-dimensional candidate is selected, or no two-dimensional data is found.
    pub fn read_discovered_bundle_2d_relative_to<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    ) -> Result<SpectrumBundle> {
        let sources = discovered_dimension_sources(sources, DiscoveredSpectrumDimension::TwoD)?;
        self.clone()
            .two_d_only()
            .read_discovered_relative_to(base, sources)
    }

    /// Loads selected discovered source candidates as a two-dimensional spectrum bundle.
    ///
    /// This short alias mirrors [`Self::read_discovered_bundle_2d_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading the selected discovered sources fails.
    pub fn read_discovered_bundle_2d<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    ) -> Result<SpectrumBundle> {
        self.read_discovered_bundle_2d_relative_to(base, sources)
    }

    /// Loads discovered source candidates matching one generic source filter as a two-dimensional bundle.
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_bundle_2d_by_source_relative_to<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        filter: impl Into<LoadedSourceFilter>,
    ) -> Result<SpectrumBundle> {
        let filter = filter.into();
        self.read_discovered_bundle_2d_by_sources_relative_to(base, sources, [filter])
    }

    /// Loads discovered source candidates matching one generic source filter as a two-dimensional bundle.
    ///
    /// This short alias mirrors [`Self::read_discovered_bundle_2d_by_source_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_bundle_2d_by_source<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        filter: impl Into<LoadedSourceFilter>,
    ) -> Result<SpectrumBundle> {
        self.read_discovered_bundle_2d_by_source_relative_to(base, sources, filter)
    }

    /// Loads discovered source candidates matching any generic source filter as a two-dimensional bundle.
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_bundle_2d_by_sources_relative_to<'a, I, F>(
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
        self.read_discovered_bundle_2d_relative_to(base, selected)
    }

    /// Loads discovered source candidates matching any generic source filter as a two-dimensional bundle.
    ///
    /// This short alias mirrors [`Self::read_discovered_bundle_2d_by_sources_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_bundle_2d_by_sources<'a, I, F>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        filters: I,
    ) -> Result<SpectrumBundle>
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>,
    {
        self.read_discovered_bundle_2d_by_sources_relative_to(base, sources, filters)
    }
}

fn discovered_dimension_sources<'a>(
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    dimension: DiscoveredSpectrumDimension,
) -> Result<Vec<&'a DiscoveredSpectrumSource>> {
    let mut saw_source = false;
    let mut selected = Vec::new();
    for source in sources {
        saw_source = true;
        if source.dimension().is_unknown() || source.dimension() == dimension {
            selected.push(source);
        }
    }

    if !saw_source {
        return Err(RSpinError::Parse {
            format: "spectrum bundle",
            message: "no discovered sources provided".to_owned(),
        });
    }

    if selected.is_empty() {
        return Err(RSpinError::Parse {
            format: "spectrum bundle",
            message: format!(
                "no {} discovered sources selected",
                discovered_dimension_label(dimension)
            ),
        });
    }

    Ok(selected)
}

fn discovered_dimension_label(dimension: DiscoveredSpectrumDimension) -> &'static str {
    match dimension {
        DiscoveredSpectrumDimension::OneD => "one-dimensional",
        DiscoveredSpectrumDimension::TwoD => "two-dimensional",
        DiscoveredSpectrumDimension::Unknown => "unknown-dimension",
    }
}
