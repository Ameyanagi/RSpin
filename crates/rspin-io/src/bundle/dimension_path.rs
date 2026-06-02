//! Dimension-specific bundle loading helpers for source path filters.

use std::path::Path;

use rspin_core::Result;

use super::{SpectrumBundle, SpectrumBundleLoader, SpectrumBundleSummary};

macro_rules! dimension_path_helpers {
    (
        dimension = $dimension:literal;
        filter = $dimension_filter:ident;

        reader_path = $reader_path:ident;
        reader_paths = $reader_paths:ident;
        reader_path_prefix = $reader_path_prefix:ident;
        reader_path_prefixes = $reader_path_prefixes:ident;

        reader_path_relative = $reader_path_relative:ident;
        reader_paths_relative = $reader_paths_relative:ident;
        reader_path_prefix_relative = $reader_path_prefix_relative:ident;
        reader_path_prefixes_relative = $reader_path_prefixes_relative:ident;

        reader_summary_path = $reader_summary_path:ident;
        reader_summary_paths = $reader_summary_paths:ident;
        reader_summary_path_prefix = $reader_summary_path_prefix:ident;
        reader_summary_path_prefixes = $reader_summary_path_prefixes:ident;

        reader_summary_path_relative = $reader_summary_path_relative:ident;
        reader_summary_paths_relative = $reader_summary_paths_relative:ident;
        reader_summary_path_prefix_relative = $reader_summary_path_prefix_relative:ident;
        reader_summary_path_prefixes_relative = $reader_summary_path_prefixes_relative:ident;

        load_path = $load_path:ident;
        load_paths = $load_paths:ident;
        load_path_prefix = $load_path_prefix:ident;
        load_path_prefixes = $load_path_prefixes:ident;

        load_path_relative = $load_path_relative:ident;
        load_paths_relative = $load_paths_relative:ident;
        load_path_prefix_relative = $load_path_prefix_relative:ident;
        load_path_prefixes_relative = $load_path_prefixes_relative:ident;

        load_summary_path = $load_summary_path:ident;
        load_summary_paths = $load_summary_paths:ident;
        load_summary_path_prefix = $load_summary_path_prefix:ident;
        load_summary_path_prefixes = $load_summary_path_prefixes:ident;

        load_summary_path_relative = $load_summary_path_relative:ident;
        load_summary_paths_relative = $load_summary_paths_relative:ident;
        load_summary_path_prefix_relative = $load_summary_path_prefix_relative:ident;
        load_summary_path_prefixes_relative = $load_summary_path_prefixes_relative:ident;
    ) => {
        impl SpectrumBundleLoader {
            #[doc = concat!("Loads all ", $dimension, " spectra from a file or directory, restricted to one tracked source path.")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum data is found.
            pub fn $reader_path(
                &self,
                path: impl AsRef<Path>,
                source_path: impl AsRef<Path>,
            ) -> Result<SpectrumBundle> {
                self.clone()
                    .$dimension_filter()
                    .only_source_path(source_path)
                    .read_path(path)
            }

            #[doc = concat!("Loads all ", $dimension, " spectra from a file or directory, restricted to tracked source paths.")]
            ///
            /// Source paths are combined with logical OR. Passing an empty
            /// iterator leaves source-path loading unrestricted.
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum data is found.
            pub fn $reader_paths<I, P>(
                &self,
                path: impl AsRef<Path>,
                source_paths: I,
            ) -> Result<SpectrumBundle>
            where
                I: IntoIterator<Item = P>,
                P: AsRef<Path>,
            {
                self.clone()
                    .$dimension_filter()
                    .only_source_paths(source_paths)
                    .read_path(path)
            }

            #[doc = concat!("Loads all ", $dimension, " spectra from a file or directory, restricted to one tracked source path prefix.")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum data is found.
            pub fn $reader_path_prefix(
                &self,
                path: impl AsRef<Path>,
                source_path_prefix: impl AsRef<Path>,
            ) -> Result<SpectrumBundle> {
                self.clone()
                    .$dimension_filter()
                    .only_source_path_prefix(source_path_prefix)
                    .read_path(path)
            }

            #[doc = concat!("Loads all ", $dimension, " spectra from a file or directory, restricted to tracked source path prefixes.")]
            ///
            /// Source path prefixes are combined with logical OR. Passing an
            /// empty iterator leaves source-path-prefix loading unrestricted.
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum data is found.
            pub fn $reader_path_prefixes<I, P>(
                &self,
                path: impl AsRef<Path>,
                source_path_prefixes: I,
            ) -> Result<SpectrumBundle>
            where
                I: IntoIterator<Item = P>,
                P: AsRef<Path>,
            {
                self.clone()
                    .$dimension_filter()
                    .only_source_path_prefixes(source_path_prefixes)
                    .read_path(path)
            }

            #[doc = concat!("Loads all ", $dimension, " spectra from one selected path, restricted to one tracked source path.")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum data is found.
            pub fn $reader_path_relative(
                &self,
                base: impl AsRef<Path>,
                path: impl AsRef<Path>,
                source_path: impl AsRef<Path>,
            ) -> Result<SpectrumBundle> {
                self.clone()
                    .$dimension_filter()
                    .only_source_path(source_path)
                    .read_path_relative_to(base, path)
            }

            #[doc = concat!("Loads all ", $dimension, " spectra from one selected path, restricted to tracked source paths.")]
            ///
            /// Source paths are combined with logical OR. Passing an empty
            /// iterator leaves source-path loading unrestricted.
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum data is found.
            pub fn $reader_paths_relative<I, P>(
                &self,
                base: impl AsRef<Path>,
                path: impl AsRef<Path>,
                source_paths: I,
            ) -> Result<SpectrumBundle>
            where
                I: IntoIterator<Item = P>,
                P: AsRef<Path>,
            {
                self.clone()
                    .$dimension_filter()
                    .only_source_paths(source_paths)
                    .read_path_relative_to(base, path)
            }

            #[doc = concat!("Loads all ", $dimension, " spectra from one selected path, restricted to one tracked source path prefix.")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum data is found.
            pub fn $reader_path_prefix_relative(
                &self,
                base: impl AsRef<Path>,
                path: impl AsRef<Path>,
                source_path_prefix: impl AsRef<Path>,
            ) -> Result<SpectrumBundle> {
                self.clone()
                    .$dimension_filter()
                    .only_source_path_prefix(source_path_prefix)
                    .read_path_relative_to(base, path)
            }

            #[doc = concat!("Loads all ", $dimension, " spectra from one selected path, restricted to tracked source path prefixes.")]
            ///
            /// Source path prefixes are combined with logical OR. Passing an
            /// empty iterator leaves source-path-prefix loading unrestricted.
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum data is found.
            pub fn $reader_path_prefixes_relative<I, P>(
                &self,
                base: impl AsRef<Path>,
                path: impl AsRef<Path>,
                source_path_prefixes: I,
            ) -> Result<SpectrumBundle>
            where
                I: IntoIterator<Item = P>,
                P: AsRef<Path>,
            {
                self.clone()
                    .$dimension_filter()
                    .only_source_path_prefixes(source_path_prefixes)
                    .read_path_relative_to(base, path)
            }

            #[doc = concat!("Loads all ", $dimension, " spectra from a file or directory, restricted to one tracked source path, and returns summary counts.")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum data is found.
            pub fn $reader_summary_path(
                &self,
                path: impl AsRef<Path>,
                source_path: impl AsRef<Path>,
            ) -> Result<SpectrumBundleSummary> {
                self.$reader_path(path, source_path)
                    .map(|bundle| bundle.summary())
            }

            #[doc = concat!("Loads all ", $dimension, " spectra from a file or directory, restricted to tracked source paths, and returns summary counts.")]
            ///
            /// Source paths are combined with logical OR. Passing an empty
            /// iterator leaves source-path loading unrestricted.
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum data is found.
            pub fn $reader_summary_paths<I, P>(
                &self,
                path: impl AsRef<Path>,
                source_paths: I,
            ) -> Result<SpectrumBundleSummary>
            where
                I: IntoIterator<Item = P>,
                P: AsRef<Path>,
            {
                self.$reader_paths(path, source_paths)
                    .map(|bundle| bundle.summary())
            }

            #[doc = concat!("Loads all ", $dimension, " spectra from a file or directory, restricted to one tracked source path prefix, and returns summary counts.")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum data is found.
            pub fn $reader_summary_path_prefix(
                &self,
                path: impl AsRef<Path>,
                source_path_prefix: impl AsRef<Path>,
            ) -> Result<SpectrumBundleSummary> {
                self.$reader_path_prefix(path, source_path_prefix)
                    .map(|bundle| bundle.summary())
            }

            #[doc = concat!("Loads all ", $dimension, " spectra from a file or directory, restricted to tracked source path prefixes, and returns summary counts.")]
            ///
            /// Source path prefixes are combined with logical OR. Passing an
            /// empty iterator leaves source-path-prefix loading unrestricted.
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum data is found.
            pub fn $reader_summary_path_prefixes<I, P>(
                &self,
                path: impl AsRef<Path>,
                source_path_prefixes: I,
            ) -> Result<SpectrumBundleSummary>
            where
                I: IntoIterator<Item = P>,
                P: AsRef<Path>,
            {
                self.$reader_path_prefixes(path, source_path_prefixes)
                    .map(|bundle| bundle.summary())
            }

            #[doc = concat!("Loads all ", $dimension, " spectra from one selected path, restricted to one tracked source path, and returns summary counts.")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum data is found.
            pub fn $reader_summary_path_relative(
                &self,
                base: impl AsRef<Path>,
                path: impl AsRef<Path>,
                source_path: impl AsRef<Path>,
            ) -> Result<SpectrumBundleSummary> {
                self.$reader_path_relative(base, path, source_path)
                    .map(|bundle| bundle.summary())
            }

            #[doc = concat!("Loads all ", $dimension, " spectra from one selected path, restricted to tracked source paths, and returns summary counts.")]
            ///
            /// Source paths are combined with logical OR. Passing an empty
            /// iterator leaves source-path loading unrestricted.
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum data is found.
            pub fn $reader_summary_paths_relative<I, P>(
                &self,
                base: impl AsRef<Path>,
                path: impl AsRef<Path>,
                source_paths: I,
            ) -> Result<SpectrumBundleSummary>
            where
                I: IntoIterator<Item = P>,
                P: AsRef<Path>,
            {
                self.$reader_paths_relative(base, path, source_paths)
                    .map(|bundle| bundle.summary())
            }

            #[doc = concat!("Loads all ", $dimension, " spectra from one selected path, restricted to one tracked source path prefix, and returns summary counts.")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum data is found.
            pub fn $reader_summary_path_prefix_relative(
                &self,
                base: impl AsRef<Path>,
                path: impl AsRef<Path>,
                source_path_prefix: impl AsRef<Path>,
            ) -> Result<SpectrumBundleSummary> {
                self.$reader_path_prefix_relative(base, path, source_path_prefix)
                    .map(|bundle| bundle.summary())
            }

            #[doc = concat!("Loads all ", $dimension, " spectra from one selected path, restricted to tracked source path prefixes, and returns summary counts.")]
            ///
            /// Source path prefixes are combined with logical OR. Passing an
            /// empty iterator leaves source-path-prefix loading unrestricted.
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum data is found.
            pub fn $reader_summary_path_prefixes_relative<I, P>(
                &self,
                base: impl AsRef<Path>,
                path: impl AsRef<Path>,
                source_path_prefixes: I,
            ) -> Result<SpectrumBundleSummary>
            where
                I: IntoIterator<Item = P>,
                P: AsRef<Path>,
            {
                self.$reader_path_prefixes_relative(base, path, source_path_prefixes)
                    .map(|bundle| bundle.summary())
            }
        }

        #[doc = concat!("Loads all ", $dimension, " spectra from a file or directory, restricted to one tracked source path.")]
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $load_path(
            path: impl AsRef<Path>,
            source_path: impl AsRef<Path>,
        ) -> Result<SpectrumBundle> {
            SpectrumBundleLoader::new().$reader_path(path, source_path)
        }

        #[doc = concat!("Loads all ", $dimension, " spectra from a file or directory, restricted to tracked source paths.")]
        ///
        /// Source paths are combined with logical OR. Passing an empty iterator
        /// leaves source-path loading unrestricted.
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $load_paths<I, P>(
            path: impl AsRef<Path>,
            source_paths: I,
        ) -> Result<SpectrumBundle>
        where
            I: IntoIterator<Item = P>,
            P: AsRef<Path>,
        {
            SpectrumBundleLoader::new().$reader_paths(path, source_paths)
        }

        #[doc = concat!("Loads all ", $dimension, " spectra from a file or directory, restricted to one tracked source path prefix.")]
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $load_path_prefix(
            path: impl AsRef<Path>,
            source_path_prefix: impl AsRef<Path>,
        ) -> Result<SpectrumBundle> {
            SpectrumBundleLoader::new().$reader_path_prefix(path, source_path_prefix)
        }

        #[doc = concat!("Loads all ", $dimension, " spectra from a file or directory, restricted to tracked source path prefixes.")]
        ///
        /// Source path prefixes are combined with logical OR. Passing an empty
        /// iterator leaves source-path-prefix loading unrestricted.
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $load_path_prefixes<I, P>(
            path: impl AsRef<Path>,
            source_path_prefixes: I,
        ) -> Result<SpectrumBundle>
        where
            I: IntoIterator<Item = P>,
            P: AsRef<Path>,
        {
            SpectrumBundleLoader::new().$reader_path_prefixes(path, source_path_prefixes)
        }

        #[doc = concat!("Loads all ", $dimension, " spectra from one selected path, restricted to one tracked source path.")]
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $load_path_relative(
            base: impl AsRef<Path>,
            path: impl AsRef<Path>,
            source_path: impl AsRef<Path>,
        ) -> Result<SpectrumBundle> {
            SpectrumBundleLoader::new().$reader_path_relative(base, path, source_path)
        }

        #[doc = concat!("Loads all ", $dimension, " spectra from one selected path, restricted to tracked source paths.")]
        ///
        /// Source paths are combined with logical OR. Passing an empty iterator
        /// leaves source-path loading unrestricted.
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $load_paths_relative<I, P>(
            base: impl AsRef<Path>,
            path: impl AsRef<Path>,
            source_paths: I,
        ) -> Result<SpectrumBundle>
        where
            I: IntoIterator<Item = P>,
            P: AsRef<Path>,
        {
            SpectrumBundleLoader::new().$reader_paths_relative(base, path, source_paths)
        }

        #[doc = concat!("Loads all ", $dimension, " spectra from one selected path, restricted to one tracked source path prefix.")]
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $load_path_prefix_relative(
            base: impl AsRef<Path>,
            path: impl AsRef<Path>,
            source_path_prefix: impl AsRef<Path>,
        ) -> Result<SpectrumBundle> {
            SpectrumBundleLoader::new()
                .$reader_path_prefix_relative(base, path, source_path_prefix)
        }

        #[doc = concat!("Loads all ", $dimension, " spectra from one selected path, restricted to tracked source path prefixes.")]
        ///
        /// Source path prefixes are combined with logical OR. Passing an empty
        /// iterator leaves source-path-prefix loading unrestricted.
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $load_path_prefixes_relative<I, P>(
            base: impl AsRef<Path>,
            path: impl AsRef<Path>,
            source_path_prefixes: I,
        ) -> Result<SpectrumBundle>
        where
            I: IntoIterator<Item = P>,
            P: AsRef<Path>,
        {
            SpectrumBundleLoader::new()
                .$reader_path_prefixes_relative(base, path, source_path_prefixes)
        }

        #[doc = concat!("Loads all ", $dimension, " spectra from a file or directory, restricted to one tracked source path, and returns summary counts.")]
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $load_summary_path(
            path: impl AsRef<Path>,
            source_path: impl AsRef<Path>,
        ) -> Result<SpectrumBundleSummary> {
            SpectrumBundleLoader::new().$reader_summary_path(path, source_path)
        }

        #[doc = concat!("Loads all ", $dimension, " spectra from a file or directory, restricted to tracked source paths, and returns summary counts.")]
        ///
        /// Source paths are combined with logical OR. Passing an empty iterator
        /// leaves source-path loading unrestricted.
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $load_summary_paths<I, P>(
            path: impl AsRef<Path>,
            source_paths: I,
        ) -> Result<SpectrumBundleSummary>
        where
            I: IntoIterator<Item = P>,
            P: AsRef<Path>,
        {
            SpectrumBundleLoader::new().$reader_summary_paths(path, source_paths)
        }

        #[doc = concat!("Loads all ", $dimension, " spectra from a file or directory, restricted to one tracked source path prefix, and returns summary counts.")]
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $load_summary_path_prefix(
            path: impl AsRef<Path>,
            source_path_prefix: impl AsRef<Path>,
        ) -> Result<SpectrumBundleSummary> {
            SpectrumBundleLoader::new().$reader_summary_path_prefix(path, source_path_prefix)
        }

        #[doc = concat!("Loads all ", $dimension, " spectra from a file or directory, restricted to tracked source path prefixes, and returns summary counts.")]
        ///
        /// Source path prefixes are combined with logical OR. Passing an empty
        /// iterator leaves source-path-prefix loading unrestricted.
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $load_summary_path_prefixes<I, P>(
            path: impl AsRef<Path>,
            source_path_prefixes: I,
        ) -> Result<SpectrumBundleSummary>
        where
            I: IntoIterator<Item = P>,
            P: AsRef<Path>,
        {
            SpectrumBundleLoader::new()
                .$reader_summary_path_prefixes(path, source_path_prefixes)
        }

        #[doc = concat!("Loads all ", $dimension, " spectra from one selected path, restricted to one tracked source path, and returns summary counts.")]
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $load_summary_path_relative(
            base: impl AsRef<Path>,
            path: impl AsRef<Path>,
            source_path: impl AsRef<Path>,
        ) -> Result<SpectrumBundleSummary> {
            SpectrumBundleLoader::new().$reader_summary_path_relative(base, path, source_path)
        }

        #[doc = concat!("Loads all ", $dimension, " spectra from one selected path, restricted to tracked source paths, and returns summary counts.")]
        ///
        /// Source paths are combined with logical OR. Passing an empty iterator
        /// leaves source-path loading unrestricted.
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $load_summary_paths_relative<I, P>(
            base: impl AsRef<Path>,
            path: impl AsRef<Path>,
            source_paths: I,
        ) -> Result<SpectrumBundleSummary>
        where
            I: IntoIterator<Item = P>,
            P: AsRef<Path>,
        {
            SpectrumBundleLoader::new()
                .$reader_summary_paths_relative(base, path, source_paths)
        }

        #[doc = concat!("Loads all ", $dimension, " spectra from one selected path, restricted to one tracked source path prefix, and returns summary counts.")]
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $load_summary_path_prefix_relative(
            base: impl AsRef<Path>,
            path: impl AsRef<Path>,
            source_path_prefix: impl AsRef<Path>,
        ) -> Result<SpectrumBundleSummary> {
            SpectrumBundleLoader::new()
                .$reader_summary_path_prefix_relative(base, path, source_path_prefix)
        }

        #[doc = concat!("Loads all ", $dimension, " spectra from one selected path, restricted to tracked source path prefixes, and returns summary counts.")]
        ///
        /// Source path prefixes are combined with logical OR. Passing an empty
        /// iterator leaves source-path-prefix loading unrestricted.
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $load_summary_path_prefixes_relative<I, P>(
            base: impl AsRef<Path>,
            path: impl AsRef<Path>,
            source_path_prefixes: I,
        ) -> Result<SpectrumBundleSummary>
        where
            I: IntoIterator<Item = P>,
            P: AsRef<Path>,
        {
            SpectrumBundleLoader::new()
                .$reader_summary_path_prefixes_relative(base, path, source_path_prefixes)
        }
    };
}

dimension_path_helpers! {
    dimension = "one-dimensional";
    filter = one_d_only;

    reader_path = read_bundle_1d_by_source_path;
    reader_paths = read_bundle_1d_by_source_paths;
    reader_path_prefix = read_bundle_1d_by_source_path_prefix;
    reader_path_prefixes = read_bundle_1d_by_source_path_prefixes;

    reader_path_relative = read_bundle_1d_by_source_path_relative_to;
    reader_paths_relative = read_bundle_1d_by_source_paths_relative_to;
    reader_path_prefix_relative = read_bundle_1d_by_source_path_prefix_relative_to;
    reader_path_prefixes_relative = read_bundle_1d_by_source_path_prefixes_relative_to;

    reader_summary_path = read_bundle_1d_summary_by_source_path;
    reader_summary_paths = read_bundle_1d_summary_by_source_paths;
    reader_summary_path_prefix = read_bundle_1d_summary_by_source_path_prefix;
    reader_summary_path_prefixes = read_bundle_1d_summary_by_source_path_prefixes;

    reader_summary_path_relative = read_bundle_1d_summary_by_source_path_relative_to;
    reader_summary_paths_relative = read_bundle_1d_summary_by_source_paths_relative_to;
    reader_summary_path_prefix_relative = read_bundle_1d_summary_by_source_path_prefix_relative_to;
    reader_summary_path_prefixes_relative = read_bundle_1d_summary_by_source_path_prefixes_relative_to;

    load_path = load_spectra_1d_by_source_path;
    load_paths = load_spectra_1d_by_source_paths;
    load_path_prefix = load_spectra_1d_by_source_path_prefix;
    load_path_prefixes = load_spectra_1d_by_source_path_prefixes;

    load_path_relative = load_spectra_1d_by_source_path_relative_to;
    load_paths_relative = load_spectra_1d_by_source_paths_relative_to;
    load_path_prefix_relative = load_spectra_1d_by_source_path_prefix_relative_to;
    load_path_prefixes_relative = load_spectra_1d_by_source_path_prefixes_relative_to;

    load_summary_path = load_spectra_1d_summary_by_source_path;
    load_summary_paths = load_spectra_1d_summary_by_source_paths;
    load_summary_path_prefix = load_spectra_1d_summary_by_source_path_prefix;
    load_summary_path_prefixes = load_spectra_1d_summary_by_source_path_prefixes;

    load_summary_path_relative = load_spectra_1d_summary_by_source_path_relative_to;
    load_summary_paths_relative = load_spectra_1d_summary_by_source_paths_relative_to;
    load_summary_path_prefix_relative = load_spectra_1d_summary_by_source_path_prefix_relative_to;
    load_summary_path_prefixes_relative = load_spectra_1d_summary_by_source_path_prefixes_relative_to;
}

dimension_path_helpers! {
    dimension = "two-dimensional";
    filter = two_d_only;

    reader_path = read_bundle_2d_by_source_path;
    reader_paths = read_bundle_2d_by_source_paths;
    reader_path_prefix = read_bundle_2d_by_source_path_prefix;
    reader_path_prefixes = read_bundle_2d_by_source_path_prefixes;

    reader_path_relative = read_bundle_2d_by_source_path_relative_to;
    reader_paths_relative = read_bundle_2d_by_source_paths_relative_to;
    reader_path_prefix_relative = read_bundle_2d_by_source_path_prefix_relative_to;
    reader_path_prefixes_relative = read_bundle_2d_by_source_path_prefixes_relative_to;

    reader_summary_path = read_bundle_2d_summary_by_source_path;
    reader_summary_paths = read_bundle_2d_summary_by_source_paths;
    reader_summary_path_prefix = read_bundle_2d_summary_by_source_path_prefix;
    reader_summary_path_prefixes = read_bundle_2d_summary_by_source_path_prefixes;

    reader_summary_path_relative = read_bundle_2d_summary_by_source_path_relative_to;
    reader_summary_paths_relative = read_bundle_2d_summary_by_source_paths_relative_to;
    reader_summary_path_prefix_relative = read_bundle_2d_summary_by_source_path_prefix_relative_to;
    reader_summary_path_prefixes_relative = read_bundle_2d_summary_by_source_path_prefixes_relative_to;

    load_path = load_spectra_2d_by_source_path;
    load_paths = load_spectra_2d_by_source_paths;
    load_path_prefix = load_spectra_2d_by_source_path_prefix;
    load_path_prefixes = load_spectra_2d_by_source_path_prefixes;

    load_path_relative = load_spectra_2d_by_source_path_relative_to;
    load_paths_relative = load_spectra_2d_by_source_paths_relative_to;
    load_path_prefix_relative = load_spectra_2d_by_source_path_prefix_relative_to;
    load_path_prefixes_relative = load_spectra_2d_by_source_path_prefixes_relative_to;

    load_summary_path = load_spectra_2d_summary_by_source_path;
    load_summary_paths = load_spectra_2d_summary_by_source_paths;
    load_summary_path_prefix = load_spectra_2d_summary_by_source_path_prefix;
    load_summary_path_prefixes = load_spectra_2d_summary_by_source_path_prefixes;

    load_summary_path_relative = load_spectra_2d_summary_by_source_path_relative_to;
    load_summary_paths_relative = load_spectra_2d_summary_by_source_paths_relative_to;
    load_summary_path_prefix_relative = load_spectra_2d_summary_by_source_path_prefix_relative_to;
    load_summary_path_prefixes_relative = load_spectra_2d_summary_by_source_path_prefixes_relative_to;
}
