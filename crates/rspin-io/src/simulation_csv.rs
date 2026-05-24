//! CSV import and export for exact simulation payloads.

use rspin_core::{RSpinError, Result};
use rspin_simulation::ExactTransition;

use crate::{
    SpectrumReader, SpectrumWriter,
    csv_common::{format_float, parse_float},
};

/// Writer for exact transition line CSV.
#[derive(Clone, Copy, Debug, Default)]
pub struct CsvExactTransitions;

impl SpectrumReader for CsvExactTransitions {
    type Output = Vec<ExactTransition>;

    fn read_str(&self, input: &str) -> Result<Self::Output> {
        read_exact_transitions_csv(input)
    }
}

impl SpectrumWriter<[ExactTransition]> for CsvExactTransitions {
    fn write_string(&self, transitions: &[ExactTransition]) -> Result<String> {
        write_exact_transitions_csv(transitions)
    }
}

/// Reads exact spin-1/2 transition lines from CSV.
///
/// Comment rows beginning with `#` and empty rows are ignored. The first
/// non-comment row must be the header emitted by [`write_exact_transitions_csv`].
///
/// # Errors
///
/// Returns an error when the header, column count, or numeric fields are invalid.
pub fn read_exact_transitions_csv(input: &str) -> Result<Vec<ExactTransition>> {
    let mut saw_header = false;
    let mut transitions = Vec::new();
    for (line_index, line) in input.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let line_number = line_index + 1;
        if saw_header {
            transitions.push(parse_transition_row(trimmed, line_number)?);
        } else {
            validate_header(trimmed, line_number)?;
            saw_header = true;
        }
    }

    if saw_header {
        Ok(transitions)
    } else {
        Err(RSpinError::Parse {
            format: "CSV",
            message: "missing exact transition CSV header".to_owned(),
        })
    }
}

/// Writes exact spin-1/2 transition lines as CSV.
///
/// The output begins with a comment row identifying the payload, followed by one
/// row per transition in input order.
///
/// # Errors
///
/// Returns an error when any exported numeric field is not finite.
pub fn write_exact_transitions_csv(transitions: &[ExactTransition]) -> Result<String> {
    let mut output = String::new();
    output.push_str("# format=RSpin Exact Transitions CSV\n");
    row(
        &mut output,
        [
            "frequency_hz",
            "offset_hz",
            "center_ppm",
            "intensity",
            "contribution_count",
        ],
    );
    for transition in transitions {
        row(
            &mut output,
            &[
                finite("transition frequency_hz", transition.frequency_hz)?,
                finite("transition offset_hz", transition.offset_hz)?,
                finite("transition center_ppm", transition.center_ppm)?,
                finite("transition intensity", transition.intensity)?,
                transition.contribution_count.to_string(),
            ],
        );
    }
    Ok(output)
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
        output.push_str(column.as_ref());
    }
    output.push('\n');
}

fn finite(field: &'static str, value: f64) -> Result<String> {
    if !value.is_finite() {
        return Err(RSpinError::NonFinite { field });
    }
    Ok(format_float(value))
}

fn validate_header(line: &str, line_number: usize) -> Result<()> {
    let columns = split_columns(line);
    let expected = [
        "frequency_hz",
        "offset_hz",
        "center_ppm",
        "intensity",
        "contribution_count",
    ];
    if columns.len() != expected.len() {
        return Err(RSpinError::Parse {
            format: "CSV",
            message: format!(
                "line {line_number}: expected {} exact transition columns but found {}",
                expected.len(),
                columns.len()
            ),
        });
    }
    for (actual, expected) in columns.iter().zip(expected) {
        if *actual != expected {
            return Err(RSpinError::Parse {
                format: "CSV",
                message: format!(
                    "line {line_number}: expected exact transition column '{expected}' but found '{actual}'"
                ),
            });
        }
    }
    Ok(())
}

fn parse_transition_row(line: &str, line_number: usize) -> Result<ExactTransition> {
    let columns = split_columns(line);
    if columns.len() != 5 {
        return Err(RSpinError::Parse {
            format: "CSV",
            message: format!(
                "line {line_number}: expected 5 exact transition columns but found {}",
                columns.len()
            ),
        });
    }

    Ok(ExactTransition {
        frequency_hz: parse_float("transition frequency_hz", columns[0])?,
        offset_hz: parse_float("transition offset_hz", columns[1])?,
        center_ppm: parse_float("transition center_ppm", columns[2])?,
        intensity: parse_float("transition intensity", columns[3])?,
        contribution_count: parse_contribution_count(columns[4], line_number)?,
    })
}

fn parse_contribution_count(value: &str, line_number: usize) -> Result<u32> {
    value
        .trim()
        .parse::<u32>()
        .map_err(|error| RSpinError::Parse {
            format: "CSV",
            message: format!("line {line_number}: contribution_count: {error}"),
        })
}

fn split_columns(line: &str) -> Vec<&str> {
    line.split(',').map(str::trim).collect()
}

#[cfg(test)]
mod tests {
    use rspin_simulation::{ExactSpinOptions, SpinHalfSystem, exact_spin_half_transitions};

    use super::*;

    #[test]
    fn writes_exact_transitions_csv() -> anyhow::Result<()> {
        let transitions = exact_spin_half_transitions(
            &SpinHalfSystem::new()
                .with_spin(1.0)
                .with_spin(1.02)
                .with_coupling(0, 1, 8.0),
            &ExactSpinOptions::new().with_spectrometer_mhz(400.0),
        )?;

        let csv = write_exact_transitions_csv(&transitions)?;

        assert!(csv.starts_with("# format=RSpin Exact Transitions CSV\n"));
        assert!(csv.contains("frequency_hz,offset_hz,center_ppm,intensity,contribution_count\n"));
        assert_eq!(csv.lines().count(), transitions.len() + 2);

        let codec = CsvExactTransitions;
        assert_eq!(codec.write_string(&transitions)?, csv);
        assert_transitions_close(&codec.read_str(&csv)?, &transitions);
        assert_transitions_close(&read_exact_transitions_csv(&csv)?, &transitions);
        Ok(())
    }

    #[test]
    fn reads_exact_transition_csv_with_comments_and_blank_lines() -> anyhow::Result<()> {
        let csv = "\
# format=RSpin Exact Transitions CSV

frequency_hz,offset_hz,center_ppm,intensity,contribution_count
400,0,1,0.5,2
404,4,1.01,0.25,1
";
        let transitions = read_exact_transitions_csv(csv)?;

        assert_eq!(transitions.len(), 2);
        assert!((transitions[0].frequency_hz - 400.0).abs() < 1.0e-12);
        assert!((transitions[1].center_ppm - 1.01).abs() < 1.0e-12);
        assert_eq!(transitions[0].contribution_count, 2);
        Ok(())
    }

    #[test]
    fn rejects_invalid_transition_csv() {
        let missing_header =
            read_exact_transitions_csv("# empty").expect_err("missing header should fail");
        assert!(matches!(missing_header, RSpinError::Parse { .. }));

        let wrong_columns = read_exact_transitions_csv("frequency_hz,offset_hz\n400,0")
            .expect_err("wrong header should fail");
        assert!(matches!(wrong_columns, RSpinError::Parse { .. }));

        let non_finite = read_exact_transitions_csv(
            "frequency_hz,offset_hz,center_ppm,intensity,contribution_count\nNaN,0,1,1,1",
        )
        .expect_err("non-finite value should fail");
        assert_eq!(
            non_finite,
            RSpinError::NonFinite {
                field: "transition frequency_hz"
            }
        );
    }

    #[test]
    fn rejects_non_finite_transition_values() {
        let transitions = [ExactTransition {
            frequency_hz: f64::NAN,
            offset_hz: 0.0,
            center_ppm: 0.0,
            intensity: 1.0,
            contribution_count: 1,
        }];

        let error =
            write_exact_transitions_csv(&transitions).expect_err("non-finite export should fail");
        assert_eq!(
            error,
            RSpinError::NonFinite {
                field: "transition frequency_hz"
            }
        );
    }

    fn assert_transitions_close(parsed: &[ExactTransition], expected: &[ExactTransition]) {
        assert_eq!(parsed.len(), expected.len());
        for (parsed_transition, expected_transition) in parsed.iter().zip(expected) {
            assert!(
                (parsed_transition.frequency_hz - expected_transition.frequency_hz).abs() < 1e-9
            );
            assert!((parsed_transition.offset_hz - expected_transition.offset_hz).abs() < 1e-9);
            assert!((parsed_transition.center_ppm - expected_transition.center_ppm).abs() < 1e-12);
            assert!((parsed_transition.intensity - expected_transition.intensity).abs() < 1e-12);
            assert_eq!(
                parsed_transition.contribution_count,
                expected_transition.contribution_count
            );
        }
    }
}
