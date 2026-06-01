//! Source-path prefix helpers for loaded spectrum bundles.

use std::path::Path;

use rspin_core::{Result, Spectrum1D, Spectrum2D};

use super::{
    LoadWarning, LoadedSource, LoadedSourceFilter, LoadedSpectrum, SpectrumBundle,
    SpectrumBundleLoader,
};

/// Loads supported spectra from a file or directory, restricted to tracked source paths below a prefix.
///
/// The source path prefix is matched against relative paths recorded in the
/// loaded bundle, such as `bruker/pdata`.
///
/// # Errors
///
/// Returns an error when the path is missing or no matching readable bundle
/// data is found.
pub fn load_spectra_by_source_path_prefix(
    path: impl AsRef<Path>,
    source_path_prefix: impl AsRef<Path>,
) -> Result<SpectrumBundle> {
    SpectrumBundleLoader::new()
        .only_source_path_prefix(source_path_prefix)
        .read_path(path)
}

/// Loads one selected path relative to a base directory, restricted to tracked source paths below a prefix.
///
/// Relative input paths are resolved below `base`; absolute input paths are
/// loaded as provided. The source path prefix is matched after anchoring source
/// metadata to `base`.
///
/// # Errors
///
/// Returns an error when `base` is missing or is not a directory, the path is
/// unreadable in strict mode, or no matching readable bundle data is found.
pub fn load_spectra_by_source_path_prefix_relative_to(
    base: impl AsRef<Path>,
    path: impl AsRef<Path>,
    source_path_prefix: impl AsRef<Path>,
) -> Result<SpectrumBundle> {
    SpectrumBundleLoader::new()
        .only_source_path_prefix(source_path_prefix)
        .read_path_relative_to(base, path)
}

/// Loads supported spectra from multiple paths, restricted to tracked source paths below a prefix.
///
/// # Errors
///
/// Returns an error when no input paths are provided or no matching readable
/// bundle data is found.
pub fn load_spectra_many_by_source_path_prefix<I, P>(
    paths: I,
    source_path_prefix: impl AsRef<Path>,
) -> Result<SpectrumBundle>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new()
        .only_source_path_prefix(source_path_prefix)
        .read_paths(paths)
}

/// Loads selected paths relative to a base directory, restricted to tracked source paths below a prefix.
///
/// Relative input paths are resolved below `base`; absolute input paths are
/// loaded as provided. The source path prefix is matched after anchoring source
/// metadata to `base`.
///
/// # Errors
///
/// Returns an error when `base` is missing or is not a directory, no input
/// paths are provided, or no matching readable bundle data is found.
pub fn load_spectra_many_by_source_path_prefix_relative_to<I, P>(
    base: impl AsRef<Path>,
    paths: I,
    source_path_prefix: impl AsRef<Path>,
) -> Result<SpectrumBundle>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new()
        .only_source_path_prefix(source_path_prefix)
        .read_paths_relative_to(base, paths)
}

impl SpectrumBundle {
    /// Returns a cloned bundle containing spectra from tracked source paths below a prefix.
    ///
    /// Molecule metadata is preserved. Loader warnings are retained when their
    /// tracked paths start with the same prefix.
    #[must_use]
    pub fn source_path_prefix_subset(&self, path: impl AsRef<Path>) -> Self {
        self.source_subset(LoadedSourceFilter::path_prefix(path))
    }

    /// Returns loaded spectra from tracked source paths below a prefix.
    pub fn loaded_by_source_path_prefix(
        &self,
        path: impl AsRef<Path>,
    ) -> impl Iterator<Item = &LoadedSpectrum> + '_ {
        let path = path.as_ref().to_path_buf();
        self.spectra()
            .iter()
            .filter(move |entry| source_path_matches_prefix(entry.source(), &path))
    }

    /// Returns one-dimensional spectra and sources from tracked source paths below a prefix.
    pub fn loaded_1d_by_source_path_prefix(
        &self,
        path: impl AsRef<Path>,
    ) -> impl Iterator<Item = (&Spectrum1D, &LoadedSource)> + '_ {
        let path = path.as_ref().to_path_buf();
        self.loaded_1d()
            .filter(move |(_, source)| source_path_matches_prefix(source, &path))
    }

    /// Returns two-dimensional spectra and sources from tracked source paths below a prefix.
    pub fn loaded_2d_by_source_path_prefix(
        &self,
        path: impl AsRef<Path>,
    ) -> impl Iterator<Item = (&Spectrum2D, &LoadedSource)> + '_ {
        let path = path.as_ref().to_path_buf();
        self.loaded_2d()
            .filter(move |(_, source)| source_path_matches_prefix(source, &path))
    }

    /// Returns tracked source paths below a prefix.
    pub fn source_paths_for_path_prefix(
        &self,
        path: impl AsRef<Path>,
    ) -> impl Iterator<Item = &Path> + '_ {
        let path = path.as_ref().to_path_buf();
        self.loaded_sources()
            .filter(move |source| source_path_matches_prefix(source, &path))
            .filter_map(LoadedSource::path)
    }

    /// Returns warnings associated with tracked source paths below a prefix.
    pub fn warnings_for_source_path_prefix(
        &self,
        path: impl AsRef<Path>,
    ) -> impl Iterator<Item = &LoadWarning> + '_ {
        let path = path.as_ref().to_path_buf();
        self.warnings()
            .iter()
            .filter(move |warning| warning_path_matches_prefix(warning, &path))
    }

    /// Returns the number of loaded spectra from tracked source paths below a prefix.
    #[must_use]
    pub fn source_path_prefix_count(&self, path: impl AsRef<Path>) -> usize {
        self.loaded_by_source_path_prefix(path).count()
    }

    /// Returns true when at least one loaded spectrum has a tracked source path below a prefix.
    #[must_use]
    pub fn has_source_path_prefix(&self, path: impl AsRef<Path>) -> bool {
        self.loaded_by_source_path_prefix(path).next().is_some()
    }

    /// Consumes the bundle and keeps spectra from tracked source paths below a prefix.
    ///
    /// Molecule metadata is preserved. Loader warnings are retained when their
    /// tracked paths start with the same prefix.
    #[must_use]
    pub fn into_source_path_prefix_subset(self, path: impl AsRef<Path>) -> Self {
        self.into_source_subset(LoadedSourceFilter::path_prefix(path))
    }

    /// Consumes the bundle and returns loaded spectra from tracked source paths below a prefix.
    #[must_use]
    pub fn into_loaded_by_source_path_prefix(self, path: impl AsRef<Path>) -> Vec<LoadedSpectrum> {
        self.into_loaded_by_source(LoadedSourceFilter::path_prefix(path))
    }

    /// Consumes the bundle and returns one-dimensional spectra and sources from tracked source paths below a prefix.
    #[must_use]
    pub fn into_loaded_1d_by_source_path_prefix(
        self,
        path: impl AsRef<Path>,
    ) -> Vec<(Spectrum1D, LoadedSource)> {
        self.into_loaded_1d_by_source(LoadedSourceFilter::path_prefix(path))
    }

    /// Consumes the bundle and returns two-dimensional spectra and sources from tracked source paths below a prefix.
    #[must_use]
    pub fn into_loaded_2d_by_source_path_prefix(
        self,
        path: impl AsRef<Path>,
    ) -> Vec<(Spectrum2D, LoadedSource)> {
        self.into_loaded_2d_by_source(LoadedSourceFilter::path_prefix(path))
    }

    /// Consumes the bundle and returns one-dimensional spectra from tracked source paths below a prefix.
    #[must_use]
    pub fn into_spectra_1d_by_source_path_prefix(self, path: impl AsRef<Path>) -> Vec<Spectrum1D> {
        self.into_spectra_1d_by_source(LoadedSourceFilter::path_prefix(path))
    }

    /// Consumes the bundle and returns two-dimensional spectra from tracked source paths below a prefix.
    #[must_use]
    pub fn into_spectra_2d_by_source_path_prefix(self, path: impl AsRef<Path>) -> Vec<Spectrum2D> {
        self.into_spectra_2d_by_source(LoadedSourceFilter::path_prefix(path))
    }
}

fn source_path_matches_prefix(source: &LoadedSource, path: &Path) -> bool {
    source
        .path()
        .is_some_and(|source_path| source_path.starts_with(path))
}

fn warning_path_matches_prefix(warning: &LoadWarning, path: &Path) -> bool {
    warning
        .path()
        .is_some_and(|warning_path| warning_path.starts_with(path))
}
