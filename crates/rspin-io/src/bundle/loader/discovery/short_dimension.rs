//! Short dimension-specific bundle loaders for discovered source candidates.

use std::path::Path;

use rspin_core::Result;

use super::DiscoveredSpectrumSource;
use crate::bundle::{
    LoadedSourceDataKind, SpectrumBundle, SpectrumBundleLoader, SpectrumBundleSummary,
};

macro_rules! dimension_single_bundle_aliases {
    (
        dimension = $dimension:literal;
        filter = $filter_doc:literal;
        value = $value:ident : $value_ty:ty;
        reader_relative = $reader_relative:ident;
        reader = $reader:ident;
        free_relative = $free_relative:ident;
        free = $free:ident;
        target_relative = $target_relative:ident;
        target = $target:ident;
    ) => {
        impl SpectrumBundleLoader {
            #[doc = concat!("Loads ", $dimension, " discovered candidates matching one ", $filter_doc, " as a bundle.")]
            ///
            /// This is a short alias for the matching
            /// `read_discovered_bundle_*_by_source_*` method.
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum
            /// data is found.
            pub fn $reader_relative<'a>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                $value: $value_ty,
            ) -> Result<SpectrumBundle> {
                self.$target_relative(base, sources, $value)
            }

            #[doc = concat!("Loads ", $dimension, " discovered candidates matching one ", $filter_doc, " as a bundle.")]
            ///
            /// This is a short alias for the matching
            /// `read_discovered_bundle_*_by_source_*` method.
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum
            /// data is found.
            pub fn $reader<'a>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                $value: $value_ty,
            ) -> Result<SpectrumBundle> {
                self.$target(base, sources, $value)
            }
        }

        #[doc = concat!("Loads ", $dimension, " discovered candidates matching one ", $filter_doc, " as a bundle.")]
        ///
        /// This is a short alias for the matching
        /// `load_discovered_spectra_*_by_source_*` function.
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $free_relative<'a>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            $value: $value_ty,
        ) -> Result<SpectrumBundle> {
            SpectrumBundleLoader::new().$reader_relative(base, sources, $value)
        }

        #[doc = concat!("Loads ", $dimension, " discovered candidates matching one ", $filter_doc, " as a bundle.")]
        ///
        /// This is a short alias for the matching
        /// `load_discovered_spectra_*_by_source_*` function.
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $free<'a>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            $value: $value_ty,
        ) -> Result<SpectrumBundle> {
            SpectrumBundleLoader::new().$reader(base, sources, $value)
        }
    };
}

macro_rules! dimension_set_bundle_aliases {
    (
        dimension = $dimension:literal;
        filter = $filter_doc:literal;
        values = $values:ident;
        generics = [$($generics:tt)*];
        where = {$($where_clause:tt)*};
        reader_relative = $reader_relative:ident;
        reader = $reader:ident;
        free_relative = $free_relative:ident;
        free = $free:ident;
        target_relative = $target_relative:ident;
        target = $target:ident;
    ) => {
        impl SpectrumBundleLoader {
            #[doc = concat!("Loads ", $dimension, " discovered candidates matching any ", $filter_doc, " as a bundle.")]
            ///
            /// Values are combined with logical OR. Passing an empty iterator
            /// leaves source matching unrestricted before dimension filtering.
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum
            /// data is found.
            pub fn $reader_relative<'a, $($generics)*>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                $values: I,
            ) -> Result<SpectrumBundle>
            where
                $($where_clause)*
            {
                self.$target_relative(base, sources, $values)
            }

            #[doc = concat!("Loads ", $dimension, " discovered candidates matching any ", $filter_doc, " as a bundle.")]
            ///
            /// Values are combined with logical OR. Passing an empty iterator
            /// leaves source matching unrestricted before dimension filtering.
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum
            /// data is found.
            pub fn $reader<'a, $($generics)*>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                $values: I,
            ) -> Result<SpectrumBundle>
            where
                $($where_clause)*
            {
                self.$target(base, sources, $values)
            }
        }

        #[doc = concat!("Loads ", $dimension, " discovered candidates matching any ", $filter_doc, " as a bundle.")]
        ///
        /// Values are combined with logical OR. Passing an empty iterator
        /// leaves source matching unrestricted before dimension filtering.
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $free_relative<'a, $($generics)*>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            $values: I,
        ) -> Result<SpectrumBundle>
        where
            $($where_clause)*
        {
            SpectrumBundleLoader::new().$reader_relative(base, sources, $values)
        }

        #[doc = concat!("Loads ", $dimension, " discovered candidates matching any ", $filter_doc, " as a bundle.")]
        ///
        /// Values are combined with logical OR. Passing an empty iterator
        /// leaves source matching unrestricted before dimension filtering.
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $free<'a, $($generics)*>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            $values: I,
        ) -> Result<SpectrumBundle>
        where
            $($where_clause)*
        {
            SpectrumBundleLoader::new().$reader(base, sources, $values)
        }
    };
}

macro_rules! dimension_single_summary_aliases {
    (
        dimension = $dimension:literal;
        filter = $filter_doc:literal;
        value = $value:ident : $value_ty:ty;
        reader_relative = $reader_relative:ident;
        reader = $reader:ident;
        free_relative = $free_relative:ident;
        free = $free:ident;
        target_relative = $target_relative:ident;
        target = $target:ident;
    ) => {
        impl SpectrumBundleLoader {
            #[doc = concat!("Loads ", $dimension, " discovered candidates matching one ", $filter_doc, " and returns summary counts.")]
            ///
            /// This is a short alias for the matching
            /// `read_discovered_bundle_*_summary_by_source_*` method.
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum
            /// data is found.
            pub fn $reader_relative<'a>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                $value: $value_ty,
            ) -> Result<SpectrumBundleSummary> {
                self.$target_relative(base, sources, $value)
            }

            #[doc = concat!("Loads ", $dimension, " discovered candidates matching one ", $filter_doc, " and returns summary counts.")]
            ///
            /// This is a short alias for the matching
            /// `read_discovered_bundle_*_summary_by_source_*` method.
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum
            /// data is found.
            pub fn $reader<'a>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                $value: $value_ty,
            ) -> Result<SpectrumBundleSummary> {
                self.$target(base, sources, $value)
            }
        }

        #[doc = concat!("Loads ", $dimension, " discovered candidates matching one ", $filter_doc, " and returns summary counts.")]
        ///
        /// This is a short alias for the matching
        /// `load_discovered_spectra_*_summary_by_source_*` function.
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $free_relative<'a>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            $value: $value_ty,
        ) -> Result<SpectrumBundleSummary> {
            SpectrumBundleLoader::new().$reader_relative(base, sources, $value)
        }

        #[doc = concat!("Loads ", $dimension, " discovered candidates matching one ", $filter_doc, " and returns summary counts.")]
        ///
        /// This is a short alias for the matching
        /// `load_discovered_spectra_*_summary_by_source_*` function.
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $free<'a>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            $value: $value_ty,
        ) -> Result<SpectrumBundleSummary> {
            SpectrumBundleLoader::new().$reader(base, sources, $value)
        }
    };
}

macro_rules! dimension_set_summary_aliases {
    (
        dimension = $dimension:literal;
        filter = $filter_doc:literal;
        values = $values:ident;
        generics = [$($generics:tt)*];
        where = {$($where_clause:tt)*};
        reader_relative = $reader_relative:ident;
        reader = $reader:ident;
        free_relative = $free_relative:ident;
        free = $free:ident;
        target_relative = $target_relative:ident;
        target = $target:ident;
    ) => {
        impl SpectrumBundleLoader {
            #[doc = concat!("Loads ", $dimension, " discovered candidates matching any ", $filter_doc, " and returns summary counts.")]
            ///
            /// Values are combined with logical OR. Passing an empty iterator
            /// leaves source matching unrestricted before dimension filtering.
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum
            /// data is found.
            pub fn $reader_relative<'a, $($generics)*>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                $values: I,
            ) -> Result<SpectrumBundleSummary>
            where
                $($where_clause)*
            {
                self.$target_relative(base, sources, $values)
            }

            #[doc = concat!("Loads ", $dimension, " discovered candidates matching any ", $filter_doc, " and returns summary counts.")]
            ///
            /// Values are combined with logical OR. Passing an empty iterator
            /// leaves source matching unrestricted before dimension filtering.
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum
            /// data is found.
            pub fn $reader<'a, $($generics)*>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                $values: I,
            ) -> Result<SpectrumBundleSummary>
            where
                $($where_clause)*
            {
                self.$target(base, sources, $values)
            }
        }

        #[doc = concat!("Loads ", $dimension, " discovered candidates matching any ", $filter_doc, " and returns summary counts.")]
        ///
        /// Values are combined with logical OR. Passing an empty iterator
        /// leaves source matching unrestricted before dimension filtering.
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $free_relative<'a, $($generics)*>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            $values: I,
        ) -> Result<SpectrumBundleSummary>
        where
            $($where_clause)*
        {
            SpectrumBundleLoader::new().$reader_relative(base, sources, $values)
        }

        #[doc = concat!("Loads ", $dimension, " discovered candidates matching any ", $filter_doc, " and returns summary counts.")]
        ///
        /// Values are combined with logical OR. Passing an empty iterator
        /// leaves source matching unrestricted before dimension filtering.
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $free<'a, $($generics)*>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            $values: I,
        ) -> Result<SpectrumBundleSummary>
        where
            $($where_clause)*
        {
            SpectrumBundleLoader::new().$reader(base, sources, $values)
        }
    };
}

macro_rules! dimension_short_metadata_aliases {
    (
        dimension = $dimension:literal;
        load_prefix = $load_prefix:ident;

        format = [$format_reader_relative:ident, $format_reader:ident, $format_free_relative:ident, $format_free:ident, $format_target_relative:ident, $format_target:ident];
        formats = [$formats_reader_relative:ident, $formats_reader:ident, $formats_free_relative:ident, $formats_free:ident, $formats_target_relative:ident, $formats_target:ident];
        vendor = [$vendor_reader_relative:ident, $vendor_reader:ident, $vendor_free_relative:ident, $vendor_free:ident, $vendor_target_relative:ident, $vendor_target:ident];
        vendors = [$vendors_reader_relative:ident, $vendors_reader:ident, $vendors_free_relative:ident, $vendors_free:ident, $vendors_target_relative:ident, $vendors_target:ident];
        data_kind = [$data_kind_reader_relative:ident, $data_kind_reader:ident, $data_kind_free_relative:ident, $data_kind_free:ident, $data_kind_target_relative:ident, $data_kind_target:ident];
        data_kinds = [$data_kinds_reader_relative:ident, $data_kinds_reader:ident, $data_kinds_free_relative:ident, $data_kinds_free:ident, $data_kinds_target_relative:ident, $data_kinds_target:ident];
        path_prefix = [$prefix_reader_relative:ident, $prefix_reader:ident, $prefix_free_relative:ident, $prefix_free:ident, $prefix_target_relative:ident, $prefix_target:ident];
        path_prefixes = [$prefixes_reader_relative:ident, $prefixes_reader:ident, $prefixes_free_relative:ident, $prefixes_free:ident, $prefixes_target_relative:ident, $prefixes_target:ident];

        summary_format = [$summary_format_reader_relative:ident, $summary_format_reader:ident, $summary_format_free_relative:ident, $summary_format_free:ident, $summary_format_target_relative:ident, $summary_format_target:ident];
        summary_formats = [$summary_formats_reader_relative:ident, $summary_formats_reader:ident, $summary_formats_free_relative:ident, $summary_formats_free:ident, $summary_formats_target_relative:ident, $summary_formats_target:ident];
        summary_vendor = [$summary_vendor_reader_relative:ident, $summary_vendor_reader:ident, $summary_vendor_free_relative:ident, $summary_vendor_free:ident, $summary_vendor_target_relative:ident, $summary_vendor_target:ident];
        summary_vendors = [$summary_vendors_reader_relative:ident, $summary_vendors_reader:ident, $summary_vendors_free_relative:ident, $summary_vendors_free:ident, $summary_vendors_target_relative:ident, $summary_vendors_target:ident];
        summary_data_kind = [$summary_data_kind_reader_relative:ident, $summary_data_kind_reader:ident, $summary_data_kind_free_relative:ident, $summary_data_kind_free:ident, $summary_data_kind_target_relative:ident, $summary_data_kind_target:ident];
        summary_data_kinds = [$summary_data_kinds_reader_relative:ident, $summary_data_kinds_reader:ident, $summary_data_kinds_free_relative:ident, $summary_data_kinds_free:ident, $summary_data_kinds_target_relative:ident, $summary_data_kinds_target:ident];
        summary_path_prefix = [$summary_prefix_reader_relative:ident, $summary_prefix_reader:ident, $summary_prefix_free_relative:ident, $summary_prefix_free:ident, $summary_prefixes_target_relative:ident, $summary_prefixes_target:ident];
        summary_path_prefixes = [$summary_prefixes_reader_relative:ident, $summary_prefixes_reader:ident, $summary_prefixes_free_relative:ident, $summary_prefixes_free:ident, $summary_prefixes_target_relative_again:ident, $summary_prefixes_target_again:ident];
    ) => {
        dimension_single_bundle_aliases! {
            dimension = $dimension;
            filter = "source format";
            value = format: impl AsRef<str>;
            reader_relative = $format_reader_relative;
            reader = $format_reader;
            free_relative = $format_free_relative;
            free = $format_free;
            target_relative = $format_target_relative;
            target = $format_target;
        }

        dimension_set_bundle_aliases! {
            dimension = $dimension;
            filter = "source format";
            values = formats;
            generics = [I, F];
            where = {
                I: IntoIterator<Item = F>,
                F: AsRef<str>,
            };
            reader_relative = $formats_reader_relative;
            reader = $formats_reader;
            free_relative = $formats_free_relative;
            free = $formats_free;
            target_relative = $formats_target_relative;
            target = $formats_target;
        }

        dimension_single_bundle_aliases! {
            dimension = $dimension;
            filter = "vendor family";
            value = vendor: impl AsRef<str>;
            reader_relative = $vendor_reader_relative;
            reader = $vendor_reader;
            free_relative = $vendor_free_relative;
            free = $vendor_free;
            target_relative = $vendor_target_relative;
            target = $vendor_target;
        }

        dimension_set_bundle_aliases! {
            dimension = $dimension;
            filter = "vendor family";
            values = vendors;
            generics = [I, V];
            where = {
                I: IntoIterator<Item = V>,
                V: AsRef<str>,
            };
            reader_relative = $vendors_reader_relative;
            reader = $vendors_reader;
            free_relative = $vendors_free_relative;
            free = $vendors_free;
            target_relative = $vendors_target_relative;
            target = $vendors_target;
        }

        dimension_single_bundle_aliases! {
            dimension = $dimension;
            filter = "raw/processed source data kind";
            value = data_kind: LoadedSourceDataKind;
            reader_relative = $data_kind_reader_relative;
            reader = $data_kind_reader;
            free_relative = $data_kind_free_relative;
            free = $data_kind_free;
            target_relative = $data_kind_target_relative;
            target = $data_kind_target;
        }

        dimension_set_bundle_aliases! {
            dimension = $dimension;
            filter = "raw/processed source data kind";
            values = data_kinds;
            generics = [I];
            where = {
                I: IntoIterator<Item = LoadedSourceDataKind>,
            };
            reader_relative = $data_kinds_reader_relative;
            reader = $data_kinds_reader;
            free_relative = $data_kinds_free_relative;
            free = $data_kinds_free;
            target_relative = $data_kinds_target_relative;
            target = $data_kinds_target;
        }

        dimension_single_bundle_aliases! {
            dimension = $dimension;
            filter = "tracked source path prefix";
            value = source_path_prefix: impl AsRef<Path>;
            reader_relative = $prefix_reader_relative;
            reader = $prefix_reader;
            free_relative = $prefix_free_relative;
            free = $prefix_free;
            target_relative = $prefix_target_relative;
            target = $prefix_target;
        }

        dimension_set_bundle_aliases! {
            dimension = $dimension;
            filter = "tracked source path prefix";
            values = source_path_prefixes;
            generics = [I, P];
            where = {
                I: IntoIterator<Item = P>,
                P: AsRef<Path>,
            };
            reader_relative = $prefixes_reader_relative;
            reader = $prefixes_reader;
            free_relative = $prefixes_free_relative;
            free = $prefixes_free;
            target_relative = $prefixes_target_relative;
            target = $prefixes_target;
        }

        dimension_single_summary_aliases! {
            dimension = $dimension;
            filter = "source format";
            value = format: impl AsRef<str>;
            reader_relative = $summary_format_reader_relative;
            reader = $summary_format_reader;
            free_relative = $summary_format_free_relative;
            free = $summary_format_free;
            target_relative = $summary_format_target_relative;
            target = $summary_format_target;
        }

        dimension_set_summary_aliases! {
            dimension = $dimension;
            filter = "source format";
            values = formats;
            generics = [I, F];
            where = {
                I: IntoIterator<Item = F>,
                F: AsRef<str>,
            };
            reader_relative = $summary_formats_reader_relative;
            reader = $summary_formats_reader;
            free_relative = $summary_formats_free_relative;
            free = $summary_formats_free;
            target_relative = $summary_formats_target_relative;
            target = $summary_formats_target;
        }

        dimension_single_summary_aliases! {
            dimension = $dimension;
            filter = "vendor family";
            value = vendor: impl AsRef<str>;
            reader_relative = $summary_vendor_reader_relative;
            reader = $summary_vendor_reader;
            free_relative = $summary_vendor_free_relative;
            free = $summary_vendor_free;
            target_relative = $summary_vendor_target_relative;
            target = $summary_vendor_target;
        }

        dimension_set_summary_aliases! {
            dimension = $dimension;
            filter = "vendor family";
            values = vendors;
            generics = [I, V];
            where = {
                I: IntoIterator<Item = V>,
                V: AsRef<str>,
            };
            reader_relative = $summary_vendors_reader_relative;
            reader = $summary_vendors_reader;
            free_relative = $summary_vendors_free_relative;
            free = $summary_vendors_free;
            target_relative = $summary_vendors_target_relative;
            target = $summary_vendors_target;
        }

        dimension_single_summary_aliases! {
            dimension = $dimension;
            filter = "raw/processed source data kind";
            value = data_kind: LoadedSourceDataKind;
            reader_relative = $summary_data_kind_reader_relative;
            reader = $summary_data_kind_reader;
            free_relative = $summary_data_kind_free_relative;
            free = $summary_data_kind_free;
            target_relative = $summary_data_kind_target_relative;
            target = $summary_data_kind_target;
        }

        dimension_set_summary_aliases! {
            dimension = $dimension;
            filter = "raw/processed source data kind";
            values = data_kinds;
            generics = [I];
            where = {
                I: IntoIterator<Item = LoadedSourceDataKind>,
            };
            reader_relative = $summary_data_kinds_reader_relative;
            reader = $summary_data_kinds_reader;
            free_relative = $summary_data_kinds_free_relative;
            free = $summary_data_kinds_free;
            target_relative = $summary_data_kinds_target_relative;
            target = $summary_data_kinds_target;
        }

        impl SpectrumBundleLoader {
            #[doc = concat!("Loads ", $dimension, " discovered candidates below one tracked source path prefix and returns summary counts.")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum
            /// data is found.
            pub fn $summary_prefix_reader_relative<'a>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                source_path_prefix: impl AsRef<Path>,
            ) -> Result<SpectrumBundleSummary> {
                self.$summary_prefixes_target_relative(base, sources, [source_path_prefix])
            }

            #[doc = concat!("Loads ", $dimension, " discovered candidates below one tracked source path prefix and returns summary counts.")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum
            /// data is found.
            pub fn $summary_prefix_reader<'a>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                source_path_prefix: impl AsRef<Path>,
            ) -> Result<SpectrumBundleSummary> {
                self.$summary_prefixes_target(base, sources, [source_path_prefix])
            }
        }

        #[doc = concat!("Loads ", $dimension, " discovered candidates below one tracked source path prefix and returns summary counts.")]
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $summary_prefix_free_relative<'a>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            source_path_prefix: impl AsRef<Path>,
        ) -> Result<SpectrumBundleSummary> {
            SpectrumBundleLoader::new().$summary_prefix_reader_relative(
                base,
                sources,
                source_path_prefix,
            )
        }

        #[doc = concat!("Loads ", $dimension, " discovered candidates below one tracked source path prefix and returns summary counts.")]
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $summary_prefix_free<'a>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            source_path_prefix: impl AsRef<Path>,
        ) -> Result<SpectrumBundleSummary> {
            SpectrumBundleLoader::new().$summary_prefix_reader(base, sources, source_path_prefix)
        }

        dimension_set_summary_aliases! {
            dimension = $dimension;
            filter = "tracked source path prefix";
            values = source_path_prefixes;
            generics = [I, P];
            where = {
                I: IntoIterator<Item = P>,
                P: AsRef<Path>,
            };
            reader_relative = $summary_prefixes_reader_relative;
            reader = $summary_prefixes_reader;
            free_relative = $summary_prefixes_free_relative;
            free = $summary_prefixes_free;
            target_relative = $summary_prefixes_target_relative_again;
            target = $summary_prefixes_target_again;
        }
    };
}

dimension_short_metadata_aliases! {
    dimension = "one-dimensional";
    load_prefix = one_d;

    format = [read_discovered_bundle_1d_by_format_relative_to, read_discovered_bundle_1d_by_format, load_discovered_spectra_1d_by_format_relative_to, load_discovered_spectra_1d_by_format, read_discovered_bundle_1d_by_source_format_relative_to, read_discovered_bundle_1d_by_source_format];
    formats = [read_discovered_bundle_1d_by_formats_relative_to, read_discovered_bundle_1d_by_formats, load_discovered_spectra_1d_by_formats_relative_to, load_discovered_spectra_1d_by_formats, read_discovered_bundle_1d_by_source_formats_relative_to, read_discovered_bundle_1d_by_source_formats];
    vendor = [read_discovered_bundle_1d_by_vendor_relative_to, read_discovered_bundle_1d_by_vendor, load_discovered_spectra_1d_by_vendor_relative_to, load_discovered_spectra_1d_by_vendor, read_discovered_bundle_1d_by_source_vendor_relative_to, read_discovered_bundle_1d_by_source_vendor];
    vendors = [read_discovered_bundle_1d_by_vendors_relative_to, read_discovered_bundle_1d_by_vendors, load_discovered_spectra_1d_by_vendors_relative_to, load_discovered_spectra_1d_by_vendors, read_discovered_bundle_1d_by_source_vendors_relative_to, read_discovered_bundle_1d_by_source_vendors];
    data_kind = [read_discovered_bundle_1d_by_data_kind_relative_to, read_discovered_bundle_1d_by_data_kind, load_discovered_spectra_1d_by_data_kind_relative_to, load_discovered_spectra_1d_by_data_kind, read_discovered_bundle_1d_by_source_data_kind_relative_to, read_discovered_bundle_1d_by_source_data_kind];
    data_kinds = [read_discovered_bundle_1d_by_data_kinds_relative_to, read_discovered_bundle_1d_by_data_kinds, load_discovered_spectra_1d_by_data_kinds_relative_to, load_discovered_spectra_1d_by_data_kinds, read_discovered_bundle_1d_by_source_data_kinds_relative_to, read_discovered_bundle_1d_by_source_data_kinds];
    path_prefix = [read_discovered_bundle_1d_by_path_prefix_relative_to, read_discovered_bundle_1d_by_path_prefix, load_discovered_spectra_1d_by_path_prefix_relative_to, load_discovered_spectra_1d_by_path_prefix, read_discovered_bundle_1d_by_source_path_prefix_relative_to, read_discovered_bundle_1d_by_source_path_prefix];
    path_prefixes = [read_discovered_bundle_1d_by_path_prefixes_relative_to, read_discovered_bundle_1d_by_path_prefixes, load_discovered_spectra_1d_by_path_prefixes_relative_to, load_discovered_spectra_1d_by_path_prefixes, read_discovered_bundle_1d_by_source_path_prefixes_relative_to, read_discovered_bundle_1d_by_source_path_prefixes];

    summary_format = [read_discovered_bundle_1d_summary_by_format_relative_to, read_discovered_bundle_1d_summary_by_format, load_discovered_spectra_1d_summary_by_format_relative_to, load_discovered_spectra_1d_summary_by_format, read_discovered_bundle_1d_summary_by_source_format_relative_to, read_discovered_bundle_1d_summary_by_source_format];
    summary_formats = [read_discovered_bundle_1d_summary_by_formats_relative_to, read_discovered_bundle_1d_summary_by_formats, load_discovered_spectra_1d_summary_by_formats_relative_to, load_discovered_spectra_1d_summary_by_formats, read_discovered_bundle_1d_summary_by_source_formats_relative_to, read_discovered_bundle_1d_summary_by_source_formats];
    summary_vendor = [read_discovered_bundle_1d_summary_by_vendor_relative_to, read_discovered_bundle_1d_summary_by_vendor, load_discovered_spectra_1d_summary_by_vendor_relative_to, load_discovered_spectra_1d_summary_by_vendor, read_discovered_bundle_1d_summary_by_source_vendor_relative_to, read_discovered_bundle_1d_summary_by_source_vendor];
    summary_vendors = [read_discovered_bundle_1d_summary_by_vendors_relative_to, read_discovered_bundle_1d_summary_by_vendors, load_discovered_spectra_1d_summary_by_vendors_relative_to, load_discovered_spectra_1d_summary_by_vendors, read_discovered_bundle_1d_summary_by_source_vendors_relative_to, read_discovered_bundle_1d_summary_by_source_vendors];
    summary_data_kind = [read_discovered_bundle_1d_summary_by_data_kind_relative_to, read_discovered_bundle_1d_summary_by_data_kind, load_discovered_spectra_1d_summary_by_data_kind_relative_to, load_discovered_spectra_1d_summary_by_data_kind, read_discovered_bundle_1d_summary_by_source_data_kind_relative_to, read_discovered_bundle_1d_summary_by_source_data_kind];
    summary_data_kinds = [read_discovered_bundle_1d_summary_by_data_kinds_relative_to, read_discovered_bundle_1d_summary_by_data_kinds, load_discovered_spectra_1d_summary_by_data_kinds_relative_to, load_discovered_spectra_1d_summary_by_data_kinds, read_discovered_bundle_1d_summary_by_source_data_kinds_relative_to, read_discovered_bundle_1d_summary_by_source_data_kinds];
    summary_path_prefix = [read_discovered_bundle_1d_summary_by_path_prefix_relative_to, read_discovered_bundle_1d_summary_by_path_prefix, load_discovered_spectra_1d_summary_by_path_prefix_relative_to, load_discovered_spectra_1d_summary_by_path_prefix, read_discovered_bundle_1d_summary_by_source_path_prefixes_relative_to, read_discovered_bundle_1d_summary_by_source_path_prefixes];
    summary_path_prefixes = [read_discovered_bundle_1d_summary_by_path_prefixes_relative_to, read_discovered_bundle_1d_summary_by_path_prefixes, load_discovered_spectra_1d_summary_by_path_prefixes_relative_to, load_discovered_spectra_1d_summary_by_path_prefixes, read_discovered_bundle_1d_summary_by_source_path_prefixes_relative_to, read_discovered_bundle_1d_summary_by_source_path_prefixes];
}

dimension_short_metadata_aliases! {
    dimension = "two-dimensional";
    load_prefix = two_d;

    format = [read_discovered_bundle_2d_by_format_relative_to, read_discovered_bundle_2d_by_format, load_discovered_spectra_2d_by_format_relative_to, load_discovered_spectra_2d_by_format, read_discovered_bundle_2d_by_source_format_relative_to, read_discovered_bundle_2d_by_source_format];
    formats = [read_discovered_bundle_2d_by_formats_relative_to, read_discovered_bundle_2d_by_formats, load_discovered_spectra_2d_by_formats_relative_to, load_discovered_spectra_2d_by_formats, read_discovered_bundle_2d_by_source_formats_relative_to, read_discovered_bundle_2d_by_source_formats];
    vendor = [read_discovered_bundle_2d_by_vendor_relative_to, read_discovered_bundle_2d_by_vendor, load_discovered_spectra_2d_by_vendor_relative_to, load_discovered_spectra_2d_by_vendor, read_discovered_bundle_2d_by_source_vendor_relative_to, read_discovered_bundle_2d_by_source_vendor];
    vendors = [read_discovered_bundle_2d_by_vendors_relative_to, read_discovered_bundle_2d_by_vendors, load_discovered_spectra_2d_by_vendors_relative_to, load_discovered_spectra_2d_by_vendors, read_discovered_bundle_2d_by_source_vendors_relative_to, read_discovered_bundle_2d_by_source_vendors];
    data_kind = [read_discovered_bundle_2d_by_data_kind_relative_to, read_discovered_bundle_2d_by_data_kind, load_discovered_spectra_2d_by_data_kind_relative_to, load_discovered_spectra_2d_by_data_kind, read_discovered_bundle_2d_by_source_data_kind_relative_to, read_discovered_bundle_2d_by_source_data_kind];
    data_kinds = [read_discovered_bundle_2d_by_data_kinds_relative_to, read_discovered_bundle_2d_by_data_kinds, load_discovered_spectra_2d_by_data_kinds_relative_to, load_discovered_spectra_2d_by_data_kinds, read_discovered_bundle_2d_by_source_data_kinds_relative_to, read_discovered_bundle_2d_by_source_data_kinds];
    path_prefix = [read_discovered_bundle_2d_by_path_prefix_relative_to, read_discovered_bundle_2d_by_path_prefix, load_discovered_spectra_2d_by_path_prefix_relative_to, load_discovered_spectra_2d_by_path_prefix, read_discovered_bundle_2d_by_source_path_prefix_relative_to, read_discovered_bundle_2d_by_source_path_prefix];
    path_prefixes = [read_discovered_bundle_2d_by_path_prefixes_relative_to, read_discovered_bundle_2d_by_path_prefixes, load_discovered_spectra_2d_by_path_prefixes_relative_to, load_discovered_spectra_2d_by_path_prefixes, read_discovered_bundle_2d_by_source_path_prefixes_relative_to, read_discovered_bundle_2d_by_source_path_prefixes];

    summary_format = [read_discovered_bundle_2d_summary_by_format_relative_to, read_discovered_bundle_2d_summary_by_format, load_discovered_spectra_2d_summary_by_format_relative_to, load_discovered_spectra_2d_summary_by_format, read_discovered_bundle_2d_summary_by_source_format_relative_to, read_discovered_bundle_2d_summary_by_source_format];
    summary_formats = [read_discovered_bundle_2d_summary_by_formats_relative_to, read_discovered_bundle_2d_summary_by_formats, load_discovered_spectra_2d_summary_by_formats_relative_to, load_discovered_spectra_2d_summary_by_formats, read_discovered_bundle_2d_summary_by_source_formats_relative_to, read_discovered_bundle_2d_summary_by_source_formats];
    summary_vendor = [read_discovered_bundle_2d_summary_by_vendor_relative_to, read_discovered_bundle_2d_summary_by_vendor, load_discovered_spectra_2d_summary_by_vendor_relative_to, load_discovered_spectra_2d_summary_by_vendor, read_discovered_bundle_2d_summary_by_source_vendor_relative_to, read_discovered_bundle_2d_summary_by_source_vendor];
    summary_vendors = [read_discovered_bundle_2d_summary_by_vendors_relative_to, read_discovered_bundle_2d_summary_by_vendors, load_discovered_spectra_2d_summary_by_vendors_relative_to, load_discovered_spectra_2d_summary_by_vendors, read_discovered_bundle_2d_summary_by_source_vendors_relative_to, read_discovered_bundle_2d_summary_by_source_vendors];
    summary_data_kind = [read_discovered_bundle_2d_summary_by_data_kind_relative_to, read_discovered_bundle_2d_summary_by_data_kind, load_discovered_spectra_2d_summary_by_data_kind_relative_to, load_discovered_spectra_2d_summary_by_data_kind, read_discovered_bundle_2d_summary_by_source_data_kind_relative_to, read_discovered_bundle_2d_summary_by_source_data_kind];
    summary_data_kinds = [read_discovered_bundle_2d_summary_by_data_kinds_relative_to, read_discovered_bundle_2d_summary_by_data_kinds, load_discovered_spectra_2d_summary_by_data_kinds_relative_to, load_discovered_spectra_2d_summary_by_data_kinds, read_discovered_bundle_2d_summary_by_source_data_kinds_relative_to, read_discovered_bundle_2d_summary_by_source_data_kinds];
    summary_path_prefix = [read_discovered_bundle_2d_summary_by_path_prefix_relative_to, read_discovered_bundle_2d_summary_by_path_prefix, load_discovered_spectra_2d_summary_by_path_prefix_relative_to, load_discovered_spectra_2d_summary_by_path_prefix, read_discovered_bundle_2d_summary_by_source_path_prefixes_relative_to, read_discovered_bundle_2d_summary_by_source_path_prefixes];
    summary_path_prefixes = [read_discovered_bundle_2d_summary_by_path_prefixes_relative_to, read_discovered_bundle_2d_summary_by_path_prefixes, load_discovered_spectra_2d_summary_by_path_prefixes_relative_to, load_discovered_spectra_2d_summary_by_path_prefixes, read_discovered_bundle_2d_summary_by_source_path_prefixes_relative_to, read_discovered_bundle_2d_summary_by_source_path_prefixes];
}
