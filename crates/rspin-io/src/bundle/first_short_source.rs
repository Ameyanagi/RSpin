//! Short source-metadata aliases for first-spectrum reader helpers.

use std::path::Path;

use rspin_core::{Result, Spectrum1D, Spectrum2D};

use super::{LoadedSource, LoadedSourceDataKind, LoadedSpectrum, SpectrumBundleLoader};

macro_rules! short_first_methods {
    (
        filter = $filter_doc:literal;
        value = $value:ident : $value_ty:ty;
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
        target = $target:ident;
        target_relative = $target_relative:ident;
        target_many = $target_many:ident;
        target_many_relative = $target_many_relative:ident;
    ) => {
        impl SpectrumBundleLoader {
            #[doc = concat!("Loads a file or directory and returns the first ", $what, " matching one ", $filter_doc, ".")]
            ///
            /// This is a short alias for the matching `read_first_*_by_source_*` method.
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum is found.
            pub fn $reader(
                &self,
                path: impl AsRef<Path>,
                $value: $value_ty,
            ) -> Result<$output> {
                self.$target(path, $value)
            }

            #[doc = concat!("Loads one selected path relative to a base directory and returns the first ", $what, " matching one ", $filter_doc, ".")]
            ///
            /// This is a short alias for the matching `read_first_*_by_source_*_relative_to` method.
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum is found.
            pub fn $reader_relative(
                &self,
                base: impl AsRef<Path>,
                path: impl AsRef<Path>,
                $value: $value_ty,
            ) -> Result<$output> {
                self.$target_relative(base, path, $value)
            }

            #[doc = concat!("Loads selected paths and returns the first ", $what, " matching one ", $filter_doc, ".")]
            ///
            /// This is a short alias for the matching `read_first_*_many_by_source_*` method.
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum is found.
            pub fn $reader_many<I, P>(&self, paths: I, $value: $value_ty) -> Result<$output>
            where
                I: IntoIterator<Item = P>,
                P: AsRef<Path>,
            {
                self.$target_many(paths, $value)
            }

            #[doc = concat!("Loads selected paths relative to a base directory and returns the first ", $what, " matching one ", $filter_doc, ".")]
            ///
            /// This is a short alias for the matching `read_first_*_many_by_source_*_relative_to` method.
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum is found.
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
                self.$target_many_relative(base, paths, $value)
            }
        }

        #[doc = concat!("Loads a file or directory and returns the first ", $what, " matching one ", $filter_doc, ".")]
        ///
        /// This is a short alias for the matching `load_first_spectrum_*_by_source_*` function.
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum is found.
        pub fn $free(path: impl AsRef<Path>, $value: $value_ty) -> Result<$output> {
            SpectrumBundleLoader::new().$reader(path, $value)
        }

        #[doc = concat!("Loads one selected path relative to a base directory and returns the first ", $what, " matching one ", $filter_doc, ".")]
        ///
        /// This is a short alias for the matching `load_first_spectrum_*_by_source_*_relative_to` function.
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum is found.
        pub fn $free_relative(
            base: impl AsRef<Path>,
            path: impl AsRef<Path>,
            $value: $value_ty,
        ) -> Result<$output> {
            SpectrumBundleLoader::new().$reader_relative(base, path, $value)
        }

        #[doc = concat!("Loads selected paths and returns the first ", $what, " matching one ", $filter_doc, ".")]
        ///
        /// This is a short alias for the matching `load_first_spectrum_*_many_by_source_*` function.
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum is found.
        pub fn $free_many<I, P>(paths: I, $value: $value_ty) -> Result<$output>
        where
            I: IntoIterator<Item = P>,
            P: AsRef<Path>,
        {
            SpectrumBundleLoader::new().$reader_many(paths, $value)
        }

        #[doc = concat!("Loads selected paths relative to a base directory and returns the first ", $what, " matching one ", $filter_doc, ".")]
        ///
        /// This is a short alias for the matching `load_first_spectrum_*_many_by_source_*_relative_to` function.
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum is found.
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

macro_rules! short_first_output {
    (
        what = $what:literal;
        output = $output:ty;
        format = [$format_reader:ident, $format_reader_relative:ident, $format_reader_many:ident, $format_reader_many_relative:ident, $format_free:ident, $format_free_relative:ident, $format_free_many:ident, $format_free_many_relative:ident, $format_target:ident, $format_target_relative:ident, $format_target_many:ident, $format_target_many_relative:ident];
        vendor = [$vendor_reader:ident, $vendor_reader_relative:ident, $vendor_reader_many:ident, $vendor_reader_many_relative:ident, $vendor_free:ident, $vendor_free_relative:ident, $vendor_free_many:ident, $vendor_free_many_relative:ident, $vendor_target:ident, $vendor_target_relative:ident, $vendor_target_many:ident, $vendor_target_many_relative:ident];
        data_kind = [$data_kind_reader:ident, $data_kind_reader_relative:ident, $data_kind_reader_many:ident, $data_kind_reader_many_relative:ident, $data_kind_free:ident, $data_kind_free_relative:ident, $data_kind_free_many:ident, $data_kind_free_many_relative:ident, $data_kind_target:ident, $data_kind_target_relative:ident, $data_kind_target_many:ident, $data_kind_target_many_relative:ident];
    ) => {
        short_first_methods! {
            filter = "source format";
            value = format: impl AsRef<str>;
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
            target = $format_target;
            target_relative = $format_target_relative;
            target_many = $format_target_many;
            target_many_relative = $format_target_many_relative;
        }

        short_first_methods! {
            filter = "vendor family";
            value = vendor: impl AsRef<str>;
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
            target = $vendor_target;
            target_relative = $vendor_target_relative;
            target_many = $vendor_target_many;
            target_many_relative = $vendor_target_many_relative;
        }

        short_first_methods! {
            filter = "raw/processed source data kind";
            value = data_kind: LoadedSourceDataKind;
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
            target = $data_kind_target;
            target_relative = $data_kind_target_relative;
            target_many = $data_kind_target_many;
            target_many_relative = $data_kind_target_many_relative;
        }
    };
}

short_first_output! {
    what = "spectrum";
    output = LoadedSpectrum;
    format = [read_first_by_format, read_first_by_format_relative_to, read_first_many_by_format, read_first_many_by_format_relative_to, load_first_spectrum_by_format, load_first_spectrum_by_format_relative_to, load_first_spectrum_many_by_format, load_first_spectrum_many_by_format_relative_to, read_first_spectrum_by_source_format, read_first_spectrum_by_source_format_relative_to, read_first_spectrum_many_by_source_format, read_first_spectrum_many_by_source_format_relative_to];
    vendor = [read_first_by_vendor, read_first_by_vendor_relative_to, read_first_many_by_vendor, read_first_many_by_vendor_relative_to, load_first_spectrum_by_vendor, load_first_spectrum_by_vendor_relative_to, load_first_spectrum_many_by_vendor, load_first_spectrum_many_by_vendor_relative_to, read_first_spectrum_by_source_vendor, read_first_spectrum_by_source_vendor_relative_to, read_first_spectrum_many_by_source_vendor, read_first_spectrum_many_by_source_vendor_relative_to];
    data_kind = [read_first_by_data_kind, read_first_by_data_kind_relative_to, read_first_many_by_data_kind, read_first_many_by_data_kind_relative_to, load_first_spectrum_by_data_kind, load_first_spectrum_by_data_kind_relative_to, load_first_spectrum_many_by_data_kind, load_first_spectrum_many_by_data_kind_relative_to, read_first_spectrum_by_source_data_kind, read_first_spectrum_by_source_data_kind_relative_to, read_first_spectrum_many_by_source_data_kind, read_first_spectrum_many_by_source_data_kind_relative_to];
}

short_first_output! {
    what = "one-dimensional spectrum";
    output = Spectrum1D;
    format = [read_first_1d_by_format, read_first_1d_by_format_relative_to, read_first_1d_many_by_format, read_first_1d_many_by_format_relative_to, load_first_spectrum_1d_by_format, load_first_spectrum_1d_by_format_relative_to, load_first_spectrum_1d_many_by_format, load_first_spectrum_1d_many_by_format_relative_to, read_first_1d_by_source_format, read_first_1d_by_source_format_relative_to, read_first_1d_many_by_source_format, read_first_1d_many_by_source_format_relative_to];
    vendor = [read_first_1d_by_vendor, read_first_1d_by_vendor_relative_to, read_first_1d_many_by_vendor, read_first_1d_many_by_vendor_relative_to, load_first_spectrum_1d_by_vendor, load_first_spectrum_1d_by_vendor_relative_to, load_first_spectrum_1d_many_by_vendor, load_first_spectrum_1d_many_by_vendor_relative_to, read_first_1d_by_source_vendor, read_first_1d_by_source_vendor_relative_to, read_first_1d_many_by_source_vendor, read_first_1d_many_by_source_vendor_relative_to];
    data_kind = [read_first_1d_by_data_kind, read_first_1d_by_data_kind_relative_to, read_first_1d_many_by_data_kind, read_first_1d_many_by_data_kind_relative_to, load_first_spectrum_1d_by_data_kind, load_first_spectrum_1d_by_data_kind_relative_to, load_first_spectrum_1d_many_by_data_kind, load_first_spectrum_1d_many_by_data_kind_relative_to, read_first_1d_by_source_data_kind, read_first_1d_by_source_data_kind_relative_to, read_first_1d_many_by_source_data_kind, read_first_1d_many_by_source_data_kind_relative_to];
}

short_first_output! {
    what = "one-dimensional spectrum with source metadata";
    output = (Spectrum1D, LoadedSource);
    format = [read_first_1d_with_source_by_format, read_first_1d_with_source_by_format_relative_to, read_first_1d_many_with_source_by_format, read_first_1d_many_with_source_by_format_relative_to, load_first_spectrum_1d_with_source_by_format, load_first_spectrum_1d_with_source_by_format_relative_to, load_first_spectrum_1d_many_with_source_by_format, load_first_spectrum_1d_many_with_source_by_format_relative_to, read_first_1d_with_source_by_source_format, read_first_1d_with_source_by_source_format_relative_to, read_first_1d_many_with_source_by_source_format, read_first_1d_many_with_source_by_source_format_relative_to];
    vendor = [read_first_1d_with_source_by_vendor, read_first_1d_with_source_by_vendor_relative_to, read_first_1d_many_with_source_by_vendor, read_first_1d_many_with_source_by_vendor_relative_to, load_first_spectrum_1d_with_source_by_vendor, load_first_spectrum_1d_with_source_by_vendor_relative_to, load_first_spectrum_1d_many_with_source_by_vendor, load_first_spectrum_1d_many_with_source_by_vendor_relative_to, read_first_1d_with_source_by_source_vendor, read_first_1d_with_source_by_source_vendor_relative_to, read_first_1d_many_with_source_by_source_vendor, read_first_1d_many_with_source_by_source_vendor_relative_to];
    data_kind = [read_first_1d_with_source_by_data_kind, read_first_1d_with_source_by_data_kind_relative_to, read_first_1d_many_with_source_by_data_kind, read_first_1d_many_with_source_by_data_kind_relative_to, load_first_spectrum_1d_with_source_by_data_kind, load_first_spectrum_1d_with_source_by_data_kind_relative_to, load_first_spectrum_1d_many_with_source_by_data_kind, load_first_spectrum_1d_many_with_source_by_data_kind_relative_to, read_first_1d_with_source_by_source_data_kind, read_first_1d_with_source_by_source_data_kind_relative_to, read_first_1d_many_with_source_by_source_data_kind, read_first_1d_many_with_source_by_source_data_kind_relative_to];
}

short_first_output! {
    what = "two-dimensional spectrum";
    output = Spectrum2D;
    format = [read_first_2d_by_format, read_first_2d_by_format_relative_to, read_first_2d_many_by_format, read_first_2d_many_by_format_relative_to, load_first_spectrum_2d_by_format, load_first_spectrum_2d_by_format_relative_to, load_first_spectrum_2d_many_by_format, load_first_spectrum_2d_many_by_format_relative_to, read_first_2d_by_source_format, read_first_2d_by_source_format_relative_to, read_first_2d_many_by_source_format, read_first_2d_many_by_source_format_relative_to];
    vendor = [read_first_2d_by_vendor, read_first_2d_by_vendor_relative_to, read_first_2d_many_by_vendor, read_first_2d_many_by_vendor_relative_to, load_first_spectrum_2d_by_vendor, load_first_spectrum_2d_by_vendor_relative_to, load_first_spectrum_2d_many_by_vendor, load_first_spectrum_2d_many_by_vendor_relative_to, read_first_2d_by_source_vendor, read_first_2d_by_source_vendor_relative_to, read_first_2d_many_by_source_vendor, read_first_2d_many_by_source_vendor_relative_to];
    data_kind = [read_first_2d_by_data_kind, read_first_2d_by_data_kind_relative_to, read_first_2d_many_by_data_kind, read_first_2d_many_by_data_kind_relative_to, load_first_spectrum_2d_by_data_kind, load_first_spectrum_2d_by_data_kind_relative_to, load_first_spectrum_2d_many_by_data_kind, load_first_spectrum_2d_many_by_data_kind_relative_to, read_first_2d_by_source_data_kind, read_first_2d_by_source_data_kind_relative_to, read_first_2d_many_by_source_data_kind, read_first_2d_many_by_source_data_kind_relative_to];
}

short_first_output! {
    what = "two-dimensional spectrum with source metadata";
    output = (Spectrum2D, LoadedSource);
    format = [read_first_2d_with_source_by_format, read_first_2d_with_source_by_format_relative_to, read_first_2d_many_with_source_by_format, read_first_2d_many_with_source_by_format_relative_to, load_first_spectrum_2d_with_source_by_format, load_first_spectrum_2d_with_source_by_format_relative_to, load_first_spectrum_2d_many_with_source_by_format, load_first_spectrum_2d_many_with_source_by_format_relative_to, read_first_2d_with_source_by_source_format, read_first_2d_with_source_by_source_format_relative_to, read_first_2d_many_with_source_by_source_format, read_first_2d_many_with_source_by_source_format_relative_to];
    vendor = [read_first_2d_with_source_by_vendor, read_first_2d_with_source_by_vendor_relative_to, read_first_2d_many_with_source_by_vendor, read_first_2d_many_with_source_by_vendor_relative_to, load_first_spectrum_2d_with_source_by_vendor, load_first_spectrum_2d_with_source_by_vendor_relative_to, load_first_spectrum_2d_many_with_source_by_vendor, load_first_spectrum_2d_many_with_source_by_vendor_relative_to, read_first_2d_with_source_by_source_vendor, read_first_2d_with_source_by_source_vendor_relative_to, read_first_2d_many_with_source_by_source_vendor, read_first_2d_many_with_source_by_source_vendor_relative_to];
    data_kind = [read_first_2d_with_source_by_data_kind, read_first_2d_with_source_by_data_kind_relative_to, read_first_2d_many_with_source_by_data_kind, read_first_2d_many_with_source_by_data_kind_relative_to, load_first_spectrum_2d_with_source_by_data_kind, load_first_spectrum_2d_with_source_by_data_kind_relative_to, load_first_spectrum_2d_many_with_source_by_data_kind, load_first_spectrum_2d_many_with_source_by_data_kind_relative_to, read_first_2d_with_source_by_source_data_kind, read_first_2d_with_source_by_source_data_kind_relative_to, read_first_2d_many_with_source_by_source_data_kind, read_first_2d_many_with_source_by_source_data_kind_relative_to];
}
