//! Loading helpers for selected discovered spectrum sources.

use std::path::Path;

use rspin_core::Result;

use super::super::SpectrumBundleLoader;
use super::DiscoveredSpectrumSource;
use crate::bundle::{SpectrumBundle, SpectrumBundleSummary};

/// Slice extension methods for selected discovered spectrum source references.
///
/// These methods complete the common preflight workflow:
///
/// ```ignore
/// let sources = RSpinReader::new().discover("data")?;
/// let selected = sources.select_1d_by_source(LoadedSourceVendor::Jeol);
/// let bundle = selected.load_1d("data")?;
/// ```
pub trait SelectedDiscoveredSpectrumSourcesExt {
    /// Loads these selected discovered sources relative to a common base directory.
    ///
    /// # Errors
    ///
    /// Returns an error when the base directory is invalid, a selected source
    /// has no tracked path, strict loading would reject the source, or no data
    /// can be read.
    fn load_relative_to(&self, base: impl AsRef<Path>) -> Result<SpectrumBundle>;

    /// Loads these selected discovered sources relative to a common base directory.
    ///
    /// This short alias mirrors [`Self::load_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading the selected discovered sources fails.
    fn load(&self, base: impl AsRef<Path>) -> Result<SpectrumBundle>;

    /// Strictly loads these selected discovered sources relative to a common base directory.
    ///
    /// # Errors
    ///
    /// Returns an error when the base directory is invalid, a selected source
    /// has no tracked path, any selected source cannot be read, or no data can
    /// be read.
    fn load_strict_relative_to(&self, base: impl AsRef<Path>) -> Result<SpectrumBundle>;

    /// Strictly loads these selected discovered sources relative to a common base directory.
    ///
    /// This short alias mirrors [`Self::load_strict_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when strict loading the selected discovered sources fails.
    fn load_strict(&self, base: impl AsRef<Path>) -> Result<SpectrumBundle>;

    /// Loads these selected discovered sources and returns summary counts.
    ///
    /// # Errors
    ///
    /// Returns an error when loading the selected discovered sources fails.
    fn load_summary_relative_to(&self, base: impl AsRef<Path>) -> Result<SpectrumBundleSummary>;

    /// Loads these selected discovered sources and returns summary counts.
    ///
    /// This short alias mirrors [`Self::load_summary_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading the selected discovered sources fails.
    fn load_summary(&self, base: impl AsRef<Path>) -> Result<SpectrumBundleSummary>;

    /// Strictly loads these selected discovered sources and returns summary counts.
    ///
    /// # Errors
    ///
    /// Returns an error when strict loading the selected discovered sources fails.
    fn load_summary_strict_relative_to(
        &self,
        base: impl AsRef<Path>,
    ) -> Result<SpectrumBundleSummary>;

    /// Strictly loads these selected discovered sources and returns summary counts.
    ///
    /// This short alias mirrors [`Self::load_summary_strict_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when strict loading the selected discovered sources fails.
    fn load_summary_strict(&self, base: impl AsRef<Path>) -> Result<SpectrumBundleSummary>;

    /// Loads these selected discovered sources as a one-dimensional spectrum bundle.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails, no discovered sources are provided,
    /// no one-dimensional candidate is selected, or no one-dimensional data is
    /// found.
    fn load_1d_relative_to(&self, base: impl AsRef<Path>) -> Result<SpectrumBundle>;

    /// Loads these selected discovered sources as a one-dimensional spectrum bundle.
    ///
    /// This short alias mirrors [`Self::load_1d_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when one-dimensional bundle loading fails.
    fn load_1d(&self, base: impl AsRef<Path>) -> Result<SpectrumBundle>;

    /// Strictly loads these selected discovered sources as a one-dimensional spectrum bundle.
    ///
    /// # Errors
    ///
    /// Returns an error when strict one-dimensional bundle loading fails.
    fn load_1d_strict_relative_to(&self, base: impl AsRef<Path>) -> Result<SpectrumBundle>;

    /// Strictly loads these selected discovered sources as a one-dimensional spectrum bundle.
    ///
    /// This short alias mirrors [`Self::load_1d_strict_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when strict one-dimensional bundle loading fails.
    fn load_1d_strict(&self, base: impl AsRef<Path>) -> Result<SpectrumBundle>;

    /// Loads these selected discovered sources as a two-dimensional spectrum bundle.
    ///
    /// # Errors
    ///
    /// Returns an error when loading fails, no discovered sources are provided,
    /// no two-dimensional candidate is selected, or no two-dimensional data is
    /// found.
    fn load_2d_relative_to(&self, base: impl AsRef<Path>) -> Result<SpectrumBundle>;

    /// Loads these selected discovered sources as a two-dimensional spectrum bundle.
    ///
    /// This short alias mirrors [`Self::load_2d_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when two-dimensional bundle loading fails.
    fn load_2d(&self, base: impl AsRef<Path>) -> Result<SpectrumBundle>;

    /// Strictly loads these selected discovered sources as a two-dimensional spectrum bundle.
    ///
    /// # Errors
    ///
    /// Returns an error when strict two-dimensional bundle loading fails.
    fn load_2d_strict_relative_to(&self, base: impl AsRef<Path>) -> Result<SpectrumBundle>;

    /// Strictly loads these selected discovered sources as a two-dimensional spectrum bundle.
    ///
    /// This short alias mirrors [`Self::load_2d_strict_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when strict two-dimensional bundle loading fails.
    fn load_2d_strict(&self, base: impl AsRef<Path>) -> Result<SpectrumBundle>;
}

impl SelectedDiscoveredSpectrumSourcesExt for [&DiscoveredSpectrumSource] {
    fn load_relative_to(&self, base: impl AsRef<Path>) -> Result<SpectrumBundle> {
        SpectrumBundleLoader::new().read_discovered_relative_to(base, self.iter().copied())
    }

    fn load(&self, base: impl AsRef<Path>) -> Result<SpectrumBundle> {
        self.load_relative_to(base)
    }

    fn load_strict_relative_to(&self, base: impl AsRef<Path>) -> Result<SpectrumBundle> {
        SpectrumBundleLoader::new()
            .strict()
            .read_discovered_relative_to(base, self.iter().copied())
    }

    fn load_strict(&self, base: impl AsRef<Path>) -> Result<SpectrumBundle> {
        self.load_strict_relative_to(base)
    }

    fn load_summary_relative_to(&self, base: impl AsRef<Path>) -> Result<SpectrumBundleSummary> {
        SpectrumBundleLoader::new().read_discovered_summary_relative_to(base, self.iter().copied())
    }

    fn load_summary(&self, base: impl AsRef<Path>) -> Result<SpectrumBundleSummary> {
        self.load_summary_relative_to(base)
    }

    fn load_summary_strict_relative_to(
        &self,
        base: impl AsRef<Path>,
    ) -> Result<SpectrumBundleSummary> {
        SpectrumBundleLoader::new()
            .strict()
            .read_discovered_summary_relative_to(base, self.iter().copied())
    }

    fn load_summary_strict(&self, base: impl AsRef<Path>) -> Result<SpectrumBundleSummary> {
        self.load_summary_strict_relative_to(base)
    }

    fn load_1d_relative_to(&self, base: impl AsRef<Path>) -> Result<SpectrumBundle> {
        SpectrumBundleLoader::new()
            .read_discovered_bundle_1d_relative_to(base, self.iter().copied())
    }

    fn load_1d(&self, base: impl AsRef<Path>) -> Result<SpectrumBundle> {
        self.load_1d_relative_to(base)
    }

    fn load_1d_strict_relative_to(&self, base: impl AsRef<Path>) -> Result<SpectrumBundle> {
        SpectrumBundleLoader::new()
            .strict()
            .read_discovered_bundle_1d_relative_to(base, self.iter().copied())
    }

    fn load_1d_strict(&self, base: impl AsRef<Path>) -> Result<SpectrumBundle> {
        self.load_1d_strict_relative_to(base)
    }

    fn load_2d_relative_to(&self, base: impl AsRef<Path>) -> Result<SpectrumBundle> {
        SpectrumBundleLoader::new()
            .read_discovered_bundle_2d_relative_to(base, self.iter().copied())
    }

    fn load_2d(&self, base: impl AsRef<Path>) -> Result<SpectrumBundle> {
        self.load_2d_relative_to(base)
    }

    fn load_2d_strict_relative_to(&self, base: impl AsRef<Path>) -> Result<SpectrumBundle> {
        SpectrumBundleLoader::new()
            .strict()
            .read_discovered_bundle_2d_relative_to(base, self.iter().copied())
    }

    fn load_2d_strict(&self, base: impl AsRef<Path>) -> Result<SpectrumBundle> {
        self.load_2d_strict_relative_to(base)
    }
}
