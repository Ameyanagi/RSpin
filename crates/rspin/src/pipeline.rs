//! High-level NMR workflow templates.
//!
//! Combines [`rspin_io::load_spectra`] with
//! [`rspin_processing::process_spectrum_auto`] so callers can go from a
//! vendor-format path to phased, frequency-domain 1D spectra in one
//! call. Already-processed inputs (FFT spectra read directly from
//! vendor `1r`/`procpar`-style files) pass through unchanged.

use std::path::Path;

use rspin_core::{Result, Spectrum1D, Unit};
use rspin_io::load_spectra;
use rspin_processing::{AutoProcessingOptions, process_spectrum_auto};

/// Loads every 1D spectrum at `path` and runs the standard
/// auto-processing pipeline on FID inputs.
///
/// Time-domain spectra (`Unit::Seconds`) go through
/// [`process_spectrum_auto`] with [`AutoProcessingOptions::default`];
/// frequency-domain inputs are returned as-is.
///
/// # Errors
///
/// Returns an error when the path cannot be loaded or any processing
/// step fails.
pub fn load_and_process_auto<P: AsRef<Path>>(path: P) -> Result<Vec<Spectrum1D>> {
    load_and_process_with(path, &AutoProcessingOptions::default())
}

/// Same as [`load_and_process_auto`] but with caller-supplied options.
///
/// # Errors
///
/// Returns an error when the path cannot be loaded or any processing
/// step fails.
pub fn load_and_process_with<P: AsRef<Path>>(
    path: P,
    options: &AutoProcessingOptions,
) -> Result<Vec<Spectrum1D>> {
    let bundle = load_spectra(path)?;
    let mut out = Vec::new();
    for spectrum in bundle.spectra_1d() {
        let processed = if spectrum.x.unit == Unit::Seconds {
            process_spectrum_auto(spectrum, options)?
        } else {
            spectrum.clone()
        };
        out.push(processed);
    }
    Ok(out)
}
