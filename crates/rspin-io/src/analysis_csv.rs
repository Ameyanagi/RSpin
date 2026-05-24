//! CSV export for analysis workflow results.

use rspin_analysis::{
    DetectedMultiplet, DetectedRange, DetectedZone, OptimizedPeak, Peak, SignalSummary1D,
    SignalSummary2D, SpectrumAnalysis1D, SpectrumAnalysis2D,
};
use rspin_core::{RSpinError, Result};

use crate::{SpectrumWriter, csv_common::format_float};

/// Writer for one-dimensional analysis workflow CSV.
#[derive(Clone, Copy, Debug, Default)]
pub struct CsvAnalysis1D;

impl SpectrumWriter<SpectrumAnalysis1D> for CsvAnalysis1D {
    fn write_string(&self, analysis: &SpectrumAnalysis1D) -> Result<String> {
        write_analysis1d_csv(analysis)
    }
}

/// Writer for two-dimensional analysis workflow CSV.
#[derive(Clone, Copy, Debug, Default)]
pub struct CsvAnalysis2D;

impl SpectrumWriter<SpectrumAnalysis2D> for CsvAnalysis2D {
    fn write_string(&self, analysis: &SpectrumAnalysis2D) -> Result<String> {
        write_analysis2d_csv(analysis)
    }
}

/// Writes a one-dimensional analysis workflow result as multi-section CSV.
///
/// The output contains separate `peaks`, `optimized_peaks`, `ranges`,
/// `multiplets`, and `signals` sections, each with its own header row. Comment
/// rows beginning with `#` identify the file format and section boundaries.
///
/// # Errors
///
/// Returns an error when any exported numeric field is not finite.
pub fn write_analysis1d_csv(analysis: &SpectrumAnalysis1D) -> Result<String> {
    let mut output = String::new();
    output.push_str("# format=RSpin Analysis 1D CSV\n");
    write_peaks(&mut output, &analysis.peaks)?;
    write_optimized_peaks(&mut output, &analysis.optimized_peaks)?;
    write_ranges(&mut output, &analysis.ranges)?;
    write_multiplets(&mut output, &analysis.multiplets)?;
    write_signals_1d(&mut output, &analysis.signals)?;
    Ok(output)
}

/// Writes a two-dimensional analysis workflow result as multi-section CSV.
///
/// The output contains separate `zones` and `signals` sections, each with its
/// own header row. Comment rows beginning with `#` identify the file format and
/// section boundaries.
///
/// # Errors
///
/// Returns an error when any exported numeric field is not finite.
pub fn write_analysis2d_csv(analysis: &SpectrumAnalysis2D) -> Result<String> {
    let mut output = String::new();
    output.push_str("# format=RSpin Analysis 2D CSV\n");
    write_zones(&mut output, &analysis.zones)?;
    write_signals_2d(&mut output, &analysis.signals)?;
    Ok(output)
}

fn write_peaks(output: &mut String, peaks: &[Peak]) -> Result<()> {
    section(output, "peaks");
    row(
        output,
        ["index", "x", "intensity", "prominence", "polarity"],
    );
    for peak in peaks {
        row(
            output,
            &[
                peak.index.to_string(),
                finite("peak x", peak.x)?,
                finite("peak intensity", peak.intensity)?,
                finite("peak prominence", peak.prominence)?,
                format!("{:?}", peak.polarity),
            ],
        );
    }
    Ok(())
}

fn write_optimized_peaks(output: &mut String, optimized_peaks: &[OptimizedPeak]) -> Result<()> {
    section(output, "optimized_peaks");
    row(
        output,
        [
            "index",
            "original_x",
            "original_intensity",
            "optimized_x",
            "optimized_intensity",
            "delta_x",
            "curvature",
            "optimized",
        ],
    );
    for peak in optimized_peaks {
        row(
            output,
            &[
                peak.peak.index.to_string(),
                finite("optimized peak original x", peak.peak.x)?,
                finite("optimized peak original intensity", peak.peak.intensity)?,
                finite("optimized peak x", peak.x)?,
                finite("optimized peak intensity", peak.intensity)?,
                finite("optimized peak delta_x", peak.delta_x)?,
                optional_finite("optimized peak curvature", peak.curvature)?,
                peak.optimized.to_string(),
            ],
        );
    }
    Ok(())
}

fn write_ranges(output: &mut String, ranges: &[DetectedRange]) -> Result<()> {
    section(output, "ranges");
    row(
        output,
        [
            "start_index",
            "end_index",
            "from",
            "to",
            "active_points",
            "max_abs_intensity",
            "area",
        ],
    );
    for range in ranges {
        row(
            output,
            &[
                range.start_index.to_string(),
                range.end_index.to_string(),
                finite("range from", range.from)?,
                finite("range to", range.to)?,
                range.active_points.to_string(),
                finite("range max_abs_intensity", range.max_abs_intensity)?,
                finite("range area", range.area)?,
            ],
        );
    }
    Ok(())
}

fn write_multiplets(output: &mut String, multiplets: &[DetectedMultiplet]) -> Result<()> {
    section(output, "multiplets");
    row(
        output,
        [
            "id",
            "kind",
            "center_ppm",
            "from_ppm",
            "to_ppm",
            "total_abs_intensity",
            "peak_count",
            "peak_indices",
            "spacings_ppm",
            "estimated_j_hz",
        ],
    );
    for multiplet in multiplets {
        row(
            output,
            &[
                multiplet.id.clone(),
                format!("{:?}", multiplet.kind),
                finite("multiplet center_ppm", multiplet.center_ppm)?,
                finite("multiplet from_ppm", multiplet.from_ppm)?,
                finite("multiplet to_ppm", multiplet.to_ppm)?,
                finite(
                    "multiplet total_abs_intensity",
                    multiplet.total_abs_intensity,
                )?,
                multiplet.peaks.len().to_string(),
                peak_indices(&multiplet.peaks),
                finite_list("multiplet spacings_ppm", &multiplet.spacings_ppm)?,
                optional_finite("multiplet estimated_j_hz", multiplet.estimated_j_hz)?,
            ],
        );
    }
    Ok(())
}

fn write_signals_1d(output: &mut String, signals: &[SignalSummary1D]) -> Result<()> {
    section(output, "signals");
    row(
        output,
        [
            "id",
            "center_ppm",
            "from_ppm",
            "to_ppm",
            "peak_count",
            "area",
            "max_abs_intensity",
            "multiplet_kinds",
            "estimated_j_hz",
            "assignment_count",
            "atom_ids",
            "coupling_count",
        ],
    );
    for signal in signals {
        row(
            output,
            &[
                signal.id.clone(),
                finite("signal center_ppm", signal.center_ppm)?,
                finite("signal from_ppm", signal.from_ppm)?,
                finite("signal to_ppm", signal.to_ppm)?,
                signal.peak_count.to_string(),
                optional_finite("signal area", signal.area)?,
                finite("signal max_abs_intensity", signal.max_abs_intensity)?,
                signal
                    .multiplet_kinds
                    .iter()
                    .map(|kind| format!("{kind:?}"))
                    .collect::<Vec<_>>()
                    .join(";"),
                finite_list("signal estimated_j_hz", &signal.estimated_j_hz)?,
                signal.assignments.len().to_string(),
                atom_ids_1d(signal),
                signal.couplings.len().to_string(),
            ],
        );
    }
    Ok(())
}

fn write_zones(output: &mut String, zones: &[DetectedZone]) -> Result<()> {
    section(output, "zones");
    row(
        output,
        [
            "id",
            "x_start_index",
            "x_end_index",
            "y_start_index",
            "y_end_index",
            "x_from",
            "x_to",
            "y_from",
            "y_to",
            "centroid_x",
            "centroid_y",
            "active_points",
            "max_abs_intensity",
            "sum_intensity",
            "sum_abs_intensity",
        ],
    );
    for zone in zones {
        row(
            output,
            &[
                zone.id.clone(),
                zone.x_start_index.to_string(),
                zone.x_end_index.to_string(),
                zone.y_start_index.to_string(),
                zone.y_end_index.to_string(),
                finite("zone x_from", zone.x_from)?,
                finite("zone x_to", zone.x_to)?,
                finite("zone y_from", zone.y_from)?,
                finite("zone y_to", zone.y_to)?,
                finite("zone centroid_x", zone.centroid_x)?,
                finite("zone centroid_y", zone.centroid_y)?,
                zone.active_points.to_string(),
                finite("zone max_abs_intensity", zone.max_abs_intensity)?,
                finite("zone sum_intensity", zone.sum_intensity)?,
                finite("zone sum_abs_intensity", zone.sum_abs_intensity)?,
            ],
        );
    }
    Ok(())
}

fn write_signals_2d(output: &mut String, signals: &[SignalSummary2D]) -> Result<()> {
    section(output, "signals");
    row(
        output,
        [
            "id",
            "zone_id",
            "center_x",
            "center_y",
            "x_from",
            "x_to",
            "y_from",
            "y_to",
            "active_points",
            "max_abs_intensity",
            "sum_intensity",
            "sum_abs_intensity",
            "assignment_count",
            "atom_ids",
        ],
    );
    for signal in signals {
        row(
            output,
            &[
                signal.id.clone(),
                signal.zone.id.clone(),
                finite("signal center_x", signal.center_x)?,
                finite("signal center_y", signal.center_y)?,
                finite("signal x_from", signal.x_from)?,
                finite("signal x_to", signal.x_to)?,
                finite("signal y_from", signal.y_from)?,
                finite("signal y_to", signal.y_to)?,
                signal.active_points.to_string(),
                finite("signal max_abs_intensity", signal.max_abs_intensity)?,
                finite("signal sum_intensity", signal.sum_intensity)?,
                finite("signal sum_abs_intensity", signal.sum_abs_intensity)?,
                signal.assignments.len().to_string(),
                atom_ids_2d(signal),
            ],
        );
    }
    Ok(())
}

fn section(output: &mut String, name: &str) {
    output.push_str("# section=");
    output.push_str(name);
    output.push('\n');
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

fn finite(field: &'static str, value: f64) -> Result<String> {
    if !value.is_finite() {
        return Err(RSpinError::NonFinite { field });
    }
    Ok(format_float(value))
}

fn optional_finite(field: &'static str, value: Option<f64>) -> Result<String> {
    value.map_or_else(|| Ok(String::new()), |value| finite(field, value))
}

fn finite_list(field: &'static str, values: &[f64]) -> Result<String> {
    values
        .iter()
        .copied()
        .map(|value| finite(field, value))
        .collect::<Result<Vec<_>>>()
        .map(|values| values.join(";"))
}

fn peak_indices(peaks: &[Peak]) -> String {
    peaks
        .iter()
        .map(|peak| peak.index.to_string())
        .collect::<Vec<_>>()
        .join(";")
}

fn atom_ids_1d(signal: &SignalSummary1D) -> String {
    signal
        .atoms
        .iter()
        .map(|atom| atom.id.as_str())
        .collect::<Vec<_>>()
        .join(";")
}

fn atom_ids_2d(signal: &SignalSummary2D) -> String {
    signal
        .atoms
        .iter()
        .map(|atom| atom.id.as_str())
        .collect::<Vec<_>>()
        .join(";")
}

#[cfg(test)]
mod tests;
