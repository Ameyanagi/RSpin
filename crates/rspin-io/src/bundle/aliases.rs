//! Short aliases for common bundle loading workflows.

use std::path::Path;

use rspin_core::Result;

use super::{
    SpectrumBundle, SpectrumBundleSummary, load_spectra, load_spectra_many,
    load_spectra_many_relative_to, load_spectra_many_strict, load_spectra_many_strict_relative_to,
    load_spectra_many_summary, load_spectra_many_summary_relative_to,
    load_spectra_many_summary_strict, load_spectra_many_summary_strict_relative_to,
    load_spectra_relative_to, load_spectra_strict, load_spectra_strict_relative_to,
    load_spectra_summary, load_spectra_summary_relative_to, load_spectra_summary_strict,
    load_spectra_summary_strict_relative_to,
};

/// Loads all supported spectrum bundle data from a file or directory path.
///
/// This short alias mirrors [`load_spectra`].
///
/// # Errors
///
/// Returns an error when [`load_spectra`] would return an error.
pub fn load(path: impl AsRef<Path>) -> Result<SpectrumBundle> {
    load_spectra(path)
}

/// Loads all supported spectrum bundle data from a file or directory path in strict mode.
///
/// This short alias mirrors [`load_spectra_strict`].
///
/// # Errors
///
/// Returns an error when [`load_spectra_strict`] would return an error.
pub fn load_strict(path: impl AsRef<Path>) -> Result<SpectrumBundle> {
    load_spectra_strict(path)
}

/// Loads one selected path while anchoring source paths to a common base directory.
///
/// This short alias mirrors [`load_spectra_relative_to`].
///
/// # Errors
///
/// Returns an error when [`load_spectra_relative_to`] would return an error.
pub fn load_relative_to(base: impl AsRef<Path>, path: impl AsRef<Path>) -> Result<SpectrumBundle> {
    load_spectra_relative_to(base, path)
}

/// Strictly loads one selected path while anchoring source paths to a common base directory.
///
/// This short alias mirrors [`load_spectra_strict_relative_to`].
///
/// # Errors
///
/// Returns an error when [`load_spectra_strict_relative_to`] would return an error.
pub fn load_strict_relative_to(
    base: impl AsRef<Path>,
    path: impl AsRef<Path>,
) -> Result<SpectrumBundle> {
    load_spectra_strict_relative_to(base, path)
}

/// Loads all supported spectrum bundle data and returns only summary counts.
///
/// This short alias mirrors [`load_spectra_summary`].
///
/// # Errors
///
/// Returns an error when [`load_spectra_summary`] would return an error.
pub fn load_summary(path: impl AsRef<Path>) -> Result<SpectrumBundleSummary> {
    load_spectra_summary(path)
}

/// Strictly loads all supported spectrum bundle data and returns only summary counts.
///
/// This short alias mirrors [`load_spectra_summary_strict`].
///
/// # Errors
///
/// Returns an error when [`load_spectra_summary_strict`] would return an error.
pub fn load_summary_strict(path: impl AsRef<Path>) -> Result<SpectrumBundleSummary> {
    load_spectra_summary_strict(path)
}

/// Loads one selected path relative to a base directory and returns summary counts.
///
/// This short alias mirrors [`load_spectra_summary_relative_to`].
///
/// # Errors
///
/// Returns an error when [`load_spectra_summary_relative_to`] would return an error.
pub fn load_summary_relative_to(
    base: impl AsRef<Path>,
    path: impl AsRef<Path>,
) -> Result<SpectrumBundleSummary> {
    load_spectra_summary_relative_to(base, path)
}

/// Strictly loads one selected path relative to a base directory and returns summary counts.
///
/// This short alias mirrors [`load_spectra_summary_strict_relative_to`].
///
/// # Errors
///
/// Returns an error when [`load_spectra_summary_strict_relative_to`] would return an error.
pub fn load_summary_strict_relative_to(
    base: impl AsRef<Path>,
    path: impl AsRef<Path>,
) -> Result<SpectrumBundleSummary> {
    load_spectra_summary_strict_relative_to(base, path)
}

/// Loads supported spectra from multiple file or directory paths as one bundle.
///
/// This short alias mirrors [`load_spectra_many`].
///
/// # Errors
///
/// Returns an error when [`load_spectra_many`] would return an error.
pub fn load_many<I, P>(paths: I) -> Result<SpectrumBundle>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    load_spectra_many(paths)
}

/// Strictly loads supported spectra from multiple paths as one bundle.
///
/// This short alias mirrors [`load_spectra_many_strict`].
///
/// # Errors
///
/// Returns an error when [`load_spectra_many_strict`] would return an error.
pub fn load_many_strict<I, P>(paths: I) -> Result<SpectrumBundle>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    load_spectra_many_strict(paths)
}

/// Loads selected paths relative to a base directory as one bundle.
///
/// This short alias mirrors [`load_spectra_many_relative_to`].
///
/// # Errors
///
/// Returns an error when [`load_spectra_many_relative_to`] would return an error.
pub fn load_many_relative_to<I, P>(base: impl AsRef<Path>, paths: I) -> Result<SpectrumBundle>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    load_spectra_many_relative_to(base, paths)
}

/// Strictly loads selected paths relative to a base directory as one bundle.
///
/// This short alias mirrors [`load_spectra_many_strict_relative_to`].
///
/// # Errors
///
/// Returns an error when [`load_spectra_many_strict_relative_to`] would return an error.
pub fn load_many_strict_relative_to<I, P>(
    base: impl AsRef<Path>,
    paths: I,
) -> Result<SpectrumBundle>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    load_spectra_many_strict_relative_to(base, paths)
}

/// Loads multiple paths and returns only summary counts.
///
/// This short alias mirrors [`load_spectra_many_summary`].
///
/// # Errors
///
/// Returns an error when [`load_spectra_many_summary`] would return an error.
pub fn load_many_summary<I, P>(paths: I) -> Result<SpectrumBundleSummary>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    load_spectra_many_summary(paths)
}

/// Strictly loads multiple paths and returns only summary counts.
///
/// This short alias mirrors [`load_spectra_many_summary_strict`].
///
/// # Errors
///
/// Returns an error when [`load_spectra_many_summary_strict`] would return an error.
pub fn load_many_summary_strict<I, P>(paths: I) -> Result<SpectrumBundleSummary>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    load_spectra_many_summary_strict(paths)
}

/// Loads selected paths relative to a base directory and returns summary counts.
///
/// This short alias mirrors [`load_spectra_many_summary_relative_to`].
///
/// # Errors
///
/// Returns an error when [`load_spectra_many_summary_relative_to`] would return an error.
pub fn load_many_summary_relative_to<I, P>(
    base: impl AsRef<Path>,
    paths: I,
) -> Result<SpectrumBundleSummary>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    load_spectra_many_summary_relative_to(base, paths)
}

/// Strictly loads selected paths relative to a base directory and returns summary counts.
///
/// This short alias mirrors [`load_spectra_many_summary_strict_relative_to`].
///
/// # Errors
///
/// Returns an error when [`load_spectra_many_summary_strict_relative_to`] would return an error.
pub fn load_many_summary_strict_relative_to<I, P>(
    base: impl AsRef<Path>,
    paths: I,
) -> Result<SpectrumBundleSummary>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    load_spectra_many_summary_strict_relative_to(base, paths)
}
