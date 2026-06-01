//! Data model for lightweight spectrum source discovery.

use std::path::{Path, PathBuf};

use rspin_core::{Result, Spectrum1D, Spectrum2D};
use serde::{Deserialize, Serialize};

use super::super::SpectrumBundleLoader;
use crate::bundle::{
    LoadedSource, LoadedSourceDataKind, LoadedSourceFilter, LoadedSourceFormat, LoadedSourceVendor,
    SpectrumBundle,
};

/// Dimension identified for a discovered source candidate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveredSpectrumDimension {
    /// One-dimensional spectrum source.
    OneD,
    /// Two-dimensional spectrum source.
    TwoD,
    /// Spectrum-like source whose exact dimension is not known without parsing.
    Unknown,
}

impl DiscoveredSpectrumDimension {
    /// Returns the numeric spectrum dimension when known.
    #[must_use]
    pub const fn as_usize(self) -> Option<usize> {
        match self {
            Self::OneD => Some(1),
            Self::TwoD => Some(2),
            Self::Unknown => None,
        }
    }

    /// Returns true for a discovered one-dimensional source.
    #[must_use]
    pub const fn is_1d(self) -> bool {
        matches!(self, Self::OneD)
    }

    /// Returns true for a discovered two-dimensional source.
    #[must_use]
    pub const fn is_2d(self) -> bool {
        matches!(self, Self::TwoD)
    }

    /// Returns true when the dimension was not resolved during discovery.
    #[must_use]
    pub const fn is_unknown(self) -> bool {
        matches!(self, Self::Unknown)
    }
}

/// A source candidate that the unified loader can route.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveredSpectrumSource {
    /// Relative source path when source path tracking is enabled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
    /// Reader format that would be used for this source.
    pub format: String,
    /// Coarse raw/processed classification for the source.
    pub data_kind: LoadedSourceDataKind,
    /// Spectrum dimension inferred from path metadata or lightweight format detection.
    pub dimension: DiscoveredSpectrumDimension,
}

impl DiscoveredSpectrumSource {
    /// Creates discovered source metadata.
    #[must_use]
    pub fn new(
        path: Option<PathBuf>,
        format: impl Into<String>,
        dimension: DiscoveredSpectrumDimension,
    ) -> Self {
        let format = format.into();
        let data_kind = LoadedSourceFormat::parse(&format)
            .map_or(LoadedSourceDataKind::Other, LoadedSourceFormat::data_kind);
        Self {
            path,
            format,
            data_kind,
            dimension,
        }
    }

    /// Returns the tracked source path, if source path tracking was enabled.
    #[must_use]
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Returns the reader format that would be used for this source.
    #[must_use]
    pub fn format(&self) -> &str {
        &self.format
    }

    /// Returns the known source format, if this source uses a built-in reader name.
    #[must_use]
    pub fn format_kind(&self) -> Option<LoadedSourceFormat> {
        LoadedSourceFormat::parse(&self.format).ok()
    }

    /// Returns the vendor family for vendor-specific source formats.
    #[must_use]
    pub fn vendor(&self) -> Option<LoadedSourceVendor> {
        self.format_kind().and_then(LoadedSourceFormat::vendor)
    }

    /// Returns the coarse raw/processed source classification.
    #[must_use]
    pub const fn data_kind(&self) -> LoadedSourceDataKind {
        self.data_kind
    }

    /// Returns the inferred source dimension.
    #[must_use]
    pub const fn dimension(&self) -> DiscoveredSpectrumDimension {
        self.dimension
    }

    /// Returns source metadata with the same path and format.
    #[must_use]
    pub fn source(&self) -> LoadedSource {
        LoadedSource::new(self.path.clone(), self.format.clone())
    }

    /// Loads this discovered source relative to a common base directory.
    ///
    /// This is a convenience wrapper for `RSpinReader::read_discovered` when a
    /// preflight workflow has selected one candidate. The source must include a
    /// tracked source path.
    ///
    /// # Errors
    ///
    /// Returns an error when the base directory is invalid, this source has no
    /// tracked path, strict loading would reject the source, or no data can be read.
    pub fn load_relative_to(&self, base: impl AsRef<Path>) -> Result<SpectrumBundle> {
        SpectrumBundleLoader::new().read_discovered_relative_to(base, [self])
    }

    /// Loads this discovered source relative to a common base directory.
    ///
    /// This short alias mirrors [`Self::load_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading this discovered source fails.
    pub fn load(&self, base: impl AsRef<Path>) -> Result<SpectrumBundle> {
        self.load_relative_to(base)
    }

    /// Loads this discovered source in strict mode relative to a common base directory.
    ///
    /// # Errors
    ///
    /// Returns an error when the base directory is invalid, this source has no
    /// tracked path, the source cannot be read, or no data can be read.
    pub fn load_strict_relative_to(&self, base: impl AsRef<Path>) -> Result<SpectrumBundle> {
        SpectrumBundleLoader::new()
            .strict()
            .read_discovered_relative_to(base, [self])
    }

    /// Loads this discovered source as exactly one one-dimensional spectrum.
    ///
    /// # Errors
    ///
    /// Returns an error when loading this discovered source fails or when it does
    /// not resolve to exactly one one-dimensional spectrum.
    pub fn load_1d_relative_to(&self, base: impl AsRef<Path>) -> Result<Spectrum1D> {
        self.load_relative_to(base)?.into_only_1d()
    }

    /// Loads this discovered source as exactly one one-dimensional spectrum.
    ///
    /// This short alias mirrors [`Self::load_1d_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading this discovered source fails or when it does
    /// not resolve to exactly one one-dimensional spectrum.
    pub fn load_1d(&self, base: impl AsRef<Path>) -> Result<Spectrum1D> {
        self.load_1d_relative_to(base)
    }

    /// Loads this discovered source as exactly one one-dimensional spectrum with source metadata.
    ///
    /// # Errors
    ///
    /// Returns an error when loading this discovered source fails or when it does
    /// not resolve to exactly one one-dimensional spectrum.
    pub fn load_1d_with_source_relative_to(
        &self,
        base: impl AsRef<Path>,
    ) -> Result<(Spectrum1D, LoadedSource)> {
        self.load_relative_to(base)?.into_only_loaded_1d()
    }

    /// Loads this discovered source as exactly one one-dimensional spectrum with source metadata.
    ///
    /// This short alias mirrors [`Self::load_1d_with_source_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading this discovered source fails or when it does
    /// not resolve to exactly one one-dimensional spectrum.
    pub fn load_1d_with_source(
        &self,
        base: impl AsRef<Path>,
    ) -> Result<(Spectrum1D, LoadedSource)> {
        self.load_1d_with_source_relative_to(base)
    }

    /// Loads this discovered source as exactly one two-dimensional spectrum.
    ///
    /// # Errors
    ///
    /// Returns an error when loading this discovered source fails or when it does
    /// not resolve to exactly one two-dimensional spectrum.
    pub fn load_2d_relative_to(&self, base: impl AsRef<Path>) -> Result<Spectrum2D> {
        self.load_relative_to(base)?.into_only_2d()
    }

    /// Loads this discovered source as exactly one two-dimensional spectrum.
    ///
    /// This short alias mirrors [`Self::load_2d_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading this discovered source fails or when it does
    /// not resolve to exactly one two-dimensional spectrum.
    pub fn load_2d(&self, base: impl AsRef<Path>) -> Result<Spectrum2D> {
        self.load_2d_relative_to(base)
    }

    /// Loads this discovered source as exactly one two-dimensional spectrum with source metadata.
    ///
    /// # Errors
    ///
    /// Returns an error when loading this discovered source fails or when it does
    /// not resolve to exactly one two-dimensional spectrum.
    pub fn load_2d_with_source_relative_to(
        &self,
        base: impl AsRef<Path>,
    ) -> Result<(Spectrum2D, LoadedSource)> {
        self.load_relative_to(base)?.into_only_loaded_2d()
    }

    /// Loads this discovered source as exactly one two-dimensional spectrum with source metadata.
    ///
    /// This short alias mirrors [`Self::load_2d_with_source_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading this discovered source fails or when it does
    /// not resolve to exactly one two-dimensional spectrum.
    pub fn load_2d_with_source(
        &self,
        base: impl AsRef<Path>,
    ) -> Result<(Spectrum2D, LoadedSource)> {
        self.load_2d_with_source_relative_to(base)
    }

    /// Returns true when this discovered source matches a generic source filter.
    #[must_use]
    pub fn matches_source(&self, filter: impl Into<LoadedSourceFilter>) -> bool {
        filter.into().matches_source(&self.source())
    }

    /// Returns true when this discovered source matches any generic source filter.
    ///
    /// Empty filter iterators return false because this is a candidate predicate,
    /// not a loader restriction.
    #[must_use]
    pub fn matches_any_source<I, F>(&self, filters: I) -> bool
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>,
    {
        let source = self.source();
        filters
            .into_iter()
            .any(|filter| filter.into().matches_source(&source))
    }

    /// Returns true when this source would use the requested source format.
    #[must_use]
    pub fn is_format(&self, format: impl AsRef<str>) -> bool {
        self.source().is_format(format)
    }

    /// Returns true when this source would use a vendor-specific reader.
    #[must_use]
    pub fn is_vendor(&self, vendor: impl AsRef<str>) -> bool {
        self.source().is_vendor(vendor)
    }

    /// Returns true when discovery inferred a one-dimensional source.
    #[must_use]
    pub const fn is_1d(&self) -> bool {
        self.dimension.is_1d()
    }

    /// Returns true when discovery inferred a two-dimensional source.
    #[must_use]
    pub const fn is_2d(&self) -> bool {
        self.dimension.is_2d()
    }

    /// Returns true when discovery could not infer the spectrum dimension.
    #[must_use]
    pub const fn is_unknown_dimension(&self) -> bool {
        self.dimension.is_unknown()
    }

    /// Returns true when this source represents vendor raw acquisition data.
    #[must_use]
    pub const fn is_raw(&self) -> bool {
        matches!(self.data_kind, LoadedSourceDataKind::Raw)
    }

    /// Returns true when this source represents vendor processed data.
    #[must_use]
    pub const fn is_processed(&self) -> bool {
        matches!(self.data_kind, LoadedSourceDataKind::Processed)
    }
}
