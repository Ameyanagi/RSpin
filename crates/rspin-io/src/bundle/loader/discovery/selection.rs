//! Selection helpers for discovered spectrum source candidates.

use std::path::Path;

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

    /// Selects discovered one-dimensional source candidates matching one source path.
    #[must_use]
    fn select_1d_by_source_path(&self, path: impl AsRef<Path>) -> Vec<&DiscoveredSpectrumSource>;

    /// Selects discovered one-dimensional source candidates matching one source path.
    ///
    /// This is a short alias for [`Self::select_1d_by_source_path`].
    #[must_use]
    fn select_1d_by_path(&self, path: impl AsRef<Path>) -> Vec<&DiscoveredSpectrumSource> {
        self.select_1d_by_source_path(path)
    }

    /// Selects discovered one-dimensional source candidates matching any source path.
    #[must_use]
    fn select_1d_by_source_paths<I, P>(&self, paths: I) -> Vec<&DiscoveredSpectrumSource>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>;

    /// Selects discovered one-dimensional source candidates matching any source path.
    ///
    /// This is a short alias for [`Self::select_1d_by_source_paths`].
    #[must_use]
    fn select_1d_by_paths<I, P>(&self, paths: I) -> Vec<&DiscoveredSpectrumSource>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.select_1d_by_source_paths(paths)
    }

    /// Selects discovered one-dimensional source candidates below one source path prefix.
    #[must_use]
    fn select_1d_by_source_path_prefix(
        &self,
        path: impl AsRef<Path>,
    ) -> Vec<&DiscoveredSpectrumSource>;

    /// Selects discovered one-dimensional source candidates below one source path prefix.
    ///
    /// This is a short alias for [`Self::select_1d_by_source_path_prefix`].
    #[must_use]
    fn select_1d_by_path_prefix(&self, path: impl AsRef<Path>) -> Vec<&DiscoveredSpectrumSource> {
        self.select_1d_by_source_path_prefix(path)
    }

    /// Selects discovered one-dimensional source candidates below any source path prefix.
    #[must_use]
    fn select_1d_by_source_path_prefixes<I, P>(&self, paths: I) -> Vec<&DiscoveredSpectrumSource>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>;

    /// Selects discovered one-dimensional source candidates below any source path prefix.
    ///
    /// This is a short alias for [`Self::select_1d_by_source_path_prefixes`].
    #[must_use]
    fn select_1d_by_path_prefixes<I, P>(&self, paths: I) -> Vec<&DiscoveredSpectrumSource>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.select_1d_by_source_path_prefixes(paths)
    }

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

    /// Selects discovered two-dimensional source candidates matching one source path.
    #[must_use]
    fn select_2d_by_source_path(&self, path: impl AsRef<Path>) -> Vec<&DiscoveredSpectrumSource>;

    /// Selects discovered two-dimensional source candidates matching one source path.
    ///
    /// This is a short alias for [`Self::select_2d_by_source_path`].
    #[must_use]
    fn select_2d_by_path(&self, path: impl AsRef<Path>) -> Vec<&DiscoveredSpectrumSource> {
        self.select_2d_by_source_path(path)
    }

    /// Selects discovered two-dimensional source candidates matching any source path.
    #[must_use]
    fn select_2d_by_source_paths<I, P>(&self, paths: I) -> Vec<&DiscoveredSpectrumSource>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>;

    /// Selects discovered two-dimensional source candidates matching any source path.
    ///
    /// This is a short alias for [`Self::select_2d_by_source_paths`].
    #[must_use]
    fn select_2d_by_paths<I, P>(&self, paths: I) -> Vec<&DiscoveredSpectrumSource>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.select_2d_by_source_paths(paths)
    }

    /// Selects discovered two-dimensional source candidates below one source path prefix.
    #[must_use]
    fn select_2d_by_source_path_prefix(
        &self,
        path: impl AsRef<Path>,
    ) -> Vec<&DiscoveredSpectrumSource>;

    /// Selects discovered two-dimensional source candidates below one source path prefix.
    ///
    /// This is a short alias for [`Self::select_2d_by_source_path_prefix`].
    #[must_use]
    fn select_2d_by_path_prefix(&self, path: impl AsRef<Path>) -> Vec<&DiscoveredSpectrumSource> {
        self.select_2d_by_source_path_prefix(path)
    }

    /// Selects discovered two-dimensional source candidates below any source path prefix.
    #[must_use]
    fn select_2d_by_source_path_prefixes<I, P>(&self, paths: I) -> Vec<&DiscoveredSpectrumSource>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>;

    /// Selects discovered two-dimensional source candidates below any source path prefix.
    ///
    /// This is a short alias for [`Self::select_2d_by_source_path_prefixes`].
    #[must_use]
    fn select_2d_by_path_prefixes<I, P>(&self, paths: I) -> Vec<&DiscoveredSpectrumSource>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.select_2d_by_source_path_prefixes(paths)
    }

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

    /// Selects discovered source candidates matching one source path.
    #[must_use]
    fn select_by_source_path(&self, path: impl AsRef<Path>) -> Vec<&DiscoveredSpectrumSource>;

    /// Selects discovered source candidates matching one source path.
    ///
    /// This is a short alias for [`Self::select_by_source_path`].
    #[must_use]
    fn select_by_path(&self, path: impl AsRef<Path>) -> Vec<&DiscoveredSpectrumSource> {
        self.select_by_source_path(path)
    }

    /// Selects discovered source candidates matching any source path.
    #[must_use]
    fn select_by_source_paths<I, P>(&self, paths: I) -> Vec<&DiscoveredSpectrumSource>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>;

    /// Selects discovered source candidates matching any source path.
    ///
    /// This is a short alias for [`Self::select_by_source_paths`].
    #[must_use]
    fn select_by_paths<I, P>(&self, paths: I) -> Vec<&DiscoveredSpectrumSource>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.select_by_source_paths(paths)
    }

    /// Selects discovered source candidates below one source path prefix.
    #[must_use]
    fn select_by_source_path_prefix(
        &self,
        path: impl AsRef<Path>,
    ) -> Vec<&DiscoveredSpectrumSource>;

    /// Selects discovered source candidates below one source path prefix.
    ///
    /// This is a short alias for [`Self::select_by_source_path_prefix`].
    #[must_use]
    fn select_by_path_prefix(&self, path: impl AsRef<Path>) -> Vec<&DiscoveredSpectrumSource> {
        self.select_by_source_path_prefix(path)
    }

    /// Selects discovered source candidates below any source path prefix.
    #[must_use]
    fn select_by_source_path_prefixes<I, P>(&self, paths: I) -> Vec<&DiscoveredSpectrumSource>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>;

    /// Selects discovered source candidates below any source path prefix.
    ///
    /// This is a short alias for [`Self::select_by_source_path_prefixes`].
    #[must_use]
    fn select_by_path_prefixes<I, P>(&self, paths: I) -> Vec<&DiscoveredSpectrumSource>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.select_by_source_path_prefixes(paths)
    }

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

    fn select_1d_by_source_path(&self, path: impl AsRef<Path>) -> Vec<&DiscoveredSpectrumSource> {
        select_discovered_spectra_1d_by_source_path(self, path)
    }

    fn select_1d_by_source_paths<I, P>(&self, paths: I) -> Vec<&DiscoveredSpectrumSource>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        select_discovered_spectra_1d_by_source_paths(self, paths)
    }

    fn select_1d_by_source_path_prefix(
        &self,
        path: impl AsRef<Path>,
    ) -> Vec<&DiscoveredSpectrumSource> {
        select_discovered_spectra_1d_by_source_path_prefix(self, path)
    }

    fn select_1d_by_source_path_prefixes<I, P>(&self, paths: I) -> Vec<&DiscoveredSpectrumSource>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        select_discovered_spectra_1d_by_source_path_prefixes(self, paths)
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

    fn select_2d_by_source_path(&self, path: impl AsRef<Path>) -> Vec<&DiscoveredSpectrumSource> {
        select_discovered_spectra_2d_by_source_path(self, path)
    }

    fn select_2d_by_source_paths<I, P>(&self, paths: I) -> Vec<&DiscoveredSpectrumSource>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        select_discovered_spectra_2d_by_source_paths(self, paths)
    }

    fn select_2d_by_source_path_prefix(
        &self,
        path: impl AsRef<Path>,
    ) -> Vec<&DiscoveredSpectrumSource> {
        select_discovered_spectra_2d_by_source_path_prefix(self, path)
    }

    fn select_2d_by_source_path_prefixes<I, P>(&self, paths: I) -> Vec<&DiscoveredSpectrumSource>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        select_discovered_spectra_2d_by_source_path_prefixes(self, paths)
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

    fn select_by_source_path(&self, path: impl AsRef<Path>) -> Vec<&DiscoveredSpectrumSource> {
        select_discovered_spectra_by_source_path(self, path)
    }

    fn select_by_source_paths<I, P>(&self, paths: I) -> Vec<&DiscoveredSpectrumSource>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        select_discovered_spectra_by_source_paths(self, paths)
    }

    fn select_by_source_path_prefix(
        &self,
        path: impl AsRef<Path>,
    ) -> Vec<&DiscoveredSpectrumSource> {
        select_discovered_spectra_by_source_path_prefix(self, path)
    }

    fn select_by_source_path_prefixes<I, P>(&self, paths: I) -> Vec<&DiscoveredSpectrumSource>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        select_discovered_spectra_by_source_path_prefixes(self, paths)
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

/// Selects discovered one-dimensional source candidates matching one source path.
#[must_use]
pub fn select_discovered_spectra_1d_by_source_path(
    sources: &[DiscoveredSpectrumSource],
    path: impl AsRef<Path>,
) -> Vec<&DiscoveredSpectrumSource> {
    select_discovered_spectra_1d_by_source(sources, LoadedSourceFilter::path(path))
}

/// Selects discovered one-dimensional source candidates matching one source path.
///
/// This is a short alias for [`select_discovered_spectra_1d_by_source_path`].
#[must_use]
pub fn select_discovered_spectra_1d_by_path(
    sources: &[DiscoveredSpectrumSource],
    path: impl AsRef<Path>,
) -> Vec<&DiscoveredSpectrumSource> {
    select_discovered_spectra_1d_by_source_path(sources, path)
}

/// Selects discovered one-dimensional source candidates matching any source path.
///
/// Paths are combined with logical OR. Passing an empty iterator returns all
/// discovered one-dimensional source candidates.
#[must_use]
pub fn select_discovered_spectra_1d_by_source_paths<I, P>(
    sources: &[DiscoveredSpectrumSource],
    paths: I,
) -> Vec<&DiscoveredSpectrumSource>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    select_discovered_spectra_1d_by_sources(sources, path_filters(paths))
}

/// Selects discovered one-dimensional source candidates matching any source path.
///
/// This is a short alias for [`select_discovered_spectra_1d_by_source_paths`].
#[must_use]
pub fn select_discovered_spectra_1d_by_paths<I, P>(
    sources: &[DiscoveredSpectrumSource],
    paths: I,
) -> Vec<&DiscoveredSpectrumSource>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    select_discovered_spectra_1d_by_source_paths(sources, paths)
}

/// Selects discovered one-dimensional source candidates below one source path prefix.
#[must_use]
pub fn select_discovered_spectra_1d_by_source_path_prefix(
    sources: &[DiscoveredSpectrumSource],
    path: impl AsRef<Path>,
) -> Vec<&DiscoveredSpectrumSource> {
    select_discovered_spectra_1d_by_source(sources, LoadedSourceFilter::path_prefix(path))
}

/// Selects discovered one-dimensional source candidates below one source path prefix.
///
/// This is a short alias for [`select_discovered_spectra_1d_by_source_path_prefix`].
#[must_use]
pub fn select_discovered_spectra_1d_by_path_prefix(
    sources: &[DiscoveredSpectrumSource],
    path: impl AsRef<Path>,
) -> Vec<&DiscoveredSpectrumSource> {
    select_discovered_spectra_1d_by_source_path_prefix(sources, path)
}

/// Selects discovered one-dimensional source candidates below any source path prefix.
///
/// Prefixes are combined with logical OR. Passing an empty iterator returns all
/// discovered one-dimensional source candidates.
#[must_use]
pub fn select_discovered_spectra_1d_by_source_path_prefixes<I, P>(
    sources: &[DiscoveredSpectrumSource],
    paths: I,
) -> Vec<&DiscoveredSpectrumSource>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    select_discovered_spectra_1d_by_sources(sources, path_prefix_filters(paths))
}

/// Selects discovered one-dimensional source candidates below any source path prefix.
///
/// This is a short alias for [`select_discovered_spectra_1d_by_source_path_prefixes`].
#[must_use]
pub fn select_discovered_spectra_1d_by_path_prefixes<I, P>(
    sources: &[DiscoveredSpectrumSource],
    paths: I,
) -> Vec<&DiscoveredSpectrumSource>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    select_discovered_spectra_1d_by_source_path_prefixes(sources, paths)
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

/// Selects discovered two-dimensional source candidates matching one source path.
#[must_use]
pub fn select_discovered_spectra_2d_by_source_path(
    sources: &[DiscoveredSpectrumSource],
    path: impl AsRef<Path>,
) -> Vec<&DiscoveredSpectrumSource> {
    select_discovered_spectra_2d_by_source(sources, LoadedSourceFilter::path(path))
}

/// Selects discovered two-dimensional source candidates matching one source path.
///
/// This is a short alias for [`select_discovered_spectra_2d_by_source_path`].
#[must_use]
pub fn select_discovered_spectra_2d_by_path(
    sources: &[DiscoveredSpectrumSource],
    path: impl AsRef<Path>,
) -> Vec<&DiscoveredSpectrumSource> {
    select_discovered_spectra_2d_by_source_path(sources, path)
}

/// Selects discovered two-dimensional source candidates matching any source path.
///
/// Paths are combined with logical OR. Passing an empty iterator returns all
/// discovered two-dimensional source candidates.
#[must_use]
pub fn select_discovered_spectra_2d_by_source_paths<I, P>(
    sources: &[DiscoveredSpectrumSource],
    paths: I,
) -> Vec<&DiscoveredSpectrumSource>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    select_discovered_spectra_2d_by_sources(sources, path_filters(paths))
}

/// Selects discovered two-dimensional source candidates matching any source path.
///
/// This is a short alias for [`select_discovered_spectra_2d_by_source_paths`].
#[must_use]
pub fn select_discovered_spectra_2d_by_paths<I, P>(
    sources: &[DiscoveredSpectrumSource],
    paths: I,
) -> Vec<&DiscoveredSpectrumSource>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    select_discovered_spectra_2d_by_source_paths(sources, paths)
}

/// Selects discovered two-dimensional source candidates below one source path prefix.
#[must_use]
pub fn select_discovered_spectra_2d_by_source_path_prefix(
    sources: &[DiscoveredSpectrumSource],
    path: impl AsRef<Path>,
) -> Vec<&DiscoveredSpectrumSource> {
    select_discovered_spectra_2d_by_source(sources, LoadedSourceFilter::path_prefix(path))
}

/// Selects discovered two-dimensional source candidates below one source path prefix.
///
/// This is a short alias for [`select_discovered_spectra_2d_by_source_path_prefix`].
#[must_use]
pub fn select_discovered_spectra_2d_by_path_prefix(
    sources: &[DiscoveredSpectrumSource],
    path: impl AsRef<Path>,
) -> Vec<&DiscoveredSpectrumSource> {
    select_discovered_spectra_2d_by_source_path_prefix(sources, path)
}

/// Selects discovered two-dimensional source candidates below any source path prefix.
///
/// Prefixes are combined with logical OR. Passing an empty iterator returns all
/// discovered two-dimensional source candidates.
#[must_use]
pub fn select_discovered_spectra_2d_by_source_path_prefixes<I, P>(
    sources: &[DiscoveredSpectrumSource],
    paths: I,
) -> Vec<&DiscoveredSpectrumSource>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    select_discovered_spectra_2d_by_sources(sources, path_prefix_filters(paths))
}

/// Selects discovered two-dimensional source candidates below any source path prefix.
///
/// This is a short alias for [`select_discovered_spectra_2d_by_source_path_prefixes`].
#[must_use]
pub fn select_discovered_spectra_2d_by_path_prefixes<I, P>(
    sources: &[DiscoveredSpectrumSource],
    paths: I,
) -> Vec<&DiscoveredSpectrumSource>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    select_discovered_spectra_2d_by_source_path_prefixes(sources, paths)
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

/// Selects discovered source candidates matching one source path.
#[must_use]
pub fn select_discovered_spectra_by_source_path(
    sources: &[DiscoveredSpectrumSource],
    path: impl AsRef<Path>,
) -> Vec<&DiscoveredSpectrumSource> {
    select_discovered_spectra_by_source(sources, LoadedSourceFilter::path(path))
}

/// Selects discovered source candidates matching one source path.
///
/// This is a short alias for [`select_discovered_spectra_by_source_path`].
#[must_use]
pub fn select_discovered_spectra_by_path(
    sources: &[DiscoveredSpectrumSource],
    path: impl AsRef<Path>,
) -> Vec<&DiscoveredSpectrumSource> {
    select_discovered_spectra_by_source_path(sources, path)
}

/// Selects discovered source candidates matching any source path.
///
/// Paths are combined with logical OR. Passing an empty iterator returns all
/// provided discovered sources, matching the unrestricted loader behavior.
#[must_use]
pub fn select_discovered_spectra_by_source_paths<I, P>(
    sources: &[DiscoveredSpectrumSource],
    paths: I,
) -> Vec<&DiscoveredSpectrumSource>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    select_discovered_spectra_by_sources(sources, path_filters(paths))
}

/// Selects discovered source candidates matching any source path.
///
/// This is a short alias for [`select_discovered_spectra_by_source_paths`].
#[must_use]
pub fn select_discovered_spectra_by_paths<I, P>(
    sources: &[DiscoveredSpectrumSource],
    paths: I,
) -> Vec<&DiscoveredSpectrumSource>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    select_discovered_spectra_by_source_paths(sources, paths)
}

/// Selects discovered source candidates below one source path prefix.
#[must_use]
pub fn select_discovered_spectra_by_source_path_prefix(
    sources: &[DiscoveredSpectrumSource],
    path: impl AsRef<Path>,
) -> Vec<&DiscoveredSpectrumSource> {
    select_discovered_spectra_by_source(sources, LoadedSourceFilter::path_prefix(path))
}

/// Selects discovered source candidates below one source path prefix.
///
/// This is a short alias for [`select_discovered_spectra_by_source_path_prefix`].
#[must_use]
pub fn select_discovered_spectra_by_path_prefix(
    sources: &[DiscoveredSpectrumSource],
    path: impl AsRef<Path>,
) -> Vec<&DiscoveredSpectrumSource> {
    select_discovered_spectra_by_source_path_prefix(sources, path)
}

/// Selects discovered source candidates below any source path prefix.
///
/// Prefixes are combined with logical OR. Passing an empty iterator returns all
/// provided discovered sources, matching the unrestricted loader behavior.
#[must_use]
pub fn select_discovered_spectra_by_source_path_prefixes<I, P>(
    sources: &[DiscoveredSpectrumSource],
    paths: I,
) -> Vec<&DiscoveredSpectrumSource>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    select_discovered_spectra_by_sources(sources, path_prefix_filters(paths))
}

/// Selects discovered source candidates below any source path prefix.
///
/// This is a short alias for [`select_discovered_spectra_by_source_path_prefixes`].
#[must_use]
pub fn select_discovered_spectra_by_path_prefixes<I, P>(
    sources: &[DiscoveredSpectrumSource],
    paths: I,
) -> Vec<&DiscoveredSpectrumSource>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    select_discovered_spectra_by_source_path_prefixes(sources, paths)
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
