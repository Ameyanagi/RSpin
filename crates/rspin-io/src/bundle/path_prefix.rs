//! Source-path prefix helpers for loaded spectrum bundles.

use std::path::{Path, PathBuf};

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

/// Loads supported spectra from a file or directory, restricted to tracked source paths below any prefix.
///
/// The source path prefixes are matched against relative paths recorded in the
/// loaded bundle. Prefixes are combined with logical OR. Passing an empty
/// iterator leaves source loading unrestricted.
///
/// # Errors
///
/// Returns an error when the path is missing or no matching readable bundle
/// data is found.
pub fn load_spectra_by_source_path_prefixes<I, P>(
    path: impl AsRef<Path>,
    source_path_prefixes: I,
) -> Result<SpectrumBundle>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new()
        .only_source_path_prefixes(source_path_prefixes)
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

/// Loads one selected path relative to a base directory, restricted to tracked source paths below any prefix.
///
/// Relative input paths are resolved below `base`; absolute input paths are
/// loaded as provided. Prefixes are matched after anchoring source metadata to
/// `base` and are combined with logical OR.
///
/// # Errors
///
/// Returns an error when `base` is missing or is not a directory, the path is
/// unreadable in strict mode, or no matching readable bundle data is found.
pub fn load_spectra_by_source_path_prefixes_relative_to<I, P>(
    base: impl AsRef<Path>,
    path: impl AsRef<Path>,
    source_path_prefixes: I,
) -> Result<SpectrumBundle>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new()
        .only_source_path_prefixes(source_path_prefixes)
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

/// Loads supported spectra from multiple paths, restricted to tracked source paths below any prefix.
///
/// Prefixes are combined with logical OR. Passing an empty iterator leaves
/// source loading unrestricted.
///
/// # Errors
///
/// Returns an error when no input paths are provided or no matching readable
/// bundle data is found.
pub fn load_spectra_many_by_source_path_prefixes<I, P, J, F>(
    paths: I,
    source_path_prefixes: J,
) -> Result<SpectrumBundle>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
    J: IntoIterator<Item = F>,
    F: AsRef<Path>,
{
    SpectrumBundleLoader::new()
        .only_source_path_prefixes(source_path_prefixes)
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

/// Loads selected paths relative to a base directory, restricted to tracked source paths below any prefix.
///
/// Relative input paths are resolved below `base`; absolute input paths are
/// loaded as provided. Prefixes are matched after anchoring source metadata to
/// `base` and are combined with logical OR.
///
/// # Errors
///
/// Returns an error when `base` is missing or is not a directory, no input
/// paths are provided, or no matching readable bundle data is found.
pub fn load_spectra_many_by_source_path_prefixes_relative_to<I, P, J, F>(
    base: impl AsRef<Path>,
    paths: I,
    source_path_prefixes: J,
) -> Result<SpectrumBundle>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
    J: IntoIterator<Item = F>,
    F: AsRef<Path>,
{
    SpectrumBundleLoader::new()
        .only_source_path_prefixes(source_path_prefixes)
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

    /// Returns a cloned bundle containing spectra from tracked source paths below any prefix.
    ///
    /// Prefixes are combined with logical OR. Passing an empty iterator returns
    /// a full clone. Molecule metadata is preserved. Loader warnings are
    /// retained when their tracked paths start with any prefix.
    #[must_use]
    pub fn source_path_prefix_subset_by_prefixes<I, P>(&self, paths: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.source_subset_by_sources(path_prefix_filters(paths))
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

    /// Returns loaded spectra from tracked source paths below any prefix.
    ///
    /// Prefixes are combined with logical OR. Passing an empty iterator leaves
    /// source matching unrestricted.
    pub fn loaded_by_source_path_prefixes<I, P>(
        &self,
        paths: I,
    ) -> impl Iterator<Item = &LoadedSpectrum> + '_
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.loaded_by_sources(path_prefix_filters(paths))
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

    /// Returns one-dimensional spectra and sources from tracked source paths below any prefix.
    ///
    /// Prefixes are combined with logical OR. Passing an empty iterator leaves
    /// source matching unrestricted.
    pub fn loaded_1d_by_source_path_prefixes<I, P>(
        &self,
        paths: I,
    ) -> impl Iterator<Item = (&Spectrum1D, &LoadedSource)> + '_
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.loaded_1d_by_sources(path_prefix_filters(paths))
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

    /// Returns two-dimensional spectra and sources from tracked source paths below any prefix.
    ///
    /// Prefixes are combined with logical OR. Passing an empty iterator leaves
    /// source matching unrestricted.
    pub fn loaded_2d_by_source_path_prefixes<I, P>(
        &self,
        paths: I,
    ) -> impl Iterator<Item = (&Spectrum2D, &LoadedSource)> + '_
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.loaded_2d_by_sources(path_prefix_filters(paths))
    }

    /// Returns the only one-dimensional spectrum from tracked source paths below a prefix.
    ///
    /// Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching one-dimensional spectrum
    /// exists.
    pub fn only_1d_by_source_path_prefix(&self, path: impl AsRef<Path>) -> Result<&Spectrum1D> {
        self.only_loaded_1d_by_source_path_prefix(path)
            .map(|(spectrum, _)| spectrum)
    }

    /// Returns the only one-dimensional spectrum and source from tracked source paths below a prefix.
    ///
    /// Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching one-dimensional spectrum
    /// exists.
    pub fn only_loaded_1d_by_source_path_prefix(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<(&Spectrum1D, &LoadedSource)> {
        self.only_loaded_1d_by_source(LoadedSourceFilter::path_prefix(path))
    }

    /// Returns the only two-dimensional spectrum from tracked source paths below a prefix.
    ///
    /// Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching two-dimensional spectrum
    /// exists.
    pub fn only_2d_by_source_path_prefix(&self, path: impl AsRef<Path>) -> Result<&Spectrum2D> {
        self.only_loaded_2d_by_source_path_prefix(path)
            .map(|(spectrum, _)| spectrum)
    }

    /// Returns the only two-dimensional spectrum and source from tracked source paths below a prefix.
    ///
    /// Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching two-dimensional spectrum
    /// exists.
    pub fn only_loaded_2d_by_source_path_prefix(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<(&Spectrum2D, &LoadedSource)> {
        self.only_loaded_2d_by_source(LoadedSourceFilter::path_prefix(path))
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

    /// Returns tracked source paths below any prefix.
    ///
    /// Prefixes are combined with logical OR. Passing an empty iterator returns
    /// all tracked source paths.
    pub fn source_paths_for_path_prefixes<I, P>(&self, paths: I) -> impl Iterator<Item = &Path> + '_
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.source_paths_for_sources(path_prefix_filters(paths))
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

    /// Returns warnings associated with tracked source paths below any prefix.
    ///
    /// Prefixes are combined with logical OR. Passing an empty iterator returns
    /// all warnings.
    pub fn warnings_for_source_path_prefixes<I, P>(
        &self,
        paths: I,
    ) -> impl Iterator<Item = &LoadWarning> + '_
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        let paths = path_prefixes(paths);
        self.warnings()
            .iter()
            .filter(move |warning| warning_matches_any_prefix(warning, &paths))
    }

    /// Returns the number of non-fatal loader warnings from tracked source paths below a prefix.
    #[must_use]
    pub fn warning_path_prefix_count(&self, path: impl AsRef<Path>) -> usize {
        self.warnings_for_source_path_prefix(path).count()
    }

    /// Returns the number of non-fatal loader warnings from tracked source paths below any prefix.
    ///
    /// Passing an empty iterator counts all warnings.
    #[must_use]
    pub fn warning_path_prefix_count_by_prefixes<I, P>(&self, paths: I) -> usize
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.warnings_for_source_path_prefixes(paths).count()
    }

    /// Returns true when at least one non-fatal loader warning has a tracked source path below a prefix.
    #[must_use]
    pub fn has_warning_path_prefix(&self, path: impl AsRef<Path>) -> bool {
        self.warnings_for_source_path_prefix(path).next().is_some()
    }

    /// Returns true when at least one non-fatal loader warning has a tracked source path below any prefix.
    ///
    /// Passing an empty iterator returns true when the bundle contains any
    /// warning.
    #[must_use]
    pub fn has_any_warning_path_prefix<I, P>(&self, paths: I) -> bool
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.warnings_for_source_path_prefixes(paths)
            .next()
            .is_some()
    }

    /// Returns the number of loaded spectra from tracked source paths below a prefix.
    #[must_use]
    pub fn source_path_prefix_count(&self, path: impl AsRef<Path>) -> usize {
        self.loaded_by_source_path_prefix(path).count()
    }

    /// Returns the number of loaded spectra from tracked source paths below any prefix.
    ///
    /// Passing an empty iterator counts all loaded spectra.
    #[must_use]
    pub fn source_path_prefix_count_by_prefixes<I, P>(&self, paths: I) -> usize
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.loaded_by_source_path_prefixes(paths).count()
    }

    /// Returns true when at least one loaded spectrum has a tracked source path below a prefix.
    #[must_use]
    pub fn has_source_path_prefix(&self, path: impl AsRef<Path>) -> bool {
        self.loaded_by_source_path_prefix(path).next().is_some()
    }

    /// Returns true when at least one loaded spectrum has a tracked source path below any prefix.
    ///
    /// Passing an empty iterator returns true when the bundle contains any
    /// loaded spectrum.
    #[must_use]
    pub fn has_any_source_path_prefix<I, P>(&self, paths: I) -> bool
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.loaded_by_source_path_prefixes(paths).next().is_some()
    }

    /// Consumes the bundle and keeps spectra from tracked source paths below a prefix.
    ///
    /// Molecule metadata is preserved. Loader warnings are retained when their
    /// tracked paths start with the same prefix.
    #[must_use]
    pub fn into_source_path_prefix_subset(self, path: impl AsRef<Path>) -> Self {
        self.into_source_subset(LoadedSourceFilter::path_prefix(path))
    }

    /// Consumes the bundle and keeps spectra from tracked source paths below any prefix.
    ///
    /// Prefixes are combined with logical OR. Passing an empty iterator leaves
    /// the bundle unchanged. Molecule metadata is preserved. Loader warnings
    /// are retained when their tracked paths start with any prefix.
    #[must_use]
    pub fn into_source_path_prefix_subset_by_prefixes<I, P>(self, paths: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.into_source_subset_by_sources(path_prefix_filters(paths))
    }

    /// Consumes the bundle and returns loaded spectra from tracked source paths below a prefix.
    #[must_use]
    pub fn into_loaded_by_source_path_prefix(self, path: impl AsRef<Path>) -> Vec<LoadedSpectrum> {
        self.into_loaded_by_source(LoadedSourceFilter::path_prefix(path))
    }

    /// Consumes the bundle and returns loaded spectra from tracked source paths below any prefix.
    ///
    /// Prefixes are combined with logical OR. Passing an empty iterator leaves
    /// source matching unrestricted.
    #[must_use]
    pub fn into_loaded_by_source_path_prefixes<I, P>(self, paths: I) -> Vec<LoadedSpectrum>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.into_loaded_by_sources(path_prefix_filters(paths))
    }

    /// Consumes the bundle and returns one-dimensional spectra and sources from tracked source paths below a prefix.
    #[must_use]
    pub fn into_loaded_1d_by_source_path_prefix(
        self,
        path: impl AsRef<Path>,
    ) -> Vec<(Spectrum1D, LoadedSource)> {
        self.into_loaded_1d_by_source(LoadedSourceFilter::path_prefix(path))
    }

    /// Consumes the bundle and returns one-dimensional spectra and sources from tracked source paths below any prefix.
    ///
    /// Prefixes are combined with logical OR. Passing an empty iterator leaves
    /// source matching unrestricted.
    #[must_use]
    pub fn into_loaded_1d_by_source_path_prefixes<I, P>(
        self,
        paths: I,
    ) -> Vec<(Spectrum1D, LoadedSource)>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.into_loaded_1d_by_sources(path_prefix_filters(paths))
    }

    /// Consumes the bundle and returns two-dimensional spectra and sources from tracked source paths below a prefix.
    #[must_use]
    pub fn into_loaded_2d_by_source_path_prefix(
        self,
        path: impl AsRef<Path>,
    ) -> Vec<(Spectrum2D, LoadedSource)> {
        self.into_loaded_2d_by_source(LoadedSourceFilter::path_prefix(path))
    }

    /// Consumes the bundle and returns two-dimensional spectra and sources from tracked source paths below any prefix.
    ///
    /// Prefixes are combined with logical OR. Passing an empty iterator leaves
    /// source matching unrestricted.
    #[must_use]
    pub fn into_loaded_2d_by_source_path_prefixes<I, P>(
        self,
        paths: I,
    ) -> Vec<(Spectrum2D, LoadedSource)>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.into_loaded_2d_by_sources(path_prefix_filters(paths))
    }

    /// Consumes the bundle and returns the only one-dimensional spectrum from tracked source paths below a prefix.
    ///
    /// Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching one-dimensional spectrum
    /// exists.
    pub fn into_only_1d_by_source_path_prefix(self, path: impl AsRef<Path>) -> Result<Spectrum1D> {
        self.into_only_loaded_1d_by_source_path_prefix(path)
            .map(|(spectrum, _)| spectrum)
    }

    /// Consumes the bundle and returns the only one-dimensional spectrum and source from tracked source paths below a prefix.
    ///
    /// Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching one-dimensional spectrum
    /// exists.
    pub fn into_only_loaded_1d_by_source_path_prefix(
        self,
        path: impl AsRef<Path>,
    ) -> Result<(Spectrum1D, LoadedSource)> {
        self.into_only_loaded_1d_by_source(LoadedSourceFilter::path_prefix(path))
    }

    /// Consumes the bundle and returns the only two-dimensional spectrum from tracked source paths below a prefix.
    ///
    /// Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching two-dimensional spectrum
    /// exists.
    pub fn into_only_2d_by_source_path_prefix(self, path: impl AsRef<Path>) -> Result<Spectrum2D> {
        self.into_only_loaded_2d_by_source_path_prefix(path)
            .map(|(spectrum, _)| spectrum)
    }

    /// Consumes the bundle and returns the only two-dimensional spectrum and source from tracked source paths below a prefix.
    ///
    /// Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching two-dimensional spectrum
    /// exists.
    pub fn into_only_loaded_2d_by_source_path_prefix(
        self,
        path: impl AsRef<Path>,
    ) -> Result<(Spectrum2D, LoadedSource)> {
        self.into_only_loaded_2d_by_source(LoadedSourceFilter::path_prefix(path))
    }

    /// Consumes the bundle and returns one-dimensional spectra from tracked source paths below a prefix.
    #[must_use]
    pub fn into_spectra_1d_by_source_path_prefix(self, path: impl AsRef<Path>) -> Vec<Spectrum1D> {
        self.into_spectra_1d_by_source(LoadedSourceFilter::path_prefix(path))
    }

    /// Consumes the bundle and returns one-dimensional spectra from tracked source paths below any prefix.
    #[must_use]
    pub fn into_spectra_1d_by_source_path_prefixes<I, P>(self, paths: I) -> Vec<Spectrum1D>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.into_spectra_1d_by_sources(path_prefix_filters(paths))
    }

    /// Consumes the bundle and returns two-dimensional spectra from tracked source paths below a prefix.
    #[must_use]
    pub fn into_spectra_2d_by_source_path_prefix(self, path: impl AsRef<Path>) -> Vec<Spectrum2D> {
        self.into_spectra_2d_by_source(LoadedSourceFilter::path_prefix(path))
    }

    /// Consumes the bundle and returns two-dimensional spectra from tracked source paths below any prefix.
    #[must_use]
    pub fn into_spectra_2d_by_source_path_prefixes<I, P>(self, paths: I) -> Vec<Spectrum2D>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.into_spectra_2d_by_sources(path_prefix_filters(paths))
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

fn path_prefix_filters<I, P>(paths: I) -> Vec<LoadedSourceFilter>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    path_prefixes(paths)
        .into_iter()
        .map(LoadedSourceFilter::path_prefix)
        .collect()
}

fn path_prefixes<I, P>(paths: I) -> Vec<PathBuf>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    let mut unique = Vec::new();
    for path in paths {
        let path = path.as_ref().to_path_buf();
        if !unique.iter().any(|existing| existing == &path) {
            unique.push(path);
        }
    }
    unique
}

fn warning_matches_any_prefix(warning: &LoadWarning, paths: &[PathBuf]) -> bool {
    paths.is_empty()
        || warning.path().is_some_and(|warning_path| {
            paths
                .iter()
                .any(|path| warning_path.starts_with(path.as_path()))
        })
}
