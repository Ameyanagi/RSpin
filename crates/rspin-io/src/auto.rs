//! Text spectrum format detection and convenience readers.

use std::{fs, path::Path};

use rspin_core::{RSpinError, Result, Spectrum1D, Spectrum2D};

use crate::{
    SpectrumPathReader, SpectrumReader, read_agilent_fid_1d_dir, read_agilent_fid_2d_dir,
    read_bruker_fid_1d_dir, read_bruker_processed_1d_dir, read_bruker_processed_2d_dir,
    read_bruker_ser_2d_dir, read_jcamp_dx_1d, read_jeol_jdf_1d_file, read_jeol_jdf_2d_file,
    read_nmrml_1d_str, read_nmrml_2d_str, read_spectrum1d_csv, read_spectrum1d_json,
    read_spectrum2d_csv, read_spectrum2d_json, write_jcamp_dx_1d, write_nmrml_1d, write_nmrml_2d,
    write_spectrum1d_csv, write_spectrum1d_json, write_spectrum2d_csv, write_spectrum2d_json,
};

/// Text spectrum formats supported by the auto-detecting readers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpectrumTextFormat {
    /// `RSpin` JSON spectrum payload.
    Json,
    /// nmrML XML payload.
    NmrMl,
    /// JCAMP-DX text payload.
    JcampDx,
    /// `RSpin` CSV payload.
    Csv,
}

/// Filesystem formats supported by the one-dimensional auto reader.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spectrum1DPathFormat {
    /// Text payload detected as JSON.
    Json,
    /// Text payload detected as nmrML.
    NmrMl,
    /// Text payload detected as JCAMP-DX.
    JcampDx,
    /// Text payload detected as CSV.
    Csv,
    /// JEOL Delta `.jdf` file.
    JeolJdf,
    /// Bruker processed one-dimensional dataset directory.
    BrukerProcessed,
    /// Bruker raw one-dimensional FID dataset directory or `fid` file.
    BrukerFid,
    /// Agilent/Varian raw one-dimensional FID dataset directory or `fid` file.
    AgilentFid,
}

/// Filesystem formats supported by the two-dimensional auto reader.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spectrum2DPathFormat {
    /// Text payload detected as JSON.
    Json,
    /// Text payload detected as nmrML.
    NmrMl,
    /// Text payload detected as CSV.
    Csv,
    /// JEOL Delta `.jdf` file.
    JeolJdf,
    /// Bruker processed two-dimensional dataset directory.
    BrukerProcessed,
    /// Bruker raw two-dimensional `ser` dataset directory or `ser` file.
    BrukerSer,
    /// Agilent/Varian raw two-dimensional FID dataset directory or `fid` file.
    AgilentFid,
}

/// Filesystem text formats supported by the one-dimensional auto writer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spectrum1DWritePathFormat {
    /// `RSpin` JSON spectrum payload.
    Json,
    /// nmrML XML payload.
    NmrMl,
    /// JCAMP-DX text payload.
    JcampDx,
    /// `RSpin` CSV payload.
    Csv,
}

/// Filesystem text formats supported by the two-dimensional auto writer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spectrum2DWritePathFormat {
    /// `RSpin` JSON spectrum payload.
    Json,
    /// nmrML XML payload.
    NmrMl,
    /// `RSpin` CSV payload.
    Csv,
}

/// Auto-detecting reader for one-dimensional text spectra.
#[derive(Clone, Copy, Debug, Default)]
pub struct AutoSpectrum1DText;

impl SpectrumReader for AutoSpectrum1DText {
    type Output = Spectrum1D;

    fn read_str(&self, input: &str) -> Result<Self::Output> {
        read_spectrum1d_text(input)
    }
}

/// Auto-detecting reader for two-dimensional text spectra.
#[derive(Clone, Copy, Debug, Default)]
pub struct AutoSpectrum2DText;

impl SpectrumReader for AutoSpectrum2DText {
    type Output = Spectrum2D;

    fn read_str(&self, input: &str) -> Result<Self::Output> {
        read_spectrum2d_text(input)
    }
}

/// Auto-detecting reader for one-dimensional spectrum paths.
#[derive(Clone, Copy, Debug, Default)]
pub struct AutoSpectrum1DPath;

impl AutoSpectrum1DPath {
    /// Reads a one-dimensional spectrum from an auto-detected path.
    ///
    /// # Errors
    ///
    /// Returns an error when the path is missing, unsupported, or malformed.
    pub fn read_path(self, path: impl AsRef<Path>) -> Result<Spectrum1D> {
        read_spectrum1d_path(path)
    }
}

impl SpectrumPathReader for AutoSpectrum1DPath {
    type Output = Spectrum1D;

    fn read_path(&self, path: &Path) -> Result<Self::Output> {
        read_spectrum1d_path(path)
    }
}

/// Auto-detecting reader for two-dimensional spectrum paths.
#[derive(Clone, Copy, Debug, Default)]
pub struct AutoSpectrum2DPath;

impl AutoSpectrum2DPath {
    /// Reads a two-dimensional spectrum from an auto-detected path.
    ///
    /// # Errors
    ///
    /// Returns an error when the path is missing, unsupported, or malformed.
    pub fn read_path(self, path: impl AsRef<Path>) -> Result<Spectrum2D> {
        read_spectrum2d_path(path)
    }
}

impl SpectrumPathReader for AutoSpectrum2DPath {
    type Output = Spectrum2D;

    fn read_path(&self, path: &Path) -> Result<Self::Output> {
        read_spectrum2d_path(path)
    }
}

/// Extension-selecting writer for one-dimensional spectrum paths.
#[derive(Clone, Copy, Debug, Default)]
pub struct AutoSpectrum1DPathWriter;

impl AutoSpectrum1DPathWriter {
    /// Writes a one-dimensional spectrum to a path using the path extension.
    ///
    /// # Errors
    ///
    /// Returns an error when the path extension is unsupported, the spectrum
    /// cannot be represented by the selected writer, or the file cannot be
    /// written.
    pub fn write_path(self, spectrum: &Spectrum1D, path: impl AsRef<Path>) -> Result<()> {
        write_spectrum1d_path(spectrum, path)
    }
}

/// Extension-selecting writer for two-dimensional spectrum paths.
#[derive(Clone, Copy, Debug, Default)]
pub struct AutoSpectrum2DPathWriter;

impl AutoSpectrum2DPathWriter {
    /// Writes a two-dimensional spectrum to a path using the path extension.
    ///
    /// # Errors
    ///
    /// Returns an error when the path extension is unsupported, the spectrum
    /// cannot be represented by the selected writer, or the file cannot be
    /// written.
    pub fn write_path(self, spectrum: &Spectrum2D, path: impl AsRef<Path>) -> Result<()> {
        write_spectrum2d_path(spectrum, path)
    }
}

/// Detects a supported text spectrum format from the payload shape.
///
/// # Errors
///
/// Returns an error when the payload is empty or does not look like a
/// supported spectrum text format.
pub fn detect_spectrum_text_format(input: &str) -> Result<SpectrumTextFormat> {
    let trimmed = input.trim_start();
    if trimmed.is_empty() {
        return Err(RSpinError::Parse {
            format: "text spectrum",
            message: "empty input".to_owned(),
        });
    }

    if trimmed.starts_with('{') {
        return Ok(SpectrumTextFormat::Json);
    }

    if looks_like_nmrml(trimmed) {
        return Ok(SpectrumTextFormat::NmrMl);
    }

    if looks_like_jcamp_dx(input) {
        return Ok(SpectrumTextFormat::JcampDx);
    }

    if looks_like_csv(input) {
        return Ok(SpectrumTextFormat::Csv);
    }

    Err(RSpinError::Unsupported {
        feature: "text spectrum format",
    })
}

/// Reads a one-dimensional spectrum from JSON, nmrML, JCAMP-DX, or CSV text.
///
/// # Errors
///
/// Returns an error when the format cannot be detected or the selected parser
/// rejects the payload.
pub fn read_spectrum1d_text(input: &str) -> Result<Spectrum1D> {
    match detect_spectrum_text_format(input)? {
        SpectrumTextFormat::Json => read_spectrum1d_json(input),
        SpectrumTextFormat::NmrMl => read_nmrml_1d_str(input),
        SpectrumTextFormat::JcampDx => read_jcamp_dx_1d(input),
        SpectrumTextFormat::Csv => read_spectrum1d_csv(input),
    }
}

/// Reads a two-dimensional spectrum from JSON, nmrML, or CSV text.
///
/// # Errors
///
/// Returns an error when the format cannot be detected, the selected parser
/// rejects the payload, or the payload is a one-dimensional-only format.
pub fn read_spectrum2d_text(input: &str) -> Result<Spectrum2D> {
    match detect_spectrum_text_format(input)? {
        SpectrumTextFormat::Json => read_spectrum2d_json(input),
        SpectrumTextFormat::NmrMl => read_nmrml_2d_str(input),
        SpectrumTextFormat::Csv => read_spectrum2d_csv(input),
        SpectrumTextFormat::JcampDx => Err(RSpinError::Unsupported {
            feature: "two-dimensional JCAMP-DX text reader",
        }),
    }
}

/// Detects a supported one-dimensional spectrum format from a filesystem path.
///
/// # Errors
///
/// Returns an error when the path is missing or does not look like a supported
/// one-dimensional spectrum path.
pub fn detect_spectrum1d_path_format(path: impl AsRef<Path>) -> Result<Spectrum1DPathFormat> {
    let path = path.as_ref();
    ensure_path_exists(path)?;

    if looks_like_bruker_processed_1d(path) {
        return Ok(Spectrum1DPathFormat::BrukerProcessed);
    }
    if looks_like_bruker_fid(path) {
        return Ok(Spectrum1DPathFormat::BrukerFid);
    }
    if looks_like_agilent_fid(path) {
        return Ok(Spectrum1DPathFormat::AgilentFid);
    }
    if is_extension(path, &["jdf"]) {
        return Ok(Spectrum1DPathFormat::JeolJdf);
    }

    match text_format_from_path(path)? {
        SpectrumTextFormat::Json => Ok(Spectrum1DPathFormat::Json),
        SpectrumTextFormat::NmrMl => Ok(Spectrum1DPathFormat::NmrMl),
        SpectrumTextFormat::JcampDx => Ok(Spectrum1DPathFormat::JcampDx),
        SpectrumTextFormat::Csv => Ok(Spectrum1DPathFormat::Csv),
    }
}

/// Detects a supported two-dimensional spectrum format from a filesystem path.
///
/// # Errors
///
/// Returns an error when the path is missing or does not look like a supported
/// two-dimensional spectrum path.
pub fn detect_spectrum2d_path_format(path: impl AsRef<Path>) -> Result<Spectrum2DPathFormat> {
    let path = path.as_ref();
    ensure_path_exists(path)?;

    if looks_like_bruker_processed_2d(path) {
        return Ok(Spectrum2DPathFormat::BrukerProcessed);
    }
    if looks_like_bruker_ser(path) {
        return Ok(Spectrum2DPathFormat::BrukerSer);
    }
    if looks_like_agilent_fid(path) {
        return Ok(Spectrum2DPathFormat::AgilentFid);
    }
    if is_extension(path, &["jdf"]) {
        return Ok(Spectrum2DPathFormat::JeolJdf);
    }

    match text_format_from_path(path)? {
        SpectrumTextFormat::Json => Ok(Spectrum2DPathFormat::Json),
        SpectrumTextFormat::NmrMl => Ok(Spectrum2DPathFormat::NmrMl),
        SpectrumTextFormat::Csv => Ok(Spectrum2DPathFormat::Csv),
        SpectrumTextFormat::JcampDx => Err(RSpinError::Unsupported {
            feature: "two-dimensional JCAMP-DX path reader",
        }),
    }
}

/// Detects the one-dimensional text writer format selected by a destination path.
///
/// Unlike reader detection, writer detection uses only the path extension and
/// does not require the destination path to exist.
///
/// # Errors
///
/// Returns an unsupported-feature error when the extension is not a supported
/// one-dimensional text export format.
pub fn detect_spectrum1d_write_path_format(
    path: impl AsRef<Path>,
) -> Result<Spectrum1DWritePathFormat> {
    let path = path.as_ref();
    if is_extension(path, &["json"]) {
        return Ok(Spectrum1DWritePathFormat::Json);
    }
    if is_extension(path, &["nmrml", "xml"]) {
        return Ok(Spectrum1DWritePathFormat::NmrMl);
    }
    if is_extension(path, &["jdx", "dx"]) {
        return Ok(Spectrum1DWritePathFormat::JcampDx);
    }
    if is_extension(path, &["csv"]) {
        return Ok(Spectrum1DWritePathFormat::Csv);
    }
    Err(RSpinError::Unsupported {
        feature: "one-dimensional spectrum path writer format",
    })
}

/// Detects the two-dimensional text writer format selected by a destination path.
///
/// Unlike reader detection, writer detection uses only the path extension and
/// does not require the destination path to exist.
///
/// # Errors
///
/// Returns an unsupported-feature error when the extension is not a supported
/// two-dimensional text export format.
pub fn detect_spectrum2d_write_path_format(
    path: impl AsRef<Path>,
) -> Result<Spectrum2DWritePathFormat> {
    let path = path.as_ref();
    if is_extension(path, &["json"]) {
        return Ok(Spectrum2DWritePathFormat::Json);
    }
    if is_extension(path, &["nmrml", "xml"]) {
        return Ok(Spectrum2DWritePathFormat::NmrMl);
    }
    if is_extension(path, &["csv"]) {
        return Ok(Spectrum2DWritePathFormat::Csv);
    }
    Err(RSpinError::Unsupported {
        feature: "two-dimensional spectrum path writer format",
    })
}

/// Reads a one-dimensional spectrum from an auto-detected path.
///
/// # Errors
///
/// Returns an error when the format cannot be detected or the selected parser
/// rejects the path contents.
pub fn read_spectrum1d_path(path: impl AsRef<Path>) -> Result<Spectrum1D> {
    let path = path.as_ref();
    match detect_spectrum1d_path_format(path)? {
        Spectrum1DPathFormat::Json => read_spectrum1d_json(&read_text_file(path)?),
        Spectrum1DPathFormat::NmrMl => read_nmrml_1d_str(&read_text_file(path)?),
        Spectrum1DPathFormat::JcampDx => read_jcamp_dx_1d(&read_text_file(path)?),
        Spectrum1DPathFormat::Csv => read_spectrum1d_csv(&read_text_file(path)?),
        Spectrum1DPathFormat::JeolJdf => read_jeol_jdf_1d_file(path),
        Spectrum1DPathFormat::BrukerProcessed => read_bruker_processed_1d_dir(path),
        Spectrum1DPathFormat::BrukerFid => read_bruker_fid_1d_dir(path),
        Spectrum1DPathFormat::AgilentFid => read_agilent_fid_1d_dir(path),
    }
}

/// Reads a two-dimensional spectrum from an auto-detected path.
///
/// # Errors
///
/// Returns an error when the format cannot be detected or the selected parser
/// rejects the path contents.
pub fn read_spectrum2d_path(path: impl AsRef<Path>) -> Result<Spectrum2D> {
    let path = path.as_ref();
    match detect_spectrum2d_path_format(path)? {
        Spectrum2DPathFormat::Json => read_spectrum2d_json(&read_text_file(path)?),
        Spectrum2DPathFormat::NmrMl => read_nmrml_2d_str(&read_text_file(path)?),
        Spectrum2DPathFormat::Csv => read_spectrum2d_csv(&read_text_file(path)?),
        Spectrum2DPathFormat::JeolJdf => read_jeol_jdf_2d_file(path),
        Spectrum2DPathFormat::BrukerProcessed => read_bruker_processed_2d_dir(path),
        Spectrum2DPathFormat::BrukerSer => read_bruker_ser_2d_dir(path),
        Spectrum2DPathFormat::AgilentFid => read_agilent_fid_2d_dir(path),
    }
}

/// Writes a one-dimensional spectrum to JSON, nmrML, JCAMP-DX, or CSV by extension.
///
/// # Errors
///
/// Returns an error when the extension is unsupported, the spectrum cannot be
/// represented by the selected writer, or the file cannot be written.
pub fn write_spectrum1d_path(spectrum: &Spectrum1D, path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    let payload = match detect_spectrum1d_write_path_format(path)? {
        Spectrum1DWritePathFormat::Json => write_spectrum1d_json(spectrum)?,
        Spectrum1DWritePathFormat::NmrMl => write_nmrml_1d(spectrum)?,
        Spectrum1DWritePathFormat::JcampDx => write_jcamp_dx_1d(spectrum)?,
        Spectrum1DWritePathFormat::Csv => write_spectrum1d_csv(spectrum)?,
    };
    write_text_file(path, &payload)
}

/// Writes a two-dimensional spectrum to JSON, nmrML, or CSV by extension.
///
/// # Errors
///
/// Returns an error when the extension is unsupported, the spectrum cannot be
/// represented by the selected writer, or the file cannot be written.
pub fn write_spectrum2d_path(spectrum: &Spectrum2D, path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    let payload = match detect_spectrum2d_write_path_format(path)? {
        Spectrum2DWritePathFormat::Json => write_spectrum2d_json(spectrum)?,
        Spectrum2DWritePathFormat::NmrMl => write_nmrml_2d(spectrum)?,
        Spectrum2DWritePathFormat::Csv => write_spectrum2d_csv(spectrum)?,
    };
    write_text_file(path, &payload)
}

fn looks_like_nmrml(input: &str) -> bool {
    (input.starts_with("<nmrML") || input.starts_with("<?xml"))
        && contains_ascii_case_insensitive(input, "<nmrML")
}

fn looks_like_jcamp_dx(input: &str) -> bool {
    contains_ascii_case_insensitive(input, "##TITLE=")
        || contains_ascii_case_insensitive(input, "##JCAMP-DX=")
        || contains_ascii_case_insensitive(input, "##XYDATA=")
        || contains_ascii_case_insensitive(input, "##DATA TABLE=")
}

fn looks_like_csv(input: &str) -> bool {
    input
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .any(|line| line.contains(','))
}

fn contains_ascii_case_insensitive(input: &str, needle: &str) -> bool {
    let needle = needle.as_bytes();
    input
        .as_bytes()
        .windows(needle.len())
        .any(|window| window.eq_ignore_ascii_case(needle))
}

fn text_format_from_path(path: &Path) -> Result<SpectrumTextFormat> {
    if is_extension(path, &["json"]) {
        return Ok(SpectrumTextFormat::Json);
    }
    if is_extension(path, &["nmrml", "xml"]) {
        return Ok(SpectrumTextFormat::NmrMl);
    }
    if is_extension(path, &["jdx", "dx"]) {
        return Ok(SpectrumTextFormat::JcampDx);
    }
    if is_extension(path, &["csv"]) {
        return Ok(SpectrumTextFormat::Csv);
    }
    detect_spectrum_text_format(&read_text_file(path)?)
}

fn ensure_path_exists(path: &Path) -> Result<()> {
    if path.exists() {
        Ok(())
    } else {
        Err(RSpinError::Parse {
            format: "spectrum path",
            message: format!("{} does not exist", path.display()),
        })
    }
}

fn read_text_file(path: &Path) -> Result<String> {
    fs::read_to_string(path).map_err(|error| RSpinError::Parse {
        format: "text spectrum file",
        message: format!("failed to read {}: {error}", path.display()),
    })
}

fn write_text_file(path: &Path, payload: &str) -> Result<()> {
    fs::write(path, payload).map_err(|error| RSpinError::Parse {
        format: "text spectrum file",
        message: format!("failed to write {}: {error}", path.display()),
    })
}

fn looks_like_bruker_processed_1d(path: &Path) -> bool {
    let processed = processed_dir(path);
    processed.join("procs").is_file() && processed.join("1r").is_file()
}

fn looks_like_bruker_processed_2d(path: &Path) -> bool {
    let processed = processed_dir(path);
    processed.join("procs").is_file()
        && processed.join("proc2s").is_file()
        && processed.join("2rr").is_file()
}

fn looks_like_bruker_fid(path: &Path) -> bool {
    let dataset = dataset_dir(path);
    dataset.join("fid").is_file() && dataset.join("acqus").is_file()
}

fn looks_like_bruker_ser(path: &Path) -> bool {
    let dataset = dataset_dir(path);
    dataset.join("ser").is_file()
        && dataset.join("acqus").is_file()
        && dataset.join("acqu2s").is_file()
}

fn looks_like_agilent_fid(path: &Path) -> bool {
    let dataset = dataset_dir(path);
    dataset.join("fid").is_file() && dataset.join("procpar").is_file()
}

fn processed_dir(path: &Path) -> std::path::PathBuf {
    if path.join("pdata").join("1").is_dir() {
        path.join("pdata").join("1")
    } else if path.is_file() {
        path.parent()
            .map_or_else(std::path::PathBuf::new, Path::to_path_buf)
    } else {
        path.to_path_buf()
    }
}

fn dataset_dir(path: &Path) -> std::path::PathBuf {
    if path.is_file() {
        path.parent()
            .map_or_else(std::path::PathBuf::new, Path::to_path_buf)
    } else {
        path.to_path_buf()
    }
}

fn is_extension(path: &Path, candidates: &[&str]) -> bool {
    path.extension()
        .and_then(std::ffi::OsStr::to_str)
        .is_some_and(|extension| {
            candidates
                .iter()
                .any(|candidate| extension.eq_ignore_ascii_case(candidate))
        })
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use rspin_core::{Axis, Metadata, Unit};

    use super::*;
    use crate::{write_spectrum1d_json, write_spectrum2d_json};

    #[test]
    fn detects_text_formats() -> Result<()> {
        assert_eq!(
            detect_spectrum_text_format(r#"{"x":{"label":"x"}}"#)?,
            SpectrumTextFormat::Json
        );
        assert_eq!(
            detect_spectrum_text_format(
                r#"<?xml version="1.0"?><nmrML version="v1.0.rc1"></nmrML>"#
            )?,
            SpectrumTextFormat::NmrMl
        );
        assert_eq!(
            detect_spectrum_text_format("##TITLE=demo\n##XYDATA=(X++(Y..Y))\n")?,
            SpectrumTextFormat::JcampDx
        );
        assert_eq!(
            detect_spectrum_text_format("# name=demo\nx,intensity\n0,1\n")?,
            SpectrumTextFormat::Csv
        );
        Ok(())
    }

    #[test]
    fn reads_json_1d_text() -> Result<()> {
        let spectrum = Spectrum1D::new(
            Axis::linear("shift", Unit::Ppm, 0.0, 2.0, 3)?,
            vec![1.0, 2.0, 3.0],
            Metadata::named("one"),
        )?;
        let text = write_spectrum1d_json(&spectrum)?;
        let parsed = read_spectrum1d_text(&text)?;
        assert_eq!(parsed, spectrum);
        Ok(())
    }

    #[test]
    fn reads_csv_1d_text_with_trait_api() -> Result<()> {
        let input = "# name=csv\n# x_unit=ppm\nx,intensity\n0.0,1.0\n1.0,2.0\n";
        let parsed = SpectrumReader::read_str(&AutoSpectrum1DText, input)?;
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed.metadata.name.as_deref(), Some("csv"));
        assert_eq!(parsed.intensities, vec![1.0, 2.0]);
        Ok(())
    }

    #[test]
    fn reads_nmrml_2d_text() -> Result<()> {
        let input = r#"
            <?xml version="1.0" encoding="UTF-8"?>
            <nmrML version="v1.0.rc1">
              <acquisition>
                <acquisitionMultiD>
                  <acquisitionParameterSet>
                    <directDimensionParameterSet decoupled="false" numberOfDataPoints="2"/>
                    <indirectDimensionParameterSet decoupled="false" numberOfDataPoints="1"/>
                  </acquisitionParameterSet>
                </acquisitionMultiD>
              </acquisition>
              <spectrumList>
                <spectrumMultiD id="processed" numberOfDataPoints="2">
                  <spectrumDataArray compressed="false" encodedLength="24" byteFormat="float64">
                    AAAAAAAA8D8AAAAAAAAAQA==
                  </spectrumDataArray>
                  <xAxis unitName="parts per million" startValue="1.0" endValue="0.0"/>
                  <firstDimensionProcessingParameterSet/>
                  <higherDimensionProcessingParameterSet/>
                </spectrumMultiD>
              </spectrumList>
            </nmrML>
        "#;

        let parsed = read_spectrum2d_text(input)?;
        assert_eq!(parsed.shape(), (2, 1));
        assert_eq!(parsed.z, vec![1.0, 2.0]);
        Ok(())
    }

    #[test]
    fn reads_json_2d_text_with_trait_api() -> Result<()> {
        let spectrum = Spectrum2D::new(
            Axis::linear("x", Unit::Ppm, 0.0, 1.0, 2)?,
            Axis::linear("y", Unit::Ppm, 10.0, 10.0, 1)?,
            vec![1.0, 2.0],
            Metadata::named("two"),
        )?;
        let text = write_spectrum2d_json(&spectrum)?;
        let parsed = SpectrumReader::read_str(&AutoSpectrum2DText, &text)?;
        assert_eq!(parsed, spectrum);
        Ok(())
    }

    #[test]
    fn rejects_empty_text() {
        let error = detect_spectrum_text_format(" \n\t").expect_err("empty input should fail");
        assert!(matches!(error, RSpinError::Parse { .. }));
    }

    #[test]
    fn rejects_jcamp_as_2d_text() {
        let error = read_spectrum2d_text("##TITLE=demo\n##XYDATA=(X++(Y..Y))\n")
            .expect_err("2D JCAMP text is not supported");
        assert!(matches!(error, RSpinError::Unsupported { .. }));
    }

    #[test]
    fn reads_csv_1d_path_with_trait_api() -> anyhow::Result<()> {
        let root = temp_dir("csv-1d")?;
        let path = root.join("one.csv");
        fs::write(
            &path,
            "\
# name=path one
# x_unit=PPM
x,intensity
0,1
1,2
",
        )?;

        assert_eq!(
            detect_spectrum1d_path_format(&path)?,
            Spectrum1DPathFormat::Csv
        );
        let parsed = SpectrumPathReader::read_path(&AutoSpectrum1DPath, &path)?;

        assert_eq!(parsed.metadata.name.as_deref(), Some("path one"));
        assert_eq!(parsed.x.unit, Unit::Ppm);
        assert_eq!(parsed.intensities, vec![1.0, 2.0]);

        remove_dir(root)?;
        Ok(())
    }

    #[test]
    fn reads_json_2d_path_by_content() -> anyhow::Result<()> {
        let root = temp_dir("json-2d")?;
        let path = root.join("two.spectrum");
        let spectrum = Spectrum2D::new(
            Axis::linear("x", Unit::Ppm, 0.0, 1.0, 2)?,
            Axis::linear("y", Unit::Ppm, 10.0, 10.0, 1)?,
            vec![1.0, 2.0],
            Metadata::named("path two"),
        )?;
        fs::write(&path, write_spectrum2d_json(&spectrum)?)?;

        assert_eq!(
            detect_spectrum2d_path_format(&path)?,
            Spectrum2DPathFormat::Json
        );
        let parsed = AutoSpectrum2DPath.read_path(&path)?;

        assert_eq!(parsed, spectrum);

        remove_dir(root)?;
        Ok(())
    }

    #[test]
    fn writes_1d_paths_by_extension() -> anyhow::Result<()> {
        let root = temp_dir("write-1d")?;
        let spectrum = Spectrum1D::new(
            Axis::linear("x", Unit::Ppm, 0.0, 2.0, 3)?,
            vec![1.0, 2.0, 3.0],
            Metadata::named("auto write one"),
        )?;

        let json_path = root.join("one.json");
        let csv_path = root.join("one.csv");
        let nmrml_path = root.join("one.nmrml");
        let jcamp_path = root.join("one.jdx");

        assert_eq!(
            detect_spectrum1d_write_path_format(&json_path)?,
            Spectrum1DWritePathFormat::Json
        );
        assert_eq!(
            detect_spectrum1d_write_path_format(&csv_path)?,
            Spectrum1DWritePathFormat::Csv
        );
        assert_eq!(
            detect_spectrum1d_write_path_format(&nmrml_path)?,
            Spectrum1DWritePathFormat::NmrMl
        );
        assert_eq!(
            detect_spectrum1d_write_path_format(&jcamp_path)?,
            Spectrum1DWritePathFormat::JcampDx
        );

        AutoSpectrum1DPathWriter.write_path(&spectrum, &json_path)?;
        write_spectrum1d_path(&spectrum, &csv_path)?;
        write_spectrum1d_path(&spectrum, &nmrml_path)?;
        write_spectrum1d_path(&spectrum, &jcamp_path)?;

        assert_eq!(read_spectrum1d_path(&json_path)?, spectrum);

        let csv = read_spectrum1d_path(&csv_path)?;
        assert_eq!(csv.x, spectrum.x);
        assert_eq!(csv.intensities, spectrum.intensities);
        assert_eq!(csv.metadata.name, spectrum.metadata.name);

        let nmrml = read_spectrum1d_path(&nmrml_path)?;
        assert_eq!(nmrml.x.unit, spectrum.x.unit);
        assert_eq!(nmrml.x.values, spectrum.x.values);
        assert_eq!(nmrml.intensities, spectrum.intensities);
        assert_eq!(nmrml.metadata.name, spectrum.metadata.name);

        let jcamp = read_spectrum1d_path(&jcamp_path)?;
        assert_eq!(jcamp.x, spectrum.x);
        assert_eq!(jcamp.intensities, spectrum.intensities);
        assert_eq!(jcamp.metadata.name, spectrum.metadata.name);

        remove_dir(root)?;
        Ok(())
    }

    #[test]
    fn writes_2d_paths_by_extension() -> anyhow::Result<()> {
        let root = temp_dir("write-2d")?;
        let spectrum = Spectrum2D::new(
            Axis::linear("x", Unit::Ppm, 0.0, 1.0, 2)?,
            Axis::linear("y", Unit::Ppm, 10.0, 11.0, 2)?,
            vec![1.0, 2.0, 3.0, 4.0],
            Metadata::named("auto write two"),
        )?;

        let json_path = root.join("two.json");
        let csv_path = root.join("two.csv");
        let nmrml_path = root.join("two.nmrml");

        assert_eq!(
            detect_spectrum2d_write_path_format(&json_path)?,
            Spectrum2DWritePathFormat::Json
        );
        assert_eq!(
            detect_spectrum2d_write_path_format(&csv_path)?,
            Spectrum2DWritePathFormat::Csv
        );
        assert_eq!(
            detect_spectrum2d_write_path_format(&nmrml_path)?,
            Spectrum2DWritePathFormat::NmrMl
        );

        AutoSpectrum2DPathWriter.write_path(&spectrum, &json_path)?;
        write_spectrum2d_path(&spectrum, &csv_path)?;
        write_spectrum2d_path(&spectrum, &nmrml_path)?;

        assert_eq!(read_spectrum2d_path(&json_path)?, spectrum);

        let csv = read_spectrum2d_path(&csv_path)?;
        assert_eq!(csv.x, spectrum.x);
        assert_eq!(csv.y, spectrum.y);
        assert_eq!(csv.z, spectrum.z);
        assert_eq!(csv.metadata.name, spectrum.metadata.name);

        let nmrml = read_spectrum2d_path(&nmrml_path)?;
        assert_eq!(nmrml.x.unit, spectrum.x.unit);
        assert_eq!(nmrml.x.values, spectrum.x.values);
        assert_eq!(nmrml.y.unit, spectrum.y.unit);
        assert_eq!(nmrml.y.values, spectrum.y.values);
        assert_eq!(nmrml.z, spectrum.z);
        assert_eq!(nmrml.metadata.name, spectrum.metadata.name);

        remove_dir(root)?;
        Ok(())
    }

    #[test]
    fn rejects_unsupported_write_path_extensions() {
        let error = detect_spectrum1d_write_path_format("one.bin")
            .expect_err("unsupported 1D write extension should fail");
        assert!(matches!(error, RSpinError::Unsupported { .. }));

        let error = detect_spectrum2d_write_path_format("two.dx")
            .expect_err("2D JCAMP-DX path writer should not be supported");
        assert!(matches!(error, RSpinError::Unsupported { .. }));
    }

    #[test]
    fn reads_bruker_processed_1d_path() -> anyhow::Result<()> {
        let root = temp_dir("bruker-1d")?;
        let processed = root.join("pdata/1");
        fs::create_dir_all(&processed)?;
        fs::write(
            processed.join("procs"),
            "\
##$SI= 3
##$BYTORDP= 0
##$DTYPP= 0
##$NC_proc= 0
",
        )?;
        fs::write(processed.join("1r"), i32_bytes(&[1, -2, 3]))?;

        assert_eq!(
            detect_spectrum1d_path_format(&root)?,
            Spectrum1DPathFormat::BrukerProcessed
        );
        let parsed = read_spectrum1d_path(&root)?;

        assert_eq!(parsed.x.unit, Unit::Points);
        assert_eq!(parsed.x.values, vec![0.0, 1.0, 2.0]);
        assert_eq!(parsed.intensities, vec![1.0, -2.0, 3.0]);

        remove_dir(root)?;
        Ok(())
    }

    #[test]
    fn reads_bruker_processed_2d_path() -> anyhow::Result<()> {
        let root = temp_dir("bruker-2d")?;
        let processed = root.join("pdata/1");
        fs::create_dir_all(&processed)?;
        fs::write(
            processed.join("procs"),
            "\
##$SI= 2
##$BYTORDP= 0
##$DTYPP= 0
##$NC_proc= 0
",
        )?;
        fs::write(
            processed.join("proc2s"),
            "\
##$SI= 2
",
        )?;
        fs::write(processed.join("2rr"), i32_bytes(&[1, 2, 3, 4]))?;

        assert_eq!(
            detect_spectrum2d_path_format(&root)?,
            Spectrum2DPathFormat::BrukerProcessed
        );
        let parsed = read_spectrum2d_path(&root)?;

        assert_eq!(parsed.shape(), (2, 2));
        assert_eq!(parsed.x.unit, Unit::Points);
        assert_eq!(parsed.y.unit, Unit::Points);
        assert_eq!(parsed.z, vec![1.0, 2.0, 3.0, 4.0]);

        remove_dir(root)?;
        Ok(())
    }

    fn temp_dir(name: &str) -> anyhow::Result<PathBuf> {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let mut path = std::env::temp_dir();
        path.push(format!("rspin-auto-{name}-{}-{nanos}", std::process::id()));
        fs::create_dir_all(&path)?;
        Ok(path)
    }

    fn remove_dir(path: PathBuf) -> anyhow::Result<()> {
        fs::remove_dir_all(path)?;
        Ok(())
    }

    fn i32_bytes(values: &[i32]) -> Vec<u8> {
        values
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect::<Vec<_>>()
    }
}
