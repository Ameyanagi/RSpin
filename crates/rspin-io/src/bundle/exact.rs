//! Exact single-spectrum convenience readers.

use std::path::Path;

use rspin_core::{Result, Spectrum1D, Spectrum2D};

use super::{LoadedSource, SpectrumBundleLoader};

impl SpectrumBundleLoader {
    /// Loads exactly one one-dimensional spectrum from a file or directory path.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the path does not resolve to
    /// exactly one one-dimensional spectrum.
    pub fn read_1d(&self, path: impl AsRef<Path>) -> Result<Spectrum1D> {
        self.read_path(path)?.into_only_1d()
    }

    /// Loads exactly one one-dimensional spectrum with source metadata from a file or directory path.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the path does not resolve to
    /// exactly one one-dimensional spectrum.
    pub fn read_1d_with_source(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<(Spectrum1D, LoadedSource)> {
        self.read_path(path)?.into_only_loaded_1d()
    }

    /// Loads exactly one two-dimensional spectrum from a file or directory path.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the path does not resolve to
    /// exactly one two-dimensional spectrum.
    pub fn read_2d(&self, path: impl AsRef<Path>) -> Result<Spectrum2D> {
        self.read_path(path)?.into_only_2d()
    }

    /// Loads exactly one two-dimensional spectrum with source metadata from a file or directory path.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the path does not resolve to
    /// exactly one two-dimensional spectrum.
    pub fn read_2d_with_source(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<(Spectrum2D, LoadedSource)> {
        self.read_path(path)?.into_only_loaded_2d()
    }

    /// Loads exactly one one-dimensional spectrum read with a source format.
    ///
    /// Source format aliases such as `jdx` and `jdf` are accepted. Other
    /// matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// one-dimensional spectrum is not found.
    pub fn read_1d_by_source_format(
        &self,
        path: impl AsRef<Path>,
        format: impl AsRef<str>,
    ) -> Result<Spectrum1D> {
        self.read_path(path)?.into_only_1d_by_source_format(format)
    }

    /// Loads exactly one one-dimensional spectrum and source read with a source format.
    ///
    /// Source format aliases such as `jdx` and `jdf` are accepted. Other
    /// matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// one-dimensional spectrum is not found.
    pub fn read_1d_with_source_by_source_format(
        &self,
        path: impl AsRef<Path>,
        format: impl AsRef<str>,
    ) -> Result<(Spectrum1D, LoadedSource)> {
        self.read_path(path)?
            .into_only_loaded_1d_by_source_format(format)
    }

    /// Loads exactly one two-dimensional spectrum read with a source format.
    ///
    /// Source format aliases such as `jdx` and `jdf` are accepted. Other
    /// matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// two-dimensional spectrum is not found.
    pub fn read_2d_by_source_format(
        &self,
        path: impl AsRef<Path>,
        format: impl AsRef<str>,
    ) -> Result<Spectrum2D> {
        self.read_path(path)?.into_only_2d_by_source_format(format)
    }

    /// Loads exactly one two-dimensional spectrum and source read with a source format.
    ///
    /// Source format aliases such as `jdx` and `jdf` are accepted. Other
    /// matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// two-dimensional spectrum is not found.
    pub fn read_2d_with_source_by_source_format(
        &self,
        path: impl AsRef<Path>,
        format: impl AsRef<str>,
    ) -> Result<(Spectrum2D, LoadedSource)> {
        self.read_path(path)?
            .into_only_loaded_2d_by_source_format(format)
    }

    /// Loads exactly one one-dimensional spectrum read with a vendor-specific reader.
    ///
    /// Vendor aliases such as `agilent` and `varian` are accepted. Other
    /// matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// one-dimensional spectrum is not found.
    pub fn read_1d_by_source_vendor(
        &self,
        path: impl AsRef<Path>,
        vendor: impl AsRef<str>,
    ) -> Result<Spectrum1D> {
        self.read_path(path)?.into_only_1d_by_source_vendor(vendor)
    }

    /// Loads exactly one one-dimensional spectrum and source read with a vendor-specific reader.
    ///
    /// Vendor aliases such as `agilent` and `varian` are accepted. Other
    /// matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// one-dimensional spectrum is not found.
    pub fn read_1d_with_source_by_source_vendor(
        &self,
        path: impl AsRef<Path>,
        vendor: impl AsRef<str>,
    ) -> Result<(Spectrum1D, LoadedSource)> {
        self.read_path(path)?
            .into_only_loaded_1d_by_source_vendor(vendor)
    }

    /// Loads exactly one two-dimensional spectrum read with a vendor-specific reader.
    ///
    /// Vendor aliases such as `agilent` and `varian` are accepted. Other
    /// matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// two-dimensional spectrum is not found.
    pub fn read_2d_by_source_vendor(
        &self,
        path: impl AsRef<Path>,
        vendor: impl AsRef<str>,
    ) -> Result<Spectrum2D> {
        self.read_path(path)?.into_only_2d_by_source_vendor(vendor)
    }

    /// Loads exactly one two-dimensional spectrum and source read with a vendor-specific reader.
    ///
    /// Vendor aliases such as `agilent` and `varian` are accepted. Other
    /// matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// two-dimensional spectrum is not found.
    pub fn read_2d_with_source_by_source_vendor(
        &self,
        path: impl AsRef<Path>,
        vendor: impl AsRef<str>,
    ) -> Result<(Spectrum2D, LoadedSource)> {
        self.read_path(path)?
            .into_only_loaded_2d_by_source_vendor(vendor)
    }

    /// Loads exactly one one-dimensional spectrum from a path relative to a base directory.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the path does not resolve to
    /// exactly one one-dimensional spectrum.
    pub fn read_1d_relative_to(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
    ) -> Result<Spectrum1D> {
        self.read_path_relative_to(base, path)?.into_only_1d()
    }

    /// Loads exactly one one-dimensional spectrum with source metadata from a path relative to a base directory.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the path does not resolve to
    /// exactly one one-dimensional spectrum.
    pub fn read_1d_with_source_relative_to(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
    ) -> Result<(Spectrum1D, LoadedSource)> {
        self.read_path_relative_to(base, path)?
            .into_only_loaded_1d()
    }

    /// Loads exactly one two-dimensional spectrum from a path relative to a base directory.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the path does not resolve to
    /// exactly one two-dimensional spectrum.
    pub fn read_2d_relative_to(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
    ) -> Result<Spectrum2D> {
        self.read_path_relative_to(base, path)?.into_only_2d()
    }

    /// Loads exactly one two-dimensional spectrum with source metadata from a path relative to a base directory.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the path does not resolve to
    /// exactly one two-dimensional spectrum.
    pub fn read_2d_with_source_relative_to(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
    ) -> Result<(Spectrum2D, LoadedSource)> {
        self.read_path_relative_to(base, path)?
            .into_only_loaded_2d()
    }

    /// Loads exactly one one-dimensional spectrum from selected file or directory paths.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the selected paths do not resolve
    /// to exactly one one-dimensional spectrum.
    pub fn read_1d_paths<I, P>(&self, paths: I) -> Result<Spectrum1D>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths(paths)?.into_only_1d()
    }

    /// Loads exactly one one-dimensional spectrum with source metadata from selected file or directory paths.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the selected paths do not resolve
    /// to exactly one one-dimensional spectrum.
    pub fn read_1d_paths_with_source<I, P>(&self, paths: I) -> Result<(Spectrum1D, LoadedSource)>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths(paths)?.into_only_loaded_1d()
    }

    /// Loads exactly one one-dimensional spectrum from selected file or directory paths.
    ///
    /// This is a short alias for [`Self::read_1d_paths`] matching the
    /// [`Self::read_many`] bundle-reader naming.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the selected paths do not resolve
    /// to exactly one one-dimensional spectrum.
    pub fn read_1d_many<I, P>(&self, paths: I) -> Result<Spectrum1D>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_1d_paths(paths)
    }

    /// Loads exactly one one-dimensional spectrum with source metadata from selected file or directory paths.
    ///
    /// This is a short alias for [`Self::read_1d_paths_with_source`] matching
    /// the [`Self::read_many`] bundle-reader naming.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the selected paths do not resolve
    /// to exactly one one-dimensional spectrum.
    pub fn read_1d_many_with_source<I, P>(&self, paths: I) -> Result<(Spectrum1D, LoadedSource)>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_1d_paths_with_source(paths)
    }

    /// Loads exactly one two-dimensional spectrum from selected file or directory paths.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the selected paths do not resolve
    /// to exactly one two-dimensional spectrum.
    pub fn read_2d_paths<I, P>(&self, paths: I) -> Result<Spectrum2D>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths(paths)?.into_only_2d()
    }

    /// Loads exactly one two-dimensional spectrum with source metadata from selected file or directory paths.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the selected paths do not resolve
    /// to exactly one two-dimensional spectrum.
    pub fn read_2d_paths_with_source<I, P>(&self, paths: I) -> Result<(Spectrum2D, LoadedSource)>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths(paths)?.into_only_loaded_2d()
    }

    /// Loads exactly one two-dimensional spectrum from selected file or directory paths.
    ///
    /// This is a short alias for [`Self::read_2d_paths`] matching the
    /// [`Self::read_many`] bundle-reader naming.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the selected paths do not resolve
    /// to exactly one two-dimensional spectrum.
    pub fn read_2d_many<I, P>(&self, paths: I) -> Result<Spectrum2D>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_2d_paths(paths)
    }

    /// Loads exactly one two-dimensional spectrum with source metadata from selected file or directory paths.
    ///
    /// This is a short alias for [`Self::read_2d_paths_with_source`] matching
    /// the [`Self::read_many`] bundle-reader naming.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the selected paths do not resolve
    /// to exactly one two-dimensional spectrum.
    pub fn read_2d_many_with_source<I, P>(&self, paths: I) -> Result<(Spectrum2D, LoadedSource)>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_2d_paths_with_source(paths)
    }

    /// Loads exactly one one-dimensional spectrum from paths relative to a base directory.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the selected paths do not resolve
    /// to exactly one one-dimensional spectrum.
    pub fn read_1d_paths_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
    ) -> Result<Spectrum1D>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths_relative_to(base, paths)?.into_only_1d()
    }

    /// Loads exactly one one-dimensional spectrum with source metadata from paths relative to a base directory.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the selected paths do not resolve
    /// to exactly one one-dimensional spectrum.
    pub fn read_1d_paths_with_source_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
    ) -> Result<(Spectrum1D, LoadedSource)>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths_relative_to(base, paths)?
            .into_only_loaded_1d()
    }

    /// Loads exactly one one-dimensional spectrum from paths relative to a base directory.
    ///
    /// This is a short alias for [`Self::read_1d_paths_relative_to`] matching
    /// the [`Self::read_many_relative_to`] bundle-reader naming.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the selected paths do not resolve
    /// to exactly one one-dimensional spectrum.
    pub fn read_1d_many_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
    ) -> Result<Spectrum1D>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_1d_paths_relative_to(base, paths)
    }

    /// Loads exactly one one-dimensional spectrum with source metadata from paths relative to a base directory.
    ///
    /// This is a short alias for [`Self::read_1d_paths_with_source_relative_to`]
    /// matching the [`Self::read_many_relative_to`] bundle-reader naming.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the selected paths do not resolve
    /// to exactly one one-dimensional spectrum.
    pub fn read_1d_many_with_source_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
    ) -> Result<(Spectrum1D, LoadedSource)>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_1d_paths_with_source_relative_to(base, paths)
    }

    /// Loads exactly one two-dimensional spectrum from paths relative to a base directory.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the selected paths do not resolve
    /// to exactly one two-dimensional spectrum.
    pub fn read_2d_paths_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
    ) -> Result<Spectrum2D>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths_relative_to(base, paths)?.into_only_2d()
    }

    /// Loads exactly one two-dimensional spectrum with source metadata from paths relative to a base directory.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the selected paths do not resolve
    /// to exactly one two-dimensional spectrum.
    pub fn read_2d_paths_with_source_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
    ) -> Result<(Spectrum2D, LoadedSource)>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths_relative_to(base, paths)?
            .into_only_loaded_2d()
    }

    /// Loads exactly one two-dimensional spectrum from paths relative to a base directory.
    ///
    /// This is a short alias for [`Self::read_2d_paths_relative_to`] matching
    /// the [`Self::read_many_relative_to`] bundle-reader naming.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the selected paths do not resolve
    /// to exactly one two-dimensional spectrum.
    pub fn read_2d_many_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
    ) -> Result<Spectrum2D>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_2d_paths_relative_to(base, paths)
    }

    /// Loads exactly one two-dimensional spectrum with source metadata from paths relative to a base directory.
    ///
    /// This is a short alias for [`Self::read_2d_paths_with_source_relative_to`]
    /// matching the [`Self::read_many_relative_to`] bundle-reader naming.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or the selected paths do not resolve
    /// to exactly one two-dimensional spectrum.
    pub fn read_2d_many_with_source_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
    ) -> Result<(Spectrum2D, LoadedSource)>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_2d_paths_with_source_relative_to(base, paths)
    }
}

/// Loads exactly one one-dimensional spectrum from a file or directory path.
///
/// # Errors
///
/// Returns an error when loading fails or the path does not resolve to exactly
/// one one-dimensional spectrum.
pub fn load_spectrum_1d(path: impl AsRef<Path>) -> Result<Spectrum1D> {
    SpectrumBundleLoader::new().read_1d(path)
}

/// Loads exactly one one-dimensional spectrum with source metadata from a file or directory path.
///
/// # Errors
///
/// Returns an error when loading fails or the path does not resolve to exactly
/// one one-dimensional spectrum.
pub fn load_spectrum_1d_with_source(path: impl AsRef<Path>) -> Result<(Spectrum1D, LoadedSource)> {
    SpectrumBundleLoader::new().read_1d_with_source(path)
}

/// Loads exactly one two-dimensional spectrum from a file or directory path.
///
/// # Errors
///
/// Returns an error when loading fails or the path does not resolve to exactly
/// one two-dimensional spectrum.
pub fn load_spectrum_2d(path: impl AsRef<Path>) -> Result<Spectrum2D> {
    SpectrumBundleLoader::new().read_2d(path)
}

/// Loads exactly one two-dimensional spectrum with source metadata from a file or directory path.
///
/// # Errors
///
/// Returns an error when loading fails or the path does not resolve to exactly
/// one two-dimensional spectrum.
pub fn load_spectrum_2d_with_source(path: impl AsRef<Path>) -> Result<(Spectrum2D, LoadedSource)> {
    SpectrumBundleLoader::new().read_2d_with_source(path)
}

/// Loads exactly one one-dimensional spectrum read with a source format.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching one-dimensional
/// spectrum is not found.
pub fn load_spectrum_1d_by_source_format(
    path: impl AsRef<Path>,
    format: impl AsRef<str>,
) -> Result<Spectrum1D> {
    SpectrumBundleLoader::new().read_1d_by_source_format(path, format)
}

/// Loads exactly one one-dimensional spectrum and source read with a source format.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching one-dimensional
/// spectrum is not found.
pub fn load_spectrum_1d_with_source_by_source_format(
    path: impl AsRef<Path>,
    format: impl AsRef<str>,
) -> Result<(Spectrum1D, LoadedSource)> {
    SpectrumBundleLoader::new().read_1d_with_source_by_source_format(path, format)
}

/// Loads exactly one two-dimensional spectrum read with a source format.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching two-dimensional
/// spectrum is not found.
pub fn load_spectrum_2d_by_source_format(
    path: impl AsRef<Path>,
    format: impl AsRef<str>,
) -> Result<Spectrum2D> {
    SpectrumBundleLoader::new().read_2d_by_source_format(path, format)
}

/// Loads exactly one two-dimensional spectrum and source read with a source format.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching two-dimensional
/// spectrum is not found.
pub fn load_spectrum_2d_with_source_by_source_format(
    path: impl AsRef<Path>,
    format: impl AsRef<str>,
) -> Result<(Spectrum2D, LoadedSource)> {
    SpectrumBundleLoader::new().read_2d_with_source_by_source_format(path, format)
}

/// Loads exactly one one-dimensional spectrum read with a vendor-specific reader.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching one-dimensional
/// spectrum is not found.
pub fn load_spectrum_1d_by_source_vendor(
    path: impl AsRef<Path>,
    vendor: impl AsRef<str>,
) -> Result<Spectrum1D> {
    SpectrumBundleLoader::new().read_1d_by_source_vendor(path, vendor)
}

/// Loads exactly one one-dimensional spectrum and source read with a vendor-specific reader.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching one-dimensional
/// spectrum is not found.
pub fn load_spectrum_1d_with_source_by_source_vendor(
    path: impl AsRef<Path>,
    vendor: impl AsRef<str>,
) -> Result<(Spectrum1D, LoadedSource)> {
    SpectrumBundleLoader::new().read_1d_with_source_by_source_vendor(path, vendor)
}

/// Loads exactly one two-dimensional spectrum read with a vendor-specific reader.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching two-dimensional
/// spectrum is not found.
pub fn load_spectrum_2d_by_source_vendor(
    path: impl AsRef<Path>,
    vendor: impl AsRef<str>,
) -> Result<Spectrum2D> {
    SpectrumBundleLoader::new().read_2d_by_source_vendor(path, vendor)
}

/// Loads exactly one two-dimensional spectrum and source read with a vendor-specific reader.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching two-dimensional
/// spectrum is not found.
pub fn load_spectrum_2d_with_source_by_source_vendor(
    path: impl AsRef<Path>,
    vendor: impl AsRef<str>,
) -> Result<(Spectrum2D, LoadedSource)> {
    SpectrumBundleLoader::new().read_2d_with_source_by_source_vendor(path, vendor)
}

/// Loads exactly one one-dimensional spectrum from a path relative to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or the path does not resolve to exactly
/// one one-dimensional spectrum.
pub fn load_spectrum_1d_relative_to(
    base: impl AsRef<Path>,
    path: impl AsRef<Path>,
) -> Result<Spectrum1D> {
    SpectrumBundleLoader::new().read_1d_relative_to(base, path)
}

/// Loads exactly one one-dimensional spectrum with source metadata from a path relative to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or the path does not resolve to exactly
/// one one-dimensional spectrum.
pub fn load_spectrum_1d_with_source_relative_to(
    base: impl AsRef<Path>,
    path: impl AsRef<Path>,
) -> Result<(Spectrum1D, LoadedSource)> {
    SpectrumBundleLoader::new().read_1d_with_source_relative_to(base, path)
}

/// Loads exactly one two-dimensional spectrum from a path relative to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or the path does not resolve to exactly
/// one two-dimensional spectrum.
pub fn load_spectrum_2d_relative_to(
    base: impl AsRef<Path>,
    path: impl AsRef<Path>,
) -> Result<Spectrum2D> {
    SpectrumBundleLoader::new().read_2d_relative_to(base, path)
}

/// Loads exactly one two-dimensional spectrum with source metadata from a path relative to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or the path does not resolve to exactly
/// one two-dimensional spectrum.
pub fn load_spectrum_2d_with_source_relative_to(
    base: impl AsRef<Path>,
    path: impl AsRef<Path>,
) -> Result<(Spectrum2D, LoadedSource)> {
    SpectrumBundleLoader::new().read_2d_with_source_relative_to(base, path)
}

/// Loads exactly one one-dimensional spectrum from selected file or directory paths.
///
/// # Errors
///
/// Returns an error when loading fails or the selected paths do not resolve to
/// exactly one one-dimensional spectrum.
pub fn load_spectrum_1d_many<I, P>(paths: I) -> Result<Spectrum1D>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_1d_many(paths)
}

/// Loads exactly one one-dimensional spectrum from selected file or directory paths.
///
/// This is a spelling alias for [`load_spectrum_1d_many`].
///
/// # Errors
///
/// Returns an error when loading fails or the selected paths do not resolve to
/// exactly one one-dimensional spectrum.
pub fn load_spectrum_1d_paths<I, P>(paths: I) -> Result<Spectrum1D>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_1d_paths(paths)
}

/// Loads exactly one one-dimensional spectrum with source metadata from selected file or directory paths.
///
/// # Errors
///
/// Returns an error when loading fails or the selected paths do not resolve to
/// exactly one one-dimensional spectrum.
pub fn load_spectrum_1d_many_with_source<I, P>(paths: I) -> Result<(Spectrum1D, LoadedSource)>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_1d_many_with_source(paths)
}

/// Loads exactly one one-dimensional spectrum with source metadata from selected file or directory paths.
///
/// This is a spelling alias for [`load_spectrum_1d_many_with_source`].
///
/// # Errors
///
/// Returns an error when loading fails or the selected paths do not resolve to
/// exactly one one-dimensional spectrum.
pub fn load_spectrum_1d_paths_with_source<I, P>(paths: I) -> Result<(Spectrum1D, LoadedSource)>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_1d_paths_with_source(paths)
}

/// Loads exactly one two-dimensional spectrum from selected file or directory paths.
///
/// # Errors
///
/// Returns an error when loading fails or the selected paths do not resolve to
/// exactly one two-dimensional spectrum.
pub fn load_spectrum_2d_many<I, P>(paths: I) -> Result<Spectrum2D>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_2d_many(paths)
}

/// Loads exactly one two-dimensional spectrum from selected file or directory paths.
///
/// This is a spelling alias for [`load_spectrum_2d_many`].
///
/// # Errors
///
/// Returns an error when loading fails or the selected paths do not resolve to
/// exactly one two-dimensional spectrum.
pub fn load_spectrum_2d_paths<I, P>(paths: I) -> Result<Spectrum2D>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_2d_paths(paths)
}

/// Loads exactly one two-dimensional spectrum with source metadata from selected file or directory paths.
///
/// # Errors
///
/// Returns an error when loading fails or the selected paths do not resolve to
/// exactly one two-dimensional spectrum.
pub fn load_spectrum_2d_many_with_source<I, P>(paths: I) -> Result<(Spectrum2D, LoadedSource)>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_2d_many_with_source(paths)
}

/// Loads exactly one two-dimensional spectrum with source metadata from selected file or directory paths.
///
/// This is a spelling alias for [`load_spectrum_2d_many_with_source`].
///
/// # Errors
///
/// Returns an error when loading fails or the selected paths do not resolve to
/// exactly one two-dimensional spectrum.
pub fn load_spectrum_2d_paths_with_source<I, P>(paths: I) -> Result<(Spectrum2D, LoadedSource)>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_2d_paths_with_source(paths)
}

/// Loads exactly one one-dimensional spectrum from paths relative to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or the selected paths do not resolve to
/// exactly one one-dimensional spectrum.
pub fn load_spectrum_1d_many_relative_to<I, P>(
    base: impl AsRef<Path>,
    paths: I,
) -> Result<Spectrum1D>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_1d_many_relative_to(base, paths)
}

/// Loads exactly one one-dimensional spectrum from paths relative to a base directory.
///
/// This is a spelling alias for [`load_spectrum_1d_many_relative_to`].
///
/// # Errors
///
/// Returns an error when loading fails or the selected paths do not resolve to
/// exactly one one-dimensional spectrum.
pub fn load_spectrum_1d_paths_relative_to<I, P>(
    base: impl AsRef<Path>,
    paths: I,
) -> Result<Spectrum1D>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_1d_paths_relative_to(base, paths)
}

/// Loads exactly one one-dimensional spectrum with source metadata from paths relative to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or the selected paths do not resolve to
/// exactly one one-dimensional spectrum.
pub fn load_spectrum_1d_many_with_source_relative_to<I, P>(
    base: impl AsRef<Path>,
    paths: I,
) -> Result<(Spectrum1D, LoadedSource)>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_1d_many_with_source_relative_to(base, paths)
}

/// Loads exactly one one-dimensional spectrum with source metadata from paths relative to a base directory.
///
/// This is a spelling alias for [`load_spectrum_1d_many_with_source_relative_to`].
///
/// # Errors
///
/// Returns an error when loading fails or the selected paths do not resolve to
/// exactly one one-dimensional spectrum.
pub fn load_spectrum_1d_paths_with_source_relative_to<I, P>(
    base: impl AsRef<Path>,
    paths: I,
) -> Result<(Spectrum1D, LoadedSource)>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_1d_paths_with_source_relative_to(base, paths)
}

/// Loads exactly one two-dimensional spectrum from paths relative to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or the selected paths do not resolve to
/// exactly one two-dimensional spectrum.
pub fn load_spectrum_2d_many_relative_to<I, P>(
    base: impl AsRef<Path>,
    paths: I,
) -> Result<Spectrum2D>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_2d_many_relative_to(base, paths)
}

/// Loads exactly one two-dimensional spectrum from paths relative to a base directory.
///
/// This is a spelling alias for [`load_spectrum_2d_many_relative_to`].
///
/// # Errors
///
/// Returns an error when loading fails or the selected paths do not resolve to
/// exactly one two-dimensional spectrum.
pub fn load_spectrum_2d_paths_relative_to<I, P>(
    base: impl AsRef<Path>,
    paths: I,
) -> Result<Spectrum2D>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_2d_paths_relative_to(base, paths)
}

/// Loads exactly one two-dimensional spectrum with source metadata from paths relative to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or the selected paths do not resolve to
/// exactly one two-dimensional spectrum.
pub fn load_spectrum_2d_many_with_source_relative_to<I, P>(
    base: impl AsRef<Path>,
    paths: I,
) -> Result<(Spectrum2D, LoadedSource)>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_2d_many_with_source_relative_to(base, paths)
}

/// Loads exactly one two-dimensional spectrum with source metadata from paths relative to a base directory.
///
/// This is a spelling alias for [`load_spectrum_2d_many_with_source_relative_to`].
///
/// # Errors
///
/// Returns an error when loading fails or the selected paths do not resolve to
/// exactly one two-dimensional spectrum.
pub fn load_spectrum_2d_paths_with_source_relative_to<I, P>(
    base: impl AsRef<Path>,
    paths: I,
) -> Result<(Spectrum2D, LoadedSource)>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_2d_paths_with_source_relative_to(base, paths)
}
