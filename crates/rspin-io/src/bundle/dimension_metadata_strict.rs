//! Strict dimension-specific bundle loading helpers for source metadata filters.

use std::path::{Path, PathBuf};

use rspin_core::Result;

use super::{LoadedSourceDataKind, SpectrumBundle, SpectrumBundleLoader, SpectrumBundleSummary};

fn discovered_base_for_path(path: &Path) -> PathBuf {
    if !path.is_file() {
        return path.to_path_buf();
    }

    let Some(parent) = path.parent() else {
        return path.to_path_buf();
    };

    if parent.as_os_str().is_empty() {
        return PathBuf::from(".");
    }

    parent.to_path_buf()
}

macro_rules! strict_dimension_metadata_helpers {
    (
        dimension = $dimension:literal;
        dimension_filter = $dimension_filter:ident;
        discovered_reader = $discovered_reader:ident;

        reader_format = $reader_format:ident;
        reader_formats = $reader_formats:ident;
        reader_vendor = $reader_vendor:ident;
        reader_vendors = $reader_vendors:ident;
        reader_data_kind = $reader_data_kind:ident;
        reader_data_kinds = $reader_data_kinds:ident;

        reader_format_relative = $reader_format_relative:ident;
        reader_formats_relative = $reader_formats_relative:ident;
        reader_vendor_relative = $reader_vendor_relative:ident;
        reader_vendors_relative = $reader_vendors_relative:ident;
        reader_data_kind_relative = $reader_data_kind_relative:ident;
        reader_data_kinds_relative = $reader_data_kinds_relative:ident;

        reader_summary_format = $reader_summary_format:ident;
        reader_summary_formats = $reader_summary_formats:ident;
        reader_summary_vendor = $reader_summary_vendor:ident;
        reader_summary_vendors = $reader_summary_vendors:ident;
        reader_summary_data_kind = $reader_summary_data_kind:ident;
        reader_summary_data_kinds = $reader_summary_data_kinds:ident;

        reader_summary_format_relative = $reader_summary_format_relative:ident;
        reader_summary_formats_relative = $reader_summary_formats_relative:ident;
        reader_summary_vendor_relative = $reader_summary_vendor_relative:ident;
        reader_summary_vendors_relative = $reader_summary_vendors_relative:ident;
        reader_summary_data_kind_relative = $reader_summary_data_kind_relative:ident;
        reader_summary_data_kinds_relative = $reader_summary_data_kinds_relative:ident;

        load_format = $load_format:ident;
        load_formats = $load_formats:ident;
        load_vendor = $load_vendor:ident;
        load_vendors = $load_vendors:ident;
        load_data_kind = $load_data_kind:ident;
        load_data_kinds = $load_data_kinds:ident;

        load_format_relative = $load_format_relative:ident;
        load_formats_relative = $load_formats_relative:ident;
        load_vendor_relative = $load_vendor_relative:ident;
        load_vendors_relative = $load_vendors_relative:ident;
        load_data_kind_relative = $load_data_kind_relative:ident;
        load_data_kinds_relative = $load_data_kinds_relative:ident;

        load_summary_format = $load_summary_format:ident;
        load_summary_formats = $load_summary_formats:ident;
        load_summary_vendor = $load_summary_vendor:ident;
        load_summary_vendors = $load_summary_vendors:ident;
        load_summary_data_kind = $load_summary_data_kind:ident;
        load_summary_data_kinds = $load_summary_data_kinds:ident;

        load_summary_format_relative = $load_summary_format_relative:ident;
        load_summary_formats_relative = $load_summary_formats_relative:ident;
        load_summary_vendor_relative = $load_summary_vendor_relative:ident;
        load_summary_vendors_relative = $load_summary_vendors_relative:ident;
        load_summary_data_kind_relative = $load_summary_data_kind_relative:ident;
        load_summary_data_kinds_relative = $load_summary_data_kinds_relative:ident;
    ) => {
        impl SpectrumBundleLoader {
            #[doc = concat!("Strictly loads all ", $dimension, " spectra from a file or directory, restricted to one source format.")]
            ///
            /// # Errors
            ///
            /// Returns an error when strict loading fails or no matching spectrum data is found.
            pub fn $reader_format(
                &self,
                path: impl AsRef<Path>,
                format: impl AsRef<str>,
            ) -> Result<SpectrumBundle> {
                let path = path.as_ref();
                let base = discovered_base_for_path(path);
                let loader = self
                    .clone()
                    .strict()
                    .$dimension_filter()
                    .only_source_format(format);
                let sources = loader.discover_path(path)?;
                loader.$discovered_reader(base, &sources)
            }

            #[doc = concat!("Strictly loads all ", $dimension, " spectra from a file or directory, restricted to source formats.")]
            ///
            /// Formats are combined with logical OR. Passing an empty iterator
            /// leaves source loading unrestricted.
            ///
            /// # Errors
            ///
            /// Returns an error when strict loading fails or no matching spectrum data is found.
            pub fn $reader_formats<I, F>(
                &self,
                path: impl AsRef<Path>,
                formats: I,
            ) -> Result<SpectrumBundle>
            where
                I: IntoIterator<Item = F>,
                F: AsRef<str>,
            {
                let path = path.as_ref();
                let base = discovered_base_for_path(path);
                let loader = self
                    .clone()
                    .strict()
                    .$dimension_filter()
                    .only_source_formats(formats);
                let sources = loader.discover_path(path)?;
                loader.$discovered_reader(base, &sources)
            }

            #[doc = concat!("Strictly loads all ", $dimension, " spectra from a file or directory, restricted to one vendor family.")]
            ///
            /// # Errors
            ///
            /// Returns an error when strict loading fails or no matching spectrum data is found.
            pub fn $reader_vendor(
                &self,
                path: impl AsRef<Path>,
                vendor: impl AsRef<str>,
            ) -> Result<SpectrumBundle> {
                let path = path.as_ref();
                let base = discovered_base_for_path(path);
                let loader = self
                    .clone()
                    .strict()
                    .$dimension_filter()
                    .only_source_vendor(vendor);
                let sources = loader.discover_path(path)?;
                loader.$discovered_reader(base, &sources)
            }

            #[doc = concat!("Strictly loads all ", $dimension, " spectra from a file or directory, restricted to vendor families.")]
            ///
            /// Vendors are combined with logical OR. Passing an empty iterator
            /// leaves source loading unrestricted.
            ///
            /// # Errors
            ///
            /// Returns an error when strict loading fails or no matching spectrum data is found.
            pub fn $reader_vendors<I, V>(
                &self,
                path: impl AsRef<Path>,
                vendors: I,
            ) -> Result<SpectrumBundle>
            where
                I: IntoIterator<Item = V>,
                V: AsRef<str>,
            {
                let path = path.as_ref();
                let base = discovered_base_for_path(path);
                let loader = self
                    .clone()
                    .strict()
                    .$dimension_filter()
                    .only_source_vendors(vendors);
                let sources = loader.discover_path(path)?;
                loader.$discovered_reader(base, &sources)
            }

            #[doc = concat!("Strictly loads all ", $dimension, " spectra from a file or directory, restricted to one source data kind.")]
            ///
            /// # Errors
            ///
            /// Returns an error when strict loading fails or no matching spectrum data is found.
            pub fn $reader_data_kind(
                &self,
                path: impl AsRef<Path>,
                data_kind: LoadedSourceDataKind,
            ) -> Result<SpectrumBundle> {
                let path = path.as_ref();
                let base = discovered_base_for_path(path);
                let loader = self
                    .clone()
                    .strict()
                    .$dimension_filter()
                    .only_source_data_kind(data_kind);
                let sources = loader.discover_path(path)?;
                loader.$discovered_reader(base, &sources)
            }

            #[doc = concat!("Strictly loads all ", $dimension, " spectra from a file or directory, restricted to source data kinds.")]
            ///
            /// Data kinds are combined with logical OR. Passing an empty
            /// iterator leaves source loading unrestricted.
            ///
            /// # Errors
            ///
            /// Returns an error when strict loading fails or no matching spectrum data is found.
            pub fn $reader_data_kinds<I>(
                &self,
                path: impl AsRef<Path>,
                data_kinds: I,
            ) -> Result<SpectrumBundle>
            where
                I: IntoIterator<Item = LoadedSourceDataKind>,
            {
                let path = path.as_ref();
                let base = discovered_base_for_path(path);
                let loader = self
                    .clone()
                    .strict()
                    .$dimension_filter()
                    .only_source_data_kinds(data_kinds);
                let sources = loader.discover_path(path)?;
                loader.$discovered_reader(base, &sources)
            }

            #[doc = concat!("Strictly loads all ", $dimension, " spectra from one selected path relative to a base directory, restricted to one source format.")]
            ///
            /// # Errors
            ///
            /// Returns an error when strict loading fails or no matching spectrum data is found.
            pub fn $reader_format_relative(
                &self,
                base: impl AsRef<Path>,
                path: impl AsRef<Path>,
                format: impl AsRef<str>,
            ) -> Result<SpectrumBundle> {
                let base = base.as_ref();
                let loader = self
                    .clone()
                    .strict()
                    .$dimension_filter()
                    .only_source_format(format);
                let sources = loader.discover_path_relative_to(base, path)?;
                loader.$discovered_reader(base, &sources)
            }

            #[doc = concat!("Strictly loads all ", $dimension, " spectra from one selected path relative to a base directory, restricted to source formats.")]
            ///
            /// # Errors
            ///
            /// Returns an error when strict loading fails or no matching spectrum data is found.
            pub fn $reader_formats_relative<I, F>(
                &self,
                base: impl AsRef<Path>,
                path: impl AsRef<Path>,
                formats: I,
            ) -> Result<SpectrumBundle>
            where
                I: IntoIterator<Item = F>,
                F: AsRef<str>,
            {
                let base = base.as_ref();
                let loader = self
                    .clone()
                    .strict()
                    .$dimension_filter()
                    .only_source_formats(formats);
                let sources = loader.discover_path_relative_to(base, path)?;
                loader.$discovered_reader(base, &sources)
            }

            #[doc = concat!("Strictly loads all ", $dimension, " spectra from one selected path relative to a base directory, restricted to one vendor family.")]
            ///
            /// # Errors
            ///
            /// Returns an error when strict loading fails or no matching spectrum data is found.
            pub fn $reader_vendor_relative(
                &self,
                base: impl AsRef<Path>,
                path: impl AsRef<Path>,
                vendor: impl AsRef<str>,
            ) -> Result<SpectrumBundle> {
                let base = base.as_ref();
                let loader = self
                    .clone()
                    .strict()
                    .$dimension_filter()
                    .only_source_vendor(vendor);
                let sources = loader.discover_path_relative_to(base, path)?;
                loader.$discovered_reader(base, &sources)
            }

            #[doc = concat!("Strictly loads all ", $dimension, " spectra from one selected path relative to a base directory, restricted to vendor families.")]
            ///
            /// # Errors
            ///
            /// Returns an error when strict loading fails or no matching spectrum data is found.
            pub fn $reader_vendors_relative<I, V>(
                &self,
                base: impl AsRef<Path>,
                path: impl AsRef<Path>,
                vendors: I,
            ) -> Result<SpectrumBundle>
            where
                I: IntoIterator<Item = V>,
                V: AsRef<str>,
            {
                let base = base.as_ref();
                let loader = self
                    .clone()
                    .strict()
                    .$dimension_filter()
                    .only_source_vendors(vendors);
                let sources = loader.discover_path_relative_to(base, path)?;
                loader.$discovered_reader(base, &sources)
            }

            #[doc = concat!("Strictly loads all ", $dimension, " spectra from one selected path relative to a base directory, restricted to one source data kind.")]
            ///
            /// # Errors
            ///
            /// Returns an error when strict loading fails or no matching spectrum data is found.
            pub fn $reader_data_kind_relative(
                &self,
                base: impl AsRef<Path>,
                path: impl AsRef<Path>,
                data_kind: LoadedSourceDataKind,
            ) -> Result<SpectrumBundle> {
                let base = base.as_ref();
                let loader = self
                    .clone()
                    .strict()
                    .$dimension_filter()
                    .only_source_data_kind(data_kind);
                let sources = loader.discover_path_relative_to(base, path)?;
                loader.$discovered_reader(base, &sources)
            }

            #[doc = concat!("Strictly loads all ", $dimension, " spectra from one selected path relative to a base directory, restricted to source data kinds.")]
            ///
            /// # Errors
            ///
            /// Returns an error when strict loading fails or no matching spectrum data is found.
            pub fn $reader_data_kinds_relative<I>(
                &self,
                base: impl AsRef<Path>,
                path: impl AsRef<Path>,
                data_kinds: I,
            ) -> Result<SpectrumBundle>
            where
                I: IntoIterator<Item = LoadedSourceDataKind>,
            {
                let base = base.as_ref();
                let loader = self
                    .clone()
                    .strict()
                    .$dimension_filter()
                    .only_source_data_kinds(data_kinds);
                let sources = loader.discover_path_relative_to(base, path)?;
                loader.$discovered_reader(base, &sources)
            }

            #[doc = concat!("Strictly loads all ", $dimension, " spectra from a file or directory by source format and returns summary counts.")]
            ///
            /// # Errors
            ///
            /// Returns an error when strict loading fails or no matching spectrum data is found.
            pub fn $reader_summary_format(
                &self,
                path: impl AsRef<Path>,
                format: impl AsRef<str>,
            ) -> Result<SpectrumBundleSummary> {
                self.$reader_format(path, format).map(|bundle| bundle.summary())
            }

            #[doc = concat!("Strictly loads all ", $dimension, " spectra from a file or directory by source formats and returns summary counts.")]
            ///
            /// # Errors
            ///
            /// Returns an error when strict loading fails or no matching spectrum data is found.
            pub fn $reader_summary_formats<I, F>(
                &self,
                path: impl AsRef<Path>,
                formats: I,
            ) -> Result<SpectrumBundleSummary>
            where
                I: IntoIterator<Item = F>,
                F: AsRef<str>,
            {
                self.$reader_formats(path, formats)
                    .map(|bundle| bundle.summary())
            }

            #[doc = concat!("Strictly loads all ", $dimension, " spectra from a file or directory by vendor family and returns summary counts.")]
            ///
            /// # Errors
            ///
            /// Returns an error when strict loading fails or no matching spectrum data is found.
            pub fn $reader_summary_vendor(
                &self,
                path: impl AsRef<Path>,
                vendor: impl AsRef<str>,
            ) -> Result<SpectrumBundleSummary> {
                self.$reader_vendor(path, vendor).map(|bundle| bundle.summary())
            }

            #[doc = concat!("Strictly loads all ", $dimension, " spectra from a file or directory by vendor families and returns summary counts.")]
            ///
            /// # Errors
            ///
            /// Returns an error when strict loading fails or no matching spectrum data is found.
            pub fn $reader_summary_vendors<I, V>(
                &self,
                path: impl AsRef<Path>,
                vendors: I,
            ) -> Result<SpectrumBundleSummary>
            where
                I: IntoIterator<Item = V>,
                V: AsRef<str>,
            {
                self.$reader_vendors(path, vendors)
                    .map(|bundle| bundle.summary())
            }

            #[doc = concat!("Strictly loads all ", $dimension, " spectra from a file or directory by source data kind and returns summary counts.")]
            ///
            /// # Errors
            ///
            /// Returns an error when strict loading fails or no matching spectrum data is found.
            pub fn $reader_summary_data_kind(
                &self,
                path: impl AsRef<Path>,
                data_kind: LoadedSourceDataKind,
            ) -> Result<SpectrumBundleSummary> {
                self.$reader_data_kind(path, data_kind)
                    .map(|bundle| bundle.summary())
            }

            #[doc = concat!("Strictly loads all ", $dimension, " spectra from a file or directory by source data kinds and returns summary counts.")]
            ///
            /// # Errors
            ///
            /// Returns an error when strict loading fails or no matching spectrum data is found.
            pub fn $reader_summary_data_kinds<I>(
                &self,
                path: impl AsRef<Path>,
                data_kinds: I,
            ) -> Result<SpectrumBundleSummary>
            where
                I: IntoIterator<Item = LoadedSourceDataKind>,
            {
                self.$reader_data_kinds(path, data_kinds)
                    .map(|bundle| bundle.summary())
            }

            #[doc = concat!("Strictly loads all ", $dimension, " spectra from one selected path relative to a base directory by source format and returns summary counts.")]
            ///
            /// # Errors
            ///
            /// Returns an error when strict loading fails or no matching spectrum data is found.
            pub fn $reader_summary_format_relative(
                &self,
                base: impl AsRef<Path>,
                path: impl AsRef<Path>,
                format: impl AsRef<str>,
            ) -> Result<SpectrumBundleSummary> {
                self.$reader_format_relative(base, path, format)
                    .map(|bundle| bundle.summary())
            }

            #[doc = concat!("Strictly loads all ", $dimension, " spectra from one selected path relative to a base directory by source formats and returns summary counts.")]
            ///
            /// # Errors
            ///
            /// Returns an error when strict loading fails or no matching spectrum data is found.
            pub fn $reader_summary_formats_relative<I, F>(
                &self,
                base: impl AsRef<Path>,
                path: impl AsRef<Path>,
                formats: I,
            ) -> Result<SpectrumBundleSummary>
            where
                I: IntoIterator<Item = F>,
                F: AsRef<str>,
            {
                self.$reader_formats_relative(base, path, formats)
                    .map(|bundle| bundle.summary())
            }

            #[doc = concat!("Strictly loads all ", $dimension, " spectra from one selected path relative to a base directory by vendor family and returns summary counts.")]
            ///
            /// # Errors
            ///
            /// Returns an error when strict loading fails or no matching spectrum data is found.
            pub fn $reader_summary_vendor_relative(
                &self,
                base: impl AsRef<Path>,
                path: impl AsRef<Path>,
                vendor: impl AsRef<str>,
            ) -> Result<SpectrumBundleSummary> {
                self.$reader_vendor_relative(base, path, vendor)
                    .map(|bundle| bundle.summary())
            }

            #[doc = concat!("Strictly loads all ", $dimension, " spectra from one selected path relative to a base directory by vendor families and returns summary counts.")]
            ///
            /// # Errors
            ///
            /// Returns an error when strict loading fails or no matching spectrum data is found.
            pub fn $reader_summary_vendors_relative<I, V>(
                &self,
                base: impl AsRef<Path>,
                path: impl AsRef<Path>,
                vendors: I,
            ) -> Result<SpectrumBundleSummary>
            where
                I: IntoIterator<Item = V>,
                V: AsRef<str>,
            {
                self.$reader_vendors_relative(base, path, vendors)
                    .map(|bundle| bundle.summary())
            }

            #[doc = concat!("Strictly loads all ", $dimension, " spectra from one selected path relative to a base directory by source data kind and returns summary counts.")]
            ///
            /// # Errors
            ///
            /// Returns an error when strict loading fails or no matching spectrum data is found.
            pub fn $reader_summary_data_kind_relative(
                &self,
                base: impl AsRef<Path>,
                path: impl AsRef<Path>,
                data_kind: LoadedSourceDataKind,
            ) -> Result<SpectrumBundleSummary> {
                self.$reader_data_kind_relative(base, path, data_kind)
                    .map(|bundle| bundle.summary())
            }

            #[doc = concat!("Strictly loads all ", $dimension, " spectra from one selected path relative to a base directory by source data kinds and returns summary counts.")]
            ///
            /// # Errors
            ///
            /// Returns an error when strict loading fails or no matching spectrum data is found.
            pub fn $reader_summary_data_kinds_relative<I>(
                &self,
                base: impl AsRef<Path>,
                path: impl AsRef<Path>,
                data_kinds: I,
            ) -> Result<SpectrumBundleSummary>
            where
                I: IntoIterator<Item = LoadedSourceDataKind>,
            {
                self.$reader_data_kinds_relative(base, path, data_kinds)
                    .map(|bundle| bundle.summary())
            }
        }

        #[doc = concat!("Strictly loads all ", $dimension, " spectra from a file or directory, restricted to one source format.")]
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading fails or no matching spectrum data is found.
        pub fn $load_format(
            path: impl AsRef<Path>,
            format: impl AsRef<str>,
        ) -> Result<SpectrumBundle> {
            SpectrumBundleLoader::new().$reader_format(path, format)
        }

        #[doc = concat!("Strictly loads all ", $dimension, " spectra from a file or directory, restricted to source formats.")]
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading fails or no matching spectrum data is found.
        pub fn $load_formats<I, F>(path: impl AsRef<Path>, formats: I) -> Result<SpectrumBundle>
        where
            I: IntoIterator<Item = F>,
            F: AsRef<str>,
        {
            SpectrumBundleLoader::new().$reader_formats(path, formats)
        }

        #[doc = concat!("Strictly loads all ", $dimension, " spectra from a file or directory, restricted to one vendor family.")]
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading fails or no matching spectrum data is found.
        pub fn $load_vendor(
            path: impl AsRef<Path>,
            vendor: impl AsRef<str>,
        ) -> Result<SpectrumBundle> {
            SpectrumBundleLoader::new().$reader_vendor(path, vendor)
        }

        #[doc = concat!("Strictly loads all ", $dimension, " spectra from a file or directory, restricted to vendor families.")]
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading fails or no matching spectrum data is found.
        pub fn $load_vendors<I, V>(path: impl AsRef<Path>, vendors: I) -> Result<SpectrumBundle>
        where
            I: IntoIterator<Item = V>,
            V: AsRef<str>,
        {
            SpectrumBundleLoader::new().$reader_vendors(path, vendors)
        }

        #[doc = concat!("Strictly loads all ", $dimension, " spectra from a file or directory, restricted to one source data kind.")]
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading fails or no matching spectrum data is found.
        pub fn $load_data_kind(
            path: impl AsRef<Path>,
            data_kind: LoadedSourceDataKind,
        ) -> Result<SpectrumBundle> {
            SpectrumBundleLoader::new().$reader_data_kind(path, data_kind)
        }

        #[doc = concat!("Strictly loads all ", $dimension, " spectra from a file or directory, restricted to source data kinds.")]
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading fails or no matching spectrum data is found.
        pub fn $load_data_kinds<I>(
            path: impl AsRef<Path>,
            data_kinds: I,
        ) -> Result<SpectrumBundle>
        where
            I: IntoIterator<Item = LoadedSourceDataKind>,
        {
            SpectrumBundleLoader::new().$reader_data_kinds(path, data_kinds)
        }

        #[doc = concat!("Strictly loads all ", $dimension, " spectra from one selected path relative to a base directory, restricted to one source format.")]
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading fails or no matching spectrum data is found.
        pub fn $load_format_relative(
            base: impl AsRef<Path>,
            path: impl AsRef<Path>,
            format: impl AsRef<str>,
        ) -> Result<SpectrumBundle> {
            SpectrumBundleLoader::new().$reader_format_relative(base, path, format)
        }

        #[doc = concat!("Strictly loads all ", $dimension, " spectra from one selected path relative to a base directory, restricted to source formats.")]
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading fails or no matching spectrum data is found.
        pub fn $load_formats_relative<I, F>(
            base: impl AsRef<Path>,
            path: impl AsRef<Path>,
            formats: I,
        ) -> Result<SpectrumBundle>
        where
            I: IntoIterator<Item = F>,
            F: AsRef<str>,
        {
            SpectrumBundleLoader::new().$reader_formats_relative(base, path, formats)
        }

        #[doc = concat!("Strictly loads all ", $dimension, " spectra from one selected path relative to a base directory, restricted to one vendor family.")]
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading fails or no matching spectrum data is found.
        pub fn $load_vendor_relative(
            base: impl AsRef<Path>,
            path: impl AsRef<Path>,
            vendor: impl AsRef<str>,
        ) -> Result<SpectrumBundle> {
            SpectrumBundleLoader::new().$reader_vendor_relative(base, path, vendor)
        }

        #[doc = concat!("Strictly loads all ", $dimension, " spectra from one selected path relative to a base directory, restricted to vendor families.")]
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading fails or no matching spectrum data is found.
        pub fn $load_vendors_relative<I, V>(
            base: impl AsRef<Path>,
            path: impl AsRef<Path>,
            vendors: I,
        ) -> Result<SpectrumBundle>
        where
            I: IntoIterator<Item = V>,
            V: AsRef<str>,
        {
            SpectrumBundleLoader::new().$reader_vendors_relative(base, path, vendors)
        }

        #[doc = concat!("Strictly loads all ", $dimension, " spectra from one selected path relative to a base directory, restricted to one source data kind.")]
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading fails or no matching spectrum data is found.
        pub fn $load_data_kind_relative(
            base: impl AsRef<Path>,
            path: impl AsRef<Path>,
            data_kind: LoadedSourceDataKind,
        ) -> Result<SpectrumBundle> {
            SpectrumBundleLoader::new().$reader_data_kind_relative(base, path, data_kind)
        }

        #[doc = concat!("Strictly loads all ", $dimension, " spectra from one selected path relative to a base directory, restricted to source data kinds.")]
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading fails or no matching spectrum data is found.
        pub fn $load_data_kinds_relative<I>(
            base: impl AsRef<Path>,
            path: impl AsRef<Path>,
            data_kinds: I,
        ) -> Result<SpectrumBundle>
        where
            I: IntoIterator<Item = LoadedSourceDataKind>,
        {
            SpectrumBundleLoader::new().$reader_data_kinds_relative(base, path, data_kinds)
        }

        #[doc = concat!("Strictly loads all ", $dimension, " spectra by source format and returns summary counts.")]
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading fails or no matching spectrum data is found.
        pub fn $load_summary_format(
            path: impl AsRef<Path>,
            format: impl AsRef<str>,
        ) -> Result<SpectrumBundleSummary> {
            SpectrumBundleLoader::new().$reader_summary_format(path, format)
        }

        #[doc = concat!("Strictly loads all ", $dimension, " spectra by source formats and returns summary counts.")]
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading fails or no matching spectrum data is found.
        pub fn $load_summary_formats<I, F>(
            path: impl AsRef<Path>,
            formats: I,
        ) -> Result<SpectrumBundleSummary>
        where
            I: IntoIterator<Item = F>,
            F: AsRef<str>,
        {
            SpectrumBundleLoader::new().$reader_summary_formats(path, formats)
        }

        #[doc = concat!("Strictly loads all ", $dimension, " spectra by vendor family and returns summary counts.")]
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading fails or no matching spectrum data is found.
        pub fn $load_summary_vendor(
            path: impl AsRef<Path>,
            vendor: impl AsRef<str>,
        ) -> Result<SpectrumBundleSummary> {
            SpectrumBundleLoader::new().$reader_summary_vendor(path, vendor)
        }

        #[doc = concat!("Strictly loads all ", $dimension, " spectra by vendor families and returns summary counts.")]
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading fails or no matching spectrum data is found.
        pub fn $load_summary_vendors<I, V>(
            path: impl AsRef<Path>,
            vendors: I,
        ) -> Result<SpectrumBundleSummary>
        where
            I: IntoIterator<Item = V>,
            V: AsRef<str>,
        {
            SpectrumBundleLoader::new().$reader_summary_vendors(path, vendors)
        }

        #[doc = concat!("Strictly loads all ", $dimension, " spectra by source data kind and returns summary counts.")]
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading fails or no matching spectrum data is found.
        pub fn $load_summary_data_kind(
            path: impl AsRef<Path>,
            data_kind: LoadedSourceDataKind,
        ) -> Result<SpectrumBundleSummary> {
            SpectrumBundleLoader::new().$reader_summary_data_kind(path, data_kind)
        }

        #[doc = concat!("Strictly loads all ", $dimension, " spectra by source data kinds and returns summary counts.")]
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading fails or no matching spectrum data is found.
        pub fn $load_summary_data_kinds<I>(
            path: impl AsRef<Path>,
            data_kinds: I,
        ) -> Result<SpectrumBundleSummary>
        where
            I: IntoIterator<Item = LoadedSourceDataKind>,
        {
            SpectrumBundleLoader::new().$reader_summary_data_kinds(path, data_kinds)
        }

        #[doc = concat!("Strictly loads all ", $dimension, " spectra from one selected path relative to a base directory by source format and returns summary counts.")]
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading fails or no matching spectrum data is found.
        pub fn $load_summary_format_relative(
            base: impl AsRef<Path>,
            path: impl AsRef<Path>,
            format: impl AsRef<str>,
        ) -> Result<SpectrumBundleSummary> {
            SpectrumBundleLoader::new().$reader_summary_format_relative(base, path, format)
        }

        #[doc = concat!("Strictly loads all ", $dimension, " spectra from one selected path relative to a base directory by source formats and returns summary counts.")]
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading fails or no matching spectrum data is found.
        pub fn $load_summary_formats_relative<I, F>(
            base: impl AsRef<Path>,
            path: impl AsRef<Path>,
            formats: I,
        ) -> Result<SpectrumBundleSummary>
        where
            I: IntoIterator<Item = F>,
            F: AsRef<str>,
        {
            SpectrumBundleLoader::new().$reader_summary_formats_relative(base, path, formats)
        }

        #[doc = concat!("Strictly loads all ", $dimension, " spectra from one selected path relative to a base directory by vendor family and returns summary counts.")]
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading fails or no matching spectrum data is found.
        pub fn $load_summary_vendor_relative(
            base: impl AsRef<Path>,
            path: impl AsRef<Path>,
            vendor: impl AsRef<str>,
        ) -> Result<SpectrumBundleSummary> {
            SpectrumBundleLoader::new().$reader_summary_vendor_relative(base, path, vendor)
        }

        #[doc = concat!("Strictly loads all ", $dimension, " spectra from one selected path relative to a base directory by vendor families and returns summary counts.")]
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading fails or no matching spectrum data is found.
        pub fn $load_summary_vendors_relative<I, V>(
            base: impl AsRef<Path>,
            path: impl AsRef<Path>,
            vendors: I,
        ) -> Result<SpectrumBundleSummary>
        where
            I: IntoIterator<Item = V>,
            V: AsRef<str>,
        {
            SpectrumBundleLoader::new().$reader_summary_vendors_relative(base, path, vendors)
        }

        #[doc = concat!("Strictly loads all ", $dimension, " spectra from one selected path relative to a base directory by source data kind and returns summary counts.")]
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading fails or no matching spectrum data is found.
        pub fn $load_summary_data_kind_relative(
            base: impl AsRef<Path>,
            path: impl AsRef<Path>,
            data_kind: LoadedSourceDataKind,
        ) -> Result<SpectrumBundleSummary> {
            SpectrumBundleLoader::new().$reader_summary_data_kind_relative(base, path, data_kind)
        }

        #[doc = concat!("Strictly loads all ", $dimension, " spectra from one selected path relative to a base directory by source data kinds and returns summary counts.")]
        ///
        /// # Errors
        ///
        /// Returns an error when strict loading fails or no matching spectrum data is found.
        pub fn $load_summary_data_kinds_relative<I>(
            base: impl AsRef<Path>,
            path: impl AsRef<Path>,
            data_kinds: I,
        ) -> Result<SpectrumBundleSummary>
        where
            I: IntoIterator<Item = LoadedSourceDataKind>,
        {
            SpectrumBundleLoader::new().$reader_summary_data_kinds_relative(base, path, data_kinds)
        }
    };
}

strict_dimension_metadata_helpers! {
    dimension = "one-dimensional";
    dimension_filter = one_d_only;
    discovered_reader = read_discovered_bundle_1d_relative_to;

    reader_format = read_bundle_1d_strict_by_source_format;
    reader_formats = read_bundle_1d_strict_by_source_formats;
    reader_vendor = read_bundle_1d_strict_by_source_vendor;
    reader_vendors = read_bundle_1d_strict_by_source_vendors;
    reader_data_kind = read_bundle_1d_strict_by_source_data_kind;
    reader_data_kinds = read_bundle_1d_strict_by_source_data_kinds;

    reader_format_relative = read_bundle_1d_strict_by_source_format_relative_to;
    reader_formats_relative = read_bundle_1d_strict_by_source_formats_relative_to;
    reader_vendor_relative = read_bundle_1d_strict_by_source_vendor_relative_to;
    reader_vendors_relative = read_bundle_1d_strict_by_source_vendors_relative_to;
    reader_data_kind_relative = read_bundle_1d_strict_by_source_data_kind_relative_to;
    reader_data_kinds_relative = read_bundle_1d_strict_by_source_data_kinds_relative_to;

    reader_summary_format = read_bundle_1d_summary_strict_by_source_format;
    reader_summary_formats = read_bundle_1d_summary_strict_by_source_formats;
    reader_summary_vendor = read_bundle_1d_summary_strict_by_source_vendor;
    reader_summary_vendors = read_bundle_1d_summary_strict_by_source_vendors;
    reader_summary_data_kind = read_bundle_1d_summary_strict_by_source_data_kind;
    reader_summary_data_kinds = read_bundle_1d_summary_strict_by_source_data_kinds;

    reader_summary_format_relative = read_bundle_1d_summary_strict_by_source_format_relative_to;
    reader_summary_formats_relative = read_bundle_1d_summary_strict_by_source_formats_relative_to;
    reader_summary_vendor_relative = read_bundle_1d_summary_strict_by_source_vendor_relative_to;
    reader_summary_vendors_relative = read_bundle_1d_summary_strict_by_source_vendors_relative_to;
    reader_summary_data_kind_relative = read_bundle_1d_summary_strict_by_source_data_kind_relative_to;
    reader_summary_data_kinds_relative = read_bundle_1d_summary_strict_by_source_data_kinds_relative_to;

    load_format = load_spectra_1d_strict_by_source_format;
    load_formats = load_spectra_1d_strict_by_source_formats;
    load_vendor = load_spectra_1d_strict_by_source_vendor;
    load_vendors = load_spectra_1d_strict_by_source_vendors;
    load_data_kind = load_spectra_1d_strict_by_source_data_kind;
    load_data_kinds = load_spectra_1d_strict_by_source_data_kinds;

    load_format_relative = load_spectra_1d_strict_by_source_format_relative_to;
    load_formats_relative = load_spectra_1d_strict_by_source_formats_relative_to;
    load_vendor_relative = load_spectra_1d_strict_by_source_vendor_relative_to;
    load_vendors_relative = load_spectra_1d_strict_by_source_vendors_relative_to;
    load_data_kind_relative = load_spectra_1d_strict_by_source_data_kind_relative_to;
    load_data_kinds_relative = load_spectra_1d_strict_by_source_data_kinds_relative_to;

    load_summary_format = load_spectra_1d_summary_strict_by_source_format;
    load_summary_formats = load_spectra_1d_summary_strict_by_source_formats;
    load_summary_vendor = load_spectra_1d_summary_strict_by_source_vendor;
    load_summary_vendors = load_spectra_1d_summary_strict_by_source_vendors;
    load_summary_data_kind = load_spectra_1d_summary_strict_by_source_data_kind;
    load_summary_data_kinds = load_spectra_1d_summary_strict_by_source_data_kinds;

    load_summary_format_relative = load_spectra_1d_summary_strict_by_source_format_relative_to;
    load_summary_formats_relative = load_spectra_1d_summary_strict_by_source_formats_relative_to;
    load_summary_vendor_relative = load_spectra_1d_summary_strict_by_source_vendor_relative_to;
    load_summary_vendors_relative = load_spectra_1d_summary_strict_by_source_vendors_relative_to;
    load_summary_data_kind_relative = load_spectra_1d_summary_strict_by_source_data_kind_relative_to;
    load_summary_data_kinds_relative = load_spectra_1d_summary_strict_by_source_data_kinds_relative_to;
}

strict_dimension_metadata_helpers! {
    dimension = "two-dimensional";
    dimension_filter = two_d_only;
    discovered_reader = read_discovered_bundle_2d_relative_to;

    reader_format = read_bundle_2d_strict_by_source_format;
    reader_formats = read_bundle_2d_strict_by_source_formats;
    reader_vendor = read_bundle_2d_strict_by_source_vendor;
    reader_vendors = read_bundle_2d_strict_by_source_vendors;
    reader_data_kind = read_bundle_2d_strict_by_source_data_kind;
    reader_data_kinds = read_bundle_2d_strict_by_source_data_kinds;

    reader_format_relative = read_bundle_2d_strict_by_source_format_relative_to;
    reader_formats_relative = read_bundle_2d_strict_by_source_formats_relative_to;
    reader_vendor_relative = read_bundle_2d_strict_by_source_vendor_relative_to;
    reader_vendors_relative = read_bundle_2d_strict_by_source_vendors_relative_to;
    reader_data_kind_relative = read_bundle_2d_strict_by_source_data_kind_relative_to;
    reader_data_kinds_relative = read_bundle_2d_strict_by_source_data_kinds_relative_to;

    reader_summary_format = read_bundle_2d_summary_strict_by_source_format;
    reader_summary_formats = read_bundle_2d_summary_strict_by_source_formats;
    reader_summary_vendor = read_bundle_2d_summary_strict_by_source_vendor;
    reader_summary_vendors = read_bundle_2d_summary_strict_by_source_vendors;
    reader_summary_data_kind = read_bundle_2d_summary_strict_by_source_data_kind;
    reader_summary_data_kinds = read_bundle_2d_summary_strict_by_source_data_kinds;

    reader_summary_format_relative = read_bundle_2d_summary_strict_by_source_format_relative_to;
    reader_summary_formats_relative = read_bundle_2d_summary_strict_by_source_formats_relative_to;
    reader_summary_vendor_relative = read_bundle_2d_summary_strict_by_source_vendor_relative_to;
    reader_summary_vendors_relative = read_bundle_2d_summary_strict_by_source_vendors_relative_to;
    reader_summary_data_kind_relative = read_bundle_2d_summary_strict_by_source_data_kind_relative_to;
    reader_summary_data_kinds_relative = read_bundle_2d_summary_strict_by_source_data_kinds_relative_to;

    load_format = load_spectra_2d_strict_by_source_format;
    load_formats = load_spectra_2d_strict_by_source_formats;
    load_vendor = load_spectra_2d_strict_by_source_vendor;
    load_vendors = load_spectra_2d_strict_by_source_vendors;
    load_data_kind = load_spectra_2d_strict_by_source_data_kind;
    load_data_kinds = load_spectra_2d_strict_by_source_data_kinds;

    load_format_relative = load_spectra_2d_strict_by_source_format_relative_to;
    load_formats_relative = load_spectra_2d_strict_by_source_formats_relative_to;
    load_vendor_relative = load_spectra_2d_strict_by_source_vendor_relative_to;
    load_vendors_relative = load_spectra_2d_strict_by_source_vendors_relative_to;
    load_data_kind_relative = load_spectra_2d_strict_by_source_data_kind_relative_to;
    load_data_kinds_relative = load_spectra_2d_strict_by_source_data_kinds_relative_to;

    load_summary_format = load_spectra_2d_summary_strict_by_source_format;
    load_summary_formats = load_spectra_2d_summary_strict_by_source_formats;
    load_summary_vendor = load_spectra_2d_summary_strict_by_source_vendor;
    load_summary_vendors = load_spectra_2d_summary_strict_by_source_vendors;
    load_summary_data_kind = load_spectra_2d_summary_strict_by_source_data_kind;
    load_summary_data_kinds = load_spectra_2d_summary_strict_by_source_data_kinds;

    load_summary_format_relative = load_spectra_2d_summary_strict_by_source_format_relative_to;
    load_summary_formats_relative = load_spectra_2d_summary_strict_by_source_formats_relative_to;
    load_summary_vendor_relative = load_spectra_2d_summary_strict_by_source_vendor_relative_to;
    load_summary_vendors_relative = load_spectra_2d_summary_strict_by_source_vendors_relative_to;
    load_summary_data_kind_relative = load_spectra_2d_summary_strict_by_source_data_kind_relative_to;
    load_summary_data_kinds_relative = load_spectra_2d_summary_strict_by_source_data_kinds_relative_to;
}
