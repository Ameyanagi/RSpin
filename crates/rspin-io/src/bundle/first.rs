//! Fallible first-spectrum helpers for bundle inspection workflows.

use std::path::Path;

use rspin_core::{RSpinError, Result, Spectrum1D, Spectrum2D};

use super::{
    LoadedSource, LoadedSourceFilter, LoadedSpectrum, SpectrumBundle, SpectrumBundleLoader,
};

/// Loads a file or directory and returns the first spectrum of any supported dimension.
///
/// This is for quick inspection workflows where callers do not want to choose
/// one-dimensional or two-dimensional data up front.
///
/// # Errors
///
/// Returns an error when loading fails or no spectrum is found.
pub fn load_first_spectrum(path: impl AsRef<Path>) -> Result<LoadedSpectrum> {
    SpectrumBundleLoader::new().read_first_spectrum(path)
}

/// Loads one selected path relative to a base directory and returns the first spectrum of any supported dimension.
///
/// # Errors
///
/// Returns an error when loading fails or no spectrum is found.
pub fn load_first_spectrum_relative_to(
    base: impl AsRef<Path>,
    path: impl AsRef<Path>,
) -> Result<LoadedSpectrum> {
    SpectrumBundleLoader::new().read_first_spectrum_relative_to(base, path)
}

/// Loads selected paths and returns the first spectrum of any supported dimension.
///
/// # Errors
///
/// Returns an error when loading fails or no spectrum is found.
pub fn load_first_spectrum_many<I, P>(paths: I) -> Result<LoadedSpectrum>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_first_spectrum_many(paths)
}

/// Loads selected paths relative to a base directory and returns the first spectrum of any supported dimension.
///
/// # Errors
///
/// Returns an error when loading fails or no spectrum is found.
pub fn load_first_spectrum_many_relative_to<I, P>(
    base: impl AsRef<Path>,
    paths: I,
) -> Result<LoadedSpectrum>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_first_spectrum_many_relative_to(base, paths)
}

/// Loads a file or directory and returns the first one-dimensional spectrum.
///
/// This is for quick inspection workflows. Use [`crate::load_spectrum_1d`] when
/// the input must resolve to exactly one one-dimensional spectrum.
///
/// # Errors
///
/// Returns an error when loading fails or no one-dimensional spectrum is found.
pub fn load_first_spectrum_1d(path: impl AsRef<Path>) -> Result<Spectrum1D> {
    SpectrumBundleLoader::new().read_first_1d(path)
}

/// Loads a file or directory and returns the first one-dimensional spectrum with source metadata.
///
/// # Errors
///
/// Returns an error when loading fails or no one-dimensional spectrum is found.
pub fn load_first_spectrum_1d_with_source(
    path: impl AsRef<Path>,
) -> Result<(Spectrum1D, LoadedSource)> {
    SpectrumBundleLoader::new().read_first_1d_with_source(path)
}

/// Loads a file or directory and returns the first two-dimensional spectrum.
///
/// This is for quick inspection workflows. Use [`crate::load_spectrum_2d`] when
/// the input must resolve to exactly one two-dimensional spectrum.
///
/// # Errors
///
/// Returns an error when loading fails or no two-dimensional spectrum is found.
pub fn load_first_spectrum_2d(path: impl AsRef<Path>) -> Result<Spectrum2D> {
    SpectrumBundleLoader::new().read_first_2d(path)
}

/// Loads a file or directory and returns the first two-dimensional spectrum with source metadata.
///
/// # Errors
///
/// Returns an error when loading fails or no two-dimensional spectrum is found.
pub fn load_first_spectrum_2d_with_source(
    path: impl AsRef<Path>,
) -> Result<(Spectrum2D, LoadedSource)> {
    SpectrumBundleLoader::new().read_first_2d_with_source(path)
}

/// Loads one selected path relative to a base directory and returns the first one-dimensional spectrum.
///
/// # Errors
///
/// Returns an error when loading fails or no one-dimensional spectrum is found.
pub fn load_first_spectrum_1d_relative_to(
    base: impl AsRef<Path>,
    path: impl AsRef<Path>,
) -> Result<Spectrum1D> {
    SpectrumBundleLoader::new().read_first_1d_relative_to(base, path)
}

/// Loads one selected path relative to a base directory and returns the first one-dimensional spectrum with source metadata.
///
/// # Errors
///
/// Returns an error when loading fails or no one-dimensional spectrum is found.
pub fn load_first_spectrum_1d_with_source_relative_to(
    base: impl AsRef<Path>,
    path: impl AsRef<Path>,
) -> Result<(Spectrum1D, LoadedSource)> {
    SpectrumBundleLoader::new().read_first_1d_with_source_relative_to(base, path)
}

/// Loads one selected path relative to a base directory and returns the first two-dimensional spectrum.
///
/// # Errors
///
/// Returns an error when loading fails or no two-dimensional spectrum is found.
pub fn load_first_spectrum_2d_relative_to(
    base: impl AsRef<Path>,
    path: impl AsRef<Path>,
) -> Result<Spectrum2D> {
    SpectrumBundleLoader::new().read_first_2d_relative_to(base, path)
}

/// Loads one selected path relative to a base directory and returns the first two-dimensional spectrum with source metadata.
///
/// # Errors
///
/// Returns an error when loading fails or no two-dimensional spectrum is found.
pub fn load_first_spectrum_2d_with_source_relative_to(
    base: impl AsRef<Path>,
    path: impl AsRef<Path>,
) -> Result<(Spectrum2D, LoadedSource)> {
    SpectrumBundleLoader::new().read_first_2d_with_source_relative_to(base, path)
}

/// Loads selected paths and returns the first one-dimensional spectrum.
///
/// # Errors
///
/// Returns an error when loading fails or no one-dimensional spectrum is found.
pub fn load_first_spectrum_1d_many<I, P>(paths: I) -> Result<Spectrum1D>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_first_1d_many(paths)
}

/// Loads selected paths and returns the first one-dimensional spectrum with source metadata.
///
/// # Errors
///
/// Returns an error when loading fails or no one-dimensional spectrum is found.
pub fn load_first_spectrum_1d_many_with_source<I, P>(paths: I) -> Result<(Spectrum1D, LoadedSource)>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_first_1d_many_with_source(paths)
}

/// Loads selected paths and returns the first two-dimensional spectrum.
///
/// # Errors
///
/// Returns an error when loading fails or no two-dimensional spectrum is found.
pub fn load_first_spectrum_2d_many<I, P>(paths: I) -> Result<Spectrum2D>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_first_2d_many(paths)
}

/// Loads selected paths and returns the first two-dimensional spectrum with source metadata.
///
/// # Errors
///
/// Returns an error when loading fails or no two-dimensional spectrum is found.
pub fn load_first_spectrum_2d_many_with_source<I, P>(paths: I) -> Result<(Spectrum2D, LoadedSource)>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_first_2d_many_with_source(paths)
}

/// Loads selected paths relative to a base directory and returns the first one-dimensional spectrum.
///
/// # Errors
///
/// Returns an error when loading fails or no one-dimensional spectrum is found.
pub fn load_first_spectrum_1d_many_relative_to<I, P>(
    base: impl AsRef<Path>,
    paths: I,
) -> Result<Spectrum1D>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_first_1d_many_relative_to(base, paths)
}

/// Loads selected paths relative to a base directory and returns the first one-dimensional spectrum with source metadata.
///
/// # Errors
///
/// Returns an error when loading fails or no one-dimensional spectrum is found.
pub fn load_first_spectrum_1d_many_with_source_relative_to<I, P>(
    base: impl AsRef<Path>,
    paths: I,
) -> Result<(Spectrum1D, LoadedSource)>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_first_1d_many_with_source_relative_to(base, paths)
}

/// Loads selected paths relative to a base directory and returns the first two-dimensional spectrum.
///
/// # Errors
///
/// Returns an error when loading fails or no two-dimensional spectrum is found.
pub fn load_first_spectrum_2d_many_relative_to<I, P>(
    base: impl AsRef<Path>,
    paths: I,
) -> Result<Spectrum2D>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_first_2d_many_relative_to(base, paths)
}

/// Loads selected paths relative to a base directory and returns the first two-dimensional spectrum with source metadata.
///
/// # Errors
///
/// Returns an error when loading fails or no two-dimensional spectrum is found.
pub fn load_first_spectrum_2d_many_with_source_relative_to<I, P>(
    base: impl AsRef<Path>,
    paths: I,
) -> Result<(Spectrum2D, LoadedSource)>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_first_2d_many_with_source_relative_to(base, paths)
}

impl SpectrumBundle {
    /// Returns the first loaded spectrum of any supported dimension.
    #[must_use]
    pub fn first_spectrum(&self) -> Option<&LoadedSpectrum> {
        self.spectra().first()
    }

    /// Returns the first loaded spectrum of any supported dimension.
    ///
    /// # Errors
    ///
    /// Returns an error when the bundle contains no spectra.
    pub fn require_first_spectrum(&self) -> Result<&LoadedSpectrum> {
        self.first_spectrum()
            .ok_or_else(|| first_error("spectrum", self.len_1d(), self.len_2d()))
    }

    /// Returns the first loaded spectrum matching a generic source filter.
    ///
    /// # Errors
    ///
    /// Returns an error when no loaded spectrum matches the source filter.
    pub fn require_first_by_source(
        &self,
        filter: impl Into<LoadedSourceFilter>,
    ) -> Result<&LoadedSpectrum> {
        self.first_by_source(filter)
            .ok_or_else(|| first_error("spectrum", self.len_1d(), self.len_2d()))
    }

    /// Returns the first loaded spectrum matching any generic source filter.
    ///
    /// # Errors
    ///
    /// Returns an error when no loaded spectrum matches the source filters.
    pub fn require_first_by_sources<I, F>(&self, filters: I) -> Result<&LoadedSpectrum>
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>,
    {
        self.first_by_sources(filters)
            .ok_or_else(|| first_error("spectrum", self.len_1d(), self.len_2d()))
    }

    /// Consumes the bundle and returns the first loaded spectrum of any supported dimension.
    ///
    /// # Errors
    ///
    /// Returns an error when the bundle contains no spectra.
    pub fn into_first_spectrum(self) -> Result<LoadedSpectrum> {
        let one_d = self.len_1d();
        let two_d = self.len_2d();
        let (spectra, _, _) = self.into_parts();
        spectra
            .into_iter()
            .next()
            .ok_or_else(|| first_error("spectrum", one_d, two_d))
    }

    /// Consumes the bundle and returns the first loaded spectrum matching a generic source filter.
    ///
    /// # Errors
    ///
    /// Returns an error when no loaded spectrum matches the source filter.
    pub fn into_first_by_source(
        self,
        filter: impl Into<LoadedSourceFilter>,
    ) -> Result<LoadedSpectrum> {
        self.into_first_by_sources([filter.into()])
    }

    /// Consumes the bundle and returns the first loaded spectrum matching any generic source filter.
    ///
    /// # Errors
    ///
    /// Returns an error when no loaded spectrum matches the source filters.
    pub fn into_first_by_sources<I, F>(self, filters: I) -> Result<LoadedSpectrum>
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>,
    {
        let one_d = self.len_1d();
        let two_d = self.len_2d();
        self.into_loaded_by_sources(filters)
            .into_iter()
            .next()
            .ok_or_else(|| first_error("spectrum", one_d, two_d))
    }

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

    /// Returns the first one-dimensional spectrum matching a generic source filter.
    ///
    /// # Errors
    ///
    /// Returns an error when no one-dimensional spectrum matches the source filter.
    pub fn require_first_1d_by_source(
        &self,
        filter: impl Into<LoadedSourceFilter>,
    ) -> Result<&Spectrum1D> {
        self.first_1d_by_source(filter)
            .ok_or_else(|| first_error("one-dimensional", self.len_1d(), self.len_2d()))
    }

    /// Returns the first one-dimensional spectrum and source matching a generic source filter.
    ///
    /// # Errors
    ///
    /// Returns an error when no one-dimensional spectrum matches the source filter.
    pub fn require_first_loaded_1d_by_source(
        &self,
        filter: impl Into<LoadedSourceFilter>,
    ) -> Result<(&Spectrum1D, &LoadedSource)> {
        self.first_loaded_1d_by_source(filter)
            .ok_or_else(|| first_error("one-dimensional", self.len_1d(), self.len_2d()))
    }

    /// Returns the first one-dimensional spectrum matching any generic source filter.
    ///
    /// # Errors
    ///
    /// Returns an error when no one-dimensional spectrum matches the source filters.
    pub fn require_first_1d_by_sources<I, F>(&self, filters: I) -> Result<&Spectrum1D>
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>,
    {
        self.first_1d_by_sources(filters)
            .ok_or_else(|| first_error("one-dimensional", self.len_1d(), self.len_2d()))
    }

    /// Returns the first one-dimensional spectrum and source matching any generic source filter.
    ///
    /// # Errors
    ///
    /// Returns an error when no one-dimensional spectrum matches the source filters.
    pub fn require_first_loaded_1d_by_sources<I, F>(
        &self,
        filters: I,
    ) -> Result<(&Spectrum1D, &LoadedSource)>
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>,
    {
        self.first_loaded_1d_by_sources(filters)
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

    /// Returns the first two-dimensional spectrum matching a generic source filter.
    ///
    /// # Errors
    ///
    /// Returns an error when no two-dimensional spectrum matches the source filter.
    pub fn require_first_2d_by_source(
        &self,
        filter: impl Into<LoadedSourceFilter>,
    ) -> Result<&Spectrum2D> {
        self.first_2d_by_source(filter)
            .ok_or_else(|| first_error("two-dimensional", self.len_1d(), self.len_2d()))
    }

    /// Returns the first two-dimensional spectrum and source matching a generic source filter.
    ///
    /// # Errors
    ///
    /// Returns an error when no two-dimensional spectrum matches the source filter.
    pub fn require_first_loaded_2d_by_source(
        &self,
        filter: impl Into<LoadedSourceFilter>,
    ) -> Result<(&Spectrum2D, &LoadedSource)> {
        self.first_loaded_2d_by_source(filter)
            .ok_or_else(|| first_error("two-dimensional", self.len_1d(), self.len_2d()))
    }

    /// Returns the first two-dimensional spectrum matching any generic source filter.
    ///
    /// # Errors
    ///
    /// Returns an error when no two-dimensional spectrum matches the source filters.
    pub fn require_first_2d_by_sources<I, F>(&self, filters: I) -> Result<&Spectrum2D>
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>,
    {
        self.first_2d_by_sources(filters)
            .ok_or_else(|| first_error("two-dimensional", self.len_1d(), self.len_2d()))
    }

    /// Returns the first two-dimensional spectrum and source matching any generic source filter.
    ///
    /// # Errors
    ///
    /// Returns an error when no two-dimensional spectrum matches the source filters.
    pub fn require_first_loaded_2d_by_sources<I, F>(
        &self,
        filters: I,
    ) -> Result<(&Spectrum2D, &LoadedSource)>
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>,
    {
        self.first_loaded_2d_by_sources(filters)
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

    /// Consumes the bundle and returns the first one-dimensional spectrum matching a generic source filter.
    ///
    /// # Errors
    ///
    /// Returns an error when no one-dimensional spectrum matches the source filter.
    pub fn into_first_1d_by_source(
        self,
        filter: impl Into<LoadedSourceFilter>,
    ) -> Result<Spectrum1D> {
        self.into_first_loaded_1d_by_source(filter)
            .map(|(spectrum, _)| spectrum)
    }

    /// Consumes the bundle and returns the first one-dimensional spectrum and source matching a generic source filter.
    ///
    /// # Errors
    ///
    /// Returns an error when no one-dimensional spectrum matches the source filter.
    pub fn into_first_loaded_1d_by_source(
        self,
        filter: impl Into<LoadedSourceFilter>,
    ) -> Result<(Spectrum1D, LoadedSource)> {
        self.into_first_loaded_1d_by_sources([filter.into()])
    }

    /// Consumes the bundle and returns the first one-dimensional spectrum matching any generic source filter.
    ///
    /// # Errors
    ///
    /// Returns an error when no one-dimensional spectrum matches the source filters.
    pub fn into_first_1d_by_sources<I, F>(self, filters: I) -> Result<Spectrum1D>
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>,
    {
        self.into_first_loaded_1d_by_sources(filters)
            .map(|(spectrum, _)| spectrum)
    }

    /// Consumes the bundle and returns the first one-dimensional spectrum and source matching any generic source filter.
    ///
    /// # Errors
    ///
    /// Returns an error when no one-dimensional spectrum matches the source filters.
    pub fn into_first_loaded_1d_by_sources<I, F>(
        self,
        filters: I,
    ) -> Result<(Spectrum1D, LoadedSource)>
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>,
    {
        let one_d = self.len_1d();
        let two_d = self.len_2d();
        self.into_loaded_1d_by_sources(filters)
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

    /// Consumes the bundle and returns the first two-dimensional spectrum matching a generic source filter.
    ///
    /// # Errors
    ///
    /// Returns an error when no two-dimensional spectrum matches the source filter.
    pub fn into_first_2d_by_source(
        self,
        filter: impl Into<LoadedSourceFilter>,
    ) -> Result<Spectrum2D> {
        self.into_first_loaded_2d_by_source(filter)
            .map(|(spectrum, _)| spectrum)
    }

    /// Consumes the bundle and returns the first two-dimensional spectrum and source matching a generic source filter.
    ///
    /// # Errors
    ///
    /// Returns an error when no two-dimensional spectrum matches the source filter.
    pub fn into_first_loaded_2d_by_source(
        self,
        filter: impl Into<LoadedSourceFilter>,
    ) -> Result<(Spectrum2D, LoadedSource)> {
        self.into_first_loaded_2d_by_sources([filter.into()])
    }

    /// Consumes the bundle and returns the first two-dimensional spectrum matching any generic source filter.
    ///
    /// # Errors
    ///
    /// Returns an error when no two-dimensional spectrum matches the source filters.
    pub fn into_first_2d_by_sources<I, F>(self, filters: I) -> Result<Spectrum2D>
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>,
    {
        self.into_first_loaded_2d_by_sources(filters)
            .map(|(spectrum, _)| spectrum)
    }

    /// Consumes the bundle and returns the first two-dimensional spectrum and source matching any generic source filter.
    ///
    /// # Errors
    ///
    /// Returns an error when no two-dimensional spectrum matches the source filters.
    pub fn into_first_loaded_2d_by_sources<I, F>(
        self,
        filters: I,
    ) -> Result<(Spectrum2D, LoadedSource)>
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>,
    {
        let one_d = self.len_1d();
        let two_d = self.len_2d();
        self.into_loaded_2d_by_sources(filters)
            .into_iter()
            .next()
            .ok_or_else(|| first_error("two-dimensional", one_d, two_d))
    }
}

impl SpectrumBundleLoader {
    /// Loads a file or directory and returns the first spectrum of any supported dimension.
    ///
    /// This is for quick inspection workflows where callers do not want to choose
    /// one-dimensional or two-dimensional data up front.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or no spectrum is found.
    pub fn read_first_spectrum(&self, path: impl AsRef<Path>) -> Result<LoadedSpectrum> {
        self.read_path(path)?.into_first_spectrum()
    }

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

    /// Loads one selected path relative to a base directory and returns the first spectrum of any supported dimension.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or no spectrum is found.
    pub fn read_first_spectrum_relative_to(
        &self,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
    ) -> Result<LoadedSpectrum> {
        self.read_path_relative_to(base, path)?
            .into_first_spectrum()
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

    /// Loads selected paths and returns the first spectrum of any supported dimension.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or no spectrum is found.
    pub fn read_first_spectrum_many<I, P>(&self, paths: I) -> Result<LoadedSpectrum>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths(paths)?.into_first_spectrum()
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

    /// Loads selected paths relative to a base directory and returns the first spectrum of any supported dimension.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails or no spectrum is found.
    pub fn read_first_spectrum_many_relative_to<I, P>(
        &self,
        base: impl AsRef<Path>,
        paths: I,
    ) -> Result<LoadedSpectrum>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.read_paths_relative_to(base, paths)?
            .into_first_spectrum()
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
    let expected = match expected {
        "spectrum" => "spectrum".to_owned(),
        other => format!("{other} spectrum"),
    };
    RSpinError::Parse {
        format: "spectrum bundle",
        message: format!(
            "expected at least one {expected}, found {one_d} one-dimensional and {two_d} two-dimensional spectra"
        ),
    }
}
