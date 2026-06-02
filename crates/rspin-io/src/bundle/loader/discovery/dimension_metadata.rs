//! Dimension-specific source metadata convenience loading from discovered candidates.

use std::path::Path;

use rspin_core::Result;

use super::DiscoveredSpectrumSource;
use crate::bundle::{
    LoadedSourceDataKind, LoadedSourceFilter, SpectrumBundle, SpectrumBundleLoader,
    SpectrumBundleSummary,
};

macro_rules! dimension_metadata_helpers {
    (
        dimension = $dimension:literal;
        bundle_by_source = $bundle_by_source:ident;
        bundle_by_sources = $bundle_by_sources:ident;

        reader_format_relative = $reader_format_relative:ident;
        reader_format = $reader_format:ident;
        reader_formats_relative = $reader_formats_relative:ident;
        reader_formats = $reader_formats:ident;
        reader_vendor_relative = $reader_vendor_relative:ident;
        reader_vendor = $reader_vendor:ident;
        reader_vendors_relative = $reader_vendors_relative:ident;
        reader_vendors = $reader_vendors:ident;
        reader_data_kind_relative = $reader_data_kind_relative:ident;
        reader_data_kind = $reader_data_kind:ident;
        reader_data_kinds_relative = $reader_data_kinds_relative:ident;
        reader_data_kinds = $reader_data_kinds:ident;

        reader_summary_format_relative = $reader_summary_format_relative:ident;
        reader_summary_format = $reader_summary_format:ident;
        reader_summary_formats_relative = $reader_summary_formats_relative:ident;
        reader_summary_formats = $reader_summary_formats:ident;
        reader_summary_vendor_relative = $reader_summary_vendor_relative:ident;
        reader_summary_vendor = $reader_summary_vendor:ident;
        reader_summary_vendors_relative = $reader_summary_vendors_relative:ident;
        reader_summary_vendors = $reader_summary_vendors:ident;
        reader_summary_data_kind_relative = $reader_summary_data_kind_relative:ident;
        reader_summary_data_kind = $reader_summary_data_kind:ident;
        reader_summary_data_kinds_relative = $reader_summary_data_kinds_relative:ident;
        reader_summary_data_kinds = $reader_summary_data_kinds:ident;

        load_format_relative = $load_format_relative:ident;
        load_format = $load_format:ident;
        load_formats_relative = $load_formats_relative:ident;
        load_formats = $load_formats:ident;
        load_vendor_relative = $load_vendor_relative:ident;
        load_vendor = $load_vendor:ident;
        load_vendors_relative = $load_vendors_relative:ident;
        load_vendors = $load_vendors:ident;
        load_data_kind_relative = $load_data_kind_relative:ident;
        load_data_kind = $load_data_kind:ident;
        load_data_kinds_relative = $load_data_kinds_relative:ident;
        load_data_kinds = $load_data_kinds:ident;

        strict_format_relative = $strict_format_relative:ident;
        strict_format = $strict_format:ident;
        strict_formats_relative = $strict_formats_relative:ident;
        strict_formats = $strict_formats:ident;
        strict_vendor_relative = $strict_vendor_relative:ident;
        strict_vendor = $strict_vendor:ident;
        strict_vendors_relative = $strict_vendors_relative:ident;
        strict_vendors = $strict_vendors:ident;
        strict_data_kind_relative = $strict_data_kind_relative:ident;
        strict_data_kind = $strict_data_kind:ident;
        strict_data_kinds_relative = $strict_data_kinds_relative:ident;
        strict_data_kinds = $strict_data_kinds:ident;

        summary_format_relative = $summary_format_relative:ident;
        summary_format = $summary_format:ident;
        summary_formats_relative = $summary_formats_relative:ident;
        summary_formats = $summary_formats:ident;
        summary_vendor_relative = $summary_vendor_relative:ident;
        summary_vendor = $summary_vendor:ident;
        summary_vendors_relative = $summary_vendors_relative:ident;
        summary_vendors = $summary_vendors:ident;
        summary_data_kind_relative = $summary_data_kind_relative:ident;
        summary_data_kind = $summary_data_kind:ident;
        summary_data_kinds_relative = $summary_data_kinds_relative:ident;
        summary_data_kinds = $summary_data_kinds:ident;

        strict_summary_format_relative = $strict_summary_format_relative:ident;
        strict_summary_format = $strict_summary_format:ident;
        strict_summary_formats_relative = $strict_summary_formats_relative:ident;
        strict_summary_formats = $strict_summary_formats:ident;
        strict_summary_vendor_relative = $strict_summary_vendor_relative:ident;
        strict_summary_vendor = $strict_summary_vendor:ident;
        strict_summary_vendors_relative = $strict_summary_vendors_relative:ident;
        strict_summary_vendors = $strict_summary_vendors:ident;
        strict_summary_data_kind_relative = $strict_summary_data_kind_relative:ident;
        strict_summary_data_kind = $strict_summary_data_kind:ident;
        strict_summary_data_kinds_relative = $strict_summary_data_kinds_relative:ident;
        strict_summary_data_kinds = $strict_summary_data_kinds:ident;
    ) => {
        #[doc = concat!("Loads ", $dimension, " discovered source candidates matching one source format.")]
        ///
        /// # Errors
        ///
        /// Returns an error when loading the matching discovered sources fails.
        pub fn $load_format_relative<'a>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            format: impl AsRef<str>,
        ) -> Result<SpectrumBundle> {
            SpectrumBundleLoader::new().$reader_format_relative(base, sources, format)
        }

        #[doc = concat!("Loads ", $dimension, " discovered source candidates matching one source format.")]
        ///
        /// # Errors
        ///
        /// Returns an error when loading the matching discovered sources fails.
        pub fn $load_format<'a>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            format: impl AsRef<str>,
        ) -> Result<SpectrumBundle> {
            $load_format_relative(base, sources, format)
        }

        #[doc = concat!("Loads ", $dimension, " discovered source candidates matching any source format.")]
        ///
        /// Formats are combined with logical OR. Passing an empty iterator loads
        /// all provided discovered sources in the requested dimension.
        ///
        /// # Errors
        ///
        /// Returns an error when loading the matching discovered sources fails.
        pub fn $load_formats_relative<'a, I, F>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            formats: I,
        ) -> Result<SpectrumBundle>
        where
            I: IntoIterator<Item = F>,
            F: AsRef<str>,
        {
            SpectrumBundleLoader::new().$reader_formats_relative(base, sources, formats)
        }

        #[doc = concat!("Loads ", $dimension, " discovered source candidates matching any source format.")]
        ///
        /// # Errors
        ///
        /// Returns an error when loading the matching discovered sources fails.
        pub fn $load_formats<'a, I, F>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            formats: I,
        ) -> Result<SpectrumBundle>
        where
            I: IntoIterator<Item = F>,
            F: AsRef<str>,
        {
            $load_formats_relative(base, sources, formats)
        }

        #[doc = concat!("Loads ", $dimension, " discovered source candidates matching one vendor family.")]
        ///
        /// # Errors
        ///
        /// Returns an error when loading the matching discovered sources fails.
        pub fn $load_vendor_relative<'a>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            vendor: impl AsRef<str>,
        ) -> Result<SpectrumBundle> {
            SpectrumBundleLoader::new().$reader_vendor_relative(base, sources, vendor)
        }

        #[doc = concat!("Loads ", $dimension, " discovered source candidates matching one vendor family.")]
        ///
        /// # Errors
        ///
        /// Returns an error when loading the matching discovered sources fails.
        pub fn $load_vendor<'a>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            vendor: impl AsRef<str>,
        ) -> Result<SpectrumBundle> {
            $load_vendor_relative(base, sources, vendor)
        }

        #[doc = concat!("Loads ", $dimension, " discovered source candidates matching any vendor family.")]
        ///
        /// Vendors are combined with logical OR. Passing an empty iterator loads
        /// all provided discovered sources in the requested dimension.
        ///
        /// # Errors
        ///
        /// Returns an error when loading the matching discovered sources fails.
        pub fn $load_vendors_relative<'a, I, V>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            vendors: I,
        ) -> Result<SpectrumBundle>
        where
            I: IntoIterator<Item = V>,
            V: AsRef<str>,
        {
            SpectrumBundleLoader::new().$reader_vendors_relative(base, sources, vendors)
        }

        #[doc = concat!("Loads ", $dimension, " discovered source candidates matching any vendor family.")]
        ///
        /// # Errors
        ///
        /// Returns an error when loading the matching discovered sources fails.
        pub fn $load_vendors<'a, I, V>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            vendors: I,
        ) -> Result<SpectrumBundle>
        where
            I: IntoIterator<Item = V>,
            V: AsRef<str>,
        {
            $load_vendors_relative(base, sources, vendors)
        }

        #[doc = concat!("Loads ", $dimension, " discovered source candidates matching one raw/processed source data kind.")]
        ///
        /// # Errors
        ///
        /// Returns an error when loading the matching discovered sources fails.
        pub fn $load_data_kind_relative<'a>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            data_kind: LoadedSourceDataKind,
        ) -> Result<SpectrumBundle> {
            SpectrumBundleLoader::new().$reader_data_kind_relative(base, sources, data_kind)
        }

        #[doc = concat!("Loads ", $dimension, " discovered source candidates matching one raw/processed source data kind.")]
        ///
        /// # Errors
        ///
        /// Returns an error when loading the matching discovered sources fails.
        pub fn $load_data_kind<'a>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            data_kind: LoadedSourceDataKind,
        ) -> Result<SpectrumBundle> {
            $load_data_kind_relative(base, sources, data_kind)
        }

        #[doc = concat!("Loads ", $dimension, " discovered source candidates matching any raw/processed source data kind.")]
        ///
        /// Data kinds are combined with logical OR. Passing an empty iterator
        /// loads all provided discovered sources in the requested dimension.
        ///
        /// # Errors
        ///
        /// Returns an error when loading the matching discovered sources fails.
        pub fn $load_data_kinds_relative<'a, I>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            data_kinds: I,
        ) -> Result<SpectrumBundle>
        where
            I: IntoIterator<Item = LoadedSourceDataKind>,
        {
            SpectrumBundleLoader::new().$reader_data_kinds_relative(base, sources, data_kinds)
        }

        #[doc = concat!("Loads ", $dimension, " discovered source candidates matching any raw/processed source data kind.")]
        ///
        /// # Errors
        ///
        /// Returns an error when loading the matching discovered sources fails.
        pub fn $load_data_kinds<'a, I>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            data_kinds: I,
        ) -> Result<SpectrumBundle>
        where
            I: IntoIterator<Item = LoadedSourceDataKind>,
        {
            $load_data_kinds_relative(base, sources, data_kinds)
        }

        #[doc = concat!("Strictly loads ", $dimension, " discovered source candidates matching one source format.")]
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading the matching discovered sources fails.
        pub fn $strict_format_relative<'a>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            format: impl AsRef<str>,
        ) -> Result<SpectrumBundle> {
            SpectrumBundleLoader::new()
                .strict()
                .$reader_format_relative(base, sources, format)
        }

        #[doc = concat!("Strictly loads ", $dimension, " discovered source candidates matching one source format.")]
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading the matching discovered sources fails.
        pub fn $strict_format<'a>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            format: impl AsRef<str>,
        ) -> Result<SpectrumBundle> {
            $strict_format_relative(base, sources, format)
        }

        #[doc = concat!("Strictly loads ", $dimension, " discovered source candidates matching any source format.")]
        ///
        /// Formats are combined with logical OR. Passing an empty iterator loads
        /// all provided discovered sources in the requested dimension.
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading the matching discovered sources fails.
        pub fn $strict_formats_relative<'a, I, F>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            formats: I,
        ) -> Result<SpectrumBundle>
        where
            I: IntoIterator<Item = F>,
            F: AsRef<str>,
        {
            SpectrumBundleLoader::new()
                .strict()
                .$reader_formats_relative(base, sources, formats)
        }

        #[doc = concat!("Strictly loads ", $dimension, " discovered source candidates matching any source format.")]
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading the matching discovered sources fails.
        pub fn $strict_formats<'a, I, F>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            formats: I,
        ) -> Result<SpectrumBundle>
        where
            I: IntoIterator<Item = F>,
            F: AsRef<str>,
        {
            $strict_formats_relative(base, sources, formats)
        }

        #[doc = concat!("Strictly loads ", $dimension, " discovered source candidates matching one vendor family.")]
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading the matching discovered sources fails.
        pub fn $strict_vendor_relative<'a>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            vendor: impl AsRef<str>,
        ) -> Result<SpectrumBundle> {
            SpectrumBundleLoader::new()
                .strict()
                .$reader_vendor_relative(base, sources, vendor)
        }

        #[doc = concat!("Strictly loads ", $dimension, " discovered source candidates matching one vendor family.")]
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading the matching discovered sources fails.
        pub fn $strict_vendor<'a>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            vendor: impl AsRef<str>,
        ) -> Result<SpectrumBundle> {
            $strict_vendor_relative(base, sources, vendor)
        }

        #[doc = concat!("Strictly loads ", $dimension, " discovered source candidates matching any vendor family.")]
        ///
        /// Vendors are combined with logical OR. Passing an empty iterator loads
        /// all provided discovered sources in the requested dimension.
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading the matching discovered sources fails.
        pub fn $strict_vendors_relative<'a, I, V>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            vendors: I,
        ) -> Result<SpectrumBundle>
        where
            I: IntoIterator<Item = V>,
            V: AsRef<str>,
        {
            SpectrumBundleLoader::new()
                .strict()
                .$reader_vendors_relative(base, sources, vendors)
        }

        #[doc = concat!("Strictly loads ", $dimension, " discovered source candidates matching any vendor family.")]
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading the matching discovered sources fails.
        pub fn $strict_vendors<'a, I, V>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            vendors: I,
        ) -> Result<SpectrumBundle>
        where
            I: IntoIterator<Item = V>,
            V: AsRef<str>,
        {
            $strict_vendors_relative(base, sources, vendors)
        }

        #[doc = concat!("Strictly loads ", $dimension, " discovered source candidates matching one raw/processed source data kind.")]
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading the matching discovered sources fails.
        pub fn $strict_data_kind_relative<'a>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            data_kind: LoadedSourceDataKind,
        ) -> Result<SpectrumBundle> {
            SpectrumBundleLoader::new()
                .strict()
                .$reader_data_kind_relative(base, sources, data_kind)
        }

        #[doc = concat!("Strictly loads ", $dimension, " discovered source candidates matching one raw/processed source data kind.")]
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading the matching discovered sources fails.
        pub fn $strict_data_kind<'a>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            data_kind: LoadedSourceDataKind,
        ) -> Result<SpectrumBundle> {
            $strict_data_kind_relative(base, sources, data_kind)
        }

        #[doc = concat!("Strictly loads ", $dimension, " discovered source candidates matching any raw/processed source data kind.")]
        ///
        /// Data kinds are combined with logical OR. Passing an empty iterator
        /// loads all provided discovered sources in the requested dimension.
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading the matching discovered sources fails.
        pub fn $strict_data_kinds_relative<'a, I>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            data_kinds: I,
        ) -> Result<SpectrumBundle>
        where
            I: IntoIterator<Item = LoadedSourceDataKind>,
        {
            SpectrumBundleLoader::new()
                .strict()
                .$reader_data_kinds_relative(base, sources, data_kinds)
        }

        #[doc = concat!("Strictly loads ", $dimension, " discovered source candidates matching any raw/processed source data kind.")]
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading the matching discovered sources fails.
        pub fn $strict_data_kinds<'a, I>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            data_kinds: I,
        ) -> Result<SpectrumBundle>
        where
            I: IntoIterator<Item = LoadedSourceDataKind>,
        {
            $strict_data_kinds_relative(base, sources, data_kinds)
        }

        #[doc = concat!("Loads ", $dimension, " discovered source candidates matching one source format and returns summary counts.")]
        ///
        /// # Errors
        ///
        /// Returns an error when loading the matching discovered sources fails.
        pub fn $summary_format_relative<'a>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            format: impl AsRef<str>,
        ) -> Result<SpectrumBundleSummary> {
            SpectrumBundleLoader::new().$reader_summary_format_relative(base, sources, format)
        }

        #[doc = concat!("Loads ", $dimension, " discovered source candidates matching one source format and returns summary counts.")]
        ///
        /// # Errors
        ///
        /// Returns an error when loading the matching discovered sources fails.
        pub fn $summary_format<'a>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            format: impl AsRef<str>,
        ) -> Result<SpectrumBundleSummary> {
            $summary_format_relative(base, sources, format)
        }

        #[doc = concat!("Loads ", $dimension, " discovered source candidates matching any source format and returns summary counts.")]
        ///
        /// Formats are combined with logical OR. Passing an empty iterator loads
        /// all provided discovered sources in the requested dimension.
        ///
        /// # Errors
        ///
        /// Returns an error when loading the matching discovered sources fails.
        pub fn $summary_formats_relative<'a, I, F>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            formats: I,
        ) -> Result<SpectrumBundleSummary>
        where
            I: IntoIterator<Item = F>,
            F: AsRef<str>,
        {
            SpectrumBundleLoader::new().$reader_summary_formats_relative(base, sources, formats)
        }

        #[doc = concat!("Loads ", $dimension, " discovered source candidates matching any source format and returns summary counts.")]
        ///
        /// # Errors
        ///
        /// Returns an error when loading the matching discovered sources fails.
        pub fn $summary_formats<'a, I, F>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            formats: I,
        ) -> Result<SpectrumBundleSummary>
        where
            I: IntoIterator<Item = F>,
            F: AsRef<str>,
        {
            $summary_formats_relative(base, sources, formats)
        }

        #[doc = concat!("Loads ", $dimension, " discovered source candidates matching one vendor family and returns summary counts.")]
        ///
        /// # Errors
        ///
        /// Returns an error when loading the matching discovered sources fails.
        pub fn $summary_vendor_relative<'a>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            vendor: impl AsRef<str>,
        ) -> Result<SpectrumBundleSummary> {
            SpectrumBundleLoader::new().$reader_summary_vendor_relative(base, sources, vendor)
        }

        #[doc = concat!("Loads ", $dimension, " discovered source candidates matching one vendor family and returns summary counts.")]
        ///
        /// # Errors
        ///
        /// Returns an error when loading the matching discovered sources fails.
        pub fn $summary_vendor<'a>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            vendor: impl AsRef<str>,
        ) -> Result<SpectrumBundleSummary> {
            $summary_vendor_relative(base, sources, vendor)
        }

        #[doc = concat!("Loads ", $dimension, " discovered source candidates matching any vendor family and returns summary counts.")]
        ///
        /// Vendors are combined with logical OR. Passing an empty iterator loads
        /// all provided discovered sources in the requested dimension.
        ///
        /// # Errors
        ///
        /// Returns an error when loading the matching discovered sources fails.
        pub fn $summary_vendors_relative<'a, I, V>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            vendors: I,
        ) -> Result<SpectrumBundleSummary>
        where
            I: IntoIterator<Item = V>,
            V: AsRef<str>,
        {
            SpectrumBundleLoader::new().$reader_summary_vendors_relative(base, sources, vendors)
        }

        #[doc = concat!("Loads ", $dimension, " discovered source candidates matching any vendor family and returns summary counts.")]
        ///
        /// # Errors
        ///
        /// Returns an error when loading the matching discovered sources fails.
        pub fn $summary_vendors<'a, I, V>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            vendors: I,
        ) -> Result<SpectrumBundleSummary>
        where
            I: IntoIterator<Item = V>,
            V: AsRef<str>,
        {
            $summary_vendors_relative(base, sources, vendors)
        }

        #[doc = concat!("Loads ", $dimension, " discovered source candidates matching one raw/processed source data kind and returns summary counts.")]
        ///
        /// # Errors
        ///
        /// Returns an error when loading the matching discovered sources fails.
        pub fn $summary_data_kind_relative<'a>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            data_kind: LoadedSourceDataKind,
        ) -> Result<SpectrumBundleSummary> {
            SpectrumBundleLoader::new().$reader_summary_data_kind_relative(
                base, sources, data_kind,
            )
        }

        #[doc = concat!("Loads ", $dimension, " discovered source candidates matching one raw/processed source data kind and returns summary counts.")]
        ///
        /// # Errors
        ///
        /// Returns an error when loading the matching discovered sources fails.
        pub fn $summary_data_kind<'a>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            data_kind: LoadedSourceDataKind,
        ) -> Result<SpectrumBundleSummary> {
            $summary_data_kind_relative(base, sources, data_kind)
        }

        #[doc = concat!("Loads ", $dimension, " discovered source candidates matching any raw/processed source data kind and returns summary counts.")]
        ///
        /// Data kinds are combined with logical OR. Passing an empty iterator
        /// loads all provided discovered sources in the requested dimension.
        ///
        /// # Errors
        ///
        /// Returns an error when loading the matching discovered sources fails.
        pub fn $summary_data_kinds_relative<'a, I>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            data_kinds: I,
        ) -> Result<SpectrumBundleSummary>
        where
            I: IntoIterator<Item = LoadedSourceDataKind>,
        {
            SpectrumBundleLoader::new().$reader_summary_data_kinds_relative(
                base, sources, data_kinds,
            )
        }

        #[doc = concat!("Loads ", $dimension, " discovered source candidates matching any raw/processed source data kind and returns summary counts.")]
        ///
        /// # Errors
        ///
        /// Returns an error when loading the matching discovered sources fails.
        pub fn $summary_data_kinds<'a, I>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            data_kinds: I,
        ) -> Result<SpectrumBundleSummary>
        where
            I: IntoIterator<Item = LoadedSourceDataKind>,
        {
            $summary_data_kinds_relative(base, sources, data_kinds)
        }

        #[doc = concat!("Strictly loads ", $dimension, " discovered source candidates matching one source format and returns summary counts.")]
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading the matching discovered sources fails.
        pub fn $strict_summary_format_relative<'a>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            format: impl AsRef<str>,
        ) -> Result<SpectrumBundleSummary> {
            SpectrumBundleLoader::new()
                .strict()
                .$reader_summary_format_relative(base, sources, format)
        }

        #[doc = concat!("Strictly loads ", $dimension, " discovered source candidates matching one source format and returns summary counts.")]
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading the matching discovered sources fails.
        pub fn $strict_summary_format<'a>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            format: impl AsRef<str>,
        ) -> Result<SpectrumBundleSummary> {
            $strict_summary_format_relative(base, sources, format)
        }

        #[doc = concat!("Strictly loads ", $dimension, " discovered source candidates matching any source format and returns summary counts.")]
        ///
        /// Formats are combined with logical OR. Passing an empty iterator loads
        /// all provided discovered sources in the requested dimension.
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading the matching discovered sources fails.
        pub fn $strict_summary_formats_relative<'a, I, F>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            formats: I,
        ) -> Result<SpectrumBundleSummary>
        where
            I: IntoIterator<Item = F>,
            F: AsRef<str>,
        {
            SpectrumBundleLoader::new()
                .strict()
                .$reader_summary_formats_relative(base, sources, formats)
        }

        #[doc = concat!("Strictly loads ", $dimension, " discovered source candidates matching any source format and returns summary counts.")]
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading the matching discovered sources fails.
        pub fn $strict_summary_formats<'a, I, F>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            formats: I,
        ) -> Result<SpectrumBundleSummary>
        where
            I: IntoIterator<Item = F>,
            F: AsRef<str>,
        {
            $strict_summary_formats_relative(base, sources, formats)
        }

        #[doc = concat!("Strictly loads ", $dimension, " discovered source candidates matching one vendor family and returns summary counts.")]
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading the matching discovered sources fails.
        pub fn $strict_summary_vendor_relative<'a>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            vendor: impl AsRef<str>,
        ) -> Result<SpectrumBundleSummary> {
            SpectrumBundleLoader::new()
                .strict()
                .$reader_summary_vendor_relative(base, sources, vendor)
        }

        #[doc = concat!("Strictly loads ", $dimension, " discovered source candidates matching one vendor family and returns summary counts.")]
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading the matching discovered sources fails.
        pub fn $strict_summary_vendor<'a>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            vendor: impl AsRef<str>,
        ) -> Result<SpectrumBundleSummary> {
            $strict_summary_vendor_relative(base, sources, vendor)
        }

        #[doc = concat!("Strictly loads ", $dimension, " discovered source candidates matching any vendor family and returns summary counts.")]
        ///
        /// Vendors are combined with logical OR. Passing an empty iterator loads
        /// all provided discovered sources in the requested dimension.
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading the matching discovered sources fails.
        pub fn $strict_summary_vendors_relative<'a, I, V>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            vendors: I,
        ) -> Result<SpectrumBundleSummary>
        where
            I: IntoIterator<Item = V>,
            V: AsRef<str>,
        {
            SpectrumBundleLoader::new()
                .strict()
                .$reader_summary_vendors_relative(base, sources, vendors)
        }

        #[doc = concat!("Strictly loads ", $dimension, " discovered source candidates matching any vendor family and returns summary counts.")]
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading the matching discovered sources fails.
        pub fn $strict_summary_vendors<'a, I, V>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            vendors: I,
        ) -> Result<SpectrumBundleSummary>
        where
            I: IntoIterator<Item = V>,
            V: AsRef<str>,
        {
            $strict_summary_vendors_relative(base, sources, vendors)
        }

        #[doc = concat!("Strictly loads ", $dimension, " discovered source candidates matching one raw/processed source data kind and returns summary counts.")]
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading the matching discovered sources fails.
        pub fn $strict_summary_data_kind_relative<'a>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            data_kind: LoadedSourceDataKind,
        ) -> Result<SpectrumBundleSummary> {
            SpectrumBundleLoader::new()
                .strict()
                .$reader_summary_data_kind_relative(base, sources, data_kind)
        }

        #[doc = concat!("Strictly loads ", $dimension, " discovered source candidates matching one raw/processed source data kind and returns summary counts.")]
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading the matching discovered sources fails.
        pub fn $strict_summary_data_kind<'a>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            data_kind: LoadedSourceDataKind,
        ) -> Result<SpectrumBundleSummary> {
            $strict_summary_data_kind_relative(base, sources, data_kind)
        }

        #[doc = concat!("Strictly loads ", $dimension, " discovered source candidates matching any raw/processed source data kind and returns summary counts.")]
        ///
        /// Data kinds are combined with logical OR. Passing an empty iterator
        /// loads all provided discovered sources in the requested dimension.
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading the matching discovered sources fails.
        pub fn $strict_summary_data_kinds_relative<'a, I>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            data_kinds: I,
        ) -> Result<SpectrumBundleSummary>
        where
            I: IntoIterator<Item = LoadedSourceDataKind>,
        {
            SpectrumBundleLoader::new()
                .strict()
                .$reader_summary_data_kinds_relative(base, sources, data_kinds)
        }

        #[doc = concat!("Strictly loads ", $dimension, " discovered source candidates matching any raw/processed source data kind and returns summary counts.")]
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading the matching discovered sources fails.
        pub fn $strict_summary_data_kinds<'a, I>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            data_kinds: I,
        ) -> Result<SpectrumBundleSummary>
        where
            I: IntoIterator<Item = LoadedSourceDataKind>,
        {
            $strict_summary_data_kinds_relative(base, sources, data_kinds)
        }

        impl SpectrumBundleLoader {
            #[doc = concat!("Loads ", $dimension, " discovered source candidates matching one source format.")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading the matching discovered sources fails.
            pub fn $reader_format_relative<'a>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                format: impl AsRef<str>,
            ) -> Result<SpectrumBundle> {
                self.$bundle_by_source(base, sources, LoadedSourceFilter::format(format))
            }

            #[doc = concat!("Loads ", $dimension, " discovered source candidates matching one source format.")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading the matching discovered sources fails.
            pub fn $reader_format<'a>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                format: impl AsRef<str>,
            ) -> Result<SpectrumBundle> {
                self.$reader_format_relative(base, sources, format)
            }

            #[doc = concat!("Loads ", $dimension, " discovered source candidates matching any source format.")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading the matching discovered sources fails.
            pub fn $reader_formats_relative<'a, I, F>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                formats: I,
            ) -> Result<SpectrumBundle>
            where
                I: IntoIterator<Item = F>,
                F: AsRef<str>,
            {
                self.$bundle_by_sources(base, sources, format_filters(formats))
            }

            #[doc = concat!("Loads ", $dimension, " discovered source candidates matching any source format.")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading the matching discovered sources fails.
            pub fn $reader_formats<'a, I, F>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                formats: I,
            ) -> Result<SpectrumBundle>
            where
                I: IntoIterator<Item = F>,
                F: AsRef<str>,
            {
                self.$reader_formats_relative(base, sources, formats)
            }

            #[doc = concat!("Loads ", $dimension, " discovered source candidates matching one vendor family.")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading the matching discovered sources fails.
            pub fn $reader_vendor_relative<'a>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                vendor: impl AsRef<str>,
            ) -> Result<SpectrumBundle> {
                self.$bundle_by_source(base, sources, LoadedSourceFilter::vendor(vendor))
            }

            #[doc = concat!("Loads ", $dimension, " discovered source candidates matching one vendor family.")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading the matching discovered sources fails.
            pub fn $reader_vendor<'a>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                vendor: impl AsRef<str>,
            ) -> Result<SpectrumBundle> {
                self.$reader_vendor_relative(base, sources, vendor)
            }

            #[doc = concat!("Loads ", $dimension, " discovered source candidates matching any vendor family.")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading the matching discovered sources fails.
            pub fn $reader_vendors_relative<'a, I, V>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                vendors: I,
            ) -> Result<SpectrumBundle>
            where
                I: IntoIterator<Item = V>,
                V: AsRef<str>,
            {
                self.$bundle_by_sources(base, sources, vendor_filters(vendors))
            }

            #[doc = concat!("Loads ", $dimension, " discovered source candidates matching any vendor family.")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading the matching discovered sources fails.
            pub fn $reader_vendors<'a, I, V>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                vendors: I,
            ) -> Result<SpectrumBundle>
            where
                I: IntoIterator<Item = V>,
                V: AsRef<str>,
            {
                self.$reader_vendors_relative(base, sources, vendors)
            }

            #[doc = concat!("Loads ", $dimension, " discovered source candidates matching one raw/processed source data kind.")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading the matching discovered sources fails.
            pub fn $reader_data_kind_relative<'a>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                data_kind: LoadedSourceDataKind,
            ) -> Result<SpectrumBundle> {
                self.$bundle_by_source(base, sources, LoadedSourceFilter::data_kind(data_kind))
            }

            #[doc = concat!("Loads ", $dimension, " discovered source candidates matching one raw/processed source data kind.")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading the matching discovered sources fails.
            pub fn $reader_data_kind<'a>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                data_kind: LoadedSourceDataKind,
            ) -> Result<SpectrumBundle> {
                self.$reader_data_kind_relative(base, sources, data_kind)
            }

            #[doc = concat!("Loads ", $dimension, " discovered source candidates matching any raw/processed source data kind.")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading the matching discovered sources fails.
            pub fn $reader_data_kinds_relative<'a, I>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                data_kinds: I,
            ) -> Result<SpectrumBundle>
            where
                I: IntoIterator<Item = LoadedSourceDataKind>,
            {
                self.$bundle_by_sources(base, sources, data_kind_filters(data_kinds))
            }

            #[doc = concat!("Loads ", $dimension, " discovered source candidates matching any raw/processed source data kind.")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading the matching discovered sources fails.
            pub fn $reader_data_kinds<'a, I>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                data_kinds: I,
            ) -> Result<SpectrumBundle>
            where
                I: IntoIterator<Item = LoadedSourceDataKind>,
            {
                self.$reader_data_kinds_relative(base, sources, data_kinds)
            }

            #[doc = concat!("Loads ", $dimension, " discovered source candidates matching one source format and returns summary counts.")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading the matching discovered sources fails.
            pub fn $reader_summary_format_relative<'a>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                format: impl AsRef<str>,
            ) -> Result<SpectrumBundleSummary> {
                self.$reader_format_relative(base, sources, format)
                    .map(|bundle| bundle.summary())
            }

            #[doc = concat!("Loads ", $dimension, " discovered source candidates matching one source format and returns summary counts.")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading the matching discovered sources fails.
            pub fn $reader_summary_format<'a>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                format: impl AsRef<str>,
            ) -> Result<SpectrumBundleSummary> {
                self.$reader_summary_format_relative(base, sources, format)
            }

            #[doc = concat!("Loads ", $dimension, " discovered source candidates matching any source format and returns summary counts.")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading the matching discovered sources fails.
            pub fn $reader_summary_formats_relative<'a, I, F>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                formats: I,
            ) -> Result<SpectrumBundleSummary>
            where
                I: IntoIterator<Item = F>,
                F: AsRef<str>,
            {
                self.$reader_formats_relative(base, sources, formats)
                    .map(|bundle| bundle.summary())
            }

            #[doc = concat!("Loads ", $dimension, " discovered source candidates matching any source format and returns summary counts.")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading the matching discovered sources fails.
            pub fn $reader_summary_formats<'a, I, F>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                formats: I,
            ) -> Result<SpectrumBundleSummary>
            where
                I: IntoIterator<Item = F>,
                F: AsRef<str>,
            {
                self.$reader_summary_formats_relative(base, sources, formats)
            }

            #[doc = concat!("Loads ", $dimension, " discovered source candidates matching one vendor family and returns summary counts.")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading the matching discovered sources fails.
            pub fn $reader_summary_vendor_relative<'a>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                vendor: impl AsRef<str>,
            ) -> Result<SpectrumBundleSummary> {
                self.$reader_vendor_relative(base, sources, vendor)
                    .map(|bundle| bundle.summary())
            }

            #[doc = concat!("Loads ", $dimension, " discovered source candidates matching one vendor family and returns summary counts.")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading the matching discovered sources fails.
            pub fn $reader_summary_vendor<'a>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                vendor: impl AsRef<str>,
            ) -> Result<SpectrumBundleSummary> {
                self.$reader_summary_vendor_relative(base, sources, vendor)
            }

            #[doc = concat!("Loads ", $dimension, " discovered source candidates matching any vendor family and returns summary counts.")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading the matching discovered sources fails.
            pub fn $reader_summary_vendors_relative<'a, I, V>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                vendors: I,
            ) -> Result<SpectrumBundleSummary>
            where
                I: IntoIterator<Item = V>,
                V: AsRef<str>,
            {
                self.$reader_vendors_relative(base, sources, vendors)
                    .map(|bundle| bundle.summary())
            }

            #[doc = concat!("Loads ", $dimension, " discovered source candidates matching any vendor family and returns summary counts.")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading the matching discovered sources fails.
            pub fn $reader_summary_vendors<'a, I, V>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                vendors: I,
            ) -> Result<SpectrumBundleSummary>
            where
                I: IntoIterator<Item = V>,
                V: AsRef<str>,
            {
                self.$reader_summary_vendors_relative(base, sources, vendors)
            }

            #[doc = concat!("Loads ", $dimension, " discovered source candidates matching one raw/processed source data kind and returns summary counts.")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading the matching discovered sources fails.
            pub fn $reader_summary_data_kind_relative<'a>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                data_kind: LoadedSourceDataKind,
            ) -> Result<SpectrumBundleSummary> {
                self.$reader_data_kind_relative(base, sources, data_kind)
                    .map(|bundle| bundle.summary())
            }

            #[doc = concat!("Loads ", $dimension, " discovered source candidates matching one raw/processed source data kind and returns summary counts.")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading the matching discovered sources fails.
            pub fn $reader_summary_data_kind<'a>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                data_kind: LoadedSourceDataKind,
            ) -> Result<SpectrumBundleSummary> {
                self.$reader_summary_data_kind_relative(base, sources, data_kind)
            }

            #[doc = concat!("Loads ", $dimension, " discovered source candidates matching any raw/processed source data kind and returns summary counts.")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading the matching discovered sources fails.
            pub fn $reader_summary_data_kinds_relative<'a, I>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                data_kinds: I,
            ) -> Result<SpectrumBundleSummary>
            where
                I: IntoIterator<Item = LoadedSourceDataKind>,
            {
                self.$reader_data_kinds_relative(base, sources, data_kinds)
                    .map(|bundle| bundle.summary())
            }

            #[doc = concat!("Loads ", $dimension, " discovered source candidates matching any raw/processed source data kind and returns summary counts.")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading the matching discovered sources fails.
            pub fn $reader_summary_data_kinds<'a, I>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                data_kinds: I,
            ) -> Result<SpectrumBundleSummary>
            where
                I: IntoIterator<Item = LoadedSourceDataKind>,
            {
                self.$reader_summary_data_kinds_relative(base, sources, data_kinds)
            }
        }
    };
}

dimension_metadata_helpers! {
    dimension = "one-dimensional";
    bundle_by_source = read_discovered_bundle_1d_by_source_relative_to;
    bundle_by_sources = read_discovered_bundle_1d_by_sources_relative_to;

    reader_format_relative = read_discovered_bundle_1d_by_source_format_relative_to;
    reader_format = read_discovered_bundle_1d_by_source_format;
    reader_formats_relative = read_discovered_bundle_1d_by_source_formats_relative_to;
    reader_formats = read_discovered_bundle_1d_by_source_formats;
    reader_vendor_relative = read_discovered_bundle_1d_by_source_vendor_relative_to;
    reader_vendor = read_discovered_bundle_1d_by_source_vendor;
    reader_vendors_relative = read_discovered_bundle_1d_by_source_vendors_relative_to;
    reader_vendors = read_discovered_bundle_1d_by_source_vendors;
    reader_data_kind_relative = read_discovered_bundle_1d_by_source_data_kind_relative_to;
    reader_data_kind = read_discovered_bundle_1d_by_source_data_kind;
    reader_data_kinds_relative = read_discovered_bundle_1d_by_source_data_kinds_relative_to;
    reader_data_kinds = read_discovered_bundle_1d_by_source_data_kinds;

    reader_summary_format_relative = read_discovered_bundle_1d_summary_by_source_format_relative_to;
    reader_summary_format = read_discovered_bundle_1d_summary_by_source_format;
    reader_summary_formats_relative = read_discovered_bundle_1d_summary_by_source_formats_relative_to;
    reader_summary_formats = read_discovered_bundle_1d_summary_by_source_formats;
    reader_summary_vendor_relative = read_discovered_bundle_1d_summary_by_source_vendor_relative_to;
    reader_summary_vendor = read_discovered_bundle_1d_summary_by_source_vendor;
    reader_summary_vendors_relative = read_discovered_bundle_1d_summary_by_source_vendors_relative_to;
    reader_summary_vendors = read_discovered_bundle_1d_summary_by_source_vendors;
    reader_summary_data_kind_relative = read_discovered_bundle_1d_summary_by_source_data_kind_relative_to;
    reader_summary_data_kind = read_discovered_bundle_1d_summary_by_source_data_kind;
    reader_summary_data_kinds_relative = read_discovered_bundle_1d_summary_by_source_data_kinds_relative_to;
    reader_summary_data_kinds = read_discovered_bundle_1d_summary_by_source_data_kinds;

    load_format_relative = load_discovered_spectra_1d_by_source_format_relative_to;
    load_format = load_discovered_spectra_1d_by_source_format;
    load_formats_relative = load_discovered_spectra_1d_by_source_formats_relative_to;
    load_formats = load_discovered_spectra_1d_by_source_formats;
    load_vendor_relative = load_discovered_spectra_1d_by_source_vendor_relative_to;
    load_vendor = load_discovered_spectra_1d_by_source_vendor;
    load_vendors_relative = load_discovered_spectra_1d_by_source_vendors_relative_to;
    load_vendors = load_discovered_spectra_1d_by_source_vendors;
    load_data_kind_relative = load_discovered_spectra_1d_by_source_data_kind_relative_to;
    load_data_kind = load_discovered_spectra_1d_by_source_data_kind;
    load_data_kinds_relative = load_discovered_spectra_1d_by_source_data_kinds_relative_to;
    load_data_kinds = load_discovered_spectra_1d_by_source_data_kinds;

    strict_format_relative = load_discovered_spectra_1d_strict_by_source_format_relative_to;
    strict_format = load_discovered_spectra_1d_strict_by_source_format;
    strict_formats_relative = load_discovered_spectra_1d_strict_by_source_formats_relative_to;
    strict_formats = load_discovered_spectra_1d_strict_by_source_formats;
    strict_vendor_relative = load_discovered_spectra_1d_strict_by_source_vendor_relative_to;
    strict_vendor = load_discovered_spectra_1d_strict_by_source_vendor;
    strict_vendors_relative = load_discovered_spectra_1d_strict_by_source_vendors_relative_to;
    strict_vendors = load_discovered_spectra_1d_strict_by_source_vendors;
    strict_data_kind_relative = load_discovered_spectra_1d_strict_by_source_data_kind_relative_to;
    strict_data_kind = load_discovered_spectra_1d_strict_by_source_data_kind;
    strict_data_kinds_relative = load_discovered_spectra_1d_strict_by_source_data_kinds_relative_to;
    strict_data_kinds = load_discovered_spectra_1d_strict_by_source_data_kinds;

    summary_format_relative = load_discovered_spectra_1d_summary_by_source_format_relative_to;
    summary_format = load_discovered_spectra_1d_summary_by_source_format;
    summary_formats_relative = load_discovered_spectra_1d_summary_by_source_formats_relative_to;
    summary_formats = load_discovered_spectra_1d_summary_by_source_formats;
    summary_vendor_relative = load_discovered_spectra_1d_summary_by_source_vendor_relative_to;
    summary_vendor = load_discovered_spectra_1d_summary_by_source_vendor;
    summary_vendors_relative = load_discovered_spectra_1d_summary_by_source_vendors_relative_to;
    summary_vendors = load_discovered_spectra_1d_summary_by_source_vendors;
    summary_data_kind_relative = load_discovered_spectra_1d_summary_by_source_data_kind_relative_to;
    summary_data_kind = load_discovered_spectra_1d_summary_by_source_data_kind;
    summary_data_kinds_relative = load_discovered_spectra_1d_summary_by_source_data_kinds_relative_to;
    summary_data_kinds = load_discovered_spectra_1d_summary_by_source_data_kinds;

    strict_summary_format_relative = load_discovered_spectra_1d_summary_strict_by_source_format_relative_to;
    strict_summary_format = load_discovered_spectra_1d_summary_strict_by_source_format;
    strict_summary_formats_relative = load_discovered_spectra_1d_summary_strict_by_source_formats_relative_to;
    strict_summary_formats = load_discovered_spectra_1d_summary_strict_by_source_formats;
    strict_summary_vendor_relative = load_discovered_spectra_1d_summary_strict_by_source_vendor_relative_to;
    strict_summary_vendor = load_discovered_spectra_1d_summary_strict_by_source_vendor;
    strict_summary_vendors_relative = load_discovered_spectra_1d_summary_strict_by_source_vendors_relative_to;
    strict_summary_vendors = load_discovered_spectra_1d_summary_strict_by_source_vendors;
    strict_summary_data_kind_relative = load_discovered_spectra_1d_summary_strict_by_source_data_kind_relative_to;
    strict_summary_data_kind = load_discovered_spectra_1d_summary_strict_by_source_data_kind;
    strict_summary_data_kinds_relative = load_discovered_spectra_1d_summary_strict_by_source_data_kinds_relative_to;
    strict_summary_data_kinds = load_discovered_spectra_1d_summary_strict_by_source_data_kinds;
}

dimension_metadata_helpers! {
    dimension = "two-dimensional";
    bundle_by_source = read_discovered_bundle_2d_by_source_relative_to;
    bundle_by_sources = read_discovered_bundle_2d_by_sources_relative_to;

    reader_format_relative = read_discovered_bundle_2d_by_source_format_relative_to;
    reader_format = read_discovered_bundle_2d_by_source_format;
    reader_formats_relative = read_discovered_bundle_2d_by_source_formats_relative_to;
    reader_formats = read_discovered_bundle_2d_by_source_formats;
    reader_vendor_relative = read_discovered_bundle_2d_by_source_vendor_relative_to;
    reader_vendor = read_discovered_bundle_2d_by_source_vendor;
    reader_vendors_relative = read_discovered_bundle_2d_by_source_vendors_relative_to;
    reader_vendors = read_discovered_bundle_2d_by_source_vendors;
    reader_data_kind_relative = read_discovered_bundle_2d_by_source_data_kind_relative_to;
    reader_data_kind = read_discovered_bundle_2d_by_source_data_kind;
    reader_data_kinds_relative = read_discovered_bundle_2d_by_source_data_kinds_relative_to;
    reader_data_kinds = read_discovered_bundle_2d_by_source_data_kinds;

    reader_summary_format_relative = read_discovered_bundle_2d_summary_by_source_format_relative_to;
    reader_summary_format = read_discovered_bundle_2d_summary_by_source_format;
    reader_summary_formats_relative = read_discovered_bundle_2d_summary_by_source_formats_relative_to;
    reader_summary_formats = read_discovered_bundle_2d_summary_by_source_formats;
    reader_summary_vendor_relative = read_discovered_bundle_2d_summary_by_source_vendor_relative_to;
    reader_summary_vendor = read_discovered_bundle_2d_summary_by_source_vendor;
    reader_summary_vendors_relative = read_discovered_bundle_2d_summary_by_source_vendors_relative_to;
    reader_summary_vendors = read_discovered_bundle_2d_summary_by_source_vendors;
    reader_summary_data_kind_relative = read_discovered_bundle_2d_summary_by_source_data_kind_relative_to;
    reader_summary_data_kind = read_discovered_bundle_2d_summary_by_source_data_kind;
    reader_summary_data_kinds_relative = read_discovered_bundle_2d_summary_by_source_data_kinds_relative_to;
    reader_summary_data_kinds = read_discovered_bundle_2d_summary_by_source_data_kinds;

    load_format_relative = load_discovered_spectra_2d_by_source_format_relative_to;
    load_format = load_discovered_spectra_2d_by_source_format;
    load_formats_relative = load_discovered_spectra_2d_by_source_formats_relative_to;
    load_formats = load_discovered_spectra_2d_by_source_formats;
    load_vendor_relative = load_discovered_spectra_2d_by_source_vendor_relative_to;
    load_vendor = load_discovered_spectra_2d_by_source_vendor;
    load_vendors_relative = load_discovered_spectra_2d_by_source_vendors_relative_to;
    load_vendors = load_discovered_spectra_2d_by_source_vendors;
    load_data_kind_relative = load_discovered_spectra_2d_by_source_data_kind_relative_to;
    load_data_kind = load_discovered_spectra_2d_by_source_data_kind;
    load_data_kinds_relative = load_discovered_spectra_2d_by_source_data_kinds_relative_to;
    load_data_kinds = load_discovered_spectra_2d_by_source_data_kinds;

    strict_format_relative = load_discovered_spectra_2d_strict_by_source_format_relative_to;
    strict_format = load_discovered_spectra_2d_strict_by_source_format;
    strict_formats_relative = load_discovered_spectra_2d_strict_by_source_formats_relative_to;
    strict_formats = load_discovered_spectra_2d_strict_by_source_formats;
    strict_vendor_relative = load_discovered_spectra_2d_strict_by_source_vendor_relative_to;
    strict_vendor = load_discovered_spectra_2d_strict_by_source_vendor;
    strict_vendors_relative = load_discovered_spectra_2d_strict_by_source_vendors_relative_to;
    strict_vendors = load_discovered_spectra_2d_strict_by_source_vendors;
    strict_data_kind_relative = load_discovered_spectra_2d_strict_by_source_data_kind_relative_to;
    strict_data_kind = load_discovered_spectra_2d_strict_by_source_data_kind;
    strict_data_kinds_relative = load_discovered_spectra_2d_strict_by_source_data_kinds_relative_to;
    strict_data_kinds = load_discovered_spectra_2d_strict_by_source_data_kinds;

    summary_format_relative = load_discovered_spectra_2d_summary_by_source_format_relative_to;
    summary_format = load_discovered_spectra_2d_summary_by_source_format;
    summary_formats_relative = load_discovered_spectra_2d_summary_by_source_formats_relative_to;
    summary_formats = load_discovered_spectra_2d_summary_by_source_formats;
    summary_vendor_relative = load_discovered_spectra_2d_summary_by_source_vendor_relative_to;
    summary_vendor = load_discovered_spectra_2d_summary_by_source_vendor;
    summary_vendors_relative = load_discovered_spectra_2d_summary_by_source_vendors_relative_to;
    summary_vendors = load_discovered_spectra_2d_summary_by_source_vendors;
    summary_data_kind_relative = load_discovered_spectra_2d_summary_by_source_data_kind_relative_to;
    summary_data_kind = load_discovered_spectra_2d_summary_by_source_data_kind;
    summary_data_kinds_relative = load_discovered_spectra_2d_summary_by_source_data_kinds_relative_to;
    summary_data_kinds = load_discovered_spectra_2d_summary_by_source_data_kinds;

    strict_summary_format_relative = load_discovered_spectra_2d_summary_strict_by_source_format_relative_to;
    strict_summary_format = load_discovered_spectra_2d_summary_strict_by_source_format;
    strict_summary_formats_relative = load_discovered_spectra_2d_summary_strict_by_source_formats_relative_to;
    strict_summary_formats = load_discovered_spectra_2d_summary_strict_by_source_formats;
    strict_summary_vendor_relative = load_discovered_spectra_2d_summary_strict_by_source_vendor_relative_to;
    strict_summary_vendor = load_discovered_spectra_2d_summary_strict_by_source_vendor;
    strict_summary_vendors_relative = load_discovered_spectra_2d_summary_strict_by_source_vendors_relative_to;
    strict_summary_vendors = load_discovered_spectra_2d_summary_strict_by_source_vendors;
    strict_summary_data_kind_relative = load_discovered_spectra_2d_summary_strict_by_source_data_kind_relative_to;
    strict_summary_data_kind = load_discovered_spectra_2d_summary_strict_by_source_data_kind;
    strict_summary_data_kinds_relative = load_discovered_spectra_2d_summary_strict_by_source_data_kinds_relative_to;
    strict_summary_data_kinds = load_discovered_spectra_2d_summary_strict_by_source_data_kinds;
}

fn format_filters<I, F>(formats: I) -> Vec<LoadedSourceFilter>
where
    I: IntoIterator<Item = F>,
    F: AsRef<str>,
{
    let mut filters = Vec::new();
    for format in formats {
        filters.push(LoadedSourceFilter::format(format));
    }
    filters
}

fn vendor_filters<I, V>(vendors: I) -> Vec<LoadedSourceFilter>
where
    I: IntoIterator<Item = V>,
    V: AsRef<str>,
{
    let mut filters = Vec::new();
    for vendor in vendors {
        filters.push(LoadedSourceFilter::vendor(vendor));
    }
    filters
}

fn data_kind_filters<I>(data_kinds: I) -> Vec<LoadedSourceFilter>
where
    I: IntoIterator<Item = LoadedSourceDataKind>,
{
    let mut filters = Vec::new();
    for data_kind in data_kinds {
        filters.push(LoadedSourceFilter::data_kind(data_kind));
    }
    filters
}
