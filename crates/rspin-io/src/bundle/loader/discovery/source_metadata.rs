//! Source metadata convenience loading from discovered source candidates.

use std::path::Path;

use rspin_core::Result;

use super::DiscoveredSpectrumSource;
use crate::bundle::{
    LoadedSourceDataKind, LoadedSourceFilter, SpectrumBundle, SpectrumBundleLoader,
    SpectrumBundleSummary,
};

/// Loads discovered source candidates matching one source format.
///
/// Source format aliases such as `jdx`, `jdf`, and `varian fid` are accepted.
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_by_source_format_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    format: impl AsRef<str>,
) -> Result<SpectrumBundle> {
    SpectrumBundleLoader::new().read_discovered_by_source_format_relative_to(base, sources, format)
}

/// Loads discovered source candidates matching one source format.
///
/// This short alias mirrors [`load_discovered_spectra_by_source_format_relative_to`].
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_by_source_format<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    format: impl AsRef<str>,
) -> Result<SpectrumBundle> {
    load_discovered_spectra_by_source_format_relative_to(base, sources, format)
}

/// Loads discovered source candidates matching any source format.
///
/// Formats are combined with logical OR. Passing an empty iterator loads all
/// provided discovered sources.
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_by_source_formats_relative_to<'a, I, F>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    formats: I,
) -> Result<SpectrumBundle>
where
    I: IntoIterator<Item = F>,
    F: AsRef<str>,
{
    SpectrumBundleLoader::new()
        .read_discovered_by_source_formats_relative_to(base, sources, formats)
}

/// Loads discovered source candidates matching any source format.
///
/// This short alias mirrors [`load_discovered_spectra_by_source_formats_relative_to`].
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_by_source_formats<'a, I, F>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    formats: I,
) -> Result<SpectrumBundle>
where
    I: IntoIterator<Item = F>,
    F: AsRef<str>,
{
    load_discovered_spectra_by_source_formats_relative_to(base, sources, formats)
}

/// Loads discovered source candidates matching one vendor family.
///
/// Vendor aliases such as `agilent` and `varian` are accepted.
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_by_source_vendor_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    vendor: impl AsRef<str>,
) -> Result<SpectrumBundle> {
    SpectrumBundleLoader::new().read_discovered_by_source_vendor_relative_to(base, sources, vendor)
}

/// Loads discovered source candidates matching one vendor family.
///
/// This short alias mirrors [`load_discovered_spectra_by_source_vendor_relative_to`].
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_by_source_vendor<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    vendor: impl AsRef<str>,
) -> Result<SpectrumBundle> {
    load_discovered_spectra_by_source_vendor_relative_to(base, sources, vendor)
}

/// Loads discovered source candidates matching any vendor family.
///
/// Vendors are combined with logical OR. Passing an empty iterator loads all
/// provided discovered sources.
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_by_source_vendors_relative_to<'a, I, V>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    vendors: I,
) -> Result<SpectrumBundle>
where
    I: IntoIterator<Item = V>,
    V: AsRef<str>,
{
    SpectrumBundleLoader::new()
        .read_discovered_by_source_vendors_relative_to(base, sources, vendors)
}

/// Loads discovered source candidates matching any vendor family.
///
/// This short alias mirrors [`load_discovered_spectra_by_source_vendors_relative_to`].
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_by_source_vendors<'a, I, V>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    vendors: I,
) -> Result<SpectrumBundle>
where
    I: IntoIterator<Item = V>,
    V: AsRef<str>,
{
    load_discovered_spectra_by_source_vendors_relative_to(base, sources, vendors)
}

/// Loads discovered source candidates matching one raw/processed source data kind.
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_by_source_data_kind_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    data_kind: LoadedSourceDataKind,
) -> Result<SpectrumBundle> {
    SpectrumBundleLoader::new()
        .read_discovered_by_source_data_kind_relative_to(base, sources, data_kind)
}

/// Loads discovered source candidates matching one raw/processed source data kind.
///
/// This short alias mirrors [`load_discovered_spectra_by_source_data_kind_relative_to`].
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_by_source_data_kind<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    data_kind: LoadedSourceDataKind,
) -> Result<SpectrumBundle> {
    load_discovered_spectra_by_source_data_kind_relative_to(base, sources, data_kind)
}

/// Loads discovered source candidates matching any raw/processed source data kind.
///
/// Data kinds are combined with logical OR. Passing an empty iterator loads all
/// provided discovered sources.
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_by_source_data_kinds_relative_to<'a, I>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    data_kinds: I,
) -> Result<SpectrumBundle>
where
    I: IntoIterator<Item = LoadedSourceDataKind>,
{
    SpectrumBundleLoader::new()
        .read_discovered_by_source_data_kinds_relative_to(base, sources, data_kinds)
}

/// Loads discovered source candidates matching any raw/processed source data kind.
///
/// This short alias mirrors [`load_discovered_spectra_by_source_data_kinds_relative_to`].
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_by_source_data_kinds<'a, I>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    data_kinds: I,
) -> Result<SpectrumBundle>
where
    I: IntoIterator<Item = LoadedSourceDataKind>,
{
    load_discovered_spectra_by_source_data_kinds_relative_to(base, sources, data_kinds)
}

/// Strictly loads discovered source candidates matching one source format.
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_strict_by_source_format_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    format: impl AsRef<str>,
) -> Result<SpectrumBundle> {
    SpectrumBundleLoader::new()
        .strict()
        .read_discovered_by_source_format_relative_to(base, sources, format)
}

/// Strictly loads discovered source candidates matching one source format.
///
/// This short alias mirrors [`load_discovered_spectra_strict_by_source_format_relative_to`].
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_strict_by_source_format<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    format: impl AsRef<str>,
) -> Result<SpectrumBundle> {
    load_discovered_spectra_strict_by_source_format_relative_to(base, sources, format)
}

/// Strictly loads discovered source candidates matching any source format.
///
/// Formats are combined with logical OR. Passing an empty iterator loads all
/// provided discovered sources.
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_strict_by_source_formats_relative_to<'a, I, F>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    formats: I,
) -> Result<SpectrumBundle>
where
    I: IntoIterator<Item = F>,
    F: AsRef<str>,
{
    SpectrumBundleLoader::new()
        .strict()
        .read_discovered_by_source_formats_relative_to(base, sources, formats)
}

/// Strictly loads discovered source candidates matching any source format.
///
/// This short alias mirrors [`load_discovered_spectra_strict_by_source_formats_relative_to`].
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_strict_by_source_formats<'a, I, F>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    formats: I,
) -> Result<SpectrumBundle>
where
    I: IntoIterator<Item = F>,
    F: AsRef<str>,
{
    load_discovered_spectra_strict_by_source_formats_relative_to(base, sources, formats)
}

/// Strictly loads discovered source candidates matching one vendor family.
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_strict_by_source_vendor_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    vendor: impl AsRef<str>,
) -> Result<SpectrumBundle> {
    SpectrumBundleLoader::new()
        .strict()
        .read_discovered_by_source_vendor_relative_to(base, sources, vendor)
}

/// Strictly loads discovered source candidates matching one vendor family.
///
/// This short alias mirrors [`load_discovered_spectra_strict_by_source_vendor_relative_to`].
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_strict_by_source_vendor<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    vendor: impl AsRef<str>,
) -> Result<SpectrumBundle> {
    load_discovered_spectra_strict_by_source_vendor_relative_to(base, sources, vendor)
}

/// Strictly loads discovered source candidates matching any vendor family.
///
/// Vendors are combined with logical OR. Passing an empty iterator loads all
/// provided discovered sources.
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_strict_by_source_vendors_relative_to<'a, I, V>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    vendors: I,
) -> Result<SpectrumBundle>
where
    I: IntoIterator<Item = V>,
    V: AsRef<str>,
{
    SpectrumBundleLoader::new()
        .strict()
        .read_discovered_by_source_vendors_relative_to(base, sources, vendors)
}

/// Strictly loads discovered source candidates matching any vendor family.
///
/// This short alias mirrors [`load_discovered_spectra_strict_by_source_vendors_relative_to`].
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_strict_by_source_vendors<'a, I, V>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    vendors: I,
) -> Result<SpectrumBundle>
where
    I: IntoIterator<Item = V>,
    V: AsRef<str>,
{
    load_discovered_spectra_strict_by_source_vendors_relative_to(base, sources, vendors)
}

/// Strictly loads discovered source candidates matching one raw/processed source data kind.
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_strict_by_source_data_kind_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    data_kind: LoadedSourceDataKind,
) -> Result<SpectrumBundle> {
    SpectrumBundleLoader::new()
        .strict()
        .read_discovered_by_source_data_kind_relative_to(base, sources, data_kind)
}

/// Strictly loads discovered source candidates matching one raw/processed source data kind.
///
/// This short alias mirrors [`load_discovered_spectra_strict_by_source_data_kind_relative_to`].
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_strict_by_source_data_kind<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    data_kind: LoadedSourceDataKind,
) -> Result<SpectrumBundle> {
    load_discovered_spectra_strict_by_source_data_kind_relative_to(base, sources, data_kind)
}

/// Strictly loads discovered source candidates matching any raw/processed source data kind.
///
/// Data kinds are combined with logical OR. Passing an empty iterator loads all
/// provided discovered sources.
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_strict_by_source_data_kinds_relative_to<'a, I>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    data_kinds: I,
) -> Result<SpectrumBundle>
where
    I: IntoIterator<Item = LoadedSourceDataKind>,
{
    SpectrumBundleLoader::new()
        .strict()
        .read_discovered_by_source_data_kinds_relative_to(base, sources, data_kinds)
}

/// Strictly loads discovered source candidates matching any raw/processed source data kind.
///
/// This short alias mirrors [`load_discovered_spectra_strict_by_source_data_kinds_relative_to`].
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_strict_by_source_data_kinds<'a, I>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    data_kinds: I,
) -> Result<SpectrumBundle>
where
    I: IntoIterator<Item = LoadedSourceDataKind>,
{
    load_discovered_spectra_strict_by_source_data_kinds_relative_to(base, sources, data_kinds)
}

/// Loads discovered source candidates matching one source format and returns summary counts.
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_by_source_format_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    format: impl AsRef<str>,
) -> Result<SpectrumBundleSummary> {
    SpectrumBundleLoader::new()
        .read_discovered_summary_by_source_format_relative_to(base, sources, format)
}

/// Loads discovered source candidates matching one source format and returns summary counts.
///
/// This short alias mirrors [`load_discovered_spectra_summary_by_source_format_relative_to`].
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_by_source_format<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    format: impl AsRef<str>,
) -> Result<SpectrumBundleSummary> {
    load_discovered_spectra_summary_by_source_format_relative_to(base, sources, format)
}

/// Loads discovered source candidates matching any source format and returns summary counts.
///
/// Formats are combined with logical OR. Passing an empty iterator loads all
/// provided discovered sources.
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_by_source_formats_relative_to<'a, I, F>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    formats: I,
) -> Result<SpectrumBundleSummary>
where
    I: IntoIterator<Item = F>,
    F: AsRef<str>,
{
    SpectrumBundleLoader::new()
        .read_discovered_summary_by_source_formats_relative_to(base, sources, formats)
}

/// Loads discovered source candidates matching any source format and returns summary counts.
///
/// This short alias mirrors [`load_discovered_spectra_summary_by_source_formats_relative_to`].
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_by_source_formats<'a, I, F>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    formats: I,
) -> Result<SpectrumBundleSummary>
where
    I: IntoIterator<Item = F>,
    F: AsRef<str>,
{
    load_discovered_spectra_summary_by_source_formats_relative_to(base, sources, formats)
}

/// Loads discovered source candidates matching one vendor family and returns summary counts.
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_by_source_vendor_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    vendor: impl AsRef<str>,
) -> Result<SpectrumBundleSummary> {
    SpectrumBundleLoader::new()
        .read_discovered_summary_by_source_vendor_relative_to(base, sources, vendor)
}

/// Loads discovered source candidates matching one vendor family and returns summary counts.
///
/// This short alias mirrors [`load_discovered_spectra_summary_by_source_vendor_relative_to`].
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_by_source_vendor<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    vendor: impl AsRef<str>,
) -> Result<SpectrumBundleSummary> {
    load_discovered_spectra_summary_by_source_vendor_relative_to(base, sources, vendor)
}

/// Loads discovered source candidates matching any vendor family and returns summary counts.
///
/// Vendors are combined with logical OR. Passing an empty iterator loads all
/// provided discovered sources.
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_by_source_vendors_relative_to<'a, I, V>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    vendors: I,
) -> Result<SpectrumBundleSummary>
where
    I: IntoIterator<Item = V>,
    V: AsRef<str>,
{
    SpectrumBundleLoader::new()
        .read_discovered_summary_by_source_vendors_relative_to(base, sources, vendors)
}

/// Loads discovered source candidates matching any vendor family and returns summary counts.
///
/// This short alias mirrors [`load_discovered_spectra_summary_by_source_vendors_relative_to`].
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_by_source_vendors<'a, I, V>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    vendors: I,
) -> Result<SpectrumBundleSummary>
where
    I: IntoIterator<Item = V>,
    V: AsRef<str>,
{
    load_discovered_spectra_summary_by_source_vendors_relative_to(base, sources, vendors)
}

/// Loads discovered source candidates matching one raw/processed source data kind and returns summary counts.
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_by_source_data_kind_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    data_kind: LoadedSourceDataKind,
) -> Result<SpectrumBundleSummary> {
    SpectrumBundleLoader::new()
        .read_discovered_summary_by_source_data_kind_relative_to(base, sources, data_kind)
}

/// Loads discovered source candidates matching one raw/processed source data kind and returns summary counts.
///
/// This short alias mirrors [`load_discovered_spectra_summary_by_source_data_kind_relative_to`].
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_by_source_data_kind<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    data_kind: LoadedSourceDataKind,
) -> Result<SpectrumBundleSummary> {
    load_discovered_spectra_summary_by_source_data_kind_relative_to(base, sources, data_kind)
}

/// Loads discovered source candidates matching any raw/processed source data kind and returns summary counts.
///
/// Data kinds are combined with logical OR. Passing an empty iterator loads all
/// provided discovered sources.
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_by_source_data_kinds_relative_to<'a, I>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    data_kinds: I,
) -> Result<SpectrumBundleSummary>
where
    I: IntoIterator<Item = LoadedSourceDataKind>,
{
    SpectrumBundleLoader::new()
        .read_discovered_summary_by_source_data_kinds_relative_to(base, sources, data_kinds)
}

/// Loads discovered source candidates matching any raw/processed source data kind and returns summary counts.
///
/// This short alias mirrors [`load_discovered_spectra_summary_by_source_data_kinds_relative_to`].
///
/// # Errors
///
/// Returns an error when loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_by_source_data_kinds<'a, I>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    data_kinds: I,
) -> Result<SpectrumBundleSummary>
where
    I: IntoIterator<Item = LoadedSourceDataKind>,
{
    load_discovered_spectra_summary_by_source_data_kinds_relative_to(base, sources, data_kinds)
}

/// Strictly loads discovered source candidates matching one source format and returns summary counts.
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_strict_by_source_format_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    format: impl AsRef<str>,
) -> Result<SpectrumBundleSummary> {
    SpectrumBundleLoader::new()
        .strict()
        .read_discovered_summary_by_source_format_relative_to(base, sources, format)
}

/// Strictly loads discovered source candidates matching one source format and returns summary counts.
///
/// This short alias mirrors [`load_discovered_spectra_summary_strict_by_source_format_relative_to`].
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_strict_by_source_format<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    format: impl AsRef<str>,
) -> Result<SpectrumBundleSummary> {
    load_discovered_spectra_summary_strict_by_source_format_relative_to(base, sources, format)
}

/// Strictly loads discovered source candidates matching any source format and returns summary counts.
///
/// Formats are combined with logical OR. Passing an empty iterator loads all
/// provided discovered sources.
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_strict_by_source_formats_relative_to<'a, I, F>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    formats: I,
) -> Result<SpectrumBundleSummary>
where
    I: IntoIterator<Item = F>,
    F: AsRef<str>,
{
    SpectrumBundleLoader::new()
        .strict()
        .read_discovered_summary_by_source_formats_relative_to(base, sources, formats)
}

/// Strictly loads discovered source candidates matching any source format and returns summary counts.
///
/// This short alias mirrors [`load_discovered_spectra_summary_strict_by_source_formats_relative_to`].
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_strict_by_source_formats<'a, I, F>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    formats: I,
) -> Result<SpectrumBundleSummary>
where
    I: IntoIterator<Item = F>,
    F: AsRef<str>,
{
    load_discovered_spectra_summary_strict_by_source_formats_relative_to(base, sources, formats)
}

/// Strictly loads discovered source candidates matching one vendor family and returns summary counts.
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_strict_by_source_vendor_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    vendor: impl AsRef<str>,
) -> Result<SpectrumBundleSummary> {
    SpectrumBundleLoader::new()
        .strict()
        .read_discovered_summary_by_source_vendor_relative_to(base, sources, vendor)
}

/// Strictly loads discovered source candidates matching one vendor family and returns summary counts.
///
/// This short alias mirrors [`load_discovered_spectra_summary_strict_by_source_vendor_relative_to`].
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_strict_by_source_vendor<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    vendor: impl AsRef<str>,
) -> Result<SpectrumBundleSummary> {
    load_discovered_spectra_summary_strict_by_source_vendor_relative_to(base, sources, vendor)
}

/// Strictly loads discovered source candidates matching any vendor family and returns summary counts.
///
/// Vendors are combined with logical OR. Passing an empty iterator loads all
/// provided discovered sources.
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_strict_by_source_vendors_relative_to<'a, I, V>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    vendors: I,
) -> Result<SpectrumBundleSummary>
where
    I: IntoIterator<Item = V>,
    V: AsRef<str>,
{
    SpectrumBundleLoader::new()
        .strict()
        .read_discovered_summary_by_source_vendors_relative_to(base, sources, vendors)
}

/// Strictly loads discovered source candidates matching any vendor family and returns summary counts.
///
/// This short alias mirrors [`load_discovered_spectra_summary_strict_by_source_vendors_relative_to`].
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_strict_by_source_vendors<'a, I, V>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    vendors: I,
) -> Result<SpectrumBundleSummary>
where
    I: IntoIterator<Item = V>,
    V: AsRef<str>,
{
    load_discovered_spectra_summary_strict_by_source_vendors_relative_to(base, sources, vendors)
}

/// Strictly loads discovered source candidates matching one raw/processed source data kind and returns summary counts.
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_strict_by_source_data_kind_relative_to<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    data_kind: LoadedSourceDataKind,
) -> Result<SpectrumBundleSummary> {
    SpectrumBundleLoader::new()
        .strict()
        .read_discovered_summary_by_source_data_kind_relative_to(base, sources, data_kind)
}

/// Strictly loads discovered source candidates matching one raw/processed source data kind and returns summary counts.
///
/// This short alias mirrors [`load_discovered_spectra_summary_strict_by_source_data_kind_relative_to`].
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_strict_by_source_data_kind<'a>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    data_kind: LoadedSourceDataKind,
) -> Result<SpectrumBundleSummary> {
    load_discovered_spectra_summary_strict_by_source_data_kind_relative_to(base, sources, data_kind)
}

/// Strictly loads discovered source candidates matching any raw/processed source data kind and returns summary counts.
///
/// Data kinds are combined with logical OR. Passing an empty iterator loads all
/// provided discovered sources.
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_strict_by_source_data_kinds_relative_to<'a, I>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    data_kinds: I,
) -> Result<SpectrumBundleSummary>
where
    I: IntoIterator<Item = LoadedSourceDataKind>,
{
    SpectrumBundleLoader::new()
        .strict()
        .read_discovered_summary_by_source_data_kinds_relative_to(base, sources, data_kinds)
}

/// Strictly loads discovered source candidates matching any raw/processed source data kind and returns summary counts.
///
/// This short alias mirrors [`load_discovered_spectra_summary_strict_by_source_data_kinds_relative_to`].
///
/// # Errors
///
/// Returns an error when strict loading the matching discovered sources fails.
pub fn load_discovered_spectra_summary_strict_by_source_data_kinds<'a, I>(
    base: impl AsRef<Path>,
    sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
    data_kinds: I,
) -> Result<SpectrumBundleSummary>
where
    I: IntoIterator<Item = LoadedSourceDataKind>,
{
    load_discovered_spectra_summary_strict_by_source_data_kinds_relative_to(
        base, sources, data_kinds,
    )
}

impl SpectrumBundleLoader {
    /// Loads discovered source candidates matching one source format.
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_by_source_format_relative_to<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        format: impl AsRef<str>,
    ) -> Result<SpectrumBundle> {
        self.read_discovered_by_source_relative_to(
            base,
            sources,
            LoadedSourceFilter::format(format),
        )
    }

    /// Loads discovered source candidates matching one source format.
    ///
    /// This short alias mirrors [`Self::read_discovered_by_source_format_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_by_source_format<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        format: impl AsRef<str>,
    ) -> Result<SpectrumBundle> {
        self.read_discovered_by_source_format_relative_to(base, sources, format)
    }

    /// Loads discovered source candidates matching any source format.
    ///
    /// Formats are combined with logical OR. Passing an empty iterator loads all
    /// provided discovered sources.
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_by_source_formats_relative_to<'a, I, F>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        formats: I,
    ) -> Result<SpectrumBundle>
    where
        I: IntoIterator<Item = F>,
        F: AsRef<str>,
    {
        self.read_discovered_by_sources_relative_to(base, sources, format_filters(formats))
    }

    /// Loads discovered source candidates matching any source format.
    ///
    /// This short alias mirrors [`Self::read_discovered_by_source_formats_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_by_source_formats<'a, I, F>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        formats: I,
    ) -> Result<SpectrumBundle>
    where
        I: IntoIterator<Item = F>,
        F: AsRef<str>,
    {
        self.read_discovered_by_source_formats_relative_to(base, sources, formats)
    }

    /// Loads discovered source candidates matching one vendor family.
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_by_source_vendor_relative_to<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        vendor: impl AsRef<str>,
    ) -> Result<SpectrumBundle> {
        self.read_discovered_by_source_relative_to(
            base,
            sources,
            LoadedSourceFilter::vendor(vendor),
        )
    }

    /// Loads discovered source candidates matching one vendor family.
    ///
    /// This short alias mirrors [`Self::read_discovered_by_source_vendor_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_by_source_vendor<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        vendor: impl AsRef<str>,
    ) -> Result<SpectrumBundle> {
        self.read_discovered_by_source_vendor_relative_to(base, sources, vendor)
    }

    /// Loads discovered source candidates matching any vendor family.
    ///
    /// Vendors are combined with logical OR. Passing an empty iterator loads all
    /// provided discovered sources.
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_by_source_vendors_relative_to<'a, I, V>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        vendors: I,
    ) -> Result<SpectrumBundle>
    where
        I: IntoIterator<Item = V>,
        V: AsRef<str>,
    {
        self.read_discovered_by_sources_relative_to(base, sources, vendor_filters(vendors))
    }

    /// Loads discovered source candidates matching any vendor family.
    ///
    /// This short alias mirrors [`Self::read_discovered_by_source_vendors_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_by_source_vendors<'a, I, V>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        vendors: I,
    ) -> Result<SpectrumBundle>
    where
        I: IntoIterator<Item = V>,
        V: AsRef<str>,
    {
        self.read_discovered_by_source_vendors_relative_to(base, sources, vendors)
    }

    /// Loads discovered source candidates matching one raw/processed source data kind.
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_by_source_data_kind_relative_to<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        data_kind: LoadedSourceDataKind,
    ) -> Result<SpectrumBundle> {
        self.read_discovered_by_source_relative_to(
            base,
            sources,
            LoadedSourceFilter::data_kind(data_kind),
        )
    }

    /// Loads discovered source candidates matching one raw/processed source data kind.
    ///
    /// This short alias mirrors [`Self::read_discovered_by_source_data_kind_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_by_source_data_kind<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        data_kind: LoadedSourceDataKind,
    ) -> Result<SpectrumBundle> {
        self.read_discovered_by_source_data_kind_relative_to(base, sources, data_kind)
    }

    /// Loads discovered source candidates matching any raw/processed source data kind.
    ///
    /// Data kinds are combined with logical OR. Passing an empty iterator loads
    /// all provided discovered sources.
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_by_source_data_kinds_relative_to<'a, I>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        data_kinds: I,
    ) -> Result<SpectrumBundle>
    where
        I: IntoIterator<Item = LoadedSourceDataKind>,
    {
        self.read_discovered_by_sources_relative_to(base, sources, data_kind_filters(data_kinds))
    }

    /// Loads discovered source candidates matching any raw/processed source data kind.
    ///
    /// This short alias mirrors [`Self::read_discovered_by_source_data_kinds_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_by_source_data_kinds<'a, I>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        data_kinds: I,
    ) -> Result<SpectrumBundle>
    where
        I: IntoIterator<Item = LoadedSourceDataKind>,
    {
        self.read_discovered_by_source_data_kinds_relative_to(base, sources, data_kinds)
    }

    /// Loads discovered source candidates matching one source format and returns summary counts.
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_summary_by_source_format_relative_to<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        format: impl AsRef<str>,
    ) -> Result<SpectrumBundleSummary> {
        self.read_discovered_by_source_format_relative_to(base, sources, format)
            .map(|bundle| bundle.summary())
    }

    /// Loads discovered source candidates matching one source format and returns summary counts.
    ///
    /// This short alias mirrors [`Self::read_discovered_summary_by_source_format_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_summary_by_source_format<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        format: impl AsRef<str>,
    ) -> Result<SpectrumBundleSummary> {
        self.read_discovered_summary_by_source_format_relative_to(base, sources, format)
    }

    /// Loads discovered source candidates matching any source format and returns summary counts.
    ///
    /// Formats are combined with logical OR. Passing an empty iterator loads all
    /// provided discovered sources.
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_summary_by_source_formats_relative_to<'a, I, F>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        formats: I,
    ) -> Result<SpectrumBundleSummary>
    where
        I: IntoIterator<Item = F>,
        F: AsRef<str>,
    {
        self.read_discovered_by_source_formats_relative_to(base, sources, formats)
            .map(|bundle| bundle.summary())
    }

    /// Loads discovered source candidates matching any source format and returns summary counts.
    ///
    /// This short alias mirrors [`Self::read_discovered_summary_by_source_formats_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_summary_by_source_formats<'a, I, F>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        formats: I,
    ) -> Result<SpectrumBundleSummary>
    where
        I: IntoIterator<Item = F>,
        F: AsRef<str>,
    {
        self.read_discovered_summary_by_source_formats_relative_to(base, sources, formats)
    }

    /// Loads discovered source candidates matching one vendor family and returns summary counts.
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_summary_by_source_vendor_relative_to<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        vendor: impl AsRef<str>,
    ) -> Result<SpectrumBundleSummary> {
        self.read_discovered_by_source_vendor_relative_to(base, sources, vendor)
            .map(|bundle| bundle.summary())
    }

    /// Loads discovered source candidates matching one vendor family and returns summary counts.
    ///
    /// This short alias mirrors [`Self::read_discovered_summary_by_source_vendor_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_summary_by_source_vendor<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        vendor: impl AsRef<str>,
    ) -> Result<SpectrumBundleSummary> {
        self.read_discovered_summary_by_source_vendor_relative_to(base, sources, vendor)
    }

    /// Loads discovered source candidates matching any vendor family and returns summary counts.
    ///
    /// Vendors are combined with logical OR. Passing an empty iterator loads all
    /// provided discovered sources.
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_summary_by_source_vendors_relative_to<'a, I, V>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        vendors: I,
    ) -> Result<SpectrumBundleSummary>
    where
        I: IntoIterator<Item = V>,
        V: AsRef<str>,
    {
        self.read_discovered_by_source_vendors_relative_to(base, sources, vendors)
            .map(|bundle| bundle.summary())
    }

    /// Loads discovered source candidates matching any vendor family and returns summary counts.
    ///
    /// This short alias mirrors [`Self::read_discovered_summary_by_source_vendors_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_summary_by_source_vendors<'a, I, V>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        vendors: I,
    ) -> Result<SpectrumBundleSummary>
    where
        I: IntoIterator<Item = V>,
        V: AsRef<str>,
    {
        self.read_discovered_summary_by_source_vendors_relative_to(base, sources, vendors)
    }

    /// Loads discovered source candidates matching one raw/processed source data kind and returns summary counts.
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_summary_by_source_data_kind_relative_to<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        data_kind: LoadedSourceDataKind,
    ) -> Result<SpectrumBundleSummary> {
        self.read_discovered_by_source_data_kind_relative_to(base, sources, data_kind)
            .map(|bundle| bundle.summary())
    }

    /// Loads discovered source candidates matching one raw/processed source data kind and returns summary counts.
    ///
    /// This short alias mirrors [`Self::read_discovered_summary_by_source_data_kind_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_summary_by_source_data_kind<'a>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        data_kind: LoadedSourceDataKind,
    ) -> Result<SpectrumBundleSummary> {
        self.read_discovered_summary_by_source_data_kind_relative_to(base, sources, data_kind)
    }

    /// Loads discovered source candidates matching any raw/processed source data kind and returns summary counts.
    ///
    /// Data kinds are combined with logical OR. Passing an empty iterator loads
    /// all provided discovered sources.
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_summary_by_source_data_kinds_relative_to<'a, I>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        data_kinds: I,
    ) -> Result<SpectrumBundleSummary>
    where
        I: IntoIterator<Item = LoadedSourceDataKind>,
    {
        self.read_discovered_by_source_data_kinds_relative_to(base, sources, data_kinds)
            .map(|bundle| bundle.summary())
    }

    /// Loads discovered source candidates matching any raw/processed source data kind and returns summary counts.
    ///
    /// This short alias mirrors [`Self::read_discovered_summary_by_source_data_kinds_relative_to`].
    ///
    /// # Errors
    ///
    /// Returns an error when loading the matching discovered sources fails.
    pub fn read_discovered_summary_by_source_data_kinds<'a, I>(
        &self,
        base: impl AsRef<Path>,
        sources: impl IntoIterator<Item = &'a DiscoveredSpectrumSource>,
        data_kinds: I,
    ) -> Result<SpectrumBundleSummary>
    where
        I: IntoIterator<Item = LoadedSourceDataKind>,
    {
        self.read_discovered_summary_by_source_data_kinds_relative_to(base, sources, data_kinds)
    }
}

fn format_filters<I, F>(formats: I) -> Vec<LoadedSourceFilter>
where
    I: IntoIterator<Item = F>,
    F: AsRef<str>,
{
    let mut filters = Vec::new();
    for format in formats {
        filters.push(LoadedSourceFilter::format(format));
    }
    filters
}

fn vendor_filters<I, V>(vendors: I) -> Vec<LoadedSourceFilter>
where
    I: IntoIterator<Item = V>,
    V: AsRef<str>,
{
    let mut filters = Vec::new();
    for vendor in vendors {
        filters.push(LoadedSourceFilter::vendor(vendor));
    }
    filters
}

fn data_kind_filters<I>(data_kinds: I) -> Vec<LoadedSourceFilter>
where
    I: IntoIterator<Item = LoadedSourceDataKind>,
{
    let mut filters = Vec::new();
    for data_kind in data_kinds {
        filters.push(LoadedSourceFilter::data_kind(data_kind));
    }
    filters
}
