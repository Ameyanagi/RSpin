//! Strict first-spectrum reader helpers for quick inspection workflows.

use std::path::Path;

use rspin_core::{Result, Spectrum1D, Spectrum2D};

use super::{LoadedSource, LoadedSpectrum, SpectrumBundleLoader};

macro_rules! strict_first_methods {
    (
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
        into_first = $into_first:ident;
    ) => {
        impl SpectrumBundleLoader {
            #[doc = concat!("Strictly loads a file or directory and returns the first ", $what, ".")]
            ///
            /// # Errors
            ///
            /// Returns an error when strict loading fails or no matching spectrum is found.
            pub fn $reader(&self, path: impl AsRef<Path>) -> Result<$output> {
                self.clone().strict().read_path(path)?.$into_first()
            }

            #[doc = concat!("Strictly loads one selected path relative to a base directory and returns the first ", $what, ".")]
            ///
            /// # Errors
            ///
            /// Returns an error when strict loading fails or no matching spectrum is found.
            pub fn $reader_relative(
                &self,
                base: impl AsRef<Path>,
                path: impl AsRef<Path>,
            ) -> Result<$output> {
                self.clone()
                    .strict()
                    .read_path_relative_to(base, path)?
                    .$into_first()
            }

            #[doc = concat!("Strictly loads selected paths and returns the first ", $what, ".")]
            ///
            /// # Errors
            ///
            /// Returns an error when strict loading fails or no matching spectrum is found.
            pub fn $reader_many<I, P>(&self, paths: I) -> Result<$output>
            where
                I: IntoIterator<Item = P>,
                P: AsRef<Path>,
            {
                self.clone().strict().read_paths(paths)?.$into_first()
            }

            #[doc = concat!("Strictly loads selected paths relative to a base directory and returns the first ", $what, ".")]
            ///
            /// # Errors
            ///
            /// Returns an error when strict loading fails or no matching spectrum is found.
            pub fn $reader_many_relative<I, P>(
                &self,
                base: impl AsRef<Path>,
                paths: I,
            ) -> Result<$output>
            where
                I: IntoIterator<Item = P>,
                P: AsRef<Path>,
            {
                self.clone()
                    .strict()
                    .read_paths_relative_to(base, paths)?
                    .$into_first()
            }
        }

        #[doc = concat!("Strictly loads a file or directory and returns the first ", $what, ".")]
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading fails or no matching spectrum is found.
        pub fn $free(path: impl AsRef<Path>) -> Result<$output> {
            SpectrumBundleLoader::new().$reader(path)
        }

        #[doc = concat!("Strictly loads one selected path relative to a base directory and returns the first ", $what, ".")]
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading fails or no matching spectrum is found.
        pub fn $free_relative(
            base: impl AsRef<Path>,
            path: impl AsRef<Path>,
        ) -> Result<$output> {
            SpectrumBundleLoader::new().$reader_relative(base, path)
        }

        #[doc = concat!("Strictly loads selected paths and returns the first ", $what, ".")]
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading fails or no matching spectrum is found.
        pub fn $free_many<I, P>(paths: I) -> Result<$output>
        where
            I: IntoIterator<Item = P>,
            P: AsRef<Path>,
        {
            SpectrumBundleLoader::new().$reader_many(paths)
        }

        #[doc = concat!("Strictly loads selected paths relative to a base directory and returns the first ", $what, ".")]
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading fails or no matching spectrum is found.
        pub fn $free_many_relative<I, P>(
            base: impl AsRef<Path>,
            paths: I,
        ) -> Result<$output>
        where
            I: IntoIterator<Item = P>,
            P: AsRef<Path>,
        {
            SpectrumBundleLoader::new().$reader_many_relative(base, paths)
        }
    };
}

strict_first_methods! {
    what = "spectrum of any supported dimension";
    output = LoadedSpectrum;
    reader = read_first_spectrum_strict;
    reader_relative = read_first_spectrum_strict_relative_to;
    reader_many = read_first_spectrum_many_strict;
    reader_many_relative = read_first_spectrum_many_strict_relative_to;
    free = load_first_spectrum_strict;
    free_relative = load_first_spectrum_strict_relative_to;
    free_many = load_first_spectrum_many_strict;
    free_many_relative = load_first_spectrum_many_strict_relative_to;
    into_first = into_first_spectrum;
}

strict_first_methods! {
    what = "one-dimensional spectrum";
    output = Spectrum1D;
    reader = read_first_1d_strict;
    reader_relative = read_first_1d_strict_relative_to;
    reader_many = read_first_1d_many_strict;
    reader_many_relative = read_first_1d_many_strict_relative_to;
    free = load_first_spectrum_1d_strict;
    free_relative = load_first_spectrum_1d_strict_relative_to;
    free_many = load_first_spectrum_1d_many_strict;
    free_many_relative = load_first_spectrum_1d_many_strict_relative_to;
    into_first = into_first_1d;
}

strict_first_methods! {
    what = "one-dimensional spectrum with source metadata";
    output = (Spectrum1D, LoadedSource);
    reader = read_first_1d_with_source_strict;
    reader_relative = read_first_1d_with_source_strict_relative_to;
    reader_many = read_first_1d_many_with_source_strict;
    reader_many_relative = read_first_1d_many_with_source_strict_relative_to;
    free = load_first_spectrum_1d_with_source_strict;
    free_relative = load_first_spectrum_1d_with_source_strict_relative_to;
    free_many = load_first_spectrum_1d_many_with_source_strict;
    free_many_relative = load_first_spectrum_1d_many_with_source_strict_relative_to;
    into_first = into_first_loaded_1d;
}

strict_first_methods! {
    what = "two-dimensional spectrum";
    output = Spectrum2D;
    reader = read_first_2d_strict;
    reader_relative = read_first_2d_strict_relative_to;
    reader_many = read_first_2d_many_strict;
    reader_many_relative = read_first_2d_many_strict_relative_to;
    free = load_first_spectrum_2d_strict;
    free_relative = load_first_spectrum_2d_strict_relative_to;
    free_many = load_first_spectrum_2d_many_strict;
    free_many_relative = load_first_spectrum_2d_many_strict_relative_to;
    into_first = into_first_2d;
}

strict_first_methods! {
    what = "two-dimensional spectrum with source metadata";
    output = (Spectrum2D, LoadedSource);
    reader = read_first_2d_with_source_strict;
    reader_relative = read_first_2d_with_source_strict_relative_to;
    reader_many = read_first_2d_many_with_source_strict;
    reader_many_relative = read_first_2d_many_with_source_strict_relative_to;
    free = load_first_spectrum_2d_with_source_strict;
    free_relative = load_first_spectrum_2d_with_source_strict_relative_to;
    free_many = load_first_spectrum_2d_many_with_source_strict;
    free_many_relative = load_first_spectrum_2d_many_with_source_strict_relative_to;
    into_first = into_first_loaded_2d;
}
