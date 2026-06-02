//! Short source-filtered exact loaders for discovered source candidates.

use std::path::Path;

use rspin_core::{Result, Spectrum1D, Spectrum2D};

use super::DiscoveredSpectrumSource;
use crate::bundle::{LoadedSource, LoadedSourceDataKind, LoadedSourceFilter, SpectrumBundleLoader};

macro_rules! short_discovered_exact_methods {
    (
        filter = $filter_doc:literal;
        value = $value:ident : $value_ty:ty;
        build = $build:expr;
        what = $what:literal;
        output = $output:ty;
        reader_relative = $reader_relative:ident;
        reader = $reader:ident;
        free_relative = $free_relative:ident;
        free = $free:ident;
        target_relative = $target_relative:ident;
        target = $target:ident;
    ) => {
        impl SpectrumBundleLoader {
            #[doc = concat!("Loads discovered candidates matching one ", $filter_doc, " as exactly one ", $what, ".")]
            ///
            /// This is a short alias for the matching
            /// `read_discovered_*_by_source_*` exact loader method.
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or exactly one matching
            /// spectrum is not found.
            pub fn $reader_relative<'a>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                $value: $value_ty,
            ) -> Result<$output> {
                self.$target_relative(base, sources, $build($value))
            }

            #[doc = concat!("Loads discovered candidates matching one ", $filter_doc, " as exactly one ", $what, ".")]
            ///
            /// This is a short alias for the matching
            /// `read_discovered_*_by_source_*` exact loader method.
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or exactly one matching
            /// spectrum is not found.
            pub fn $reader<'a>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                $value: $value_ty,
            ) -> Result<$output> {
                self.$target(base, sources, $build($value))
            }
        }

        #[doc = concat!("Loads discovered candidates matching one ", $filter_doc, " as exactly one ", $what, ".")]
        ///
        /// This is a short alias for the matching
        /// `load_discovered_spectrum_*_by_source_*` exact loader function.
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or exactly one matching spectrum is not found.
        pub fn $free_relative<'a>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            $value: $value_ty,
        ) -> Result<$output> {
            SpectrumBundleLoader::new().$reader_relative(base, sources, $value)
        }

        #[doc = concat!("Loads discovered candidates matching one ", $filter_doc, " as exactly one ", $what, ".")]
        ///
        /// This is a short alias for the matching
        /// `load_discovered_spectrum_*_by_source_*` exact loader function.
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or exactly one matching spectrum is not found.
        pub fn $free<'a>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            $value: $value_ty,
        ) -> Result<$output> {
            SpectrumBundleLoader::new().$reader(base, sources, $value)
        }
    };
}

macro_rules! short_discovered_exact_output {
    (
        what = $what:literal;
        output = $output:ty;
        format = [$format_reader_relative:ident, $format_reader:ident, $format_free_relative:ident, $format_free:ident, $format_target_relative:ident, $format_target:ident];
        vendor = [$vendor_reader_relative:ident, $vendor_reader:ident, $vendor_free_relative:ident, $vendor_free:ident, $vendor_target_relative:ident, $vendor_target:ident];
        data_kind = [$data_kind_reader_relative:ident, $data_kind_reader:ident, $data_kind_free_relative:ident, $data_kind_free:ident, $data_kind_target_relative:ident, $data_kind_target:ident];
        path_prefix = [$prefix_reader_relative:ident, $prefix_reader:ident, $prefix_free_relative:ident, $prefix_free:ident, $prefix_target_relative:ident, $prefix_target:ident];
    ) => {
        short_discovered_exact_methods! {
            filter = "source format";
            value = format: impl AsRef<str>;
            build = LoadedSourceFilter::format;
            what = $what;
            output = $output;
            reader_relative = $format_reader_relative;
            reader = $format_reader;
            free_relative = $format_free_relative;
            free = $format_free;
            target_relative = $format_target_relative;
            target = $format_target;
        }

        short_discovered_exact_methods! {
            filter = "vendor family";
            value = vendor: impl AsRef<str>;
            build = LoadedSourceFilter::vendor;
            what = $what;
            output = $output;
            reader_relative = $vendor_reader_relative;
            reader = $vendor_reader;
            free_relative = $vendor_free_relative;
            free = $vendor_free;
            target_relative = $vendor_target_relative;
            target = $vendor_target;
        }

        short_discovered_exact_methods! {
            filter = "raw/processed source data kind";
            value = data_kind: LoadedSourceDataKind;
            build = LoadedSourceFilter::data_kind;
            what = $what;
            output = $output;
            reader_relative = $data_kind_reader_relative;
            reader = $data_kind_reader;
            free_relative = $data_kind_free_relative;
            free = $data_kind_free;
            target_relative = $data_kind_target_relative;
            target = $data_kind_target;
        }

        short_discovered_exact_methods! {
            filter = "tracked source path prefix";
            value = source_path_prefix: impl AsRef<Path>;
            build = LoadedSourceFilter::path_prefix;
            what = $what;
            output = $output;
            reader_relative = $prefix_reader_relative;
            reader = $prefix_reader;
            free_relative = $prefix_free_relative;
            free = $prefix_free;
            target_relative = $prefix_target_relative;
            target = $prefix_target;
        }
    };
}

short_discovered_exact_output! {
    what = "one-dimensional spectrum";
    output = Spectrum1D;
    format = [read_discovered_1d_by_format_relative_to, read_discovered_1d_by_format, load_discovered_spectrum_1d_by_format_relative_to, load_discovered_spectrum_1d_by_format, read_discovered_1d_by_source_relative_to, read_discovered_1d_by_source];
    vendor = [read_discovered_1d_by_vendor_relative_to, read_discovered_1d_by_vendor, load_discovered_spectrum_1d_by_vendor_relative_to, load_discovered_spectrum_1d_by_vendor, read_discovered_1d_by_source_relative_to, read_discovered_1d_by_source];
    data_kind = [read_discovered_1d_by_data_kind_relative_to, read_discovered_1d_by_data_kind, load_discovered_spectrum_1d_by_data_kind_relative_to, load_discovered_spectrum_1d_by_data_kind, read_discovered_1d_by_source_relative_to, read_discovered_1d_by_source];
    path_prefix = [read_discovered_1d_by_path_prefix_relative_to, read_discovered_1d_by_path_prefix, load_discovered_spectrum_1d_by_path_prefix_relative_to, load_discovered_spectrum_1d_by_path_prefix, read_discovered_1d_by_source_relative_to, read_discovered_1d_by_source];
}

short_discovered_exact_output! {
    what = "one-dimensional spectrum with source metadata";
    output = (Spectrum1D, LoadedSource);
    format = [read_discovered_1d_with_source_by_format_relative_to, read_discovered_1d_with_source_by_format, load_discovered_spectrum_1d_with_source_by_format_relative_to, load_discovered_spectrum_1d_with_source_by_format, read_discovered_1d_with_source_by_source_relative_to, read_discovered_1d_with_source_by_source];
    vendor = [read_discovered_1d_with_source_by_vendor_relative_to, read_discovered_1d_with_source_by_vendor, load_discovered_spectrum_1d_with_source_by_vendor_relative_to, load_discovered_spectrum_1d_with_source_by_vendor, read_discovered_1d_with_source_by_source_relative_to, read_discovered_1d_with_source_by_source];
    data_kind = [read_discovered_1d_with_source_by_data_kind_relative_to, read_discovered_1d_with_source_by_data_kind, load_discovered_spectrum_1d_with_source_by_data_kind_relative_to, load_discovered_spectrum_1d_with_source_by_data_kind, read_discovered_1d_with_source_by_source_relative_to, read_discovered_1d_with_source_by_source];
    path_prefix = [read_discovered_1d_with_source_by_path_prefix_relative_to, read_discovered_1d_with_source_by_path_prefix, load_discovered_spectrum_1d_with_source_by_path_prefix_relative_to, load_discovered_spectrum_1d_with_source_by_path_prefix, read_discovered_1d_with_source_by_source_relative_to, read_discovered_1d_with_source_by_source];
}

short_discovered_exact_output! {
    what = "two-dimensional spectrum";
    output = Spectrum2D;
    format = [read_discovered_2d_by_format_relative_to, read_discovered_2d_by_format, load_discovered_spectrum_2d_by_format_relative_to, load_discovered_spectrum_2d_by_format, read_discovered_2d_by_source_relative_to, read_discovered_2d_by_source];
    vendor = [read_discovered_2d_by_vendor_relative_to, read_discovered_2d_by_vendor, load_discovered_spectrum_2d_by_vendor_relative_to, load_discovered_spectrum_2d_by_vendor, read_discovered_2d_by_source_relative_to, read_discovered_2d_by_source];
    data_kind = [read_discovered_2d_by_data_kind_relative_to, read_discovered_2d_by_data_kind, load_discovered_spectrum_2d_by_data_kind_relative_to, load_discovered_spectrum_2d_by_data_kind, read_discovered_2d_by_source_relative_to, read_discovered_2d_by_source];
    path_prefix = [read_discovered_2d_by_path_prefix_relative_to, read_discovered_2d_by_path_prefix, load_discovered_spectrum_2d_by_path_prefix_relative_to, load_discovered_spectrum_2d_by_path_prefix, read_discovered_2d_by_source_relative_to, read_discovered_2d_by_source];
}

short_discovered_exact_output! {
    what = "two-dimensional spectrum with source metadata";
    output = (Spectrum2D, LoadedSource);
    format = [read_discovered_2d_with_source_by_format_relative_to, read_discovered_2d_with_source_by_format, load_discovered_spectrum_2d_with_source_by_format_relative_to, load_discovered_spectrum_2d_with_source_by_format, read_discovered_2d_with_source_by_source_relative_to, read_discovered_2d_with_source_by_source];
    vendor = [read_discovered_2d_with_source_by_vendor_relative_to, read_discovered_2d_with_source_by_vendor, load_discovered_spectrum_2d_with_source_by_vendor_relative_to, load_discovered_spectrum_2d_with_source_by_vendor, read_discovered_2d_with_source_by_source_relative_to, read_discovered_2d_with_source_by_source];
    data_kind = [read_discovered_2d_with_source_by_data_kind_relative_to, read_discovered_2d_with_source_by_data_kind, load_discovered_spectrum_2d_with_source_by_data_kind_relative_to, load_discovered_spectrum_2d_with_source_by_data_kind, read_discovered_2d_with_source_by_source_relative_to, read_discovered_2d_with_source_by_source];
    path_prefix = [read_discovered_2d_with_source_by_path_prefix_relative_to, read_discovered_2d_with_source_by_path_prefix, load_discovered_spectrum_2d_with_source_by_path_prefix_relative_to, load_discovered_spectrum_2d_with_source_by_path_prefix, read_discovered_2d_with_source_by_source_relative_to, read_discovered_2d_with_source_by_source];
}
