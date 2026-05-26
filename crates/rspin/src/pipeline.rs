//! High-level NMR workflow templates.
//!
//! Combines [`rspin_io::load_spectra`] with
//! [`rspin_processing::process_spectrum_auto`] so callers can go from a
//! vendor-format path to phased, frequency-domain 1D spectra in one
//! call. Already-processed inputs (FFT spectra read directly from
//! vendor `1r`/`procpar`-style files) pass through unchanged.

use std::path::Path;

use rspin_core::{RSpinError, Result, Spectrum1D, Unit};
use rspin_io::load_spectra;
use rspin_processing::{
    AutoProcessingOptions, FftDirection, NucleusLbDefaults, ProcessingRecipe1D,
    process_spectrum_auto,
};

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

/// Quick "raw FID → magnitude spectrum" recipe for survey-style
/// inspection of a time-domain input. Applies nucleus-aware
/// exponential-multiplication (EM) apodization, zero-fills to 2×N,
/// runs a forward FFT, takes the magnitude, and normalizes to peak 1.
///
/// The default LB lookup matches [`NucleusLbDefaults::default`]
/// (0.3 Hz for ¹H, 1.0 Hz for ¹³C, 2.0 Hz for ¹⁵N/¹⁹F/³¹P) and falls
/// back to 1.0 Hz when the spectrum's nucleus is unknown.
///
/// # Errors
///
/// Returns an error when `fid` is not time-domain, its dwell time is
/// non-positive, or any processing step fails.
pub fn quick_magnitude_spectrum(fid: &Spectrum1D) -> Result<Spectrum1D> {
    quick_magnitude_spectrum_with(fid, &NucleusLbDefaults::default(), 1.0)
}

/// Same as [`quick_magnitude_spectrum`] but with caller-supplied
/// nucleus → LB defaults and a `fallback_lb_hz` for nuclei not in the
/// lookup. Use this when you want to apply a tighter or looser window
/// than the default 0.3 Hz / 1.0 Hz / 2.0 Hz table.
///
/// # Errors
///
/// Same as [`quick_magnitude_spectrum`].
pub fn quick_magnitude_spectrum_with(
    fid: &Spectrum1D,
    lb_defaults: &NucleusLbDefaults,
    fallback_lb_hz: f64,
) -> Result<Spectrum1D> {
    if fid.x.unit != Unit::Seconds {
        return Err(RSpinError::InvalidSpectrum {
            message: "quick_magnitude_spectrum input must be time-domain (Unit::Seconds)"
                .to_owned(),
        });
    }
    let dwell = dwell_time_seconds(fid)?;
    let target_len = fid
        .len()
        .checked_mul(2)
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "quick_magnitude_spectrum target length overflow".to_owned(),
        })?;
    let lb_hz = lb_defaults
        .lookup(fid.metadata.nucleus.as_ref())
        .unwrap_or(fallback_lb_hz);
    ProcessingRecipe1D::new()
        .exponential_apodization(lb_hz, dwell)
        .zero_fill(target_len)
        .fft(FftDirection::Forward)
        .magnitude()
        .normalize_max_abs()
        .apply(fid)
}

fn dwell_time_seconds(spectrum: &Spectrum1D) -> Result<f64> {
    let first = spectrum
        .x
        .values
        .first()
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "raw FID needs at least two time points".to_owned(),
        })?;
    let second = spectrum
        .x
        .values
        .get(1)
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "raw FID needs at least two time points".to_owned(),
        })?;
    let dwell = (second - first).abs();
    if !dwell.is_finite() || dwell <= 0.0 {
        return Err(RSpinError::InvalidSpectrum {
            message: "raw FID dwell time must be positive and finite".to_owned(),
        });
    }
    Ok(dwell)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rspin_core::{Axis, Metadata, Nucleus, Unit};

    fn synthetic_fid(npts: u32, dwell: f64) -> Spectrum1D {
        let axis_values: Vec<f64> = (0..npts).map(|i| f64::from(i) * dwell).collect();
        let axis = Axis::new("time", Unit::Seconds, axis_values).expect("axis");
        let intensities: Vec<f64> = (0..npts)
            .map(|i| (f64::from(i) * 0.1).cos())
            .collect();
        let imag: Vec<f64> = (0..npts).map(|i| (f64::from(i) * 0.1).sin()).collect();
        let metadata = Metadata::default()
            .with_frequency_mhz(400.0)
            .with_nucleus(Nucleus::Hydrogen1);
        Spectrum1D::new_complex(axis, intensities, Some(imag), metadata).expect("spectrum")
    }

    #[test]
    fn quick_magnitude_spectrum_picks_hydrogen_default_lb() -> Result<()> {
        let fid = synthetic_fid(64, 1.0 / 4000.0);
        let magnitude = quick_magnitude_spectrum(&fid)?;
        assert_eq!(magnitude.x.unit, Unit::Ppm);
        assert_eq!(magnitude.len(), 128);
        let peak = magnitude
            .intensities
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        assert!((peak - 1.0).abs() < 1e-9, "peak={peak}");
        Ok(())
    }

    #[test]
    fn quick_magnitude_spectrum_rejects_frequency_domain() {
        let mut fid = synthetic_fid(8, 0.001);
        fid.x.unit = Unit::Ppm;
        let err = quick_magnitude_spectrum(&fid).expect_err("frequency-domain should reject");
        assert!(matches!(err, RSpinError::InvalidSpectrum { .. }));
    }
}
