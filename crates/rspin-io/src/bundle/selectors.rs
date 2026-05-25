//! Exact source-filtered bundle selectors.

use rspin_core::{RSpinError, Result, Spectrum1D, Spectrum2D};

use super::{
    LoadedSource, LoadedSourceVendor, LoadedSpectrum, SpectrumBundle, source_format_count_name,
    source_format_matches,
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
