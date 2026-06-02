//! Strict source-filtered helpers for direct spectrum bundle loading.

use std::path::Path;

use rspin_core::Result;

use super::{
    LoadedSourceDataKind, LoadedSourceFilter, SpectrumBundle, SpectrumBundleLoader,
    SpectrumBundleSummary,
};

macro_rules! strict_single_read_helpers {
    (
        what = $what:literal;
        load = $load:ident;
        load_relative = $load_relative:ident;
        load_many = $load_many:ident;
        load_many_relative = $load_many_relative:ident;
        setter = $setter:ident;
        value = $value:ident : $value_ty:ty;
    ) => {
        #[doc = concat!("Strictly loads a file or directory, restricted by one ", $what, ".")]
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $load(path: impl AsRef<Path>, $value: $value_ty) -> Result<SpectrumBundle> {
            SpectrumBundleLoader::new()
                .strict()
                .$setter($value)
                .read_path(path)
        }

        #[doc = concat!("Strictly loads one selected path relative to a base directory, restricted by one ", $what, ".")]
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $load_relative(
            base: impl AsRef<Path>,
            path: impl AsRef<Path>,
            $value: $value_ty,
        ) -> Result<SpectrumBundle> {
            SpectrumBundleLoader::new()
                .strict()
                .$setter($value)
                .read_path_relative_to(base, path)
        }

        #[doc = concat!("Strictly loads multiple paths, restricted by one ", $what, ".")]
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $load_many<I, P>(paths: I, $value: $value_ty) -> Result<SpectrumBundle>
        where
            I: IntoIterator<Item = P>,
            P: AsRef<Path>,
        {
            SpectrumBundleLoader::new()
                .strict()
                .$setter($value)
                .read_paths(paths)
        }

        #[doc = concat!("Strictly loads selected paths relative to a base directory, restricted by one ", $what, ".")]
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $load_many_relative<I, P>(
            base: impl AsRef<Path>,
            paths: I,
            $value: $value_ty,
        ) -> Result<SpectrumBundle>
        where
            I: IntoIterator<Item = P>,
            P: AsRef<Path>,
        {
            SpectrumBundleLoader::new()
                .strict()
                .$setter($value)
                .read_paths_relative_to(base, paths)
        }
    };
}

macro_rules! strict_set_read_helpers {
    (
        what = $what:literal;
        load = $load:ident;
        load_relative = $load_relative:ident;
        load_many = $load_many:ident;
        load_many_relative = $load_many_relative:ident;
        setter = $setter:ident;
        values = $values:ident;
        generics = [$($generics:tt)*];
        where = {$($where_clause:tt)*};
    ) => {
        #[doc = concat!("Strictly loads a file or directory, restricted by ", $what, ".")]
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
        ) -> Result<SpectrumBundle>
        where
            $($where_clause)*
        {
            SpectrumBundleLoader::new()
                .strict()
                .$setter($values)
                .read_path(path)
        }

        #[doc = concat!("Strictly loads one selected path relative to a base directory, restricted by ", $what, ".")]
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
        ) -> Result<SpectrumBundle>
        where
            $($where_clause)*
        {
            SpectrumBundleLoader::new()
                .strict()
                .$setter($values)
                .read_path_relative_to(base, path)
        }

        #[doc = concat!("Strictly loads multiple paths, restricted by ", $what, ".")]
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
        ) -> Result<SpectrumBundle>
        where
            I: IntoIterator<Item = P>,
            P: AsRef<Path>,
            $($where_clause)*
        {
            SpectrumBundleLoader::new()
                .strict()
                .$setter($values)
                .read_paths(paths)
        }

        #[doc = concat!("Strictly loads selected paths relative to a base directory, restricted by ", $what, ".")]
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
        ) -> Result<SpectrumBundle>
        where
            I: IntoIterator<Item = P>,
            P: AsRef<Path>,
            $($where_clause)*
        {
            SpectrumBundleLoader::new()
                .strict()
                .$setter($values)
                .read_paths_relative_to(base, paths)
        }
    };
}

macro_rules! strict_single_summary_helpers {
    (
        what = $what:literal;
        load = $load:ident;
        load_relative = $load_relative:ident;
        load_many = $load_many:ident;
        load_many_relative = $load_many_relative:ident;
        setter = $setter:ident;
        value = $value:ident : $value_ty:ty;
    ) => {
        #[doc = concat!("Strictly loads a file or directory and returns summary counts, restricted by one ", $what, ".")]
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $load(
            path: impl AsRef<Path>,
            $value: $value_ty,
        ) -> Result<SpectrumBundleSummary> {
            SpectrumBundleLoader::new()
                .strict()
                .$setter($value)
                .read_summary_path(path)
        }

        #[doc = concat!("Strictly loads one selected path relative to a base directory and returns summary counts, restricted by one ", $what, ".")]
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $load_relative(
            base: impl AsRef<Path>,
            path: impl AsRef<Path>,
            $value: $value_ty,
        ) -> Result<SpectrumBundleSummary> {
            SpectrumBundleLoader::new()
                .strict()
                .$setter($value)
                .read_summary_path_relative_to(base, path)
        }

        #[doc = concat!("Strictly loads multiple paths and returns summary counts, restricted by one ", $what, ".")]
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $load_many<I, P>(
            paths: I,
            $value: $value_ty,
        ) -> Result<SpectrumBundleSummary>
        where
            I: IntoIterator<Item = P>,
            P: AsRef<Path>,
        {
            SpectrumBundleLoader::new()
                .strict()
                .$setter($value)
                .read_summary_paths(paths)
        }

        #[doc = concat!("Strictly loads selected paths relative to a base directory and returns summary counts, restricted by one ", $what, ".")]
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
            SpectrumBundleLoader::new()
                .strict()
                .$setter($value)
                .read_summary_paths_relative_to(base, paths)
        }
    };
}

macro_rules! strict_set_summary_helpers {
    (
        what = $what:literal;
        load = $load:ident;
        load_relative = $load_relative:ident;
        load_many = $load_many:ident;
        load_many_relative = $load_many_relative:ident;
        setter = $setter:ident;
        values = $values:ident;
        generics = [$($generics:tt)*];
        where = {$($where_clause:tt)*};
    ) => {
        #[doc = concat!("Strictly loads a file or directory and returns summary counts, restricted by ", $what, ".")]
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
            SpectrumBundleLoader::new()
                .strict()
                .$setter($values)
                .read_summary_path(path)
        }

        #[doc = concat!("Strictly loads one selected path relative to a base directory and returns summary counts, restricted by ", $what, ".")]
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
            SpectrumBundleLoader::new()
                .strict()
                .$setter($values)
                .read_summary_path_relative_to(base, path)
        }

        #[doc = concat!("Strictly loads multiple paths and returns summary counts, restricted by ", $what, ".")]
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
            SpectrumBundleLoader::new()
                .strict()
                .$setter($values)
                .read_summary_paths(paths)
        }

        #[doc = concat!("Strictly loads selected paths relative to a base directory and returns summary counts, restricted by ", $what, ".")]
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
            SpectrumBundleLoader::new()
                .strict()
                .$setter($values)
                .read_summary_paths_relative_to(base, paths)
        }
    };
}

strict_single_read_helpers! {
    what = "generic source filter";
    load = load_spectra_strict_by_source;
    load_relative = load_spectra_strict_by_source_relative_to;
    load_many = load_spectra_many_strict_by_source;
    load_many_relative = load_spectra_many_strict_by_source_relative_to;
    setter = only_source;
    value = filter: impl Into<LoadedSourceFilter>;
}

strict_set_read_helpers! {
    what = "generic source filters";
    load = load_spectra_strict_by_sources;
    load_relative = load_spectra_strict_by_sources_relative_to;
    load_many = load_spectra_many_strict_by_sources;
    load_many_relative = load_spectra_many_strict_by_sources_relative_to;
    setter = only_sources;
    values = filters;
    generics = [J, F];
    where = { J: IntoIterator<Item = F>, F: Into<LoadedSourceFilter>, };
}

strict_single_read_helpers! {
    what = "source format";
    load = load_spectra_strict_by_source_format;
    load_relative = load_spectra_strict_by_source_format_relative_to;
    load_many = load_spectra_many_strict_by_source_format;
    load_many_relative = load_spectra_many_strict_by_source_format_relative_to;
    setter = only_source_format;
    value = format: impl AsRef<str>;
}

strict_set_read_helpers! {
    what = "source formats";
    load = load_spectra_strict_by_source_formats;
    load_relative = load_spectra_strict_by_source_formats_relative_to;
    load_many = load_spectra_many_strict_by_source_formats;
    load_many_relative = load_spectra_many_strict_by_source_formats_relative_to;
    setter = only_source_formats;
    values = formats;
    generics = [J, F];
    where = { J: IntoIterator<Item = F>, F: AsRef<str>, };
}

strict_single_read_helpers! {
    what = "vendor family";
    load = load_spectra_strict_by_source_vendor;
    load_relative = load_spectra_strict_by_source_vendor_relative_to;
    load_many = load_spectra_many_strict_by_source_vendor;
    load_many_relative = load_spectra_many_strict_by_source_vendor_relative_to;
    setter = only_source_vendor;
    value = vendor: impl AsRef<str>;
}

strict_set_read_helpers! {
    what = "vendor families";
    load = load_spectra_strict_by_source_vendors;
    load_relative = load_spectra_strict_by_source_vendors_relative_to;
    load_many = load_spectra_many_strict_by_source_vendors;
    load_many_relative = load_spectra_many_strict_by_source_vendors_relative_to;
    setter = only_source_vendors;
    values = vendors;
    generics = [J, V];
    where = { J: IntoIterator<Item = V>, V: AsRef<str>, };
}

strict_single_read_helpers! {
    what = "raw/processed source data kind";
    load = load_spectra_strict_by_source_data_kind;
    load_relative = load_spectra_strict_by_source_data_kind_relative_to;
    load_many = load_spectra_many_strict_by_source_data_kind;
    load_many_relative = load_spectra_many_strict_by_source_data_kind_relative_to;
    setter = only_source_data_kind;
    value = data_kind: LoadedSourceDataKind;
}

strict_set_read_helpers! {
    what = "raw/processed source data kinds";
    load = load_spectra_strict_by_source_data_kinds;
    load_relative = load_spectra_strict_by_source_data_kinds_relative_to;
    load_many = load_spectra_many_strict_by_source_data_kinds;
    load_many_relative = load_spectra_many_strict_by_source_data_kinds_relative_to;
    setter = only_source_data_kinds;
    values = data_kinds;
    generics = [J];
    where = { J: IntoIterator<Item = LoadedSourceDataKind>, };
}

strict_single_read_helpers! {
    what = "tracked source path";
    load = load_spectra_strict_by_source_path;
    load_relative = load_spectra_strict_by_source_path_relative_to;
    load_many = load_spectra_many_strict_by_source_path;
    load_many_relative = load_spectra_many_strict_by_source_path_relative_to;
    setter = only_source_path;
    value = source_path: impl AsRef<Path>;
}

strict_set_read_helpers! {
    what = "tracked source paths";
    load = load_spectra_strict_by_source_paths;
    load_relative = load_spectra_strict_by_source_paths_relative_to;
    load_many = load_spectra_many_strict_by_source_paths;
    load_many_relative = load_spectra_many_strict_by_source_paths_relative_to;
    setter = only_source_paths;
    values = source_paths;
    generics = [J, F];
    where = { J: IntoIterator<Item = F>, F: AsRef<Path>, };
}

strict_single_read_helpers! {
    what = "tracked source path prefix";
    load = load_spectra_strict_by_source_path_prefix;
    load_relative = load_spectra_strict_by_source_path_prefix_relative_to;
    load_many = load_spectra_many_strict_by_source_path_prefix;
    load_many_relative = load_spectra_many_strict_by_source_path_prefix_relative_to;
    setter = only_source_path_prefix;
    value = source_path_prefix: impl AsRef<Path>;
}

strict_set_read_helpers! {
    what = "tracked source path prefixes";
    load = load_spectra_strict_by_source_path_prefixes;
    load_relative = load_spectra_strict_by_source_path_prefixes_relative_to;
    load_many = load_spectra_many_strict_by_source_path_prefixes;
    load_many_relative = load_spectra_many_strict_by_source_path_prefixes_relative_to;
    setter = only_source_path_prefixes;
    values = source_path_prefixes;
    generics = [J, F];
    where = { J: IntoIterator<Item = F>, F: AsRef<Path>, };
}

strict_single_summary_helpers! {
    what = "generic source filter";
    load = load_spectra_summary_strict_by_source;
    load_relative = load_spectra_summary_strict_by_source_relative_to;
    load_many = load_spectra_many_summary_strict_by_source;
    load_many_relative = load_spectra_many_summary_strict_by_source_relative_to;
    setter = only_source;
    value = filter: impl Into<LoadedSourceFilter>;
}

strict_set_summary_helpers! {
    what = "generic source filters";
    load = load_spectra_summary_strict_by_sources;
    load_relative = load_spectra_summary_strict_by_sources_relative_to;
    load_many = load_spectra_many_summary_strict_by_sources;
    load_many_relative = load_spectra_many_summary_strict_by_sources_relative_to;
    setter = only_sources;
    values = filters;
    generics = [J, F];
    where = { J: IntoIterator<Item = F>, F: Into<LoadedSourceFilter>, };
}

strict_single_summary_helpers! {
    what = "source format";
    load = load_spectra_summary_strict_by_source_format;
    load_relative = load_spectra_summary_strict_by_source_format_relative_to;
    load_many = load_spectra_many_summary_strict_by_source_format;
    load_many_relative = load_spectra_many_summary_strict_by_source_format_relative_to;
    setter = only_source_format;
    value = format: impl AsRef<str>;
}

strict_set_summary_helpers! {
    what = "source formats";
    load = load_spectra_summary_strict_by_source_formats;
    load_relative = load_spectra_summary_strict_by_source_formats_relative_to;
    load_many = load_spectra_many_summary_strict_by_source_formats;
    load_many_relative = load_spectra_many_summary_strict_by_source_formats_relative_to;
    setter = only_source_formats;
    values = formats;
    generics = [J, F];
    where = { J: IntoIterator<Item = F>, F: AsRef<str>, };
}

strict_single_summary_helpers! {
    what = "vendor family";
    load = load_spectra_summary_strict_by_source_vendor;
    load_relative = load_spectra_summary_strict_by_source_vendor_relative_to;
    load_many = load_spectra_many_summary_strict_by_source_vendor;
    load_many_relative = load_spectra_many_summary_strict_by_source_vendor_relative_to;
    setter = only_source_vendor;
    value = vendor: impl AsRef<str>;
}

strict_set_summary_helpers! {
    what = "vendor families";
    load = load_spectra_summary_strict_by_source_vendors;
    load_relative = load_spectra_summary_strict_by_source_vendors_relative_to;
    load_many = load_spectra_many_summary_strict_by_source_vendors;
    load_many_relative = load_spectra_many_summary_strict_by_source_vendors_relative_to;
    setter = only_source_vendors;
    values = vendors;
    generics = [J, V];
    where = { J: IntoIterator<Item = V>, V: AsRef<str>, };
}

strict_single_summary_helpers! {
    what = "raw/processed source data kind";
    load = load_spectra_summary_strict_by_source_data_kind;
    load_relative = load_spectra_summary_strict_by_source_data_kind_relative_to;
    load_many = load_spectra_many_summary_strict_by_source_data_kind;
    load_many_relative = load_spectra_many_summary_strict_by_source_data_kind_relative_to;
    setter = only_source_data_kind;
    value = data_kind: LoadedSourceDataKind;
}

strict_set_summary_helpers! {
    what = "raw/processed source data kinds";
    load = load_spectra_summary_strict_by_source_data_kinds;
    load_relative = load_spectra_summary_strict_by_source_data_kinds_relative_to;
    load_many = load_spectra_many_summary_strict_by_source_data_kinds;
    load_many_relative = load_spectra_many_summary_strict_by_source_data_kinds_relative_to;
    setter = only_source_data_kinds;
    values = data_kinds;
    generics = [J];
    where = { J: IntoIterator<Item = LoadedSourceDataKind>, };
}

strict_single_summary_helpers! {
    what = "tracked source path";
    load = load_spectra_summary_strict_by_source_path;
    load_relative = load_spectra_summary_strict_by_source_path_relative_to;
    load_many = load_spectra_many_summary_strict_by_source_path;
    load_many_relative = load_spectra_many_summary_strict_by_source_path_relative_to;
    setter = only_source_path;
    value = source_path: impl AsRef<Path>;
}

strict_set_summary_helpers! {
    what = "tracked source paths";
    load = load_spectra_summary_strict_by_source_paths;
    load_relative = load_spectra_summary_strict_by_source_paths_relative_to;
    load_many = load_spectra_many_summary_strict_by_source_paths;
    load_many_relative = load_spectra_many_summary_strict_by_source_paths_relative_to;
    setter = only_source_paths;
    values = source_paths;
    generics = [J, F];
    where = { J: IntoIterator<Item = F>, F: AsRef<Path>, };
}

strict_single_summary_helpers! {
    what = "tracked source path prefix";
    load = load_spectra_summary_strict_by_source_path_prefix;
    load_relative = load_spectra_summary_strict_by_source_path_prefix_relative_to;
    load_many = load_spectra_many_summary_strict_by_source_path_prefix;
    load_many_relative = load_spectra_many_summary_strict_by_source_path_prefix_relative_to;
    setter = only_source_path_prefix;
    value = source_path_prefix: impl AsRef<Path>;
}

strict_set_summary_helpers! {
    what = "tracked source path prefixes";
    load = load_spectra_summary_strict_by_source_path_prefixes;
    load_relative = load_spectra_summary_strict_by_source_path_prefixes_relative_to;
    load_many = load_spectra_many_summary_strict_by_source_path_prefixes;
    load_many_relative = load_spectra_many_summary_strict_by_source_path_prefixes_relative_to;
    setter = only_source_path_prefixes;
    values = source_path_prefixes;
    generics = [J, F];
    where = { J: IntoIterator<Item = F>, F: AsRef<Path>, };
}
