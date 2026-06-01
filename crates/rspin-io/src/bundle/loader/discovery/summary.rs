//! Summary counts for lightweight source discovery.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::{DiscoveredSpectrumDimension, DiscoveredSpectrumSource};
use crate::bundle::{
    LoadedSourceDataKind, LoadedSourceFilter, LoadedSourceVendor, SourceDataKindCount,
    SourceFormatCount, SourceVendorCount, source_format_matches,
};

const DISCOVERY_DIMENSIONS: &[DiscoveredSpectrumDimension] = &[
    DiscoveredSpectrumDimension::OneD,
    DiscoveredSpectrumDimension::TwoD,
    DiscoveredSpectrumDimension::Unknown,
];

/// Deterministic count of discovered sources for one inferred dimension.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveredSpectrumDimensionCount {
    /// Inferred spectrum dimension.
    pub dimension: DiscoveredSpectrumDimension,
    /// Number of discovered sources with this dimension.
    pub count: usize,
}

impl DiscoveredSpectrumDimensionCount {
    /// Creates a discovered dimension count.
    #[must_use]
    pub const fn new(dimension: DiscoveredSpectrumDimension, count: usize) -> Self {
        Self { dimension, count }
    }

    /// Returns the inferred spectrum dimension.
    #[must_use]
    pub const fn dimension(&self) -> DiscoveredSpectrumDimension {
        self.dimension
    }

    /// Returns the number of discovered sources with this dimension.
    #[must_use]
    pub const fn count(&self) -> usize {
        self.count
    }
}

/// Deterministic count of discovered sources for one tracked source path.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveredSpectrumPathCount {
    /// Tracked source path.
    pub path: PathBuf,
    /// Number of discovered sources with this path.
    pub count: usize,
}

impl DiscoveredSpectrumPathCount {
    /// Creates a discovered source-path count.
    #[must_use]
    pub fn new(path: impl Into<PathBuf>, count: usize) -> Self {
        Self {
            path: path.into(),
            count,
        }
    }

    /// Returns the tracked source path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the number of discovered sources with this path.
    #[must_use]
    pub const fn count(&self) -> usize {
        self.count
    }
}

/// Summary counts for lightweight source discovery.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveredSpectrumSummary {
    /// Total number of discovered source candidates.
    pub sources: usize,
    /// Number of discovered one-dimensional source candidates.
    pub sources_1d: usize,
    /// Number of discovered two-dimensional source candidates.
    pub sources_2d: usize,
    /// Number of discovered source candidates whose dimension is unknown.
    pub sources_unknown: usize,
    /// Counts of discovered sources by reader format.
    pub source_formats: Vec<SourceFormatCount>,
    /// Counts of discovered sources by source vendor family.
    pub source_vendors: Vec<SourceVendorCount>,
    /// Counts of discovered sources by coarse source data kind.
    pub source_data_kinds: Vec<SourceDataKindCount>,
    /// Counts of discovered sources by tracked source path.
    #[serde(default)]
    pub source_paths: Vec<DiscoveredSpectrumPathCount>,
    /// Counts of discovered sources by inferred dimension.
    pub dimensions: Vec<DiscoveredSpectrumDimensionCount>,
}

impl DiscoveredSpectrumSummary {
    /// Creates discovery summary counts from source candidates.
    #[must_use]
    pub fn new(sources: &[DiscoveredSpectrumSource]) -> Self {
        let source_formats = discovered_source_format_counts(sources);
        let source_vendors = discovered_source_vendor_counts(sources);
        let source_data_kinds = discovered_source_data_kind_counts(sources);
        let source_paths = discovered_source_path_counts(sources);
        let dimensions = discovered_dimension_counts(sources);
        Self {
            sources: sources.len(),
            sources_1d: dimension_count_from_counts(&dimensions, DiscoveredSpectrumDimension::OneD),
            sources_2d: dimension_count_from_counts(&dimensions, DiscoveredSpectrumDimension::TwoD),
            sources_unknown: dimension_count_from_counts(
                &dimensions,
                DiscoveredSpectrumDimension::Unknown,
            ),
            source_formats,
            source_vendors,
            source_data_kinds,
            source_paths,
            dimensions,
        }
    }

    /// Returns the number of discovered source candidates.
    #[must_use]
    pub const fn sources(&self) -> usize {
        self.sources
    }

    /// Returns the number of discovered one-dimensional source candidates.
    #[must_use]
    pub const fn sources_1d(&self) -> usize {
        self.sources_1d
    }

    /// Returns the number of discovered two-dimensional source candidates.
    #[must_use]
    pub const fn sources_2d(&self) -> usize {
        self.sources_2d
    }

    /// Returns the number of discovered source candidates whose dimension is unknown.
    #[must_use]
    pub const fn sources_unknown(&self) -> usize {
        self.sources_unknown
    }

    /// Returns the number of discovered sources matching one generic source filter.
    #[must_use]
    pub fn source_count(&self, filter: impl Into<LoadedSourceFilter>) -> usize {
        match filter.into() {
            LoadedSourceFilter::Format { format } => self.source_format_count(format),
            LoadedSourceFilter::Vendor { vendor } => self.source_vendor_count(vendor),
            LoadedSourceFilter::DataKind { data_kind } => self.source_data_kind_count(data_kind),
            LoadedSourceFilter::Path { path } => self.source_path_count(path),
            LoadedSourceFilter::PathPrefix { path } => self.source_path_prefix_count(path),
        }
    }

    /// Returns true when a discovered source matches one generic source filter.
    #[must_use]
    pub fn has_source(&self, filter: impl Into<LoadedSourceFilter>) -> bool {
        self.source_count(filter) > 0
    }

    /// Returns the number of discovered sources read with a source format.
    #[must_use]
    pub fn source_format_count(&self, format: impl AsRef<str>) -> usize {
        let format = format.as_ref();
        self.source_formats
            .iter()
            .filter(|count| source_format_matches(count.format(), format))
            .map(SourceFormatCount::count)
            .sum()
    }

    /// Returns true when a discovered source would use a source format.
    #[must_use]
    pub fn has_source_format(&self, format: impl AsRef<str>) -> bool {
        self.source_format_count(format) > 0
    }

    /// Returns the number of discovered sources with a vendor-specific reader.
    #[must_use]
    pub fn source_vendor_count(&self, vendor: impl AsRef<str>) -> usize {
        let Ok(vendor) = LoadedSourceVendor::parse(vendor.as_ref()) else {
            return 0;
        };
        self.source_vendors
            .iter()
            .find(|count| count.vendor_kind() == Some(vendor))
            .map_or(0, SourceVendorCount::count)
    }

    /// Returns true when a discovered source would use a vendor-specific reader.
    #[must_use]
    pub fn has_source_vendor(&self, vendor: impl AsRef<str>) -> bool {
        self.source_vendor_count(vendor) > 0
    }

    /// Returns the number of discovered sources with one raw/processed data kind.
    #[must_use]
    pub fn source_data_kind_count(&self, data_kind: LoadedSourceDataKind) -> usize {
        self.source_data_kinds
            .iter()
            .find(|count| count.data_kind() == data_kind)
            .map_or(0, SourceDataKindCount::count)
    }

    /// Returns true when a discovered source has one raw/processed data kind.
    #[must_use]
    pub fn has_source_data_kind(&self, data_kind: LoadedSourceDataKind) -> bool {
        self.source_data_kind_count(data_kind) > 0
    }

    /// Returns the number of discovered sources with one tracked source path.
    #[must_use]
    pub fn source_path_count(&self, path: impl AsRef<Path>) -> usize {
        let path = path.as_ref();
        self.source_paths
            .iter()
            .find(|count| count.path() == path)
            .map_or(0, DiscoveredSpectrumPathCount::count)
    }

    /// Returns true when a discovered source has one tracked source path.
    #[must_use]
    pub fn has_source_path(&self, path: impl AsRef<Path>) -> bool {
        self.source_path_count(path) > 0
    }

    /// Returns the number of discovered sources whose tracked source path starts with a prefix.
    #[must_use]
    pub fn source_path_prefix_count(&self, path: impl AsRef<Path>) -> usize {
        let path = path.as_ref();
        self.source_paths
            .iter()
            .filter(|count| count.path().starts_with(path))
            .map(DiscoveredSpectrumPathCount::count)
            .sum()
    }

    /// Returns true when a discovered source path starts with a prefix.
    #[must_use]
    pub fn has_source_path_prefix(&self, path: impl AsRef<Path>) -> bool {
        self.source_path_prefix_count(path) > 0
    }

    /// Returns the number of discovered sources with one inferred dimension.
    #[must_use]
    pub fn dimension_count(&self, dimension: DiscoveredSpectrumDimension) -> usize {
        dimension_count_from_counts(&self.dimensions, dimension)
    }

    /// Returns true when a discovered source has one inferred dimension.
    #[must_use]
    pub fn has_dimension(&self, dimension: DiscoveredSpectrumDimension) -> bool {
        self.dimension_count(dimension) > 0
    }
}

/// Returns serializable summary counts for discovered source candidates.
#[must_use]
pub fn summarize_discovered_spectra(
    sources: &[DiscoveredSpectrumSource],
) -> DiscoveredSpectrumSummary {
    DiscoveredSpectrumSummary::new(sources)
}

fn discovered_source_format_counts(sources: &[DiscoveredSpectrumSource]) -> Vec<SourceFormatCount> {
    let mut counts: Vec<SourceFormatCount> = Vec::new();
    for source in sources {
        match counts
            .iter_mut()
            .find(|count| source_format_matches(count.format(), source.format()))
        {
            Some(count) => count.count += 1,
            None => counts.push(SourceFormatCount::new(source.format(), 1)),
        }
    }
    counts
}

fn discovered_source_vendor_counts(sources: &[DiscoveredSpectrumSource]) -> Vec<SourceVendorCount> {
    let mut counts: Vec<SourceVendorCount> = Vec::new();
    for vendor in sources.iter().filter_map(DiscoveredSpectrumSource::vendor) {
        match counts
            .iter_mut()
            .find(|count| count.vendor_kind() == Some(vendor))
        {
            Some(count) => count.count += 1,
            None => counts.push(SourceVendorCount::new(vendor.as_str(), 1)),
        }
    }
    counts
}

fn discovered_source_data_kind_counts(
    sources: &[DiscoveredSpectrumSource],
) -> Vec<SourceDataKindCount> {
    let mut counts: Vec<SourceDataKindCount> = Vec::new();
    for data_kind in sources.iter().map(DiscoveredSpectrumSource::data_kind) {
        match counts
            .iter_mut()
            .find(|count| count.data_kind() == data_kind)
        {
            Some(count) => count.count += 1,
            None => counts.push(SourceDataKindCount::new(data_kind, 1)),
        }
    }
    counts
}

fn discovered_source_path_counts(
    sources: &[DiscoveredSpectrumSource],
) -> Vec<DiscoveredSpectrumPathCount> {
    let mut counts: Vec<DiscoveredSpectrumPathCount> = Vec::new();
    for path in sources.iter().filter_map(DiscoveredSpectrumSource::path) {
        match counts.iter_mut().find(|count| count.path() == path) {
            Some(count) => count.count += 1,
            None => counts.push(DiscoveredSpectrumPathCount::new(path.to_path_buf(), 1)),
        }
    }
    counts
}

fn discovered_dimension_counts(
    sources: &[DiscoveredSpectrumSource],
) -> Vec<DiscoveredSpectrumDimensionCount> {
    let mut counts = Vec::new();
    for dimension in DISCOVERY_DIMENSIONS {
        let count = sources
            .iter()
            .filter(|source| source.dimension() == *dimension)
            .count();
        if count > 0 {
            counts.push(DiscoveredSpectrumDimensionCount::new(*dimension, count));
        }
    }
    counts
}

fn dimension_count_from_counts(
    counts: &[DiscoveredSpectrumDimensionCount],
    dimension: DiscoveredSpectrumDimension,
) -> usize {
    counts
        .iter()
        .find(|count| count.dimension() == dimension)
        .map_or(0, DiscoveredSpectrumDimensionCount::count)
}
