//! Strict source-filtered reader methods for direct spectrum bundle loading.

use std::path::Path;

use rspin_core::Result;

use super::{LoadedSourceDataKind, LoadedSourceFilter, SpectrumBundle, SpectrumBundleLoader};
use crate::bundle::SpectrumBundleSummary;

macro_rules! strict_single_read_methods {
    (
        what = $what:literal;
        reader = $reader:ident;
        reader_relative = $reader_relative:ident;
        reader_many = $reader_many:ident;
        reader_many_relative = $reader_many_relative:ident;
        setter = $setter:ident;
        value = $value:ident : $value_ty:ty;
    ) => {
        impl SpectrumBundleLoader {
            #[doc = concat!("Strictly loads a file or directory, restricted by one ", $what, ".")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum data is found.
            pub fn $reader(
                &self,
                path: impl AsRef<Path>,
                $value: $value_ty,
            ) -> Result<SpectrumBundle> {
                self.clone().strict().$setter($value).read_path(path)
            }

            #[doc = concat!("Strictly loads one selected path relative to a base directory, restricted by one ", $what, ".")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum data is found.
            pub fn $reader_relative(
                &self,
                base: impl AsRef<Path>,
                path: impl AsRef<Path>,
                $value: $value_ty,
            ) -> Result<SpectrumBundle> {
                self.clone()
                    .strict()
                    .$setter($value)
                    .read_path_relative_to(base, path)
            }

            #[doc = concat!("Strictly loads multiple paths, restricted by one ", $what, ".")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum data is found.
            pub fn $reader_many<I, P>(&self, paths: I, $value: $value_ty) -> Result<SpectrumBundle>
            where
                I: IntoIterator<Item = P>,
                P: AsRef<Path>,
            {
                self.clone().strict().$setter($value).read_paths(paths)
            }

            #[doc = concat!("Strictly loads selected paths relative to a base directory, restricted by one ", $what, ".")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum data is found.
            pub fn $reader_many_relative<I, P>(
                &self,
                base: impl AsRef<Path>,
                paths: I,
                $value: $value_ty,
            ) -> Result<SpectrumBundle>
            where
                I: IntoIterator<Item = P>,
                P: AsRef<Path>,
            {
                self.clone()
                    .strict()
                    .$setter($value)
                    .read_paths_relative_to(base, paths)
            }
        }
    };
}

macro_rules! strict_set_read_methods {
    (
        what = $what:literal;
        reader = $reader:ident;
        reader_relative = $reader_relative:ident;
        reader_many = $reader_many:ident;
        reader_many_relative = $reader_many_relative:ident;
        setter = $setter:ident;
        values = $values:ident;
        generics = [$($generics:tt)*];
        where = {$($where_clause:tt)*};
    ) => {
        impl SpectrumBundleLoader {
            #[doc = concat!("Strictly loads a file or directory, restricted by ", $what, ".")]
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
            ) -> Result<SpectrumBundle>
            where
                $($where_clause)*
            {
                self.clone().strict().$setter($values).read_path(path)
            }

            #[doc = concat!("Strictly loads one selected path relative to a base directory, restricted by ", $what, ".")]
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
            ) -> Result<SpectrumBundle>
            where
                $($where_clause)*
            {
                self.clone()
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
            pub fn $reader_many<I, P, $($generics)*>(
                &self,
                paths: I,
                $values: J,
            ) -> Result<SpectrumBundle>
            where
                I: IntoIterator<Item = P>,
                P: AsRef<Path>,
                $($where_clause)*
            {
                self.clone().strict().$setter($values).read_paths(paths)
            }

            #[doc = concat!("Strictly loads selected paths relative to a base directory, restricted by ", $what, ".")]
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
            ) -> Result<SpectrumBundle>
            where
                I: IntoIterator<Item = P>,
                P: AsRef<Path>,
                $($where_clause)*
            {
                self.clone()
                    .strict()
                    .$setter($values)
                    .read_paths_relative_to(base, paths)
            }
        }
    };
}

macro_rules! strict_single_summary_methods {
    (
        what = $what:literal;
        reader = $reader:ident;
        reader_relative = $reader_relative:ident;
        reader_many = $reader_many:ident;
        reader_many_relative = $reader_many_relative:ident;
        setter = $setter:ident;
        value = $value:ident : $value_ty:ty;
    ) => {
        impl SpectrumBundleLoader {
            #[doc = concat!("Strictly loads a file or directory and returns summary counts, restricted by one ", $what, ".")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum data is found.
            pub fn $reader(
                &self,
                path: impl AsRef<Path>,
                $value: $value_ty,
            ) -> Result<SpectrumBundleSummary> {
                self.clone()
                    .strict()
                    .$setter($value)
                    .read_summary_path(path)
            }

            #[doc = concat!("Strictly loads one selected path relative to a base directory and returns summary counts, restricted by one ", $what, ".")]
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
                    .strict()
                    .$setter($value)
                    .read_summary_path_relative_to(base, path)
            }

            #[doc = concat!("Strictly loads multiple paths and returns summary counts, restricted by one ", $what, ".")]
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
                self.clone()
                    .strict()
                    .$setter($value)
                    .read_summary_paths(paths)
            }

            #[doc = concat!("Strictly loads selected paths relative to a base directory and returns summary counts, restricted by one ", $what, ".")]
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
                    .strict()
                    .$setter($value)
                    .read_summary_paths_relative_to(base, paths)
            }
        }
    };
}

macro_rules! strict_set_summary_methods {
    (
        what = $what:literal;
        reader = $reader:ident;
        reader_relative = $reader_relative:ident;
        reader_many = $reader_many:ident;
        reader_many_relative = $reader_many_relative:ident;
        setter = $setter:ident;
        values = $values:ident;
        generics = [$($generics:tt)*];
        where = {$($where_clause:tt)*};
    ) => {
        impl SpectrumBundleLoader {
            #[doc = concat!("Strictly loads a file or directory and returns summary counts, restricted by ", $what, ".")]
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
                self.clone()
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
                self.clone()
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
                    .strict()
                    .$setter($values)
                    .read_summary_paths_relative_to(base, paths)
            }
        }
    };
}

strict_single_read_methods! {
    what = "generic source filter";
    reader = read_strict_by_source;
    reader_relative = read_strict_by_source_relative_to;
    reader_many = read_many_strict_by_source;
    reader_many_relative = read_many_strict_by_source_relative_to;
    setter = only_source;
    value = filter: impl Into<LoadedSourceFilter>;
}

strict_set_read_methods! {
    what = "generic source filters";
    reader = read_strict_by_sources;
    reader_relative = read_strict_by_sources_relative_to;
    reader_many = read_many_strict_by_sources;
    reader_many_relative = read_many_strict_by_sources_relative_to;
    setter = only_sources;
    values = filters;
    generics = [J, F];
    where = { J: IntoIterator<Item = F>, F: Into<LoadedSourceFilter>, };
}

strict_single_read_methods! {
    what = "source format";
    reader = read_strict_by_source_format;
    reader_relative = read_strict_by_source_format_relative_to;
    reader_many = read_many_strict_by_source_format;
    reader_many_relative = read_many_strict_by_source_format_relative_to;
    setter = only_source_format;
    value = format: impl AsRef<str>;
}

strict_set_read_methods! {
    what = "source formats";
    reader = read_strict_by_source_formats;
    reader_relative = read_strict_by_source_formats_relative_to;
    reader_many = read_many_strict_by_source_formats;
    reader_many_relative = read_many_strict_by_source_formats_relative_to;
    setter = only_source_formats;
    values = formats;
    generics = [J, F];
    where = { J: IntoIterator<Item = F>, F: AsRef<str>, };
}

strict_single_read_methods! {
    what = "vendor family";
    reader = read_strict_by_source_vendor;
    reader_relative = read_strict_by_source_vendor_relative_to;
    reader_many = read_many_strict_by_source_vendor;
    reader_many_relative = read_many_strict_by_source_vendor_relative_to;
    setter = only_source_vendor;
    value = vendor: impl AsRef<str>;
}

strict_set_read_methods! {
    what = "vendor families";
    reader = read_strict_by_source_vendors;
    reader_relative = read_strict_by_source_vendors_relative_to;
    reader_many = read_many_strict_by_source_vendors;
    reader_many_relative = read_many_strict_by_source_vendors_relative_to;
    setter = only_source_vendors;
    values = vendors;
    generics = [J, V];
    where = { J: IntoIterator<Item = V>, V: AsRef<str>, };
}

strict_single_read_methods! {
    what = "raw/processed source data kind";
    reader = read_strict_by_source_data_kind;
    reader_relative = read_strict_by_source_data_kind_relative_to;
    reader_many = read_many_strict_by_source_data_kind;
    reader_many_relative = read_many_strict_by_source_data_kind_relative_to;
    setter = only_source_data_kind;
    value = data_kind: LoadedSourceDataKind;
}

strict_set_read_methods! {
    what = "raw/processed source data kinds";
    reader = read_strict_by_source_data_kinds;
    reader_relative = read_strict_by_source_data_kinds_relative_to;
    reader_many = read_many_strict_by_source_data_kinds;
    reader_many_relative = read_many_strict_by_source_data_kinds_relative_to;
    setter = only_source_data_kinds;
    values = data_kinds;
    generics = [J];
    where = { J: IntoIterator<Item = LoadedSourceDataKind>, };
}

strict_single_read_methods! {
    what = "tracked source path";
    reader = read_strict_by_source_path;
    reader_relative = read_strict_by_source_path_relative_to;
    reader_many = read_many_strict_by_source_path;
    reader_many_relative = read_many_strict_by_source_path_relative_to;
    setter = only_source_path;
    value = source_path: impl AsRef<Path>;
}

strict_set_read_methods! {
    what = "tracked source paths";
    reader = read_strict_by_source_paths;
    reader_relative = read_strict_by_source_paths_relative_to;
    reader_many = read_many_strict_by_source_paths;
    reader_many_relative = read_many_strict_by_source_paths_relative_to;
    setter = only_source_paths;
    values = source_paths;
    generics = [J, F];
    where = { J: IntoIterator<Item = F>, F: AsRef<Path>, };
}

strict_single_read_methods! {
    what = "tracked source path prefix";
    reader = read_strict_by_source_path_prefix;
    reader_relative = read_strict_by_source_path_prefix_relative_to;
    reader_many = read_many_strict_by_source_path_prefix;
    reader_many_relative = read_many_strict_by_source_path_prefix_relative_to;
    setter = only_source_path_prefix;
    value = source_path_prefix: impl AsRef<Path>;
}

strict_set_read_methods! {
    what = "tracked source path prefixes";
    reader = read_strict_by_source_path_prefixes;
    reader_relative = read_strict_by_source_path_prefixes_relative_to;
    reader_many = read_many_strict_by_source_path_prefixes;
    reader_many_relative = read_many_strict_by_source_path_prefixes_relative_to;
    setter = only_source_path_prefixes;
    values = source_path_prefixes;
    generics = [J, F];
    where = { J: IntoIterator<Item = F>, F: AsRef<Path>, };
}

strict_single_summary_methods! {
    what = "generic source filter";
    reader = read_summary_strict_by_source;
    reader_relative = read_summary_strict_by_source_relative_to;
    reader_many = read_summary_many_strict_by_source;
    reader_many_relative = read_summary_many_strict_by_source_relative_to;
    setter = only_source;
    value = filter: impl Into<LoadedSourceFilter>;
}

strict_set_summary_methods! {
    what = "generic source filters";
    reader = read_summary_strict_by_sources;
    reader_relative = read_summary_strict_by_sources_relative_to;
    reader_many = read_summary_many_strict_by_sources;
    reader_many_relative = read_summary_many_strict_by_sources_relative_to;
    setter = only_sources;
    values = filters;
    generics = [J, F];
    where = { J: IntoIterator<Item = F>, F: Into<LoadedSourceFilter>, };
}

strict_single_summary_methods! {
    what = "source format";
    reader = read_summary_strict_by_source_format;
    reader_relative = read_summary_strict_by_source_format_relative_to;
    reader_many = read_summary_many_strict_by_source_format;
    reader_many_relative = read_summary_many_strict_by_source_format_relative_to;
    setter = only_source_format;
    value = format: impl AsRef<str>;
}

strict_set_summary_methods! {
    what = "source formats";
    reader = read_summary_strict_by_source_formats;
    reader_relative = read_summary_strict_by_source_formats_relative_to;
    reader_many = read_summary_many_strict_by_source_formats;
    reader_many_relative = read_summary_many_strict_by_source_formats_relative_to;
    setter = only_source_formats;
    values = formats;
    generics = [J, F];
    where = { J: IntoIterator<Item = F>, F: AsRef<str>, };
}

strict_single_summary_methods! {
    what = "vendor family";
    reader = read_summary_strict_by_source_vendor;
    reader_relative = read_summary_strict_by_source_vendor_relative_to;
    reader_many = read_summary_many_strict_by_source_vendor;
    reader_many_relative = read_summary_many_strict_by_source_vendor_relative_to;
    setter = only_source_vendor;
    value = vendor: impl AsRef<str>;
}

strict_set_summary_methods! {
    what = "vendor families";
    reader = read_summary_strict_by_source_vendors;
    reader_relative = read_summary_strict_by_source_vendors_relative_to;
    reader_many = read_summary_many_strict_by_source_vendors;
    reader_many_relative = read_summary_many_strict_by_source_vendors_relative_to;
    setter = only_source_vendors;
    values = vendors;
    generics = [J, V];
    where = { J: IntoIterator<Item = V>, V: AsRef<str>, };
}

strict_single_summary_methods! {
    what = "raw/processed source data kind";
    reader = read_summary_strict_by_source_data_kind;
    reader_relative = read_summary_strict_by_source_data_kind_relative_to;
    reader_many = read_summary_many_strict_by_source_data_kind;
    reader_many_relative = read_summary_many_strict_by_source_data_kind_relative_to;
    setter = only_source_data_kind;
    value = data_kind: LoadedSourceDataKind;
}

strict_set_summary_methods! {
    what = "raw/processed source data kinds";
    reader = read_summary_strict_by_source_data_kinds;
    reader_relative = read_summary_strict_by_source_data_kinds_relative_to;
    reader_many = read_summary_many_strict_by_source_data_kinds;
    reader_many_relative = read_summary_many_strict_by_source_data_kinds_relative_to;
    setter = only_source_data_kinds;
    values = data_kinds;
    generics = [J];
    where = { J: IntoIterator<Item = LoadedSourceDataKind>, };
}

strict_single_summary_methods! {
    what = "tracked source path";
    reader = read_summary_strict_by_source_path;
    reader_relative = read_summary_strict_by_source_path_relative_to;
    reader_many = read_summary_many_strict_by_source_path;
    reader_many_relative = read_summary_many_strict_by_source_path_relative_to;
    setter = only_source_path;
    value = source_path: impl AsRef<Path>;
}

strict_set_summary_methods! {
    what = "tracked source paths";
    reader = read_summary_strict_by_source_paths;
    reader_relative = read_summary_strict_by_source_paths_relative_to;
    reader_many = read_summary_many_strict_by_source_paths;
    reader_many_relative = read_summary_many_strict_by_source_paths_relative_to;
    setter = only_source_paths;
    values = source_paths;
    generics = [J, F];
    where = { J: IntoIterator<Item = F>, F: AsRef<Path>, };
}

strict_single_summary_methods! {
    what = "tracked source path prefix";
    reader = read_summary_strict_by_source_path_prefix;
    reader_relative = read_summary_strict_by_source_path_prefix_relative_to;
    reader_many = read_summary_many_strict_by_source_path_prefix;
    reader_many_relative = read_summary_many_strict_by_source_path_prefix_relative_to;
    setter = only_source_path_prefix;
    value = source_path_prefix: impl AsRef<Path>;
}

strict_set_summary_methods! {
    what = "tracked source path prefixes";
    reader = read_summary_strict_by_source_path_prefixes;
    reader_relative = read_summary_strict_by_source_path_prefixes_relative_to;
    reader_many = read_summary_many_strict_by_source_path_prefixes;
    reader_many_relative = read_summary_many_strict_by_source_path_prefixes_relative_to;
    setter = only_source_path_prefixes;
    values = source_path_prefixes;
    generics = [J, F];
    where = { J: IntoIterator<Item = F>, F: AsRef<Path>, };
}
