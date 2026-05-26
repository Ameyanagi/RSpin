//! Private filesystem routing for the spectrum bundle loader.

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
use crate::bundle::{
    LoadWarning, LoadedSource, LoadedSpectrum, SpectrumBundle, clear_bundle_source_paths,
    collect_tree, disabled_candidate_message, disabled_dimension_error, disabled_dimension_message,
    fallback_message, file_candidate_kind, format_from_file, is_agilent_arrayed_1d_fid_path,
    is_agilent_arrayed_2d_fid_path, is_agilent_fid_dir, is_agilent_format,
    is_agilent_processed_dir, is_bruker_fid_dir, is_bruker_processed_1d_dir,
    is_bruker_processed_2d_dir, is_bruker_ser_dir, is_json_file, is_nmredata_file,
    is_standalone_spectrum_file, nmredata_record_molecule, prefix_bundle_source_paths,
    relative_source_path, selected_path_candidate_kind, source_format_1d, source_format_2d,
    source_format_candidate_kind, source_format_matches,
};

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
                self.read_nmredata_candidate(source_root, file, bundle)?;
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
            && self.allows_source_format("bruker_ser")
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
            && self.allows_source_format("bruker_fid")
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
            && self.allows_source_format("bruker_processed")
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
            && self.allows_source_format("bruker_processed")
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
            && self.allows_source_format("agilent_fid")
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
            && self.allows_source_format("agilent_processed")
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
        match file_candidate_kind(file) {
            FileCandidateKind::Raw if !self.raw.is_enabled() => {
                return self.handle_error_message(
                    bundle,
                    root,
                    file,
                    format!(
                        "raw spectrum candidates are disabled for {}",
                        file.display()
                    ),
                );
            }
            FileCandidateKind::Processed if !self.processed.is_enabled() => {
                return self.handle_error_message(
                    bundle,
                    root,
                    file,
                    format!(
                        "processed spectrum candidates are disabled for {}",
                        file.display()
                    ),
                );
            }
            FileCandidateKind::Raw | FileCandidateKind::Processed | FileCandidateKind::Other => {}
        }

        if is_nmredata_file(file) {
            return self.read_nmredata_candidate(root, file, bundle);
        }
        if self.raw.is_enabled()
            && self.allows_source_format("agilent_fid")
            && is_agilent_arrayed_1d_fid_path(file)
        {
            if !self.one_d.is_enabled() {
                return self.handle_error_message(
                    bundle,
                    root,
                    file,
                    disabled_dimension_message(file, "one-dimensional"),
                );
            }
            return self.add_1d_results(
                bundle,
                root,
                file,
                "agilent_fid",
                read_agilent_arrayed_fid_1d_dir(file),
            );
        }
        if self.raw.is_enabled()
            && self.allows_source_format("agilent_fid")
            && is_agilent_arrayed_2d_fid_path(file)
        {
            if !self.two_d.is_enabled() {
                return self.handle_error_message(
                    bundle,
                    root,
                    file,
                    disabled_dimension_message(file, "two-dimensional"),
                );
            }
            return self.add_2d_results(
                bundle,
                root,
                file,
                "agilent_fid",
                read_agilent_arrayed_fid_2d_dir(file),
            );
        }
        if is_json_file(file) {
            return self.add_1d_or_2d_or_bundle_result(
                bundle,
                root,
                file,
                || read_spectrum1d_path(file),
                || read_spectrum2d_path(file),
                || read_spectrum_bundle_json_file(file),
            );
        }
        if let Some(message) = self.disabled_dimension_file_message(file) {
            return self.handle_error_message(bundle, root, file, message);
        }

        let format = format_from_file(file);
        if format != "auto" && !self.allows_source_format(format) {
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
            bundle.push_warning(LoadWarning::new(self.source_path(root, path), message));
            Ok(())
        }
    }

    fn loaded_source(&self, root: &Path, path: &Path, format: impl Into<String>) -> LoadedSource {
        LoadedSource::new(self.source_path(root, path), format)
    }

    fn push_1d_if_allowed(
        &self,
        bundle: &mut SpectrumBundle,
        root: &Path,
        path: &Path,
        format: &'static str,
        spectrum: Spectrum1D,
    ) {
        if self.allows_source_format(format) {
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
        if self.allows_source_format(format) {
            bundle.push_2d(spectrum, self.loaded_source(root, path, format));
        }
    }

    fn allows_source_format(&self, format: &str) -> bool {
        self.source_formats.is_empty()
            || self
                .source_formats
                .iter()
                .any(|allowed| source_format_matches(format, allowed))
    }

    fn allows_source_candidate_kind(&self, format: &str) -> bool {
        match source_format_candidate_kind(format) {
            FileCandidateKind::Raw => self.raw.is_enabled(),
            FileCandidateKind::Processed => self.processed.is_enabled(),
            FileCandidateKind::Other => true,
        }
    }

    fn bundle_with_source_context(
        &self,
        root: &Path,
        path: &Path,
        mut bundle: SpectrumBundle,
    ) -> SpectrumBundle {
        self.filter_bundle_dimensions(&mut bundle);
        if !self.source_paths.is_enabled() {
            clear_bundle_source_paths(&mut bundle);
            return bundle;
        }

        if root == path {
            return bundle;
        }

        let Some(container_path) = self.source_path(root, path) else {
            return bundle;
        };
        prefix_bundle_source_paths(&mut bundle, &container_path);
        bundle
    }

    fn source_path(&self, root: &Path, path: &Path) -> Option<PathBuf> {
        if !self.source_paths.is_enabled() {
            return None;
        }
        relative_source_path(root, path)
    }
}
