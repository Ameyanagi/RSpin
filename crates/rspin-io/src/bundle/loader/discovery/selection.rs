//! Selection helpers for discovered spectrum source candidates.

use super::summary::DiscoveredSpectrumSummary;
use super::{DiscoveredSpectrumDimension, DiscoveredSpectrumSource};
use crate::bundle::LoadedSourceFilter;

/// Slice extension methods for discovered spectrum source candidates.
///
/// This keeps preflight workflows readable without replacing the free
/// selection helpers:
///
/// ```ignore
/// let sources = RSpinReader::new().discover("data")?;
/// let selected = sources.select_1d_by_source(LoadedSourceVendor::Jeol);
/// ```
pub trait DiscoveredSpectrumSourcesExt {
    /// Selects discovered one-dimensional source candidates.
    #[must_use]
    fn select_1d(&self) -> Vec<&DiscoveredSpectrumSource>;

    /// Selects discovered one-dimensional source candidates matching one source filter.
    #[must_use]
    fn select_1d_by_source(
        &self,
        filter: impl Into<LoadedSourceFilter>,
    ) -> Vec<&DiscoveredSpectrumSource>;

    /// Selects discovered one-dimensional source candidates matching any source filter.
    #[must_use]
    fn select_1d_by_sources<I, F>(&self, filters: I) -> Vec<&DiscoveredSpectrumSource>
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>;

    /// Selects discovered two-dimensional source candidates.
    #[must_use]
    fn select_2d(&self) -> Vec<&DiscoveredSpectrumSource>;

    /// Selects discovered two-dimensional source candidates matching one source filter.
    #[must_use]
    fn select_2d_by_source(
        &self,
        filter: impl Into<LoadedSourceFilter>,
    ) -> Vec<&DiscoveredSpectrumSource>;

    /// Selects discovered two-dimensional source candidates matching any source filter.
    #[must_use]
    fn select_2d_by_sources<I, F>(&self, filters: I) -> Vec<&DiscoveredSpectrumSource>
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>;

    /// Selects discovered source candidates with one inferred dimension.
    #[must_use]
    fn select_by_dimension(
        &self,
        dimension: DiscoveredSpectrumDimension,
    ) -> Vec<&DiscoveredSpectrumSource>;

    /// Selects discovered source candidates with one inferred dimension and source filter.
    #[must_use]
    fn select_by_dimension_and_source(
        &self,
        dimension: DiscoveredSpectrumDimension,
        filter: impl Into<LoadedSourceFilter>,
    ) -> Vec<&DiscoveredSpectrumSource>;

    /// Selects discovered source candidates with one inferred dimension and any source filter.
    #[must_use]
    fn select_by_dimension_and_sources<I, F>(
        &self,
        dimension: DiscoveredSpectrumDimension,
        filters: I,
    ) -> Vec<&DiscoveredSpectrumSource>
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>;

    /// Selects discovered source candidates matching one source filter.
    #[must_use]
    fn select_by_source(
        &self,
        filter: impl Into<LoadedSourceFilter>,
    ) -> Vec<&DiscoveredSpectrumSource>;

    /// Selects discovered source candidates matching any source filter.
    #[must_use]
    fn select_by_sources<I, F>(&self, filters: I) -> Vec<&DiscoveredSpectrumSource>
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>;

    /// Summarizes discovered source candidates.
    #[must_use]
    fn summarize(&self) -> DiscoveredSpectrumSummary;
}

impl DiscoveredSpectrumSourcesExt for [DiscoveredSpectrumSource] {
    fn select_1d(&self) -> Vec<&DiscoveredSpectrumSource> {
        select_discovered_spectra_1d(self)
    }

    fn select_1d_by_source(
        &self,
        filter: impl Into<LoadedSourceFilter>,
    ) -> Vec<&DiscoveredSpectrumSource> {
        select_discovered_spectra_1d_by_source(self, filter)
    }

    fn select_1d_by_sources<I, F>(&self, filters: I) -> Vec<&DiscoveredSpectrumSource>
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>,
    {
        select_discovered_spectra_1d_by_sources(self, filters)
    }

    fn select_2d(&self) -> Vec<&DiscoveredSpectrumSource> {
        select_discovered_spectra_2d(self)
    }

    fn select_2d_by_source(
        &self,
        filter: impl Into<LoadedSourceFilter>,
    ) -> Vec<&DiscoveredSpectrumSource> {
        select_discovered_spectra_2d_by_source(self, filter)
    }

    fn select_2d_by_sources<I, F>(&self, filters: I) -> Vec<&DiscoveredSpectrumSource>
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>,
    {
        select_discovered_spectra_2d_by_sources(self, filters)
    }

    fn select_by_dimension(
        &self,
        dimension: DiscoveredSpectrumDimension,
    ) -> Vec<&DiscoveredSpectrumSource> {
        select_discovered_spectra_by_dimension(self, dimension)
    }

    fn select_by_dimension_and_source(
        &self,
        dimension: DiscoveredSpectrumDimension,
        filter: impl Into<LoadedSourceFilter>,
    ) -> Vec<&DiscoveredSpectrumSource> {
        select_discovered_spectra_by_dimension_and_source(self, dimension, filter)
    }

    fn select_by_dimension_and_sources<I, F>(
        &self,
        dimension: DiscoveredSpectrumDimension,
        filters: I,
    ) -> Vec<&DiscoveredSpectrumSource>
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>,
    {
        select_discovered_spectra_by_dimension_and_sources(self, dimension, filters)
    }

    fn select_by_source(
        &self,
        filter: impl Into<LoadedSourceFilter>,
    ) -> Vec<&DiscoveredSpectrumSource> {
        select_discovered_spectra_by_source(self, filter)
    }

    fn select_by_sources<I, F>(&self, filters: I) -> Vec<&DiscoveredSpectrumSource>
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>,
    {
        select_discovered_spectra_by_sources(self, filters)
    }

    fn summarize(&self) -> DiscoveredSpectrumSummary {
        DiscoveredSpectrumSummary::new(self)
    }
}

/// Selects discovered one-dimensional source candidates.
#[must_use]
pub fn select_discovered_spectra_1d(
    sources: &[DiscoveredSpectrumSource],
) -> Vec<&DiscoveredSpectrumSource> {
    select_discovered_spectra_by_dimension(sources, DiscoveredSpectrumDimension::OneD)
}

/// Selects discovered one-dimensional source candidates matching one generic source filter.
#[must_use]
pub fn select_discovered_spectra_1d_by_source(
    sources: &[DiscoveredSpectrumSource],
    filter: impl Into<LoadedSourceFilter>,
) -> Vec<&DiscoveredSpectrumSource> {
    let filter = filter.into();
    select_discovered_spectra_1d_by_sources(sources, [filter])
}

/// Selects discovered one-dimensional source candidates matching any generic source filter.
///
/// Filters are combined with logical OR. Passing an empty iterator returns all
/// discovered one-dimensional source candidates.
#[must_use]
pub fn select_discovered_spectra_1d_by_sources<I, F>(
    sources: &[DiscoveredSpectrumSource],
    filters: I,
) -> Vec<&DiscoveredSpectrumSource>
where
    I: IntoIterator<Item = F>,
    F: Into<LoadedSourceFilter>,
{
    select_discovered_spectra_by_dimension_and_sources(
        sources,
        DiscoveredSpectrumDimension::OneD,
        filters,
    )
}

/// Selects discovered two-dimensional source candidates.
#[must_use]
pub fn select_discovered_spectra_2d(
    sources: &[DiscoveredSpectrumSource],
) -> Vec<&DiscoveredSpectrumSource> {
    select_discovered_spectra_by_dimension(sources, DiscoveredSpectrumDimension::TwoD)
}

/// Selects discovered two-dimensional source candidates matching one generic source filter.
#[must_use]
pub fn select_discovered_spectra_2d_by_source(
    sources: &[DiscoveredSpectrumSource],
    filter: impl Into<LoadedSourceFilter>,
) -> Vec<&DiscoveredSpectrumSource> {
    let filter = filter.into();
    select_discovered_spectra_2d_by_sources(sources, [filter])
}

/// Selects discovered two-dimensional source candidates matching any generic source filter.
///
/// Filters are combined with logical OR. Passing an empty iterator returns all
/// discovered two-dimensional source candidates.
#[must_use]
pub fn select_discovered_spectra_2d_by_sources<I, F>(
    sources: &[DiscoveredSpectrumSource],
    filters: I,
) -> Vec<&DiscoveredSpectrumSource>
where
    I: IntoIterator<Item = F>,
    F: Into<LoadedSourceFilter>,
{
    select_discovered_spectra_by_dimension_and_sources(
        sources,
        DiscoveredSpectrumDimension::TwoD,
        filters,
    )
}

/// Selects discovered source candidates with one inferred dimension.
#[must_use]
pub fn select_discovered_spectra_by_dimension(
    sources: &[DiscoveredSpectrumSource],
    dimension: DiscoveredSpectrumDimension,
) -> Vec<&DiscoveredSpectrumSource> {
    select_discovered_source_refs_by_dimension(sources.iter(), dimension)
}

/// Selects discovered source candidates with one inferred dimension and one generic source filter.
#[must_use]
pub fn select_discovered_spectra_by_dimension_and_source(
    sources: &[DiscoveredSpectrumSource],
    dimension: DiscoveredSpectrumDimension,
    filter: impl Into<LoadedSourceFilter>,
) -> Vec<&DiscoveredSpectrumSource> {
    let filter = filter.into();
    select_discovered_spectra_by_dimension_and_sources(sources, dimension, [filter])
}

/// Selects discovered source candidates with one inferred dimension and any generic source filter.
///
/// Filters are combined with logical OR. Passing an empty iterator returns all
/// discovered source candidates with the requested inferred dimension.
#[must_use]
pub fn select_discovered_spectra_by_dimension_and_sources<I, F>(
    sources: &[DiscoveredSpectrumSource],
    dimension: DiscoveredSpectrumDimension,
    filters: I,
) -> Vec<&DiscoveredSpectrumSource>
where
    I: IntoIterator<Item = F>,
    F: Into<LoadedSourceFilter>,
{
    select_discovered_source_refs_by_dimension_and_sources(sources.iter(), dimension, filters)
}

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

pub(super) fn select_discovered_source_refs_by_dimension<'a, S>(
    sources: S,
    dimension: DiscoveredSpectrumDimension,
) -> Vec<&'a DiscoveredSpectrumSource>
where
    S: IntoIterator<Item = &'a DiscoveredSpectrumSource>,
{
    sources
        .into_iter()
        .filter(|source| source.dimension() == dimension)
        .collect()
}

pub(super) fn select_discovered_source_refs_by_dimension_and_sources<'a, S, I, F>(
    sources: S,
    dimension: DiscoveredSpectrumDimension,
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
        .filter(|source| {
            source.dimension() == dimension
                && (filters.is_empty() || source.matches_any_source(filters.iter()))
        })
        .collect()
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
