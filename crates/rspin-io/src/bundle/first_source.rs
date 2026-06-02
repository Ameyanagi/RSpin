//! Generic source-filtered first-spectrum reader helpers.

use std::path::Path;

use rspin_core::{Result, Spectrum1D, Spectrum2D};

use super::{LoadedSource, LoadedSourceFilter, LoadedSpectrum, SpectrumBundleLoader};

macro_rules! first_reader_methods {
    (
        what = $what:literal;
        output = $output:ty;
        single = $single:ident;
        sources = $sources:ident;
        single_relative = $single_relative:ident;
        sources_relative = $sources_relative:ident;
        many_single = $many_single:ident;
        many_sources = $many_sources:ident;
        many_single_relative = $many_single_relative:ident;
        many_sources_relative = $many_sources_relative:ident;
        into_single = $into_single:ident;
        into_sources = $into_sources:ident;
    ) => {
        impl SpectrumBundleLoader {
            #[doc = concat!("Loads a file or directory and returns the first ", $what, " matching a generic source filter.")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum is found.
            pub fn $single(
                &self,
                path: impl AsRef<Path>,
                filter: impl Into<LoadedSourceFilter>,
            ) -> Result<$output> {
                self.read_path(path)?.$into_single(filter)
            }

            #[doc = concat!("Loads a file or directory and returns the first ", $what, " matching any generic source filter.")]
            ///
            /// Filters are combined with logical OR. Passing an empty iterator
            /// leaves source matching unrestricted.
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum is found.
            pub fn $sources<I, F>(&self, path: impl AsRef<Path>, filters: I) -> Result<$output>
            where
                I: IntoIterator<Item = F>,
                F: Into<LoadedSourceFilter>,
            {
                self.read_path(path)?.$into_sources(filters)
            }

            #[doc = concat!("Loads one selected path relative to a base directory and returns the first ", $what, " matching a generic source filter.")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum is found.
            pub fn $single_relative(
                &self,
                base: impl AsRef<Path>,
                path: impl AsRef<Path>,
                filter: impl Into<LoadedSourceFilter>,
            ) -> Result<$output> {
                self.read_path_relative_to(base, path)?.$into_single(filter)
            }

            #[doc = concat!("Loads one selected path relative to a base directory and returns the first ", $what, " matching any generic source filter.")]
            ///
            /// Filters are combined with logical OR. Passing an empty iterator
            /// leaves source matching unrestricted.
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum is found.
            pub fn $sources_relative<I, F>(
                &self,
                base: impl AsRef<Path>,
                path: impl AsRef<Path>,
                filters: I,
            ) -> Result<$output>
            where
                I: IntoIterator<Item = F>,
                F: Into<LoadedSourceFilter>,
            {
                self.read_path_relative_to(base, path)?
                    .$into_sources(filters)
            }

            #[doc = concat!("Loads selected paths and returns the first ", $what, " matching a generic source filter.")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum is found.
            pub fn $many_single<I, P>(
                &self,
                paths: I,
                filter: impl Into<LoadedSourceFilter>,
            ) -> Result<$output>
            where
                I: IntoIterator<Item = P>,
                P: AsRef<Path>,
            {
                self.read_paths(paths)?.$into_single(filter)
            }

            #[doc = concat!("Loads selected paths and returns the first ", $what, " matching any generic source filter.")]
            ///
            /// Filters are combined with logical OR. Passing an empty iterator
            /// leaves source matching unrestricted.
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum is found.
            pub fn $many_sources<I, P, J, F>(&self, paths: I, filters: J) -> Result<$output>
            where
                I: IntoIterator<Item = P>,
                P: AsRef<Path>,
                J: IntoIterator<Item = F>,
                F: Into<LoadedSourceFilter>,
            {
                self.read_paths(paths)?.$into_sources(filters)
            }

            #[doc = concat!("Loads selected paths relative to a base directory and returns the first ", $what, " matching a generic source filter.")]
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum is found.
            pub fn $many_single_relative<I, P>(
                &self,
                base: impl AsRef<Path>,
                paths: I,
                filter: impl Into<LoadedSourceFilter>,
            ) -> Result<$output>
            where
                I: IntoIterator<Item = P>,
                P: AsRef<Path>,
            {
                self.read_paths_relative_to(base, paths)?
                    .$into_single(filter)
            }

            #[doc = concat!("Loads selected paths relative to a base directory and returns the first ", $what, " matching any generic source filter.")]
            ///
            /// Filters are combined with logical OR. Passing an empty iterator
            /// leaves source matching unrestricted.
            ///
            /// # Errors
            ///
            /// Returns an error when loading fails or no matching spectrum is found.
            pub fn $many_sources_relative<I, P, J, F>(
                &self,
                base: impl AsRef<Path>,
                paths: I,
                filters: J,
            ) -> Result<$output>
            where
                I: IntoIterator<Item = P>,
                P: AsRef<Path>,
                J: IntoIterator<Item = F>,
                F: Into<LoadedSourceFilter>,
            {
                self.read_paths_relative_to(base, paths)?
                    .$into_sources(filters)
            }
        }
    };
}

macro_rules! first_free_functions {
    (
        what = $what:literal;
        output = $output:ty;
        single = $single:ident;
        sources = $sources:ident;
        single_relative = $single_relative:ident;
        sources_relative = $sources_relative:ident;
        many_single = $many_single:ident;
        many_sources = $many_sources:ident;
        many_single_relative = $many_single_relative:ident;
        many_sources_relative = $many_sources_relative:ident;
        reader_single = $reader_single:ident;
        reader_sources = $reader_sources:ident;
        reader_single_relative = $reader_single_relative:ident;
        reader_sources_relative = $reader_sources_relative:ident;
        reader_many_single = $reader_many_single:ident;
        reader_many_sources = $reader_many_sources:ident;
        reader_many_single_relative = $reader_many_single_relative:ident;
        reader_many_sources_relative = $reader_many_sources_relative:ident;
    ) => {
        #[doc = concat!("Loads a file or directory and returns the first ", $what, " matching a generic source filter.")]
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum is found.
        pub fn $single(
            path: impl AsRef<Path>,
            filter: impl Into<LoadedSourceFilter>,
        ) -> Result<$output> {
            SpectrumBundleLoader::new().$reader_single(path, filter)
        }

        #[doc = concat!("Loads a file or directory and returns the first ", $what, " matching any generic source filter.")]
        ///
        /// Filters are combined with logical OR. Passing an empty iterator
        /// leaves source matching unrestricted.
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum is found.
        pub fn $sources<I, F>(path: impl AsRef<Path>, filters: I) -> Result<$output>
        where
            I: IntoIterator<Item = F>,
            F: Into<LoadedSourceFilter>,
        {
            SpectrumBundleLoader::new().$reader_sources(path, filters)
        }

        #[doc = concat!("Loads one selected path relative to a base directory and returns the first ", $what, " matching a generic source filter.")]
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum is found.
        pub fn $single_relative(
            base: impl AsRef<Path>,
            path: impl AsRef<Path>,
            filter: impl Into<LoadedSourceFilter>,
        ) -> Result<$output> {
            SpectrumBundleLoader::new().$reader_single_relative(base, path, filter)
        }

        #[doc = concat!("Loads one selected path relative to a base directory and returns the first ", $what, " matching any generic source filter.")]
        ///
        /// Filters are combined with logical OR. Passing an empty iterator
        /// leaves source matching unrestricted.
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum is found.
        pub fn $sources_relative<I, F>(
            base: impl AsRef<Path>,
            path: impl AsRef<Path>,
            filters: I,
        ) -> Result<$output>
        where
            I: IntoIterator<Item = F>,
            F: Into<LoadedSourceFilter>,
        {
            SpectrumBundleLoader::new().$reader_sources_relative(base, path, filters)
        }

        #[doc = concat!("Loads selected paths and returns the first ", $what, " matching a generic source filter.")]
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum is found.
        pub fn $many_single<I, P>(
            paths: I,
            filter: impl Into<LoadedSourceFilter>,
        ) -> Result<$output>
        where
            I: IntoIterator<Item = P>,
            P: AsRef<Path>,
        {
            SpectrumBundleLoader::new().$reader_many_single(paths, filter)
        }

        #[doc = concat!("Loads selected paths and returns the first ", $what, " matching any generic source filter.")]
        ///
        /// Filters are combined with logical OR. Passing an empty iterator
        /// leaves source matching unrestricted.
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum is found.
        pub fn $many_sources<I, P, J, F>(paths: I, filters: J) -> Result<$output>
        where
            I: IntoIterator<Item = P>,
            P: AsRef<Path>,
            J: IntoIterator<Item = F>,
            F: Into<LoadedSourceFilter>,
        {
            SpectrumBundleLoader::new().$reader_many_sources(paths, filters)
        }

        #[doc = concat!("Loads selected paths relative to a base directory and returns the first ", $what, " matching a generic source filter.")]
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum is found.
        pub fn $many_single_relative<I, P>(
            base: impl AsRef<Path>,
            paths: I,
            filter: impl Into<LoadedSourceFilter>,
        ) -> Result<$output>
        where
            I: IntoIterator<Item = P>,
            P: AsRef<Path>,
        {
            SpectrumBundleLoader::new().$reader_many_single_relative(base, paths, filter)
        }

        #[doc = concat!("Loads selected paths relative to a base directory and returns the first ", $what, " matching any generic source filter.")]
        ///
        /// Filters are combined with logical OR. Passing an empty iterator
        /// leaves source matching unrestricted.
        ///
        /// # Errors
        ///
        /// Returns an error when loading fails or no matching spectrum is found.
        pub fn $many_sources_relative<I, P, J, F>(
            base: impl AsRef<Path>,
            paths: I,
            filters: J,
        ) -> Result<$output>
        where
            I: IntoIterator<Item = P>,
            P: AsRef<Path>,
            J: IntoIterator<Item = F>,
            F: Into<LoadedSourceFilter>,
        {
            SpectrumBundleLoader::new().$reader_many_sources_relative(base, paths, filters)
        }
    };
}

first_reader_methods! {
    what = "spectrum";
    output = LoadedSpectrum;
    single = read_first_spectrum_by_source;
    sources = read_first_spectrum_by_sources;
    single_relative = read_first_spectrum_by_source_relative_to;
    sources_relative = read_first_spectrum_by_sources_relative_to;
    many_single = read_first_spectrum_many_by_source;
    many_sources = read_first_spectrum_many_by_sources;
    many_single_relative = read_first_spectrum_many_by_source_relative_to;
    many_sources_relative = read_first_spectrum_many_by_sources_relative_to;
    into_single = into_first_by_source;
    into_sources = into_first_by_sources;
}

first_reader_methods! {
    what = "one-dimensional spectrum";
    output = Spectrum1D;
    single = read_first_1d_by_source;
    sources = read_first_1d_by_sources;
    single_relative = read_first_1d_by_source_relative_to;
    sources_relative = read_first_1d_by_sources_relative_to;
    many_single = read_first_1d_many_by_source;
    many_sources = read_first_1d_many_by_sources;
    many_single_relative = read_first_1d_many_by_source_relative_to;
    many_sources_relative = read_first_1d_many_by_sources_relative_to;
    into_single = into_first_1d_by_source;
    into_sources = into_first_1d_by_sources;
}

first_reader_methods! {
    what = "one-dimensional spectrum with source metadata";
    output = (Spectrum1D, LoadedSource);
    single = read_first_1d_with_source_by_source;
    sources = read_first_1d_with_source_by_sources;
    single_relative = read_first_1d_with_source_by_source_relative_to;
    sources_relative = read_first_1d_with_source_by_sources_relative_to;
    many_single = read_first_1d_many_with_source_by_source;
    many_sources = read_first_1d_many_with_source_by_sources;
    many_single_relative = read_first_1d_many_with_source_by_source_relative_to;
    many_sources_relative = read_first_1d_many_with_source_by_sources_relative_to;
    into_single = into_first_loaded_1d_by_source;
    into_sources = into_first_loaded_1d_by_sources;
}

first_reader_methods! {
    what = "two-dimensional spectrum";
    output = Spectrum2D;
    single = read_first_2d_by_source;
    sources = read_first_2d_by_sources;
    single_relative = read_first_2d_by_source_relative_to;
    sources_relative = read_first_2d_by_sources_relative_to;
    many_single = read_first_2d_many_by_source;
    many_sources = read_first_2d_many_by_sources;
    many_single_relative = read_first_2d_many_by_source_relative_to;
    many_sources_relative = read_first_2d_many_by_sources_relative_to;
    into_single = into_first_2d_by_source;
    into_sources = into_first_2d_by_sources;
}

first_reader_methods! {
    what = "two-dimensional spectrum with source metadata";
    output = (Spectrum2D, LoadedSource);
    single = read_first_2d_with_source_by_source;
    sources = read_first_2d_with_source_by_sources;
    single_relative = read_first_2d_with_source_by_source_relative_to;
    sources_relative = read_first_2d_with_source_by_sources_relative_to;
    many_single = read_first_2d_many_with_source_by_source;
    many_sources = read_first_2d_many_with_source_by_sources;
    many_single_relative = read_first_2d_many_with_source_by_source_relative_to;
    many_sources_relative = read_first_2d_many_with_source_by_sources_relative_to;
    into_single = into_first_loaded_2d_by_source;
    into_sources = into_first_loaded_2d_by_sources;
}

first_free_functions! {
    what = "spectrum";
    output = LoadedSpectrum;
    single = load_first_spectrum_by_source;
    sources = load_first_spectrum_by_sources;
    single_relative = load_first_spectrum_by_source_relative_to;
    sources_relative = load_first_spectrum_by_sources_relative_to;
    many_single = load_first_spectrum_many_by_source;
    many_sources = load_first_spectrum_many_by_sources;
    many_single_relative = load_first_spectrum_many_by_source_relative_to;
    many_sources_relative = load_first_spectrum_many_by_sources_relative_to;
    reader_single = read_first_spectrum_by_source;
    reader_sources = read_first_spectrum_by_sources;
    reader_single_relative = read_first_spectrum_by_source_relative_to;
    reader_sources_relative = read_first_spectrum_by_sources_relative_to;
    reader_many_single = read_first_spectrum_many_by_source;
    reader_many_sources = read_first_spectrum_many_by_sources;
    reader_many_single_relative = read_first_spectrum_many_by_source_relative_to;
    reader_many_sources_relative = read_first_spectrum_many_by_sources_relative_to;
}

first_free_functions! {
    what = "one-dimensional spectrum";
    output = Spectrum1D;
    single = load_first_spectrum_1d_by_source;
    sources = load_first_spectrum_1d_by_sources;
    single_relative = load_first_spectrum_1d_by_source_relative_to;
    sources_relative = load_first_spectrum_1d_by_sources_relative_to;
    many_single = load_first_spectrum_1d_many_by_source;
    many_sources = load_first_spectrum_1d_many_by_sources;
    many_single_relative = load_first_spectrum_1d_many_by_source_relative_to;
    many_sources_relative = load_first_spectrum_1d_many_by_sources_relative_to;
    reader_single = read_first_1d_by_source;
    reader_sources = read_first_1d_by_sources;
    reader_single_relative = read_first_1d_by_source_relative_to;
    reader_sources_relative = read_first_1d_by_sources_relative_to;
    reader_many_single = read_first_1d_many_by_source;
    reader_many_sources = read_first_1d_many_by_sources;
    reader_many_single_relative = read_first_1d_many_by_source_relative_to;
    reader_many_sources_relative = read_first_1d_many_by_sources_relative_to;
}

first_free_functions! {
    what = "one-dimensional spectrum with source metadata";
    output = (Spectrum1D, LoadedSource);
    single = load_first_spectrum_1d_with_source_by_source;
    sources = load_first_spectrum_1d_with_source_by_sources;
    single_relative = load_first_spectrum_1d_with_source_by_source_relative_to;
    sources_relative = load_first_spectrum_1d_with_source_by_sources_relative_to;
    many_single = load_first_spectrum_1d_many_with_source_by_source;
    many_sources = load_first_spectrum_1d_many_with_source_by_sources;
    many_single_relative = load_first_spectrum_1d_many_with_source_by_source_relative_to;
    many_sources_relative = load_first_spectrum_1d_many_with_source_by_sources_relative_to;
    reader_single = read_first_1d_with_source_by_source;
    reader_sources = read_first_1d_with_source_by_sources;
    reader_single_relative = read_first_1d_with_source_by_source_relative_to;
    reader_sources_relative = read_first_1d_with_source_by_sources_relative_to;
    reader_many_single = read_first_1d_many_with_source_by_source;
    reader_many_sources = read_first_1d_many_with_source_by_sources;
    reader_many_single_relative = read_first_1d_many_with_source_by_source_relative_to;
    reader_many_sources_relative = read_first_1d_many_with_source_by_sources_relative_to;
}

first_free_functions! {
    what = "two-dimensional spectrum";
    output = Spectrum2D;
    single = load_first_spectrum_2d_by_source;
    sources = load_first_spectrum_2d_by_sources;
    single_relative = load_first_spectrum_2d_by_source_relative_to;
    sources_relative = load_first_spectrum_2d_by_sources_relative_to;
    many_single = load_first_spectrum_2d_many_by_source;
    many_sources = load_first_spectrum_2d_many_by_sources;
    many_single_relative = load_first_spectrum_2d_many_by_source_relative_to;
    many_sources_relative = load_first_spectrum_2d_many_by_sources_relative_to;
    reader_single = read_first_2d_by_source;
    reader_sources = read_first_2d_by_sources;
    reader_single_relative = read_first_2d_by_source_relative_to;
    reader_sources_relative = read_first_2d_by_sources_relative_to;
    reader_many_single = read_first_2d_many_by_source;
    reader_many_sources = read_first_2d_many_by_sources;
    reader_many_single_relative = read_first_2d_many_by_source_relative_to;
    reader_many_sources_relative = read_first_2d_many_by_sources_relative_to;
}

first_free_functions! {
    what = "two-dimensional spectrum with source metadata";
    output = (Spectrum2D, LoadedSource);
    single = load_first_spectrum_2d_with_source_by_source;
    sources = load_first_spectrum_2d_with_source_by_sources;
    single_relative = load_first_spectrum_2d_with_source_by_source_relative_to;
    sources_relative = load_first_spectrum_2d_with_source_by_sources_relative_to;
    many_single = load_first_spectrum_2d_many_with_source_by_source;
    many_sources = load_first_spectrum_2d_many_with_source_by_sources;
    many_single_relative = load_first_spectrum_2d_many_with_source_by_source_relative_to;
    many_sources_relative = load_first_spectrum_2d_many_with_source_by_sources_relative_to;
    reader_single = read_first_2d_with_source_by_source;
    reader_sources = read_first_2d_with_source_by_sources;
    reader_single_relative = read_first_2d_with_source_by_source_relative_to;
    reader_sources_relative = read_first_2d_with_source_by_sources_relative_to;
    reader_many_single = read_first_2d_many_with_source_by_source;
    reader_many_sources = read_first_2d_many_with_source_by_sources;
    reader_many_single_relative = read_first_2d_many_with_source_by_source_relative_to;
    reader_many_sources_relative = read_first_2d_many_with_source_by_sources_relative_to;
}
