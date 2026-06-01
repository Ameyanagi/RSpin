//! Exact source-filtered bundle selectors.

use std::path::Path;

use rspin_core::{RSpinError, Result, Spectrum1D, Spectrum2D};

use super::source_filter::source_filters;
use super::{
    LoadedSource, LoadedSourceDataKind, LoadedSourceFilter, LoadedSourceVendor, LoadedSpectrum,
    SpectrumBundle, source_format_count_name, source_format_matches,
};

impl SpectrumBundle {
    /// Returns the only one-dimensional spectrum read with a source format.
    ///
    /// Source format aliases such as `jdx` and `jdf` are accepted. Other
    /// matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching one-dimensional spectrum
    /// exists.
    pub fn only_1d_by_source_format(&self, format: impl AsRef<str>) -> Result<&Spectrum1D> {
        self.only_loaded_1d_by_source_format(format)
            .map(|(spectrum, _)| spectrum)
    }

    /// Returns the only one-dimensional spectrum and source read with a source format.
    ///
    /// Source format aliases such as `jdx` and `jdf` are accepted. Other
    /// matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching one-dimensional spectrum
    /// exists.
    pub fn only_loaded_1d_by_source_format(
        &self,
        format: impl AsRef<str>,
    ) -> Result<(&Spectrum1D, &LoadedSource)> {
        let format = format.as_ref().to_owned();
        let label = format!("source format {}", source_format_count_name(&format));
        self.only_loaded_1d_matching_source(&label, move |source| {
            source_format_matches(source.format(), format.as_str())
        })
    }

    /// Returns the only two-dimensional spectrum read with a source format.
    ///
    /// Source format aliases such as `jdx` and `jdf` are accepted. Other
    /// matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching two-dimensional spectrum
    /// exists.
    pub fn only_2d_by_source_format(&self, format: impl AsRef<str>) -> Result<&Spectrum2D> {
        self.only_loaded_2d_by_source_format(format)
            .map(|(spectrum, _)| spectrum)
    }

    /// Returns the only two-dimensional spectrum and source read with a source format.
    ///
    /// Source format aliases such as `jdx` and `jdf` are accepted. Other
    /// matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching two-dimensional spectrum
    /// exists.
    pub fn only_loaded_2d_by_source_format(
        &self,
        format: impl AsRef<str>,
    ) -> Result<(&Spectrum2D, &LoadedSource)> {
        let format = format.as_ref().to_owned();
        let label = format!("source format {}", source_format_count_name(&format));
        self.only_loaded_2d_matching_source(&label, move |source| {
            source_format_matches(source.format(), format.as_str())
        })
    }

    /// Returns the only one-dimensional spectrum read with a vendor-specific reader.
    ///
    /// Vendor aliases such as `agilent` and `varian` are accepted. Other
    /// matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching one-dimensional spectrum
    /// exists.
    pub fn only_1d_by_source_vendor(&self, vendor: impl AsRef<str>) -> Result<&Spectrum1D> {
        self.only_loaded_1d_by_source_vendor(vendor)
            .map(|(spectrum, _)| spectrum)
    }

    /// Returns the only one-dimensional spectrum and source read with a vendor-specific reader.
    ///
    /// Vendor aliases such as `agilent` and `varian` are accepted. Other
    /// matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching one-dimensional spectrum
    /// exists.
    pub fn only_loaded_1d_by_source_vendor(
        &self,
        vendor: impl AsRef<str>,
    ) -> Result<(&Spectrum1D, &LoadedSource)> {
        let vendor = vendor.as_ref();
        let parsed_vendor = LoadedSourceVendor::parse(vendor).ok();
        let label = source_vendor_filter_label(vendor, parsed_vendor);
        self.only_loaded_1d_matching_source(&label, move |source| match parsed_vendor {
            Some(vendor) => source.vendor() == Some(vendor),
            None => false,
        })
    }

    /// Returns the only two-dimensional spectrum read with a vendor-specific reader.
    ///
    /// Vendor aliases such as `agilent` and `varian` are accepted. Other
    /// matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching two-dimensional spectrum
    /// exists.
    pub fn only_2d_by_source_vendor(&self, vendor: impl AsRef<str>) -> Result<&Spectrum2D> {
        self.only_loaded_2d_by_source_vendor(vendor)
            .map(|(spectrum, _)| spectrum)
    }

    /// Returns the only two-dimensional spectrum and source read with a vendor-specific reader.
    ///
    /// Vendor aliases such as `agilent` and `varian` are accepted. Other
    /// matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching two-dimensional spectrum
    /// exists.
    pub fn only_loaded_2d_by_source_vendor(
        &self,
        vendor: impl AsRef<str>,
    ) -> Result<(&Spectrum2D, &LoadedSource)> {
        let vendor = vendor.as_ref();
        let parsed_vendor = LoadedSourceVendor::parse(vendor).ok();
        let label = source_vendor_filter_label(vendor, parsed_vendor);
        self.only_loaded_2d_matching_source(&label, move |source| match parsed_vendor {
            Some(vendor) => source.vendor() == Some(vendor),
            None => false,
        })
    }

    /// Returns the only one-dimensional spectrum read from a source data kind.
    ///
    /// Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching one-dimensional spectrum
    /// exists.
    pub fn only_1d_by_source_data_kind(
        &self,
        data_kind: LoadedSourceDataKind,
    ) -> Result<&Spectrum1D> {
        self.only_loaded_1d_by_source_data_kind(data_kind)
            .map(|(spectrum, _)| spectrum)
    }

    /// Returns the only one-dimensional spectrum and source read from a source data kind.
    ///
    /// Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching one-dimensional spectrum
    /// exists.
    pub fn only_loaded_1d_by_source_data_kind(
        &self,
        data_kind: LoadedSourceDataKind,
    ) -> Result<(&Spectrum1D, &LoadedSource)> {
        self.only_loaded_1d_by_source(LoadedSourceFilter::data_kind(data_kind))
    }

    /// Returns the only two-dimensional spectrum read from a source data kind.
    ///
    /// Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching two-dimensional spectrum
    /// exists.
    pub fn only_2d_by_source_data_kind(
        &self,
        data_kind: LoadedSourceDataKind,
    ) -> Result<&Spectrum2D> {
        self.only_loaded_2d_by_source_data_kind(data_kind)
            .map(|(spectrum, _)| spectrum)
    }

    /// Returns the only two-dimensional spectrum and source read from a source data kind.
    ///
    /// Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching two-dimensional spectrum
    /// exists.
    pub fn only_loaded_2d_by_source_data_kind(
        &self,
        data_kind: LoadedSourceDataKind,
    ) -> Result<(&Spectrum2D, &LoadedSource)> {
        self.only_loaded_2d_by_source(LoadedSourceFilter::data_kind(data_kind))
    }

    /// Returns the only one-dimensional spectrum read from a tracked source path.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching one-dimensional spectrum
    /// exists.
    pub fn only_1d_by_source_path(&self, path: impl AsRef<Path>) -> Result<&Spectrum1D> {
        self.only_loaded_1d_by_source_path(path)
            .map(|(spectrum, _)| spectrum)
    }

    /// Returns the only one-dimensional spectrum and source read from a tracked source path.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching one-dimensional spectrum
    /// exists.
    pub fn only_loaded_1d_by_source_path(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<(&Spectrum1D, &LoadedSource)> {
        let path = path.as_ref().to_path_buf();
        let label = source_path_filter_label(&path);
        self.only_loaded_1d_matching_source(&label, move |source| {
            source.path() == Some(path.as_path())
        })
    }

    /// Returns the only two-dimensional spectrum read from a tracked source path.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching two-dimensional spectrum
    /// exists.
    pub fn only_2d_by_source_path(&self, path: impl AsRef<Path>) -> Result<&Spectrum2D> {
        self.only_loaded_2d_by_source_path(path)
            .map(|(spectrum, _)| spectrum)
    }

    /// Returns the only two-dimensional spectrum and source read from a tracked source path.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching two-dimensional spectrum
    /// exists.
    pub fn only_loaded_2d_by_source_path(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<(&Spectrum2D, &LoadedSource)> {
        let path = path.as_ref().to_path_buf();
        let label = source_path_filter_label(&path);
        self.only_loaded_2d_matching_source(&label, move |source| {
            source.path() == Some(path.as_path())
        })
    }

    /// Returns the only one-dimensional spectrum matching a generic source filter.
    ///
    /// The filter may target a source format, vendor family, source data kind,
    /// or tracked source path. Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching one-dimensional spectrum
    /// exists.
    pub fn only_1d_by_source(&self, filter: impl Into<LoadedSourceFilter>) -> Result<&Spectrum1D> {
        self.only_loaded_1d_by_source(filter)
            .map(|(spectrum, _)| spectrum)
    }

    /// Returns the only one-dimensional spectrum and source matching a generic source filter.
    ///
    /// The filter may target a source format, vendor family, source data kind,
    /// or tracked source path. Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching one-dimensional spectrum
    /// exists.
    pub fn only_loaded_1d_by_source(
        &self,
        filter: impl Into<LoadedSourceFilter>,
    ) -> Result<(&Spectrum1D, &LoadedSource)> {
        let filter = filter.into();
        let label = source_filter_label(&filter);
        self.only_loaded_1d_matching_source(&label, move |source| filter.matches_source(source))
    }

    /// Returns the only two-dimensional spectrum matching a generic source filter.
    ///
    /// The filter may target a source format, vendor family, source data kind,
    /// or tracked source path. Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching two-dimensional spectrum
    /// exists.
    pub fn only_2d_by_source(&self, filter: impl Into<LoadedSourceFilter>) -> Result<&Spectrum2D> {
        self.only_loaded_2d_by_source(filter)
            .map(|(spectrum, _)| spectrum)
    }

    /// Returns the only two-dimensional spectrum and source matching a generic source filter.
    ///
    /// The filter may target a source format, vendor family, source data kind,
    /// or tracked source path. Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching two-dimensional spectrum
    /// exists.
    pub fn only_loaded_2d_by_source(
        &self,
        filter: impl Into<LoadedSourceFilter>,
    ) -> Result<(&Spectrum2D, &LoadedSource)> {
        let filter = filter.into();
        let label = source_filter_label(&filter);
        self.only_loaded_2d_matching_source(&label, move |source| filter.matches_source(source))
    }

    /// Returns the only one-dimensional spectrum matching any generic source filter.
    ///
    /// Filters are combined with logical OR. Passing an empty iterator leaves
    /// source matching unrestricted. Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching one-dimensional spectrum
    /// exists.
    pub fn only_1d_by_sources<I, F>(&self, filters: I) -> Result<&Spectrum1D>
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>,
    {
        self.only_loaded_1d_by_sources(filters)
            .map(|(spectrum, _)| spectrum)
    }

    /// Returns the only one-dimensional spectrum and source matching any generic source filter.
    ///
    /// Filters are combined with logical OR. Passing an empty iterator leaves
    /// source matching unrestricted. Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching one-dimensional spectrum
    /// exists.
    pub fn only_loaded_1d_by_sources<I, F>(
        &self,
        filters: I,
    ) -> Result<(&Spectrum1D, &LoadedSource)>
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>,
    {
        let filters = source_filters(filters);
        let label = source_filters_label(&filters);
        self.only_loaded_1d_matching_source(&label, move |source| {
            source_matches_any(&filters, source)
        })
    }

    /// Returns the only two-dimensional spectrum matching any generic source filter.
    ///
    /// Filters are combined with logical OR. Passing an empty iterator leaves
    /// source matching unrestricted. Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching two-dimensional spectrum
    /// exists.
    pub fn only_2d_by_sources<I, F>(&self, filters: I) -> Result<&Spectrum2D>
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>,
    {
        self.only_loaded_2d_by_sources(filters)
            .map(|(spectrum, _)| spectrum)
    }

    /// Returns the only two-dimensional spectrum and source matching any generic source filter.
    ///
    /// Filters are combined with logical OR. Passing an empty iterator leaves
    /// source matching unrestricted. Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching two-dimensional spectrum
    /// exists.
    pub fn only_loaded_2d_by_sources<I, F>(
        &self,
        filters: I,
    ) -> Result<(&Spectrum2D, &LoadedSource)>
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>,
    {
        let filters = source_filters(filters);
        let label = source_filters_label(&filters);
        self.only_loaded_2d_matching_source(&label, move |source| {
            source_matches_any(&filters, source)
        })
    }

    /// Consumes the bundle and returns the only one-dimensional spectrum read with a source format.
    ///
    /// Source format aliases such as `jdx` and `jdf` are accepted. Other
    /// matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching one-dimensional spectrum
    /// exists.
    pub fn into_only_1d_by_source_format(self, format: impl AsRef<str>) -> Result<Spectrum1D> {
        self.into_only_loaded_1d_by_source_format(format)
            .map(|(spectrum, _)| spectrum)
    }

    /// Consumes the bundle and returns the only one-dimensional spectrum and source read with a source format.
    ///
    /// Source format aliases such as `jdx` and `jdf` are accepted. Other
    /// matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching one-dimensional spectrum
    /// exists.
    pub fn into_only_loaded_1d_by_source_format(
        self,
        format: impl AsRef<str>,
    ) -> Result<(Spectrum1D, LoadedSource)> {
        let format = format.as_ref().to_owned();
        let label = format!("source format {}", source_format_count_name(&format));
        self.into_only_loaded_1d_matching_source(&label, move |source| {
            source_format_matches(source.format(), format.as_str())
        })
    }

    /// Consumes the bundle and returns the only two-dimensional spectrum read with a source format.
    ///
    /// Source format aliases such as `jdx` and `jdf` are accepted. Other
    /// matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching two-dimensional spectrum
    /// exists.
    pub fn into_only_2d_by_source_format(self, format: impl AsRef<str>) -> Result<Spectrum2D> {
        self.into_only_loaded_2d_by_source_format(format)
            .map(|(spectrum, _)| spectrum)
    }

    /// Consumes the bundle and returns the only two-dimensional spectrum and source read with a source format.
    ///
    /// Source format aliases such as `jdx` and `jdf` are accepted. Other
    /// matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching two-dimensional spectrum
    /// exists.
    pub fn into_only_loaded_2d_by_source_format(
        self,
        format: impl AsRef<str>,
    ) -> Result<(Spectrum2D, LoadedSource)> {
        let format = format.as_ref().to_owned();
        let label = format!("source format {}", source_format_count_name(&format));
        self.into_only_loaded_2d_matching_source(&label, move |source| {
            source_format_matches(source.format(), format.as_str())
        })
    }

    /// Consumes the bundle and returns the only one-dimensional spectrum read with a vendor-specific reader.
    ///
    /// Vendor aliases such as `agilent` and `varian` are accepted. Other
    /// matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching one-dimensional spectrum
    /// exists.
    pub fn into_only_1d_by_source_vendor(self, vendor: impl AsRef<str>) -> Result<Spectrum1D> {
        self.into_only_loaded_1d_by_source_vendor(vendor)
            .map(|(spectrum, _)| spectrum)
    }

    /// Consumes the bundle and returns the only one-dimensional spectrum and source read with a vendor-specific reader.
    ///
    /// Vendor aliases such as `agilent` and `varian` are accepted. Other
    /// matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching one-dimensional spectrum
    /// exists.
    pub fn into_only_loaded_1d_by_source_vendor(
        self,
        vendor: impl AsRef<str>,
    ) -> Result<(Spectrum1D, LoadedSource)> {
        let vendor = vendor.as_ref();
        let parsed_vendor = LoadedSourceVendor::parse(vendor).ok();
        let label = source_vendor_filter_label(vendor, parsed_vendor);
        self.into_only_loaded_1d_matching_source(&label, move |source| match parsed_vendor {
            Some(vendor) => source.vendor() == Some(vendor),
            None => false,
        })
    }

    /// Consumes the bundle and returns the only two-dimensional spectrum read with a vendor-specific reader.
    ///
    /// Vendor aliases such as `agilent` and `varian` are accepted. Other
    /// matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching two-dimensional spectrum
    /// exists.
    pub fn into_only_2d_by_source_vendor(self, vendor: impl AsRef<str>) -> Result<Spectrum2D> {
        self.into_only_loaded_2d_by_source_vendor(vendor)
            .map(|(spectrum, _)| spectrum)
    }

    /// Consumes the bundle and returns the only two-dimensional spectrum and source read with a vendor-specific reader.
    ///
    /// Vendor aliases such as `agilent` and `varian` are accepted. Other
    /// matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching two-dimensional spectrum
    /// exists.
    pub fn into_only_loaded_2d_by_source_vendor(
        self,
        vendor: impl AsRef<str>,
    ) -> Result<(Spectrum2D, LoadedSource)> {
        let vendor = vendor.as_ref();
        let parsed_vendor = LoadedSourceVendor::parse(vendor).ok();
        let label = source_vendor_filter_label(vendor, parsed_vendor);
        self.into_only_loaded_2d_matching_source(&label, move |source| match parsed_vendor {
            Some(vendor) => source.vendor() == Some(vendor),
            None => false,
        })
    }

    /// Consumes the bundle and returns the only one-dimensional spectrum read from a source data kind.
    ///
    /// Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching one-dimensional spectrum
    /// exists.
    pub fn into_only_1d_by_source_data_kind(
        self,
        data_kind: LoadedSourceDataKind,
    ) -> Result<Spectrum1D> {
        self.into_only_loaded_1d_by_source_data_kind(data_kind)
            .map(|(spectrum, _)| spectrum)
    }

    /// Consumes the bundle and returns the only one-dimensional spectrum and source read from a source data kind.
    ///
    /// Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching one-dimensional spectrum
    /// exists.
    pub fn into_only_loaded_1d_by_source_data_kind(
        self,
        data_kind: LoadedSourceDataKind,
    ) -> Result<(Spectrum1D, LoadedSource)> {
        self.into_only_loaded_1d_by_source(LoadedSourceFilter::data_kind(data_kind))
    }

    /// Consumes the bundle and returns the only two-dimensional spectrum read from a source data kind.
    ///
    /// Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching two-dimensional spectrum
    /// exists.
    pub fn into_only_2d_by_source_data_kind(
        self,
        data_kind: LoadedSourceDataKind,
    ) -> Result<Spectrum2D> {
        self.into_only_loaded_2d_by_source_data_kind(data_kind)
            .map(|(spectrum, _)| spectrum)
    }

    /// Consumes the bundle and returns the only two-dimensional spectrum and source read from a source data kind.
    ///
    /// Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching two-dimensional spectrum
    /// exists.
    pub fn into_only_loaded_2d_by_source_data_kind(
        self,
        data_kind: LoadedSourceDataKind,
    ) -> Result<(Spectrum2D, LoadedSource)> {
        self.into_only_loaded_2d_by_source(LoadedSourceFilter::data_kind(data_kind))
    }

    /// Consumes the bundle and returns the only one-dimensional spectrum read from a tracked source path.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching one-dimensional spectrum
    /// exists.
    pub fn into_only_1d_by_source_path(self, path: impl AsRef<Path>) -> Result<Spectrum1D> {
        self.into_only_loaded_1d_by_source_path(path)
            .map(|(spectrum, _)| spectrum)
    }

    /// Consumes the bundle and returns the only one-dimensional spectrum and source read from a tracked source path.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching one-dimensional spectrum
    /// exists.
    pub fn into_only_loaded_1d_by_source_path(
        self,
        path: impl AsRef<Path>,
    ) -> Result<(Spectrum1D, LoadedSource)> {
        let path = path.as_ref().to_path_buf();
        let label = source_path_filter_label(&path);
        self.into_only_loaded_1d_matching_source(&label, move |source| {
            source.path() == Some(path.as_path())
        })
    }

    /// Consumes the bundle and returns the only two-dimensional spectrum read from a tracked source path.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching two-dimensional spectrum
    /// exists.
    pub fn into_only_2d_by_source_path(self, path: impl AsRef<Path>) -> Result<Spectrum2D> {
        self.into_only_loaded_2d_by_source_path(path)
            .map(|(spectrum, _)| spectrum)
    }

    /// Consumes the bundle and returns the only two-dimensional spectrum and source read from a tracked source path.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching two-dimensional spectrum
    /// exists.
    pub fn into_only_loaded_2d_by_source_path(
        self,
        path: impl AsRef<Path>,
    ) -> Result<(Spectrum2D, LoadedSource)> {
        let path = path.as_ref().to_path_buf();
        let label = source_path_filter_label(&path);
        self.into_only_loaded_2d_matching_source(&label, move |source| {
            source.path() == Some(path.as_path())
        })
    }

    /// Consumes the bundle and returns the only one-dimensional spectrum matching a generic source filter.
    ///
    /// The filter may target a source format, vendor family, source data kind,
    /// or tracked source path. Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching one-dimensional spectrum
    /// exists.
    pub fn into_only_1d_by_source(
        self,
        filter: impl Into<LoadedSourceFilter>,
    ) -> Result<Spectrum1D> {
        self.into_only_loaded_1d_by_source(filter)
            .map(|(spectrum, _)| spectrum)
    }

    /// Consumes the bundle and returns the only one-dimensional spectrum and source matching a generic source filter.
    ///
    /// The filter may target a source format, vendor family, source data kind,
    /// or tracked source path. Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching one-dimensional spectrum
    /// exists.
    pub fn into_only_loaded_1d_by_source(
        self,
        filter: impl Into<LoadedSourceFilter>,
    ) -> Result<(Spectrum1D, LoadedSource)> {
        let filter = filter.into();
        let label = source_filter_label(&filter);
        self.into_only_loaded_1d_matching_source(&label, move |source| {
            filter.matches_source(source)
        })
    }

    /// Consumes the bundle and returns the only two-dimensional spectrum matching a generic source filter.
    ///
    /// The filter may target a source format, vendor family, source data kind,
    /// or tracked source path. Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching two-dimensional spectrum
    /// exists.
    pub fn into_only_2d_by_source(
        self,
        filter: impl Into<LoadedSourceFilter>,
    ) -> Result<Spectrum2D> {
        self.into_only_loaded_2d_by_source(filter)
            .map(|(spectrum, _)| spectrum)
    }

    /// Consumes the bundle and returns the only two-dimensional spectrum and source matching a generic source filter.
    ///
    /// The filter may target a source format, vendor family, source data kind,
    /// or tracked source path. Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching two-dimensional spectrum
    /// exists.
    pub fn into_only_loaded_2d_by_source(
        self,
        filter: impl Into<LoadedSourceFilter>,
    ) -> Result<(Spectrum2D, LoadedSource)> {
        let filter = filter.into();
        let label = source_filter_label(&filter);
        self.into_only_loaded_2d_matching_source(&label, move |source| {
            filter.matches_source(source)
        })
    }

    /// Consumes the bundle and returns the only one-dimensional spectrum matching any generic source filter.
    ///
    /// Filters are combined with logical OR. Passing an empty iterator leaves
    /// source matching unrestricted. Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching one-dimensional spectrum
    /// exists.
    pub fn into_only_1d_by_sources<I, F>(self, filters: I) -> Result<Spectrum1D>
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>,
    {
        self.into_only_loaded_1d_by_sources(filters)
            .map(|(spectrum, _)| spectrum)
    }

    /// Consumes the bundle and returns the only one-dimensional spectrum and source matching any generic source filter.
    ///
    /// Filters are combined with logical OR. Passing an empty iterator leaves
    /// source matching unrestricted. Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching one-dimensional spectrum
    /// exists.
    pub fn into_only_loaded_1d_by_sources<I, F>(
        self,
        filters: I,
    ) -> Result<(Spectrum1D, LoadedSource)>
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>,
    {
        let filters = source_filters(filters);
        let label = source_filters_label(&filters);
        self.into_only_loaded_1d_matching_source(&label, move |source| {
            source_matches_any(&filters, source)
        })
    }

    /// Consumes the bundle and returns the only two-dimensional spectrum matching any generic source filter.
    ///
    /// Filters are combined with logical OR. Passing an empty iterator leaves
    /// source matching unrestricted. Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching two-dimensional spectrum
    /// exists.
    pub fn into_only_2d_by_sources<I, F>(self, filters: I) -> Result<Spectrum2D>
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>,
    {
        self.into_only_loaded_2d_by_sources(filters)
            .map(|(spectrum, _)| spectrum)
    }

    /// Consumes the bundle and returns the only two-dimensional spectrum and source matching any generic source filter.
    ///
    /// Filters are combined with logical OR. Passing an empty iterator leaves
    /// source matching unrestricted. Other matching dimensions do not prevent success.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly one matching two-dimensional spectrum
    /// exists.
    pub fn into_only_loaded_2d_by_sources<I, F>(
        self,
        filters: I,
    ) -> Result<(Spectrum2D, LoadedSource)>
    where
        I: IntoIterator<Item = F>,
        F: Into<LoadedSourceFilter>,
    {
        let filters = source_filters(filters);
        let label = source_filters_label(&filters);
        self.into_only_loaded_2d_matching_source(&label, move |source| {
            source_matches_any(&filters, source)
        })
    }

    fn only_loaded_1d_matching_source(
        &self,
        filter: &str,
        mut matches_source: impl FnMut(&LoadedSource) -> bool,
    ) -> Result<(&Spectrum1D, &LoadedSource)> {
        let mut one_d = 0;
        let mut two_d = 0;
        let mut matched = None;

        for entry in &self.spectra {
            if !matches_source(entry.source()) {
                continue;
            }
            match entry {
                LoadedSpectrum::OneD { spectrum, source } => {
                    one_d += 1;
                    matched = Some((spectrum, source));
                }
                LoadedSpectrum::TwoD { .. } => two_d += 1,
            }
        }

        match matched {
            Some(loaded) if one_d == 1 => Ok(loaded),
            Some(_) | None => Err(only_source_filter_error(
                "one-dimensional",
                filter,
                one_d,
                two_d,
            )),
        }
    }

    fn only_loaded_2d_matching_source(
        &self,
        filter: &str,
        mut matches_source: impl FnMut(&LoadedSource) -> bool,
    ) -> Result<(&Spectrum2D, &LoadedSource)> {
        let mut one_d = 0;
        let mut two_d = 0;
        let mut matched = None;

        for entry in &self.spectra {
            if !matches_source(entry.source()) {
                continue;
            }
            match entry {
                LoadedSpectrum::OneD { .. } => one_d += 1,
                LoadedSpectrum::TwoD { spectrum, source } => {
                    two_d += 1;
                    matched = Some((spectrum, source));
                }
            }
        }

        match matched {
            Some(loaded) if two_d == 1 => Ok(loaded),
            Some(_) | None => Err(only_source_filter_error(
                "two-dimensional",
                filter,
                one_d,
                two_d,
            )),
        }
    }

    fn into_only_loaded_1d_matching_source(
        self,
        filter: &str,
        mut matches_source: impl FnMut(&LoadedSource) -> bool,
    ) -> Result<(Spectrum1D, LoadedSource)> {
        let mut one_d = 0;
        let mut two_d = 0;
        let mut matched = None;

        for entry in self.spectra {
            if !matches_source(entry.source()) {
                continue;
            }
            match entry {
                LoadedSpectrum::OneD { spectrum, source } => {
                    one_d += 1;
                    matched = Some((spectrum, source));
                }
                LoadedSpectrum::TwoD { .. } => two_d += 1,
            }
        }

        match matched {
            Some(loaded) if one_d == 1 => Ok(loaded),
            Some(_) | None => Err(only_source_filter_error(
                "one-dimensional",
                filter,
                one_d,
                two_d,
            )),
        }
    }

    fn into_only_loaded_2d_matching_source(
        self,
        filter: &str,
        mut matches_source: impl FnMut(&LoadedSource) -> bool,
    ) -> Result<(Spectrum2D, LoadedSource)> {
        let mut one_d = 0;
        let mut two_d = 0;
        let mut matched = None;

        for entry in self.spectra {
            if !matches_source(entry.source()) {
                continue;
            }
            match entry {
                LoadedSpectrum::OneD { .. } => one_d += 1,
                LoadedSpectrum::TwoD { spectrum, source } => {
                    two_d += 1;
                    matched = Some((spectrum, source));
                }
            }
        }

        match matched {
            Some(loaded) if two_d == 1 => Ok(loaded),
            Some(_) | None => Err(only_source_filter_error(
                "two-dimensional",
                filter,
                one_d,
                two_d,
            )),
        }
    }
}

fn only_source_filter_error(
    expected: &'static str,
    filter: &str,
    one_d: usize,
    two_d: usize,
) -> RSpinError {
    RSpinError::Parse {
        format: "spectrum bundle",
        message: format!(
            "expected exactly one {expected} spectrum for {filter}, found {one_d} one-dimensional and {two_d} two-dimensional spectra"
        ),
    }
}

fn source_vendor_filter_label(vendor: &str, parsed_vendor: Option<LoadedSourceVendor>) -> String {
    let vendor = match parsed_vendor {
        Some(parsed_vendor) => parsed_vendor.as_str(),
        None => vendor.trim(),
    };
    format!("source vendor {vendor}")
}

fn source_path_filter_label(path: &Path) -> String {
    format!("source path {}", path.display())
}

fn source_filter_label(filter: &LoadedSourceFilter) -> String {
    match filter {
        LoadedSourceFilter::Format { format } => {
            format!("source format {}", source_format_count_name(format))
        }
        LoadedSourceFilter::Vendor { vendor } => {
            source_vendor_filter_label(vendor, LoadedSourceVendor::parse(vendor).ok())
        }
        LoadedSourceFilter::DataKind { data_kind } => {
            format!("source data kind {data_kind}")
        }
        LoadedSourceFilter::Path { path } => source_path_filter_label(path),
        LoadedSourceFilter::PathPrefix { path } => {
            format!("source path prefix {}", path.display())
        }
    }
}

fn source_filters_label(filters: &[LoadedSourceFilter]) -> String {
    if filters.is_empty() {
        return "any source".to_owned();
    }

    filters
        .iter()
        .map(source_filter_label)
        .collect::<Vec<_>>()
        .join(" or ")
}

fn source_matches_any(filters: &[LoadedSourceFilter], source: &LoadedSource) -> bool {
    filters.is_empty() || filters.iter().any(|filter| filter.matches_source(source))
}
