//! CSV import and export for prediction payloads.

use std::str::FromStr;

use rspin_core::{Nucleus, RSpinError, Result};
use rspin_prediction::{
    Experiment, PredictedCorrelation2D, PredictedSignal1D, PredictionProvenance, PredictionSet,
};

use crate::{
    SpectrumReader, SpectrumWriter,
    csv_common::{format_float, normalized_key, parse_float, push_comment},
};

/// Reader and writer for prediction CSV payloads.
#[derive(Clone, Copy, Debug, Default)]
pub struct CsvPrediction;

impl SpectrumReader for CsvPrediction {
    type Output = PredictionSet;

    fn read_str(&self, input: &str) -> Result<Self::Output> {
        read_prediction_csv(input)
    }
}

impl SpectrumWriter<PredictionSet> for CsvPrediction {
    fn write_string(&self, prediction: &PredictionSet) -> Result<String> {
        write_prediction_csv(prediction)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Section {
    Signals1D,
    Correlations2D,
}

#[derive(Debug, Default)]
struct CsvState {
    prediction: PredictionSet,
    provenance_source: Option<String>,
    provenance_version: Option<String>,
    section: Option<Section>,
    expects_header: bool,
    saw_section: bool,
}

impl CsvState {
    fn finish(mut self) -> Result<PredictionSet> {
        if self.expects_header {
            return Err(parse_error(
                "prediction CSV section is missing a header row",
            ));
        }
        if let Some(source) = self.provenance_source {
            if source.trim().is_empty() {
                return Err(parse_error(
                    "prediction provenance source must not be empty",
                ));
            }
            let mut provenance = PredictionProvenance::new(source);
            if let Some(version) = self.provenance_version {
                provenance.version = Some(version);
            }
            self.prediction.provenance = Some(provenance);
        } else if self.provenance_version.is_some() {
            return Err(parse_error(
                "prediction provenance version requires provenance source",
            ));
        }
        self.prediction.validate()?;
        Ok(self.prediction)
    }
}

/// Reads a prediction payload from multi-section CSV.
///
/// The CSV may contain `signals_1d` and `correlations_2d` sections. Assignment
/// columns are compact JSON string arrays so assignment labels round-trip even
/// when they contain commas or quotes.
///
/// # Errors
///
/// Returns an error when section headers, row widths, numeric fields, nuclei,
/// assignment JSON, or prediction validation fail.
pub fn read_prediction_csv(input: &str) -> Result<PredictionSet> {
    let mut state = CsvState::default();
    for (line_index, line) in input.lines().enumerate() {
        let line_number = line_index + 1;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(comment) = trimmed.strip_prefix('#') {
            apply_comment(&mut state, comment.trim(), line_number)?;
            continue;
        }
        let section = state.section.ok_or_else(|| {
            parse_error(format!(
                "line {line_number}: prediction CSV row appears before a section"
            ))
        })?;
        let columns = split_csv_row(trimmed, line_number)?;
        if state.expects_header {
            validate_header(section, &columns, line_number)?;
            state.expects_header = false;
            continue;
        }
        parse_section_row(&mut state.prediction, section, &columns, line_number)?;
    }

    if !state.saw_section {
        return Err(parse_error("missing prediction CSV sections"));
    }
    state.finish()
}

/// Writes a prediction payload to multi-section CSV.
///
/// The output contains `signals_1d` and `correlations_2d` sections. Assignment
/// vectors are stored as compact JSON arrays inside CSV fields.
///
/// # Errors
///
/// Returns an error when the prediction contains invalid numeric values,
/// confidence scores, or assignment data that cannot be serialized.
pub fn write_prediction_csv(prediction: &PredictionSet) -> Result<String> {
    prediction.validate()?;

    let mut output = String::new();
    push_comment(&mut output, "format", "RSpin Prediction CSV");
    if let Some(name) = &prediction.name {
        push_comment(&mut output, "name", name);
    }
    if let Some(provenance) = &prediction.provenance {
        push_comment(&mut output, "provenance.source", &provenance.source);
        if let Some(version) = &provenance.version {
            push_comment(&mut output, "provenance.version", version);
        }
    }
    write_signal_section(&mut output, &prediction.signals_1d)?;
    write_correlation_section(&mut output, &prediction.correlations_2d)?;
    Ok(output)
}

fn apply_comment(state: &mut CsvState, comment: &str, line_number: usize) -> Result<()> {
    let Some((key, value)) = comment.split_once('=') else {
        return Ok(());
    };
    let value = value.trim();
    match normalized_key(key).as_str() {
        "name" => state.prediction.name = Some(value.to_owned()),
        "provenancesource" => state.provenance_source = Some(value.to_owned()),
        "provenanceversion" => state.provenance_version = Some(value.to_owned()),
        "section" => {
            state.section = Some(parse_section(value, line_number)?);
            state.expects_header = true;
            state.saw_section = true;
        }
        _ => {}
    }
    Ok(())
}

fn parse_section(value: &str, line_number: usize) -> Result<Section> {
    match normalized_key(value).as_str() {
        "signals1d" | "signals" => Ok(Section::Signals1D),
        "correlations2d" | "correlations" => Ok(Section::Correlations2D),
        _ => Err(parse_error(format!(
            "line {line_number}: unsupported prediction CSV section '{value}'"
        ))),
    }
}

fn validate_header(section: Section, columns: &[String], line_number: usize) -> Result<()> {
    let expected: &[&str] = match section {
        Section::Signals1D => &[
            "experiment",
            "nucleus",
            "delta_ppm",
            "intensity",
            "confidence",
            "assignments_json",
        ],
        Section::Correlations2D => &[
            "experiment",
            "x_nucleus",
            "y_nucleus",
            "x_ppm",
            "y_ppm",
            "intensity",
            "confidence",
            "assignments_json",
        ],
    };
    if columns.len() != expected.len() {
        return Err(parse_error(format!(
            "line {line_number}: expected {} prediction CSV header columns but found {}",
            expected.len(),
            columns.len()
        )));
    }
    for (actual, expected) in columns.iter().zip(expected) {
        if normalized_key(actual) != normalized_key(expected) {
            return Err(parse_error(format!(
                "line {line_number}: expected prediction CSV column '{expected}' but found '{actual}'"
            )));
        }
    }
    Ok(())
}

fn parse_section_row(
    prediction: &mut PredictionSet,
    section: Section,
    columns: &[String],
    line_number: usize,
) -> Result<()> {
    match section {
        Section::Signals1D => {
            require_columns(columns, 6, line_number)?;
            prediction.signals_1d.push(PredictedSignal1D {
                experiment: parse_experiment(&columns[0], line_number)?,
                nucleus: Nucleus::from_str(&columns[1])?,
                delta_ppm: parse_float("prediction delta_ppm", &columns[2])?,
                intensity: parse_float("prediction intensity", &columns[3])?,
                confidence: parse_optional_float("prediction confidence", &columns[4])?,
                assignments: parse_assignments(&columns[5], line_number)?,
            });
        }
        Section::Correlations2D => {
            require_columns(columns, 8, line_number)?;
            prediction.correlations_2d.push(PredictedCorrelation2D {
                experiment: parse_experiment(&columns[0], line_number)?,
                x_nucleus: Nucleus::from_str(&columns[1])?,
                y_nucleus: Nucleus::from_str(&columns[2])?,
                x_ppm: parse_float("prediction x_ppm", &columns[3])?,
                y_ppm: parse_float("prediction y_ppm", &columns[4])?,
                intensity: parse_float("prediction intensity", &columns[5])?,
                confidence: parse_optional_float("prediction confidence", &columns[6])?,
                assignments: parse_assignments(&columns[7], line_number)?,
            });
        }
    }
    Ok(())
}

fn require_columns(columns: &[String], expected: usize, line_number: usize) -> Result<()> {
    if columns.len() == expected {
        Ok(())
    } else {
        Err(parse_error(format!(
            "line {line_number}: expected {expected} prediction CSV columns but found {}",
            columns.len()
        )))
    }
}

fn parse_optional_float(field: &'static str, value: &str) -> Result<Option<f64>> {
    if value.trim().is_empty() {
        Ok(None)
    } else {
        parse_float(field, value).map(Some)
    }
}

fn parse_assignments(value: &str, line_number: usize) -> Result<Vec<String>> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    if trimmed.starts_with('[') {
        serde_json::from_str::<Vec<String>>(trimmed).map_err(|error| {
            parse_error(format!(
                "line {line_number}: invalid assignments_json: {error}"
            ))
        })
    } else {
        Ok(trimmed
            .split(';')
            .map(str::trim)
            .filter(|assignment| !assignment.is_empty())
            .map(str::to_owned)
            .collect())
    }
}

fn parse_experiment(value: &str, line_number: usize) -> Result<Experiment> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(parse_error(format!(
            "line {line_number}: empty prediction experiment"
        )));
    }
    match normalized_key(trimmed).as_str() {
        "proton1d" | "1h" | "h1" => Ok(Experiment::Proton1D),
        "carbon131d" | "13c" | "c13" => Ok(Experiment::Carbon13_1D),
        "cosy" => Ok(Experiment::Cosy),
        "hsqc" => Ok(Experiment::Hsqc),
        "hmbc" => Ok(Experiment::Hmbc),
        _ => trimmed.strip_prefix("Other:").map_or_else(
            || Ok(Experiment::Other(trimmed.to_owned())),
            other_experiment,
        ),
    }
}

fn other_experiment(value: &str) -> Result<Experiment> {
    if value.trim().is_empty() {
        Err(parse_error(
            "prediction Other experiment label must not be empty",
        ))
    } else {
        Ok(Experiment::Other(value.to_owned()))
    }
}

fn write_signal_section(output: &mut String, signals: &[PredictedSignal1D]) -> Result<()> {
    section(output, "signals_1d");
    row(
        output,
        [
            "experiment",
            "nucleus",
            "delta_ppm",
            "intensity",
            "confidence",
            "assignments_json",
        ],
    );
    for signal in signals {
        row(
            output,
            &[
                experiment_label(&signal.experiment),
                signal.nucleus.to_string(),
                finite("prediction delta_ppm", signal.delta_ppm)?,
                finite("prediction intensity", signal.intensity)?,
                optional_finite("prediction confidence", signal.confidence)?,
                assignments_json(&signal.assignments)?,
            ],
        );
    }
    Ok(())
}

fn write_correlation_section(
    output: &mut String,
    correlations: &[PredictedCorrelation2D],
) -> Result<()> {
    section(output, "correlations_2d");
    row(
        output,
        [
            "experiment",
            "x_nucleus",
            "y_nucleus",
            "x_ppm",
            "y_ppm",
            "intensity",
            "confidence",
            "assignments_json",
        ],
    );
    for correlation in correlations {
        row(
            output,
            &[
                experiment_label(&correlation.experiment),
                correlation.x_nucleus.to_string(),
                correlation.y_nucleus.to_string(),
                finite("prediction x_ppm", correlation.x_ppm)?,
                finite("prediction y_ppm", correlation.y_ppm)?,
                finite("prediction intensity", correlation.intensity)?,
                optional_finite("prediction confidence", correlation.confidence)?,
                assignments_json(&correlation.assignments)?,
            ],
        );
    }
    Ok(())
}

fn section(output: &mut String, name: &str) {
    push_comment(output, "section", name);
}

fn row<I, S>(output: &mut String, columns: I)
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut first = true;
    for column in columns {
        if first {
            first = false;
        } else {
            output.push(',');
        }
        push_csv_field(output, column.as_ref());
    }
    output.push('\n');
}

fn push_csv_field(output: &mut String, field: &str) {
    if field.contains(',') || field.contains('"') || field.contains('\n') || field.contains('\r') {
        output.push('"');
        for character in field.chars() {
            if character == '"' {
                output.push('"');
            }
            output.push(character);
        }
        output.push('"');
    } else {
        output.push_str(field);
    }
}

fn split_csv_row(line: &str, line_number: usize) -> Result<Vec<String>> {
    let mut columns = Vec::new();
    let mut column = String::new();
    let mut chars = line.chars().peekable();
    let mut in_quotes = false;

    while let Some(character) = chars.next() {
        match character {
            '"' if in_quotes && chars.peek().is_some_and(|next| *next == '"') => {
                column.push('"');
                let _ = chars.next();
            }
            '"' => in_quotes = !in_quotes,
            ',' if !in_quotes => {
                columns.push(column.trim().to_owned());
                column.clear();
            }
            _ => column.push(character),
        }
    }

    if in_quotes {
        return Err(parse_error(format!(
            "line {line_number}: unterminated quoted CSV field"
        )));
    }
    columns.push(column.trim().to_owned());
    Ok(columns)
}

fn experiment_label(experiment: &Experiment) -> String {
    match experiment {
        Experiment::Proton1D => "Proton1D".to_owned(),
        Experiment::Carbon13_1D => "Carbon13_1D".to_owned(),
        Experiment::Cosy => "Cosy".to_owned(),
        Experiment::Hsqc => "Hsqc".to_owned(),
        Experiment::Hmbc => "Hmbc".to_owned(),
        Experiment::Other(label) => format!("Other:{label}"),
        other => format!("{other:?}"),
    }
}

fn assignments_json(assignments: &[String]) -> Result<String> {
    serde_json::to_string(assignments).map_err(|error| {
        parse_error(format!(
            "failed to serialize prediction assignments: {error}"
        ))
    })
}

fn finite(field: &'static str, value: f64) -> Result<String> {
    if !value.is_finite() {
        return Err(RSpinError::NonFinite { field });
    }
    Ok(format_float(value))
}

fn optional_finite(field: &'static str, value: Option<f64>) -> Result<String> {
    value.map_or_else(|| Ok(String::new()), |value| finite(field, value))
}

fn parse_error(message: impl Into<String>) -> RSpinError {
    RSpinError::Parse {
        format: "CSV",
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_prediction_csv_with_trait_api() -> anyhow::Result<()> {
        let prediction = prediction_fixture();
        let codec = CsvPrediction;
        let text = codec.write_string(&prediction)?;
        let parsed = codec.read_str(&text)?;

        assert!(text.starts_with("# format=RSpin Prediction CSV\n"));
        assert!(text.contains("# section=signals_1d\n"));
        assert!(text.contains("# section=correlations_2d\n"));
        assert!(text.contains("H1, alpha"));
        assert_eq!(parsed, prediction);
        assert_eq!(format!("{codec:?}"), "CsvPrediction");
        Ok(())
    }

    #[test]
    fn reads_prediction_csv_with_manual_assignment_lists() -> anyhow::Result<()> {
        let csv = "\
# format=RSpin Prediction CSV
# name=manual
# provenance.source=fixture
# section=signals_1d
experiment,nucleus,delta_ppm,intensity,confidence,assignments_json
Proton1D,1H,1.25,2,,H1;H2
# section=correlations_2d
experiment,x_nucleus,y_nucleus,x_ppm,y_ppm,intensity,confidence,assignments_json
Hsqc,1H,13C,1.25,63,0.5,0.7,H1-C1
";
        let prediction = read_prediction_csv(csv)?;

        assert_eq!(prediction.name.as_deref(), Some("manual"));
        assert_eq!(prediction.signals_1d.len(), 1);
        assert_eq!(prediction.signals_1d[0].assignments, vec!["H1", "H2"]);
        assert_eq!(prediction.signals_1d[0].confidence, None);
        assert_eq!(prediction.correlations_2d.len(), 1);
        assert_eq!(prediction.correlations_2d[0].confidence, Some(0.7));
        assert_eq!(
            prediction
                .provenance
                .as_ref()
                .map(|item| item.source.as_str()),
            Some("fixture")
        );
        Ok(())
    }

    #[test]
    fn rejects_invalid_prediction_csv() {
        let missing_sections = read_prediction_csv("# format=RSpin Prediction CSV")
            .expect_err("sections are required");
        assert!(matches!(missing_sections, RSpinError::Parse { .. }));

        let wrong_header =
            read_prediction_csv("# section=signals_1d\nexperiment,nucleus\nProton1D,1H\n")
                .expect_err("wrong header should fail");
        assert!(matches!(wrong_header, RSpinError::Parse { .. }));

        let invalid_confidence = read_prediction_csv(
            "# section=signals_1d\nexperiment,nucleus,delta_ppm,intensity,confidence,assignments_json\nProton1D,1H,1,1,1.5,[]\n# section=correlations_2d\nexperiment,x_nucleus,y_nucleus,x_ppm,y_ppm,intensity,confidence,assignments_json\n",
        )
        .expect_err("invalid confidence should fail validation");
        assert!(matches!(
            invalid_confidence,
            RSpinError::InvalidSpectrum { .. }
        ));

        let invalid_assignments = read_prediction_csv(
            "# section=signals_1d\nexperiment,nucleus,delta_ppm,intensity,confidence,assignments_json\nProton1D,1H,1,1,,[1]\n# section=correlations_2d\nexperiment,x_nucleus,y_nucleus,x_ppm,y_ppm,intensity,confidence,assignments_json\n",
        )
        .expect_err("invalid assignments should fail");
        assert!(matches!(invalid_assignments, RSpinError::Parse { .. }));
    }

    #[test]
    fn rejects_non_finite_prediction_csv_export() {
        let prediction = PredictionSet::new().with_signal_1d(PredictedSignal1D {
            experiment: Experiment::Proton1D,
            nucleus: Nucleus::Hydrogen1,
            delta_ppm: f64::NAN,
            intensity: 1.0,
            confidence: None,
            assignments: Vec::new(),
        });

        let error =
            write_prediction_csv(&prediction).expect_err("non-finite prediction should fail");
        assert_eq!(error, RSpinError::NonFinite { field: "delta_ppm" });
    }

    fn prediction_fixture() -> PredictionSet {
        PredictionSet::new()
            .with_name("demo")
            .with_signal_1d(
                PredictedSignal1D::new(Experiment::Proton1D, Nucleus::Hydrogen1, 1.25)
                    .with_intensity(2.0)
                    .with_confidence(0.8)
                    .with_assignment("H1, alpha")
                    .with_assignment("H\"2"),
            )
            .with_correlation_2d(
                PredictedCorrelation2D::new(
                    Experiment::Hsqc,
                    Nucleus::Hydrogen1,
                    Nucleus::Carbon13,
                    1.25,
                    63.0,
                )
                .with_intensity(0.5)
                .with_confidence(0.7)
                .with_assignment("H1-C1"),
            )
            .with_provenance(PredictionProvenance::new("fixture").with_version("1"))
    }
}
