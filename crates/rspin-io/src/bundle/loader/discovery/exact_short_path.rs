//! Short exact path loaders for discovered source candidates.

use std::path::Path;

use rspin_core::{Result, Spectrum1D, Spectrum2D};

use super::DiscoveredSpectrumSource;
use crate::bundle::{LoadedSource, SpectrumBundleLoader};

macro_rules! short_exact_path_aliases {
    (
        what = $what:literal;
        output = $output:ty;
        reader_relative = $reader_relative:ident;
        reader = $reader:ident;
        free_relative = $free_relative:ident;
        free = $free:ident;
        target_relative = $target_relative:ident;
        target = $target:ident;
    ) => {
        impl SpectrumBundleLoader {
            #[doc = concat!("Loads discovered candidates matching one tracked source path as exactly one ", $what, ".")]
            ///
            /// This is a short alias for the matching
            /// `read_discovered_*_by_source_path*` exact loader method.
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or exactly one matching
            /// spectrum is not found.
            pub fn $reader_relative<'a>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                path: impl AsRef<Path>,
            ) -> Result<$output> {
                self.$target_relative(base, sources, path)
            }

            #[doc = concat!("Loads discovered candidates matching one tracked source path as exactly one ", $what, ".")]
            ///
            /// This is a short alias for the matching
            /// `read_discovered_*_by_source_path*` exact loader method.
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or exactly one matching
            /// spectrum is not found.
            pub fn $reader<'a>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                path: impl AsRef<Path>,
            ) -> Result<$output> {
                self.$target(base, sources, path)
            }
        }

        #[doc = concat!("Loads discovered candidates matching one tracked source path as exactly one ", $what, ".")]
        ///
        /// This is a short alias for the matching
        /// `load_discovered_spectrum_*_by_source_path*` exact loader function.
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or exactly one matching spectrum is not found.
        pub fn $free_relative<'a>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            path: impl AsRef<Path>,
        ) -> Result<$output> {
            SpectrumBundleLoader::new().$reader_relative(base, sources, path)
        }

        #[doc = concat!("Loads discovered candidates matching one tracked source path as exactly one ", $what, ".")]
        ///
        /// This is a short alias for the matching
        /// `load_discovered_spectrum_*_by_source_path*` exact loader function.
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or exactly one matching spectrum is not found.
        pub fn $free<'a>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            path: impl AsRef<Path>,
        ) -> Result<$output> {
            SpectrumBundleLoader::new().$reader(base, sources, path)
        }
    };
}

macro_rules! short_exact_path_prefix_set_aliases {
    (
        what = $what:literal;
        output = $output:ty;
        reader_relative = $reader_relative:ident;
        reader = $reader:ident;
        free_relative = $free_relative:ident;
        free = $free:ident;
        target_relative = $target_relative:ident;
        target = $target:ident;
    ) => {
        impl SpectrumBundleLoader {
            #[doc = concat!("Loads discovered candidates below any tracked source path prefix as exactly one ", $what, ".")]
            ///
            /// Prefixes are combined with logical OR. Passing an empty
            /// iterator leaves source matching unrestricted.
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or exactly one matching
            /// spectrum is not found.
            pub fn $reader_relative<'a, I, P>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                paths: I,
            ) -> Result<$output>
            where
                I: IntoIterator<Item = P>,
                P: AsRef<Path>,
            {
                self.$target_relative(base, sources, paths)
            }

            #[doc = concat!("Loads discovered candidates below any tracked source path prefix as exactly one ", $what, ".")]
            ///
            /// Prefixes are combined with logical OR. Passing an empty
            /// iterator leaves source matching unrestricted.
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or exactly one matching
            /// spectrum is not found.
            pub fn $reader<'a, I, P>(
                &self,
                base: impl AsRef<Path>,
                sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
                paths: I,
            ) -> Result<$output>
            where
                I: IntoIterator<Item = P>,
                P: AsRef<Path>,
            {
                self.$target(base, sources, paths)
            }
        }

        #[doc = concat!("Loads discovered candidates below any tracked source path prefix as exactly one ", $what, ".")]
        ///
        /// Prefixes are combined with logical OR. Passing an empty iterator
        /// leaves source matching unrestricted.
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or exactly one matching spectrum is not found.
        pub fn $free_relative<'a, I, P>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            paths: I,
        ) -> Result<$output>
        where
            I: IntoIterator<Item = P>,
            P: AsRef<Path>,
        {
            SpectrumBundleLoader::new().$reader_relative(base, sources, paths)
        }

        #[doc = concat!("Loads discovered candidates below any tracked source path prefix as exactly one ", $what, ".")]
        ///
        /// Prefixes are combined with logical OR. Passing an empty iterator
        /// leaves source matching unrestricted.
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or exactly one matching spectrum is not found.
        pub fn $free<'a, I, P>(
            base: impl AsRef<Path>,
            sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
            paths: I,
        ) -> Result<$output>
        where
            I: IntoIterator<Item = P>,
            P: AsRef<Path>,
        {
            SpectrumBundleLoader::new().$reader(base, sources, paths)
        }
    };
}

short_exact_path_aliases! {
    what = "one-dimensional spectrum";
    output = Spectrum1D;
    reader_relative = read_discovered_1d_by_path_relative_to;
    reader = read_discovered_1d_by_path;
    free_relative = load_discovered_spectrum_1d_by_path_relative_to;
    free = load_discovered_spectrum_1d_by_path;
    target_relative = read_discovered_1d_by_source_path_relative_to;
    target = read_discovered_1d_by_source_path;
}

short_exact_path_aliases! {
    what = "one-dimensional spectrum with source metadata";
    output = (Spectrum1D, LoadedSource);
    reader_relative = read_discovered_1d_with_source_by_path_relative_to;
    reader = read_discovered_1d_with_source_by_path;
    free_relative = load_discovered_spectrum_1d_with_source_by_path_relative_to;
    free = load_discovered_spectrum_1d_with_source_by_path;
    target_relative = read_discovered_1d_with_source_by_source_path_relative_to;
    target = read_discovered_1d_with_source_by_source_path;
}

short_exact_path_aliases! {
    what = "two-dimensional spectrum";
    output = Spectrum2D;
    reader_relative = read_discovered_2d_by_path_relative_to;
    reader = read_discovered_2d_by_path;
    free_relative = load_discovered_spectrum_2d_by_path_relative_to;
    free = load_discovered_spectrum_2d_by_path;
    target_relative = read_discovered_2d_by_source_path_relative_to;
    target = read_discovered_2d_by_source_path;
}

short_exact_path_aliases! {
    what = "two-dimensional spectrum with source metadata";
    output = (Spectrum2D, LoadedSource);
    reader_relative = read_discovered_2d_with_source_by_path_relative_to;
    reader = read_discovered_2d_with_source_by_path;
    free_relative = load_discovered_spectrum_2d_with_source_by_path_relative_to;
    free = load_discovered_spectrum_2d_with_source_by_path;
    target_relative = read_discovered_2d_with_source_by_source_path_relative_to;
    target = read_discovered_2d_with_source_by_source_path;
}

short_exact_path_prefix_set_aliases! {
    what = "one-dimensional spectrum";
    output = Spectrum1D;
    reader_relative = read_discovered_1d_by_path_prefixes_relative_to;
    reader = read_discovered_1d_by_path_prefixes;
    free_relative = load_discovered_spectrum_1d_by_path_prefixes_relative_to;
    free = load_discovered_spectrum_1d_by_path_prefixes;
    target_relative = read_discovered_1d_by_source_path_prefixes_relative_to;
    target = read_discovered_1d_by_source_path_prefixes;
}

short_exact_path_prefix_set_aliases! {
    what = "one-dimensional spectrum with source metadata";
    output = (Spectrum1D, LoadedSource);
    reader_relative = read_discovered_1d_with_source_by_path_prefixes_relative_to;
    reader = read_discovered_1d_with_source_by_path_prefixes;
    free_relative = load_discovered_spectrum_1d_with_source_by_path_prefixes_relative_to;
    free = load_discovered_spectrum_1d_with_source_by_path_prefixes;
    target_relative = read_discovered_1d_with_source_by_source_path_prefixes_relative_to;
    target = read_discovered_1d_with_source_by_source_path_prefixes;
}

short_exact_path_prefix_set_aliases! {
    what = "two-dimensional spectrum";
    output = Spectrum2D;
    reader_relative = read_discovered_2d_by_path_prefixes_relative_to;
    reader = read_discovered_2d_by_path_prefixes;
    free_relative = load_discovered_spectrum_2d_by_path_prefixes_relative_to;
    free = load_discovered_spectrum_2d_by_path_prefixes;
    target_relative = read_discovered_2d_by_source_path_prefixes_relative_to;
    target = read_discovered_2d_by_source_path_prefixes;
}

short_exact_path_prefix_set_aliases! {
    what = "two-dimensional spectrum with source metadata";
    output = (Spectrum2D, LoadedSource);
    reader_relative = read_discovered_2d_with_source_by_path_prefixes_relative_to;
    reader = read_discovered_2d_with_source_by_path_prefixes;
    free_relative = load_discovered_spectrum_2d_with_source_by_path_prefixes_relative_to;
    free = load_discovered_spectrum_2d_with_source_by_path_prefixes;
    target_relative = read_discovered_2d_with_source_by_source_path_prefixes_relative_to;
    target = read_discovered_2d_with_source_by_source_path_prefixes;
}
