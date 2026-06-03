//! Short dimension-specific path loaders for discovered source candidates.

use std::path::Path;

use rspin_core::Result;

use super::DiscoveredSpectrumSource;
use crate::bundle::{SpectrumBundle, SpectrumBundleLoader, SpectrumBundleSummary};

macro_rules! dimension_path_bundle_aliases {
    (
        dimension = $dimension:literal;
        reader_relative = $reader_relative:ident;
        reader = $reader:ident;
        free_relative = $free_relative:ident;
        free = $free:ident;
        target_relative = $target_relative:ident;
        target = $target:ident;
    ) => {
        impl SpectrumBundleLoader {
            #[doc = concat!("Loads ", $dimension, " discovered candidates matching one tracked source path as a bundle.")]
            ///
            /// This is a short alias for the matching
            /// `read_discovered_bundle_*_by_source_path*` method.
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum
            /// data is found.
            pub fn $reader_relative<'a>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                source_path: impl AsRef<Path>,
            ) -> Result<SpectrumBundle> {
                self.$target_relative(base, sources, source_path)
            }

            #[doc = concat!("Loads ", $dimension, " discovered candidates matching one tracked source path as a bundle.")]
            ///
            /// This is a short alias for the matching
            /// `read_discovered_bundle_*_by_source_path*` method.
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum
            /// data is found.
            pub fn $reader<'a>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                source_path: impl AsRef<Path>,
            ) -> Result<SpectrumBundle> {
                self.$target(base, sources, source_path)
            }
        }

        #[doc = concat!("Loads ", $dimension, " discovered candidates matching one tracked source path as a bundle.")]
        ///
        /// This is a short alias for the matching
        /// `load_discovered_spectra_*_by_source_path*` function.
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $free_relative<'a>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            source_path: impl AsRef<Path>,
        ) -> Result<SpectrumBundle> {
            SpectrumBundleLoader::new().$reader_relative(base, sources, source_path)
        }

        #[doc = concat!("Loads ", $dimension, " discovered candidates matching one tracked source path as a bundle.")]
        ///
        /// This is a short alias for the matching
        /// `load_discovered_spectra_*_by_source_path*` function.
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $free<'a>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            source_path: impl AsRef<Path>,
        ) -> Result<SpectrumBundle> {
            SpectrumBundleLoader::new().$reader(base, sources, source_path)
        }
    };
}

macro_rules! dimension_paths_bundle_aliases {
    (
        dimension = $dimension:literal;
        reader_relative = $reader_relative:ident;
        reader = $reader:ident;
        free_relative = $free_relative:ident;
        free = $free:ident;
        target_relative = $target_relative:ident;
        target = $target:ident;
    ) => {
        impl SpectrumBundleLoader {
            #[doc = concat!("Loads ", $dimension, " discovered candidates matching any tracked source path as a bundle.")]
            ///
            /// Paths are combined with logical OR. Passing an empty iterator
            /// leaves source matching unrestricted before dimension filtering.
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum
            /// data is found.
            pub fn $reader_relative<'a, I, P>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                source_paths: I,
            ) -> Result<SpectrumBundle>
            where
                I: IntoIterator<Item = P>,
                P: AsRef<Path>,
            {
                self.$target_relative(base, sources, source_paths)
            }

            #[doc = concat!("Loads ", $dimension, " discovered candidates matching any tracked source path as a bundle.")]
            ///
            /// Paths are combined with logical OR. Passing an empty iterator
            /// leaves source matching unrestricted before dimension filtering.
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum
            /// data is found.
            pub fn $reader<'a, I, P>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                source_paths: I,
            ) -> Result<SpectrumBundle>
            where
                I: IntoIterator<Item = P>,
                P: AsRef<Path>,
            {
                self.$target(base, sources, source_paths)
            }
        }

        #[doc = concat!("Loads ", $dimension, " discovered candidates matching any tracked source path as a bundle.")]
        ///
        /// Paths are combined with logical OR. Passing an empty iterator
        /// leaves source matching unrestricted before dimension filtering.
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $free_relative<'a, I, P>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            source_paths: I,
        ) -> Result<SpectrumBundle>
        where
            I: IntoIterator<Item = P>,
            P: AsRef<Path>,
        {
            SpectrumBundleLoader::new().$reader_relative(base, sources, source_paths)
        }

        #[doc = concat!("Loads ", $dimension, " discovered candidates matching any tracked source path as a bundle.")]
        ///
        /// Paths are combined with logical OR. Passing an empty iterator
        /// leaves source matching unrestricted before dimension filtering.
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $free<'a, I, P>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            source_paths: I,
        ) -> Result<SpectrumBundle>
        where
            I: IntoIterator<Item = P>,
            P: AsRef<Path>,
        {
            SpectrumBundleLoader::new().$reader(base, sources, source_paths)
        }
    };
}

macro_rules! dimension_path_summary_aliases {
    (
        dimension = $dimension:literal;
        reader_relative = $reader_relative:ident;
        reader = $reader:ident;
        free_relative = $free_relative:ident;
        free = $free:ident;
        target_relative = $target_relative:ident;
        target = $target:ident;
    ) => {
        impl SpectrumBundleLoader {
            #[doc = concat!("Loads ", $dimension, " discovered candidates matching one tracked source path and returns summary counts.")]
            ///
            /// This is a short alias for the matching
            /// `read_discovered_bundle_*_summary_by_source_path*` method.
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum
            /// data is found.
            pub fn $reader_relative<'a>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                source_path: impl AsRef<Path>,
            ) -> Result<SpectrumBundleSummary> {
                self.$target_relative(base, sources, [source_path])
            }

            #[doc = concat!("Loads ", $dimension, " discovered candidates matching one tracked source path and returns summary counts.")]
            ///
            /// This is a short alias for the matching
            /// `read_discovered_bundle_*_summary_by_source_path*` method.
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum
            /// data is found.
            pub fn $reader<'a>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                source_path: impl AsRef<Path>,
            ) -> Result<SpectrumBundleSummary> {
                self.$target(base, sources, [source_path])
            }
        }

        #[doc = concat!("Loads ", $dimension, " discovered candidates matching one tracked source path and returns summary counts.")]
        ///
        /// This is a short alias for the matching
        /// `load_discovered_spectra_*_summary_by_source_path*` function.
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $free_relative<'a>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            source_path: impl AsRef<Path>,
        ) -> Result<SpectrumBundleSummary> {
            SpectrumBundleLoader::new().$reader_relative(base, sources, source_path)
        }

        #[doc = concat!("Loads ", $dimension, " discovered candidates matching one tracked source path and returns summary counts.")]
        ///
        /// This is a short alias for the matching
        /// `load_discovered_spectra_*_summary_by_source_path*` function.
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $free<'a>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            source_path: impl AsRef<Path>,
        ) -> Result<SpectrumBundleSummary> {
            SpectrumBundleLoader::new().$reader(base, sources, source_path)
        }
    };
}

macro_rules! dimension_paths_summary_aliases {
    (
        dimension = $dimension:literal;
        reader_relative = $reader_relative:ident;
        reader = $reader:ident;
        free_relative = $free_relative:ident;
        free = $free:ident;
        target_relative = $target_relative:ident;
        target = $target:ident;
    ) => {
        impl SpectrumBundleLoader {
            #[doc = concat!("Loads ", $dimension, " discovered candidates matching any tracked source path and returns summary counts.")]
            ///
            /// Paths are combined with logical OR. Passing an empty iterator
            /// leaves source matching unrestricted before dimension filtering.
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum
            /// data is found.
            pub fn $reader_relative<'a, I, P>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                source_paths: I,
            ) -> Result<SpectrumBundleSummary>
            where
                I: IntoIterator<Item = P>,
                P: AsRef<Path>,
            {
                self.$target_relative(base, sources, source_paths)
            }

            #[doc = concat!("Loads ", $dimension, " discovered candidates matching any tracked source path and returns summary counts.")]
            ///
            /// Paths are combined with logical OR. Passing an empty iterator
            /// leaves source matching unrestricted before dimension filtering.
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum
            /// data is found.
            pub fn $reader<'a, I, P>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                source_paths: I,
            ) -> Result<SpectrumBundleSummary>
            where
                I: IntoIterator<Item = P>,
                P: AsRef<Path>,
            {
                self.$target(base, sources, source_paths)
            }
        }

        #[doc = concat!("Loads ", $dimension, " discovered candidates matching any tracked source path and returns summary counts.")]
        ///
        /// Paths are combined with logical OR. Passing an empty iterator
        /// leaves source matching unrestricted before dimension filtering.
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $free_relative<'a, I, P>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            source_paths: I,
        ) -> Result<SpectrumBundleSummary>
        where
            I: IntoIterator<Item = P>,
            P: AsRef<Path>,
        {
            SpectrumBundleLoader::new().$reader_relative(base, sources, source_paths)
        }

        #[doc = concat!("Loads ", $dimension, " discovered candidates matching any tracked source path and returns summary counts.")]
        ///
        /// Paths are combined with logical OR. Passing an empty iterator
        /// leaves source matching unrestricted before dimension filtering.
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum data is found.
        pub fn $free<'a, I, P>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            source_paths: I,
        ) -> Result<SpectrumBundleSummary>
        where
            I: IntoIterator<Item = P>,
            P: AsRef<Path>,
        {
            SpectrumBundleLoader::new().$reader(base, sources, source_paths)
        }
    };
}

dimension_path_bundle_aliases! {
    dimension = "one-dimensional";
    reader_relative = read_discovered_bundle_1d_by_path_relative_to;
    reader = read_discovered_bundle_1d_by_path;
    free_relative = load_discovered_spectra_1d_by_path_relative_to;
    free = load_discovered_spectra_1d_by_path;
    target_relative = read_discovered_bundle_1d_by_source_path_relative_to;
    target = read_discovered_bundle_1d_by_source_path;
}

dimension_paths_bundle_aliases! {
    dimension = "one-dimensional";
    reader_relative = read_discovered_bundle_1d_by_paths_relative_to;
    reader = read_discovered_bundle_1d_by_paths;
    free_relative = load_discovered_spectra_1d_by_paths_relative_to;
    free = load_discovered_spectra_1d_by_paths;
    target_relative = read_discovered_bundle_1d_by_source_paths_relative_to;
    target = read_discovered_bundle_1d_by_source_paths;
}

dimension_path_summary_aliases! {
    dimension = "one-dimensional";
    reader_relative = read_discovered_bundle_1d_summary_by_path_relative_to;
    reader = read_discovered_bundle_1d_summary_by_path;
    free_relative = load_discovered_spectra_1d_summary_by_path_relative_to;
    free = load_discovered_spectra_1d_summary_by_path;
    target_relative = read_discovered_bundle_1d_summary_by_source_paths_relative_to;
    target = read_discovered_bundle_1d_summary_by_source_paths;
}

dimension_paths_summary_aliases! {
    dimension = "one-dimensional";
    reader_relative = read_discovered_bundle_1d_summary_by_paths_relative_to;
    reader = read_discovered_bundle_1d_summary_by_paths;
    free_relative = load_discovered_spectra_1d_summary_by_paths_relative_to;
    free = load_discovered_spectra_1d_summary_by_paths;
    target_relative = read_discovered_bundle_1d_summary_by_source_paths_relative_to;
    target = read_discovered_bundle_1d_summary_by_source_paths;
}

dimension_path_bundle_aliases! {
    dimension = "two-dimensional";
    reader_relative = read_discovered_bundle_2d_by_path_relative_to;
    reader = read_discovered_bundle_2d_by_path;
    free_relative = load_discovered_spectra_2d_by_path_relative_to;
    free = load_discovered_spectra_2d_by_path;
    target_relative = read_discovered_bundle_2d_by_source_path_relative_to;
    target = read_discovered_bundle_2d_by_source_path;
}

dimension_paths_bundle_aliases! {
    dimension = "two-dimensional";
    reader_relative = read_discovered_bundle_2d_by_paths_relative_to;
    reader = read_discovered_bundle_2d_by_paths;
    free_relative = load_discovered_spectra_2d_by_paths_relative_to;
    free = load_discovered_spectra_2d_by_paths;
    target_relative = read_discovered_bundle_2d_by_source_paths_relative_to;
    target = read_discovered_bundle_2d_by_source_paths;
}

dimension_path_summary_aliases! {
    dimension = "two-dimensional";
    reader_relative = read_discovered_bundle_2d_summary_by_path_relative_to;
    reader = read_discovered_bundle_2d_summary_by_path;
    free_relative = load_discovered_spectra_2d_summary_by_path_relative_to;
    free = load_discovered_spectra_2d_summary_by_path;
    target_relative = read_discovered_bundle_2d_summary_by_source_paths_relative_to;
    target = read_discovered_bundle_2d_summary_by_source_paths;
}

dimension_paths_summary_aliases! {
    dimension = "two-dimensional";
    reader_relative = read_discovered_bundle_2d_summary_by_paths_relative_to;
    reader = read_discovered_bundle_2d_summary_by_paths;
    free_relative = load_discovered_spectra_2d_summary_by_paths_relative_to;
    free = load_discovered_spectra_2d_summary_by_paths;
    target_relative = read_discovered_bundle_2d_summary_by_source_paths_relative_to;
    target = read_discovered_bundle_2d_summary_by_source_paths;
}
