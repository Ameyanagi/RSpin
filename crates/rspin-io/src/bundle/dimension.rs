//! Dimension-specific bundle loading helpers.

use std::path::Path;

use rspin_core::Result;

use super::{SpectrumBundle, SpectrumBundleLoader};

impl SpectrumBundleLoader {
    /// Loads all one-dimensional spectra from a file or directory path as a bundle.
    ///
    /// This preserves the loader's other options and source filters. Use
    /// [`Self::read_1d`] when the input must resolve to exactly one
    /// one-dimensional spectrum.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or no one-dimensional bundle data is found.
    pub fn read_bundle_1d(&self, path: impl AsRef<Path>) -> Result<SpectrumBundle> {
        self.clone().one_d_only().read_path(path)
    }

    /// Loads all two-dimensional spectra from a file or directory path as a bundle.
    ///
    /// This preserves the loader's other options and source filters. Use
    /// [`Self::read_2d`] when the input must resolve to exactly one
    /// two-dimensional spectrum.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or no two-dimensional bundle data is found.
    pub fn read_bundle_2d(&self, path: impl AsRef<Path>) -> Result<SpectrumBundle> {
        self.clone().two_d_only().read_path(path)
    }

    /// Loads all one-dimensional spectra from one selected path relative to a base directory.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or no one-dimensional bundle data is found.
    pub fn read_bundle_1d_relative_to(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
    ) -> Result<SpectrumBundle> {
        self.clone().one_d_only().read_path_relative_to(base, path)
    }

    /// Loads all two-dimensional spectra from one selected path relative to a base directory.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or no two-dimensional bundle data is found.
    pub fn read_bundle_2d_relative_to(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
    ) -> Result<SpectrumBundle> {
        self.clone().two_d_only().read_path_relative_to(base, path)
    }

    /// Loads all one-dimensional spectra from multiple selected paths as one bundle.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or no one-dimensional bundle data is found.
    pub fn read_bundle_1d_many<I, P>(&self, paths: I) -> Result<SpectrumBundle>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.clone().one_d_only().read_paths(paths)
    }

    /// Loads all two-dimensional spectra from multiple selected paths as one bundle.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or no two-dimensional bundle data is found.
    pub fn read_bundle_2d_many<I, P>(&self, paths: I) -> Result<SpectrumBundle>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.clone().two_d_only().read_paths(paths)
    }

    /// Loads all one-dimensional spectra from selected paths relative to a base directory.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or no one-dimensional bundle data is found.
    pub fn read_bundle_1d_many_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
    ) -> Result<SpectrumBundle>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.clone()
            .one_d_only()
            .read_paths_relative_to(base, paths)
    }

    /// Loads all two-dimensional spectra from selected paths relative to a base directory.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or no two-dimensional bundle data is found.
    pub fn read_bundle_2d_many_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
    ) -> Result<SpectrumBundle>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.clone()
            .two_d_only()
            .read_paths_relative_to(base, paths)
    }
}

/// Loads all one-dimensional spectra from a file or directory path as a bundle.
///
/// Use [`super::load_spectrum_1d`] when the input must resolve to exactly one
/// one-dimensional spectrum.
///
/// # Errors
///
/// Returns an error when loading fails or no one-dimensional bundle data is found.
pub fn load_spectra_1d(path: impl AsRef<Path>) -> Result<SpectrumBundle> {
    SpectrumBundleLoader::new().read_bundle_1d(path)
}

/// Loads all two-dimensional spectra from a file or directory path as a bundle.
///
/// Use [`super::load_spectrum_2d`] when the input must resolve to exactly one
/// two-dimensional spectrum.
///
/// # Errors
///
/// Returns an error when loading fails or no two-dimensional bundle data is found.
pub fn load_spectra_2d(path: impl AsRef<Path>) -> Result<SpectrumBundle> {
    SpectrumBundleLoader::new().read_bundle_2d(path)
}

/// Loads all one-dimensional spectra from one selected path relative to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or no one-dimensional bundle data is found.
pub fn load_spectra_1d_relative_to(
    base: impl AsRef<Path>,
    path: impl AsRef<Path>,
) -> Result<SpectrumBundle> {
    SpectrumBundleLoader::new().read_bundle_1d_relative_to(base, path)
}

/// Loads all two-dimensional spectra from one selected path relative to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or no two-dimensional bundle data is found.
pub fn load_spectra_2d_relative_to(
    base: impl AsRef<Path>,
    path: impl AsRef<Path>,
) -> Result<SpectrumBundle> {
    SpectrumBundleLoader::new().read_bundle_2d_relative_to(base, path)
}

/// Loads all one-dimensional spectra from multiple selected paths as one bundle.
///
/// # Errors
///
/// Returns an error when loading fails or no one-dimensional bundle data is found.
pub fn load_spectra_1d_many<I, P>(paths: I) -> Result<SpectrumBundle>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_bundle_1d_many(paths)
}

/// Loads all two-dimensional spectra from multiple selected paths as one bundle.
///
/// # Errors
///
/// Returns an error when loading fails or no two-dimensional bundle data is found.
pub fn load_spectra_2d_many<I, P>(paths: I) -> Result<SpectrumBundle>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_bundle_2d_many(paths)
}

/// Loads all one-dimensional spectra from selected paths relative to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or no one-dimensional bundle data is found.
pub fn load_spectra_1d_many_relative_to<I, P>(
    base: impl AsRef<Path>,
    paths: I,
) -> Result<SpectrumBundle>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_bundle_1d_many_relative_to(base, paths)
}

/// Loads all two-dimensional spectra from selected paths relative to a base directory.
///
/// # Errors
///
/// Returns an error when loading fails or no two-dimensional bundle data is found.
pub fn load_spectra_2d_many_relative_to<I, P>(
    base: impl AsRef<Path>,
    paths: I,
) -> Result<SpectrumBundle>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_bundle_2d_many_relative_to(base, paths)
}
