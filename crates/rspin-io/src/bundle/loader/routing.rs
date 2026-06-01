//! Private filesystem routing for the spectrum bundle loader.

mod source_hints;

use std::path::{Path, PathBuf};

use rspin_core::{RSpinError, Result, Spectrum1D, Spectrum2D};

use crate::{
    read_agilent_arrayed_fid_1d_dir, read_agilent_arrayed_fid_2d_dir, read_agilent_fid_1d_dir,
    read_agilent_fid_2d_dir, read_agilent_processed_1d_dir, read_agilent_processed_2d_dir,
    read_bruker_fid_1d_dir, read_bruker_processed_1d_dir, read_bruker_processed_2d_dir,
    read_bruker_ser_2d_dir, read_nmredata_records_file, read_spectrum_bundle_json_file,
    read_spectrum1d_path, read_spectrum2d_path,
};

use super::{FileCandidateKind, SpectrumBundleLoader};
use crate::bundle::LoadedSourceFilter;
use crate::bundle::{
    LoadWarning, LoadedSource, LoadedSourceDataKind, LoadedSourceFormat, LoadedSpectrum,
    SpectrumBundle, clear_bundle_source_paths, collect_tree, disabled_candidate_message,
    disabled_dimension_error, disabled_dimension_message, fallback_message, file_candidate_kind,
    format_from_file, is_agilent_arrayed_1d_fid_path, is_agilent_arrayed_2d_fid_path,
    is_agilent_fid_dir, is_agilent_format, is_agilent_processed_dir, is_bruker_fid_dir,
    is_bruker_processed_1d_dir, is_bruker_processed_2d_dir, is_bruker_ser_dir, is_json_file,
    is_nmredata_file, is_standalone_spectrum_file, nmredata_record_molecule,
    prefix_bundle_source_paths, relative_source_path, selected_path_candidate_kind,
    source_format_1d, source_format_2d, source_format_candidate_kind, source_format_matches,
};
use source_hints::{auto_file_source_formats, selected_path_source_formats};

impl SpectrumBundleLoader {
    pub(super) fn read_existing_path_into(
        &self,
        source_root: &Path,
        path: &Path,
        bundle: &mut SpectrumBundle,
    ) -> Result<()> {
        if path.is_dir() {
            self.read_directory(source_root, path, bundle)
        } else {
            self.read_file_candidate(source_root, path, bundle)
        }
    }

    fn read_directory(
        &self,
        source_root: &Path,
        directory: &Path,
        bundle: &mut SpectrumBundle,
    ) -> Result<()> {
        let tree = collect_tree(directory)?;
        for directory in &tree.directories {
            self.read_directory_candidate(source_root, directory, bundle)?;
        }
        for file in &tree.files {
            if is_nmredata_file(file) {
                if self.allows_source_path(source_root, file) {
                    self.read_nmredata_candidate(source_root, file, bundle)?;
                }
            } else if is_standalone_spectrum_file(file) {
                self.read_file_candidate(source_root, file, bundle)?;
            }
        }
        Ok(())
    }

    fn read_directory_candidate(
        &self,
        root: &Path,
        directory: &Path,
        bundle: &mut SpectrumBundle,
    ) -> Result<()> {
        if !self.allows_source_path(root, directory) {
            return Ok(());
        }

        self.read_bruker_directory_candidate(root, directory, bundle)?;
        self.read_agilent_directory_candidate(root, directory, bundle)?;
        Ok(())
    }

    fn read_bruker_directory_candidate(
        &self,
        root: &Path,
        directory: &Path,
        bundle: &mut SpectrumBundle,
    ) -> Result<()> {
        if self.raw.is_enabled()
            && self.two_d.is_enabled()
            && self.allows_routed_source_format("bruker_ser")
            && is_bruker_ser_dir(directory)
        {
            self.add_2d_result(
                bundle,
                root,
                directory,
                "bruker_ser",
                read_bruker_ser_2d_dir(directory),
            )?;
        }
        if self.raw.is_enabled()
            && self.one_d.is_enabled()
            && self.allows_routed_source_format("bruker_fid")
            && is_bruker_fid_dir(directory)
        {
            self.add_1d_result(
                bundle,
                root,
                directory,
                "bruker_fid",
                read_bruker_fid_1d_dir(directory),
            )?;
        }
        if self.processed.is_enabled()
            && self.two_d.is_enabled()
            && self.allows_routed_source_format("bruker_processed")
            && is_bruker_processed_2d_dir(directory)
        {
            self.add_2d_result(
                bundle,
                root,
                directory,
                "bruker_processed",
                read_bruker_processed_2d_dir(directory),
            )?;
        }
        if self.processed.is_enabled()
            && self.one_d.is_enabled()
            && self.allows_routed_source_format("bruker_processed")
            && is_bruker_processed_1d_dir(directory)
        {
            self.add_1d_result(
                bundle,
                root,
                directory,
                "bruker_processed",
                read_bruker_processed_1d_dir(directory),
            )?;
        }
        Ok(())
    }

    fn read_agilent_directory_candidate(
        &self,
        root: &Path,
        directory: &Path,
        bundle: &mut SpectrumBundle,
    ) -> Result<()> {
        if self.raw.is_enabled()
            && self.allows_routed_source_format("agilent_fid")
            && is_agilent_fid_dir(directory)
        {
            if is_agilent_arrayed_2d_fid_path(directory) {
                if self.two_d.is_enabled() {
                    self.add_2d_results(
                        bundle,
                        root,
                        directory,
                        "agilent_fid",
                        read_agilent_arrayed_fid_2d_dir(directory),
                    )?;
                }
                return Ok(());
            }
            if is_agilent_arrayed_1d_fid_path(directory) {
                if self.one_d.is_enabled() {
                    self.add_1d_results(
                        bundle,
                        root,
                        directory,
                        "agilent_fid",
                        read_agilent_arrayed_fid_1d_dir(directory),
                    )?;
                }
                return Ok(());
            }
            self.add_1d_or_2d_result(
                bundle,
                root,
                directory,
                "agilent_fid",
                || read_agilent_fid_1d_dir(directory),
                || read_agilent_fid_2d_dir(directory),
            )?;
        }
        if self.processed.is_enabled()
            && self.allows_routed_source_format("agilent_processed")
            && is_agilent_processed_dir(directory)
        {
            self.add_1d_or_2d_result(
                bundle,
                root,
                directory,
                "agilent_processed",
                || read_agilent_processed_1d_dir(directory),
                || read_agilent_processed_2d_dir(directory),
            )?;
        }
        Ok(())
    }

    fn read_file_candidate(
        &self,
        root: &Path,
        file: &Path,
        bundle: &mut SpectrumBundle,
    ) -> Result<()> {
        let candidate_kind = file_candidate_kind(file);
        if self.handle_disabled_file_candidate(root, file, bundle, candidate_kind)? {
            return Ok(());
        }
        if !self.allows_file_candidate_kind(candidate_kind) {
            return Ok(());
        }

        if is_nmredata_file(file) {
            if !self.allows_source_path(root, file) {
                return Ok(());
            }
            return self.read_nmredata_candidate(root, file, bundle);
        }
        if self.read_arrayed_agilent_file_candidate(root, file, bundle)? {
            return Ok(());
        }
        if is_json_file(file) {
            return self.read_json_file_candidate(root, file, bundle);
        }
        if let Some(message) = self.disabled_dimension_file_message(file) {
            return self.handle_error_message(bundle, root, file, message);
        }

        let format = format_from_file(file);
        if format != "auto" && !self.allows_source_format(format) {
            return Ok(());
        }
        if format != "auto" && !self.allows_source_candidate_kind(format) {
            return Ok(());
        }
        if format == "auto" && !self.allows_any_auto_file_source_format(file) {
            return Ok(());
        }
        if !self.allows_source_path(root, file) {
            return Ok(());
        }

        self.add_1d_or_2d_result(
            bundle,
            root,
            file,
            format,
            || read_spectrum1d_path(file),
            || read_spectrum2d_path(file),
        )
    }

    fn handle_disabled_file_candidate(
        &self,
        root: &Path,
        file: &Path,
        bundle: &mut SpectrumBundle,
        candidate_kind: FileCandidateKind,
    ) -> Result<bool> {
        match candidate_kind {
            FileCandidateKind::Raw if !self.raw.is_enabled() => {
                self.handle_error_message(
                    bundle,
                    root,
                    file,
                    format!(
                        "raw spectrum candidates are disabled for {}",
                        file.display()
                    ),
                )?;
                Ok(true)
            }
            FileCandidateKind::Processed if !self.processed.is_enabled() => {
                self.handle_error_message(
                    bundle,
                    root,
                    file,
                    format!(
                        "processed spectrum candidates are disabled for {}",
                        file.display()
                    ),
                )?;
                Ok(true)
            }
            FileCandidateKind::Raw | FileCandidateKind::Processed | FileCandidateKind::Other => {
                Ok(false)
            }
        }
    }

    fn read_arrayed_agilent_file_candidate(
        &self,
        root: &Path,
        file: &Path,
        bundle: &mut SpectrumBundle,
    ) -> Result<bool> {
        if !(self.raw.is_enabled() && self.allows_routed_source_format("agilent_fid")) {
            return Ok(false);
        }
        if is_agilent_arrayed_1d_fid_path(file) {
            if !self.one_d.is_enabled() {
                self.handle_error_message(
                    bundle,
                    root,
                    file,
                    disabled_dimension_message(file, "one-dimensional"),
                )?;
                return Ok(true);
            }
            self.add_1d_results(
                bundle,
                root,
                file,
                "agilent_fid",
                read_agilent_arrayed_fid_1d_dir(file),
            )?;
            return Ok(true);
        }
        if is_agilent_arrayed_2d_fid_path(file) {
            if !self.two_d.is_enabled() {
                self.handle_error_message(
                    bundle,
                    root,
                    file,
                    disabled_dimension_message(file, "two-dimensional"),
                )?;
                return Ok(true);
            }
            self.add_2d_results(
                bundle,
                root,
                file,
                "agilent_fid",
                read_agilent_arrayed_fid_2d_dir(file),
            )?;
            return Ok(true);
        }
        Ok(false)
    }

    fn read_json_file_candidate(
        &self,
        root: &Path,
        file: &Path,
        bundle: &mut SpectrumBundle,
    ) -> Result<()> {
        if !self.allows_or_may_contain_source_path(root, file) {
            return Ok(());
        }
        self.add_1d_or_2d_or_bundle_result(
            bundle,
            root,
            file,
            || read_spectrum1d_path(file),
            || read_spectrum2d_path(file),
            || read_spectrum_bundle_json_file(file),
        )
    }

    fn add_1d_or_2d_or_bundle_result(
        &self,
        bundle: &mut SpectrumBundle,
        root: &Path,
        path: &Path,
        read_1d: impl FnOnce() -> Result<Spectrum1D>,
        read_2d: impl FnOnce() -> Result<Spectrum2D>,
        read_bundle: impl FnOnce() -> Result<SpectrumBundle>,
    ) -> Result<()> {
        let format = format_from_file(path);
        match read_1d() {
            Ok(spectrum) => {
                if self.one_d.is_enabled() {
                    self.push_1d_if_allowed(
                        bundle,
                        root,
                        path,
                        source_format_1d(path, format),
                        spectrum,
                    );
                    return Ok(());
                }

                let first_error = disabled_dimension_error(path, "one-dimensional");
                if self.two_d.is_enabled() {
                    return match read_2d() {
                        Ok(spectrum) => {
                            self.push_2d_if_allowed(
                                bundle,
                                root,
                                path,
                                source_format_2d(path, format),
                                spectrum,
                            );
                            Ok(())
                        }
                        Err(second_error) => self.add_bundle_or_warning(
                            bundle,
                            root,
                            path,
                            read_bundle,
                            Some(&first_error),
                            Some(&second_error),
                        ),
                    };
                }

                self.add_bundle_or_warning(
                    bundle,
                    root,
                    path,
                    read_bundle,
                    Some(&first_error),
                    None,
                )
            }
            Err(first_error) => {
                if self.two_d.is_enabled() {
                    return match read_2d() {
                        Ok(spectrum) => {
                            self.push_2d_if_allowed(
                                bundle,
                                root,
                                path,
                                source_format_2d(path, format),
                                spectrum,
                            );
                            Ok(())
                        }
                        Err(second_error) => self.add_bundle_or_warning(
                            bundle,
                            root,
                            path,
                            read_bundle,
                            Some(&first_error),
                            Some(&second_error),
                        ),
                    };
                }
                let second_error = match read_2d() {
                    Ok(_) => disabled_dimension_error(path, "two-dimensional"),
                    Err(error) => error,
                };
                self.add_bundle_or_warning(
                    bundle,
                    root,
                    path,
                    read_bundle,
                    Some(&first_error),
                    Some(&second_error),
                )
            }
        }
    }

    fn add_bundle_or_warning(
        &self,
        bundle: &mut SpectrumBundle,
        root: &Path,
        path: &Path,
        read_bundle: impl FnOnce() -> Result<SpectrumBundle>,
        first_error: Option<&RSpinError>,
        second_error: Option<&RSpinError>,
    ) -> Result<()> {
        match read_bundle() {
            Ok(loaded) => {
                let loaded = self.bundle_with_source_context(root, path, loaded);
                bundle.extend_bundle(loaded);
                Ok(())
            }
            Err(third_error) => {
                let message = fallback_message(first_error, second_error, &third_error);
                self.handle_error_message(bundle, root, path, message)
            }
        }
    }

    fn read_nmredata_candidate(
        &self,
        root: &Path,
        file: &Path,
        bundle: &mut SpectrumBundle,
    ) -> Result<()> {
        match read_nmredata_records_file(file) {
            Ok(records) => {
                for (record_index, record) in records.iter().enumerate() {
                    if let Some(molecule) =
                        nmredata_record_molecule(root, file, record_index, record)
                    {
                        bundle.push_molecule(molecule);
                    }
                }
                Ok(())
            }
            Err(error) => self.handle_error(bundle, root, file, error),
        }
    }

    fn add_1d_result(
        &self,
        bundle: &mut SpectrumBundle,
        root: &Path,
        path: &Path,
        format: &'static str,
        result: Result<Spectrum1D>,
    ) -> Result<()> {
        if !self.one_d.is_enabled() {
            return Ok(());
        }
        match result {
            Ok(spectrum) => {
                self.push_1d_if_allowed(bundle, root, path, format, spectrum);
                Ok(())
            }
            Err(error) => self.handle_error(bundle, root, path, error),
        }
    }

    fn add_1d_results(
        &self,
        bundle: &mut SpectrumBundle,
        root: &Path,
        path: &Path,
        format: &'static str,
        result: Result<Vec<Spectrum1D>>,
    ) -> Result<()> {
        if !self.one_d.is_enabled() {
            return Ok(());
        }
        match result {
            Ok(spectra) => {
                for spectrum in spectra {
                    self.push_1d_if_allowed(bundle, root, path, format, spectrum);
                }
                Ok(())
            }
            Err(error) => self.handle_error(bundle, root, path, error),
        }
    }

    fn add_2d_result(
        &self,
        bundle: &mut SpectrumBundle,
        root: &Path,
        path: &Path,
        format: &'static str,
        result: Result<Spectrum2D>,
    ) -> Result<()> {
        if !self.two_d.is_enabled() {
            return Ok(());
        }
        match result {
            Ok(spectrum) => {
                self.push_2d_if_allowed(bundle, root, path, format, spectrum);
                Ok(())
            }
            Err(error) => self.handle_error(bundle, root, path, error),
        }
    }

    fn add_2d_results(
        &self,
        bundle: &mut SpectrumBundle,
        root: &Path,
        path: &Path,
        format: &'static str,
        result: Result<Vec<Spectrum2D>>,
    ) -> Result<()> {
        if !self.two_d.is_enabled() {
            return Ok(());
        }
        match result {
            Ok(spectra) => {
                for spectrum in spectra {
                    self.push_2d_if_allowed(bundle, root, path, format, spectrum);
                }
                Ok(())
            }
            Err(error) => self.handle_error(bundle, root, path, error),
        }
    }

    fn add_1d_or_2d_result(
        &self,
        bundle: &mut SpectrumBundle,
        root: &Path,
        path: &Path,
        format: &'static str,
        read_1d: impl FnOnce() -> Result<Spectrum1D>,
        read_2d: impl FnOnce() -> Result<Spectrum2D>,
    ) -> Result<()> {
        if is_agilent_format(format)
            && let Some(message) = self.disabled_dimension_file_message(path)
        {
            if root == path {
                return self.handle_error_message(bundle, root, path, message);
            }
            return Ok(());
        }

        if self.one_d.is_enabled() {
            match read_1d() {
                Ok(spectrum) => {
                    self.push_1d_if_allowed(
                        bundle,
                        root,
                        path,
                        source_format_1d(path, format),
                        spectrum,
                    );
                    return Ok(());
                }
                Err(first_error) if !self.two_d.is_enabled() => {
                    return self.handle_error(bundle, root, path, first_error);
                }
                Err(first_error) => {
                    return match read_2d() {
                        Ok(spectrum) => {
                            self.push_2d_if_allowed(
                                bundle,
                                root,
                                path,
                                source_format_2d(path, format),
                                spectrum,
                            );
                            Ok(())
                        }
                        Err(second_error) => {
                            let message =
                                format!("{first_error}; two-dimensional fallback: {second_error}");
                            self.handle_error_message(bundle, root, path, message)
                        }
                    };
                }
            }
        }

        if self.two_d.is_enabled() {
            match read_2d() {
                Ok(spectrum) => {
                    self.push_2d_if_allowed(
                        bundle,
                        root,
                        path,
                        source_format_2d(path, format),
                        spectrum,
                    );
                    Ok(())
                }
                Err(error) => self.handle_error(bundle, root, path, error),
            }
        } else {
            Ok(())
        }
    }

    fn disabled_dimension_file_message(&self, path: &Path) -> Option<String> {
        let supports_1d = crate::detect_spectrum1d_path_format(path).is_ok();
        let supports_2d = crate::detect_spectrum2d_path_format(path).is_ok();

        if !self.one_d.is_enabled() && supports_1d && (!self.two_d.is_enabled() || !supports_2d) {
            return Some(disabled_dimension_message(path, "one-dimensional"));
        }
        if !self.two_d.is_enabled() && supports_2d && (!self.one_d.is_enabled() || !supports_1d) {
            return Some(disabled_dimension_message(path, "two-dimensional"));
        }
        None
    }

    pub(super) fn add_selected_path_disabled_warning(
        &self,
        root: &Path,
        bundle: &mut SpectrumBundle,
    ) -> Result<()> {
        if bundle.has_data() || !bundle.warnings.is_empty() {
            return Ok(());
        }

        if let Some(message) = self.disabled_selected_path_message(root) {
            self.handle_error_message(bundle, root, root, message)?;
        }
        Ok(())
    }

    pub(super) fn disabled_selected_path_message(&self, path: &Path) -> Option<String> {
        match selected_path_candidate_kind(path) {
            FileCandidateKind::Raw if !self.raw.is_enabled() => {
                return Some(disabled_candidate_message(path, "raw spectrum"));
            }
            FileCandidateKind::Processed if !self.processed.is_enabled() => {
                return Some(disabled_candidate_message(path, "processed spectrum"));
            }
            FileCandidateKind::Raw | FileCandidateKind::Processed | FileCandidateKind::Other => {}
        }

        if is_agilent_arrayed_1d_fid_path(path) && !self.one_d.is_enabled() {
            return Some(disabled_dimension_message(path, "one-dimensional"));
        }
        if is_agilent_arrayed_2d_fid_path(path) && !self.two_d.is_enabled() {
            return Some(disabled_dimension_message(path, "two-dimensional"));
        }

        self.disabled_dimension_file_message(path)
    }

    pub(super) fn selected_path_is_filtered_out(&self, root: &Path, path: &Path) -> bool {
        if !self.allows_or_may_contain_source_path(root, path) {
            return true;
        }

        let source_formats = selected_path_source_formats(path);
        !source_formats.is_empty()
            && !source_formats
                .iter()
                .any(|format| self.allows_routed_source_format(format))
    }

    fn filter_bundle_dimensions(&self, bundle: &mut SpectrumBundle) {
        let include_1d = self.one_d.is_enabled();
        let include_2d = self.two_d.is_enabled();
        bundle.spectra.retain(|entry| match entry {
            LoadedSpectrum::OneD { source, .. } => {
                include_1d
                    && self.allows_source_format(source.format())
                    && self.allows_source_candidate_kind(source.format())
            }
            LoadedSpectrum::TwoD { source, .. } => {
                include_2d
                    && self.allows_source_format(source.format())
                    && self.allows_source_candidate_kind(source.format())
            }
        });
    }

    fn filter_bundle_source_paths(&self, bundle: &mut SpectrumBundle) {
        if self.source_path_filters.is_empty() && self.source_path_prefix_filters.is_empty() {
            return;
        }

        bundle.spectra.retain(|entry| {
            entry.source().path().is_some_and(|path| {
                self.source_path_filters.is_empty()
                    || self
                        .source_path_filters
                        .iter()
                        .any(|allowed| allowed.as_path() == path)
            }) && entry.source().path().is_some_and(|path| {
                self.source_path_prefix_filters.is_empty()
                    || self
                        .source_path_prefix_filters
                        .iter()
                        .any(|allowed| path.starts_with(allowed))
            })
        });
        bundle.warnings.retain(|warning| {
            warning.path().is_some_and(|path| {
                self.source_path_filters.is_empty()
                    || self
                        .source_path_filters
                        .iter()
                        .any(|allowed| allowed.as_path() == path)
            }) && warning.path().is_some_and(|path| {
                self.source_path_prefix_filters.is_empty()
                    || self
                        .source_path_prefix_filters
                        .iter()
                        .any(|allowed| path.starts_with(allowed))
            })
        });
    }

    fn filter_bundle_sources(&self, bundle: &mut SpectrumBundle) {
        if self.source_filters.is_empty() {
            return;
        }

        bundle.spectra.retain(|entry| {
            self.source_filters
                .iter()
                .any(|filter| filter.matches_source(entry.source()))
        });

        let keeps_format_or_vendor_warnings = self
            .source_filters
            .iter()
            .any(|filter| !filter.is_path_filter());
        bundle.warnings.retain(|warning| {
            keeps_format_or_vendor_warnings
                || warning.path().is_some_and(|path| {
                    self.source_filters
                        .iter()
                        .any(|filter| filter.may_match_path(path))
                })
        });
    }

    pub(super) fn handle_error(
        &self,
        bundle: &mut SpectrumBundle,
        root: &Path,
        path: &Path,
        error: RSpinError,
    ) -> Result<()> {
        if self.strict.is_enabled() {
            Err(error)
        } else {
            self.handle_error_message(bundle, root, path, error.to_string())
        }
    }

    pub(super) fn handle_error_message(
        &self,
        bundle: &mut SpectrumBundle,
        root: &Path,
        path: &Path,
        message: String,
    ) -> Result<()> {
        if self.strict.is_enabled() {
            Err(RSpinError::Parse {
                format: "spectrum bundle",
                message,
            })
        } else {
            bundle.push_warning(LoadWarning::new(
                self.source_path_for_metadata(root, path),
                message,
            ));
            Ok(())
        }
    }

    fn loaded_source(&self, root: &Path, path: &Path, format: impl Into<String>) -> LoadedSource {
        LoadedSource::new(self.source_path_for_metadata(root, path), format)
    }

    fn push_1d_if_allowed(
        &self,
        bundle: &mut SpectrumBundle,
        root: &Path,
        path: &Path,
        format: &'static str,
        spectrum: Spectrum1D,
    ) {
        if self.allows_source(root, path, format) {
            bundle.push_1d(spectrum, self.loaded_source(root, path, format));
        }
    }

    fn push_2d_if_allowed(
        &self,
        bundle: &mut SpectrumBundle,
        root: &Path,
        path: &Path,
        format: &'static str,
        spectrum: Spectrum2D,
    ) {
        if self.allows_source(root, path, format) {
            bundle.push_2d(spectrum, self.loaded_source(root, path, format));
        }
    }

    fn allows_source(&self, root: &Path, path: &Path, format: &str) -> bool {
        self.allows_source_format(format)
            && self.allows_source_candidate_kind(format)
            && self.allows_source_path(root, path)
            && self.allows_source_filter(root, path, format)
    }

    fn allows_source_format(&self, format: &str) -> bool {
        let allowed_by_named_formats = self.source_formats.is_empty()
            || self
                .source_formats
                .iter()
                .any(|allowed| source_format_matches(format, allowed));
        let allowed_by_generic_filters = self.source_filters.is_empty()
            || self
                .source_filters
                .iter()
                .any(|filter| filter.may_match_format(format));
        allowed_by_named_formats && allowed_by_generic_filters
    }

    fn allows_routed_source_format(&self, format: &str) -> bool {
        self.allows_source_format(format) && self.allows_source_candidate_kind(format)
    }

    fn allows_any_auto_file_source_format(&self, file: &Path) -> bool {
        let source_formats = auto_file_source_formats(file);
        source_formats.is_empty()
            || source_formats
                .iter()
                .any(|format| self.allows_routed_source_format(format))
    }

    fn allows_file_candidate_kind(&self, kind: FileCandidateKind) -> bool {
        let allowed_by_toggle = match kind {
            FileCandidateKind::Raw => self.raw.is_enabled(),
            FileCandidateKind::Processed => self.processed.is_enabled(),
            FileCandidateKind::Other => true,
        };
        let allowed_by_data_kind = self.source_data_kind_filters.is_empty()
            || self
                .source_data_kind_filters
                .iter()
                .any(|allowed| *allowed == source_data_kind_for_candidate(kind));
        let allowed_by_generic_filters = self.source_filters.is_empty()
            || self
                .source_filters
                .iter()
                .any(|filter| filter_may_match_candidate_kind(filter, kind));
        allowed_by_toggle && allowed_by_data_kind && allowed_by_generic_filters
    }

    fn allows_source_candidate_kind(&self, format: &str) -> bool {
        let allowed_by_toggle = match source_format_candidate_kind(format) {
            FileCandidateKind::Raw => self.raw.is_enabled(),
            FileCandidateKind::Processed => self.processed.is_enabled(),
            FileCandidateKind::Other => true,
        };
        let allowed_by_data_kind = self.source_data_kind_filters.is_empty()
            || self
                .source_data_kind_filters
                .iter()
                .any(|allowed| *allowed == source_data_kind_for_format(format));
        allowed_by_toggle && allowed_by_data_kind
    }

    fn allows_source_path(&self, root: &Path, path: &Path) -> bool {
        if self.source_path_filters.is_empty()
            && self.source_path_prefix_filters.is_empty()
            && self.source_filters.is_empty()
        {
            return true;
        }

        relative_source_path(root, path).is_some_and(|source_path| {
            let allowed_by_named_paths = self.source_path_filters.is_empty()
                || self
                    .source_path_filters
                    .iter()
                    .any(|allowed| allowed == &source_path);
            let allowed_by_path_prefixes = self.source_path_prefix_filters.is_empty()
                || self
                    .source_path_prefix_filters
                    .iter()
                    .any(|allowed| source_path.starts_with(allowed));
            let allowed_by_generic_filters = self.source_filters.is_empty()
                || self
                    .source_filters
                    .iter()
                    .any(|filter| filter.may_match_path(&source_path));
            allowed_by_named_paths && allowed_by_path_prefixes && allowed_by_generic_filters
        })
    }

    fn allows_source_filter(&self, root: &Path, path: &Path, format: &str) -> bool {
        if self.source_filters.is_empty() {
            return true;
        }
        let source = LoadedSource::new(relative_source_path(root, path), format);
        self.source_filters
            .iter()
            .any(|filter| filter.matches_source(&source))
    }

    fn allows_or_may_contain_source_path(&self, root: &Path, path: &Path) -> bool {
        let has_named_path_filters =
            !self.source_path_filters.is_empty() || !self.source_path_prefix_filters.is_empty();
        let generic_filters_only_paths = !self.source_filters.is_empty()
            && self
                .source_filters
                .iter()
                .all(LoadedSourceFilter::is_path_filter);
        if (!has_named_path_filters && !generic_filters_only_paths) || root == path {
            return true;
        }

        relative_source_path(root, path).is_some_and(|container_path| {
            let may_contain_exact_path = self.source_path_filters.iter().any(|allowed| {
                allowed == &container_path || allowed.starts_with(container_path.as_path())
            });
            let may_contain_path_prefix = self.source_path_prefix_filters.iter().any(|allowed| {
                allowed.starts_with(container_path.as_path())
                    || container_path.starts_with(allowed.as_path())
            });
            let may_match_generic_filter = !generic_filters_only_paths
                || self
                    .source_filters
                    .iter()
                    .any(|filter| filter_may_match_container_path(filter, &container_path));
            (self.source_path_filters.is_empty() || may_contain_exact_path)
                && (self.source_path_prefix_filters.is_empty() || may_contain_path_prefix)
                && may_match_generic_filter
        })
    }

    fn bundle_with_source_context(
        &self,
        root: &Path,
        path: &Path,
        mut bundle: SpectrumBundle,
    ) -> SpectrumBundle {
        self.filter_bundle_dimensions(&mut bundle);
        if root != path
            && let Some(container_path) = relative_source_path(root, path)
        {
            prefix_bundle_source_paths(&mut bundle, &container_path);
        }
        self.filter_bundle_source_paths(&mut bundle);
        self.filter_bundle_sources(&mut bundle);
        if !self.source_paths.is_enabled() {
            clear_bundle_source_paths(&mut bundle);
        }
        bundle
    }

    fn source_path_for_metadata(&self, root: &Path, path: &Path) -> Option<PathBuf> {
        if !self.source_paths.is_enabled() {
            return None;
        }
        relative_source_path(root, path)
    }
}

fn source_data_kind_for_format(format: &str) -> LoadedSourceDataKind {
    LoadedSourceFormat::parse(format)
        .map_or(LoadedSourceDataKind::Other, LoadedSourceFormat::data_kind)
}

fn source_data_kind_for_candidate(kind: FileCandidateKind) -> LoadedSourceDataKind {
    match kind {
        FileCandidateKind::Raw => LoadedSourceDataKind::Raw,
        FileCandidateKind::Processed => LoadedSourceDataKind::Processed,
        FileCandidateKind::Other => LoadedSourceDataKind::Other,
    }
}

fn filter_may_match_candidate_kind(filter: &LoadedSourceFilter, kind: FileCandidateKind) -> bool {
    match filter {
        LoadedSourceFilter::DataKind { data_kind } => {
            *data_kind == source_data_kind_for_candidate(kind)
        }
        LoadedSourceFilter::Format { .. }
        | LoadedSourceFilter::Vendor { .. }
        | LoadedSourceFilter::Path { .. }
        | LoadedSourceFilter::PathPrefix { .. } => true,
    }
}

fn filter_may_match_container_path(filter: &LoadedSourceFilter, container_path: &Path) -> bool {
    match filter {
        LoadedSourceFilter::Path { path } => {
            path == container_path || path.starts_with(container_path)
        }
        LoadedSourceFilter::PathPrefix { path } => {
            path.starts_with(container_path) || container_path.starts_with(path)
        }
        LoadedSourceFilter::Format { .. }
        | LoadedSourceFilter::Vendor { .. }
        | LoadedSourceFilter::DataKind { .. } => true,
    }
}
