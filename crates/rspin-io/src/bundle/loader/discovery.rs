//! Source discovery for the spectrum bundle loader.

use std::path::{Path, PathBuf};

use rspin_core::{RSpinError, Result};
use serde::{Deserialize, Serialize};

use crate::{Spectrum1DPathFormat, Spectrum2DPathFormat};

use super::SpectrumBundleLoader;
use crate::bundle::{
    LoadWarning, LoadedSource, LoadedSourceDataKind, LoadedSourceFormat, LoadedSourceVendor,
    SourceDataKindCount, SourceFormatCount, SourceVendorCount, SpectrumBundle, collect_tree,
    file_candidate_kind, format_from_file, is_agilent_arrayed_1d_fid_path,
    is_agilent_arrayed_2d_fid_path, is_agilent_fid_dir, is_agilent_processed_dir,
    is_bruker_fid_dir, is_bruker_processed_1d_dir, is_bruker_processed_2d_dir, is_bruker_ser_dir,
    is_nmredata_file, is_standalone_spectrum_file, no_data_error_in_inputs,
    selected_path_from_base, source_format_matches,
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

const DISCOVERY_DIMENSIONS: &[DiscoveredSpectrumDimension] = &[
    DiscoveredSpectrumDimension::OneD,
    DiscoveredSpectrumDimension::TwoD,
    DiscoveredSpectrumDimension::Unknown,
];

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

/// Returns serializable summary counts for discovered source candidates.
#[must_use]
pub fn summarize_discovered_spectra(
    sources: &[DiscoveredSpectrumSource],
) -> DiscoveredSpectrumSummary {
    DiscoveredSpectrumSummary::new(sources)
}

/// Loads selected discovered source candidates relative to a common base directory.
///
/// This is a convenience wrapper for the common preflight workflow: discover
/// candidates, let the caller select some of them, then load only those exact
/// source path and format pairs.
///
/// # Errors
///
/// Returns an error when `base` is missing or not a directory, no discovered
/// sources are provided, a selected source does not include a tracked source
/// path, strict loading rejects a selected source, or no selected source can be
/// read.
pub fn load_discovered_spectra_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
) -> Result<SpectrumBundle> {
    SpectrumBundleLoader::new().read_discovered_relative_to(base, sources)
}

/// Loads selected discovered source candidates relative to a common base directory.
///
/// This short alias mirrors [`load_discovered_spectra_relative_to`].
///
/// # Errors
///
/// Returns an error when loading the selected discovered sources fails.
pub fn load_discovered_spectra<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
) -> Result<SpectrumBundle> {
    load_discovered_spectra_relative_to(base, sources)
}

impl SpectrumBundleLoader {
    /// Discovers source candidates below a file or directory without loading full spectra.
    ///
    /// Discovery respects source-format, vendor, data-kind, path, dimension,
    /// and raw/processed loader filters. Empty results are allowed so callers
    /// can use this as a lightweight preflight step.
    ///
    /// # Errors
    ///
    /// Returns an error when the input path is missing or a directory cannot be read.
    pub fn discover_path(&self, path: impl AsRef<Path>) -> Result<Vec<DiscoveredSpectrumSource>> {
        let root = path.as_ref();
        if !root.exists() {
            return Err(RSpinError::Parse {
                format: "spectrum source discovery",
                message: format!("{} does not exist", root.display()),
            });
        }

        let mut sources = Vec::new();
        self.discover_existing_path_into(root, root, &mut sources)?;
        Ok(sources)
    }

    /// Discovers source candidates below a file or directory.
    ///
    /// This is a short alias for [`Self::discover_path`].
    ///
    /// # Errors
    ///
    /// Returns an error when the input path is missing or a directory cannot be read.
    pub fn discover(&self, path: impl AsRef<Path>) -> Result<Vec<DiscoveredSpectrumSource>> {
        self.discover_path(path)
    }

    /// Discovers one selected path while anchoring source paths to a base directory.
    ///
    /// # Errors
    ///
    /// Returns an error when the base directory or selected path is missing.
    pub fn discover_path_relative_to(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
    ) -> Result<Vec<DiscoveredSpectrumSource>> {
        self.discover_paths_relative_to(base, [path])
    }

    /// Discovers one selected path while anchoring source paths to a base directory.
    ///
    /// This is a short alias for [`Self::discover_path_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when the base directory or selected path is missing.
    pub fn discover_relative_to(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
    ) -> Result<Vec<DiscoveredSpectrumSource>> {
        self.discover_path_relative_to(base, path)
    }

    /// Discovers source candidates below multiple file or directory paths.
    ///
    /// # Errors
    ///
    /// Returns an error when no paths are provided or an input path is missing.
    pub fn discover_paths<I, P>(&self, paths: I) -> Result<Vec<DiscoveredSpectrumSource>>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        let mut sources = Vec::new();
        let mut saw_input = false;

        for path in paths {
            saw_input = true;
            let path = path.as_ref();
            if !path.exists() {
                return Err(RSpinError::Parse {
                    format: "spectrum source discovery",
                    message: format!("{} does not exist", path.display()),
                });
            }
            self.discover_existing_path_into(path, path, &mut sources)?;
        }

        if !saw_input {
            return Err(no_discovery_inputs_error());
        }

        Ok(sources)
    }

    /// Discovers source candidates below multiple file or directory paths.
    ///
    /// This is a short alias for [`Self::discover_paths`].
    ///
    /// # Errors
    ///
    /// Returns an error when no paths are provided or an input path is missing.
    pub fn discover_many<I, P>(&self, paths: I) -> Result<Vec<DiscoveredSpectrumSource>>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.discover_paths(paths)
    }

    /// Discovers selected paths while anchoring source paths to a base directory.
    ///
    /// # Errors
    ///
    /// Returns an error when the base directory is missing, no selected paths
    /// are provided, or a selected path is missing.
    pub fn discover_paths_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
    ) -> Result<Vec<DiscoveredSpectrumSource>>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        let base = base.as_ref();
        if !base.exists() {
            return Err(RSpinError::Parse {
                format: "spectrum source discovery",
                message: format!("{} does not exist", base.display()),
            });
        }
        if !base.is_dir() {
            return Err(RSpinError::Parse {
                format: "spectrum source discovery",
                message: format!("{} is not a directory", base.display()),
            });
        }

        let mut sources = Vec::new();
        let mut saw_input = false;
        for path in paths {
            saw_input = true;
            let path = selected_path_from_base(base, path.as_ref());
            if !path.exists() {
                return Err(RSpinError::Parse {
                    format: "spectrum source discovery",
                    message: format!("{} does not exist", path.display()),
                });
            }
            self.discover_existing_path_into(base, &path, &mut sources)?;
        }

        if !saw_input {
            return Err(no_discovery_inputs_error());
        }

        Ok(sources)
    }

    /// Discovers selected paths while anchoring source paths to a base directory.
    ///
    /// This is a short alias for [`Self::discover_paths_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when the base directory is missing, no selected paths
    /// are provided, or a selected path is missing.
    pub fn discover_many_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
    ) -> Result<Vec<DiscoveredSpectrumSource>>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.discover_paths_relative_to(base, paths)
    }

    /// Loads selected discovered source candidates relative to a common base directory.
    ///
    /// Each discovered source must have source-path tracking enabled. The
    /// loader reads each selected source with both its discovered path and
    /// discovered source format as filters, preserving any other options already
    /// configured on this reader.
    ///
    /// # Errors
    ///
    /// Returns an error when `base` is missing or not a directory, no discovered
    /// sources are provided, a selected source does not include a tracked source
    /// path, strict loading rejects a selected source, or no selected source can
    /// be read.
    pub fn read_discovered_relative_to<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    ) -> Result<SpectrumBundle> {
        let base = base.as_ref();
        validate_discovered_base(base)?;

        let mut bundle = SpectrumBundle::new();
        let mut saw_source = false;

        for source in sources {
            saw_source = true;
            let source_path = source.path().ok_or_else(discovered_source_path_error)?;
            let selected_path = selected_path_from_base(base, source_path);
            let loader = self.loader_for_discovered_source(source_path, source);

            match loader.read_path_relative_to(base, source_path) {
                Ok(selected_bundle) => bundle.extend_bundle(selected_bundle),
                Err(error) if self.strict.is_enabled() => return Err(error),
                Err(error) => bundle.push_warning(LoadWarning::new(
                    self.source_path_for_metadata(base, &selected_path),
                    error.to_string(),
                )),
            }
        }

        if !saw_source {
            return Err(no_discovered_sources_error());
        }

        if bundle.has_data() {
            Ok(bundle)
        } else {
            Err(no_data_error_in_inputs(&bundle))
        }
    }

    /// Loads selected discovered source candidates relative to a common base directory.
    ///
    /// This short alias mirrors [`Self::read_discovered_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading the selected discovered sources fails.
    pub fn read_discovered<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    ) -> Result<SpectrumBundle> {
        self.read_discovered_relative_to(base, sources)
    }

    fn discover_existing_path_into(
        &self,
        source_root: &Path,
        path: &Path,
        sources: &mut Vec<DiscoveredSpectrumSource>,
    ) -> Result<()> {
        if path.is_dir() {
            self.discover_directory(source_root, path, sources)
        } else {
            self.discover_file_candidate(source_root, path, sources);
            Ok(())
        }
    }

    fn discover_directory(
        &self,
        source_root: &Path,
        directory: &Path,
        sources: &mut Vec<DiscoveredSpectrumSource>,
    ) -> Result<()> {
        let tree = collect_tree(directory)?;
        for directory in &tree.directories {
            self.discover_directory_candidate(source_root, directory, sources);
        }
        for file in &tree.files {
            if is_nmredata_file(file) {
                continue;
            }
            if is_standalone_spectrum_file(file) {
                self.discover_file_candidate(source_root, file, sources);
            }
        }
        Ok(())
    }

    fn discover_directory_candidate(
        &self,
        root: &Path,
        directory: &Path,
        sources: &mut Vec<DiscoveredSpectrumSource>,
    ) {
        if is_bruker_ser_dir(directory) {
            self.push_discovered_source_if_allowed(
                sources,
                root,
                directory,
                "bruker_ser",
                DiscoveredSpectrumDimension::TwoD,
            );
        }
        if is_bruker_fid_dir(directory) {
            self.push_discovered_source_if_allowed(
                sources,
                root,
                directory,
                "bruker_fid",
                DiscoveredSpectrumDimension::OneD,
            );
        }
        if is_bruker_processed_2d_dir(directory) {
            self.push_discovered_source_if_allowed(
                sources,
                root,
                directory,
                "bruker_processed",
                DiscoveredSpectrumDimension::TwoD,
            );
        }
        if is_bruker_processed_1d_dir(directory) {
            self.push_discovered_source_if_allowed(
                sources,
                root,
                directory,
                "bruker_processed",
                DiscoveredSpectrumDimension::OneD,
            );
        }
        if is_agilent_fid_dir(directory) {
            self.discover_agilent_fid_path(root, directory, sources);
        }
        if is_agilent_processed_dir(directory) {
            self.discover_detected_path(root, directory, Some("agilent_processed"), sources);
        }
    }

    fn discover_file_candidate(
        &self,
        root: &Path,
        file: &Path,
        sources: &mut Vec<DiscoveredSpectrumSource>,
    ) {
        if is_nmredata_file(file) || !self.allows_file_candidate_kind(file_candidate_kind(file)) {
            return;
        }

        let format = format_from_file(file);
        if format == "auto" {
            self.discover_detected_path(root, file, None, sources);
        } else {
            self.discover_detected_path(root, file, Some(format), sources);
        }
    }

    fn discover_agilent_fid_path(
        &self,
        root: &Path,
        path: &Path,
        sources: &mut Vec<DiscoveredSpectrumSource>,
    ) {
        if is_agilent_arrayed_2d_fid_path(path) {
            self.push_discovered_source_if_allowed(
                sources,
                root,
                path,
                "agilent_fid",
                DiscoveredSpectrumDimension::TwoD,
            );
            return;
        }

        if is_agilent_arrayed_1d_fid_path(path) {
            self.push_discovered_source_if_allowed(
                sources,
                root,
                path,
                "agilent_fid",
                DiscoveredSpectrumDimension::OneD,
            );
            return;
        }

        self.discover_detected_path(root, path, Some("agilent_fid"), sources);
    }

    fn discover_detected_path(
        &self,
        root: &Path,
        path: &Path,
        fallback_format: Option<&'static str>,
        sources: &mut Vec<DiscoveredSpectrumSource>,
    ) {
        let mut pushed = false;

        if let Ok(format) = crate::detect_spectrum1d_path_format(path) {
            pushed = self.push_discovered_source_if_allowed(
                sources,
                root,
                path,
                path_format_1d(format, fallback_format),
                DiscoveredSpectrumDimension::OneD,
            ) || pushed;
        }

        if let Ok(format) = crate::detect_spectrum2d_path_format(path) {
            pushed = self.push_discovered_source_if_allowed(
                sources,
                root,
                path,
                path_format_2d(format, fallback_format),
                DiscoveredSpectrumDimension::TwoD,
            ) || pushed;
        }

        if !pushed && let Some(format) = fallback_format {
            self.push_discovered_source_if_allowed(
                sources,
                root,
                path,
                format,
                DiscoveredSpectrumDimension::Unknown,
            );
        }
    }

    fn push_discovered_source_if_allowed(
        &self,
        sources: &mut Vec<DiscoveredSpectrumSource>,
        root: &Path,
        path: &Path,
        format: &'static str,
        dimension: DiscoveredSpectrumDimension,
    ) -> bool {
        if !self.allows_discovered_dimension(dimension) || !self.allows_source(root, path, format) {
            return false;
        }

        sources.push(DiscoveredSpectrumSource::new(
            self.source_path_for_metadata(root, path),
            format,
            dimension,
        ));
        true
    }

    fn allows_discovered_dimension(&self, dimension: DiscoveredSpectrumDimension) -> bool {
        match dimension {
            DiscoveredSpectrumDimension::OneD => self.one_d.is_enabled(),
            DiscoveredSpectrumDimension::TwoD => self.two_d.is_enabled(),
            DiscoveredSpectrumDimension::Unknown => {
                self.one_d.is_enabled() || self.two_d.is_enabled()
            }
        }
    }

    fn loader_for_discovered_source(
        &self,
        source_path: &Path,
        source: &DiscoveredSpectrumSource,
    ) -> Self {
        let loader = self
            .clone()
            .source_path(source_path)
            .source_format(source.format());
        match source.dimension() {
            DiscoveredSpectrumDimension::OneD => loader.with_2d(false),
            DiscoveredSpectrumDimension::TwoD => loader.with_1d(false),
            DiscoveredSpectrumDimension::Unknown => loader,
        }
    }
}

fn validate_discovered_base(base: &Path) -> Result<()> {
    if !base.exists() {
        return Err(RSpinError::Parse {
            format: "spectrum bundle",
            message: format!("{} does not exist", base.display()),
        });
    }
    if !base.is_dir() {
        return Err(RSpinError::Parse {
            format: "spectrum bundle",
            message: format!("{} is not a directory", base.display()),
        });
    }
    Ok(())
}

fn discovered_source_path_error() -> RSpinError {
    RSpinError::Parse {
        format: "spectrum bundle",
        message:
            "discovered source is missing a tracked source path; discover with source paths enabled"
                .to_owned(),
    }
}

fn no_discovered_sources_error() -> RSpinError {
    RSpinError::Parse {
        format: "spectrum bundle",
        message: "no discovered sources provided".to_owned(),
    }
}

fn path_format_1d(
    detected: Spectrum1DPathFormat,
    fallback_format: Option<&'static str>,
) -> &'static str {
    if let Some(format) = fallback_format {
        format
    } else {
        detected.as_str()
    }
}

fn path_format_2d(
    detected: Spectrum2DPathFormat,
    fallback_format: Option<&'static str>,
) -> &'static str {
    if let Some(format) = fallback_format {
        format
    } else {
        detected.as_str()
    }
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

fn no_discovery_inputs_error() -> RSpinError {
    RSpinError::Parse {
        format: "spectrum source discovery",
        message: "no input paths provided".to_owned(),
    }
}
