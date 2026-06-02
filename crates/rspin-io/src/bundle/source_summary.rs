//! Source-filtered summary helpers for direct spectrum bundle loading.

use std::path::Path;

use rspin_core::Result;

use super::{
    LoadedSourceDataKind, LoadedSourceFilter, SpectrumBundleLoader, SpectrumBundleSummary,
};

macro_rules! single_summary_helpers {
    (
        what = $what:literal;
        reader = $reader:ident;
        reader_relative = $reader_relative:ident;
        reader_many = $reader_many:ident;
        reader_many_relative = $reader_many_relative:ident;
        load = $load:ident;
        load_relative = $load_relative:ident;
        load_many = $load_many:ident;
        load_many_relative = $load_many_relative:ident;
        setter = $setter:ident;
        value = $value:ident : $value_ty:ty;
    ) => {
        impl SpectrumBundleLoader {
            #[doc = concat!("Loads a file or directory and returns summary counts, restricted by one ", $what, ".")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum data is found.
            pub fn $reader(
                &self,
                path: impl AsRef<Path>,
                $value: $value_ty,
            ) -> Result<SpectrumBundleSummary> {
                self.clone().$setter($value).read_summary_path(path)
            }

            #[doc = concat!("Loads one selected path relative to a base directory and returns summary counts, restricted by one ", $what, ".")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum data is found.
            pub fn $reader_relative(
                &self,
                base: impl AsRef<Path>,
                path: impl AsRef<Path>,
                $value: $value_ty,
            ) -> Result<SpectrumBundleSummary> {
                self.clone()
                    .$setter($value)
                    .read_summary_path_relative_to(base, path)
            }

            #[doc = concat!("Loads multiple paths and returns summary counts, restricted by one ", $what, ".")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum data is found.
            pub fn $reader_many<I, P>(
                &self,
                paths: I,
                $value: $value_ty,
            ) -> Result<SpectrumBundleSummary>
            where
                I: IntoIterator<Item = P>,
                P: AsRef<Path>,
            {
                self.clone().$setter($value).read_summary_paths(paths)
            }

            #[doc = concat!("Loads selected paths relative to a base directory and returns summary counts, restricted by one ", $what, ".")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum data is found.
            pub fn $reader_many_relative<I, P>(
                &self,
                base: impl AsRef<Path>,
                paths: I,
                $value: $value_ty,
            ) -> Result<SpectrumBundleSummary>
            where
                I: IntoIterator<Item = P>,
                P: AsRef<Path>,
            {
                self.clone()
                    .$setter($value)
                    .read_summary_paths_relative_to(base, paths)
            }
        }

        #[doc = concat!("Loads a file or directory and returns summary counts, restricted by one ", $what, ".")]
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $load(path: impl AsRef<Path>, $value: $value_ty) -> Result<SpectrumBundleSummary> {
            SpectrumBundleLoader::new().$reader(path, $value)
        }

        #[doc = concat!("Loads one selected path relative to a base directory and returns summary counts, restricted by one ", $what, ".")]
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $load_relative(
            base: impl AsRef<Path>,
            path: impl AsRef<Path>,
            $value: $value_ty,
        ) -> Result<SpectrumBundleSummary> {
            SpectrumBundleLoader::new().$reader_relative(base, path, $value)
        }

        #[doc = concat!("Loads multiple paths and returns summary counts, restricted by one ", $what, ".")]
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $load_many<I, P>(paths: I, $value: $value_ty) -> Result<SpectrumBundleSummary>
        where
            I: IntoIterator<Item = P>,
            P: AsRef<Path>,
        {
            SpectrumBundleLoader::new().$reader_many(paths, $value)
        }

        #[doc = concat!("Loads selected paths relative to a base directory and returns summary counts, restricted by one ", $what, ".")]
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $load_many_relative<I, P>(
            base: impl AsRef<Path>,
            paths: I,
            $value: $value_ty,
        ) -> Result<SpectrumBundleSummary>
        where
            I: IntoIterator<Item = P>,
            P: AsRef<Path>,
        {
            SpectrumBundleLoader::new().$reader_many_relative(base, paths, $value)
        }
    };
}

macro_rules! set_summary_helpers {
    (
        what = $what:literal;
        reader = $reader:ident;
        reader_relative = $reader_relative:ident;
        reader_many = $reader_many:ident;
        reader_many_relative = $reader_many_relative:ident;
        load = $load:ident;
        load_relative = $load_relative:ident;
        load_many = $load_many:ident;
        load_many_relative = $load_many_relative:ident;
        setter = $setter:ident;
        values = $values:ident;
        generics = [$($generics:tt)*];
        where = {$($where_clause:tt)*};
    ) => {
        impl SpectrumBundleLoader {
            #[doc = concat!("Loads a file or directory and returns summary counts, restricted by ", $what, ".")]
            ///
            /// Values are combined with logical OR. Passing an empty iterator
            /// leaves source loading unrestricted.
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum data is found.
            pub fn $reader<$($generics)*>(
                &self,
                path: impl AsRef<Path>,
                $values: J,
            ) -> Result<SpectrumBundleSummary>
            where
                $($where_clause)*
            {
                self.clone().$setter($values).read_summary_path(path)
            }

            #[doc = concat!("Loads one selected path relative to a base directory and returns summary counts, restricted by ", $what, ".")]
            ///
            /// Values are combined with logical OR. Passing an empty iterator
            /// leaves source loading unrestricted.
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum data is found.
            pub fn $reader_relative<$($generics)*>(
                &self,
                base: impl AsRef<Path>,
                path: impl AsRef<Path>,
                $values: J,
            ) -> Result<SpectrumBundleSummary>
            where
                $($where_clause)*
            {
                self.clone()
                    .$setter($values)
                    .read_summary_path_relative_to(base, path)
            }

            #[doc = concat!("Loads multiple paths and returns summary counts, restricted by ", $what, ".")]
            ///
            /// Values are combined with logical OR. Passing an empty iterator
            /// leaves source loading unrestricted.
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum data is found.
            pub fn $reader_many<I, P, $($generics)*>(
                &self,
                paths: I,
                $values: J,
            ) -> Result<SpectrumBundleSummary>
            where
                I: IntoIterator<Item = P>,
                P: AsRef<Path>,
                $($where_clause)*
            {
                self.clone().$setter($values).read_summary_paths(paths)
            }

            #[doc = concat!("Loads selected paths relative to a base directory and returns summary counts, restricted by ", $what, ".")]
            ///
            /// Values are combined with logical OR. Passing an empty iterator
            /// leaves source loading unrestricted.
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum data is found.
            pub fn $reader_many_relative<I, P, $($generics)*>(
                &self,
                base: impl AsRef<Path>,
                paths: I,
                $values: J,
            ) -> Result<SpectrumBundleSummary>
            where
                I: IntoIterator<Item = P>,
                P: AsRef<Path>,
                $($where_clause)*
            {
                self.clone()
                    .$setter($values)
                    .read_summary_paths_relative_to(base, paths)
            }
        }

        #[doc = concat!("Loads a file or directory and returns summary counts, restricted by ", $what, ".")]
        ///
        /// Values are combined with logical OR. Passing an empty iterator
        /// leaves source loading unrestricted.
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $load<$($generics)*>(
            path: impl AsRef<Path>,
            $values: J,
        ) -> Result<SpectrumBundleSummary>
        where
            $($where_clause)*
        {
            SpectrumBundleLoader::new().$reader(path, $values)
        }

        #[doc = concat!("Loads one selected path relative to a base directory and returns summary counts, restricted by ", $what, ".")]
        ///
        /// Values are combined with logical OR. Passing an empty iterator
        /// leaves source loading unrestricted.
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $load_relative<$($generics)*>(
            base: impl AsRef<Path>,
            path: impl AsRef<Path>,
            $values: J,
        ) -> Result<SpectrumBundleSummary>
        where
            $($where_clause)*
        {
            SpectrumBundleLoader::new().$reader_relative(base, path, $values)
        }

        #[doc = concat!("Loads multiple paths and returns summary counts, restricted by ", $what, ".")]
        ///
        /// Values are combined with logical OR. Passing an empty iterator
        /// leaves source loading unrestricted.
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $load_many<I, P, $($generics)*>(
            paths: I,
            $values: J,
        ) -> Result<SpectrumBundleSummary>
        where
            I: IntoIterator<Item = P>,
            P: AsRef<Path>,
            $($where_clause)*
        {
            SpectrumBundleLoader::new().$reader_many(paths, $values)
        }

        #[doc = concat!("Loads selected paths relative to a base directory and returns summary counts, restricted by ", $what, ".")]
        ///
        /// Values are combined with logical OR. Passing an empty iterator
        /// leaves source loading unrestricted.
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $load_many_relative<I, P, $($generics)*>(
            base: impl AsRef<Path>,
            paths: I,
            $values: J,
        ) -> Result<SpectrumBundleSummary>
        where
            I: IntoIterator<Item = P>,
            P: AsRef<Path>,
            $($where_clause)*
        {
            SpectrumBundleLoader::new().$reader_many_relative(base, paths, $values)
        }
    };
}

single_summary_helpers! {
    what = "generic source filter";
    reader = read_summary_by_source;
    reader_relative = read_summary_by_source_relative_to;
    reader_many = read_summary_many_by_source;
    reader_many_relative = read_summary_many_by_source_relative_to;
    load = load_spectra_summary_by_source;
    load_relative = load_spectra_summary_by_source_relative_to;
    load_many = load_spectra_many_summary_by_source;
    load_many_relative = load_spectra_many_summary_by_source_relative_to;
    setter = only_source;
    value = filter: impl Into<LoadedSourceFilter>;
}

set_summary_helpers! {
    what = "generic source filters";
    reader = read_summary_by_sources;
    reader_relative = read_summary_by_sources_relative_to;
    reader_many = read_summary_many_by_sources;
    reader_many_relative = read_summary_many_by_sources_relative_to;
    load = load_spectra_summary_by_sources;
    load_relative = load_spectra_summary_by_sources_relative_to;
    load_many = load_spectra_many_summary_by_sources;
    load_many_relative = load_spectra_many_summary_by_sources_relative_to;
    setter = only_sources;
    values = filters;
    generics = [J, F];
    where = { J: IntoIterator<Item = F>, F: Into<LoadedSourceFilter>, };
}

single_summary_helpers! {
    what = "source format";
    reader = read_summary_by_source_format;
    reader_relative = read_summary_by_source_format_relative_to;
    reader_many = read_summary_many_by_source_format;
    reader_many_relative = read_summary_many_by_source_format_relative_to;
    load = load_spectra_summary_by_source_format;
    load_relative = load_spectra_summary_by_source_format_relative_to;
    load_many = load_spectra_many_summary_by_source_format;
    load_many_relative = load_spectra_many_summary_by_source_format_relative_to;
    setter = only_source_format;
    value = format: impl AsRef<str>;
}

set_summary_helpers! {
    what = "source formats";
    reader = read_summary_by_source_formats;
    reader_relative = read_summary_by_source_formats_relative_to;
    reader_many = read_summary_many_by_source_formats;
    reader_many_relative = read_summary_many_by_source_formats_relative_to;
    load = load_spectra_summary_by_source_formats;
    load_relative = load_spectra_summary_by_source_formats_relative_to;
    load_many = load_spectra_many_summary_by_source_formats;
    load_many_relative = load_spectra_many_summary_by_source_formats_relative_to;
    setter = only_source_formats;
    values = formats;
    generics = [J, F];
    where = { J: IntoIterator<Item = F>, F: AsRef<str>, };
}

single_summary_helpers! {
    what = "vendor family";
    reader = read_summary_by_source_vendor;
    reader_relative = read_summary_by_source_vendor_relative_to;
    reader_many = read_summary_many_by_source_vendor;
    reader_many_relative = read_summary_many_by_source_vendor_relative_to;
    load = load_spectra_summary_by_source_vendor;
    load_relative = load_spectra_summary_by_source_vendor_relative_to;
    load_many = load_spectra_many_summary_by_source_vendor;
    load_many_relative = load_spectra_many_summary_by_source_vendor_relative_to;
    setter = only_source_vendor;
    value = vendor: impl AsRef<str>;
}

set_summary_helpers! {
    what = "vendor families";
    reader = read_summary_by_source_vendors;
    reader_relative = read_summary_by_source_vendors_relative_to;
    reader_many = read_summary_many_by_source_vendors;
    reader_many_relative = read_summary_many_by_source_vendors_relative_to;
    load = load_spectra_summary_by_source_vendors;
    load_relative = load_spectra_summary_by_source_vendors_relative_to;
    load_many = load_spectra_many_summary_by_source_vendors;
    load_many_relative = load_spectra_many_summary_by_source_vendors_relative_to;
    setter = only_source_vendors;
    values = vendors;
    generics = [J, V];
    where = { J: IntoIterator<Item = V>, V: AsRef<str>, };
}

single_summary_helpers! {
    what = "raw/processed source data kind";
    reader = read_summary_by_source_data_kind;
    reader_relative = read_summary_by_source_data_kind_relative_to;
    reader_many = read_summary_many_by_source_data_kind;
    reader_many_relative = read_summary_many_by_source_data_kind_relative_to;
    load = load_spectra_summary_by_source_data_kind;
    load_relative = load_spectra_summary_by_source_data_kind_relative_to;
    load_many = load_spectra_many_summary_by_source_data_kind;
    load_many_relative = load_spectra_many_summary_by_source_data_kind_relative_to;
    setter = only_source_data_kind;
    value = data_kind: LoadedSourceDataKind;
}

set_summary_helpers! {
    what = "raw/processed source data kinds";
    reader = read_summary_by_source_data_kinds;
    reader_relative = read_summary_by_source_data_kinds_relative_to;
    reader_many = read_summary_many_by_source_data_kinds;
    reader_many_relative = read_summary_many_by_source_data_kinds_relative_to;
    load = load_spectra_summary_by_source_data_kinds;
    load_relative = load_spectra_summary_by_source_data_kinds_relative_to;
    load_many = load_spectra_many_summary_by_source_data_kinds;
    load_many_relative = load_spectra_many_summary_by_source_data_kinds_relative_to;
    setter = only_source_data_kinds;
    values = data_kinds;
    generics = [J];
    where = { J: IntoIterator<Item = LoadedSourceDataKind>, };
}

single_summary_helpers! {
    what = "tracked source path";
    reader = read_summary_by_source_path;
    reader_relative = read_summary_by_source_path_relative_to;
    reader_many = read_summary_many_by_source_path;
    reader_many_relative = read_summary_many_by_source_path_relative_to;
    load = load_spectra_summary_by_source_path;
    load_relative = load_spectra_summary_by_source_path_relative_to;
    load_many = load_spectra_many_summary_by_source_path;
    load_many_relative = load_spectra_many_summary_by_source_path_relative_to;
    setter = only_source_path;
    value = source_path: impl AsRef<Path>;
}

set_summary_helpers! {
    what = "tracked source paths";
    reader = read_summary_by_source_paths;
    reader_relative = read_summary_by_source_paths_relative_to;
    reader_many = read_summary_many_by_source_paths;
    reader_many_relative = read_summary_many_by_source_paths_relative_to;
    load = load_spectra_summary_by_source_paths;
    load_relative = load_spectra_summary_by_source_paths_relative_to;
    load_many = load_spectra_many_summary_by_source_paths;
    load_many_relative = load_spectra_many_summary_by_source_paths_relative_to;
    setter = only_source_paths;
    values = source_paths;
    generics = [J, F];
    where = { J: IntoIterator<Item = F>, F: AsRef<Path>, };
}

single_summary_helpers! {
    what = "tracked source path prefix";
    reader = read_summary_by_source_path_prefix;
    reader_relative = read_summary_by_source_path_prefix_relative_to;
    reader_many = read_summary_many_by_source_path_prefix;
    reader_many_relative = read_summary_many_by_source_path_prefix_relative_to;
    load = load_spectra_summary_by_source_path_prefix;
    load_relative = load_spectra_summary_by_source_path_prefix_relative_to;
    load_many = load_spectra_many_summary_by_source_path_prefix;
    load_many_relative = load_spectra_many_summary_by_source_path_prefix_relative_to;
    setter = only_source_path_prefix;
    value = source_path_prefix: impl AsRef<Path>;
}

set_summary_helpers! {
    what = "tracked source path prefixes";
    reader = read_summary_by_source_path_prefixes;
    reader_relative = read_summary_by_source_path_prefixes_relative_to;
    reader_many = read_summary_many_by_source_path_prefixes;
    reader_many_relative = read_summary_many_by_source_path_prefixes_relative_to;
    load = load_spectra_summary_by_source_path_prefixes;
    load_relative = load_spectra_summary_by_source_path_prefixes_relative_to;
    load_many = load_spectra_many_summary_by_source_path_prefixes;
    load_many_relative = load_spectra_many_summary_by_source_path_prefixes_relative_to;
    setter = only_source_path_prefixes;
    values = source_path_prefixes;
    generics = [J, F];
    where = { J: IntoIterator<Item = F>, F: AsRef<Path>, };
}
