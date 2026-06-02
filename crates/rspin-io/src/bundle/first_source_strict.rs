//! Strict source-filtered first-spectrum reader helpers.

use std::path::Path;

use rspin_core::{Result, Spectrum1D, Spectrum2D};

use super::{LoadedSource, LoadedSourceFilter, LoadedSpectrum, SpectrumBundleLoader};

macro_rules! strict_first_source_reader_methods {
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
        into_first = $into_first:ident;
    ) => {
        impl SpectrumBundleLoader {
            #[doc = concat!("Strictly loads a file or directory and returns the first ", $what, " matching a generic source filter.")]
            ///
            /// Source filtering is applied before candidate files are read, so
            /// unrelated malformed candidates do not fail the strict load.
            ///
            /// # Errors
            ///
            /// Returns an error when strict loading fails or no matching
            /// spectrum is found.
            pub fn $single(
                &self,
                path: impl AsRef<Path>,
                filter: impl Into<LoadedSourceFilter>,
            ) -> Result<$output> {
                self.read_strict_by_source(path, filter)?.$into_first()
            }

            #[doc = concat!("Strictly loads a file or directory and returns the first ", $what, " matching any generic source filter.")]
            ///
            /// Filters are combined with logical OR. Passing an empty iterator
            /// leaves source matching unrestricted.
            ///
            /// Source filtering is applied before candidate files are read, so
            /// unrelated malformed candidates do not fail the strict load.
            ///
            /// # Errors
            ///
            /// Returns an error when strict loading fails or no matching
            /// spectrum is found.
            pub fn $sources<I, F>(&self, path: impl AsRef<Path>, filters: I) -> Result<$output>
            where
                I: IntoIterator<Item = F>,
                F: Into<LoadedSourceFilter>,
            {
                self.read_strict_by_sources(path, filters)?.$into_first()
            }

            #[doc = concat!("Strictly loads one selected path relative to a base directory and returns the first ", $what, " matching a generic source filter.")]
            ///
            /// Source filtering is applied before candidate files are read, so
            /// unrelated malformed candidates do not fail the strict load.
            ///
            /// # Errors
            ///
            /// Returns an error when strict loading fails or no matching
            /// spectrum is found.
            pub fn $single_relative(
                &self,
                base: impl AsRef<Path>,
                path: impl AsRef<Path>,
                filter: impl Into<LoadedSourceFilter>,
            ) -> Result<$output> {
                self.read_strict_by_source_relative_to(base, path, filter)?
                    .$into_first()
            }

            #[doc = concat!("Strictly loads one selected path relative to a base directory and returns the first ", $what, " matching any generic source filter.")]
            ///
            /// Filters are combined with logical OR. Passing an empty iterator
            /// leaves source matching unrestricted.
            ///
            /// Source filtering is applied before candidate files are read, so
            /// unrelated malformed candidates do not fail the strict load.
            ///
            /// # Errors
            ///
            /// Returns an error when strict loading fails or no matching
            /// spectrum is found.
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
                self.read_strict_by_sources_relative_to(base, path, filters)?
                    .$into_first()
            }

            #[doc = concat!("Strictly loads selected paths and returns the first ", $what, " matching a generic source filter.")]
            ///
            /// Source filtering is applied before candidate files are read, so
            /// unrelated malformed candidates do not fail the strict load.
            ///
            /// # Errors
            ///
            /// Returns an error when strict loading fails or no matching
            /// spectrum is found.
            pub fn $many_single<I, P>(
                &self,
                paths: I,
                filter: impl Into<LoadedSourceFilter>,
            ) -> Result<$output>
            where
                I: IntoIterator<Item = P>,
                P: AsRef<Path>,
            {
                self.read_many_strict_by_source(paths, filter)?
                    .$into_first()
            }

            #[doc = concat!("Strictly loads selected paths and returns the first ", $what, " matching any generic source filter.")]
            ///
            /// Filters are combined with logical OR. Passing an empty iterator
            /// leaves source matching unrestricted.
            ///
            /// Source filtering is applied before candidate files are read, so
            /// unrelated malformed candidates do not fail the strict load.
            ///
            /// # Errors
            ///
            /// Returns an error when strict loading fails or no matching
            /// spectrum is found.
            pub fn $many_sources<I, P, J, F>(&self, paths: I, filters: J) -> Result<$output>
            where
                I: IntoIterator<Item = P>,
                P: AsRef<Path>,
                J: IntoIterator<Item = F>,
                F: Into<LoadedSourceFilter>,
            {
                self.read_many_strict_by_sources(paths, filters)?
                    .$into_first()
            }

            #[doc = concat!("Strictly loads selected paths relative to a base directory and returns the first ", $what, " matching a generic source filter.")]
            ///
            /// Source filtering is applied before candidate files are read, so
            /// unrelated malformed candidates do not fail the strict load.
            ///
            /// # Errors
            ///
            /// Returns an error when strict loading fails or no matching
            /// spectrum is found.
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
                self.read_many_strict_by_source_relative_to(base, paths, filter)?
                    .$into_first()
            }

            #[doc = concat!("Strictly loads selected paths relative to a base directory and returns the first ", $what, " matching any generic source filter.")]
            ///
            /// Filters are combined with logical OR. Passing an empty iterator
            /// leaves source matching unrestricted.
            ///
            /// Source filtering is applied before candidate files are read, so
            /// unrelated malformed candidates do not fail the strict load.
            ///
            /// # Errors
            ///
            /// Returns an error when strict loading fails or no matching
            /// spectrum is found.
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
                self.read_many_strict_by_sources_relative_to(base, paths, filters)?
                    .$into_first()
            }
        }
    };
}

macro_rules! strict_first_source_free_functions {
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
        #[doc = concat!("Strictly loads a file or directory and returns the first ", $what, " matching a generic source filter.")]
        ///
        /// Source filtering is applied before candidate files are read, so
        /// unrelated malformed candidates do not fail the strict load.
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading fails or no matching spectrum is found.
        pub fn $single(
            path: impl AsRef<Path>,
            filter: impl Into<LoadedSourceFilter>,
        ) -> Result<$output> {
            SpectrumBundleLoader::new().$reader_single(path, filter)
        }

        #[doc = concat!("Strictly loads a file or directory and returns the first ", $what, " matching any generic source filter.")]
        ///
        /// Filters are combined with logical OR. Passing an empty iterator
        /// leaves source matching unrestricted.
        ///
        /// Source filtering is applied before candidate files are read, so
        /// unrelated malformed candidates do not fail the strict load.
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading fails or no matching spectrum is found.
        pub fn $sources<I, F>(path: impl AsRef<Path>, filters: I) -> Result<$output>
        where
            I: IntoIterator<Item = F>,
            F: Into<LoadedSourceFilter>,
        {
            SpectrumBundleLoader::new().$reader_sources(path, filters)
        }

        #[doc = concat!("Strictly loads one selected path relative to a base directory and returns the first ", $what, " matching a generic source filter.")]
        ///
        /// Source filtering is applied before candidate files are read, so
        /// unrelated malformed candidates do not fail the strict load.
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading fails or no matching spectrum is found.
        pub fn $single_relative(
            base: impl AsRef<Path>,
            path: impl AsRef<Path>,
            filter: impl Into<LoadedSourceFilter>,
        ) -> Result<$output> {
            SpectrumBundleLoader::new().$reader_single_relative(base, path, filter)
        }

        #[doc = concat!("Strictly loads one selected path relative to a base directory and returns the first ", $what, " matching any generic source filter.")]
        ///
        /// Filters are combined with logical OR. Passing an empty iterator
        /// leaves source matching unrestricted.
        ///
        /// Source filtering is applied before candidate files are read, so
        /// unrelated malformed candidates do not fail the strict load.
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading fails or no matching spectrum is found.
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

        #[doc = concat!("Strictly loads selected paths and returns the first ", $what, " matching a generic source filter.")]
        ///
        /// Source filtering is applied before candidate files are read, so
        /// unrelated malformed candidates do not fail the strict load.
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading fails or no matching spectrum is found.
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

        #[doc = concat!("Strictly loads selected paths and returns the first ", $what, " matching any generic source filter.")]
        ///
        /// Filters are combined with logical OR. Passing an empty iterator
        /// leaves source matching unrestricted.
        ///
        /// Source filtering is applied before candidate files are read, so
        /// unrelated malformed candidates do not fail the strict load.
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading fails or no matching spectrum is found.
        pub fn $many_sources<I, P, J, F>(paths: I, filters: J) -> Result<$output>
        where
            I: IntoIterator<Item = P>,
            P: AsRef<Path>,
            J: IntoIterator<Item = F>,
            F: Into<LoadedSourceFilter>,
        {
            SpectrumBundleLoader::new().$reader_many_sources(paths, filters)
        }

        #[doc = concat!("Strictly loads selected paths relative to a base directory and returns the first ", $what, " matching a generic source filter.")]
        ///
        /// Source filtering is applied before candidate files are read, so
        /// unrelated malformed candidates do not fail the strict load.
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading fails or no matching spectrum is found.
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

        #[doc = concat!("Strictly loads selected paths relative to a base directory and returns the first ", $what, " matching any generic source filter.")]
        ///
        /// Filters are combined with logical OR. Passing an empty iterator
        /// leaves source matching unrestricted.
        ///
        /// Source filtering is applied before candidate files are read, so
        /// unrelated malformed candidates do not fail the strict load.
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading fails or no matching spectrum is found.
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

strict_first_source_reader_methods! {
    what = "spectrum";
    output = LoadedSpectrum;
    single = read_first_spectrum_strict_by_source;
    sources = read_first_spectrum_strict_by_sources;
    single_relative = read_first_spectrum_strict_by_source_relative_to;
    sources_relative = read_first_spectrum_strict_by_sources_relative_to;
    many_single = read_first_spectrum_many_strict_by_source;
    many_sources = read_first_spectrum_many_strict_by_sources;
    many_single_relative = read_first_spectrum_many_strict_by_source_relative_to;
    many_sources_relative = read_first_spectrum_many_strict_by_sources_relative_to;
    into_first = into_first_spectrum;
}

strict_first_source_reader_methods! {
    what = "one-dimensional spectrum";
    output = Spectrum1D;
    single = read_first_1d_strict_by_source;
    sources = read_first_1d_strict_by_sources;
    single_relative = read_first_1d_strict_by_source_relative_to;
    sources_relative = read_first_1d_strict_by_sources_relative_to;
    many_single = read_first_1d_many_strict_by_source;
    many_sources = read_first_1d_many_strict_by_sources;
    many_single_relative = read_first_1d_many_strict_by_source_relative_to;
    many_sources_relative = read_first_1d_many_strict_by_sources_relative_to;
    into_first = into_first_1d;
}

strict_first_source_reader_methods! {
    what = "one-dimensional spectrum with source metadata";
    output = (Spectrum1D, LoadedSource);
    single = read_first_1d_with_source_strict_by_source;
    sources = read_first_1d_with_source_strict_by_sources;
    single_relative = read_first_1d_with_source_strict_by_source_relative_to;
    sources_relative = read_first_1d_with_source_strict_by_sources_relative_to;
    many_single = read_first_1d_many_with_source_strict_by_source;
    many_sources = read_first_1d_many_with_source_strict_by_sources;
    many_single_relative = read_first_1d_many_with_source_strict_by_source_relative_to;
    many_sources_relative = read_first_1d_many_with_source_strict_by_sources_relative_to;
    into_first = into_first_loaded_1d;
}

strict_first_source_reader_methods! {
    what = "two-dimensional spectrum";
    output = Spectrum2D;
    single = read_first_2d_strict_by_source;
    sources = read_first_2d_strict_by_sources;
    single_relative = read_first_2d_strict_by_source_relative_to;
    sources_relative = read_first_2d_strict_by_sources_relative_to;
    many_single = read_first_2d_many_strict_by_source;
    many_sources = read_first_2d_many_strict_by_sources;
    many_single_relative = read_first_2d_many_strict_by_source_relative_to;
    many_sources_relative = read_first_2d_many_strict_by_sources_relative_to;
    into_first = into_first_2d;
}

strict_first_source_reader_methods! {
    what = "two-dimensional spectrum with source metadata";
    output = (Spectrum2D, LoadedSource);
    single = read_first_2d_with_source_strict_by_source;
    sources = read_first_2d_with_source_strict_by_sources;
    single_relative = read_first_2d_with_source_strict_by_source_relative_to;
    sources_relative = read_first_2d_with_source_strict_by_sources_relative_to;
    many_single = read_first_2d_many_with_source_strict_by_source;
    many_sources = read_first_2d_many_with_source_strict_by_sources;
    many_single_relative = read_first_2d_many_with_source_strict_by_source_relative_to;
    many_sources_relative = read_first_2d_many_with_source_strict_by_sources_relative_to;
    into_first = into_first_loaded_2d;
}

strict_first_source_free_functions! {
    what = "spectrum";
    output = LoadedSpectrum;
    single = load_first_spectrum_strict_by_source;
    sources = load_first_spectrum_strict_by_sources;
    single_relative = load_first_spectrum_strict_by_source_relative_to;
    sources_relative = load_first_spectrum_strict_by_sources_relative_to;
    many_single = load_first_spectrum_many_strict_by_source;
    many_sources = load_first_spectrum_many_strict_by_sources;
    many_single_relative = load_first_spectrum_many_strict_by_source_relative_to;
    many_sources_relative = load_first_spectrum_many_strict_by_sources_relative_to;
    reader_single = read_first_spectrum_strict_by_source;
    reader_sources = read_first_spectrum_strict_by_sources;
    reader_single_relative = read_first_spectrum_strict_by_source_relative_to;
    reader_sources_relative = read_first_spectrum_strict_by_sources_relative_to;
    reader_many_single = read_first_spectrum_many_strict_by_source;
    reader_many_sources = read_first_spectrum_many_strict_by_sources;
    reader_many_single_relative = read_first_spectrum_many_strict_by_source_relative_to;
    reader_many_sources_relative = read_first_spectrum_many_strict_by_sources_relative_to;
}

strict_first_source_free_functions! {
    what = "one-dimensional spectrum";
    output = Spectrum1D;
    single = load_first_spectrum_1d_strict_by_source;
    sources = load_first_spectrum_1d_strict_by_sources;
    single_relative = load_first_spectrum_1d_strict_by_source_relative_to;
    sources_relative = load_first_spectrum_1d_strict_by_sources_relative_to;
    many_single = load_first_spectrum_1d_many_strict_by_source;
    many_sources = load_first_spectrum_1d_many_strict_by_sources;
    many_single_relative = load_first_spectrum_1d_many_strict_by_source_relative_to;
    many_sources_relative = load_first_spectrum_1d_many_strict_by_sources_relative_to;
    reader_single = read_first_1d_strict_by_source;
    reader_sources = read_first_1d_strict_by_sources;
    reader_single_relative = read_first_1d_strict_by_source_relative_to;
    reader_sources_relative = read_first_1d_strict_by_sources_relative_to;
    reader_many_single = read_first_1d_many_strict_by_source;
    reader_many_sources = read_first_1d_many_strict_by_sources;
    reader_many_single_relative = read_first_1d_many_strict_by_source_relative_to;
    reader_many_sources_relative = read_first_1d_many_strict_by_sources_relative_to;
}

strict_first_source_free_functions! {
    what = "one-dimensional spectrum with source metadata";
    output = (Spectrum1D, LoadedSource);
    single = load_first_spectrum_1d_with_source_strict_by_source;
    sources = load_first_spectrum_1d_with_source_strict_by_sources;
    single_relative = load_first_spectrum_1d_with_source_strict_by_source_relative_to;
    sources_relative = load_first_spectrum_1d_with_source_strict_by_sources_relative_to;
    many_single = load_first_spectrum_1d_many_with_source_strict_by_source;
    many_sources = load_first_spectrum_1d_many_with_source_strict_by_sources;
    many_single_relative = load_first_spectrum_1d_many_with_source_strict_by_source_relative_to;
    many_sources_relative = load_first_spectrum_1d_many_with_source_strict_by_sources_relative_to;
    reader_single = read_first_1d_with_source_strict_by_source;
    reader_sources = read_first_1d_with_source_strict_by_sources;
    reader_single_relative = read_first_1d_with_source_strict_by_source_relative_to;
    reader_sources_relative = read_first_1d_with_source_strict_by_sources_relative_to;
    reader_many_single = read_first_1d_many_with_source_strict_by_source;
    reader_many_sources = read_first_1d_many_with_source_strict_by_sources;
    reader_many_single_relative = read_first_1d_many_with_source_strict_by_source_relative_to;
    reader_many_sources_relative = read_first_1d_many_with_source_strict_by_sources_relative_to;
}

strict_first_source_free_functions! {
    what = "two-dimensional spectrum";
    output = Spectrum2D;
    single = load_first_spectrum_2d_strict_by_source;
    sources = load_first_spectrum_2d_strict_by_sources;
    single_relative = load_first_spectrum_2d_strict_by_source_relative_to;
    sources_relative = load_first_spectrum_2d_strict_by_sources_relative_to;
    many_single = load_first_spectrum_2d_many_strict_by_source;
    many_sources = load_first_spectrum_2d_many_strict_by_sources;
    many_single_relative = load_first_spectrum_2d_many_strict_by_source_relative_to;
    many_sources_relative = load_first_spectrum_2d_many_strict_by_sources_relative_to;
    reader_single = read_first_2d_strict_by_source;
    reader_sources = read_first_2d_strict_by_sources;
    reader_single_relative = read_first_2d_strict_by_source_relative_to;
    reader_sources_relative = read_first_2d_strict_by_sources_relative_to;
    reader_many_single = read_first_2d_many_strict_by_source;
    reader_many_sources = read_first_2d_many_strict_by_sources;
    reader_many_single_relative = read_first_2d_many_strict_by_source_relative_to;
    reader_many_sources_relative = read_first_2d_many_strict_by_sources_relative_to;
}

strict_first_source_free_functions! {
    what = "two-dimensional spectrum with source metadata";
    output = (Spectrum2D, LoadedSource);
    single = load_first_spectrum_2d_with_source_strict_by_source;
    sources = load_first_spectrum_2d_with_source_strict_by_sources;
    single_relative = load_first_spectrum_2d_with_source_strict_by_source_relative_to;
    sources_relative = load_first_spectrum_2d_with_source_strict_by_sources_relative_to;
    many_single = load_first_spectrum_2d_many_with_source_strict_by_source;
    many_sources = load_first_spectrum_2d_many_with_source_strict_by_sources;
    many_single_relative = load_first_spectrum_2d_many_with_source_strict_by_source_relative_to;
    many_sources_relative = load_first_spectrum_2d_many_with_source_strict_by_sources_relative_to;
    reader_single = read_first_2d_with_source_strict_by_source;
    reader_sources = read_first_2d_with_source_strict_by_sources;
    reader_single_relative = read_first_2d_with_source_strict_by_source_relative_to;
    reader_sources_relative = read_first_2d_with_source_strict_by_sources_relative_to;
    reader_many_single = read_first_2d_many_with_source_strict_by_source;
    reader_many_sources = read_first_2d_many_with_source_strict_by_sources;
    reader_many_single_relative = read_first_2d_many_with_source_strict_by_source_relative_to;
    reader_many_sources_relative = read_first_2d_many_with_source_strict_by_sources_relative_to;
}
