//! JSON-oriented API helpers for WASM bindings.

use serde::{Serialize, de::DeserializeOwned};

use rspin_analysis::{IntegralRegion, PeakPickOptions, integrate_region, pick_peaks};
use rspin_core::{RSpinError, Result, Spectrum1D};
use rspin_io::read_jcamp_dx_1d;
use rspin_prediction::PredictionSet;
use rspin_processing::{normalize_max_abs, scale_intensity};
use rspin_simulation::{FirstOrderMultiplet, SimulationOptions, simulate_multiplet_1d};

/// Parses JCAMP-DX text into serialized `Spectrum1D` JSON.
///
/// # Errors
///
/// Returns an error when parsing or serialization fails.
pub fn parse_jcamp_dx_1d_json(input: &str) -> Result<String> {
    let spectrum = read_jcamp_dx_1d(input)?;
    to_json(&spectrum)
}

/// Scales serialized `Spectrum1D` JSON.
///
/// # Errors
///
/// Returns an error when deserialization, processing, or serialization fails.
pub fn scale_spectrum_1d_json(spectrum_json: &str, factor: f64) -> Result<String> {
    let spectrum: Spectrum1D = from_json(spectrum_json)?;
    let processed = scale_intensity(&spectrum, factor)?;
    to_json(&processed)
}

/// Normalizes serialized `Spectrum1D` JSON by maximum absolute intensity.
///
/// # Errors
///
/// Returns an error when deserialization, processing, or serialization fails.
pub fn normalize_spectrum_1d_json(spectrum_json: &str) -> Result<String> {
    let spectrum: Spectrum1D = from_json(spectrum_json)?;
    let processed = normalize_max_abs(&spectrum)?;
    to_json(&processed)
}

/// Picks peaks from serialized `Spectrum1D` JSON.
///
/// # Errors
///
/// Returns an error when deserialization, analysis, or serialization fails.
pub fn pick_peaks_json(spectrum_json: &str, options_json: &str) -> Result<String> {
    let spectrum: Spectrum1D = from_json(spectrum_json)?;
    let options: PeakPickOptions = from_json(options_json)?;
    let peaks = pick_peaks(&spectrum, options)?;
    to_json(&peaks)
}

/// Integrates serialized `Spectrum1D` JSON over a serialized region.
///
/// # Errors
///
/// Returns an error when deserialization, analysis, or serialization fails.
pub fn integrate_region_json(spectrum_json: &str, region_json: &str) -> Result<String> {
    let spectrum: Spectrum1D = from_json(spectrum_json)?;
    let region: IntegralRegion = from_json(region_json)?;
    let integral = integrate_region(&spectrum, region)?;
    to_json(&integral)
}

/// Simulates a serialized first-order multiplet and options into `Spectrum1D` JSON.
///
/// # Errors
///
/// Returns an error when deserialization, simulation, or serialization fails.
pub fn simulate_first_order_multiplet_json(
    multiplet_json: &str,
    options_json: &str,
) -> Result<String> {
    let multiplet: FirstOrderMultiplet = from_json(multiplet_json)?;
    let options: SimulationOptions = from_json(options_json)?;
    let spectrum = simulate_multiplet_1d(&multiplet, options)?;
    to_json(&spectrum)
}

/// Validates serialized prediction JSON and returns normalized JSON.
///
/// # Errors
///
/// Returns an error when deserialization, validation, or serialization fails.
pub fn validate_prediction_json(prediction_json: &str) -> Result<String> {
    let prediction: PredictionSet = from_json(prediction_json)?;
    prediction.validate()?;
    to_json(&prediction)
}

fn from_json<T: DeserializeOwned>(input: &str) -> Result<T> {
    serde_json::from_str(input).map_err(|error| RSpinError::Parse {
        format: "JSON",
        message: error.to_string(),
    })
}

fn to_json<T: Serialize>(value: &T) -> Result<String> {
    serde_json::to_string(value).map_err(|error| RSpinError::Parse {
        format: "JSON",
        message: error.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use rspin_core::Spectrum1D;

    use super::*;

    #[test]
    fn parses_jcamp_to_json() -> anyhow::Result<()> {
        let json = parse_jcamp_dx_1d_json(
            "\
##TITLE=demo
##XUNITS=PPM
##FIRSTX=0
##LASTX=2
##XYDATA=(X++(Y..Y))
0 1 2 3
##END=
",
        )?;
        let spectrum: Spectrum1D = from_json(&json)?;
        assert_eq!(spectrum.len(), 3);
        Ok(())
    }

    #[test]
    fn scales_spectrum_json() -> anyhow::Result<()> {
        let spectrum_json = parse_jcamp_dx_1d_json(
            "\
##TITLE=demo
##FIRSTX=0
##LASTX=1
##XYDATA=(X++(Y..Y))
0 2 4
##END=
",
        )?;
        let scaled_json = scale_spectrum_1d_json(&spectrum_json, 0.5)?;
        let scaled: Spectrum1D = from_json(&scaled_json)?;
        assert_eq!(scaled.intensities, vec![1.0, 2.0]);
        Ok(())
    }

    #[test]
    fn picks_peaks_json() -> anyhow::Result<()> {
        let spectrum_json = parse_jcamp_dx_1d_json(
            "\
##TITLE=demo
##FIRSTX=0
##LASTX=4
##XYDATA=(X++(Y..Y))
0 0 2 0 3 0
##END=
",
        )?;
        let peaks_json = pick_peaks_json(
            &spectrum_json,
            r#"{"min_abs_intensity":1.0,"min_prominence":1.0,"polarity":"Positive"}"#,
        )?;
        let peaks: Vec<rspin_analysis::Peak> = from_json(&peaks_json)?;
        assert_eq!(peaks.len(), 2);
        Ok(())
    }

    #[test]
    fn integrates_region_json() -> anyhow::Result<()> {
        let spectrum_json = parse_jcamp_dx_1d_json(
            "\
##TITLE=demo
##FIRSTX=0
##LASTX=2
##XYDATA=(X++(Y..Y))
0 0 1 2
##END=
",
        )?;
        let integral_json = integrate_region_json(&spectrum_json, r#"{"from":0.0,"to":2.0}"#)?;
        let integral: rspin_analysis::Integral = from_json(&integral_json)?;
        assert!((integral.area - 2.0).abs() < 1e-12);
        Ok(())
    }

    #[test]
    fn simulates_first_order_json() -> anyhow::Result<()> {
        let spectrum_json = simulate_first_order_multiplet_json(
            r#"{"center_ppm":7.0,"area":1.0,"couplings":[{"j_hz":8.0,"equivalent_spins":1}]}"#,
            r#"{"from_ppm":6.95,"to_ppm":7.05,"points":16,"line_width_hz":1.0,"spectrometer_mhz":400.0,"line_shape":"Lorentzian"}"#,
        )?;
        let spectrum: Spectrum1D = from_json(&spectrum_json)?;
        assert_eq!(spectrum.len(), 16);
        Ok(())
    }

    #[test]
    fn validates_prediction_json() -> anyhow::Result<()> {
        let json = validate_prediction_json(
            r#"{"name":"demo","signals_1d":[{"experiment":"Proton1D","nucleus":"Hydrogen1","delta_ppm":1.0,"intensity":1.0,"confidence":0.9,"assignments":[]}],"correlations_2d":[],"provenance":null}"#,
        )?;
        let prediction: PredictionSet = from_json(&json)?;
        assert_eq!(prediction.signals_1d.len(), 1);
        Ok(())
    }
}
