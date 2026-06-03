//! Short exact path aliases for loaded spectrum bundles.

use std::path::Path;

use rspin_core::{Result, Spectrum1D, Spectrum2D};

use super::{LoadedSource, LoadedSpectrum, SpectrumBundle};

impl SpectrumBundle {
    /// Returns a loaded spectrum by its tracked source path, if present.
    ///
    /// This is a short alias for [`Self::loaded_by_source_path`].
    #[must_use]
    pub fn loaded_by_path(&self, path: impl AsRef<Path>) -> Option<&LoadedSpectrum> {
        self.loaded_by_source_path(path)
    }

    /// Returns one-dimensional spectrum and source by tracked source path, if present.
    ///
    /// This is a short alias for [`Self::loaded_1d_by_source_path`].
    #[must_use]
    pub fn loaded_1d_by_path(
        &self,
        path: impl AsRef<Path>,
    ) -> Option<(&Spectrum1D, &LoadedSource)> {
        self.loaded_1d_by_source_path(path)
    }

    /// Returns two-dimensional spectrum and source by tracked source path, if present.
    ///
    /// This is a short alias for [`Self::loaded_2d_by_source_path`].
    #[must_use]
    pub fn loaded_2d_by_path(
        &self,
        path: impl AsRef<Path>,
    ) -> Option<(&Spectrum2D, &LoadedSource)> {
        self.loaded_2d_by_source_path(path)
    }

    /// Returns the only one-dimensional spectrum read from a tracked source path.
    ///
    /// This is a short alias for [`Self::only_1d_by_source_path`].
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching one-dimensional spectrum
    /// exists.
    pub fn only_1d_by_path(&self, path: impl AsRef<Path>) -> Result<&Spectrum1D> {
        self.only_1d_by_source_path(path)
    }

    /// Returns the only one-dimensional spectrum and source read from a tracked source path.
    ///
    /// This is a short alias for [`Self::only_loaded_1d_by_source_path`].
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching one-dimensional spectrum
    /// exists.
    pub fn only_loaded_1d_by_path(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<(&Spectrum1D, &LoadedSource)> {
        self.only_loaded_1d_by_source_path(path)
    }

    /// Returns the only two-dimensional spectrum read from a tracked source path.
    ///
    /// This is a short alias for [`Self::only_2d_by_source_path`].
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching two-dimensional spectrum
    /// exists.
    pub fn only_2d_by_path(&self, path: impl AsRef<Path>) -> Result<&Spectrum2D> {
        self.only_2d_by_source_path(path)
    }

    /// Returns the only two-dimensional spectrum and source read from a tracked source path.
    ///
    /// This is a short alias for [`Self::only_loaded_2d_by_source_path`].
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching two-dimensional spectrum
    /// exists.
    pub fn only_loaded_2d_by_path(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<(&Spectrum2D, &LoadedSource)> {
        self.only_loaded_2d_by_source_path(path)
    }

    /// Consumes the bundle and returns the only one-dimensional spectrum read from a tracked source path.
    ///
    /// This is a short alias for [`Self::into_only_1d_by_source_path`].
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching one-dimensional spectrum
    /// exists.
    pub fn into_only_1d_by_path(self, path: impl AsRef<Path>) -> Result<Spectrum1D> {
        self.into_only_1d_by_source_path(path)
    }

    /// Consumes the bundle and returns the only one-dimensional spectrum and source read from a tracked source path.
    ///
    /// This is a short alias for [`Self::into_only_loaded_1d_by_source_path`].
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching one-dimensional spectrum
    /// exists.
    pub fn into_only_loaded_1d_by_path(
        self,
        path: impl AsRef<Path>,
    ) -> Result<(Spectrum1D, LoadedSource)> {
        self.into_only_loaded_1d_by_source_path(path)
    }

    /// Consumes the bundle and returns the only two-dimensional spectrum read from a tracked source path.
    ///
    /// This is a short alias for [`Self::into_only_2d_by_source_path`].
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching two-dimensional spectrum
    /// exists.
    pub fn into_only_2d_by_path(self, path: impl AsRef<Path>) -> Result<Spectrum2D> {
        self.into_only_2d_by_source_path(path)
    }

    /// Consumes the bundle and returns the only two-dimensional spectrum and source read from a tracked source path.
    ///
    /// This is a short alias for [`Self::into_only_loaded_2d_by_source_path`].
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching two-dimensional spectrum
    /// exists.
    pub fn into_only_loaded_2d_by_path(
        self,
        path: impl AsRef<Path>,
    ) -> Result<(Spectrum2D, LoadedSource)> {
        self.into_only_loaded_2d_by_source_path(path)
    }
}
