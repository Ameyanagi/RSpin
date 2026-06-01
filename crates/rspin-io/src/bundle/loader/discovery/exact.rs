//! Exact single-spectrum loading from discovered source candidates.

use std::path::Path;

use rspin_core::{Result, Spectrum1D, Spectrum2D};

use super::{DiscoveredSpectrumSource, selection};
use crate::bundle::{LoadedSource, LoadedSourceFilter, SpectrumBundleLoader};

impl SpectrumBundleLoader {
    /// Loads selected discovered source candidates as exactly one one-dimensional spectrum.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the selected discovered sources do
    /// not resolve to exactly one one-dimensional spectrum.
    pub fn read_discovered_1d_relative_to<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    ) -> Result<Spectrum1D> {
        self.read_discovered_relative_to(base, sources)?
            .into_only_1d()
    }

    /// Loads selected discovered source candidates as exactly one one-dimensional spectrum.
    ///
    /// This short alias mirrors [`Self::read_discovered_1d_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the selected discovered sources do
    /// not resolve to exactly one one-dimensional spectrum.
    pub fn read_discovered_1d<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    ) -> Result<Spectrum1D> {
        self.read_discovered_1d_relative_to(base, sources)
    }

    /// Loads discovered source candidates matching one generic source filter as exactly one one-dimensional spectrum.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the matching discovered sources do
    /// not resolve to exactly one one-dimensional spectrum.
    pub fn read_discovered_1d_by_source_relative_to<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        filter: impl Into<LoadedSourceFilter>,
    ) -> Result<Spectrum1D> {
        let filter = filter.into();
        self.read_discovered_1d_by_sources_relative_to(base, sources, [filter])
    }

    /// Loads discovered source candidates matching one generic source filter as exactly one one-dimensional spectrum.
    ///
    /// This short alias mirrors [`Self::read_discovered_1d_by_source_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the matching discovered sources do
    /// not resolve to exactly one one-dimensional spectrum.
    pub fn read_discovered_1d_by_source<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        filter: impl Into<LoadedSourceFilter>,
    ) -> Result<Spectrum1D> {
        self.read_discovered_1d_by_source_relative_to(base, sources, filter)
    }

    /// Loads discovered source candidates matching any generic source filter as exactly one one-dimensional spectrum.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the matching discovered sources do
    /// not resolve to exactly one one-dimensional spectrum.
    pub fn read_discovered_1d_by_sources_relative_to<'a, I, F>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        filters: I,
    ) -> Result<Spectrum1D>
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>,
    {
        let selected = selection::select_discovered_source_refs(sources, filters);
        self.read_discovered_1d_relative_to(base, selected)
    }

    /// Loads discovered source candidates matching any generic source filter as exactly one one-dimensional spectrum.
    ///
    /// This short alias mirrors [`Self::read_discovered_1d_by_sources_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the matching discovered sources do
    /// not resolve to exactly one one-dimensional spectrum.
    pub fn read_discovered_1d_by_sources<'a, I, F>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        filters: I,
    ) -> Result<Spectrum1D>
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>,
    {
        self.read_discovered_1d_by_sources_relative_to(base, sources, filters)
    }

    /// Loads selected discovered source candidates as exactly one one-dimensional spectrum with source metadata.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the selected discovered sources do
    /// not resolve to exactly one one-dimensional spectrum.
    pub fn read_discovered_1d_with_source_relative_to<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    ) -> Result<(Spectrum1D, LoadedSource)> {
        self.read_discovered_relative_to(base, sources)?
            .into_only_loaded_1d()
    }

    /// Loads selected discovered source candidates as exactly one one-dimensional spectrum with source metadata.
    ///
    /// This short alias mirrors [`Self::read_discovered_1d_with_source_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the selected discovered sources do
    /// not resolve to exactly one one-dimensional spectrum.
    pub fn read_discovered_1d_with_source<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    ) -> Result<(Spectrum1D, LoadedSource)> {
        self.read_discovered_1d_with_source_relative_to(base, sources)
    }

    /// Loads discovered source candidates matching one generic source filter as exactly one one-dimensional spectrum with source metadata.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the matching discovered sources do
    /// not resolve to exactly one one-dimensional spectrum.
    pub fn read_discovered_1d_with_source_by_source_relative_to<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        filter: impl Into<LoadedSourceFilter>,
    ) -> Result<(Spectrum1D, LoadedSource)> {
        let filter = filter.into();
        self.read_discovered_1d_with_source_by_sources_relative_to(base, sources, [filter])
    }

    /// Loads discovered source candidates matching one generic source filter as exactly one one-dimensional spectrum with source metadata.
    ///
    /// This short alias mirrors [`Self::read_discovered_1d_with_source_by_source_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the matching discovered sources do
    /// not resolve to exactly one one-dimensional spectrum.
    pub fn read_discovered_1d_with_source_by_source<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        filter: impl Into<LoadedSourceFilter>,
    ) -> Result<(Spectrum1D, LoadedSource)> {
        self.read_discovered_1d_with_source_by_source_relative_to(base, sources, filter)
    }

    /// Loads discovered source candidates matching any generic source filter as exactly one one-dimensional spectrum with source metadata.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the matching discovered sources do
    /// not resolve to exactly one one-dimensional spectrum.
    pub fn read_discovered_1d_with_source_by_sources_relative_to<'a, I, F>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        filters: I,
    ) -> Result<(Spectrum1D, LoadedSource)>
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>,
    {
        let selected = selection::select_discovered_source_refs(sources, filters);
        self.read_discovered_1d_with_source_relative_to(base, selected)
    }

    /// Loads discovered source candidates matching any generic source filter as exactly one one-dimensional spectrum with source metadata.
    ///
    /// This short alias mirrors [`Self::read_discovered_1d_with_source_by_sources_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the matching discovered sources do
    /// not resolve to exactly one one-dimensional spectrum.
    pub fn read_discovered_1d_with_source_by_sources<'a, I, F>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        filters: I,
    ) -> Result<(Spectrum1D, LoadedSource)>
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>,
    {
        self.read_discovered_1d_with_source_by_sources_relative_to(base, sources, filters)
    }

    /// Loads selected discovered source candidates as exactly one two-dimensional spectrum.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the selected discovered sources do
    /// not resolve to exactly one two-dimensional spectrum.
    pub fn read_discovered_2d_relative_to<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    ) -> Result<Spectrum2D> {
        self.read_discovered_relative_to(base, sources)?
            .into_only_2d()
    }

    /// Loads selected discovered source candidates as exactly one two-dimensional spectrum.
    ///
    /// This short alias mirrors [`Self::read_discovered_2d_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the selected discovered sources do
    /// not resolve to exactly one two-dimensional spectrum.
    pub fn read_discovered_2d<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    ) -> Result<Spectrum2D> {
        self.read_discovered_2d_relative_to(base, sources)
    }

    /// Loads discovered source candidates matching one generic source filter as exactly one two-dimensional spectrum.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the matching discovered sources do
    /// not resolve to exactly one two-dimensional spectrum.
    pub fn read_discovered_2d_by_source_relative_to<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        filter: impl Into<LoadedSourceFilter>,
    ) -> Result<Spectrum2D> {
        let filter = filter.into();
        self.read_discovered_2d_by_sources_relative_to(base, sources, [filter])
    }

    /// Loads discovered source candidates matching one generic source filter as exactly one two-dimensional spectrum.
    ///
    /// This short alias mirrors [`Self::read_discovered_2d_by_source_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the matching discovered sources do
    /// not resolve to exactly one two-dimensional spectrum.
    pub fn read_discovered_2d_by_source<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        filter: impl Into<LoadedSourceFilter>,
    ) -> Result<Spectrum2D> {
        self.read_discovered_2d_by_source_relative_to(base, sources, filter)
    }

    /// Loads discovered source candidates matching any generic source filter as exactly one two-dimensional spectrum.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the matching discovered sources do
    /// not resolve to exactly one two-dimensional spectrum.
    pub fn read_discovered_2d_by_sources_relative_to<'a, I, F>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        filters: I,
    ) -> Result<Spectrum2D>
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>,
    {
        let selected = selection::select_discovered_source_refs(sources, filters);
        self.read_discovered_2d_relative_to(base, selected)
    }

    /// Loads discovered source candidates matching any generic source filter as exactly one two-dimensional spectrum.
    ///
    /// This short alias mirrors [`Self::read_discovered_2d_by_sources_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the matching discovered sources do
    /// not resolve to exactly one two-dimensional spectrum.
    pub fn read_discovered_2d_by_sources<'a, I, F>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        filters: I,
    ) -> Result<Spectrum2D>
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>,
    {
        self.read_discovered_2d_by_sources_relative_to(base, sources, filters)
    }

    /// Loads selected discovered source candidates as exactly one two-dimensional spectrum with source metadata.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the selected discovered sources do
    /// not resolve to exactly one two-dimensional spectrum.
    pub fn read_discovered_2d_with_source_relative_to<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    ) -> Result<(Spectrum2D, LoadedSource)> {
        self.read_discovered_relative_to(base, sources)?
            .into_only_loaded_2d()
    }

    /// Loads selected discovered source candidates as exactly one two-dimensional spectrum with source metadata.
    ///
    /// This short alias mirrors [`Self::read_discovered_2d_with_source_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the selected discovered sources do
    /// not resolve to exactly one two-dimensional spectrum.
    pub fn read_discovered_2d_with_source<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    ) -> Result<(Spectrum2D, LoadedSource)> {
        self.read_discovered_2d_with_source_relative_to(base, sources)
    }

    /// Loads discovered source candidates matching one generic source filter as exactly one two-dimensional spectrum with source metadata.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the matching discovered sources do
    /// not resolve to exactly one two-dimensional spectrum.
    pub fn read_discovered_2d_with_source_by_source_relative_to<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        filter: impl Into<LoadedSourceFilter>,
    ) -> Result<(Spectrum2D, LoadedSource)> {
        let filter = filter.into();
        self.read_discovered_2d_with_source_by_sources_relative_to(base, sources, [filter])
    }

    /// Loads discovered source candidates matching one generic source filter as exactly one two-dimensional spectrum with source metadata.
    ///
    /// This short alias mirrors [`Self::read_discovered_2d_with_source_by_source_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the matching discovered sources do
    /// not resolve to exactly one two-dimensional spectrum.
    pub fn read_discovered_2d_with_source_by_source<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        filter: impl Into<LoadedSourceFilter>,
    ) -> Result<(Spectrum2D, LoadedSource)> {
        self.read_discovered_2d_with_source_by_source_relative_to(base, sources, filter)
    }

    /// Loads discovered source candidates matching any generic source filter as exactly one two-dimensional spectrum with source metadata.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the matching discovered sources do
    /// not resolve to exactly one two-dimensional spectrum.
    pub fn read_discovered_2d_with_source_by_sources_relative_to<'a, I, F>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        filters: I,
    ) -> Result<(Spectrum2D, LoadedSource)>
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>,
    {
        let selected = selection::select_discovered_source_refs(sources, filters);
        self.read_discovered_2d_with_source_relative_to(base, selected)
    }

    /// Loads discovered source candidates matching any generic source filter as exactly one two-dimensional spectrum with source metadata.
    ///
    /// This short alias mirrors [`Self::read_discovered_2d_with_source_by_sources_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the matching discovered sources do
    /// not resolve to exactly one two-dimensional spectrum.
    pub fn read_discovered_2d_with_source_by_sources<'a, I, F>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        filters: I,
    ) -> Result<(Spectrum2D, LoadedSource)>
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>,
    {
        self.read_discovered_2d_with_source_by_sources_relative_to(base, sources, filters)
    }
}
