use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow};
use plotters::prelude::*;
use rspin::{
    PeakPickOptions, PeakPolarity, RangeDetectionOptions, Spectrum1D, Spectrum2D,
    SpectrumAnalysis1D, SpectrumAnalysis1DOptions, SpectrumAnalysis2D, SpectrumAnalysis2DOptions,
    ZoneConnectivity, ZoneDetectionOptions, analyze_spectrum_1d, analyze_spectrum_2d,
    read_agilent_fid_1d_dir, read_agilent_fid_2d_dir, read_bruker_fid_1d_dir,
    read_bruker_ser_2d_dir, read_jcamp_dx_1d, read_jcamp_dx_2d, read_jeol_jdf_1d_file,
    read_jeol_jdf_2d_file, read_nmrml_1d_file, read_nmrml_2d_file,
};

const OUT_WIDTH: u32 = 1600;
const OUT_HEIGHT: u32 = 1100;
const HEADER_HEIGHT: u32 = 170;
const MAX_LINE_POINTS: usize = 9000;
const MAX_HEATMAP_BINS: usize = 360;
const MAX_PEAK_MARKERS: usize = 200;
const MAX_RANGE_MARKERS: usize = 120;
const MAX_ZONE_MARKERS: usize = 120;

fn main() -> Result<()> {
    let repo_root = repo_root()?;
    let data_root = external_root()?;
    let output_root = repo_root.join("target").join("analysis-flow-png");
    fs::create_dir_all(&output_root).with_context(|| {
        format!(
            "failed to create output directory {}",
            output_root.display()
        )
    })?;
    clear_previous_outputs(&output_root)?;

    let cases = collect_cases(&data_root)?;
    let summary_path = output_root.join("summary.tsv");
    let mut summary = fs::File::create(&summary_path)
        .with_context(|| format!("failed to create {}", summary_path.display()))?;
    writeln!(
        summary,
        "index\tkind\treader\tlabel\tstatus\tpoints\tfeatures\tpng"
    )?;

    for (index, case) in cases.iter().enumerate() {
        let file_name = format!("{:03}_{}.png", index + 1, sanitize(&case.label));
        let output_path = output_root.join(file_name);
        let result = render_case(case, &output_path);
        match result {
            Ok(row) => {
                writeln!(
                    summary,
                    "{}\t{}\t{}\t{}\tok\t{}\t{}\t{}",
                    index + 1,
                    row.kind,
                    case.reader.label(),
                    case.label,
                    row.points,
                    row.features,
                    output_path.display()
                )?;
            }
            Err(error) => {
                render_error_png(case, &output_path, &error)?;
                writeln!(
                    summary,
                    "{}\tunknown\t{}\t{}\terror\t\t{}\t{}",
                    index + 1,
                    case.reader.label(),
                    case.label,
                    sanitize_summary(&error_chain(&error)),
                    output_path.display()
                )?;
            }
        }
    }

    println!("Rendered {} cases", cases.len());
    println!("PNG output: {}", output_root.display());
    println!("Summary: {}", summary_path.display());
    Ok(())
}

fn repo_root() -> Result<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let examples_dir = manifest_dir
        .parent()
        .context("example manifest directory has no parent")?;
    let root = examples_dir
        .parent()
        .context("examples directory has no parent")?;
    Ok(root.to_path_buf())
}

fn clear_previous_outputs(output_root: &Path) -> Result<()> {
    for entry in fs::read_dir(output_root)
        .with_context(|| format!("failed to read output directory {}", output_root.display()))?
    {
        let entry =
            entry.with_context(|| format!("failed to read entry in {}", output_root.display()))?;
        let path = entry.path();
        let remove = path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("png"))
            || path.file_name().and_then(|name| name.to_str()) == Some("summary.tsv");
        if remove {
            fs::remove_file(&path)
                .with_context(|| format!("failed to remove old output {}", path.display()))?;
        }
    }
    Ok(())
}

#[derive(Clone, Debug)]
struct Case {
    label: String,
    path: PathBuf,
    reader: Reader,
}

#[derive(Clone, Copy, Debug)]
enum Reader {
    JcampAuto,
    JeolAuto,
    NmrMlAuto,
    Agilent1D,
    Agilent2D,
    Bruker1D,
    Bruker2D,
}

impl Reader {
    fn label(self) -> &'static str {
        match self {
            Self::JcampAuto => "jcamp-auto",
            Self::JeolAuto => "jeol-auto",
            Self::NmrMlAuto => "nmrml-auto",
            Self::Agilent1D => "agilent-fid-1d",
            Self::Agilent2D => "agilent-fid-2d",
            Self::Bruker1D => "bruker-fid-1d",
            Self::Bruker2D => "bruker-ser-2d",
        }
    }
}

#[derive(Debug)]
enum Parsed {
    OneD(Spectrum1D),
    TwoD(Spectrum2D),
}

#[derive(Debug)]
struct RenderRow {
    kind: &'static str,
    points: String,
    features: String,
}

fn external_root() -> Result<PathBuf> {
    match env::var("RSPIN_EXTERNAL_TESTDATA") {
        Ok(value) if !value.trim().is_empty() => Ok(PathBuf::from(value)),
        _ => {
            let root = repo_root()?;
            let parent = root.parent().context("repo root has no parent")?;
            Ok(parent.join("rspin-external-testdata"))
        }
    }
}

fn collect_cases(data_root: &Path) -> Result<Vec<Case>> {
    if !data_root.exists() {
        return Err(anyhow!(
            "external fixture root does not exist: {}",
            data_root.display()
        ));
    }

    let mut cases = Vec::new();
    collect_jcamp_cases(data_root, &mut cases)?;
    collect_jeol_cases(data_root, &mut cases)?;
    collect_nmrml_cases(data_root, &mut cases)?;
    collect_nmrglue_vendor_cases(data_root, &mut cases)?;
    cases.sort_by(|a, b| a.label.cmp(&b.label));
    Ok(cases)
}

fn collect_jcamp_cases(data_root: &Path, cases: &mut Vec<Case>) -> Result<()> {
    let root = data_root
        .join("unpacked")
        .join("jcamp-data-test-2.5.0")
        .join("data")
        .join("nmr");
    if !root.exists() {
        return Ok(());
    }

    let mut paths = Vec::new();
    collect_matching_files(&root, &mut paths, |path| {
        path.extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| {
                extension.eq_ignore_ascii_case("jdx") || extension.eq_ignore_ascii_case("dx")
            })
    })?;
    paths.sort();

    for path in paths {
        let label = format!("jcamp/{}", relative_label(&root, &path));
        cases.push(Case {
            label,
            path,
            reader: Reader::JcampAuto,
        });
    }
    Ok(())
}

fn collect_jeol_cases(data_root: &Path, cases: &mut Vec<Case>) -> Result<()> {
    let root = data_root
        .join("unpacked")
        .join("jeol-data-test-1.0.0")
        .join("data");
    if !root.exists() {
        return Ok(());
    }

    let mut paths = Vec::new();
    collect_matching_files(&root, &mut paths, |path| {
        path.extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("jdf"))
    })?;
    paths.sort();

    for path in paths {
        let label = format!("jeol/{}", relative_label(&root, &path));
        cases.push(Case {
            label,
            path,
            reader: Reader::JeolAuto,
        });
    }
    Ok(())
}

fn collect_nmrml_cases(data_root: &Path, cases: &mut Vec<Case>) -> Result<()> {
    let root = data_root.join("nmrml").join("examples");
    if !root.exists() {
        return Ok(());
    }

    let mut paths = Vec::new();
    collect_matching_files(&root, &mut paths, |path| {
        path.extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("nmrML"))
    })?;
    paths.sort();

    for path in paths {
        let label = format!("nmrml/{}", relative_label(&root, &path));
        cases.push(Case {
            label,
            path,
            reader: Reader::NmrMlAuto,
        });
    }
    Ok(())
}

fn collect_nmrglue_vendor_cases(data_root: &Path, cases: &mut Vec<Case>) -> Result<()> {
    let root = data_root
        .join("unpacked")
        .join("nmrglue-test-data-v0.4-dev");
    let vendor_cases = [
        ("nmrglue/agilent_1d", "agilent_1d", Reader::Agilent1D),
        ("nmrglue/agilent_2d", "agilent_2d", Reader::Agilent2D),
        ("nmrglue/bruker_1d", "bruker_1d", Reader::Bruker1D),
        ("nmrglue/bruker_2d", "bruker_2d", Reader::Bruker2D),
    ];

    for (label, directory, reader) in vendor_cases {
        let path = root.join(directory);
        if path.exists() {
            cases.push(Case {
                label: label.to_owned(),
                path,
                reader,
            });
        }
    }
    Ok(())
}

fn collect_matching_files(
    directory: &Path,
    paths: &mut Vec<PathBuf>,
    accepts: impl Fn(&Path) -> bool + Copy,
) -> Result<()> {
    for entry in fs::read_dir(directory)
        .with_context(|| format!("failed to read directory {}", directory.display()))?
    {
        let entry =
            entry.with_context(|| format!("failed to read entry in {}", directory.display()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_matching_files(&path, paths, accepts)?;
        } else if accepts(&path) {
            paths.push(path);
        }
    }
    Ok(())
}

fn relative_label(root: &Path, path: &Path) -> String {
    match path.strip_prefix(root) {
        Ok(relative) => relative,
        Err(_) => path,
    }
    .to_string_lossy()
    .replace('\\', "/")
}

fn render_case(case: &Case, output_path: &Path) -> Result<RenderRow> {
    match read_case(case)? {
        Parsed::OneD(spectrum) => {
            let analysis = analyze_1d(&spectrum)?;
            render_1d(case, &spectrum, &analysis, output_path)?;
            Ok(RenderRow {
                kind: "1d",
                points: spectrum.len().to_string(),
                features: format!(
                    "peaks={},ranges={},integrals={},multiplets={},signals={}",
                    analysis.peaks.len(),
                    analysis.ranges.len(),
                    analysis.integrals.len(),
                    analysis.multiplets.len(),
                    analysis.signals.len()
                ),
            })
        }
        Parsed::TwoD(spectrum) => {
            let analysis = analyze_2d(&spectrum)?;
            render_2d(case, &spectrum, &analysis, output_path)?;
            Ok(RenderRow {
                kind: "2d",
                points: format!("{}x{}", spectrum.x.len(), spectrum.y.len()),
                features: format!(
                    "zones={},integrals={},signals={}",
                    analysis.zones.len(),
                    analysis.integrals.len(),
                    analysis.signals.len()
                ),
            })
        }
    }
}

fn read_case(case: &Case) -> Result<Parsed> {
    match case.reader {
        Reader::JcampAuto => {
            let text = fs::read_to_string(&case.path)
                .with_context(|| format!("failed to read {}", case.path.display()))?;
            if looks_like_jcamp_2d(&text) {
                match read_jcamp_dx_2d(&text) {
                    Ok(spectrum) => Ok(Parsed::TwoD(spectrum)),
                    Err(two_d_error) => match read_jcamp_dx_1d(&text) {
                        Ok(spectrum) => Ok(Parsed::OneD(spectrum)),
                        Err(one_d_error) => Err(anyhow!(
                            "JCAMP parse failed as 2D ({two_d_error}) and 1D ({one_d_error})"
                        )),
                    },
                }
            } else {
                match read_jcamp_dx_1d(&text) {
                    Ok(spectrum) => Ok(Parsed::OneD(spectrum)),
                    Err(one_d_error) => match read_jcamp_dx_2d(&text) {
                        Ok(spectrum) => Ok(Parsed::TwoD(spectrum)),
                        Err(two_d_error) => Err(anyhow!(
                            "JCAMP parse failed as 1D ({one_d_error}) and 2D ({two_d_error})"
                        )),
                    },
                }
            }
        }
        Reader::JeolAuto => match read_jeol_jdf_2d_file(&case.path) {
            Ok(spectrum) => Ok(Parsed::TwoD(spectrum)),
            Err(two_d_error) => match read_jeol_jdf_1d_file(&case.path) {
                Ok(spectrum) => Ok(Parsed::OneD(spectrum)),
                Err(one_d_error) => Err(anyhow!(
                    "JEOL parse failed as 2D ({two_d_error}) and 1D ({one_d_error})"
                )),
            },
        },
        Reader::NmrMlAuto => match read_nmrml_2d_file(&case.path) {
            Ok(spectrum) => Ok(Parsed::TwoD(spectrum)),
            Err(two_d_error) => match read_nmrml_1d_file(&case.path) {
                Ok(spectrum) => Ok(Parsed::OneD(spectrum)),
                Err(one_d_error) => Err(anyhow!(
                    "nmrML parse failed as 2D ({two_d_error}) and 1D ({one_d_error})"
                )),
            },
        },
        Reader::Agilent1D => read_agilent_fid_1d_dir(&case.path)
            .map(Parsed::OneD)
            .map_err(|error| anyhow!(error)),
        Reader::Agilent2D => read_agilent_fid_2d_dir(&case.path)
            .map(Parsed::TwoD)
            .map_err(|error| anyhow!(error)),
        Reader::Bruker1D => read_bruker_fid_1d_dir(&case.path)
            .map(Parsed::OneD)
            .map_err(|error| anyhow!(error)),
        Reader::Bruker2D => read_bruker_ser_2d_dir(&case.path)
            .map(Parsed::TwoD)
            .map_err(|error| anyhow!(error)),
    }
    .with_context(|| format!("while reading {}", case.path.display()))
}

fn looks_like_jcamp_2d(text: &str) -> bool {
    text.lines()
        .map(str::trim)
        .filter(|line| line.starts_with("##"))
        .any(|line| {
            let compact = line.to_ascii_uppercase().replace([' ', '\t'], "");
            compact.starts_with("##PAGE=")
                || compact.starts_with("##NUMDIM=2")
                || compact.starts_with("##NUMDIMENSIONS=2")
                || compact.starts_with("##VARDIM=") && compact.matches(',').count() >= 2
        })
}

fn analyze_1d(spectrum: &Spectrum1D) -> Result<SpectrumAnalysis1D> {
    let max_abs = max_abs(&spectrum.intensities);
    let p99 = percentile_abs(&spectrum.intensities, 0.995);
    let range_threshold = nonzero_threshold(max_abs, p99, 0.03, 0.20);
    let peak_threshold = nonzero_threshold(max_abs, p99, 0.06, 0.30);
    let prominence = nonzero_threshold(max_abs, p99, 0.01, 0.05);
    let options = SpectrumAnalysis1DOptions::new()
        .with_peak_options(
            PeakPickOptions::new()
                .with_min_abs_intensity(peak_threshold)
                .with_min_prominence(prominence)
                .with_polarity(PeakPolarity::Both),
        )
        .with_range_options(
            RangeDetectionOptions::new()
                .with_threshold_abs(range_threshold)
                .with_min_active_points(2)
                .with_merge_gap_points(2),
        );
    analyze_spectrum_1d(spectrum, options).map_err(|error| anyhow!(error))
}

fn analyze_2d(spectrum: &Spectrum2D) -> Result<SpectrumAnalysis2D> {
    let max_abs = max_abs(&spectrum.z);
    let p999 = percentile_abs(&spectrum.z, 0.999);
    let threshold = nonzero_threshold(max_abs, p999, 0.08, 0.30);
    let options = SpectrumAnalysis2DOptions::new().with_zone_options(
        ZoneDetectionOptions::new()
            .with_threshold_abs(threshold)
            .with_min_active_points(4)
            .with_connectivity(ZoneConnectivity::Eight),
    );
    analyze_spectrum_2d(spectrum, options).map_err(|error| anyhow!(error))
}

fn render_1d(
    case: &Case,
    spectrum: &Spectrum1D,
    analysis: &SpectrumAnalysis1D,
    output_path: &Path,
) -> Result<()> {
    let drawing_area = BitMapBackend::new(output_path, (OUT_WIDTH, OUT_HEIGHT)).into_drawing_area();
    drawing_area.fill(&WHITE)?;
    let (header, plot) = drawing_area.split_vertically(HEADER_HEIGHT);

    draw_header(
        &header,
        &case.label,
        &format!(
            "1D workflow: read -> peak picking -> range detection -> integration -> multiplet grouping -> signal summary"
        ),
        &[
            format!("source: {}", case.path.display()),
            format!(
                "points: {} | peaks: {} | ranges: {} | integrals: {} | multiplets: {} | signals: {}",
                spectrum.len(),
                analysis.peaks.len(),
                analysis.ranges.len(),
                analysis.integrals.len(),
                analysis.multiplets.len(),
                analysis.signals.len()
            ),
            metadata_line_1d(spectrum),
        ],
    )?;

    let (x_min, x_max) = axis_bounds(&spectrum.x.values)?;
    let (y_min, y_max) = visible_y_bounds(&spectrum.intensities)?;

    let mut chart = ChartBuilder::on(&plot)
        .margin(18)
        .set_label_area_size(LabelAreaPosition::Left, 80)
        .set_label_area_size(LabelAreaPosition::Bottom, 65)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)?;

    chart
        .configure_mesh()
        .x_desc(format!("{} ({:?})", spectrum.x.label, spectrum.x.unit))
        .y_desc("intensity")
        .light_line_style(RGBColor(235, 235, 235))
        .draw()?;

    let mut ranges: Vec<_> = analysis.ranges.iter().collect();
    ranges.sort_by(|left, right| {
        right
            .max_abs_intensity
            .total_cmp(&left.max_abs_intensity)
            .then_with(|| left.start_index.cmp(&right.start_index))
    });
    ranges.truncate(MAX_RANGE_MARKERS);
    chart.draw_series(ranges.iter().map(|range| {
        let x0 = range.from.min(range.to);
        let x1 = range.from.max(range.to);
        Rectangle::new(
            [(x0, y_min), (x1, y_max)],
            RGBAColor(47, 130, 120, 0.10).filled(),
        )
    }))?;

    chart.draw_series(LineSeries::new(
        downsample_line(&spectrum.x.values, &spectrum.intensities, MAX_LINE_POINTS),
        ShapeStyle::from(&RGBColor(32, 74, 110)).stroke_width(2),
    ))?;

    let mut peaks: Vec<_> = analysis.peaks.iter().collect();
    peaks.sort_by(|left, right| {
        right
            .intensity
            .abs()
            .total_cmp(&left.intensity.abs())
            .then_with(|| left.index.cmp(&right.index))
    });
    peaks.truncate(MAX_PEAK_MARKERS);
    chart.draw_series(peaks.iter().map(|peak| {
        Circle::new(
            (peak.x, peak.intensity.clamp(y_min, y_max)),
            4,
            ShapeStyle::from(&RGBColor(190, 54, 45)).filled(),
        )
    }))?;

    drawing_area.present()?;
    Ok(())
}

fn render_2d(
    case: &Case,
    spectrum: &Spectrum2D,
    analysis: &SpectrumAnalysis2D,
    output_path: &Path,
) -> Result<()> {
    let drawing_area = BitMapBackend::new(output_path, (OUT_WIDTH, OUT_HEIGHT)).into_drawing_area();
    drawing_area.fill(&WHITE)?;
    let (header, plot) = drawing_area.split_vertically(HEADER_HEIGHT);

    draw_header(
        &header,
        &case.label,
        "2D workflow: read -> zone detection -> zone integration -> signal summary",
        &[
            format!("source: {}", case.path.display()),
            format!(
                "shape: {} x {} | zones: {} | integrals: {} | signals: {}",
                spectrum.x.len(),
                spectrum.y.len(),
                analysis.zones.len(),
                analysis.integrals.len(),
                analysis.signals.len()
            ),
            metadata_line_2d(spectrum),
        ],
    )?;

    let (x_min, x_max) = axis_bounds(&spectrum.x.values)?;
    let (y_min, y_max) = axis_bounds(&spectrum.y.values)?;
    let mut chart = ChartBuilder::on(&plot)
        .margin(18)
        .set_label_area_size(LabelAreaPosition::Left, 80)
        .set_label_area_size(LabelAreaPosition::Bottom, 65)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)?;

    chart
        .configure_mesh()
        .x_desc(format!("{} ({:?})", spectrum.x.label, spectrum.x.unit))
        .y_desc(format!("{} ({:?})", spectrum.y.label, spectrum.y.unit))
        .light_line_style(RGBColor(235, 235, 235))
        .draw()?;

    let max_abs = max_abs(&spectrum.z);
    chart.draw_series(heatmap_blocks(spectrum).into_iter().map(|block| {
        Rectangle::new(
            [(block.x0, block.y0), (block.x1, block.y1)],
            heatmap_color(block.value, max_abs).filled(),
        )
    }))?;

    let mut zones: Vec<_> = analysis.zones.iter().collect();
    zones.sort_by(|left, right| {
        right
            .max_abs_intensity
            .total_cmp(&left.max_abs_intensity)
            .then_with(|| left.id.cmp(&right.id))
    });
    zones.truncate(MAX_ZONE_MARKERS);
    chart.draw_series(zones.iter().map(|zone| {
        let x0 = zone.x_from.min(zone.x_to);
        let x1 = zone.x_from.max(zone.x_to);
        let y0 = zone.y_from.min(zone.y_to);
        let y1 = zone.y_from.max(zone.y_to);
        Rectangle::new(
            [(x0, y0), (x1, y1)],
            ShapeStyle::from(&BLACK).stroke_width(2),
        )
    }))?;

    drawing_area.present()?;
    Ok(())
}

fn render_error_png(case: &Case, output_path: &Path, error: &anyhow::Error) -> Result<()> {
    let drawing_area = BitMapBackend::new(output_path, (OUT_WIDTH, OUT_HEIGHT)).into_drawing_area();
    drawing_area.fill(&WHITE)?;
    let text_style = ("sans-serif", 32).into_font().color(&RGBColor(40, 40, 40));
    drawing_area.draw(&Text::new(
        format!("Could not render {}", case.label),
        (40, 60),
        text_style,
    ))?;
    let detail_style = ("sans-serif", 22).into_font().color(&RGBColor(150, 45, 45));
    let message = error_chain(error);
    for (line_index, line) in wrap_text(&message, 110).iter().enumerate() {
        let y = 120 + line_offset(line_index, 34);
        drawing_area.draw(&Text::new(line.clone(), (40, y), detail_style.clone()))?;
    }
    drawing_area.present()?;
    Ok(())
}

fn draw_header<DB>(
    area: &DrawingArea<DB, plotters::coord::Shift>,
    title: &str,
    flow: &str,
    lines: &[String],
) -> Result<()>
where
    DB: DrawingBackend,
    DB::ErrorType: 'static,
{
    area.fill(&RGBColor(247, 247, 244))?;
    area.draw(&Text::new(
        title.to_owned(),
        (28, 34),
        ("sans-serif", 30).into_font().color(&RGBColor(28, 33, 38)),
    ))?;
    area.draw(&Text::new(
        flow.to_owned(),
        (28, 72),
        ("sans-serif", 21).into_font().color(&RGBColor(62, 74, 82)),
    ))?;
    for (index, line) in lines.iter().enumerate() {
        let y = 105 + line_offset(index, 26);
        area.draw(&Text::new(
            line.to_owned(),
            (28, y),
            ("sans-serif", 18).into_font().color(&RGBColor(70, 70, 70)),
        ))?;
    }
    Ok(())
}

#[derive(Clone, Copy, Debug)]
struct HeatmapBlock {
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
    value: f64,
}

fn heatmap_blocks(spectrum: &Spectrum2D) -> Vec<HeatmapBlock> {
    let x_len = spectrum.x.len();
    let y_len = spectrum.y.len();
    if x_len == 0 || y_len == 0 {
        return Vec::new();
    }

    let x_step = x_len.div_ceil(MAX_HEATMAP_BINS).max(1);
    let y_step = y_len.div_ceil(MAX_HEATMAP_BINS).max(1);
    let mut blocks = Vec::new();

    for y_start in (0..y_len).step_by(y_step) {
        let y_end = (y_start + y_step).min(y_len);
        for x_start in (0..x_len).step_by(x_step) {
            let x_end = (x_start + x_step).min(x_len);
            let mut selected = 0.0_f64;
            for y_index in y_start..y_end {
                let row_offset = y_index * x_len;
                for x_index in x_start..x_end {
                    let value = spectrum.z[row_offset + x_index];
                    if value.abs() > selected.abs() {
                        selected = value;
                    }
                }
            }

            let x_last = x_end.saturating_sub(1);
            let y_last = y_end.saturating_sub(1);
            let x0 = spectrum.x.values[x_start].min(spectrum.x.values[x_last]);
            let x1 = spectrum.x.values[x_start].max(spectrum.x.values[x_last]);
            let y0 = spectrum.y.values[y_start].min(spectrum.y.values[y_last]);
            let y1 = spectrum.y.values[y_start].max(spectrum.y.values[y_last]);
            blocks.push(HeatmapBlock {
                x0,
                x1,
                y0,
                y1,
                value: selected,
            });
        }
    }

    blocks
}

fn heatmap_color(value: f64, max_abs: f64) -> RGBAColor {
    if max_abs <= 0.0 {
        return RGBAColor(230, 230, 230, 0.25);
    }

    let magnitude = (value.abs() / max_abs).sqrt().clamp(0.0, 1.0);
    if value >= 0.0 {
        RGBColor(
            (244.0 * magnitude) as u8,
            (70.0 + 80.0 * (1.0 - magnitude)) as u8,
            (48.0 + 80.0 * (1.0 - magnitude)) as u8,
        )
        .mix(0.82)
    } else {
        RGBColor(
            (46.0 + 90.0 * (1.0 - magnitude)) as u8,
            (101.0 + 70.0 * (1.0 - magnitude)) as u8,
            (186.0 + 40.0 * magnitude) as u8,
        )
        .mix(0.82)
    }
}

fn downsample_line(x: &[f64], y: &[f64], max_points: usize) -> Vec<(f64, f64)> {
    if x.len() <= max_points || max_points < 2 {
        return x.iter().copied().zip(y.iter().copied()).collect();
    }

    let step = x.len().div_ceil(max_points);
    x.iter()
        .copied()
        .zip(y.iter().copied())
        .step_by(step)
        .collect()
}

fn axis_bounds(values: &[f64]) -> Result<(f64, f64)> {
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    for value in values.iter().copied().filter(|value| value.is_finite()) {
        min = min.min(value);
        max = max.max(value);
    }
    if !min.is_finite() || !max.is_finite() {
        return Err(anyhow!("axis has no finite values"));
    }
    if (max - min).abs() <= f64::EPSILON {
        Ok((min - 1.0, max + 1.0))
    } else {
        Ok((min, max))
    }
}

fn visible_y_bounds(values: &[f64]) -> Result<(f64, f64)> {
    let mut finite: Vec<_> = values
        .iter()
        .copied()
        .filter(|value| value.is_finite())
        .collect();
    if finite.is_empty() {
        return Err(anyhow!("spectrum has no finite intensities"));
    }
    finite.sort_by(f64::total_cmp);

    let low = percentile_sorted(&finite, 0.005);
    let high = percentile_sorted(&finite, 0.995);
    let min = finite[0].min(low);
    let max = finite[finite.len() - 1].max(high);
    let mut visible_low = low;
    let mut visible_high = high;
    if (visible_high - visible_low).abs() <= f64::EPSILON {
        visible_low = min - 1.0;
        visible_high = max + 1.0;
    }
    let padding = (visible_high - visible_low).abs() * 0.08;
    Ok((visible_low - padding, visible_high + padding))
}

fn max_abs(values: &[f64]) -> f64 {
    values
        .iter()
        .copied()
        .filter(|value| value.is_finite())
        .map(f64::abs)
        .fold(0.0, f64::max)
}

fn percentile_abs(values: &[f64], quantile: f64) -> f64 {
    let mut finite_abs: Vec<_> = values
        .iter()
        .copied()
        .filter(|value| value.is_finite())
        .map(f64::abs)
        .collect();
    if finite_abs.is_empty() {
        return 0.0;
    }
    finite_abs.sort_by(f64::total_cmp);
    percentile_sorted(&finite_abs, quantile)
}

fn percentile_sorted(sorted: &[f64], quantile: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let last_index = sorted.len().saturating_sub(1);
    let scaled = (last_index as f64 * quantile.clamp(0.0, 1.0)).round();
    let index = (scaled as usize).min(last_index);
    sorted[index]
}

fn nonzero_threshold(
    max_abs: f64,
    percentile: f64,
    max_fraction: f64,
    percentile_fraction: f64,
) -> f64 {
    let threshold = (max_abs * max_fraction).max(percentile * percentile_fraction);
    if threshold.is_finite() && threshold > 0.0 {
        threshold
    } else {
        0.0
    }
}

fn metadata_line_1d(spectrum: &Spectrum1D) -> String {
    format!(
        "axis: {} {:?} | nucleus: {} | frequency: {} | imaginary: {}",
        spectrum.x.label,
        spectrum.x.unit,
        option_debug(&spectrum.metadata.nucleus),
        option_number(spectrum.metadata.frequency_mhz),
        option_len(&spectrum.imaginary)
    )
}

fn metadata_line_2d(spectrum: &Spectrum2D) -> String {
    format!(
        "axes: {} {:?}, {} {:?} | nucleus: {} | frequency: {} | imaginary: {}",
        spectrum.x.label,
        spectrum.x.unit,
        spectrum.y.label,
        spectrum.y.unit,
        option_debug(&spectrum.metadata.nucleus),
        option_number(spectrum.metadata.frequency_mhz),
        option_len(&spectrum.imaginary)
    )
}

fn line_offset(index: usize, step: i32) -> i32 {
    match i32::try_from(index) {
        Ok(value) => value.saturating_mul(step),
        Err(_) => i32::MAX,
    }
}

fn option_len<T>(value: &Option<Vec<T>>) -> usize {
    match value {
        Some(values) => values.len(),
        None => 0,
    }
}

fn option_debug<T: std::fmt::Debug>(value: &Option<T>) -> String {
    match value {
        Some(inner) => format!("{inner:?}"),
        None => "n/a".to_owned(),
    }
}

fn option_number(value: Option<f64>) -> String {
    match value {
        Some(number) => format!("{number:.6} MHz"),
        None => "n/a".to_owned(),
    }
}

fn sanitize(value: &str) -> String {
    let mut result = String::new();
    let mut previous_was_sep = false;
    for character in value.chars() {
        let next = if character.is_ascii_alphanumeric() {
            previous_was_sep = false;
            Some(character.to_ascii_lowercase())
        } else if previous_was_sep {
            None
        } else {
            previous_was_sep = true;
            Some('_')
        };
        if let Some(character) = next {
            result.push(character);
        }
        if result.len() >= 120 {
            break;
        }
    }
    result.trim_matches('_').to_owned()
}

fn sanitize_summary(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}

fn error_chain(error: &anyhow::Error) -> String {
    error
        .chain()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(" | ")
}

fn wrap_text(value: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in value.split_whitespace() {
        if !current.is_empty() && current.len() + word.len() + 1 > width {
            lines.push(current);
            current = String::new();
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(value.to_owned());
    }
    lines
}
