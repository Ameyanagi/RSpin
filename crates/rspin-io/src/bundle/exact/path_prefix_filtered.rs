//! Source-path-prefix exact single-spectrum readers.

use std::path::Path;

use rspin_core::{Result, Spectrum1D, Spectrum2D};

use crate::bundle::{LoadedSource, SpectrumBundleLoader};

impl SpectrumBundleLoader {
    /// Loads exactly one one-dimensional spectrum from tracked source paths below a prefix.
    ///
    /// Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// one-dimensional spectrum is not found.
    pub fn read_1d_by_source_path_prefix(
        &self,
        path: impl AsRef<Path>,
        source_path_prefix: impl AsRef<Path>,
    ) -> Result<Spectrum1D> {
        self.read_path(path)?
            .into_only_1d_by_source_path_prefix(source_path_prefix)
    }

    /// Loads exactly one one-dimensional spectrum and source from tracked source paths below a prefix.
    ///
    /// Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// one-dimensional spectrum is not found.
    pub fn read_1d_with_source_by_source_path_prefix(
        &self,
        path: impl AsRef<Path>,
        source_path_prefix: impl AsRef<Path>,
    ) -> Result<(Spectrum1D, LoadedSource)> {
        self.read_path(path)?
            .into_only_loaded_1d_by_source_path_prefix(source_path_prefix)
    }

    /// Loads exactly one two-dimensional spectrum from tracked source paths below a prefix.
    ///
    /// Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// two-dimensional spectrum is not found.
    pub fn read_2d_by_source_path_prefix(
        &self,
        path: impl AsRef<Path>,
        source_path_prefix: impl AsRef<Path>,
    ) -> Result<Spectrum2D> {
        self.read_path(path)?
            .into_only_2d_by_source_path_prefix(source_path_prefix)
    }

    /// Loads exactly one two-dimensional spectrum and source from tracked source paths below a prefix.
    ///
    /// Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// two-dimensional spectrum is not found.
    pub fn read_2d_with_source_by_source_path_prefix(
        &self,
        path: impl AsRef<Path>,
        source_path_prefix: impl AsRef<Path>,
    ) -> Result<(Spectrum2D, LoadedSource)> {
        self.read_path(path)?
            .into_only_loaded_2d_by_source_path_prefix(source_path_prefix)
    }

    /// Loads exactly one one-dimensional spectrum from tracked source paths below a prefix while anchoring input to a base directory.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// one-dimensional spectrum is not found.
    pub fn read_1d_by_source_path_prefix_relative_to(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
        source_path_prefix: impl AsRef<Path>,
    ) -> Result<Spectrum1D> {
        self.read_path_relative_to(base, path)?
            .into_only_1d_by_source_path_prefix(source_path_prefix)
    }

    /// Loads exactly one one-dimensional spectrum and source from tracked source paths below a prefix while anchoring input to a base directory.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// one-dimensional spectrum is not found.
    pub fn read_1d_with_source_by_source_path_prefix_relative_to(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
        source_path_prefix: impl AsRef<Path>,
    ) -> Result<(Spectrum1D, LoadedSource)> {
        self.read_path_relative_to(base, path)?
            .into_only_loaded_1d_by_source_path_prefix(source_path_prefix)
    }

    /// Loads exactly one two-dimensional spectrum from tracked source paths below a prefix while anchoring input to a base directory.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// two-dimensional spectrum is not found.
    pub fn read_2d_by_source_path_prefix_relative_to(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
        source_path_prefix: impl AsRef<Path>,
    ) -> Result<Spectrum2D> {
        self.read_path_relative_to(base, path)?
            .into_only_2d_by_source_path_prefix(source_path_prefix)
    }

    /// Loads exactly one two-dimensional spectrum and source from tracked source paths below a prefix while anchoring input to a base directory.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// two-dimensional spectrum is not found.
    pub fn read_2d_with_source_by_source_path_prefix_relative_to(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
        source_path_prefix: impl AsRef<Path>,
    ) -> Result<(Spectrum2D, LoadedSource)> {
        self.read_path_relative_to(base, path)?
            .into_only_loaded_2d_by_source_path_prefix(source_path_prefix)
    }

    /// Loads exactly one one-dimensional spectrum from tracked source paths below a prefix in selected paths.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// one-dimensional spectrum is not found.
    pub fn read_1d_many_by_source_path_prefix<I, P>(
        &self,
        paths: I,
        source_path_prefix: impl AsRef<Path>,
    ) -> Result<Spectrum1D>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths(paths)?
            .into_only_1d_by_source_path_prefix(source_path_prefix)
    }

    /// Loads exactly one one-dimensional spectrum and source from tracked source paths below a prefix in selected paths.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// one-dimensional spectrum is not found.
    pub fn read_1d_many_with_source_by_source_path_prefix<I, P>(
        &self,
        paths: I,
        source_path_prefix: impl AsRef<Path>,
    ) -> Result<(Spectrum1D, LoadedSource)>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths(paths)?
            .into_only_loaded_1d_by_source_path_prefix(source_path_prefix)
    }

    /// Loads exactly one two-dimensional spectrum from tracked source paths below a prefix in selected paths.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// two-dimensional spectrum is not found.
    pub fn read_2d_many_by_source_path_prefix<I, P>(
        &self,
        paths: I,
        source_path_prefix: impl AsRef<Path>,
    ) -> Result<Spectrum2D>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths(paths)?
            .into_only_2d_by_source_path_prefix(source_path_prefix)
    }

    /// Loads exactly one two-dimensional spectrum and source from tracked source paths below a prefix in selected paths.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// two-dimensional spectrum is not found.
    pub fn read_2d_many_with_source_by_source_path_prefix<I, P>(
        &self,
        paths: I,
        source_path_prefix: impl AsRef<Path>,
    ) -> Result<(Spectrum2D, LoadedSource)>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths(paths)?
            .into_only_loaded_2d_by_source_path_prefix(source_path_prefix)
    }

    /// Loads exactly one one-dimensional spectrum from tracked source paths below a prefix in selected paths relative to a base directory.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// one-dimensional spectrum is not found.
    pub fn read_1d_many_by_source_path_prefix_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
        source_path_prefix: impl AsRef<Path>,
    ) -> Result<Spectrum1D>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths_relative_to(base, paths)?
            .into_only_1d_by_source_path_prefix(source_path_prefix)
    }

    /// Loads exactly one one-dimensional spectrum and source from tracked source paths below a prefix in selected paths relative to a base directory.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// one-dimensional spectrum is not found.
    pub fn read_1d_many_with_source_by_source_path_prefix_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
        source_path_prefix: impl AsRef<Path>,
    ) -> Result<(Spectrum1D, LoadedSource)>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths_relative_to(base, paths)?
            .into_only_loaded_1d_by_source_path_prefix(source_path_prefix)
    }

    /// Loads exactly one two-dimensional spectrum from tracked source paths below a prefix in selected paths relative to a base directory.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// two-dimensional spectrum is not found.
    pub fn read_2d_many_by_source_path_prefix_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
        source_path_prefix: impl AsRef<Path>,
    ) -> Result<Spectrum2D>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths_relative_to(base, paths)?
            .into_only_2d_by_source_path_prefix(source_path_prefix)
    }

    /// Loads exactly one two-dimensional spectrum and source from tracked source paths below a prefix in selected paths relative to a base directory.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// two-dimensional spectrum is not found.
    pub fn read_2d_many_with_source_by_source_path_prefix_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
        source_path_prefix: impl AsRef<Path>,
    ) -> Result<(Spectrum2D, LoadedSource)>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths_relative_to(base, paths)?
            .into_only_loaded_2d_by_source_path_prefix(source_path_prefix)
    }
}

/// Loads exactly one one-dimensional spectrum from tracked source paths below a prefix.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching one-dimensional
/// spectrum is not found.
pub fn load_spectrum_1d_by_source_path_prefix(
    path: impl AsRef<Path>,
    source_path_prefix: impl AsRef<Path>,
) -> Result<Spectrum1D> {
    SpectrumBundleLoader::new().read_1d_by_source_path_prefix(path, source_path_prefix)
}

/// Loads exactly one one-dimensional spectrum and source from tracked source paths below a prefix.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching one-dimensional
/// spectrum is not found.
pub fn load_spectrum_1d_with_source_by_source_path_prefix(
    path: impl AsRef<Path>,
    source_path_prefix: impl AsRef<Path>,
) -> Result<(Spectrum1D, LoadedSource)> {
    SpectrumBundleLoader::new().read_1d_with_source_by_source_path_prefix(path, source_path_prefix)
}

/// Loads exactly one two-dimensional spectrum from tracked source paths below a prefix.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching two-dimensional
/// spectrum is not found.
pub fn load_spectrum_2d_by_source_path_prefix(
    path: impl AsRef<Path>,
    source_path_prefix: impl AsRef<Path>,
) -> Result<Spectrum2D> {
    SpectrumBundleLoader::new().read_2d_by_source_path_prefix(path, source_path_prefix)
}

/// Loads exactly one two-dimensional spectrum and source from tracked source paths below a prefix.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching two-dimensional
/// spectrum is not found.
pub fn load_spectrum_2d_with_source_by_source_path_prefix(
    path: impl AsRef<Path>,
    source_path_prefix: impl AsRef<Path>,
) -> Result<(Spectrum2D, LoadedSource)> {
    SpectrumBundleLoader::new().read_2d_with_source_by_source_path_prefix(path, source_path_prefix)
}

/// Loads exactly one one-dimensional spectrum from tracked source paths below a prefix while anchoring input to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching one-dimensional
/// spectrum is not found.
pub fn load_spectrum_1d_by_source_path_prefix_relative_to(
    base: impl AsRef<Path>,
    path: impl AsRef<Path>,
    source_path_prefix: impl AsRef<Path>,
) -> Result<Spectrum1D> {
    SpectrumBundleLoader::new().read_1d_by_source_path_prefix_relative_to(
        base,
        path,
        source_path_prefix,
    )
}

/// Loads exactly one one-dimensional spectrum and source from tracked source paths below a prefix while anchoring input to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching one-dimensional
/// spectrum is not found.
pub fn load_spectrum_1d_with_source_by_source_path_prefix_relative_to(
    base: impl AsRef<Path>,
    path: impl AsRef<Path>,
    source_path_prefix: impl AsRef<Path>,
) -> Result<(Spectrum1D, LoadedSource)> {
    SpectrumBundleLoader::new().read_1d_with_source_by_source_path_prefix_relative_to(
        base,
        path,
        source_path_prefix,
    )
}

/// Loads exactly one two-dimensional spectrum from tracked source paths below a prefix while anchoring input to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching two-dimensional
/// spectrum is not found.
pub fn load_spectrum_2d_by_source_path_prefix_relative_to(
    base: impl AsRef<Path>,
    path: impl AsRef<Path>,
    source_path_prefix: impl AsRef<Path>,
) -> Result<Spectrum2D> {
    SpectrumBundleLoader::new().read_2d_by_source_path_prefix_relative_to(
        base,
        path,
        source_path_prefix,
    )
}

/// Loads exactly one two-dimensional spectrum and source from tracked source paths below a prefix while anchoring input to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching two-dimensional
/// spectrum is not found.
pub fn load_spectrum_2d_with_source_by_source_path_prefix_relative_to(
    base: impl AsRef<Path>,
    path: impl AsRef<Path>,
    source_path_prefix: impl AsRef<Path>,
) -> Result<(Spectrum2D, LoadedSource)> {
    SpectrumBundleLoader::new().read_2d_with_source_by_source_path_prefix_relative_to(
        base,
        path,
        source_path_prefix,
    )
}

/// Loads exactly one one-dimensional spectrum from tracked source paths below a prefix in selected paths.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching one-dimensional
/// spectrum is not found.
pub fn load_spectrum_1d_many_by_source_path_prefix<I, P>(
    paths: I,
    source_path_prefix: impl AsRef<Path>,
) -> Result<Spectrum1D>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_1d_many_by_source_path_prefix(paths, source_path_prefix)
}

/// Loads exactly one one-dimensional spectrum and source from tracked source paths below a prefix in selected paths.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching one-dimensional
/// spectrum is not found.
pub fn load_spectrum_1d_many_with_source_by_source_path_prefix<I, P>(
    paths: I,
    source_path_prefix: impl AsRef<Path>,
) -> Result<(Spectrum1D, LoadedSource)>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new()
        .read_1d_many_with_source_by_source_path_prefix(paths, source_path_prefix)
}

/// Loads exactly one two-dimensional spectrum from tracked source paths below a prefix in selected paths.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching two-dimensional
/// spectrum is not found.
pub fn load_spectrum_2d_many_by_source_path_prefix<I, P>(
    paths: I,
    source_path_prefix: impl AsRef<Path>,
) -> Result<Spectrum2D>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_2d_many_by_source_path_prefix(paths, source_path_prefix)
}

/// Loads exactly one two-dimensional spectrum and source from tracked source paths below a prefix in selected paths.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching two-dimensional
/// spectrum is not found.
pub fn load_spectrum_2d_many_with_source_by_source_path_prefix<I, P>(
    paths: I,
    source_path_prefix: impl AsRef<Path>,
) -> Result<(Spectrum2D, LoadedSource)>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new()
        .read_2d_many_with_source_by_source_path_prefix(paths, source_path_prefix)
}

/// Loads exactly one one-dimensional spectrum from tracked source paths below a prefix in selected paths relative to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching one-dimensional
/// spectrum is not found.
pub fn load_spectrum_1d_many_by_source_path_prefix_relative_to<I, P>(
    base: impl AsRef<Path>,
    paths: I,
    source_path_prefix: impl AsRef<Path>,
) -> Result<Spectrum1D>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_1d_many_by_source_path_prefix_relative_to(
        base,
        paths,
        source_path_prefix,
    )
}

/// Loads exactly one one-dimensional spectrum and source from tracked source paths below a prefix in selected paths relative to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching one-dimensional
/// spectrum is not found.
pub fn load_spectrum_1d_many_with_source_by_source_path_prefix_relative_to<I, P>(
    base: impl AsRef<Path>,
    paths: I,
    source_path_prefix: impl AsRef<Path>,
) -> Result<(Spectrum1D, LoadedSource)>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_1d_many_with_source_by_source_path_prefix_relative_to(
        base,
        paths,
        source_path_prefix,
    )
}

/// Loads exactly one two-dimensional spectrum from tracked source paths below a prefix in selected paths relative to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching two-dimensional
/// spectrum is not found.
pub fn load_spectrum_2d_many_by_source_path_prefix_relative_to<I, P>(
    base: impl AsRef<Path>,
    paths: I,
    source_path_prefix: impl AsRef<Path>,
) -> Result<Spectrum2D>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_2d_many_by_source_path_prefix_relative_to(
        base,
        paths,
        source_path_prefix,
    )
}

/// Loads exactly one two-dimensional spectrum and source from tracked source paths below a prefix in selected paths relative to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching two-dimensional
/// spectrum is not found.
pub fn load_spectrum_2d_many_with_source_by_source_path_prefix_relative_to<I, P>(
    base: impl AsRef<Path>,
    paths: I,
    source_path_prefix: impl AsRef<Path>,
) -> Result<(Spectrum2D, LoadedSource)>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_2d_many_with_source_by_source_path_prefix_relative_to(
        base,
        paths,
        source_path_prefix,
    )
}
