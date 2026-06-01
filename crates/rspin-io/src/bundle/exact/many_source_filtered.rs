//! Source-filtered exact single-spectrum readers for selected path sets.

use std::path::Path;

use rspin_core::{Result, Spectrum1D, Spectrum2D};

use crate::bundle::{LoadedSource, SpectrumBundleLoader};

impl SpectrumBundleLoader {
    /// Loads exactly one one-dimensional spectrum read with a source format from selected paths.
    ///
    /// Source format aliases such as `jdx`, `jdf`, and `varian fid` are
    /// accepted. Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// one-dimensional spectrum is not found.
    pub fn read_1d_many_by_source_format<I, P>(
        &self,
        paths: I,
        format: impl AsRef<str>,
    ) -> Result<Spectrum1D>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths(paths)?
            .into_only_1d_by_source_format(format)
    }

    /// Loads exactly one one-dimensional spectrum and source read with a source format from selected paths.
    ///
    /// Source format aliases such as `jdx`, `jdf`, and `varian fid` are
    /// accepted. Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// one-dimensional spectrum is not found.
    pub fn read_1d_many_with_source_by_source_format<I, P>(
        &self,
        paths: I,
        format: impl AsRef<str>,
    ) -> Result<(Spectrum1D, LoadedSource)>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths(paths)?
            .into_only_loaded_1d_by_source_format(format)
    }

    /// Loads exactly one two-dimensional spectrum read with a source format from selected paths.
    ///
    /// Source format aliases such as `jdx`, `jdf`, and `varian fid` are
    /// accepted. Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// two-dimensional spectrum is not found.
    pub fn read_2d_many_by_source_format<I, P>(
        &self,
        paths: I,
        format: impl AsRef<str>,
    ) -> Result<Spectrum2D>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths(paths)?
            .into_only_2d_by_source_format(format)
    }

    /// Loads exactly one two-dimensional spectrum and source read with a source format from selected paths.
    ///
    /// Source format aliases such as `jdx`, `jdf`, and `varian fid` are
    /// accepted. Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// two-dimensional spectrum is not found.
    pub fn read_2d_many_with_source_by_source_format<I, P>(
        &self,
        paths: I,
        format: impl AsRef<str>,
    ) -> Result<(Spectrum2D, LoadedSource)>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths(paths)?
            .into_only_loaded_2d_by_source_format(format)
    }

    /// Loads exactly one one-dimensional spectrum read with a vendor-specific reader from selected paths.
    ///
    /// Vendor aliases such as `agilent` and `varian` are accepted. Other
    /// matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// one-dimensional spectrum is not found.
    pub fn read_1d_many_by_source_vendor<I, P>(
        &self,
        paths: I,
        vendor: impl AsRef<str>,
    ) -> Result<Spectrum1D>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths(paths)?
            .into_only_1d_by_source_vendor(vendor)
    }

    /// Loads exactly one one-dimensional spectrum and source read with a vendor-specific reader from selected paths.
    ///
    /// Vendor aliases such as `agilent` and `varian` are accepted. Other
    /// matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// one-dimensional spectrum is not found.
    pub fn read_1d_many_with_source_by_source_vendor<I, P>(
        &self,
        paths: I,
        vendor: impl AsRef<str>,
    ) -> Result<(Spectrum1D, LoadedSource)>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths(paths)?
            .into_only_loaded_1d_by_source_vendor(vendor)
    }

    /// Loads exactly one two-dimensional spectrum read with a vendor-specific reader from selected paths.
    ///
    /// Vendor aliases such as `agilent` and `varian` are accepted. Other
    /// matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// two-dimensional spectrum is not found.
    pub fn read_2d_many_by_source_vendor<I, P>(
        &self,
        paths: I,
        vendor: impl AsRef<str>,
    ) -> Result<Spectrum2D>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths(paths)?
            .into_only_2d_by_source_vendor(vendor)
    }

    /// Loads exactly one two-dimensional spectrum and source read with a vendor-specific reader from selected paths.
    ///
    /// Vendor aliases such as `agilent` and `varian` are accepted. Other
    /// matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// two-dimensional spectrum is not found.
    pub fn read_2d_many_with_source_by_source_vendor<I, P>(
        &self,
        paths: I,
        vendor: impl AsRef<str>,
    ) -> Result<(Spectrum2D, LoadedSource)>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths(paths)?
            .into_only_loaded_2d_by_source_vendor(vendor)
    }

    /// Loads exactly one one-dimensional spectrum read from a tracked source path in selected paths.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// one-dimensional spectrum is not found.
    pub fn read_1d_many_by_source_path<I, P>(
        &self,
        paths: I,
        source_path: impl AsRef<Path>,
    ) -> Result<Spectrum1D>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths(paths)?
            .into_only_1d_by_source_path(source_path)
    }

    /// Loads exactly one one-dimensional spectrum and source read from a tracked source path in selected paths.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// one-dimensional spectrum is not found.
    pub fn read_1d_many_with_source_by_source_path<I, P>(
        &self,
        paths: I,
        source_path: impl AsRef<Path>,
    ) -> Result<(Spectrum1D, LoadedSource)>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths(paths)?
            .into_only_loaded_1d_by_source_path(source_path)
    }

    /// Loads exactly one two-dimensional spectrum read from a tracked source path in selected paths.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// two-dimensional spectrum is not found.
    pub fn read_2d_many_by_source_path<I, P>(
        &self,
        paths: I,
        source_path: impl AsRef<Path>,
    ) -> Result<Spectrum2D>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths(paths)?
            .into_only_2d_by_source_path(source_path)
    }

    /// Loads exactly one two-dimensional spectrum and source read from a tracked source path in selected paths.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// two-dimensional spectrum is not found.
    pub fn read_2d_many_with_source_by_source_path<I, P>(
        &self,
        paths: I,
        source_path: impl AsRef<Path>,
    ) -> Result<(Spectrum2D, LoadedSource)>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths(paths)?
            .into_only_loaded_2d_by_source_path(source_path)
    }

    /// Loads exactly one one-dimensional spectrum read with a source format from selected paths relative to a base directory.
    ///
    /// Source format aliases such as `jdx`, `jdf`, and `varian fid` are
    /// accepted. Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// one-dimensional spectrum is not found.
    pub fn read_1d_many_by_source_format_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
        format: impl AsRef<str>,
    ) -> Result<Spectrum1D>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths_relative_to(base, paths)?
            .into_only_1d_by_source_format(format)
    }

    /// Loads exactly one one-dimensional spectrum and source read with a source format from selected paths relative to a base directory.
    ///
    /// Source format aliases such as `jdx`, `jdf`, and `varian fid` are
    /// accepted. Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// one-dimensional spectrum is not found.
    pub fn read_1d_many_with_source_by_source_format_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
        format: impl AsRef<str>,
    ) -> Result<(Spectrum1D, LoadedSource)>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths_relative_to(base, paths)?
            .into_only_loaded_1d_by_source_format(format)
    }

    /// Loads exactly one two-dimensional spectrum read with a source format from selected paths relative to a base directory.
    ///
    /// Source format aliases such as `jdx`, `jdf`, and `varian fid` are
    /// accepted. Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// two-dimensional spectrum is not found.
    pub fn read_2d_many_by_source_format_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
        format: impl AsRef<str>,
    ) -> Result<Spectrum2D>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths_relative_to(base, paths)?
            .into_only_2d_by_source_format(format)
    }

    /// Loads exactly one two-dimensional spectrum and source read with a source format from selected paths relative to a base directory.
    ///
    /// Source format aliases such as `jdx`, `jdf`, and `varian fid` are
    /// accepted. Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// two-dimensional spectrum is not found.
    pub fn read_2d_many_with_source_by_source_format_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
        format: impl AsRef<str>,
    ) -> Result<(Spectrum2D, LoadedSource)>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths_relative_to(base, paths)?
            .into_only_loaded_2d_by_source_format(format)
    }

    /// Loads exactly one one-dimensional spectrum read with a vendor-specific reader from selected paths relative to a base directory.
    ///
    /// Vendor aliases such as `agilent` and `varian` are accepted. Other
    /// matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// one-dimensional spectrum is not found.
    pub fn read_1d_many_by_source_vendor_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
        vendor: impl AsRef<str>,
    ) -> Result<Spectrum1D>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths_relative_to(base, paths)?
            .into_only_1d_by_source_vendor(vendor)
    }

    /// Loads exactly one one-dimensional spectrum and source read with a vendor-specific reader from selected paths relative to a base directory.
    ///
    /// Vendor aliases such as `agilent` and `varian` are accepted. Other
    /// matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// one-dimensional spectrum is not found.
    pub fn read_1d_many_with_source_by_source_vendor_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
        vendor: impl AsRef<str>,
    ) -> Result<(Spectrum1D, LoadedSource)>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths_relative_to(base, paths)?
            .into_only_loaded_1d_by_source_vendor(vendor)
    }

    /// Loads exactly one two-dimensional spectrum read with a vendor-specific reader from selected paths relative to a base directory.
    ///
    /// Vendor aliases such as `agilent` and `varian` are accepted. Other
    /// matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// two-dimensional spectrum is not found.
    pub fn read_2d_many_by_source_vendor_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
        vendor: impl AsRef<str>,
    ) -> Result<Spectrum2D>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths_relative_to(base, paths)?
            .into_only_2d_by_source_vendor(vendor)
    }

    /// Loads exactly one two-dimensional spectrum and source read with a vendor-specific reader from selected paths relative to a base directory.
    ///
    /// Vendor aliases such as `agilent` and `varian` are accepted. Other
    /// matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// two-dimensional spectrum is not found.
    pub fn read_2d_many_with_source_by_source_vendor_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
        vendor: impl AsRef<str>,
    ) -> Result<(Spectrum2D, LoadedSource)>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths_relative_to(base, paths)?
            .into_only_loaded_2d_by_source_vendor(vendor)
    }

    /// Loads exactly one one-dimensional spectrum read from a tracked source path in selected paths relative to a base directory.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// one-dimensional spectrum is not found.
    pub fn read_1d_many_by_source_path_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
        source_path: impl AsRef<Path>,
    ) -> Result<Spectrum1D>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths_relative_to(base, paths)?
            .into_only_1d_by_source_path(source_path)
    }

    /// Loads exactly one one-dimensional spectrum and source read from a tracked source path in selected paths relative to a base directory.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// one-dimensional spectrum is not found.
    pub fn read_1d_many_with_source_by_source_path_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
        source_path: impl AsRef<Path>,
    ) -> Result<(Spectrum1D, LoadedSource)>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths_relative_to(base, paths)?
            .into_only_loaded_1d_by_source_path(source_path)
    }

    /// Loads exactly one two-dimensional spectrum read from a tracked source path in selected paths relative to a base directory.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// two-dimensional spectrum is not found.
    pub fn read_2d_many_by_source_path_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
        source_path: impl AsRef<Path>,
    ) -> Result<Spectrum2D>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths_relative_to(base, paths)?
            .into_only_2d_by_source_path(source_path)
    }

    /// Loads exactly one two-dimensional spectrum and source read from a tracked source path in selected paths relative to a base directory.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// two-dimensional spectrum is not found.
    pub fn read_2d_many_with_source_by_source_path_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
        source_path: impl AsRef<Path>,
    ) -> Result<(Spectrum2D, LoadedSource)>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths_relative_to(base, paths)?
            .into_only_loaded_2d_by_source_path(source_path)
    }
}

/// Loads exactly one one-dimensional spectrum read with a source format from selected paths.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching one-dimensional
/// spectrum is not found.
pub fn load_spectrum_1d_many_by_source_format<I, P>(
    paths: I,
    format: impl AsRef<str>,
) -> Result<Spectrum1D>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_1d_many_by_source_format(paths, format)
}

/// Loads exactly one one-dimensional spectrum and source read with a source format from selected paths.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching one-dimensional
/// spectrum is not found.
pub fn load_spectrum_1d_many_with_source_by_source_format<I, P>(
    paths: I,
    format: impl AsRef<str>,
) -> Result<(Spectrum1D, LoadedSource)>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_1d_many_with_source_by_source_format(paths, format)
}

/// Loads exactly one two-dimensional spectrum read with a source format from selected paths.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching two-dimensional
/// spectrum is not found.
pub fn load_spectrum_2d_many_by_source_format<I, P>(
    paths: I,
    format: impl AsRef<str>,
) -> Result<Spectrum2D>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_2d_many_by_source_format(paths, format)
}

/// Loads exactly one two-dimensional spectrum and source read with a source format from selected paths.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching two-dimensional
/// spectrum is not found.
pub fn load_spectrum_2d_many_with_source_by_source_format<I, P>(
    paths: I,
    format: impl AsRef<str>,
) -> Result<(Spectrum2D, LoadedSource)>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_2d_many_with_source_by_source_format(paths, format)
}

/// Loads exactly one one-dimensional spectrum read with a vendor-specific reader from selected paths.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching one-dimensional
/// spectrum is not found.
pub fn load_spectrum_1d_many_by_source_vendor<I, P>(
    paths: I,
    vendor: impl AsRef<str>,
) -> Result<Spectrum1D>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_1d_many_by_source_vendor(paths, vendor)
}

/// Loads exactly one one-dimensional spectrum and source read with a vendor-specific reader from selected paths.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching one-dimensional
/// spectrum is not found.
pub fn load_spectrum_1d_many_with_source_by_source_vendor<I, P>(
    paths: I,
    vendor: impl AsRef<str>,
) -> Result<(Spectrum1D, LoadedSource)>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_1d_many_with_source_by_source_vendor(paths, vendor)
}

/// Loads exactly one two-dimensional spectrum read with a vendor-specific reader from selected paths.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching two-dimensional
/// spectrum is not found.
pub fn load_spectrum_2d_many_by_source_vendor<I, P>(
    paths: I,
    vendor: impl AsRef<str>,
) -> Result<Spectrum2D>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_2d_many_by_source_vendor(paths, vendor)
}

/// Loads exactly one two-dimensional spectrum and source read with a vendor-specific reader from selected paths.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching two-dimensional
/// spectrum is not found.
pub fn load_spectrum_2d_many_with_source_by_source_vendor<I, P>(
    paths: I,
    vendor: impl AsRef<str>,
) -> Result<(Spectrum2D, LoadedSource)>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_2d_many_with_source_by_source_vendor(paths, vendor)
}

/// Loads exactly one one-dimensional spectrum read from a tracked source path in selected paths.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching one-dimensional
/// spectrum is not found.
pub fn load_spectrum_1d_many_by_source_path<I, P>(
    paths: I,
    source_path: impl AsRef<Path>,
) -> Result<Spectrum1D>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_1d_many_by_source_path(paths, source_path)
}

/// Loads exactly one one-dimensional spectrum and source read from a tracked source path in selected paths.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching one-dimensional
/// spectrum is not found.
pub fn load_spectrum_1d_many_with_source_by_source_path<I, P>(
    paths: I,
    source_path: impl AsRef<Path>,
) -> Result<(Spectrum1D, LoadedSource)>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_1d_many_with_source_by_source_path(paths, source_path)
}

/// Loads exactly one two-dimensional spectrum read from a tracked source path in selected paths.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching two-dimensional
/// spectrum is not found.
pub fn load_spectrum_2d_many_by_source_path<I, P>(
    paths: I,
    source_path: impl AsRef<Path>,
) -> Result<Spectrum2D>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_2d_many_by_source_path(paths, source_path)
}

/// Loads exactly one two-dimensional spectrum and source read from a tracked source path in selected paths.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching two-dimensional
/// spectrum is not found.
pub fn load_spectrum_2d_many_with_source_by_source_path<I, P>(
    paths: I,
    source_path: impl AsRef<Path>,
) -> Result<(Spectrum2D, LoadedSource)>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_2d_many_with_source_by_source_path(paths, source_path)
}

/// Loads exactly one one-dimensional spectrum read with a source format from selected paths relative to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching one-dimensional
/// spectrum is not found.
pub fn load_spectrum_1d_many_by_source_format_relative_to<I, P>(
    base: impl AsRef<Path>,
    paths: I,
    format: impl AsRef<str>,
) -> Result<Spectrum1D>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_1d_many_by_source_format_relative_to(base, paths, format)
}

/// Loads exactly one one-dimensional spectrum and source read with a source format from selected paths relative to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching one-dimensional
/// spectrum is not found.
pub fn load_spectrum_1d_many_with_source_by_source_format_relative_to<I, P>(
    base: impl AsRef<Path>,
    paths: I,
    format: impl AsRef<str>,
) -> Result<(Spectrum1D, LoadedSource)>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new()
        .read_1d_many_with_source_by_source_format_relative_to(base, paths, format)
}

/// Loads exactly one two-dimensional spectrum read with a source format from selected paths relative to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching two-dimensional
/// spectrum is not found.
pub fn load_spectrum_2d_many_by_source_format_relative_to<I, P>(
    base: impl AsRef<Path>,
    paths: I,
    format: impl AsRef<str>,
) -> Result<Spectrum2D>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_2d_many_by_source_format_relative_to(base, paths, format)
}

/// Loads exactly one two-dimensional spectrum and source read with a source format from selected paths relative to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching two-dimensional
/// spectrum is not found.
pub fn load_spectrum_2d_many_with_source_by_source_format_relative_to<I, P>(
    base: impl AsRef<Path>,
    paths: I,
    format: impl AsRef<str>,
) -> Result<(Spectrum2D, LoadedSource)>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new()
        .read_2d_many_with_source_by_source_format_relative_to(base, paths, format)
}

/// Loads exactly one one-dimensional spectrum read with a vendor-specific reader from selected paths relative to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching one-dimensional
/// spectrum is not found.
pub fn load_spectrum_1d_many_by_source_vendor_relative_to<I, P>(
    base: impl AsRef<Path>,
    paths: I,
    vendor: impl AsRef<str>,
) -> Result<Spectrum1D>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_1d_many_by_source_vendor_relative_to(base, paths, vendor)
}

/// Loads exactly one one-dimensional spectrum and source read with a vendor-specific reader from selected paths relative to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching one-dimensional
/// spectrum is not found.
pub fn load_spectrum_1d_many_with_source_by_source_vendor_relative_to<I, P>(
    base: impl AsRef<Path>,
    paths: I,
    vendor: impl AsRef<str>,
) -> Result<(Spectrum1D, LoadedSource)>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new()
        .read_1d_many_with_source_by_source_vendor_relative_to(base, paths, vendor)
}

/// Loads exactly one two-dimensional spectrum read with a vendor-specific reader from selected paths relative to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching two-dimensional
/// spectrum is not found.
pub fn load_spectrum_2d_many_by_source_vendor_relative_to<I, P>(
    base: impl AsRef<Path>,
    paths: I,
    vendor: impl AsRef<str>,
) -> Result<Spectrum2D>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_2d_many_by_source_vendor_relative_to(base, paths, vendor)
}

/// Loads exactly one two-dimensional spectrum and source read with a vendor-specific reader from selected paths relative to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching two-dimensional
/// spectrum is not found.
pub fn load_spectrum_2d_many_with_source_by_source_vendor_relative_to<I, P>(
    base: impl AsRef<Path>,
    paths: I,
    vendor: impl AsRef<str>,
) -> Result<(Spectrum2D, LoadedSource)>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new()
        .read_2d_many_with_source_by_source_vendor_relative_to(base, paths, vendor)
}

/// Loads exactly one one-dimensional spectrum read from a tracked source path in selected paths relative to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching one-dimensional
/// spectrum is not found.
pub fn load_spectrum_1d_many_by_source_path_relative_to<I, P>(
    base: impl AsRef<Path>,
    paths: I,
    source_path: impl AsRef<Path>,
) -> Result<Spectrum1D>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_1d_many_by_source_path_relative_to(base, paths, source_path)
}

/// Loads exactly one one-dimensional spectrum and source read from a tracked source path in selected paths relative to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching one-dimensional
/// spectrum is not found.
pub fn load_spectrum_1d_many_with_source_by_source_path_relative_to<I, P>(
    base: impl AsRef<Path>,
    paths: I,
    source_path: impl AsRef<Path>,
) -> Result<(Spectrum1D, LoadedSource)>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_1d_many_with_source_by_source_path_relative_to(
        base,
        paths,
        source_path,
    )
}

/// Loads exactly one two-dimensional spectrum read from a tracked source path in selected paths relative to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching two-dimensional
/// spectrum is not found.
pub fn load_spectrum_2d_many_by_source_path_relative_to<I, P>(
    base: impl AsRef<Path>,
    paths: I,
    source_path: impl AsRef<Path>,
) -> Result<Spectrum2D>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_2d_many_by_source_path_relative_to(base, paths, source_path)
}

/// Loads exactly one two-dimensional spectrum and source read from a tracked source path in selected paths relative to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching two-dimensional
/// spectrum is not found.
pub fn load_spectrum_2d_many_with_source_by_source_path_relative_to<I, P>(
    base: impl AsRef<Path>,
    paths: I,
    source_path: impl AsRef<Path>,
) -> Result<(Spectrum2D, LoadedSource)>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_2d_many_with_source_by_source_path_relative_to(
        base,
        paths,
        source_path,
    )
}
