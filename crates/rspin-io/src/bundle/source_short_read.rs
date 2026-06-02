//! Short source-metadata reader aliases for direct spectrum bundle loading.

use std::path::Path;

use rspin_core::Result;

use super::{LoadedSourceDataKind, SpectrumBundle, SpectrumBundleLoader, SpectrumBundleSummary};

macro_rules! single_bundle_aliases {
    (
        what = $what:literal;
        reader = $reader:ident;
        reader_relative = $reader_relative:ident;
        reader_many = $reader_many:ident;
        reader_many_relative = $reader_many_relative:ident;
        target = $target:ident;
        target_relative = $target_relative:ident;
        target_many = $target_many:ident;
        target_many_relative = $target_many_relative:ident;
        value = $value:ident : $value_ty:ty;
    ) => {
        impl SpectrumBundleLoader {
            #[doc = concat!("Loads a file or directory, restricted by one ", $what, ".")]
            ///
            /// This is a short alias for the matching `read_by_source_*` method.
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum data is found.
            pub fn $reader(
                &self,
                path: impl AsRef<Path>,
                $value: $value_ty,
            ) -> Result<SpectrumBundle> {
                self.$target(path, $value)
            }

            #[doc = concat!("Loads one selected path relative to a base directory, restricted by one ", $what, ".")]
            ///
            /// This is a short alias for the matching `read_by_source_*_relative_to` method.
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
                self.$target_relative(base, path, $value)
            }

            #[doc = concat!("Loads multiple paths, restricted by one ", $what, ".")]
            ///
            /// This is a short alias for the matching `read_many_by_source_*` method.
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum data is found.
            pub fn $reader_many<I, P>(&self, paths: I, $value: $value_ty) -> Result<SpectrumBundle>
            where
                I: IntoIterator<Item = P>,
                P: AsRef<Path>,
            {
                self.$target_many(paths, $value)
            }

            #[doc = concat!("Loads selected paths relative to a base directory, restricted by one ", $what, ".")]
            ///
            /// This is a short alias for the matching `read_many_by_source_*_relative_to` method.
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
                self.$target_many_relative(base, paths, $value)
            }
        }
    };
}

macro_rules! set_bundle_aliases {
    (
        what = $what:literal;
        reader = $reader:ident;
        reader_relative = $reader_relative:ident;
        reader_many = $reader_many:ident;
        reader_many_relative = $reader_many_relative:ident;
        target = $target:ident;
        target_relative = $target_relative:ident;
        target_many = $target_many:ident;
        target_many_relative = $target_many_relative:ident;
        values = $values:ident;
        generics = [$($generics:tt)*];
        where = {$($where_clause:tt)*};
    ) => {
        impl SpectrumBundleLoader {
            #[doc = concat!("Loads a file or directory, restricted by ", $what, ".")]
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
                self.$target(path, $values)
            }

            #[doc = concat!("Loads one selected path relative to a base directory, restricted by ", $what, ".")]
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
                self.$target_relative(base, path, $values)
            }

            #[doc = concat!("Loads multiple paths, restricted by ", $what, ".")]
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
                self.$target_many(paths, $values)
            }

            #[doc = concat!("Loads selected paths relative to a base directory, restricted by ", $what, ".")]
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
                self.$target_many_relative(base, paths, $values)
            }
        }
    };
}

macro_rules! single_summary_aliases {
    (
        what = $what:literal;
        reader = $reader:ident;
        reader_relative = $reader_relative:ident;
        reader_many = $reader_many:ident;
        reader_many_relative = $reader_many_relative:ident;
        target = $target:ident;
        target_relative = $target_relative:ident;
        target_many = $target_many:ident;
        target_many_relative = $target_many_relative:ident;
        value = $value:ident : $value_ty:ty;
    ) => {
        impl SpectrumBundleLoader {
            #[doc = concat!("Loads a file or directory and returns summary counts, restricted by one ", $what, ".")]
            ///
            /// This is a short alias for the matching `read_summary_by_source_*` method.
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum data is found.
            pub fn $reader(
                &self,
                path: impl AsRef<Path>,
                $value: $value_ty,
            ) -> Result<SpectrumBundleSummary> {
                self.$target(path, $value)
            }

            #[doc = concat!("Loads one selected path relative to a base directory and returns summary counts, restricted by one ", $what, ".")]
            ///
            /// This is a short alias for the matching `read_summary_by_source_*_relative_to` method.
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
                self.$target_relative(base, path, $value)
            }

            #[doc = concat!("Loads multiple paths and returns summary counts, restricted by one ", $what, ".")]
            ///
            /// This is a short alias for the matching `read_summary_many_by_source_*` method.
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
                self.$target_many(paths, $value)
            }

            #[doc = concat!("Loads selected paths relative to a base directory and returns summary counts, restricted by one ", $what, ".")]
            ///
            /// This is a short alias for the matching `read_summary_many_by_source_*_relative_to` method.
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
                self.$target_many_relative(base, paths, $value)
            }
        }
    };
}

macro_rules! set_summary_aliases {
    (
        what = $what:literal;
        reader = $reader:ident;
        reader_relative = $reader_relative:ident;
        reader_many = $reader_many:ident;
        reader_many_relative = $reader_many_relative:ident;
        target = $target:ident;
        target_relative = $target_relative:ident;
        target_many = $target_many:ident;
        target_many_relative = $target_many_relative:ident;
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
                self.$target(path, $values)
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
                self.$target_relative(base, path, $values)
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
                self.$target_many(paths, $values)
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
                self.$target_many_relative(base, paths, $values)
            }
        }
    };
}

single_bundle_aliases! {
    what = "source format";
    reader = read_by_format;
    reader_relative = read_by_format_relative_to;
    reader_many = read_many_by_format;
    reader_many_relative = read_many_by_format_relative_to;
    target = read_by_source_format;
    target_relative = read_by_source_format_relative_to;
    target_many = read_many_by_source_format;
    target_many_relative = read_many_by_source_format_relative_to;
    value = format: impl AsRef<str>;
}

set_bundle_aliases! {
    what = "source formats";
    reader = read_by_formats;
    reader_relative = read_by_formats_relative_to;
    reader_many = read_many_by_formats;
    reader_many_relative = read_many_by_formats_relative_to;
    target = read_by_source_formats;
    target_relative = read_by_source_formats_relative_to;
    target_many = read_many_by_source_formats;
    target_many_relative = read_many_by_source_formats_relative_to;
    values = formats;
    generics = [J, F];
    where = { J: IntoIterator<Item = F>, F: AsRef<str>, };
}

single_bundle_aliases! {
    what = "vendor family";
    reader = read_by_vendor;
    reader_relative = read_by_vendor_relative_to;
    reader_many = read_many_by_vendor;
    reader_many_relative = read_many_by_vendor_relative_to;
    target = read_by_source_vendor;
    target_relative = read_by_source_vendor_relative_to;
    target_many = read_many_by_source_vendor;
    target_many_relative = read_many_by_source_vendor_relative_to;
    value = vendor: impl AsRef<str>;
}

set_bundle_aliases! {
    what = "vendor families";
    reader = read_by_vendors;
    reader_relative = read_by_vendors_relative_to;
    reader_many = read_many_by_vendors;
    reader_many_relative = read_many_by_vendors_relative_to;
    target = read_by_source_vendors;
    target_relative = read_by_source_vendors_relative_to;
    target_many = read_many_by_source_vendors;
    target_many_relative = read_many_by_source_vendors_relative_to;
    values = vendors;
    generics = [J, V];
    where = { J: IntoIterator<Item = V>, V: AsRef<str>, };
}

single_bundle_aliases! {
    what = "raw/processed source data kind";
    reader = read_by_data_kind;
    reader_relative = read_by_data_kind_relative_to;
    reader_many = read_many_by_data_kind;
    reader_many_relative = read_many_by_data_kind_relative_to;
    target = read_by_source_data_kind;
    target_relative = read_by_source_data_kind_relative_to;
    target_many = read_many_by_source_data_kind;
    target_many_relative = read_many_by_source_data_kind_relative_to;
    value = data_kind: LoadedSourceDataKind;
}

set_bundle_aliases! {
    what = "raw/processed source data kinds";
    reader = read_by_data_kinds;
    reader_relative = read_by_data_kinds_relative_to;
    reader_many = read_many_by_data_kinds;
    reader_many_relative = read_many_by_data_kinds_relative_to;
    target = read_by_source_data_kinds;
    target_relative = read_by_source_data_kinds_relative_to;
    target_many = read_many_by_source_data_kinds;
    target_many_relative = read_many_by_source_data_kinds_relative_to;
    values = data_kinds;
    generics = [J];
    where = { J: IntoIterator<Item = LoadedSourceDataKind>, };
}

single_bundle_aliases! {
    what = "source format in strict mode";
    reader = read_strict_by_format;
    reader_relative = read_strict_by_format_relative_to;
    reader_many = read_many_strict_by_format;
    reader_many_relative = read_many_strict_by_format_relative_to;
    target = read_strict_by_source_format;
    target_relative = read_strict_by_source_format_relative_to;
    target_many = read_many_strict_by_source_format;
    target_many_relative = read_many_strict_by_source_format_relative_to;
    value = format: impl AsRef<str>;
}

set_bundle_aliases! {
    what = "source formats in strict mode";
    reader = read_strict_by_formats;
    reader_relative = read_strict_by_formats_relative_to;
    reader_many = read_many_strict_by_formats;
    reader_many_relative = read_many_strict_by_formats_relative_to;
    target = read_strict_by_source_formats;
    target_relative = read_strict_by_source_formats_relative_to;
    target_many = read_many_strict_by_source_formats;
    target_many_relative = read_many_strict_by_source_formats_relative_to;
    values = formats;
    generics = [J, F];
    where = { J: IntoIterator<Item = F>, F: AsRef<str>, };
}

single_bundle_aliases! {
    what = "vendor family in strict mode";
    reader = read_strict_by_vendor;
    reader_relative = read_strict_by_vendor_relative_to;
    reader_many = read_many_strict_by_vendor;
    reader_many_relative = read_many_strict_by_vendor_relative_to;
    target = read_strict_by_source_vendor;
    target_relative = read_strict_by_source_vendor_relative_to;
    target_many = read_many_strict_by_source_vendor;
    target_many_relative = read_many_strict_by_source_vendor_relative_to;
    value = vendor: impl AsRef<str>;
}

set_bundle_aliases! {
    what = "vendor families in strict mode";
    reader = read_strict_by_vendors;
    reader_relative = read_strict_by_vendors_relative_to;
    reader_many = read_many_strict_by_vendors;
    reader_many_relative = read_many_strict_by_vendors_relative_to;
    target = read_strict_by_source_vendors;
    target_relative = read_strict_by_source_vendors_relative_to;
    target_many = read_many_strict_by_source_vendors;
    target_many_relative = read_many_strict_by_source_vendors_relative_to;
    values = vendors;
    generics = [J, V];
    where = { J: IntoIterator<Item = V>, V: AsRef<str>, };
}

single_bundle_aliases! {
    what = "raw/processed source data kind in strict mode";
    reader = read_strict_by_data_kind;
    reader_relative = read_strict_by_data_kind_relative_to;
    reader_many = read_many_strict_by_data_kind;
    reader_many_relative = read_many_strict_by_data_kind_relative_to;
    target = read_strict_by_source_data_kind;
    target_relative = read_strict_by_source_data_kind_relative_to;
    target_many = read_many_strict_by_source_data_kind;
    target_many_relative = read_many_strict_by_source_data_kind_relative_to;
    value = data_kind: LoadedSourceDataKind;
}

set_bundle_aliases! {
    what = "raw/processed source data kinds in strict mode";
    reader = read_strict_by_data_kinds;
    reader_relative = read_strict_by_data_kinds_relative_to;
    reader_many = read_many_strict_by_data_kinds;
    reader_many_relative = read_many_strict_by_data_kinds_relative_to;
    target = read_strict_by_source_data_kinds;
    target_relative = read_strict_by_source_data_kinds_relative_to;
    target_many = read_many_strict_by_source_data_kinds;
    target_many_relative = read_many_strict_by_source_data_kinds_relative_to;
    values = data_kinds;
    generics = [J];
    where = { J: IntoIterator<Item = LoadedSourceDataKind>, };
}

single_summary_aliases! {
    what = "source format";
    reader = read_summary_by_format;
    reader_relative = read_summary_by_format_relative_to;
    reader_many = read_summary_many_by_format;
    reader_many_relative = read_summary_many_by_format_relative_to;
    target = read_summary_by_source_format;
    target_relative = read_summary_by_source_format_relative_to;
    target_many = read_summary_many_by_source_format;
    target_many_relative = read_summary_many_by_source_format_relative_to;
    value = format: impl AsRef<str>;
}

set_summary_aliases! {
    what = "source formats";
    reader = read_summary_by_formats;
    reader_relative = read_summary_by_formats_relative_to;
    reader_many = read_summary_many_by_formats;
    reader_many_relative = read_summary_many_by_formats_relative_to;
    target = read_summary_by_source_formats;
    target_relative = read_summary_by_source_formats_relative_to;
    target_many = read_summary_many_by_source_formats;
    target_many_relative = read_summary_many_by_source_formats_relative_to;
    values = formats;
    generics = [J, F];
    where = { J: IntoIterator<Item = F>, F: AsRef<str>, };
}

single_summary_aliases! {
    what = "vendor family";
    reader = read_summary_by_vendor;
    reader_relative = read_summary_by_vendor_relative_to;
    reader_many = read_summary_many_by_vendor;
    reader_many_relative = read_summary_many_by_vendor_relative_to;
    target = read_summary_by_source_vendor;
    target_relative = read_summary_by_source_vendor_relative_to;
    target_many = read_summary_many_by_source_vendor;
    target_many_relative = read_summary_many_by_source_vendor_relative_to;
    value = vendor: impl AsRef<str>;
}

set_summary_aliases! {
    what = "vendor families";
    reader = read_summary_by_vendors;
    reader_relative = read_summary_by_vendors_relative_to;
    reader_many = read_summary_many_by_vendors;
    reader_many_relative = read_summary_many_by_vendors_relative_to;
    target = read_summary_by_source_vendors;
    target_relative = read_summary_by_source_vendors_relative_to;
    target_many = read_summary_many_by_source_vendors;
    target_many_relative = read_summary_many_by_source_vendors_relative_to;
    values = vendors;
    generics = [J, V];
    where = { J: IntoIterator<Item = V>, V: AsRef<str>, };
}

single_summary_aliases! {
    what = "raw/processed source data kind";
    reader = read_summary_by_data_kind;
    reader_relative = read_summary_by_data_kind_relative_to;
    reader_many = read_summary_many_by_data_kind;
    reader_many_relative = read_summary_many_by_data_kind_relative_to;
    target = read_summary_by_source_data_kind;
    target_relative = read_summary_by_source_data_kind_relative_to;
    target_many = read_summary_many_by_source_data_kind;
    target_many_relative = read_summary_many_by_source_data_kind_relative_to;
    value = data_kind: LoadedSourceDataKind;
}

set_summary_aliases! {
    what = "raw/processed source data kinds";
    reader = read_summary_by_data_kinds;
    reader_relative = read_summary_by_data_kinds_relative_to;
    reader_many = read_summary_many_by_data_kinds;
    reader_many_relative = read_summary_many_by_data_kinds_relative_to;
    target = read_summary_by_source_data_kinds;
    target_relative = read_summary_by_source_data_kinds_relative_to;
    target_many = read_summary_many_by_source_data_kinds;
    target_many_relative = read_summary_many_by_source_data_kinds_relative_to;
    values = data_kinds;
    generics = [J];
    where = { J: IntoIterator<Item = LoadedSourceDataKind>, };
}

single_summary_aliases! {
    what = "source format in strict mode";
    reader = read_summary_strict_by_format;
    reader_relative = read_summary_strict_by_format_relative_to;
    reader_many = read_summary_many_strict_by_format;
    reader_many_relative = read_summary_many_strict_by_format_relative_to;
    target = read_summary_strict_by_source_format;
    target_relative = read_summary_strict_by_source_format_relative_to;
    target_many = read_summary_many_strict_by_source_format;
    target_many_relative = read_summary_many_strict_by_source_format_relative_to;
    value = format: impl AsRef<str>;
}

set_summary_aliases! {
    what = "source formats in strict mode";
    reader = read_summary_strict_by_formats;
    reader_relative = read_summary_strict_by_formats_relative_to;
    reader_many = read_summary_many_strict_by_formats;
    reader_many_relative = read_summary_many_strict_by_formats_relative_to;
    target = read_summary_strict_by_source_formats;
    target_relative = read_summary_strict_by_source_formats_relative_to;
    target_many = read_summary_many_strict_by_source_formats;
    target_many_relative = read_summary_many_strict_by_source_formats_relative_to;
    values = formats;
    generics = [J, F];
    where = { J: IntoIterator<Item = F>, F: AsRef<str>, };
}

single_summary_aliases! {
    what = "vendor family in strict mode";
    reader = read_summary_strict_by_vendor;
    reader_relative = read_summary_strict_by_vendor_relative_to;
    reader_many = read_summary_many_strict_by_vendor;
    reader_many_relative = read_summary_many_strict_by_vendor_relative_to;
    target = read_summary_strict_by_source_vendor;
    target_relative = read_summary_strict_by_source_vendor_relative_to;
    target_many = read_summary_many_strict_by_source_vendor;
    target_many_relative = read_summary_many_strict_by_source_vendor_relative_to;
    value = vendor: impl AsRef<str>;
}

set_summary_aliases! {
    what = "vendor families in strict mode";
    reader = read_summary_strict_by_vendors;
    reader_relative = read_summary_strict_by_vendors_relative_to;
    reader_many = read_summary_many_strict_by_vendors;
    reader_many_relative = read_summary_many_strict_by_vendors_relative_to;
    target = read_summary_strict_by_source_vendors;
    target_relative = read_summary_strict_by_source_vendors_relative_to;
    target_many = read_summary_many_strict_by_source_vendors;
    target_many_relative = read_summary_many_strict_by_source_vendors_relative_to;
    values = vendors;
    generics = [J, V];
    where = { J: IntoIterator<Item = V>, V: AsRef<str>, };
}

single_summary_aliases! {
    what = "raw/processed source data kind in strict mode";
    reader = read_summary_strict_by_data_kind;
    reader_relative = read_summary_strict_by_data_kind_relative_to;
    reader_many = read_summary_many_strict_by_data_kind;
    reader_many_relative = read_summary_many_strict_by_data_kind_relative_to;
    target = read_summary_strict_by_source_data_kind;
    target_relative = read_summary_strict_by_source_data_kind_relative_to;
    target_many = read_summary_many_strict_by_source_data_kind;
    target_many_relative = read_summary_many_strict_by_source_data_kind_relative_to;
    value = data_kind: LoadedSourceDataKind;
}

set_summary_aliases! {
    what = "raw/processed source data kinds in strict mode";
    reader = read_summary_strict_by_data_kinds;
    reader_relative = read_summary_strict_by_data_kinds_relative_to;
    reader_many = read_summary_many_strict_by_data_kinds;
    reader_many_relative = read_summary_many_strict_by_data_kinds_relative_to;
    target = read_summary_strict_by_source_data_kinds;
    target_relative = read_summary_strict_by_source_data_kinds_relative_to;
    target_many = read_summary_many_strict_by_source_data_kinds;
    target_many_relative = read_summary_many_strict_by_source_data_kinds_relative_to;
    values = data_kinds;
    generics = [J];
    where = { J: IntoIterator<Item = LoadedSourceDataKind>, };
}
