//! Fallible first-spectrum helpers for bundle inspection workflows.

use std::path::Path;

use rspin_core::{RSpinError, Result, Spectrum1D, Spectrum2D};

use super::{LoadedSource, SpectrumBundle, SpectrumBundleLoader};

impl SpectrumBundle {
    /// Returns the first loaded one-dimensional spectrum.
    ///
    /// # Errors
    ///
    /// Returns an error when the bundle contains no one-dimensional spectra.
    pub fn require_first_1d(&self) -> Result<&Spectrum1D> {
        self.first_1d()
            .ok_or_else(|| first_error("one-dimensional", self.len_1d(), self.len_2d()))
    }

    /// Returns the first loaded one-dimensional spectrum and its source metadata.
    ///
    /// # Errors
    ///
    /// Returns an error when the bundle contains no one-dimensional spectra.
    pub fn require_first_loaded_1d(&self) -> Result<(&Spectrum1D, &LoadedSource)> {
        self.first_loaded_1d()
            .ok_or_else(|| first_error("one-dimensional", self.len_1d(), self.len_2d()))
    }

    /// Returns the first loaded two-dimensional spectrum.
    ///
    /// # Errors
    ///
    /// Returns an error when the bundle contains no two-dimensional spectra.
    pub fn require_first_2d(&self) -> Result<&Spectrum2D> {
        self.first_2d()
            .ok_or_else(|| first_error("two-dimensional", self.len_1d(), self.len_2d()))
    }

    /// Returns the first loaded two-dimensional spectrum and its source metadata.
    ///
    /// # Errors
    ///
    /// Returns an error when the bundle contains no two-dimensional spectra.
    pub fn require_first_loaded_2d(&self) -> Result<(&Spectrum2D, &LoadedSource)> {
        self.first_loaded_2d()
            .ok_or_else(|| first_error("two-dimensional", self.len_1d(), self.len_2d()))
    }

    /// Consumes the bundle and returns the first loaded one-dimensional spectrum.
    ///
    /// # Errors
    ///
    /// Returns an error when the bundle contains no one-dimensional spectra.
    pub fn into_first_1d(self) -> Result<Spectrum1D> {
        let one_d = self.len_1d();
        let two_d = self.len_2d();
        self.into_loaded_1d()
            .into_iter()
            .next()
            .map(|(spectrum, _)| spectrum)
            .ok_or_else(|| first_error("one-dimensional", one_d, two_d))
    }

    /// Consumes the bundle and returns the first loaded one-dimensional spectrum and source metadata.
    ///
    /// # Errors
    ///
    /// Returns an error when the bundle contains no one-dimensional spectra.
    pub fn into_first_loaded_1d(self) -> Result<(Spectrum1D, LoadedSource)> {
        let one_d = self.len_1d();
        let two_d = self.len_2d();
        self.into_loaded_1d()
            .into_iter()
            .next()
            .ok_or_else(|| first_error("one-dimensional", one_d, two_d))
    }

    /// Consumes the bundle and returns the first loaded two-dimensional spectrum.
    ///
    /// # Errors
    ///
    /// Returns an error when the bundle contains no two-dimensional spectra.
    pub fn into_first_2d(self) -> Result<Spectrum2D> {
        let one_d = self.len_1d();
        let two_d = self.len_2d();
        self.into_loaded_2d()
            .into_iter()
            .next()
            .map(|(spectrum, _)| spectrum)
            .ok_or_else(|| first_error("two-dimensional", one_d, two_d))
    }

    /// Consumes the bundle and returns the first loaded two-dimensional spectrum and source metadata.
    ///
    /// # Errors
    ///
    /// Returns an error when the bundle contains no two-dimensional spectra.
    pub fn into_first_loaded_2d(self) -> Result<(Spectrum2D, LoadedSource)> {
        let one_d = self.len_1d();
        let two_d = self.len_2d();
        self.into_loaded_2d()
            .into_iter()
            .next()
            .ok_or_else(|| first_error("two-dimensional", one_d, two_d))
    }
}

impl SpectrumBundleLoader {
    /// Loads a file or directory and returns the first one-dimensional spectrum.
    ///
    /// This is for quick inspection workflows. Use [`Self::read_1d`] when the
    /// input must resolve to exactly one one-dimensional spectrum.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or no one-dimensional spectrum is found.
    pub fn read_first_1d(&self, path: impl AsRef<Path>) -> Result<Spectrum1D> {
        self.read_path(path)?.into_first_1d()
    }

    /// Loads a file or directory and returns the first one-dimensional spectrum with source metadata.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or no one-dimensional spectrum is found.
    pub fn read_first_1d_with_source(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<(Spectrum1D, LoadedSource)> {
        self.read_path(path)?.into_first_loaded_1d()
    }

    /// Loads a file or directory and returns the first two-dimensional spectrum.
    ///
    /// This is for quick inspection workflows. Use [`Self::read_2d`] when the
    /// input must resolve to exactly one two-dimensional spectrum.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or no two-dimensional spectrum is found.
    pub fn read_first_2d(&self, path: impl AsRef<Path>) -> Result<Spectrum2D> {
        self.read_path(path)?.into_first_2d()
    }

    /// Loads a file or directory and returns the first two-dimensional spectrum with source metadata.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or no two-dimensional spectrum is found.
    pub fn read_first_2d_with_source(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<(Spectrum2D, LoadedSource)> {
        self.read_path(path)?.into_first_loaded_2d()
    }

    /// Loads one selected path relative to a base directory and returns the first one-dimensional spectrum.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or no one-dimensional spectrum is found.
    pub fn read_first_1d_relative_to(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
    ) -> Result<Spectrum1D> {
        self.read_path_relative_to(base, path)?.into_first_1d()
    }

    /// Loads one selected path relative to a base directory and returns the first one-dimensional spectrum with source metadata.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or no one-dimensional spectrum is found.
    pub fn read_first_1d_with_source_relative_to(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
    ) -> Result<(Spectrum1D, LoadedSource)> {
        self.read_path_relative_to(base, path)?
            .into_first_loaded_1d()
    }

    /// Loads one selected path relative to a base directory and returns the first two-dimensional spectrum.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or no two-dimensional spectrum is found.
    pub fn read_first_2d_relative_to(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
    ) -> Result<Spectrum2D> {
        self.read_path_relative_to(base, path)?.into_first_2d()
    }

    /// Loads one selected path relative to a base directory and returns the first two-dimensional spectrum with source metadata.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or no two-dimensional spectrum is found.
    pub fn read_first_2d_with_source_relative_to(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
    ) -> Result<(Spectrum2D, LoadedSource)> {
        self.read_path_relative_to(base, path)?
            .into_first_loaded_2d()
    }

    /// Loads selected paths and returns the first one-dimensional spectrum.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or no one-dimensional spectrum is found.
    pub fn read_first_1d_many<I, P>(&self, paths: I) -> Result<Spectrum1D>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths(paths)?.into_first_1d()
    }

    /// Loads selected paths and returns the first one-dimensional spectrum with source metadata.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or no one-dimensional spectrum is found.
    pub fn read_first_1d_many_with_source<I, P>(
        &self,
        paths: I,
    ) -> Result<(Spectrum1D, LoadedSource)>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths(paths)?.into_first_loaded_1d()
    }

    /// Loads selected paths and returns the first two-dimensional spectrum.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or no two-dimensional spectrum is found.
    pub fn read_first_2d_many<I, P>(&self, paths: I) -> Result<Spectrum2D>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths(paths)?.into_first_2d()
    }

    /// Loads selected paths and returns the first two-dimensional spectrum with source metadata.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or no two-dimensional spectrum is found.
    pub fn read_first_2d_many_with_source<I, P>(
        &self,
        paths: I,
    ) -> Result<(Spectrum2D, LoadedSource)>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths(paths)?.into_first_loaded_2d()
    }

    /// Loads selected paths relative to a base directory and returns the first one-dimensional spectrum.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or no one-dimensional spectrum is found.
    pub fn read_first_1d_many_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
    ) -> Result<Spectrum1D>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths_relative_to(base, paths)?.into_first_1d()
    }

    /// Loads selected paths relative to a base directory and returns the first one-dimensional spectrum with source metadata.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or no one-dimensional spectrum is found.
    pub fn read_first_1d_many_with_source_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
    ) -> Result<(Spectrum1D, LoadedSource)>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths_relative_to(base, paths)?
            .into_first_loaded_1d()
    }

    /// Loads selected paths relative to a base directory and returns the first two-dimensional spectrum.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or no two-dimensional spectrum is found.
    pub fn read_first_2d_many_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
    ) -> Result<Spectrum2D>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths_relative_to(base, paths)?.into_first_2d()
    }

    /// Loads selected paths relative to a base directory and returns the first two-dimensional spectrum with source metadata.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or no two-dimensional spectrum is found.
    pub fn read_first_2d_many_with_source_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
    ) -> Result<(Spectrum2D, LoadedSource)>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths_relative_to(base, paths)?
            .into_first_loaded_2d()
    }
}

fn first_error(expected: &'static str, one_d: usize, two_d: usize) -> RSpinError {
    RSpinError::Parse {
        format: "spectrum bundle",
        message: format!(
            "expected at least one {expected} spectrum, found {one_d} one-dimensional and {two_d} two-dimensional spectra"
        ),
    }
}
