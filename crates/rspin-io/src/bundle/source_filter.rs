//! Generic source filters for unified bundle loading.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::{
    LoadedSource, LoadedSourceFormat, LoadedSourceVendor, canonical_source_format_filter,
    source_format_matches,
};

/// Source restriction for the unified bundle loader.
///
/// Use this when caller UI or application state can select a format, a vendor,
/// or a tracked source path through one API surface. Specific helpers such as
/// `only_source_format` and `only_source_vendor` remain available when the
/// filter kind is known statically.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LoadedSourceFilter {
    /// Restricts loading to a source format.
    Format {
        /// Canonical or custom source-format name.
        format: String,
    },
    /// Restricts loading to a vendor family.
    Vendor {
        /// Vendor name or alias.
        vendor: String,
    },
    /// Restricts loading to a tracked source path.
    Path {
        /// Source path relative to the loader root.
        path: PathBuf,
    },
}

impl LoadedSourceFilter {
    /// Creates a source-format filter.
    #[must_use]
    pub fn format(format: impl AsRef<str>) -> Self {
        Self::Format {
            format: canonical_source_format_filter(format.as_ref()),
        }
    }

    /// Creates a vendor-family filter.
    #[must_use]
    pub fn vendor(vendor: impl AsRef<str>) -> Self {
        Self::Vendor {
            vendor: vendor.as_ref().trim().to_owned(),
        }
    }

    /// Creates a tracked source-path filter.
    #[must_use]
    pub fn path(path: impl AsRef<Path>) -> Self {
        Self::Path {
            path: path.as_ref().to_path_buf(),
        }
    }

    /// Returns true when the source matches this filter.
    #[must_use]
    pub fn matches_source(&self, source: &LoadedSource) -> bool {
        match self {
            Self::Format { format } => source.is_format(format),
            Self::Vendor { vendor } => source.is_vendor(vendor),
            Self::Path { path } => source.path() == Some(path.as_path()),
        }
    }

    /// Returns true when this filter can match a source with the given format.
    ///
    /// Path filters return true because the path must be evaluated with the
    /// full source context.
    #[must_use]
    pub fn may_match_format(&self, format: impl AsRef<str>) -> bool {
        match self {
            Self::Format { format: allowed } => source_format_matches(format.as_ref(), allowed),
            Self::Vendor { vendor } => {
                let Ok(vendor) = LoadedSourceVendor::parse(vendor) else {
                    return false;
                };
                vendor
                    .source_formats()
                    .iter()
                    .any(|allowed| source_format_matches(format.as_ref(), allowed.as_str()))
            }
            Self::Path { .. } => true,
        }
    }

    /// Returns true when this filter can match a source with the given tracked path.
    ///
    /// Format and vendor filters return true because the source format must be
    /// evaluated with the full source context.
    #[must_use]
    pub fn may_match_path(&self, path: impl AsRef<Path>) -> bool {
        match self {
            Self::Path { path: allowed } => allowed == path.as_ref(),
            Self::Format { .. } | Self::Vendor { .. } => true,
        }
    }
}

impl From<LoadedSourceFormat> for LoadedSourceFilter {
    fn from(format: LoadedSourceFormat) -> Self {
        Self::format(format)
    }
}

impl From<LoadedSourceVendor> for LoadedSourceFilter {
    fn from(vendor: LoadedSourceVendor) -> Self {
        Self::vendor(vendor)
    }
}

impl From<&LoadedSourceFilter> for LoadedSourceFilter {
    fn from(filter: &LoadedSourceFilter) -> Self {
        filter.clone()
    }
}

pub(super) fn source_filters<I, F>(filters: I) -> Vec<LoadedSourceFilter>
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
