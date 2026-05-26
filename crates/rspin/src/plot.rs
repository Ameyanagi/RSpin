//! NMR-aware PNG plot helpers built on `ruviz`.
//!
//! Available only with the `visualization` feature. All helpers apply
//! the conventions NMR readers expect:
//!
//! - X-axis runs **high → low** for `Ppm` and `Hz` units (Bruker/NMR
//!   convention) so increasing chemical shift goes leftward.
//! - Y-axis is padded 5 % below the minimum and 10 % above the maximum
//!   so small negative baseline excursions stay visible.
//!
//! Errors from ruviz are surfaced as
//! [`rspin_core::RSpinError::InvalidSpectrum`] so callers can stay on
//! the unified [`Result`](rspin_core::Result) type.

use std::path::Path;

use rspin_core::{RSpinError, Result, Spectrum1D, Unit};
use ruviz::prelude::{LegendPosition, Plot};

/// Saves a single-trace PNG.
///
/// `title` and `series_label` annotate the plot; `path` must end in
/// `.png`.
///
/// # Errors
///
/// Returns an error when ruviz cannot render or write the PNG, or when
/// `path` is not valid UTF-8.
pub fn plot_spectrum(path: &Path, title: &str, spectrum: &Spectrum1D) -> Result<()> {
    plot_overlay(
        path,
        title,
        &[(
            spectrum.metadata.name.as_deref().unwrap_or("spectrum"),
            spectrum,
        )],
    )
}

/// Saves an N-trace overlay PNG. The first trace drives the x-axis
/// label.
///
/// # Errors
///
/// Returns an error when ruviz cannot render or write the PNG, when
/// `path` is not valid UTF-8, or when `traces` is empty.
pub fn plot_overlay(path: &Path, title: &str, traces: &[(&str, &Spectrum1D)]) -> Result<()> {
    let first = traces
        .first()
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "plot_overlay requires at least one trace".to_owned(),
        })?
        .1;
    let path_str = path.to_str().ok_or_else(|| RSpinError::InvalidSpectrum {
        message: format!("plot path {} is not valid UTF-8", path.display()),
    })?;

    let y_traces: Vec<&[f64]> = traces
        .iter()
        .map(|(_, s)| s.intensities.as_slice())
        .collect();

    let mut plot = Plot::new()
        .title(title)
        .xlabel(axis_label(first.x.unit))
        .ylabel("intensity")
        .max_resolution(1600, 1000)
        .legend_position(LegendPosition::Best);
    if let Some((x_max, x_min)) = nmr_x_limits(&first.x.values, first.x.unit) {
        plot = plot.xlim(x_max, x_min);
    }
    if let Some((y_min, y_max)) = padded_y_limits(&y_traces) {
        plot = plot.ylim(y_min, y_max);
    }

    let mut builder = plot;
    for (label, spectrum) in traces {
        builder = builder
            .line(&spectrum.x.values, &spectrum.intensities)
            .label(*label)
            .into();
    }
    builder
        .save(path_str)
        .map_err(|error| RSpinError::InvalidSpectrum {
            message: format!("failed to write {path_str}: {error}"),
        })?;
    Ok(())
}

/// X-axis limits in NMR convention (high → low) for `Ppm`/`Hz` units,
/// returned as `(max, min)` so callers can pass them straight to
/// [`Plot::xlim`]. Returns `None` for time-domain or point-domain
/// axes where the convention does not apply.
#[must_use]
pub fn nmr_x_limits(x: &[f64], unit: Unit) -> Option<(f64, f64)> {
    if !matches!(unit, Unit::Ppm | Unit::Hertz) {
        return None;
    }
    let (lo, hi) = finite_range(x.iter().copied())?;
    Some((hi, lo))
}

/// Y-axis limits padded 5 % below the minimum and 10 % above the
/// maximum across all supplied traces.
#[must_use]
pub fn padded_y_limits(series: &[&[f64]]) -> Option<(f64, f64)> {
    let (lo, hi) = finite_range(series.iter().flat_map(|s| s.iter().copied()))?;
    let span = hi - lo;
    Some((lo - 0.05 * span, hi + 0.10 * span))
}

fn finite_range<I: Iterator<Item = f64>>(values: I) -> Option<(f64, f64)> {
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    for value in values.filter(|value| value.is_finite()) {
        if value < lo {
            lo = value;
        }
        if value > hi {
            hi = value;
        }
    }
    if !lo.is_finite() || !hi.is_finite() || lo >= hi {
        return None;
    }
    Some((lo, hi))
}

fn axis_label(unit: Unit) -> &'static str {
    match unit {
        Unit::Ppm => "chemical shift / ppm",
        Unit::Hertz => "frequency / Hz",
        Unit::Seconds => "time / s",
        Unit::Points => "point",
        _ => "x",
    }
}
