//! Strict source-filtered exact single-spectrum readers.

use std::path::Path;

use rspin_core::{Result, Spectrum1D, Spectrum2D};

use crate::bundle::{LoadedSource, LoadedSourceDataKind, LoadedSourceFilter, SpectrumBundleLoader};

macro_rules! strict_exact_source_methods {
    (
        filter = $filter_doc:literal;
        value = $value:ident : $value_ty:ty;
        make_filter = $make_filter:expr;
        what = $what:literal;
        output = $output:ty;
        reader = $reader:ident;
        reader_relative = $reader_relative:ident;
        reader_many = $reader_many:ident;
        reader_many_relative = $reader_many_relative:ident;
        free = $free:ident;
        free_relative = $free_relative:ident;
        free_many = $free_many:ident;
        free_many_relative = $free_many_relative:ident;
        into_only = $into_only:ident;
    ) => {
        impl SpectrumBundleLoader {
            #[doc = concat!("Strictly loads a file or directory and returns exactly one ", $what, " matching one ", $filter_doc, ".")]
            ///
            /// Source filtering is applied before candidate files are read, so
            /// unrelated malformed candidates do not fail the strict load.
            ///
            /// # Errors
            ///
            /// Returns an error when strict loading fails or the filtered
            /// bundle does not contain exactly one requested spectrum.
            pub fn $reader(
                &self,
                path: impl AsRef<Path>,
                $value: $value_ty,
            ) -> Result<$output> {
                let filter: LoadedSourceFilter = $make_filter;
                self.read_strict_by_source(path, filter.clone())?
                    .$into_only(filter)
            }

            #[doc = concat!("Strictly loads one selected path relative to a base directory and returns exactly one ", $what, " matching one ", $filter_doc, ".")]
            ///
            /// Source filtering is applied before candidate files are read, so
            /// unrelated malformed candidates do not fail the strict load.
            ///
            /// # Errors
            ///
            /// Returns an error when strict loading fails or the filtered
            /// bundle does not contain exactly one requested spectrum.
            pub fn $reader_relative(
                &self,
                base: impl AsRef<Path>,
                path: impl AsRef<Path>,
                $value: $value_ty,
            ) -> Result<$output> {
                let filter: LoadedSourceFilter = $make_filter;
                self.read_strict_by_source_relative_to(base, path, filter.clone())?
                    .$into_only(filter)
            }

            #[doc = concat!("Strictly loads selected paths and returns exactly one ", $what, " matching one ", $filter_doc, ".")]
            ///
            /// Source filtering is applied before candidate files are read, so
            /// unrelated malformed candidates do not fail the strict load.
            ///
            /// # Errors
            ///
            /// Returns an error when strict loading fails or the filtered
            /// bundle does not contain exactly one requested spectrum.
            pub fn $reader_many<I, P>(&self, paths: I, $value: $value_ty) -> Result<$output>
            where
                I: IntoIterator<Item = P>,
                P: AsRef<Path>,
            {
                let filter: LoadedSourceFilter = $make_filter;
                self.read_many_strict_by_source(paths, filter.clone())?
                    .$into_only(filter)
            }

            #[doc = concat!("Strictly loads selected paths relative to a base directory and returns exactly one ", $what, " matching one ", $filter_doc, ".")]
            ///
            /// Source filtering is applied before candidate files are read, so
            /// unrelated malformed candidates do not fail the strict load.
            ///
            /// # Errors
            ///
            /// Returns an error when strict loading fails or the filtered
            /// bundle does not contain exactly one requested spectrum.
            pub fn $reader_many_relative<I, P>(
                &self,
                base: impl AsRef<Path>,
                paths: I,
                $value: $value_ty,
            ) -> Result<$output>
            where
                I: IntoIterator<Item = P>,
                P: AsRef<Path>,
            {
                let filter: LoadedSourceFilter = $make_filter;
                self.read_many_strict_by_source_relative_to(base, paths, filter.clone())?
                    .$into_only(filter)
            }
        }

        #[doc = concat!("Strictly loads a file or directory and returns exactly one ", $what, " matching one ", $filter_doc, ".")]
        ///
        /// Source filtering is applied before candidate files are read, so
        /// unrelated malformed candidates do not fail the strict load.
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading fails or the filtered bundle
        /// does not contain exactly one requested spectrum.
        pub fn $free(path: impl AsRef<Path>, $value: $value_ty) -> Result<$output> {
            SpectrumBundleLoader::new().$reader(path, $value)
        }

        #[doc = concat!("Strictly loads one selected path relative to a base directory and returns exactly one ", $what, " matching one ", $filter_doc, ".")]
        ///
        /// Source filtering is applied before candidate files are read, so
        /// unrelated malformed candidates do not fail the strict load.
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading fails or the filtered bundle
        /// does not contain exactly one requested spectrum.
        pub fn $free_relative(
            base: impl AsRef<Path>,
            path: impl AsRef<Path>,
            $value: $value_ty,
        ) -> Result<$output> {
            SpectrumBundleLoader::new().$reader_relative(base, path, $value)
        }

        #[doc = concat!("Strictly loads selected paths and returns exactly one ", $what, " matching one ", $filter_doc, ".")]
        ///
        /// Source filtering is applied before candidate files are read, so
        /// unrelated malformed candidates do not fail the strict load.
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading fails or the filtered bundle
        /// does not contain exactly one requested spectrum.
        pub fn $free_many<I, P>(paths: I, $value: $value_ty) -> Result<$output>
        where
            I: IntoIterator<Item = P>,
            P: AsRef<Path>,
        {
            SpectrumBundleLoader::new().$reader_many(paths, $value)
        }

        #[doc = concat!("Strictly loads selected paths relative to a base directory and returns exactly one ", $what, " matching one ", $filter_doc, ".")]
        ///
        /// Source filtering is applied before candidate files are read, so
        /// unrelated malformed candidates do not fail the strict load.
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading fails or the filtered bundle
        /// does not contain exactly one requested spectrum.
        pub fn $free_many_relative<I, P>(
            base: impl AsRef<Path>,
            paths: I,
            $value: $value_ty,
        ) -> Result<$output>
        where
            I: IntoIterator<Item = P>,
            P: AsRef<Path>,
        {
            SpectrumBundleLoader::new().$reader_many_relative(base, paths, $value)
        }
    };
}

macro_rules! strict_exact_output {
    (
        what = $what:literal;
        output = $output:ty;
        into_only = $into_only:ident;
        source = [$source_reader:ident, $source_reader_relative:ident, $source_reader_many:ident, $source_reader_many_relative:ident, $source_free:ident, $source_free_relative:ident, $source_free_many:ident, $source_free_many_relative:ident];
        format = [$format_reader:ident, $format_reader_relative:ident, $format_reader_many:ident, $format_reader_many_relative:ident, $format_free:ident, $format_free_relative:ident, $format_free_many:ident, $format_free_many_relative:ident];
        vendor = [$vendor_reader:ident, $vendor_reader_relative:ident, $vendor_reader_many:ident, $vendor_reader_many_relative:ident, $vendor_free:ident, $vendor_free_relative:ident, $vendor_free_many:ident, $vendor_free_many_relative:ident];
        data_kind = [$data_kind_reader:ident, $data_kind_reader_relative:ident, $data_kind_reader_many:ident, $data_kind_reader_many_relative:ident, $data_kind_free:ident, $data_kind_free_relative:ident, $data_kind_free_many:ident, $data_kind_free_many_relative:ident];
        path = [$path_reader:ident, $path_reader_relative:ident, $path_reader_many:ident, $path_reader_many_relative:ident, $path_free:ident, $path_free_relative:ident, $path_free_many:ident, $path_free_many_relative:ident];
        path_prefix = [$prefix_reader:ident, $prefix_reader_relative:ident, $prefix_reader_many:ident, $prefix_reader_many_relative:ident, $prefix_free:ident, $prefix_free_relative:ident, $prefix_free_many:ident, $prefix_free_many_relative:ident];
    ) => {
        strict_exact_source_methods! {
            filter = "generic source filter";
            value = filter: impl Into<LoadedSourceFilter>;
            make_filter = filter.into();
            what = $what;
            output = $output;
            reader = $source_reader;
            reader_relative = $source_reader_relative;
            reader_many = $source_reader_many;
            reader_many_relative = $source_reader_many_relative;
            free = $source_free;
            free_relative = $source_free_relative;
            free_many = $source_free_many;
            free_many_relative = $source_free_many_relative;
            into_only = $into_only;
        }

        strict_exact_source_methods! {
            filter = "source format";
            value = format: impl AsRef<str>;
            make_filter = LoadedSourceFilter::format(format);
            what = $what;
            output = $output;
            reader = $format_reader;
            reader_relative = $format_reader_relative;
            reader_many = $format_reader_many;
            reader_many_relative = $format_reader_many_relative;
            free = $format_free;
            free_relative = $format_free_relative;
            free_many = $format_free_many;
            free_many_relative = $format_free_many_relative;
            into_only = $into_only;
        }

        strict_exact_source_methods! {
            filter = "vendor family";
            value = vendor: impl AsRef<str>;
            make_filter = LoadedSourceFilter::vendor(vendor);
            what = $what;
            output = $output;
            reader = $vendor_reader;
            reader_relative = $vendor_reader_relative;
            reader_many = $vendor_reader_many;
            reader_many_relative = $vendor_reader_many_relative;
            free = $vendor_free;
            free_relative = $vendor_free_relative;
            free_many = $vendor_free_many;
            free_many_relative = $vendor_free_many_relative;
            into_only = $into_only;
        }

        strict_exact_source_methods! {
            filter = "raw/processed source data kind";
            value = data_kind: LoadedSourceDataKind;
            make_filter = LoadedSourceFilter::data_kind(data_kind);
            what = $what;
            output = $output;
            reader = $data_kind_reader;
            reader_relative = $data_kind_reader_relative;
            reader_many = $data_kind_reader_many;
            reader_many_relative = $data_kind_reader_many_relative;
            free = $data_kind_free;
            free_relative = $data_kind_free_relative;
            free_many = $data_kind_free_many;
            free_many_relative = $data_kind_free_many_relative;
            into_only = $into_only;
        }

        strict_exact_source_methods! {
            filter = "tracked source path";
            value = source_path: impl AsRef<Path>;
            make_filter = LoadedSourceFilter::path(source_path);
            what = $what;
            output = $output;
            reader = $path_reader;
            reader_relative = $path_reader_relative;
            reader_many = $path_reader_many;
            reader_many_relative = $path_reader_many_relative;
            free = $path_free;
            free_relative = $path_free_relative;
            free_many = $path_free_many;
            free_many_relative = $path_free_many_relative;
            into_only = $into_only;
        }

        strict_exact_source_methods! {
            filter = "tracked source path prefix";
            value = source_path_prefix: impl AsRef<Path>;
            make_filter = LoadedSourceFilter::path_prefix(source_path_prefix);
            what = $what;
            output = $output;
            reader = $prefix_reader;
            reader_relative = $prefix_reader_relative;
            reader_many = $prefix_reader_many;
            reader_many_relative = $prefix_reader_many_relative;
            free = $prefix_free;
            free_relative = $prefix_free_relative;
            free_many = $prefix_free_many;
            free_many_relative = $prefix_free_many_relative;
            into_only = $into_only;
        }
    };
}

strict_exact_output! {
    what = "one-dimensional spectrum";
    output = Spectrum1D;
    into_only = into_only_1d_by_source;
    source = [read_1d_strict_by_source, read_1d_strict_by_source_relative_to, read_1d_many_strict_by_source, read_1d_many_strict_by_source_relative_to, load_spectrum_1d_strict_by_source, load_spectrum_1d_strict_by_source_relative_to, load_spectrum_1d_many_strict_by_source, load_spectrum_1d_many_strict_by_source_relative_to];
    format = [read_1d_strict_by_source_format, read_1d_strict_by_source_format_relative_to, read_1d_many_strict_by_source_format, read_1d_many_strict_by_source_format_relative_to, load_spectrum_1d_strict_by_source_format, load_spectrum_1d_strict_by_source_format_relative_to, load_spectrum_1d_many_strict_by_source_format, load_spectrum_1d_many_strict_by_source_format_relative_to];
    vendor = [read_1d_strict_by_source_vendor, read_1d_strict_by_source_vendor_relative_to, read_1d_many_strict_by_source_vendor, read_1d_many_strict_by_source_vendor_relative_to, load_spectrum_1d_strict_by_source_vendor, load_spectrum_1d_strict_by_source_vendor_relative_to, load_spectrum_1d_many_strict_by_source_vendor, load_spectrum_1d_many_strict_by_source_vendor_relative_to];
    data_kind = [read_1d_strict_by_source_data_kind, read_1d_strict_by_source_data_kind_relative_to, read_1d_many_strict_by_source_data_kind, read_1d_many_strict_by_source_data_kind_relative_to, load_spectrum_1d_strict_by_source_data_kind, load_spectrum_1d_strict_by_source_data_kind_relative_to, load_spectrum_1d_many_strict_by_source_data_kind, load_spectrum_1d_many_strict_by_source_data_kind_relative_to];
    path = [read_1d_strict_by_source_path, read_1d_strict_by_source_path_relative_to, read_1d_many_strict_by_source_path, read_1d_many_strict_by_source_path_relative_to, load_spectrum_1d_strict_by_source_path, load_spectrum_1d_strict_by_source_path_relative_to, load_spectrum_1d_many_strict_by_source_path, load_spectrum_1d_many_strict_by_source_path_relative_to];
    path_prefix = [read_1d_strict_by_source_path_prefix, read_1d_strict_by_source_path_prefix_relative_to, read_1d_many_strict_by_source_path_prefix, read_1d_many_strict_by_source_path_prefix_relative_to, load_spectrum_1d_strict_by_source_path_prefix, load_spectrum_1d_strict_by_source_path_prefix_relative_to, load_spectrum_1d_many_strict_by_source_path_prefix, load_spectrum_1d_many_strict_by_source_path_prefix_relative_to];
}

strict_exact_output! {
    what = "one-dimensional spectrum with source metadata";
    output = (Spectrum1D, LoadedSource);
    into_only = into_only_loaded_1d_by_source;
    source = [read_1d_with_source_strict_by_source, read_1d_with_source_strict_by_source_relative_to, read_1d_many_with_source_strict_by_source, read_1d_many_with_source_strict_by_source_relative_to, load_spectrum_1d_with_source_strict_by_source, load_spectrum_1d_with_source_strict_by_source_relative_to, load_spectrum_1d_many_with_source_strict_by_source, load_spectrum_1d_many_with_source_strict_by_source_relative_to];
    format = [read_1d_with_source_strict_by_source_format, read_1d_with_source_strict_by_source_format_relative_to, read_1d_many_with_source_strict_by_source_format, read_1d_many_with_source_strict_by_source_format_relative_to, load_spectrum_1d_with_source_strict_by_source_format, load_spectrum_1d_with_source_strict_by_source_format_relative_to, load_spectrum_1d_many_with_source_strict_by_source_format, load_spectrum_1d_many_with_source_strict_by_source_format_relative_to];
    vendor = [read_1d_with_source_strict_by_source_vendor, read_1d_with_source_strict_by_source_vendor_relative_to, read_1d_many_with_source_strict_by_source_vendor, read_1d_many_with_source_strict_by_source_vendor_relative_to, load_spectrum_1d_with_source_strict_by_source_vendor, load_spectrum_1d_with_source_strict_by_source_vendor_relative_to, load_spectrum_1d_many_with_source_strict_by_source_vendor, load_spectrum_1d_many_with_source_strict_by_source_vendor_relative_to];
    data_kind = [read_1d_with_source_strict_by_source_data_kind, read_1d_with_source_strict_by_source_data_kind_relative_to, read_1d_many_with_source_strict_by_source_data_kind, read_1d_many_with_source_strict_by_source_data_kind_relative_to, load_spectrum_1d_with_source_strict_by_source_data_kind, load_spectrum_1d_with_source_strict_by_source_data_kind_relative_to, load_spectrum_1d_many_with_source_strict_by_source_data_kind, load_spectrum_1d_many_with_source_strict_by_source_data_kind_relative_to];
    path = [read_1d_with_source_strict_by_source_path, read_1d_with_source_strict_by_source_path_relative_to, read_1d_many_with_source_strict_by_source_path, read_1d_many_with_source_strict_by_source_path_relative_to, load_spectrum_1d_with_source_strict_by_source_path, load_spectrum_1d_with_source_strict_by_source_path_relative_to, load_spectrum_1d_many_with_source_strict_by_source_path, load_spectrum_1d_many_with_source_strict_by_source_path_relative_to];
    path_prefix = [read_1d_with_source_strict_by_source_path_prefix, read_1d_with_source_strict_by_source_path_prefix_relative_to, read_1d_many_with_source_strict_by_source_path_prefix, read_1d_many_with_source_strict_by_source_path_prefix_relative_to, load_spectrum_1d_with_source_strict_by_source_path_prefix, load_spectrum_1d_with_source_strict_by_source_path_prefix_relative_to, load_spectrum_1d_many_with_source_strict_by_source_path_prefix, load_spectrum_1d_many_with_source_strict_by_source_path_prefix_relative_to];
}

strict_exact_output! {
    what = "two-dimensional spectrum";
    output = Spectrum2D;
    into_only = into_only_2d_by_source;
    source = [read_2d_strict_by_source, read_2d_strict_by_source_relative_to, read_2d_many_strict_by_source, read_2d_many_strict_by_source_relative_to, load_spectrum_2d_strict_by_source, load_spectrum_2d_strict_by_source_relative_to, load_spectrum_2d_many_strict_by_source, load_spectrum_2d_many_strict_by_source_relative_to];
    format = [read_2d_strict_by_source_format, read_2d_strict_by_source_format_relative_to, read_2d_many_strict_by_source_format, read_2d_many_strict_by_source_format_relative_to, load_spectrum_2d_strict_by_source_format, load_spectrum_2d_strict_by_source_format_relative_to, load_spectrum_2d_many_strict_by_source_format, load_spectrum_2d_many_strict_by_source_format_relative_to];
    vendor = [read_2d_strict_by_source_vendor, read_2d_strict_by_source_vendor_relative_to, read_2d_many_strict_by_source_vendor, read_2d_many_strict_by_source_vendor_relative_to, load_spectrum_2d_strict_by_source_vendor, load_spectrum_2d_strict_by_source_vendor_relative_to, load_spectrum_2d_many_strict_by_source_vendor, load_spectrum_2d_many_strict_by_source_vendor_relative_to];
    data_kind = [read_2d_strict_by_source_data_kind, read_2d_strict_by_source_data_kind_relative_to, read_2d_many_strict_by_source_data_kind, read_2d_many_strict_by_source_data_kind_relative_to, load_spectrum_2d_strict_by_source_data_kind, load_spectrum_2d_strict_by_source_data_kind_relative_to, load_spectrum_2d_many_strict_by_source_data_kind, load_spectrum_2d_many_strict_by_source_data_kind_relative_to];
    path = [read_2d_strict_by_source_path, read_2d_strict_by_source_path_relative_to, read_2d_many_strict_by_source_path, read_2d_many_strict_by_source_path_relative_to, load_spectrum_2d_strict_by_source_path, load_spectrum_2d_strict_by_source_path_relative_to, load_spectrum_2d_many_strict_by_source_path, load_spectrum_2d_many_strict_by_source_path_relative_to];
    path_prefix = [read_2d_strict_by_source_path_prefix, read_2d_strict_by_source_path_prefix_relative_to, read_2d_many_strict_by_source_path_prefix, read_2d_many_strict_by_source_path_prefix_relative_to, load_spectrum_2d_strict_by_source_path_prefix, load_spectrum_2d_strict_by_source_path_prefix_relative_to, load_spectrum_2d_many_strict_by_source_path_prefix, load_spectrum_2d_many_strict_by_source_path_prefix_relative_to];
}

strict_exact_output! {
    what = "two-dimensional spectrum with source metadata";
    output = (Spectrum2D, LoadedSource);
    into_only = into_only_loaded_2d_by_source;
    source = [read_2d_with_source_strict_by_source, read_2d_with_source_strict_by_source_relative_to, read_2d_many_with_source_strict_by_source, read_2d_many_with_source_strict_by_source_relative_to, load_spectrum_2d_with_source_strict_by_source, load_spectrum_2d_with_source_strict_by_source_relative_to, load_spectrum_2d_many_with_source_strict_by_source, load_spectrum_2d_many_with_source_strict_by_source_relative_to];
    format = [read_2d_with_source_strict_by_source_format, read_2d_with_source_strict_by_source_format_relative_to, read_2d_many_with_source_strict_by_source_format, read_2d_many_with_source_strict_by_source_format_relative_to, load_spectrum_2d_with_source_strict_by_source_format, load_spectrum_2d_with_source_strict_by_source_format_relative_to, load_spectrum_2d_many_with_source_strict_by_source_format, load_spectrum_2d_many_with_source_strict_by_source_format_relative_to];
    vendor = [read_2d_with_source_strict_by_source_vendor, read_2d_with_source_strict_by_source_vendor_relative_to, read_2d_many_with_source_strict_by_source_vendor, read_2d_many_with_source_strict_by_source_vendor_relative_to, load_spectrum_2d_with_source_strict_by_source_vendor, load_spectrum_2d_with_source_strict_by_source_vendor_relative_to, load_spectrum_2d_many_with_source_strict_by_source_vendor, load_spectrum_2d_many_with_source_strict_by_source_vendor_relative_to];
    data_kind = [read_2d_with_source_strict_by_source_data_kind, read_2d_with_source_strict_by_source_data_kind_relative_to, read_2d_many_with_source_strict_by_source_data_kind, read_2d_many_with_source_strict_by_source_data_kind_relative_to, load_spectrum_2d_with_source_strict_by_source_data_kind, load_spectrum_2d_with_source_strict_by_source_data_kind_relative_to, load_spectrum_2d_many_with_source_strict_by_source_data_kind, load_spectrum_2d_many_with_source_strict_by_source_data_kind_relative_to];
    path = [read_2d_with_source_strict_by_source_path, read_2d_with_source_strict_by_source_path_relative_to, read_2d_many_with_source_strict_by_source_path, read_2d_many_with_source_strict_by_source_path_relative_to, load_spectrum_2d_with_source_strict_by_source_path, load_spectrum_2d_with_source_strict_by_source_path_relative_to, load_spectrum_2d_many_with_source_strict_by_source_path, load_spectrum_2d_many_with_source_strict_by_source_path_relative_to];
    path_prefix = [read_2d_with_source_strict_by_source_path_prefix, read_2d_with_source_strict_by_source_path_prefix_relative_to, read_2d_many_with_source_strict_by_source_path_prefix, read_2d_many_with_source_strict_by_source_path_prefix_relative_to, load_spectrum_2d_with_source_strict_by_source_path_prefix, load_spectrum_2d_with_source_strict_by_source_path_prefix_relative_to, load_spectrum_2d_many_with_source_strict_by_source_path_prefix, load_spectrum_2d_many_with_source_strict_by_source_path_prefix_relative_to];
}
