//! Lightweight source-format hints for filtered direct-file routing.

use std::path::Path;

use crate::bundle::{
    format_from_file, is_agilent_fid_dir, is_agilent_processed_dir, is_bruker_fid_dir,
    is_bruker_processed_1d_dir, is_bruker_processed_2d_dir, is_bruker_ser_dir,
};

pub(super) fn auto_file_source_formats(path: &Path) -> Vec<&'static str> {
    let Some(file_name) = path
        .file_name()
        .and_then(std::ffi::OsStr::to_str)
        .map(str::to_ascii_lowercase)
    else {
        return Vec::new();
    };

    let mut formats = Vec::new();
    match file_name.as_str() {
        "fid" => {
            push_source_format_if(
                &mut formats,
                sibling_file_exists(path, "acqus"),
                "bruker_fid",
            );
            push_source_format_if(
                &mut formats,
                sibling_file_exists(path, "procpar"),
                "agilent_fid",
            );
        }
        "ser" => {
            push_source_format_if(
                &mut formats,
                sibling_file_exists(path, "acqus") && sibling_file_exists(path, "acqu2s"),
                "bruker_ser",
            );
        }
        "1r" | "1i" => {
            push_source_format_if(
                &mut formats,
                sibling_file_exists(path, "procs"),
                "bruker_processed",
            );
        }
        "2rr" | "2ri" | "2ir" | "2ii" => {
            push_source_format_if(
                &mut formats,
                sibling_file_exists(path, "procs") && sibling_file_exists(path, "proc2s"),
                "bruker_processed",
            );
        }
        "phasefile" => {
            push_source_format_if(
                &mut formats,
                sibling_file_exists(path, "procpar") || parent_sibling_file_exists(path, "procpar"),
                "agilent_processed",
            );
        }
        _ => {}
    }
    formats
}

pub(super) fn selected_path_source_formats(path: &Path) -> Vec<&'static str> {
    if path.is_file() {
        let format = format_from_file(path);
        if format == "auto" {
            return auto_file_source_formats(path);
        }
        return vec![format];
    }

    let mut formats = Vec::new();
    push_source_format_if(&mut formats, is_bruker_ser_dir(path), "bruker_ser");
    push_source_format_if(&mut formats, is_bruker_fid_dir(path), "bruker_fid");
    push_source_format_if(
        &mut formats,
        is_bruker_processed_1d_dir(path) || is_bruker_processed_2d_dir(path),
        "bruker_processed",
    );
    push_source_format_if(&mut formats, is_agilent_fid_dir(path), "agilent_fid");
    push_source_format_if(
        &mut formats,
        is_agilent_processed_dir(path),
        "agilent_processed",
    );
    formats
}

fn push_source_format_if(formats: &mut Vec<&'static str>, condition: bool, format: &'static str) {
    if condition {
        formats.push(format);
    }
}

fn sibling_file_exists(path: &Path, file_name: &str) -> bool {
    path.parent()
        .is_some_and(|parent| parent.join(file_name).is_file())
}

fn parent_sibling_file_exists(path: &Path, file_name: &str) -> bool {
    path.parent()
        .and_then(Path::parent)
        .is_some_and(|parent| parent.join(file_name).is_file())
}
