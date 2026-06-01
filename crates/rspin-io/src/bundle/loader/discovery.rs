//! Source discovery for the spectrum bundle loader.

use std::path::{Path, PathBuf};

use rspin_core::{RSpinError, Result};
use serde::{Deserialize, Serialize};

use crate::{Spectrum1DPathFormat, Spectrum2DPathFormat};

use super::SpectrumBundleLoader;
use crate::bundle::{
    LoadedSource, LoadedSourceDataKind, LoadedSourceFormat, LoadedSourceVendor, collect_tree,
    file_candidate_kind, format_from_file, is_agilent_arrayed_1d_fid_path,
    is_agilent_arrayed_2d_fid_path, is_agilent_fid_dir, is_agilent_processed_dir,
    is_bruker_fid_dir, is_bruker_processed_1d_dir, is_bruker_processed_2d_dir, is_bruker_ser_dir,
    is_nmredata_file, is_standalone_spectrum_file, selected_path_from_base,
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

fn no_discovery_inputs_error() -> RSpinError {
    RSpinError::Parse {
        format: "spectrum source discovery",
        message: "no input paths provided".to_owned(),
    }
}
