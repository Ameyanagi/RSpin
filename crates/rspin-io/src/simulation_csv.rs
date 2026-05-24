//! CSV export for exact simulation payloads.

use rspin_core::{RSpinError, Result};
use rspin_simulation::ExactTransition;

use crate::{SpectrumWriter, csv_common::format_float};

/// Writer for exact transition line CSV.
#[derive(Clone, Copy, Debug, Default)]
pub struct CsvExactTransitions;

impl SpectrumWriter<[ExactTransition]> for CsvExactTransitions {
    fn write_string(&self, transitions: &[ExactTransition]) -> Result<String> {
        write_exact_transitions_csv(transitions)
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
        Ok(())
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
}
