//! Two-dimensional PNG plot helpers.

use std::{fs, path::Path};

use rspin_core::{RSpinError, Result, Spectrum1D, Spectrum2D, Unit};
use ruviz::prelude::{Image, LegendPosition, Plot};

use super::{axis_label, finite_range};

const DEFAULT_2D_WIDTH: u32 = 1_800;
const DEFAULT_2D_HEIGHT: u32 = 1_500;
const MARGINAL_PERCENT: u32 = 22;
const PANEL_GUTTER_PX: u32 = 16;
const MIN_MAIN_WIDTH: u32 = 600;
const MIN_MAIN_HEIGHT: u32 = 500;
const OVERLAY_POINTS_PER_PIXEL: usize = 4;
const WHITE_RGBA: [u8; 4] = [255, 255, 255, 255];

/// Caller-provided one-dimensional trace for a 2D marginal panel.
#[derive(Clone, Copy, Debug)]
pub struct PlotTrace1D<'a> {
    /// Legend label shown for the trace.
    pub label: &'a str,
    /// Spectrum to render in the marginal panel.
    pub spectrum: &'a Spectrum1D,
}

/// Rendering options for [`plot_spectrum_2d_with`].
#[derive(Clone, Debug)]
pub struct Spectrum2DPlotOptions<'a> {
    x_overlays: Vec<PlotTrace1D<'a>>,
    y_overlays: Vec<PlotTrace1D<'a>>,
    contour_levels: Option<Vec<f64>>,
    width: u32,
    height: u32,
}

impl<'a> Spectrum2DPlotOptions<'a> {
    /// Creates default 2D plot options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a one-dimensional overlay above the 2D plot.
    #[must_use]
    pub fn with_x_overlay(mut self, label: &'a str, spectrum: &'a Spectrum1D) -> Self {
        self.x_overlays.push(PlotTrace1D { label, spectrum });
        self
    }

    /// Adds a one-dimensional overlay to the left of the 2D plot.
    #[must_use]
    pub fn with_y_overlay(mut self, label: &'a str, spectrum: &'a Spectrum1D) -> Self {
        self.y_overlays.push(PlotTrace1D { label, spectrum });
        self
    }

    /// Uses explicit contour levels instead of the noise-aware default.
    #[must_use]
    pub fn with_contour_levels<I>(mut self, levels: I) -> Self
    where
        I: IntoIterator<Item = f64>,
    {
        self.contour_levels = Some(levels.into_iter().collect());
        self
    }

    /// Sets the final PNG canvas size in pixels.
    #[must_use]
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// X-axis overlays rendered above the 2D plot.
    #[must_use]
    pub fn x_overlays(&self) -> &[PlotTrace1D<'a>] {
        &self.x_overlays
    }

    /// Y-axis overlays rendered to the left of the 2D plot.
    #[must_use]
    pub fn y_overlays(&self) -> &[PlotTrace1D<'a>] {
        &self.y_overlays
    }
}

impl Default for Spectrum2DPlotOptions<'_> {
    fn default() -> Self {
        Self {
            x_overlays: Vec::new(),
            y_overlays: Vec::new(),
            contour_levels: None,
            width: DEFAULT_2D_WIDTH,
            height: DEFAULT_2D_HEIGHT,
        }
    }
}

/// Saves a two-dimensional contour PNG using NMR axis conventions.
///
/// # Errors
///
/// Returns an error when the spectrum cannot be rendered or the PNG
/// cannot be written.
pub fn plot_spectrum_2d(path: &Path, title: &str, spectrum: &Spectrum2D) -> Result<()> {
    let options = Spectrum2DPlotOptions::default();
    plot_spectrum_2d_with(path, title, spectrum, &options)
}

/// Saves a two-dimensional contour PNG with optional external 1D
/// spectra rendered as marginal overlays.
///
/// X overlays are drawn above the contour and must use the same unit
/// as `spectrum.x`. Y overlays are drawn to the left and must use the
/// same unit as `spectrum.y`. Overlay intensities are normalized per
/// marginal panel for display only.
///
/// # Errors
///
/// Returns an error when dimensions are too small, overlay units are
/// incompatible, contour levels are invalid, rendering fails, or the
/// PNG cannot be written.
pub fn plot_spectrum_2d_with(
    path: &Path,
    title: &str,
    spectrum: &Spectrum2D,
    options: &Spectrum2DPlotOptions<'_>,
) -> Result<()> {
    let image = render_spectrum_2d_image(title, spectrum, options)?;
    let bytes = image
        .encode_png()
        .map_err(|error| RSpinError::InvalidSpectrum {
            message: format!("failed to encode 2D plot PNG: {error}"),
        })?;
    fs::write(path, bytes).map_err(|error| RSpinError::InvalidSpectrum {
        message: format!("failed to write {}: {error}", path.display()),
    })?;
    Ok(())
}

#[derive(Clone, Copy, Debug)]
struct PixelRect {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

#[derive(Clone, Copy, Debug)]
struct Plot2DLayout {
    main: PixelRect,
    x_overlay: Option<PixelRect>,
    y_overlay: Option<PixelRect>,
}

#[derive(Clone, Debug)]
struct PreparedTrace<'a> {
    label: &'a str,
    coordinates: Vec<f64>,
    intensities: Vec<f64>,
}

fn render_spectrum_2d_image(
    title: &str,
    spectrum: &Spectrum2D,
    options: &Spectrum2DPlotOptions<'_>,
) -> Result<Image> {
    validate_spectrum_2d_for_plot(spectrum)?;
    let layout = plot_2d_layout(options)?;
    let levels = contour_levels(spectrum, options)?;
    let main_image = render_contour_panel(title, spectrum, &levels, layout.main)?;
    let mut image = blank_image(options.width, options.height)?;
    copy_image(&mut image, &main_image, layout.main.x, layout.main.y)?;

    if let Some(rect) = layout.x_overlay {
        let traces = prepare_overlay_traces(
            options.x_overlays(),
            spectrum.x.unit,
            &spectrum.x.values,
            rect.width,
            "x",
        )?;
        let overlay = render_x_overlay_panel(&traces, spectrum, rect)?;
        copy_image(&mut image, &overlay, rect.x, rect.y)?;
    }

    if let Some(rect) = layout.y_overlay {
        let traces = prepare_overlay_traces(
            options.y_overlays(),
            spectrum.y.unit,
            &spectrum.y.values,
            rect.height,
            "y",
        )?;
        let overlay = render_y_overlay_panel(&traces, spectrum, rect)?;
        copy_image(&mut image, &overlay, rect.x, rect.y)?;
    }

    Ok(image)
}

fn validate_spectrum_2d_for_plot(spectrum: &Spectrum2D) -> Result<()> {
    let (width, height) = spectrum.shape();
    if width < 2 || height < 2 {
        return Err(RSpinError::InvalidSpectrum {
            message: "2D plotting requires at least one cell".to_owned(),
        });
    }
    let expected = width
        .checked_mul(height)
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "2D plot dimensions overflow".to_owned(),
        })?;
    if spectrum.z.len() != expected {
        return Err(RSpinError::InvalidSpectrum {
            message: "2D plot matrix length does not match axis dimensions".to_owned(),
        });
    }
    for value in spectrum.z.iter().copied() {
        if !value.is_finite() {
            return Err(RSpinError::NonFinite {
                field: "2D plot intensity",
            });
        }
    }
    Ok(())
}

fn plot_2d_layout(options: &Spectrum2DPlotOptions<'_>) -> Result<Plot2DLayout> {
    let draw_top_panel = !options.x_overlays.is_empty();
    let draw_left_panel = !options.y_overlays.is_empty();
    let x_overlay_height = if draw_top_panel {
        options.height.saturating_mul(MARGINAL_PERCENT) / 100
    } else {
        0
    };
    let y_overlay_width = if draw_left_panel {
        options.width.saturating_mul(MARGINAL_PERCENT) / 100
    } else {
        0
    };
    let x_gutter = if draw_top_panel { PANEL_GUTTER_PX } else { 0 };
    let y_gutter = if draw_left_panel { PANEL_GUTTER_PX } else { 0 };

    let main_y = x_overlay_height
        .checked_add(x_gutter)
        .ok_or_else(|| invalid_plot_size("2D plot height overflow"))?;
    let main_x = y_overlay_width
        .checked_add(y_gutter)
        .ok_or_else(|| invalid_plot_size("2D plot width overflow"))?;
    let main_width = options
        .width
        .checked_sub(main_x)
        .ok_or_else(|| invalid_plot_size("2D plot width is too small"))?;
    let main_height = options
        .height
        .checked_sub(main_y)
        .ok_or_else(|| invalid_plot_size("2D plot height is too small"))?;

    if main_width < MIN_MAIN_WIDTH || main_height < MIN_MAIN_HEIGHT {
        return Err(invalid_plot_size(
            "2D plot canvas leaves too little room for the main contour panel",
        ));
    }

    let main = PixelRect {
        x: main_x,
        y: main_y,
        width: main_width,
        height: main_height,
    };
    let x_overlay = draw_top_panel.then_some(PixelRect {
        x: main_x,
        y: 0,
        width: main_width,
        height: x_overlay_height,
    });
    let y_overlay = draw_left_panel.then_some(PixelRect {
        x: 0,
        y: main_y,
        width: y_overlay_width,
        height: main_height,
    });

    Ok(Plot2DLayout {
        main,
        x_overlay,
        y_overlay,
    })
}

fn invalid_plot_size(message: &str) -> RSpinError {
    RSpinError::InvalidSpectrum {
        message: message.to_owned(),
    }
}

fn contour_levels(spectrum: &Spectrum2D, options: &Spectrum2DPlotOptions<'_>) -> Result<Vec<f64>> {
    if let Some(levels) = &options.contour_levels {
        validate_contour_levels(levels)?;
        return Ok(levels.clone());
    }
    Ok(autoscale_contour_levels(&spectrum.z))
}

fn validate_contour_levels(levels: &[f64]) -> Result<()> {
    if levels.is_empty() {
        return Err(RSpinError::InvalidSpectrum {
            message: "2D contour levels must not be empty".to_owned(),
        });
    }
    for level in levels.iter().copied() {
        if !level.is_finite() {
            return Err(RSpinError::NonFinite {
                field: "2D contour level",
            });
        }
    }
    Ok(())
}

fn autoscale_contour_levels(z: &[f64]) -> Vec<f64> {
    let max_abs = z.iter().copied().map(f64::abs).fold(0.0_f64, f64::max);
    if max_abs <= 0.0 {
        return vec![0.0];
    }

    let median = median_abs(z);
    let base = (median * 8.0).max(max_abs * 0.03);
    let ratio = 1.3_f64;
    let mut levels = Vec::with_capacity(20);
    let mut level = base;
    for _ in 0..20 {
        if level <= max_abs {
            levels.push(level);
        }
        level *= ratio;
    }
    if levels.is_empty() {
        levels.push(max_abs);
    }
    levels
}

fn median_abs(z: &[f64]) -> f64 {
    if z.is_empty() {
        return 0.0;
    }
    let mut values: Vec<f64> = z.iter().copied().map(f64::abs).collect();
    let index = values.len() / 2;
    let (_, median, _) = values.select_nth_unstable_by(index, f64::total_cmp);
    *median
}

fn render_contour_panel(
    title: &str,
    spectrum: &Spectrum2D,
    levels: &[f64],
    rect: PixelRect,
) -> Result<Image> {
    let mut plot = Plot::new()
        .title(title)
        .xlabel(axis_label(spectrum.x.unit))
        .ylabel(axis_label(spectrum.y.unit))
        .size_px(rect.width, rect.height);
    if let Some((x_min, x_max)) = plot_axis_limits(&spectrum.x.values, spectrum.x.unit) {
        plot = plot.xlim(x_min, x_max);
    }
    if let Some((y_min, y_max)) = plot_axis_limits(&spectrum.y.values, spectrum.y.unit) {
        plot = plot.ylim(y_min, y_max);
    }
    plot.contour(&spectrum.x.values, &spectrum.y.values, &spectrum.z)
        .level_values(levels.to_vec())
        .filled(false)
        .render()
        .map_err(|error| RSpinError::InvalidSpectrum {
            message: format!("failed to render 2D contour panel: {error}"),
        })
}

fn prepare_overlay_traces<'a>(
    traces: &[PlotTrace1D<'a>],
    expected_unit: Unit,
    reference_axis: &[f64],
    panel_extent_px: u32,
    axis_name: &'static str,
) -> Result<Vec<PreparedTrace<'a>>> {
    let (axis_lo, axis_hi) =
        finite_range(reference_axis.iter().copied()).ok_or_else(|| RSpinError::InvalidAxis {
            message: format!("{axis_name} axis has no finite plotting range"),
        })?;
    let mut clipped = Vec::with_capacity(traces.len());
    let mut max_abs = 0.0_f64;

    for trace in traces {
        if trace.spectrum.x.unit != expected_unit {
            return Err(RSpinError::InvalidSpectrum {
                message: format!(
                    "{axis_name} overlay '{}' uses {:?}, expected {:?}",
                    trace.label, trace.spectrum.x.unit, expected_unit
                ),
            });
        }

        let mut coordinates = Vec::new();
        let mut intensities = Vec::new();
        for (coordinate, intensity) in trace
            .spectrum
            .x
            .values
            .iter()
            .copied()
            .zip(trace.spectrum.intensities.iter().copied())
        {
            if coordinate.is_finite()
                && intensity.is_finite()
                && coordinate >= axis_lo
                && coordinate <= axis_hi
            {
                max_abs = max_abs.max(intensity.abs());
                coordinates.push(coordinate);
                intensities.push(intensity);
            }
        }

        if coordinates.is_empty() {
            return Err(RSpinError::InvalidSpectrum {
                message: format!(
                    "{axis_name} overlay '{}' has no points inside the 2D axis range",
                    trace.label
                ),
            });
        }
        clipped.push(PreparedTrace {
            label: trace.label,
            coordinates,
            intensities,
        });
    }

    let scale = if max_abs > 0.0 { max_abs } else { 1.0 };
    let max_points = overlay_max_points(panel_extent_px)?;
    for trace in &mut clipped {
        for intensity in &mut trace.intensities {
            *intensity /= scale;
        }
        let (coordinates, intensities) =
            decimate_trace(&trace.coordinates, &trace.intensities, max_points);
        trace.coordinates = coordinates;
        trace.intensities = intensities;
    }

    Ok(clipped)
}

fn overlay_max_points(panel_extent_px: u32) -> Result<usize> {
    let extent = usize::try_from(panel_extent_px).map_err(|_| RSpinError::InvalidSpectrum {
        message: "overlay panel is too large to decimate".to_owned(),
    })?;
    Ok(extent.saturating_mul(OVERLAY_POINTS_PER_PIXEL).max(4))
}

fn decimate_trace(
    coordinates: &[f64],
    intensities: &[f64],
    max_points: usize,
) -> (Vec<f64>, Vec<f64>) {
    let len = coordinates.len().min(intensities.len());
    if len <= max_points || max_points < 4 {
        return (
            coordinates.iter().copied().take(len).collect(),
            intensities.iter().copied().take(len).collect(),
        );
    }

    let bucket_count = ((max_points - 2) / 2).max(1);
    let interior = len - 2;
    let mut selected = Vec::with_capacity(max_points);
    selected.push(0);

    for bucket in 0..bucket_count {
        let start = 1 + bucket * interior / bucket_count;
        let end = 1 + (bucket + 1) * interior / bucket_count;
        if start >= end {
            continue;
        }
        let mut min_index = start;
        let mut max_index = start;
        for index in (start + 1)..end {
            if intensities[index] < intensities[min_index] {
                min_index = index;
            }
            if intensities[index] > intensities[max_index] {
                max_index = index;
            }
        }
        if min_index <= max_index {
            push_unique_index(&mut selected, min_index);
            push_unique_index(&mut selected, max_index);
        } else {
            push_unique_index(&mut selected, max_index);
            push_unique_index(&mut selected, min_index);
        }
    }
    push_unique_index(&mut selected, len - 1);

    let mut out_coordinates = Vec::with_capacity(selected.len());
    let mut out_intensities = Vec::with_capacity(selected.len());
    for index in selected {
        out_coordinates.push(coordinates[index]);
        out_intensities.push(intensities[index]);
    }
    (out_coordinates, out_intensities)
}

fn push_unique_index(indices: &mut Vec<usize>, index: usize) {
    if indices.last().copied() != Some(index) {
        indices.push(index);
    }
}

fn render_x_overlay_panel(
    traces: &[PreparedTrace<'_>],
    spectrum: &Spectrum2D,
    rect: PixelRect,
) -> Result<Image> {
    let mut plot = Plot::new()
        .xlabel("")
        .ylabel("normalized intensity")
        .size_px(rect.width, rect.height)
        .legend_position(LegendPosition::Best);
    if let Some((x_min, x_max)) = plot_axis_limits(&spectrum.x.values, spectrum.x.unit) {
        plot = plot.xlim(x_min, x_max);
    }
    if let Some((y_min, y_max)) = overlay_intensity_limits(traces) {
        plot = plot.ylim(y_min, y_max);
    }

    let mut builder = plot;
    for trace in traces {
        builder = builder
            .line(&trace.coordinates, &trace.intensities)
            .label(trace.label)
            .into();
    }
    builder
        .render()
        .map_err(|error| RSpinError::InvalidSpectrum {
            message: format!("failed to render X overlay panel: {error}"),
        })
}

fn render_y_overlay_panel(
    traces: &[PreparedTrace<'_>],
    spectrum: &Spectrum2D,
    rect: PixelRect,
) -> Result<Image> {
    let mut plot = Plot::new()
        .xlabel("normalized intensity")
        .ylabel("")
        .size_px(rect.width, rect.height)
        .legend_position(LegendPosition::Best);
    if let Some((x_min, x_max)) = overlay_intensity_limits(traces) {
        plot = plot.xlim(x_min, x_max);
    }
    if let Some((y_min, y_max)) = plot_axis_limits(&spectrum.y.values, spectrum.y.unit) {
        plot = plot.ylim(y_min, y_max);
    }

    let mut builder = plot;
    for trace in traces {
        builder = builder
            .line(&trace.intensities, &trace.coordinates)
            .label(trace.label)
            .into();
    }
    builder
        .render()
        .map_err(|error| RSpinError::InvalidSpectrum {
            message: format!("failed to render Y overlay panel: {error}"),
        })
}

fn overlay_intensity_limits(traces: &[PreparedTrace<'_>]) -> Option<(f64, f64)> {
    let (lo, hi) = finite_range(
        traces
            .iter()
            .flat_map(|trace| trace.intensities.iter().copied()),
    )?;
    if (hi - lo).abs() <= f64::EPSILON {
        return Some((lo - 0.05, hi + 1.0));
    }
    let span = hi - lo;
    Some((lo - 0.05 * span, hi + 0.10 * span))
}

fn plot_axis_limits(values: &[f64], unit: Unit) -> Option<(f64, f64)> {
    let (lo, hi) = finite_range(values.iter().copied())?;
    if matches!(unit, Unit::Ppm | Unit::Hertz) {
        Some((hi, lo))
    } else {
        Some((lo, hi))
    }
}

fn blank_image(width: u32, height: u32) -> Result<Image> {
    let len = rgba_len(width, height)?;
    let mut pixels = vec![0_u8; len];
    for pixel in pixels.chunks_exact_mut(4) {
        pixel.copy_from_slice(&WHITE_RGBA);
    }
    Ok(Image::new(width, height, pixels))
}

fn copy_image(target: &mut Image, source: &Image, dst_x: u32, dst_y: u32) -> Result<()> {
    let end_x = dst_x
        .checked_add(source.width)
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "composed plot x position overflow".to_owned(),
        })?;
    let end_y = dst_y
        .checked_add(source.height)
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "composed plot y position overflow".to_owned(),
        })?;
    if end_x > target.width || end_y > target.height {
        return Err(RSpinError::InvalidSpectrum {
            message: "composed plot panel exceeds canvas".to_owned(),
        });
    }
    for y in 0..source.height {
        for x in 0..source.width {
            let source_index = rgba_index(source.width, x, y)?;
            let target_index = rgba_index(target.width, dst_x + x, dst_y + y)?;
            target.pixels[target_index..target_index + 4]
                .copy_from_slice(&source.pixels[source_index..source_index + 4]);
        }
    }
    Ok(())
}

fn rgba_len(width: u32, height: u32) -> Result<usize> {
    let pixels = u64::from(width)
        .checked_mul(u64::from(height))
        .and_then(|value| value.checked_mul(4))
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "plot image dimensions overflow".to_owned(),
        })?;
    usize::try_from(pixels).map_err(|_| RSpinError::InvalidSpectrum {
        message: "plot image dimensions exceed platform limits".to_owned(),
    })
}

fn rgba_index(width: u32, x: u32, y: u32) -> Result<usize> {
    let index = u64::from(y)
        .checked_mul(u64::from(width))
        .and_then(|value| value.checked_add(u64::from(x)))
        .and_then(|value| value.checked_mul(4))
        .ok_or_else(|| RSpinError::InvalidSpectrum {
            message: "plot pixel index overflow".to_owned(),
        })?;
    usize::try_from(index).map_err(|_| RSpinError::InvalidSpectrum {
        message: "plot pixel index exceeds platform limits".to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use rspin_core::{Axis, Metadata};

    use super::*;

    #[test]
    fn renders_plain_2d_contour_image() -> Result<()> {
        let spectrum = demo_2d()?;
        let options = Spectrum2DPlotOptions::new().with_size(900, 700);
        let image = render_spectrum_2d_image("demo", &spectrum, &options)?;
        assert_eq!(image.width, 900);
        assert_eq!(image.height, 700);
        assert!(has_nonwhite_pixel(&image));
        Ok(())
    }

    #[test]
    fn renders_external_x_and_y_overlays() -> Result<()> {
        let spectrum = demo_2d()?;
        let x_overlay = demo_1d(Unit::Ppm, 0.0, 2.0)?;
        let y_overlay = demo_1d(Unit::Ppm, 10.0, 12.0)?;
        let options = Spectrum2DPlotOptions::new()
            .with_size(1_000, 800)
            .with_x_overlay("1H", &x_overlay)
            .with_y_overlay("13C", &y_overlay);
        let image = render_spectrum_2d_image("overlay", &spectrum, &options)?;
        assert_eq!(image.width, 1_000);
        assert_eq!(image.height, 800);
        assert!(has_nonwhite_pixel(&image));
        Ok(())
    }

    #[test]
    fn rejects_overlay_unit_mismatch() -> Result<()> {
        let spectrum = demo_2d()?;
        let x_overlay = demo_1d(Unit::Hertz, 0.0, 2.0)?;
        let options = Spectrum2DPlotOptions::new()
            .with_size(900, 700)
            .with_x_overlay("wrong unit", &x_overlay);
        let error = render_spectrum_2d_image("overlay", &spectrum, &options)
            .expect_err("unit mismatch should fail");
        assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));
        Ok(())
    }

    #[test]
    fn overlay_normalization_preserves_relative_panel_intensity() -> Result<()> {
        let spectrum = demo_2d()?;
        let weak = Spectrum1D::new(
            Axis::linear_ppm(0.0, 2.0, 3)?,
            vec![0.0, 2.0, 0.0],
            Metadata::named("weak"),
        )?;
        let strong = Spectrum1D::new(
            Axis::linear_ppm(0.0, 2.0, 3)?,
            vec![0.0, 4.0, 0.0],
            Metadata::named("strong"),
        )?;
        let traces = [
            PlotTrace1D {
                label: "weak",
                spectrum: &weak,
            },
            PlotTrace1D {
                label: "strong",
                spectrum: &strong,
            },
        ];
        let prepared = prepare_overlay_traces(&traces, Unit::Ppm, &spectrum.x.values, 400, "x")?;
        assert_eq!(prepared.len(), 2);
        assert!((prepared[0].intensities[1] - 0.5).abs() < 1.0e-12);
        assert!((prepared[1].intensities[1] - 1.0).abs() < 1.0e-12);
        Ok(())
    }

    #[test]
    fn decimation_preserves_endpoints_and_extrema() {
        let coordinates: Vec<f64> = (0_u32..20).map(f64::from).collect();
        let mut intensities = vec![0.0; 20];
        intensities[4] = 8.0;
        intensities[15] = -6.0;

        let (x, y) = decimate_trace(&coordinates, &intensities, 8);
        assert_eq!(x.first().copied(), Some(0.0));
        assert_eq!(x.last().copied(), Some(19.0));
        assert!(y.contains(&8.0));
        assert!(y.contains(&-6.0));
    }

    #[test]
    fn layout_supports_all_overlay_combinations() -> Result<()> {
        let x_overlay = demo_1d(Unit::Ppm, 0.0, 2.0)?;
        let y_overlay = demo_1d(Unit::Ppm, 10.0, 12.0)?;
        for options in [
            Spectrum2DPlotOptions::new().with_size(900, 700),
            Spectrum2DPlotOptions::new()
                .with_size(900, 700)
                .with_x_overlay("x", &x_overlay),
            Spectrum2DPlotOptions::new()
                .with_size(900, 700)
                .with_y_overlay("y", &y_overlay),
            Spectrum2DPlotOptions::new()
                .with_size(900, 700)
                .with_x_overlay("x", &x_overlay)
                .with_y_overlay("y", &y_overlay),
        ] {
            let layout = plot_2d_layout(&options)?;
            assert!(layout.main.width >= MIN_MAIN_WIDTH);
            assert!(layout.main.height >= MIN_MAIN_HEIGHT);
            if let Some(rect) = layout.x_overlay {
                assert_eq!(rect.x, layout.main.x);
                assert_eq!(rect.width, layout.main.width);
                assert!(rect.y + rect.height <= layout.main.y);
            }
            if let Some(rect) = layout.y_overlay {
                assert!(rect.x + rect.width <= layout.main.x);
                assert_eq!(rect.height, layout.main.height);
            }
        }
        Ok(())
    }

    fn demo_2d() -> Result<Spectrum2D> {
        Spectrum2D::new(
            Axis::linear_ppm(0.0, 2.0, 3)?,
            Axis::linear_ppm(10.0, 12.0, 3)?,
            vec![0.0, 0.2, 0.0, 0.3, 1.0, 0.3, 0.0, 0.2, 0.0],
            Metadata::named("2d"),
        )
    }

    fn demo_1d(unit: Unit, start: f64, end: f64) -> Result<Spectrum1D> {
        Spectrum1D::new(
            Axis::linear("x", unit, start, end, 3)?,
            vec![0.0, 1.0, 0.0],
            Metadata::named("1d"),
        )
    }

    fn has_nonwhite_pixel(image: &Image) -> bool {
        image
            .pixels
            .chunks_exact(4)
            .any(|pixel| pixel[0] < 250 || pixel[1] < 250 || pixel[2] < 250)
    }
}
