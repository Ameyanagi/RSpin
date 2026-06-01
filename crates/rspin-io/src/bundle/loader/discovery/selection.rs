//! Selection helpers for discovered spectrum source candidates.

use super::DiscoveredSpectrumSource;
use crate::bundle::LoadedSourceFilter;

/// Selects discovered source candidates matching one generic source filter.
///
/// This is a lightweight preflight helper for caller UI and configuration
/// workflows. The returned references can be passed directly to
/// `load_discovered_spectra_relative_to` or `RSpinReader::read_discovered`.
#[must_use]
pub fn select_discovered_spectra_by_source(
    sources: &[DiscoveredSpectrumSource],
    filter: impl Into<LoadedSourceFilter>,
) -> Vec<&DiscoveredSpectrumSource> {
    let filter = filter.into();
    select_discovered_spectra_by_sources(sources, [filter])
}

/// Selects discovered source candidates matching any generic source filter.
///
/// Filters are combined with logical OR. Passing an empty iterator returns all
/// provided discovered sources, matching the unrestricted loader behavior.
#[must_use]
pub fn select_discovered_spectra_by_sources<I, F>(
    sources: &[DiscoveredSpectrumSource],
    filters: I,
) -> Vec<&DiscoveredSpectrumSource>
where
    I: IntoIterator<Item = F>,
    F: Into<LoadedSourceFilter>,
{
    select_discovered_source_refs(sources.iter(), filters)
}

pub(super) fn select_discovered_source_refs<'a, S, I, F>(
    sources: S,
    filters: I,
) -> Vec<&'a DiscoveredSpectrumSource>
where
    S: IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    I: IntoIterator<Item = F>,
    F: Into<LoadedSourceFilter>,
{
    let filters = discovered_source_filters(filters);
    sources
        .into_iter()
        .filter(|source| filters.is_empty() || source.matches_any_source(filters.iter()))
        .collect()
}

fn discovered_source_filters<I, F>(filters: I) -> Vec<LoadedSourceFilter>
where
    I: IntoIterator<Item = F>,
    F: Into<LoadedSourceFilter>,
{
    let mut unique = Vec::new();
    for filter in filters {
        let filter = filter.into();
        if !unique.iter().any(|existing| existing == &filter) {
            unique.push(filter);
        }
    }
    unique
}
