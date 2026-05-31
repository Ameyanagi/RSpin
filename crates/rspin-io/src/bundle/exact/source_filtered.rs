//! Source-filtered exact single-spectrum readers.

use std::path::Path;

use rspin_core::{Result, Spectrum1D, Spectrum2D};

use crate::bundle::{LoadedSource, SpectrumBundleLoader};

impl SpectrumBundleLoader {
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

    /// Loads exactly one one-dimensional spectrum read with a source format from a path relative to a base directory.
    ///
    /// Source format aliases such as `jdx` and `jdf` are accepted. Other
    /// matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// one-dimensional spectrum is not found.
    pub fn read_1d_by_source_format_relative_to(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
        format: impl AsRef<str>,
    ) -> Result<Spectrum1D> {
        self.read_path_relative_to(base, path)?
            .into_only_1d_by_source_format(format)
    }

    /// Loads exactly one one-dimensional spectrum and source read with a source format from a path relative to a base directory.
    ///
    /// Source format aliases such as `jdx` and `jdf` are accepted. Other
    /// matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// one-dimensional spectrum is not found.
    pub fn read_1d_with_source_by_source_format_relative_to(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
        format: impl AsRef<str>,
    ) -> Result<(Spectrum1D, LoadedSource)> {
        self.read_path_relative_to(base, path)?
            .into_only_loaded_1d_by_source_format(format)
    }

    /// Loads exactly one two-dimensional spectrum read with a source format from a path relative to a base directory.
    ///
    /// Source format aliases such as `jdx` and `jdf` are accepted. Other
    /// matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// two-dimensional spectrum is not found.
    pub fn read_2d_by_source_format_relative_to(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
        format: impl AsRef<str>,
    ) -> Result<Spectrum2D> {
        self.read_path_relative_to(base, path)?
            .into_only_2d_by_source_format(format)
    }

    /// Loads exactly one two-dimensional spectrum and source read with a source format from a path relative to a base directory.
    ///
    /// Source format aliases such as `jdx` and `jdf` are accepted. Other
    /// matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// two-dimensional spectrum is not found.
    pub fn read_2d_with_source_by_source_format_relative_to(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
        format: impl AsRef<str>,
    ) -> Result<(Spectrum2D, LoadedSource)> {
        self.read_path_relative_to(base, path)?
            .into_only_loaded_2d_by_source_format(format)
    }

    /// Loads exactly one one-dimensional spectrum read with a vendor-specific reader from a path relative to a base directory.
    ///
    /// Vendor aliases such as `agilent` and `varian` are accepted. Other
    /// matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// one-dimensional spectrum is not found.
    pub fn read_1d_by_source_vendor_relative_to(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
        vendor: impl AsRef<str>,
    ) -> Result<Spectrum1D> {
        self.read_path_relative_to(base, path)?
            .into_only_1d_by_source_vendor(vendor)
    }

    /// Loads exactly one one-dimensional spectrum and source read with a vendor-specific reader from a path relative to a base directory.
    ///
    /// Vendor aliases such as `agilent` and `varian` are accepted. Other
    /// matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// one-dimensional spectrum is not found.
    pub fn read_1d_with_source_by_source_vendor_relative_to(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
        vendor: impl AsRef<str>,
    ) -> Result<(Spectrum1D, LoadedSource)> {
        self.read_path_relative_to(base, path)?
            .into_only_loaded_1d_by_source_vendor(vendor)
    }

    /// Loads exactly one two-dimensional spectrum read with a vendor-specific reader from a path relative to a base directory.
    ///
    /// Vendor aliases such as `agilent` and `varian` are accepted. Other
    /// matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// two-dimensional spectrum is not found.
    pub fn read_2d_by_source_vendor_relative_to(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
        vendor: impl AsRef<str>,
    ) -> Result<Spectrum2D> {
        self.read_path_relative_to(base, path)?
            .into_only_2d_by_source_vendor(vendor)
    }

    /// Loads exactly one two-dimensional spectrum and source read with a vendor-specific reader from a path relative to a base directory.
    ///
    /// Vendor aliases such as `agilent` and `varian` are accepted. Other
    /// matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// two-dimensional spectrum is not found.
    pub fn read_2d_with_source_by_source_vendor_relative_to(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
        vendor: impl AsRef<str>,
    ) -> Result<(Spectrum2D, LoadedSource)> {
        self.read_path_relative_to(base, path)?
            .into_only_loaded_2d_by_source_vendor(vendor)
    }
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

/// Loads exactly one one-dimensional spectrum read with a source format from a path relative to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching one-dimensional
/// spectrum is not found.
pub fn load_spectrum_1d_by_source_format_relative_to(
    base: impl AsRef<Path>,
    path: impl AsRef<Path>,
    format: impl AsRef<str>,
) -> Result<Spectrum1D> {
    SpectrumBundleLoader::new().read_1d_by_source_format_relative_to(base, path, format)
}

/// Loads exactly one one-dimensional spectrum and source read with a source format from a path relative to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching one-dimensional
/// spectrum is not found.
pub fn load_spectrum_1d_with_source_by_source_format_relative_to(
    base: impl AsRef<Path>,
    path: impl AsRef<Path>,
    format: impl AsRef<str>,
) -> Result<(Spectrum1D, LoadedSource)> {
    SpectrumBundleLoader::new().read_1d_with_source_by_source_format_relative_to(base, path, format)
}

/// Loads exactly one two-dimensional spectrum read with a source format from a path relative to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching two-dimensional
/// spectrum is not found.
pub fn load_spectrum_2d_by_source_format_relative_to(
    base: impl AsRef<Path>,
    path: impl AsRef<Path>,
    format: impl AsRef<str>,
) -> Result<Spectrum2D> {
    SpectrumBundleLoader::new().read_2d_by_source_format_relative_to(base, path, format)
}

/// Loads exactly one two-dimensional spectrum and source read with a source format from a path relative to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching two-dimensional
/// spectrum is not found.
pub fn load_spectrum_2d_with_source_by_source_format_relative_to(
    base: impl AsRef<Path>,
    path: impl AsRef<Path>,
    format: impl AsRef<str>,
) -> Result<(Spectrum2D, LoadedSource)> {
    SpectrumBundleLoader::new().read_2d_with_source_by_source_format_relative_to(base, path, format)
}

/// Loads exactly one one-dimensional spectrum read with a vendor-specific reader from a path relative to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching one-dimensional
/// spectrum is not found.
pub fn load_spectrum_1d_by_source_vendor_relative_to(
    base: impl AsRef<Path>,
    path: impl AsRef<Path>,
    vendor: impl AsRef<str>,
) -> Result<Spectrum1D> {
    SpectrumBundleLoader::new().read_1d_by_source_vendor_relative_to(base, path, vendor)
}

/// Loads exactly one one-dimensional spectrum and source read with a vendor-specific reader from a path relative to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching one-dimensional
/// spectrum is not found.
pub fn load_spectrum_1d_with_source_by_source_vendor_relative_to(
    base: impl AsRef<Path>,
    path: impl AsRef<Path>,
    vendor: impl AsRef<str>,
) -> Result<(Spectrum1D, LoadedSource)> {
    SpectrumBundleLoader::new().read_1d_with_source_by_source_vendor_relative_to(base, path, vendor)
}

/// Loads exactly one two-dimensional spectrum read with a vendor-specific reader from a path relative to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching two-dimensional
/// spectrum is not found.
pub fn load_spectrum_2d_by_source_vendor_relative_to(
    base: impl AsRef<Path>,
    path: impl AsRef<Path>,
    vendor: impl AsRef<str>,
) -> Result<Spectrum2D> {
    SpectrumBundleLoader::new().read_2d_by_source_vendor_relative_to(base, path, vendor)
}

/// Loads exactly one two-dimensional spectrum and source read with a vendor-specific reader from a path relative to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching two-dimensional
/// spectrum is not found.
pub fn load_spectrum_2d_with_source_by_source_vendor_relative_to(
    base: impl AsRef<Path>,
    path: impl AsRef<Path>,
    vendor: impl AsRef<str>,
) -> Result<(Spectrum2D, LoadedSource)> {
    SpectrumBundleLoader::new().read_2d_with_source_by_source_vendor_relative_to(base, path, vendor)
}
