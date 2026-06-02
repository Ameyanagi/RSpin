//! Typed source-filtered strict first-spectrum reader helpers.

use std::path::Path;

use rspin_core::{Result, Spectrum1D, Spectrum2D};

use super::{
    LoadedSource, LoadedSourceDataKind, LoadedSourceFilter, LoadedSpectrum, SpectrumBundleLoader,
};

macro_rules! typed_strict_first_methods {
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
        generic_reader = $generic_reader:ident;
        generic_reader_relative = $generic_reader_relative:ident;
        generic_reader_many = $generic_reader_many:ident;
        generic_reader_many_relative = $generic_reader_many_relative:ident;
    ) => {
        impl SpectrumBundleLoader {
            #[doc = concat!("Strictly loads a file or directory and returns the first ", $what, " matching one ", $filter_doc, ".")]
            ///
            /// Source filtering is applied before candidate files are read, so
            /// unrelated malformed candidates do not fail the strict load.
            ///
            /// # Errors
            ///
            /// Returns an error when strict loading fails or no matching
            /// spectrum is found.
            pub fn $reader(
                &self,
                path: impl AsRef<Path>,
                $value: $value_ty,
            ) -> Result<$output> {
                self.$generic_reader(path, $make_filter)
            }

            #[doc = concat!("Strictly loads one selected path relative to a base directory and returns the first ", $what, " matching one ", $filter_doc, ".")]
            ///
            /// Source filtering is applied before candidate files are read, so
            /// unrelated malformed candidates do not fail the strict load.
            ///
            /// # Errors
            ///
            /// Returns an error when strict loading fails or no matching
            /// spectrum is found.
            pub fn $reader_relative(
                &self,
                base: impl AsRef<Path>,
                path: impl AsRef<Path>,
                $value: $value_ty,
            ) -> Result<$output> {
                self.$generic_reader_relative(base, path, $make_filter)
            }

            #[doc = concat!("Strictly loads selected paths and returns the first ", $what, " matching one ", $filter_doc, ".")]
            ///
            /// Source filtering is applied before candidate files are read, so
            /// unrelated malformed candidates do not fail the strict load.
            ///
            /// # Errors
            ///
            /// Returns an error when strict loading fails or no matching
            /// spectrum is found.
            pub fn $reader_many<I, P>(&self, paths: I, $value: $value_ty) -> Result<$output>
            where
                I: IntoIterator<Item = P>,
                P: AsRef<Path>,
            {
                self.$generic_reader_many(paths, $make_filter)
            }

            #[doc = concat!("Strictly loads selected paths relative to a base directory and returns the first ", $what, " matching one ", $filter_doc, ".")]
            ///
            /// Source filtering is applied before candidate files are read, so
            /// unrelated malformed candidates do not fail the strict load.
            ///
            /// # Errors
            ///
            /// Returns an error when strict loading fails or no matching
            /// spectrum is found.
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
                self.$generic_reader_many_relative(base, paths, $make_filter)
            }
        }

        #[doc = concat!("Strictly loads a file or directory and returns the first ", $what, " matching one ", $filter_doc, ".")]
        ///
        /// Source filtering is applied before candidate files are read, so
        /// unrelated malformed candidates do not fail the strict load.
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading fails or no matching spectrum is found.
        pub fn $free(path: impl AsRef<Path>, $value: $value_ty) -> Result<$output> {
            SpectrumBundleLoader::new().$reader(path, $value)
        }

        #[doc = concat!("Strictly loads one selected path relative to a base directory and returns the first ", $what, " matching one ", $filter_doc, ".")]
        ///
        /// Source filtering is applied before candidate files are read, so
        /// unrelated malformed candidates do not fail the strict load.
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading fails or no matching spectrum is found.
        pub fn $free_relative(
            base: impl AsRef<Path>,
            path: impl AsRef<Path>,
            $value: $value_ty,
        ) -> Result<$output> {
            SpectrumBundleLoader::new().$reader_relative(base, path, $value)
        }

        #[doc = concat!("Strictly loads selected paths and returns the first ", $what, " matching one ", $filter_doc, ".")]
        ///
        /// Source filtering is applied before candidate files are read, so
        /// unrelated malformed candidates do not fail the strict load.
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading fails or no matching spectrum is found.
        pub fn $free_many<I, P>(paths: I, $value: $value_ty) -> Result<$output>
        where
            I: IntoIterator<Item = P>,
            P: AsRef<Path>,
        {
            SpectrumBundleLoader::new().$reader_many(paths, $value)
        }

        #[doc = concat!("Strictly loads selected paths relative to a base directory and returns the first ", $what, " matching one ", $filter_doc, ".")]
        ///
        /// Source filtering is applied before candidate files are read, so
        /// unrelated malformed candidates do not fail the strict load.
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading fails or no matching spectrum is found.
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

macro_rules! typed_strict_first_output {
    (
        what = $what:literal;
        output = $output:ty;
        generic_reader = $generic_reader:ident;
        generic_reader_relative = $generic_reader_relative:ident;
        generic_reader_many = $generic_reader_many:ident;
        generic_reader_many_relative = $generic_reader_many_relative:ident;
        format = [$format_reader:ident, $format_reader_relative:ident, $format_reader_many:ident, $format_reader_many_relative:ident, $format_free:ident, $format_free_relative:ident, $format_free_many:ident, $format_free_many_relative:ident];
        vendor = [$vendor_reader:ident, $vendor_reader_relative:ident, $vendor_reader_many:ident, $vendor_reader_many_relative:ident, $vendor_free:ident, $vendor_free_relative:ident, $vendor_free_many:ident, $vendor_free_many_relative:ident];
        data_kind = [$data_kind_reader:ident, $data_kind_reader_relative:ident, $data_kind_reader_many:ident, $data_kind_reader_many_relative:ident, $data_kind_free:ident, $data_kind_free_relative:ident, $data_kind_free_many:ident, $data_kind_free_many_relative:ident];
        path = [$path_reader:ident, $path_reader_relative:ident, $path_reader_many:ident, $path_reader_many_relative:ident, $path_free:ident, $path_free_relative:ident, $path_free_many:ident, $path_free_many_relative:ident];
        path_prefix = [$prefix_reader:ident, $prefix_reader_relative:ident, $prefix_reader_many:ident, $prefix_reader_many_relative:ident, $prefix_free:ident, $prefix_free_relative:ident, $prefix_free_many:ident, $prefix_free_many_relative:ident];
    ) => {
        typed_strict_first_methods! {
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
            generic_reader = $generic_reader;
            generic_reader_relative = $generic_reader_relative;
            generic_reader_many = $generic_reader_many;
            generic_reader_many_relative = $generic_reader_many_relative;
        }

        typed_strict_first_methods! {
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
            generic_reader = $generic_reader;
            generic_reader_relative = $generic_reader_relative;
            generic_reader_many = $generic_reader_many;
            generic_reader_many_relative = $generic_reader_many_relative;
        }

        typed_strict_first_methods! {
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
            generic_reader = $generic_reader;
            generic_reader_relative = $generic_reader_relative;
            generic_reader_many = $generic_reader_many;
            generic_reader_many_relative = $generic_reader_many_relative;
        }

        typed_strict_first_methods! {
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
            generic_reader = $generic_reader;
            generic_reader_relative = $generic_reader_relative;
            generic_reader_many = $generic_reader_many;
            generic_reader_many_relative = $generic_reader_many_relative;
        }

        typed_strict_first_methods! {
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
            generic_reader = $generic_reader;
            generic_reader_relative = $generic_reader_relative;
            generic_reader_many = $generic_reader_many;
            generic_reader_many_relative = $generic_reader_many_relative;
        }
    };
}

typed_strict_first_output! {
    what = "spectrum";
    output = LoadedSpectrum;
    generic_reader = read_first_spectrum_strict_by_source;
    generic_reader_relative = read_first_spectrum_strict_by_source_relative_to;
    generic_reader_many = read_first_spectrum_many_strict_by_source;
    generic_reader_many_relative = read_first_spectrum_many_strict_by_source_relative_to;
    format = [read_first_spectrum_strict_by_source_format, read_first_spectrum_strict_by_source_format_relative_to, read_first_spectrum_many_strict_by_source_format, read_first_spectrum_many_strict_by_source_format_relative_to, load_first_spectrum_strict_by_source_format, load_first_spectrum_strict_by_source_format_relative_to, load_first_spectrum_many_strict_by_source_format, load_first_spectrum_many_strict_by_source_format_relative_to];
    vendor = [read_first_spectrum_strict_by_source_vendor, read_first_spectrum_strict_by_source_vendor_relative_to, read_first_spectrum_many_strict_by_source_vendor, read_first_spectrum_many_strict_by_source_vendor_relative_to, load_first_spectrum_strict_by_source_vendor, load_first_spectrum_strict_by_source_vendor_relative_to, load_first_spectrum_many_strict_by_source_vendor, load_first_spectrum_many_strict_by_source_vendor_relative_to];
    data_kind = [read_first_spectrum_strict_by_source_data_kind, read_first_spectrum_strict_by_source_data_kind_relative_to, read_first_spectrum_many_strict_by_source_data_kind, read_first_spectrum_many_strict_by_source_data_kind_relative_to, load_first_spectrum_strict_by_source_data_kind, load_first_spectrum_strict_by_source_data_kind_relative_to, load_first_spectrum_many_strict_by_source_data_kind, load_first_spectrum_many_strict_by_source_data_kind_relative_to];
    path = [read_first_spectrum_strict_by_source_path, read_first_spectrum_strict_by_source_path_relative_to, read_first_spectrum_many_strict_by_source_path, read_first_spectrum_many_strict_by_source_path_relative_to, load_first_spectrum_strict_by_source_path, load_first_spectrum_strict_by_source_path_relative_to, load_first_spectrum_many_strict_by_source_path, load_first_spectrum_many_strict_by_source_path_relative_to];
    path_prefix = [read_first_spectrum_strict_by_source_path_prefix, read_first_spectrum_strict_by_source_path_prefix_relative_to, read_first_spectrum_many_strict_by_source_path_prefix, read_first_spectrum_many_strict_by_source_path_prefix_relative_to, load_first_spectrum_strict_by_source_path_prefix, load_first_spectrum_strict_by_source_path_prefix_relative_to, load_first_spectrum_many_strict_by_source_path_prefix, load_first_spectrum_many_strict_by_source_path_prefix_relative_to];
}

typed_strict_first_output! {
    what = "one-dimensional spectrum";
    output = Spectrum1D;
    generic_reader = read_first_1d_strict_by_source;
    generic_reader_relative = read_first_1d_strict_by_source_relative_to;
    generic_reader_many = read_first_1d_many_strict_by_source;
    generic_reader_many_relative = read_first_1d_many_strict_by_source_relative_to;
    format = [read_first_1d_strict_by_source_format, read_first_1d_strict_by_source_format_relative_to, read_first_1d_many_strict_by_source_format, read_first_1d_many_strict_by_source_format_relative_to, load_first_spectrum_1d_strict_by_source_format, load_first_spectrum_1d_strict_by_source_format_relative_to, load_first_spectrum_1d_many_strict_by_source_format, load_first_spectrum_1d_many_strict_by_source_format_relative_to];
    vendor = [read_first_1d_strict_by_source_vendor, read_first_1d_strict_by_source_vendor_relative_to, read_first_1d_many_strict_by_source_vendor, read_first_1d_many_strict_by_source_vendor_relative_to, load_first_spectrum_1d_strict_by_source_vendor, load_first_spectrum_1d_strict_by_source_vendor_relative_to, load_first_spectrum_1d_many_strict_by_source_vendor, load_first_spectrum_1d_many_strict_by_source_vendor_relative_to];
    data_kind = [read_first_1d_strict_by_source_data_kind, read_first_1d_strict_by_source_data_kind_relative_to, read_first_1d_many_strict_by_source_data_kind, read_first_1d_many_strict_by_source_data_kind_relative_to, load_first_spectrum_1d_strict_by_source_data_kind, load_first_spectrum_1d_strict_by_source_data_kind_relative_to, load_first_spectrum_1d_many_strict_by_source_data_kind, load_first_spectrum_1d_many_strict_by_source_data_kind_relative_to];
    path = [read_first_1d_strict_by_source_path, read_first_1d_strict_by_source_path_relative_to, read_first_1d_many_strict_by_source_path, read_first_1d_many_strict_by_source_path_relative_to, load_first_spectrum_1d_strict_by_source_path, load_first_spectrum_1d_strict_by_source_path_relative_to, load_first_spectrum_1d_many_strict_by_source_path, load_first_spectrum_1d_many_strict_by_source_path_relative_to];
    path_prefix = [read_first_1d_strict_by_source_path_prefix, read_first_1d_strict_by_source_path_prefix_relative_to, read_first_1d_many_strict_by_source_path_prefix, read_first_1d_many_strict_by_source_path_prefix_relative_to, load_first_spectrum_1d_strict_by_source_path_prefix, load_first_spectrum_1d_strict_by_source_path_prefix_relative_to, load_first_spectrum_1d_many_strict_by_source_path_prefix, load_first_spectrum_1d_many_strict_by_source_path_prefix_relative_to];
}

typed_strict_first_output! {
    what = "one-dimensional spectrum with source metadata";
    output = (Spectrum1D, LoadedSource);
    generic_reader = read_first_1d_with_source_strict_by_source;
    generic_reader_relative = read_first_1d_with_source_strict_by_source_relative_to;
    generic_reader_many = read_first_1d_many_with_source_strict_by_source;
    generic_reader_many_relative = read_first_1d_many_with_source_strict_by_source_relative_to;
    format = [read_first_1d_with_source_strict_by_source_format, read_first_1d_with_source_strict_by_source_format_relative_to, read_first_1d_many_with_source_strict_by_source_format, read_first_1d_many_with_source_strict_by_source_format_relative_to, load_first_spectrum_1d_with_source_strict_by_source_format, load_first_spectrum_1d_with_source_strict_by_source_format_relative_to, load_first_spectrum_1d_many_with_source_strict_by_source_format, load_first_spectrum_1d_many_with_source_strict_by_source_format_relative_to];
    vendor = [read_first_1d_with_source_strict_by_source_vendor, read_first_1d_with_source_strict_by_source_vendor_relative_to, read_first_1d_many_with_source_strict_by_source_vendor, read_first_1d_many_with_source_strict_by_source_vendor_relative_to, load_first_spectrum_1d_with_source_strict_by_source_vendor, load_first_spectrum_1d_with_source_strict_by_source_vendor_relative_to, load_first_spectrum_1d_many_with_source_strict_by_source_vendor, load_first_spectrum_1d_many_with_source_strict_by_source_vendor_relative_to];
    data_kind = [read_first_1d_with_source_strict_by_source_data_kind, read_first_1d_with_source_strict_by_source_data_kind_relative_to, read_first_1d_many_with_source_strict_by_source_data_kind, read_first_1d_many_with_source_strict_by_source_data_kind_relative_to, load_first_spectrum_1d_with_source_strict_by_source_data_kind, load_first_spectrum_1d_with_source_strict_by_source_data_kind_relative_to, load_first_spectrum_1d_many_with_source_strict_by_source_data_kind, load_first_spectrum_1d_many_with_source_strict_by_source_data_kind_relative_to];
    path = [read_first_1d_with_source_strict_by_source_path, read_first_1d_with_source_strict_by_source_path_relative_to, read_first_1d_many_with_source_strict_by_source_path, read_first_1d_many_with_source_strict_by_source_path_relative_to, load_first_spectrum_1d_with_source_strict_by_source_path, load_first_spectrum_1d_with_source_strict_by_source_path_relative_to, load_first_spectrum_1d_many_with_source_strict_by_source_path, load_first_spectrum_1d_many_with_source_strict_by_source_path_relative_to];
    path_prefix = [read_first_1d_with_source_strict_by_source_path_prefix, read_first_1d_with_source_strict_by_source_path_prefix_relative_to, read_first_1d_many_with_source_strict_by_source_path_prefix, read_first_1d_many_with_source_strict_by_source_path_prefix_relative_to, load_first_spectrum_1d_with_source_strict_by_source_path_prefix, load_first_spectrum_1d_with_source_strict_by_source_path_prefix_relative_to, load_first_spectrum_1d_many_with_source_strict_by_source_path_prefix, load_first_spectrum_1d_many_with_source_strict_by_source_path_prefix_relative_to];
}

typed_strict_first_output! {
    what = "two-dimensional spectrum";
    output = Spectrum2D;
    generic_reader = read_first_2d_strict_by_source;
    generic_reader_relative = read_first_2d_strict_by_source_relative_to;
    generic_reader_many = read_first_2d_many_strict_by_source;
    generic_reader_many_relative = read_first_2d_many_strict_by_source_relative_to;
    format = [read_first_2d_strict_by_source_format, read_first_2d_strict_by_source_format_relative_to, read_first_2d_many_strict_by_source_format, read_first_2d_many_strict_by_source_format_relative_to, load_first_spectrum_2d_strict_by_source_format, load_first_spectrum_2d_strict_by_source_format_relative_to, load_first_spectrum_2d_many_strict_by_source_format, load_first_spectrum_2d_many_strict_by_source_format_relative_to];
    vendor = [read_first_2d_strict_by_source_vendor, read_first_2d_strict_by_source_vendor_relative_to, read_first_2d_many_strict_by_source_vendor, read_first_2d_many_strict_by_source_vendor_relative_to, load_first_spectrum_2d_strict_by_source_vendor, load_first_spectrum_2d_strict_by_source_vendor_relative_to, load_first_spectrum_2d_many_strict_by_source_vendor, load_first_spectrum_2d_many_strict_by_source_vendor_relative_to];
    data_kind = [read_first_2d_strict_by_source_data_kind, read_first_2d_strict_by_source_data_kind_relative_to, read_first_2d_many_strict_by_source_data_kind, read_first_2d_many_strict_by_source_data_kind_relative_to, load_first_spectrum_2d_strict_by_source_data_kind, load_first_spectrum_2d_strict_by_source_data_kind_relative_to, load_first_spectrum_2d_many_strict_by_source_data_kind, load_first_spectrum_2d_many_strict_by_source_data_kind_relative_to];
    path = [read_first_2d_strict_by_source_path, read_first_2d_strict_by_source_path_relative_to, read_first_2d_many_strict_by_source_path, read_first_2d_many_strict_by_source_path_relative_to, load_first_spectrum_2d_strict_by_source_path, load_first_spectrum_2d_strict_by_source_path_relative_to, load_first_spectrum_2d_many_strict_by_source_path, load_first_spectrum_2d_many_strict_by_source_path_relative_to];
    path_prefix = [read_first_2d_strict_by_source_path_prefix, read_first_2d_strict_by_source_path_prefix_relative_to, read_first_2d_many_strict_by_source_path_prefix, read_first_2d_many_strict_by_source_path_prefix_relative_to, load_first_spectrum_2d_strict_by_source_path_prefix, load_first_spectrum_2d_strict_by_source_path_prefix_relative_to, load_first_spectrum_2d_many_strict_by_source_path_prefix, load_first_spectrum_2d_many_strict_by_source_path_prefix_relative_to];
}

typed_strict_first_output! {
    what = "two-dimensional spectrum with source metadata";
    output = (Spectrum2D, LoadedSource);
    generic_reader = read_first_2d_with_source_strict_by_source;
    generic_reader_relative = read_first_2d_with_source_strict_by_source_relative_to;
    generic_reader_many = read_first_2d_many_with_source_strict_by_source;
    generic_reader_many_relative = read_first_2d_many_with_source_strict_by_source_relative_to;
    format = [read_first_2d_with_source_strict_by_source_format, read_first_2d_with_source_strict_by_source_format_relative_to, read_first_2d_many_with_source_strict_by_source_format, read_first_2d_many_with_source_strict_by_source_format_relative_to, load_first_spectrum_2d_with_source_strict_by_source_format, load_first_spectrum_2d_with_source_strict_by_source_format_relative_to, load_first_spectrum_2d_many_with_source_strict_by_source_format, load_first_spectrum_2d_many_with_source_strict_by_source_format_relative_to];
    vendor = [read_first_2d_with_source_strict_by_source_vendor, read_first_2d_with_source_strict_by_source_vendor_relative_to, read_first_2d_many_with_source_strict_by_source_vendor, read_first_2d_many_with_source_strict_by_source_vendor_relative_to, load_first_spectrum_2d_with_source_strict_by_source_vendor, load_first_spectrum_2d_with_source_strict_by_source_vendor_relative_to, load_first_spectrum_2d_many_with_source_strict_by_source_vendor, load_first_spectrum_2d_many_with_source_strict_by_source_vendor_relative_to];
    data_kind = [read_first_2d_with_source_strict_by_source_data_kind, read_first_2d_with_source_strict_by_source_data_kind_relative_to, read_first_2d_many_with_source_strict_by_source_data_kind, read_first_2d_many_with_source_strict_by_source_data_kind_relative_to, load_first_spectrum_2d_with_source_strict_by_source_data_kind, load_first_spectrum_2d_with_source_strict_by_source_data_kind_relative_to, load_first_spectrum_2d_many_with_source_strict_by_source_data_kind, load_first_spectrum_2d_many_with_source_strict_by_source_data_kind_relative_to];
    path = [read_first_2d_with_source_strict_by_source_path, read_first_2d_with_source_strict_by_source_path_relative_to, read_first_2d_many_with_source_strict_by_source_path, read_first_2d_many_with_source_strict_by_source_path_relative_to, load_first_spectrum_2d_with_source_strict_by_source_path, load_first_spectrum_2d_with_source_strict_by_source_path_relative_to, load_first_spectrum_2d_many_with_source_strict_by_source_path, load_first_spectrum_2d_many_with_source_strict_by_source_path_relative_to];
    path_prefix = [read_first_2d_with_source_strict_by_source_path_prefix, read_first_2d_with_source_strict_by_source_path_prefix_relative_to, read_first_2d_many_with_source_strict_by_source_path_prefix, read_first_2d_many_with_source_strict_by_source_path_prefix_relative_to, load_first_spectrum_2d_with_source_strict_by_source_path_prefix, load_first_spectrum_2d_with_source_strict_by_source_path_prefix_relative_to, load_first_spectrum_2d_many_with_source_strict_by_source_path_prefix, load_first_spectrum_2d_many_with_source_strict_by_source_path_prefix_relative_to];
}
