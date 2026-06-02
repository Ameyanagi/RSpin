//! Source-filtered reader methods for direct spectrum bundle loading.

use std::path::Path;

use rspin_core::Result;

use super::{LoadedSourceDataKind, LoadedSourceFilter, SpectrumBundle, SpectrumBundleLoader};

macro_rules! single_read_methods {
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
            #[doc = concat!("Loads a file or directory, restricted by one ", $what, ".")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum data is found.
            pub fn $reader(
                &self,
                path: impl AsRef<Path>,
                $value: $value_ty,
            ) -> Result<SpectrumBundle> {
                self.clone().$setter($value).read_path(path)
            }

            #[doc = concat!("Loads one selected path relative to a base directory, restricted by one ", $what, ".")]
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
                    .$setter($value)
                    .read_path_relative_to(base, path)
            }

            #[doc = concat!("Loads multiple paths, restricted by one ", $what, ".")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum data is found.
            pub fn $reader_many<I, P>(&self, paths: I, $value: $value_ty) -> Result<SpectrumBundle>
            where
                I: IntoIterator<Item = P>,
                P: AsRef<Path>,
            {
                self.clone().$setter($value).read_paths(paths)
            }

            #[doc = concat!("Loads selected paths relative to a base directory, restricted by one ", $what, ".")]
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
                    .$setter($value)
                    .read_paths_relative_to(base, paths)
            }
        }
    };
}

macro_rules! set_read_methods {
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
                self.clone().$setter($values).read_path(path)
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
                self.clone()
                    .$setter($values)
                    .read_path_relative_to(base, path)
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
                self.clone().$setter($values).read_paths(paths)
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
                self.clone()
                    .$setter($values)
                    .read_paths_relative_to(base, paths)
            }
        }
    };
}

single_read_methods! {
    what = "generic source filter";
    reader = read_by_source;
    reader_relative = read_by_source_relative_to;
    reader_many = read_many_by_source;
    reader_many_relative = read_many_by_source_relative_to;
    setter = only_source;
    value = filter: impl Into<LoadedSourceFilter>;
}

set_read_methods! {
    what = "generic source filters";
    reader = read_by_sources;
    reader_relative = read_by_sources_relative_to;
    reader_many = read_many_by_sources;
    reader_many_relative = read_many_by_sources_relative_to;
    setter = only_sources;
    values = filters;
    generics = [J, F];
    where = { J: IntoIterator<Item = F>, F: Into<LoadedSourceFilter>, };
}

single_read_methods! {
    what = "source format";
    reader = read_by_source_format;
    reader_relative = read_by_source_format_relative_to;
    reader_many = read_many_by_source_format;
    reader_many_relative = read_many_by_source_format_relative_to;
    setter = only_source_format;
    value = format: impl AsRef<str>;
}

set_read_methods! {
    what = "source formats";
    reader = read_by_source_formats;
    reader_relative = read_by_source_formats_relative_to;
    reader_many = read_many_by_source_formats;
    reader_many_relative = read_many_by_source_formats_relative_to;
    setter = only_source_formats;
    values = formats;
    generics = [J, F];
    where = { J: IntoIterator<Item = F>, F: AsRef<str>, };
}

single_read_methods! {
    what = "vendor family";
    reader = read_by_source_vendor;
    reader_relative = read_by_source_vendor_relative_to;
    reader_many = read_many_by_source_vendor;
    reader_many_relative = read_many_by_source_vendor_relative_to;
    setter = only_source_vendor;
    value = vendor: impl AsRef<str>;
}

set_read_methods! {
    what = "vendor families";
    reader = read_by_source_vendors;
    reader_relative = read_by_source_vendors_relative_to;
    reader_many = read_many_by_source_vendors;
    reader_many_relative = read_many_by_source_vendors_relative_to;
    setter = only_source_vendors;
    values = vendors;
    generics = [J, V];
    where = { J: IntoIterator<Item = V>, V: AsRef<str>, };
}

single_read_methods! {
    what = "raw/processed source data kind";
    reader = read_by_source_data_kind;
    reader_relative = read_by_source_data_kind_relative_to;
    reader_many = read_many_by_source_data_kind;
    reader_many_relative = read_many_by_source_data_kind_relative_to;
    setter = only_source_data_kind;
    value = data_kind: LoadedSourceDataKind;
}

set_read_methods! {
    what = "raw/processed source data kinds";
    reader = read_by_source_data_kinds;
    reader_relative = read_by_source_data_kinds_relative_to;
    reader_many = read_many_by_source_data_kinds;
    reader_many_relative = read_many_by_source_data_kinds_relative_to;
    setter = only_source_data_kinds;
    values = data_kinds;
    generics = [J];
    where = { J: IntoIterator<Item = LoadedSourceDataKind>, };
}

single_read_methods! {
    what = "tracked source path";
    reader = read_by_source_path;
    reader_relative = read_by_source_path_relative_to;
    reader_many = read_many_by_source_path;
    reader_many_relative = read_many_by_source_path_relative_to;
    setter = only_source_path;
    value = source_path: impl AsRef<Path>;
}

set_read_methods! {
    what = "tracked source paths";
    reader = read_by_source_paths;
    reader_relative = read_by_source_paths_relative_to;
    reader_many = read_many_by_source_paths;
    reader_many_relative = read_many_by_source_paths_relative_to;
    setter = only_source_paths;
    values = source_paths;
    generics = [J, F];
    where = { J: IntoIterator<Item = F>, F: AsRef<Path>, };
}

single_read_methods! {
    what = "tracked source path prefix";
    reader = read_by_source_path_prefix;
    reader_relative = read_by_source_path_prefix_relative_to;
    reader_many = read_many_by_source_path_prefix;
    reader_many_relative = read_many_by_source_path_prefix_relative_to;
    setter = only_source_path_prefix;
    value = source_path_prefix: impl AsRef<Path>;
}

set_read_methods! {
    what = "tracked source path prefixes";
    reader = read_by_source_path_prefixes;
    reader_relative = read_by_source_path_prefixes_relative_to;
    reader_many = read_many_by_source_path_prefixes;
    reader_many_relative = read_many_by_source_path_prefixes_relative_to;
    setter = only_source_path_prefixes;
    values = source_path_prefixes;
    generics = [J, F];
    where = { J: IntoIterator<Item = F>, F: AsRef<Path>, };
}

/// Loads supported spectra from a file or directory, restricted to any tracked source path.
///
/// Source paths are combined with logical OR. Passing an empty iterator leaves
/// source loading unrestricted.
///
/// # Errors
///
/// Returns an error when the path is missing or no matching readable bundle
/// data is found.
pub fn load_spectra_by_source_paths<I, P>(
    path: impl AsRef<Path>,
    source_paths: I,
) -> Result<SpectrumBundle>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_by_source_paths(path, source_paths)
}

/// Loads one selected path relative to a base directory, restricted to any tracked source path.
///
/// Relative input paths are resolved below `base`; absolute input paths are
/// loaded as provided. Source paths are matched after anchoring source metadata
/// to `base` and are combined with logical OR.
///
/// # Errors
///
/// Returns an error when `base` is missing or is not a directory, the path is
/// unreadable in strict mode, or no matching readable bundle data is found.
pub fn load_spectra_by_source_paths_relative_to<I, P>(
    base: impl AsRef<Path>,
    path: impl AsRef<Path>,
    source_paths: I,
) -> Result<SpectrumBundle>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_by_source_paths_relative_to(base, path, source_paths)
}

/// Loads supported spectra from multiple paths, restricted to any tracked source path.
///
/// # Errors
///
/// Returns an error when no input paths are provided or no matching readable
/// bundle data is found.
pub fn load_spectra_many_by_source_paths<I, P, J, F>(
    paths: I,
    source_paths: J,
) -> Result<SpectrumBundle>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
    J: IntoIterator<Item = F>,
    F: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_many_by_source_paths(paths, source_paths)
}

/// Loads selected paths relative to a base directory, restricted to any tracked source path.
///
/// Relative input paths are resolved below `base`; absolute input paths are
/// loaded as provided. Source paths are matched after anchoring source metadata
/// to `base` and are combined with logical OR.
///
/// # Errors
///
/// Returns an error when `base` is missing or is not a directory, no input
/// paths are provided, or no matching readable bundle data is found.
pub fn load_spectra_many_by_source_paths_relative_to<I, P, J, F>(
    base: impl AsRef<Path>,
    paths: I,
    source_paths: J,
) -> Result<SpectrumBundle>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
    J: IntoIterator<Item = F>,
    F: AsRef<Path>,
{
    SpectrumBundleLoader::new().read_many_by_source_paths_relative_to(base, paths, source_paths)
}
