//! Spectrum bundle data model and accessors.

use std::path::{Path, PathBuf};

use rspin_core::{Molecule, RSpinError, Result, Spectrum1D, Spectrum2D};
use serde::{Deserialize, Serialize};

use super::{
    LoadedSourceFormat, LoadedSourceVendor, only_error_from_counts, push_source_vendor_count,
    source_format_count_name, source_format_matches, source_vendor_counts_from_format_counts,
    spectrum_dimension_counts,
};

/// Source metadata for a loaded spectrum.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoadedSource {
    /// Relative source path when source tracking is enabled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
    /// Reader format used for this source.
    pub format: String,
}

impl LoadedSource {
    /// Creates source metadata.
    #[must_use]
    pub fn new(path: Option<PathBuf>, format: impl Into<String>) -> Self {
        Self {
            path,
            format: format.into(),
        }
    }

    /// Returns the source path, if source path tracking was enabled.
    #[must_use]
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Returns the reader format used for this source.
    #[must_use]
    pub fn format(&self) -> &str {
        &self.format
    }

    /// Returns the known source format, if this source uses a built-in reader name.
    #[must_use]
    pub fn format_kind(&self) -> Option<LoadedSourceFormat> {
        LoadedSourceFormat::parse(&self.format).ok()
    }

    /// Returns the source vendor family for vendor-specific reader formats.
    #[must_use]
    pub fn vendor(&self) -> Option<LoadedSourceVendor> {
        self.format_kind().and_then(LoadedSourceFormat::vendor)
    }

    /// Returns true when this source was read with a source format.
    #[must_use]
    pub fn is_format(&self, format: impl AsRef<str>) -> bool {
        source_format_matches(self.format(), format.as_ref())
    }

    /// Returns true when this source was read with a vendor-specific reader.
    #[must_use]
    pub fn is_vendor(&self, vendor: impl AsRef<str>) -> bool {
        let Ok(vendor) = LoadedSourceVendor::parse(vendor.as_ref()) else {
            return false;
        };
        self.vendor() == Some(vendor)
    }
}

/// Deterministic count of loaded spectra for one source format.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceFormatCount {
    /// Reader format.
    pub format: String,
    /// Number of loaded spectra with this format.
    pub count: usize,
}

impl SourceFormatCount {
    /// Creates a source format count.
    #[must_use]
    pub fn new(format: impl Into<String>, count: usize) -> Self {
        Self {
            format: format.into(),
            count,
        }
    }

    /// Returns the reader format.
    #[must_use]
    pub fn format(&self) -> &str {
        &self.format
    }

    /// Returns the number of loaded spectra with this format.
    #[must_use]
    pub fn count(&self) -> usize {
        self.count
    }

    /// Returns the known source format, if this count uses a built-in reader name.
    #[must_use]
    pub fn format_kind(&self) -> Option<LoadedSourceFormat> {
        LoadedSourceFormat::parse(&self.format).ok()
    }

    /// Returns the source vendor family for vendor-specific reader formats.
    #[must_use]
    pub fn vendor(&self) -> Option<LoadedSourceVendor> {
        self.format_kind().and_then(LoadedSourceFormat::vendor)
    }
}

/// Deterministic count of loaded spectra for one source vendor family.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceVendorCount {
    /// Source vendor family.
    pub vendor: String,
    /// Number of loaded spectra with this vendor family.
    pub count: usize,
}

impl SourceVendorCount {
    /// Creates a source vendor count.
    #[must_use]
    pub fn new(vendor: impl Into<String>, count: usize) -> Self {
        Self {
            vendor: vendor.into(),
            count,
        }
    }

    /// Returns the source vendor family.
    #[must_use]
    pub fn vendor(&self) -> &str {
        &self.vendor
    }

    /// Returns the number of loaded spectra with this vendor family.
    #[must_use]
    pub fn count(&self) -> usize {
        self.count
    }

    /// Returns the known source vendor, if this count uses a built-in vendor name.
    #[must_use]
    pub fn vendor_kind(&self) -> Option<LoadedSourceVendor> {
        LoadedSourceVendor::parse(&self.vendor).ok()
    }
}

/// Summary counts for a loaded spectrum bundle.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpectrumBundleSummary {
    /// Total number of loaded spectra.
    pub spectra: usize,
    /// Number of loaded one-dimensional spectra.
    pub spectra_1d: usize,
    /// Number of loaded two-dimensional spectra.
    pub spectra_2d: usize,
    /// Number of molecule metadata records.
    pub molecules: usize,
    /// Number of non-fatal loader warnings.
    pub warnings: usize,
    /// Counts of loaded spectra by reader format.
    pub source_formats: Vec<SourceFormatCount>,
    /// Counts of loaded spectra by source vendor family.
    #[serde(default)]
    pub source_vendors: Vec<SourceVendorCount>,
}

impl SpectrumBundleSummary {
    /// Creates bundle summary counts.
    #[must_use]
    pub fn new(
        spectra: usize,
        spectra_1d: usize,
        spectra_2d: usize,
        molecules: usize,
        warnings: usize,
        source_formats: Vec<SourceFormatCount>,
    ) -> Self {
        let source_vendors = source_vendor_counts_from_format_counts(&source_formats);
        Self {
            spectra,
            spectra_1d,
            spectra_2d,
            molecules,
            warnings,
            source_formats,
            source_vendors,
        }
    }

    /// Returns the number of loaded spectra.
    #[must_use]
    pub fn spectra(&self) -> usize {
        self.spectra
    }

    /// Returns the number of loaded one-dimensional spectra.
    #[must_use]
    pub fn spectra_1d(&self) -> usize {
        self.spectra_1d
    }

    /// Returns the number of loaded two-dimensional spectra.
    #[must_use]
    pub fn spectra_2d(&self) -> usize {
        self.spectra_2d
    }

    /// Returns the number of molecule metadata records.
    #[must_use]
    pub fn molecules(&self) -> usize {
        self.molecules
    }

    /// Returns the number of non-fatal loader warnings.
    #[must_use]
    pub fn warnings(&self) -> usize {
        self.warnings
    }

    /// Returns the number of loaded spectra read with a source format.
    #[must_use]
    pub fn source_format_count(&self, format: impl AsRef<str>) -> usize {
        let format = format.as_ref();
        self.source_formats
            .iter()
            .filter(|count| source_format_matches(count.format(), format))
            .map(SourceFormatCount::count)
            .sum()
    }

    /// Returns true when a loaded spectrum was read with a source format.
    #[must_use]
    pub fn has_source_format(&self, format: impl AsRef<str>) -> bool {
        self.source_format_count(format) > 0
    }

    /// Returns the number of loaded spectra read with a vendor-specific reader.
    #[must_use]
    pub fn source_vendor_count(&self, vendor: impl AsRef<str>) -> usize {
        let Ok(vendor) = LoadedSourceVendor::parse(vendor.as_ref()) else {
            return 0;
        };
        self.source_vendor_counts()
            .iter()
            .find(|count| count.vendor_kind() == Some(vendor))
            .map_or(0, SourceVendorCount::count)
    }

    /// Returns true when a loaded spectrum was read with a vendor-specific reader.
    #[must_use]
    pub fn has_source_vendor(&self, vendor: impl AsRef<str>) -> bool {
        self.source_vendor_count(vendor) > 0
    }

    /// Returns deterministic source vendor counts in first-seen order.
    ///
    /// For summaries deserialized from older JSON that does not contain the
    /// `source_vendors` field, counts are reconstructed from `source_formats`.
    #[must_use]
    pub fn source_vendor_counts(&self) -> Vec<SourceVendorCount> {
        if self.source_vendors.is_empty() {
            return source_vendor_counts_from_format_counts(&self.source_formats);
        }
        self.source_vendors.clone()
    }
}

/// A loaded one- or two-dimensional spectrum plus source metadata.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "dimension", rename_all = "snake_case")]
pub enum LoadedSpectrum {
    /// One-dimensional spectrum.
    OneD {
        /// Spectrum payload.
        spectrum: Spectrum1D,
        /// Source metadata.
        source: LoadedSource,
    },
    /// Two-dimensional spectrum.
    TwoD {
        /// Spectrum payload.
        spectrum: Spectrum2D,
        /// Source metadata.
        source: LoadedSource,
    },
}

impl LoadedSpectrum {
    /// Returns true when this entry contains a one-dimensional spectrum.
    #[must_use]
    pub fn is_1d(&self) -> bool {
        matches!(self, Self::OneD { .. })
    }

    /// Returns true when this entry contains a two-dimensional spectrum.
    #[must_use]
    pub fn is_2d(&self) -> bool {
        matches!(self, Self::TwoD { .. })
    }

    /// Returns the source metadata.
    #[must_use]
    pub fn source(&self) -> &LoadedSource {
        match self {
            Self::OneD { source, .. } | Self::TwoD { source, .. } => source,
        }
    }

    /// Returns the one-dimensional spectrum, if present.
    #[must_use]
    pub fn as_1d(&self) -> Option<&Spectrum1D> {
        match self {
            Self::OneD { spectrum, .. } => Some(spectrum),
            Self::TwoD { .. } => None,
        }
    }

    /// Returns the two-dimensional spectrum, if present.
    #[must_use]
    pub fn as_2d(&self) -> Option<&Spectrum2D> {
        match self {
            Self::TwoD { spectrum, .. } => Some(spectrum),
            Self::OneD { .. } => None,
        }
    }

    /// Consumes this entry and returns the one-dimensional spectrum, if present.
    #[must_use]
    pub fn into_1d(self) -> Option<Spectrum1D> {
        match self {
            Self::OneD { spectrum, .. } => Some(spectrum),
            Self::TwoD { .. } => None,
        }
    }

    /// Consumes this entry and returns the two-dimensional spectrum, if present.
    #[must_use]
    pub fn into_2d(self) -> Option<Spectrum2D> {
        match self {
            Self::TwoD { spectrum, .. } => Some(spectrum),
            Self::OneD { .. } => None,
        }
    }
}

/// Non-fatal load warning emitted by the bundle loader.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoadWarning {
    /// Relative source path when source tracking is enabled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
    /// Human-readable warning message.
    pub message: String,
}

impl LoadWarning {
    /// Creates a non-fatal load warning.
    #[must_use]
    pub fn new(path: Option<PathBuf>, message: impl Into<String>) -> Self {
        Self {
            path,
            message: message.into(),
        }
    }

    /// Returns the source path for the warning, if source path tracking was enabled.
    #[must_use]
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Returns the warning message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

/// Loaded spectra, molecules, and non-fatal warnings.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SpectrumBundle {
    pub(super) spectra: Vec<LoadedSpectrum>,
    pub(super) molecules: Vec<Molecule>,
    pub(super) warnings: Vec<LoadWarning>,
}

impl SpectrumBundle {
    /// Creates an empty bundle.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a one-dimensional spectrum with source metadata and returns the bundle.
    #[must_use]
    pub fn with_1d(mut self, spectrum: Spectrum1D, source: LoadedSource) -> Self {
        self.push_1d(spectrum, source);
        self
    }

    /// Adds a two-dimensional spectrum with source metadata and returns the bundle.
    #[must_use]
    pub fn with_2d(mut self, spectrum: Spectrum2D, source: LoadedSource) -> Self {
        self.push_2d(spectrum, source);
        self
    }

    /// Adds molecule metadata and returns the bundle.
    #[must_use]
    pub fn with_molecule(mut self, molecule: Molecule) -> Self {
        self.push_molecule(molecule);
        self
    }

    /// Adds a non-fatal load warning and returns the bundle.
    #[must_use]
    pub fn with_warning(mut self, warning: LoadWarning) -> Self {
        self.push_warning(warning);
        self
    }

    /// Returns all loaded spectrum entries.
    #[must_use]
    pub fn spectra(&self) -> &[LoadedSpectrum] {
        &self.spectra
    }

    /// Returns all loaded molecules.
    #[must_use]
    pub fn molecules(&self) -> &[Molecule] {
        &self.molecules
    }

    /// Returns non-fatal loader warnings.
    #[must_use]
    pub fn warnings(&self) -> &[LoadWarning] {
        &self.warnings
    }

    /// Returns serializable summary counts for this bundle.
    #[must_use]
    pub fn summary(&self) -> SpectrumBundleSummary {
        SpectrumBundleSummary::new(
            self.len(),
            self.len_1d(),
            self.len_2d(),
            self.molecule_count(),
            self.warning_count(),
            self.source_format_counts(),
        )
    }

    /// Returns an iterator over one-dimensional spectra.
    pub fn spectra_1d(&self) -> impl Iterator<Item = &Spectrum1D> {
        self.spectra.iter().filter_map(LoadedSpectrum::as_1d)
    }

    /// Returns an iterator over two-dimensional spectra.
    pub fn spectra_2d(&self) -> impl Iterator<Item = &Spectrum2D> {
        self.spectra.iter().filter_map(LoadedSpectrum::as_2d)
    }

    /// Returns an iterator over one-dimensional spectra and their sources.
    pub fn loaded_1d(&self) -> impl Iterator<Item = (&Spectrum1D, &LoadedSource)> {
        self.spectra.iter().filter_map(|entry| match entry {
            LoadedSpectrum::OneD { spectrum, source } => Some((spectrum, source)),
            LoadedSpectrum::TwoD { .. } => None,
        })
    }

    /// Returns an iterator over two-dimensional spectra and their sources.
    pub fn loaded_2d(&self) -> impl Iterator<Item = (&Spectrum2D, &LoadedSource)> {
        self.spectra.iter().filter_map(|entry| match entry {
            LoadedSpectrum::TwoD { spectrum, source } => Some((spectrum, source)),
            LoadedSpectrum::OneD { .. } => None,
        })
    }

    /// Returns an iterator over loaded spectrum sources.
    pub fn loaded_sources(&self) -> impl Iterator<Item = &LoadedSource> {
        self.spectra.iter().map(LoadedSpectrum::source)
    }

    /// Returns an iterator over tracked source paths for loaded spectra.
    ///
    /// Spectra loaded while source path tracking is disabled are skipped.
    pub fn source_paths(&self) -> impl Iterator<Item = &Path> {
        self.loaded_sources().filter_map(LoadedSource::path)
    }

    /// Returns an iterator over source formats for loaded spectra.
    pub fn source_formats(&self) -> impl Iterator<Item = &str> {
        self.loaded_sources().map(LoadedSource::format)
    }

    /// Returns an iterator over vendor families for vendor-specific spectra.
    pub fn source_vendors(&self) -> impl Iterator<Item = LoadedSourceVendor> + '_ {
        self.loaded_sources().filter_map(LoadedSource::vendor)
    }

    /// Returns loaded spectra read with a source format.
    pub fn loaded_by_source_format(
        &self,
        format: impl AsRef<str>,
    ) -> impl Iterator<Item = &LoadedSpectrum> + '_ {
        let format = format.as_ref().to_owned();
        self.spectra
            .iter()
            .filter(move |entry| source_format_matches(entry.source().format(), format.as_str()))
    }

    /// Returns one-dimensional spectra and sources read with a source format.
    pub fn loaded_1d_by_source_format(
        &self,
        format: impl AsRef<str>,
    ) -> impl Iterator<Item = (&Spectrum1D, &LoadedSource)> + '_ {
        let format = format.as_ref().to_owned();
        self.loaded_1d()
            .filter(move |(_, source)| source_format_matches(source.format(), format.as_str()))
    }

    /// Returns two-dimensional spectra and sources read with a source format.
    pub fn loaded_2d_by_source_format(
        &self,
        format: impl AsRef<str>,
    ) -> impl Iterator<Item = (&Spectrum2D, &LoadedSource)> + '_ {
        let format = format.as_ref().to_owned();
        self.loaded_2d()
            .filter(move |(_, source)| source_format_matches(source.format(), format.as_str()))
    }

    /// Returns tracked source paths for loaded spectra read with a source format.
    ///
    /// Spectra loaded while source path tracking is disabled are skipped.
    pub fn source_paths_for_format(
        &self,
        format: impl AsRef<str>,
    ) -> impl Iterator<Item = &Path> + '_ {
        let format = format.as_ref().to_owned();
        self.loaded_sources()
            .filter(move |source| source_format_matches(source.format(), format.as_str()))
            .filter_map(LoadedSource::path)
    }

    /// Returns loaded spectra read with a vendor-specific reader.
    pub fn loaded_by_source_vendor(
        &self,
        vendor: impl AsRef<str>,
    ) -> impl Iterator<Item = &LoadedSpectrum> + '_ {
        let vendor = LoadedSourceVendor::parse(vendor.as_ref()).ok();
        self.spectra.iter().filter(move |entry| match vendor {
            Some(vendor) => entry.source().vendor() == Some(vendor),
            None => false,
        })
    }

    /// Returns one-dimensional spectra and sources read with a vendor-specific reader.
    pub fn loaded_1d_by_source_vendor(
        &self,
        vendor: impl AsRef<str>,
    ) -> impl Iterator<Item = (&Spectrum1D, &LoadedSource)> + '_ {
        let vendor = LoadedSourceVendor::parse(vendor.as_ref()).ok();
        self.loaded_1d().filter(move |(_, source)| match vendor {
            Some(vendor) => source.vendor() == Some(vendor),
            None => false,
        })
    }

    /// Returns two-dimensional spectra and sources read with a vendor-specific reader.
    pub fn loaded_2d_by_source_vendor(
        &self,
        vendor: impl AsRef<str>,
    ) -> impl Iterator<Item = (&Spectrum2D, &LoadedSource)> + '_ {
        let vendor = LoadedSourceVendor::parse(vendor.as_ref()).ok();
        self.loaded_2d().filter(move |(_, source)| match vendor {
            Some(vendor) => source.vendor() == Some(vendor),
            None => false,
        })
    }

    /// Returns tracked source paths for spectra read with a vendor-specific reader.
    ///
    /// Spectra loaded while source path tracking is disabled are skipped.
    pub fn source_paths_for_vendor(
        &self,
        vendor: impl AsRef<str>,
    ) -> impl Iterator<Item = &Path> + '_ {
        let vendor = LoadedSourceVendor::parse(vendor.as_ref()).ok();
        self.loaded_sources()
            .filter(move |source| match vendor {
                Some(vendor) => source.vendor() == Some(vendor),
                None => false,
            })
            .filter_map(LoadedSource::path)
    }

    /// Returns a loaded spectrum by its source path, if present.
    #[must_use]
    pub fn loaded_by_source_path(&self, path: impl AsRef<Path>) -> Option<&LoadedSpectrum> {
        let path = path.as_ref();
        self.spectra
            .iter()
            .find(|entry| entry.source().path() == Some(path))
    }

    /// Returns true when a loaded spectrum has the given source path.
    #[must_use]
    pub fn has_source_path(&self, path: impl AsRef<Path>) -> bool {
        self.loaded_by_source_path(path).is_some()
    }

    /// Returns the number of loaded spectra read with a source format.
    #[must_use]
    pub fn source_format_count(&self, format: impl AsRef<str>) -> usize {
        let format = format.as_ref();
        self.loaded_sources()
            .filter(|source| source_format_matches(source.format(), format))
            .count()
    }

    /// Returns true when a loaded spectrum was read with a source format.
    #[must_use]
    pub fn has_source_format(&self, format: impl AsRef<str>) -> bool {
        let format = format.as_ref();
        self.loaded_sources()
            .any(|source| source_format_matches(source.format(), format))
    }

    /// Returns the number of loaded spectra read with a vendor-specific reader.
    #[must_use]
    pub fn source_vendor_count(&self, vendor: impl AsRef<str>) -> usize {
        let Ok(vendor) = LoadedSourceVendor::parse(vendor.as_ref()) else {
            return 0;
        };
        self.loaded_sources()
            .filter(|source| source.vendor() == Some(vendor))
            .count()
    }

    /// Returns true when a loaded spectrum was read with a vendor-specific reader.
    #[must_use]
    pub fn has_source_vendor(&self, vendor: impl AsRef<str>) -> bool {
        self.source_vendor_count(vendor) > 0
    }

    /// Returns deterministic source format counts in first-seen order.
    #[must_use]
    pub fn source_format_counts(&self) -> Vec<SourceFormatCount> {
        let mut counts: Vec<SourceFormatCount> = Vec::new();
        for source in self.loaded_sources() {
            let format = source_format_count_name(source.format());
            match counts
                .iter_mut()
                .find(|count| source_format_matches(count.format(), format))
            {
                Some(count) => count.count += 1,
                None => counts.push(SourceFormatCount::new(format, 1)),
            }
        }
        counts
    }

    /// Returns deterministic source vendor counts in first-seen order.
    #[must_use]
    pub fn source_vendor_counts(&self) -> Vec<SourceVendorCount> {
        let mut counts: Vec<SourceVendorCount> = Vec::new();
        for source in self.loaded_sources() {
            let Some(vendor) = source.vendor() else {
                continue;
            };
            push_source_vendor_count(&mut counts, vendor, 1);
        }
        counts
    }

    /// Returns a one-dimensional spectrum and source by source path, if present.
    #[must_use]
    pub fn loaded_1d_by_source_path(
        &self,
        path: impl AsRef<Path>,
    ) -> Option<(&Spectrum1D, &LoadedSource)> {
        let path = path.as_ref();
        self.loaded_1d()
            .find(|(_, source)| source.path() == Some(path))
    }

    /// Returns a two-dimensional spectrum and source by source path, if present.
    #[must_use]
    pub fn loaded_2d_by_source_path(
        &self,
        path: impl AsRef<Path>,
    ) -> Option<(&Spectrum2D, &LoadedSource)> {
        let path = path.as_ref();
        self.loaded_2d()
            .find(|(_, source)| source.path() == Some(path))
    }

    /// Returns warnings associated with a source path.
    pub fn warnings_for_source_path(
        &self,
        path: impl AsRef<Path>,
    ) -> impl Iterator<Item = &LoadWarning> + '_ {
        let path = path.as_ref().to_path_buf();
        self.warnings
            .iter()
            .filter(move |warning| warning.path() == Some(path.as_path()))
    }

    /// Returns an iterator over tracked source paths for loader warnings.
    ///
    /// Warnings emitted while source path tracking is disabled are skipped.
    pub fn warning_paths(&self) -> impl Iterator<Item = &Path> {
        self.warnings.iter().filter_map(LoadWarning::path)
    }

    /// Returns an iterator over loader warning messages.
    pub fn warning_messages(&self) -> impl Iterator<Item = &str> {
        self.warnings.iter().map(LoadWarning::message)
    }

    /// Consumes the bundle and returns loaded one-dimensional spectra with sources.
    #[must_use]
    pub fn into_loaded_1d(self) -> Vec<(Spectrum1D, LoadedSource)> {
        self.spectra
            .into_iter()
            .filter_map(|entry| match entry {
                LoadedSpectrum::OneD { spectrum, source } => Some((spectrum, source)),
                LoadedSpectrum::TwoD { .. } => None,
            })
            .collect()
    }

    /// Consumes the bundle and returns loaded two-dimensional spectra with sources.
    #[must_use]
    pub fn into_loaded_2d(self) -> Vec<(Spectrum2D, LoadedSource)> {
        self.spectra
            .into_iter()
            .filter_map(|entry| match entry {
                LoadedSpectrum::TwoD { spectrum, source } => Some((spectrum, source)),
                LoadedSpectrum::OneD { .. } => None,
            })
            .collect()
    }

    /// Consumes the bundle and returns one-dimensional spectra without source metadata.
    #[must_use]
    pub fn into_spectra_1d(self) -> Vec<Spectrum1D> {
        self.into_loaded_1d()
            .into_iter()
            .map(|(spectrum, _)| spectrum)
            .collect()
    }

    /// Consumes the bundle and returns two-dimensional spectra without source metadata.
    #[must_use]
    pub fn into_spectra_2d(self) -> Vec<Spectrum2D> {
        self.into_loaded_2d()
            .into_iter()
            .map(|(spectrum, _)| spectrum)
            .collect()
    }

    /// Returns the only loaded one-dimensional spectrum.
    ///
    /// # Errors
    ///
    /// Returns an error unless the bundle contains exactly one spectrum and it
    /// is one-dimensional.
    pub fn only_1d(&self) -> Result<&Spectrum1D> {
        match self.spectra.as_slice() {
            [LoadedSpectrum::OneD { spectrum, .. }] => Ok(spectrum),
            _ => Err(self.only_error("one-dimensional")),
        }
    }

    /// Returns the only loaded one-dimensional spectrum and its source metadata.
    ///
    /// # Errors
    ///
    /// Returns an error unless the bundle contains exactly one spectrum and it
    /// is one-dimensional.
    pub fn only_loaded_1d(&self) -> Result<(&Spectrum1D, &LoadedSource)> {
        match self.spectra.as_slice() {
            [LoadedSpectrum::OneD { spectrum, source }] => Ok((spectrum, source)),
            _ => Err(self.only_error("one-dimensional")),
        }
    }

    /// Returns the only loaded two-dimensional spectrum.
    ///
    /// # Errors
    ///
    /// Returns an error unless the bundle contains exactly one spectrum and it
    /// is two-dimensional.
    pub fn only_2d(&self) -> Result<&Spectrum2D> {
        match self.spectra.as_slice() {
            [LoadedSpectrum::TwoD { spectrum, .. }] => Ok(spectrum),
            _ => Err(self.only_error("two-dimensional")),
        }
    }

    /// Returns the only loaded two-dimensional spectrum and its source metadata.
    ///
    /// # Errors
    ///
    /// Returns an error unless the bundle contains exactly one spectrum and it
    /// is two-dimensional.
    pub fn only_loaded_2d(&self) -> Result<(&Spectrum2D, &LoadedSource)> {
        match self.spectra.as_slice() {
            [LoadedSpectrum::TwoD { spectrum, source }] => Ok((spectrum, source)),
            _ => Err(self.only_error("two-dimensional")),
        }
    }

    /// Consumes the bundle and returns the only loaded one-dimensional spectrum.
    ///
    /// # Errors
    ///
    /// Returns an error unless the bundle contains exactly one spectrum and it
    /// is one-dimensional.
    pub fn into_only_1d(self) -> Result<Spectrum1D> {
        let (one_d, two_d) = spectrum_dimension_counts(self.spectra.iter());
        if self.spectra.len() != 1 {
            return Err(only_error_from_counts("one-dimensional", one_d, two_d));
        }

        match self.spectra.into_iter().next() {
            Some(LoadedSpectrum::OneD { spectrum, .. }) => Ok(spectrum),
            Some(LoadedSpectrum::TwoD { .. }) | None => {
                Err(only_error_from_counts("one-dimensional", one_d, two_d))
            }
        }
    }

    /// Consumes the bundle and returns the only loaded one-dimensional spectrum and source metadata.
    ///
    /// # Errors
    ///
    /// Returns an error unless the bundle contains exactly one spectrum and it
    /// is one-dimensional.
    pub fn into_only_loaded_1d(self) -> Result<(Spectrum1D, LoadedSource)> {
        let (one_d, two_d) = spectrum_dimension_counts(self.spectra.iter());
        if self.spectra.len() != 1 {
            return Err(only_error_from_counts("one-dimensional", one_d, two_d));
        }

        match self.spectra.into_iter().next() {
            Some(LoadedSpectrum::OneD { spectrum, source }) => Ok((spectrum, source)),
            Some(LoadedSpectrum::TwoD { .. }) | None => {
                Err(only_error_from_counts("one-dimensional", one_d, two_d))
            }
        }
    }

    /// Consumes the bundle and returns the only loaded two-dimensional spectrum.
    ///
    /// # Errors
    ///
    /// Returns an error unless the bundle contains exactly one spectrum and it
    /// is two-dimensional.
    pub fn into_only_2d(self) -> Result<Spectrum2D> {
        let (one_d, two_d) = spectrum_dimension_counts(self.spectra.iter());
        if self.spectra.len() != 1 {
            return Err(only_error_from_counts("two-dimensional", one_d, two_d));
        }

        match self.spectra.into_iter().next() {
            Some(LoadedSpectrum::TwoD { spectrum, .. }) => Ok(spectrum),
            Some(LoadedSpectrum::OneD { .. }) | None => {
                Err(only_error_from_counts("two-dimensional", one_d, two_d))
            }
        }
    }

    /// Consumes the bundle and returns the only loaded two-dimensional spectrum and source metadata.
    ///
    /// # Errors
    ///
    /// Returns an error unless the bundle contains exactly one spectrum and it
    /// is two-dimensional.
    pub fn into_only_loaded_2d(self) -> Result<(Spectrum2D, LoadedSource)> {
        let (one_d, two_d) = spectrum_dimension_counts(self.spectra.iter());
        if self.spectra.len() != 1 {
            return Err(only_error_from_counts("two-dimensional", one_d, two_d));
        }

        match self.spectra.into_iter().next() {
            Some(LoadedSpectrum::TwoD { spectrum, source }) => Ok((spectrum, source)),
            Some(LoadedSpectrum::OneD { .. }) | None => {
                Err(only_error_from_counts("two-dimensional", one_d, two_d))
            }
        }
    }

    /// Consumes the bundle and returns all loaded pieces.
    #[must_use]
    pub fn into_parts(self) -> (Vec<LoadedSpectrum>, Vec<Molecule>, Vec<LoadWarning>) {
        (self.spectra, self.molecules, self.warnings)
    }

    /// Returns the number of loaded spectra.
    #[must_use]
    pub fn len(&self) -> usize {
        self.spectra.len()
    }

    /// Returns the number of one-dimensional spectra in the bundle.
    #[must_use]
    pub fn len_1d(&self) -> usize {
        self.spectra.iter().filter(|entry| entry.is_1d()).count()
    }

    /// Returns the number of two-dimensional spectra in the bundle.
    #[must_use]
    pub fn len_2d(&self) -> usize {
        self.spectra.iter().filter(|entry| entry.is_2d()).count()
    }

    /// Returns the number of molecule metadata entries in the bundle.
    #[must_use]
    pub fn molecule_count(&self) -> usize {
        self.molecules.len()
    }

    /// Returns the number of non-fatal loader warnings in the bundle.
    #[must_use]
    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }

    /// Returns true when the bundle contains non-fatal loader warnings.
    #[must_use]
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Returns true when no spectra or molecules were loaded.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        !self.has_data()
    }

    /// Adds a one-dimensional spectrum with source metadata.
    pub fn push_1d(&mut self, spectrum: Spectrum1D, source: LoadedSource) {
        self.spectra.push(LoadedSpectrum::OneD { spectrum, source });
    }

    /// Adds a two-dimensional spectrum with source metadata.
    pub fn push_2d(&mut self, spectrum: Spectrum2D, source: LoadedSource) {
        self.spectra.push(LoadedSpectrum::TwoD { spectrum, source });
    }

    /// Adds molecule metadata.
    pub fn push_molecule(&mut self, molecule: Molecule) {
        self.molecules.push(molecule);
    }

    pub(super) fn extend_bundle(&mut self, bundle: SpectrumBundle) {
        self.spectra.extend(bundle.spectra);
        self.molecules.extend(bundle.molecules);
        self.warnings.extend(bundle.warnings);
    }

    /// Adds a non-fatal load warning.
    pub fn push_warning(&mut self, warning: LoadWarning) {
        self.warnings.push(warning);
    }

    pub(super) fn has_data(&self) -> bool {
        !self.spectra.is_empty() || !self.molecules.is_empty()
    }

    pub(super) fn only_error(&self, expected: &'static str) -> RSpinError {
        let (one_d, two_d) = spectrum_dimension_counts(self.spectra.iter());
        only_error_from_counts(expected, one_d, two_d)
    }
}
