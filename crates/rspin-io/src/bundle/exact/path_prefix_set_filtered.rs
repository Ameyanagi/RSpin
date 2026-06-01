//! Source-path-prefix-set exact single-spectrum readers.

use std::path::Path;

use rspin_core::{Result, Spectrum1D, Spectrum2D};

use crate::bundle::{LoadedSource, LoadedSourceFilter, SpectrumBundleLoader};

impl SpectrumBundleLoader {
    /// Loads exactly one one-dimensional spectrum from tracked source paths below any prefix.
    ///
    /// Prefixes are combined with logical OR. Passing an empty iterator leaves
    /// source matching unrestricted.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// one-dimensional spectrum is not found.
    pub fn read_1d_by_source_path_prefixes<I, P>(
        &self,
        path: impl AsRef<Path>,
        source_path_prefixes: I,
    ) -> Result<Spectrum1D>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_1d_by_sources(path, path_prefix_filters(source_path_prefixes))
    }

    /// Loads exactly one one-dimensional spectrum and source from tracked source paths below any prefix.
    ///
    /// Prefixes are combined with logical OR. Passing an empty iterator leaves
    /// source matching unrestricted.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// one-dimensional spectrum is not found.
    pub fn read_1d_with_source_by_source_path_prefixes<I, P>(
        &self,
        path: impl AsRef<Path>,
        source_path_prefixes: I,
    ) -> Result<(Spectrum1D, LoadedSource)>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_1d_with_source_by_sources(path, path_prefix_filters(source_path_prefixes))
    }

    /// Loads exactly one two-dimensional spectrum from tracked source paths below any prefix.
    ///
    /// Prefixes are combined with logical OR. Passing an empty iterator leaves
    /// source matching unrestricted.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// two-dimensional spectrum is not found.
    pub fn read_2d_by_source_path_prefixes<I, P>(
        &self,
        path: impl AsRef<Path>,
        source_path_prefixes: I,
    ) -> Result<Spectrum2D>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_2d_by_sources(path, path_prefix_filters(source_path_prefixes))
    }

    /// Loads exactly one two-dimensional spectrum and source from tracked source paths below any prefix.
    ///
    /// Prefixes are combined with logical OR. Passing an empty iterator leaves
    /// source matching unrestricted.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// two-dimensional spectrum is not found.
    pub fn read_2d_with_source_by_source_path_prefixes<I, P>(
        &self,
        path: impl AsRef<Path>,
        source_path_prefixes: I,
    ) -> Result<(Spectrum2D, LoadedSource)>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_2d_with_source_by_sources(path, path_prefix_filters(source_path_prefixes))
    }

    /// Loads exactly one one-dimensional spectrum from tracked source paths below any prefix while anchoring input to a base directory.
    ///
    /// Prefixes are matched after source paths are anchored to `base`.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// one-dimensional spectrum is not found.
    pub fn read_1d_by_source_path_prefixes_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
        source_path_prefixes: I,
    ) -> Result<Spectrum1D>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_1d_by_sources_relative_to(base, path, path_prefix_filters(source_path_prefixes))
    }

    /// Loads exactly one one-dimensional spectrum and source from tracked source paths below any prefix while anchoring input to a base directory.
    ///
    /// Prefixes are matched after source paths are anchored to `base`.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// one-dimensional spectrum is not found.
    pub fn read_1d_with_source_by_source_path_prefixes_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
        source_path_prefixes: I,
    ) -> Result<(Spectrum1D, LoadedSource)>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_1d_with_source_by_sources_relative_to(
            base,
            path,
            path_prefix_filters(source_path_prefixes),
        )
    }

    /// Loads exactly one two-dimensional spectrum from tracked source paths below any prefix while anchoring input to a base directory.
    ///
    /// Prefixes are matched after source paths are anchored to `base`.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// two-dimensional spectrum is not found.
    pub fn read_2d_by_source_path_prefixes_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
        source_path_prefixes: I,
    ) -> Result<Spectrum2D>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_2d_by_sources_relative_to(base, path, path_prefix_filters(source_path_prefixes))
    }

    /// Loads exactly one two-dimensional spectrum and source from tracked source paths below any prefix while anchoring input to a base directory.
    ///
    /// Prefixes are matched after source paths are anchored to `base`.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// two-dimensional spectrum is not found.
    pub fn read_2d_with_source_by_source_path_prefixes_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
        source_path_prefixes: I,
    ) -> Result<(Spectrum2D, LoadedSource)>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_2d_with_source_by_sources_relative_to(
            base,
            path,
            path_prefix_filters(source_path_prefixes),
        )
    }

    /// Loads exactly one one-dimensional spectrum from tracked source paths below any prefix in selected paths.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// one-dimensional spectrum is not found.
    pub fn read_1d_many_by_source_path_prefixes<I, P, J, F>(
        &self,
        paths: I,
        source_path_prefixes: J,
    ) -> Result<Spectrum1D>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
        J: IntoIterator<Item = F>,
        F: AsRef<Path>,
    {
        self.read_1d_many_by_sources(paths, path_prefix_filters(source_path_prefixes))
    }

    /// Loads exactly one one-dimensional spectrum and source from tracked source paths below any prefix in selected paths.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// one-dimensional spectrum is not found.
    pub fn read_1d_many_with_source_by_source_path_prefixes<I, P, J, F>(
        &self,
        paths: I,
        source_path_prefixes: J,
    ) -> Result<(Spectrum1D, LoadedSource)>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
        J: IntoIterator<Item = F>,
        F: AsRef<Path>,
    {
        self.read_1d_many_with_source_by_sources(paths, path_prefix_filters(source_path_prefixes))
    }

    /// Loads exactly one two-dimensional spectrum from tracked source paths below any prefix in selected paths.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// two-dimensional spectrum is not found.
    pub fn read_2d_many_by_source_path_prefixes<I, P, J, F>(
        &self,
        paths: I,
        source_path_prefixes: J,
    ) -> Result<Spectrum2D>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
        J: IntoIterator<Item = F>,
        F: AsRef<Path>,
    {
        self.read_2d_many_by_sources(paths, path_prefix_filters(source_path_prefixes))
    }

    /// Loads exactly one two-dimensional spectrum and source from tracked source paths below any prefix in selected paths.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// two-dimensional spectrum is not found.
    pub fn read_2d_many_with_source_by_source_path_prefixes<I, P, J, F>(
        &self,
        paths: I,
        source_path_prefixes: J,
    ) -> Result<(Spectrum2D, LoadedSource)>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
        J: IntoIterator<Item = F>,
        F: AsRef<Path>,
    {
        self.read_2d_many_with_source_by_sources(paths, path_prefix_filters(source_path_prefixes))
    }

    /// Loads exactly one one-dimensional spectrum from tracked source paths below any prefix in selected paths relative to a base directory.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// one-dimensional spectrum is not found.
    pub fn read_1d_many_by_source_path_prefixes_relative_to<I, P, J, F>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
        source_path_prefixes: J,
    ) -> Result<Spectrum1D>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
        J: IntoIterator<Item = F>,
        F: AsRef<Path>,
    {
        self.read_1d_many_by_sources_relative_to(
            base,
            paths,
            path_prefix_filters(source_path_prefixes),
        )
    }

    /// Loads exactly one one-dimensional spectrum and source from tracked source paths below any prefix in selected paths relative to a base directory.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// one-dimensional spectrum is not found.
    pub fn read_1d_many_with_source_by_source_path_prefixes_relative_to<I, P, J, F>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
        source_path_prefixes: J,
    ) -> Result<(Spectrum1D, LoadedSource)>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
        J: IntoIterator<Item = F>,
        F: AsRef<Path>,
    {
        self.read_1d_many_with_source_by_sources_relative_to(
            base,
            paths,
            path_prefix_filters(source_path_prefixes),
        )
    }

    /// Loads exactly one two-dimensional spectrum from tracked source paths below any prefix in selected paths relative to a base directory.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// two-dimensional spectrum is not found.
    pub fn read_2d_many_by_source_path_prefixes_relative_to<I, P, J, F>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
        source_path_prefixes: J,
    ) -> Result<Spectrum2D>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
        J: IntoIterator<Item = F>,
        F: AsRef<Path>,
    {
        self.read_2d_many_by_sources_relative_to(
            base,
            paths,
            path_prefix_filters(source_path_prefixes),
        )
    }

    /// Loads exactly one two-dimensional spectrum and source from tracked source paths below any prefix in selected paths relative to a base directory.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// two-dimensional spectrum is not found.
    pub fn read_2d_many_with_source_by_source_path_prefixes_relative_to<I, P, J, F>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
        source_path_prefixes: J,
    ) -> Result<(Spectrum2D, LoadedSource)>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
        J: IntoIterator<Item = F>,
        F: AsRef<Path>,
    {
        self.read_2d_many_with_source_by_sources_relative_to(
            base,
            paths,
            path_prefix_filters(source_path_prefixes),
        )
    }
}

/// Loads exactly one one-dimensional spectrum from tracked source paths below any prefix.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching one-dimensional
/// spectrum is not found.
pub fn load_spectrum_1d_by_source_path_prefixes<I, P>(
    path: impl AsRef<Path>,
    source_path_prefixes: I,
) -> Result<Spectrum1D>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_1d_by_source_path_prefixes(path, source_path_prefixes)
}

/// Loads exactly one one-dimensional spectrum and source from tracked source paths below any prefix.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching one-dimensional
/// spectrum is not found.
pub fn load_spectrum_1d_with_source_by_source_path_prefixes<I, P>(
    path: impl AsRef<Path>,
    source_path_prefixes: I,
) -> Result<(Spectrum1D, LoadedSource)>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new()
        .read_1d_with_source_by_source_path_prefixes(path, source_path_prefixes)
}

/// Loads exactly one two-dimensional spectrum from tracked source paths below any prefix.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching two-dimensional
/// spectrum is not found.
pub fn load_spectrum_2d_by_source_path_prefixes<I, P>(
    path: impl AsRef<Path>,
    source_path_prefixes: I,
) -> Result<Spectrum2D>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_2d_by_source_path_prefixes(path, source_path_prefixes)
}

/// Loads exactly one two-dimensional spectrum and source from tracked source paths below any prefix.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching two-dimensional
/// spectrum is not found.
pub fn load_spectrum_2d_with_source_by_source_path_prefixes<I, P>(
    path: impl AsRef<Path>,
    source_path_prefixes: I,
) -> Result<(Spectrum2D, LoadedSource)>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new()
        .read_2d_with_source_by_source_path_prefixes(path, source_path_prefixes)
}

/// Loads exactly one one-dimensional spectrum from tracked source paths below any prefix while anchoring input to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching one-dimensional
/// spectrum is not found.
pub fn load_spectrum_1d_by_source_path_prefixes_relative_to<I, P>(
    base: impl AsRef<Path>,
    path: impl AsRef<Path>,
    source_path_prefixes: I,
) -> Result<Spectrum1D>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_1d_by_source_path_prefixes_relative_to(
        base,
        path,
        source_path_prefixes,
    )
}

/// Loads exactly one one-dimensional spectrum and source from tracked source paths below any prefix while anchoring input to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching one-dimensional
/// spectrum is not found.
pub fn load_spectrum_1d_with_source_by_source_path_prefixes_relative_to<I, P>(
    base: impl AsRef<Path>,
    path: impl AsRef<Path>,
    source_path_prefixes: I,
) -> Result<(Spectrum1D, LoadedSource)>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_1d_with_source_by_source_path_prefixes_relative_to(
        base,
        path,
        source_path_prefixes,
    )
}

/// Loads exactly one two-dimensional spectrum from tracked source paths below any prefix while anchoring input to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching two-dimensional
/// spectrum is not found.
pub fn load_spectrum_2d_by_source_path_prefixes_relative_to<I, P>(
    base: impl AsRef<Path>,
    path: impl AsRef<Path>,
    source_path_prefixes: I,
) -> Result<Spectrum2D>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_2d_by_source_path_prefixes_relative_to(
        base,
        path,
        source_path_prefixes,
    )
}

/// Loads exactly one two-dimensional spectrum and source from tracked source paths below any prefix while anchoring input to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching two-dimensional
/// spectrum is not found.
pub fn load_spectrum_2d_with_source_by_source_path_prefixes_relative_to<I, P>(
    base: impl AsRef<Path>,
    path: impl AsRef<Path>,
    source_path_prefixes: I,
) -> Result<(Spectrum2D, LoadedSource)>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_2d_with_source_by_source_path_prefixes_relative_to(
        base,
        path,
        source_path_prefixes,
    )
}

/// Loads exactly one one-dimensional spectrum from tracked source paths below any prefix in selected paths.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching one-dimensional
/// spectrum is not found.
pub fn load_spectrum_1d_many_by_source_path_prefixes<I, P, J, F>(
    paths: I,
    source_path_prefixes: J,
) -> Result<Spectrum1D>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
    J: IntoIterator<Item = F>,
    F: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_1d_many_by_source_path_prefixes(paths, source_path_prefixes)
}

/// Loads exactly one one-dimensional spectrum and source from tracked source paths below any prefix in selected paths.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching one-dimensional
/// spectrum is not found.
pub fn load_spectrum_1d_many_with_source_by_source_path_prefixes<I, P, J, F>(
    paths: I,
    source_path_prefixes: J,
) -> Result<(Spectrum1D, LoadedSource)>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
    J: IntoIterator<Item = F>,
    F: AsRef<Path>,
{
    SpectrumBundleLoader::new()
        .read_1d_many_with_source_by_source_path_prefixes(paths, source_path_prefixes)
}

/// Loads exactly one two-dimensional spectrum from tracked source paths below any prefix in selected paths.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching two-dimensional
/// spectrum is not found.
pub fn load_spectrum_2d_many_by_source_path_prefixes<I, P, J, F>(
    paths: I,
    source_path_prefixes: J,
) -> Result<Spectrum2D>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
    J: IntoIterator<Item = F>,
    F: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_2d_many_by_source_path_prefixes(paths, source_path_prefixes)
}

/// Loads exactly one two-dimensional spectrum and source from tracked source paths below any prefix in selected paths.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching two-dimensional
/// spectrum is not found.
pub fn load_spectrum_2d_many_with_source_by_source_path_prefixes<I, P, J, F>(
    paths: I,
    source_path_prefixes: J,
) -> Result<(Spectrum2D, LoadedSource)>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
    J: IntoIterator<Item = F>,
    F: AsRef<Path>,
{
    SpectrumBundleLoader::new()
        .read_2d_many_with_source_by_source_path_prefixes(paths, source_path_prefixes)
}

/// Loads exactly one one-dimensional spectrum from tracked source paths below any prefix in selected paths relative to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching one-dimensional
/// spectrum is not found.
pub fn load_spectrum_1d_many_by_source_path_prefixes_relative_to<I, P, J, F>(
    base: impl AsRef<Path>,
    paths: I,
    source_path_prefixes: J,
) -> Result<Spectrum1D>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
    J: IntoIterator<Item = F>,
    F: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_1d_many_by_source_path_prefixes_relative_to(
        base,
        paths,
        source_path_prefixes,
    )
}

/// Loads exactly one one-dimensional spectrum and source from tracked source paths below any prefix in selected paths relative to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching one-dimensional
/// spectrum is not found.
pub fn load_spectrum_1d_many_with_source_by_source_path_prefixes_relative_to<I, P, J, F>(
    base: impl AsRef<Path>,
    paths: I,
    source_path_prefixes: J,
) -> Result<(Spectrum1D, LoadedSource)>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
    J: IntoIterator<Item = F>,
    F: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_1d_many_with_source_by_source_path_prefixes_relative_to(
        base,
        paths,
        source_path_prefixes,
    )
}

/// Loads exactly one two-dimensional spectrum from tracked source paths below any prefix in selected paths relative to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching two-dimensional
/// spectrum is not found.
pub fn load_spectrum_2d_many_by_source_path_prefixes_relative_to<I, P, J, F>(
    base: impl AsRef<Path>,
    paths: I,
    source_path_prefixes: J,
) -> Result<Spectrum2D>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
    J: IntoIterator<Item = F>,
    F: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_2d_many_by_source_path_prefixes_relative_to(
        base,
        paths,
        source_path_prefixes,
    )
}

/// Loads exactly one two-dimensional spectrum and source from tracked source paths below any prefix in selected paths relative to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching two-dimensional
/// spectrum is not found.
pub fn load_spectrum_2d_many_with_source_by_source_path_prefixes_relative_to<I, P, J, F>(
    base: impl AsRef<Path>,
    paths: I,
    source_path_prefixes: J,
) -> Result<(Spectrum2D, LoadedSource)>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
    J: IntoIterator<Item = F>,
    F: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_2d_many_with_source_by_source_path_prefixes_relative_to(
        base,
        paths,
        source_path_prefixes,
    )
}

fn path_prefix_filters<I, P>(source_path_prefixes: I) -> Vec<LoadedSourceFilter>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    let mut filters = Vec::new();
    for source_path_prefix in source_path_prefixes {
        let filter = LoadedSourceFilter::path_prefix(source_path_prefix);
        if !filters.iter().any(|existing| existing == &filter) {
            filters.push(filter);
        }
    }
    filters
}
