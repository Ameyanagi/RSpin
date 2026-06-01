//! Source-data-kind exact single-spectrum readers.

use std::path::Path;

use rspin_core::{Result, Spectrum1D, Spectrum2D};

use crate::bundle::{LoadedSource, LoadedSourceDataKind, SpectrumBundleLoader};

impl SpectrumBundleLoader {
    /// Loads exactly one one-dimensional spectrum read from a source data kind.
    ///
    /// Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// one-dimensional spectrum is not found.
    pub fn read_1d_by_source_data_kind(
        &self,
        path: impl AsRef<Path>,
        data_kind: LoadedSourceDataKind,
    ) -> Result<Spectrum1D> {
        self.read_path(path)?
            .into_only_1d_by_source_data_kind(data_kind)
    }

    /// Loads exactly one one-dimensional spectrum and source read from a source data kind.
    ///
    /// Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// one-dimensional spectrum is not found.
    pub fn read_1d_with_source_by_source_data_kind(
        &self,
        path: impl AsRef<Path>,
        data_kind: LoadedSourceDataKind,
    ) -> Result<(Spectrum1D, LoadedSource)> {
        self.read_path(path)?
            .into_only_loaded_1d_by_source_data_kind(data_kind)
    }

    /// Loads exactly one two-dimensional spectrum read from a source data kind.
    ///
    /// Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// two-dimensional spectrum is not found.
    pub fn read_2d_by_source_data_kind(
        &self,
        path: impl AsRef<Path>,
        data_kind: LoadedSourceDataKind,
    ) -> Result<Spectrum2D> {
        self.read_path(path)?
            .into_only_2d_by_source_data_kind(data_kind)
    }

    /// Loads exactly one two-dimensional spectrum and source read from a source data kind.
    ///
    /// Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// two-dimensional spectrum is not found.
    pub fn read_2d_with_source_by_source_data_kind(
        &self,
        path: impl AsRef<Path>,
        data_kind: LoadedSourceDataKind,
    ) -> Result<(Spectrum2D, LoadedSource)> {
        self.read_path(path)?
            .into_only_loaded_2d_by_source_data_kind(data_kind)
    }

    /// Loads exactly one one-dimensional spectrum read from a source data kind from a path relative to a base directory.
    ///
    /// Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// one-dimensional spectrum is not found.
    pub fn read_1d_by_source_data_kind_relative_to(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
        data_kind: LoadedSourceDataKind,
    ) -> Result<Spectrum1D> {
        self.read_path_relative_to(base, path)?
            .into_only_1d_by_source_data_kind(data_kind)
    }

    /// Loads exactly one one-dimensional spectrum and source read from a source data kind from a path relative to a base directory.
    ///
    /// Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// one-dimensional spectrum is not found.
    pub fn read_1d_with_source_by_source_data_kind_relative_to(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
        data_kind: LoadedSourceDataKind,
    ) -> Result<(Spectrum1D, LoadedSource)> {
        self.read_path_relative_to(base, path)?
            .into_only_loaded_1d_by_source_data_kind(data_kind)
    }

    /// Loads exactly one two-dimensional spectrum read from a source data kind from a path relative to a base directory.
    ///
    /// Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// two-dimensional spectrum is not found.
    pub fn read_2d_by_source_data_kind_relative_to(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
        data_kind: LoadedSourceDataKind,
    ) -> Result<Spectrum2D> {
        self.read_path_relative_to(base, path)?
            .into_only_2d_by_source_data_kind(data_kind)
    }

    /// Loads exactly one two-dimensional spectrum and source read from a source data kind from a path relative to a base directory.
    ///
    /// Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or exactly one matching
    /// two-dimensional spectrum is not found.
    pub fn read_2d_with_source_by_source_data_kind_relative_to(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
        data_kind: LoadedSourceDataKind,
    ) -> Result<(Spectrum2D, LoadedSource)> {
        self.read_path_relative_to(base, path)?
            .into_only_loaded_2d_by_source_data_kind(data_kind)
    }
}

/// Loads exactly one one-dimensional spectrum read from a source data kind.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching one-dimensional
/// spectrum is not found.
pub fn load_spectrum_1d_by_source_data_kind(
    path: impl AsRef<Path>,
    data_kind: LoadedSourceDataKind,
) -> Result<Spectrum1D> {
    SpectrumBundleLoader::new().read_1d_by_source_data_kind(path, data_kind)
}

/// Loads exactly one one-dimensional spectrum and source read from a source data kind.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching one-dimensional
/// spectrum is not found.
pub fn load_spectrum_1d_with_source_by_source_data_kind(
    path: impl AsRef<Path>,
    data_kind: LoadedSourceDataKind,
) -> Result<(Spectrum1D, LoadedSource)> {
    SpectrumBundleLoader::new().read_1d_with_source_by_source_data_kind(path, data_kind)
}

/// Loads exactly one two-dimensional spectrum read from a source data kind.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching two-dimensional
/// spectrum is not found.
pub fn load_spectrum_2d_by_source_data_kind(
    path: impl AsRef<Path>,
    data_kind: LoadedSourceDataKind,
) -> Result<Spectrum2D> {
    SpectrumBundleLoader::new().read_2d_by_source_data_kind(path, data_kind)
}

/// Loads exactly one two-dimensional spectrum and source read from a source data kind.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching two-dimensional
/// spectrum is not found.
pub fn load_spectrum_2d_with_source_by_source_data_kind(
    path: impl AsRef<Path>,
    data_kind: LoadedSourceDataKind,
) -> Result<(Spectrum2D, LoadedSource)> {
    SpectrumBundleLoader::new().read_2d_with_source_by_source_data_kind(path, data_kind)
}

/// Loads exactly one one-dimensional spectrum read from a source data kind from a path relative to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching one-dimensional
/// spectrum is not found.
pub fn load_spectrum_1d_by_source_data_kind_relative_to(
    base: impl AsRef<Path>,
    path: impl AsRef<Path>,
    data_kind: LoadedSourceDataKind,
) -> Result<Spectrum1D> {
    SpectrumBundleLoader::new().read_1d_by_source_data_kind_relative_to(base, path, data_kind)
}

/// Loads exactly one one-dimensional spectrum and source read from a source data kind from a path relative to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching one-dimensional
/// spectrum is not found.
pub fn load_spectrum_1d_with_source_by_source_data_kind_relative_to(
    base: impl AsRef<Path>,
    path: impl AsRef<Path>,
    data_kind: LoadedSourceDataKind,
) -> Result<(Spectrum1D, LoadedSource)> {
    SpectrumBundleLoader::new()
        .read_1d_with_source_by_source_data_kind_relative_to(base, path, data_kind)
}

/// Loads exactly one two-dimensional spectrum read from a source data kind from a path relative to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching two-dimensional
/// spectrum is not found.
pub fn load_spectrum_2d_by_source_data_kind_relative_to(
    base: impl AsRef<Path>,
    path: impl AsRef<Path>,
    data_kind: LoadedSourceDataKind,
) -> Result<Spectrum2D> {
    SpectrumBundleLoader::new().read_2d_by_source_data_kind_relative_to(base, path, data_kind)
}

/// Loads exactly one two-dimensional spectrum and source read from a source data kind from a path relative to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or exactly one matching two-dimensional
/// spectrum is not found.
pub fn load_spectrum_2d_with_source_by_source_data_kind_relative_to(
    base: impl AsRef<Path>,
    path: impl AsRef<Path>,
    data_kind: LoadedSourceDataKind,
) -> Result<(Spectrum2D, LoadedSource)> {
    SpectrumBundleLoader::new()
        .read_2d_with_source_by_source_data_kind_relative_to(base, path, data_kind)
}
