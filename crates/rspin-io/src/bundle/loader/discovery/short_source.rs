//! Short source-filtered bundle loaders for discovered source candidates.

use std::path::Path;

use rspin_core::Result;

use super::DiscoveredSpectrumSource;
use crate::bundle::{LoadedSourceDataKind, SpectrumBundle, SpectrumBundleLoader};

macro_rules! single_discovered_bundle_aliases {
    (
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
            #[doc = concat!("Loads discovered candidates matching one ", $filter_doc, ".")]
            ///
            /// This is a short alias for the matching
            /// `read_discovered_by_source_*` method.
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

            #[doc = concat!("Loads discovered candidates matching one ", $filter_doc, ".")]
            ///
            /// This is a short alias for the matching
            /// `read_discovered_by_source_*` method.
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

        #[doc = concat!("Loads discovered candidates matching one ", $filter_doc, ".")]
        ///
        /// This is a short alias for the matching
        /// `load_discovered_spectra_by_source_*` function.
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

        #[doc = concat!("Loads discovered candidates matching one ", $filter_doc, ".")]
        ///
        /// This is a short alias for the matching
        /// `load_discovered_spectra_by_source_*` function.
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

macro_rules! set_discovered_bundle_aliases {
    (
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
            #[doc = concat!("Loads discovered candidates matching any ", $filter_doc, ".")]
            ///
            /// Values are combined with logical OR. Passing an empty iterator
            /// loads all provided discovered sources.
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

            #[doc = concat!("Loads discovered candidates matching any ", $filter_doc, ".")]
            ///
            /// Values are combined with logical OR. Passing an empty iterator
            /// loads all provided discovered sources.
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

        #[doc = concat!("Loads discovered candidates matching any ", $filter_doc, ".")]
        ///
        /// Values are combined with logical OR. Passing an empty iterator
        /// loads all provided discovered sources.
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

        #[doc = concat!("Loads discovered candidates matching any ", $filter_doc, ".")]
        ///
        /// Values are combined with logical OR. Passing an empty iterator
        /// loads all provided discovered sources.
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

single_discovered_bundle_aliases! {
    filter = "source format";
    value = format: impl AsRef<str>;
    reader_relative = read_discovered_by_format_relative_to;
    reader = read_discovered_by_format;
    free_relative = load_discovered_spectra_by_format_relative_to;
    free = load_discovered_spectra_by_format;
    target_relative = read_discovered_by_source_format_relative_to;
    target = read_discovered_by_source_format;
}

set_discovered_bundle_aliases! {
    filter = "source format";
    values = formats;
    generics = [I, F];
    where = {
        I: IntoIterator<Item = F>,
        F: AsRef<str>,
    };
    reader_relative = read_discovered_by_formats_relative_to;
    reader = read_discovered_by_formats;
    free_relative = load_discovered_spectra_by_formats_relative_to;
    free = load_discovered_spectra_by_formats;
    target_relative = read_discovered_by_source_formats_relative_to;
    target = read_discovered_by_source_formats;
}

single_discovered_bundle_aliases! {
    filter = "vendor family";
    value = vendor: impl AsRef<str>;
    reader_relative = read_discovered_by_vendor_relative_to;
    reader = read_discovered_by_vendor;
    free_relative = load_discovered_spectra_by_vendor_relative_to;
    free = load_discovered_spectra_by_vendor;
    target_relative = read_discovered_by_source_vendor_relative_to;
    target = read_discovered_by_source_vendor;
}

set_discovered_bundle_aliases! {
    filter = "vendor family";
    values = vendors;
    generics = [I, V];
    where = {
        I: IntoIterator<Item = V>,
        V: AsRef<str>,
    };
    reader_relative = read_discovered_by_vendors_relative_to;
    reader = read_discovered_by_vendors;
    free_relative = load_discovered_spectra_by_vendors_relative_to;
    free = load_discovered_spectra_by_vendors;
    target_relative = read_discovered_by_source_vendors_relative_to;
    target = read_discovered_by_source_vendors;
}

single_discovered_bundle_aliases! {
    filter = "raw/processed source data kind";
    value = data_kind: LoadedSourceDataKind;
    reader_relative = read_discovered_by_data_kind_relative_to;
    reader = read_discovered_by_data_kind;
    free_relative = load_discovered_spectra_by_data_kind_relative_to;
    free = load_discovered_spectra_by_data_kind;
    target_relative = read_discovered_by_source_data_kind_relative_to;
    target = read_discovered_by_source_data_kind;
}

set_discovered_bundle_aliases! {
    filter = "raw/processed source data kind";
    values = data_kinds;
    generics = [I];
    where = {
        I: IntoIterator<Item = LoadedSourceDataKind>,
    };
    reader_relative = read_discovered_by_data_kinds_relative_to;
    reader = read_discovered_by_data_kinds;
    free_relative = load_discovered_spectra_by_data_kinds_relative_to;
    free = load_discovered_spectra_by_data_kinds;
    target_relative = read_discovered_by_source_data_kinds_relative_to;
    target = read_discovered_by_source_data_kinds;
}

single_discovered_bundle_aliases! {
    filter = "tracked source path prefix";
    value = source_path_prefix: impl AsRef<Path>;
    reader_relative = read_discovered_by_path_prefix_relative_to;
    reader = read_discovered_by_path_prefix;
    free_relative = load_discovered_spectra_by_path_prefix_relative_to;
    free = load_discovered_spectra_by_path_prefix;
    target_relative = read_discovered_by_source_path_prefix_relative_to;
    target = read_discovered_by_source_path_prefix;
}

set_discovered_bundle_aliases! {
    filter = "tracked source path prefix";
    values = source_path_prefixes;
    generics = [I, P];
    where = {
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    };
    reader_relative = read_discovered_by_path_prefixes_relative_to;
    reader = read_discovered_by_path_prefixes;
    free_relative = load_discovered_spectra_by_path_prefixes_relative_to;
    free = load_discovered_spectra_by_path_prefixes;
    target_relative = read_discovered_by_source_path_prefixes_relative_to;
    target = read_discovered_by_source_path_prefixes;
}
