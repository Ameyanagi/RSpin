#[cfg(feature = "visualization-ruviz")]
fn main() -> anyhow::Result<()> {
    ruviz_example::run()
}

#[cfg(not(feature = "visualization-ruviz"))]
fn main() {
    eprintln!(
        "Enable the visualization-ruviz feature to generate PNGs: \
         cargo run --manifest-path examples/ruviz-processing/Cargo.toml \
         --features visualization-ruviz"
    );
}

#[cfg(feature = "visualization-ruviz")]
mod ruviz_example {
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    use anyhow::{Context, Result};
    use rspin_analysis::{
        PeakPickOptions, PeakPolarity, RangeDetectionOptions, SpectrumAnalysis1D,
        SpectrumAnalysis1DOptions, analyze_spectrum_1d,
    };
    use rspin_core::{Axis, Metadata, Spectrum1D, Unit};
    use rspin_io::{
        SpectrumBundle, load_spectra, read_analysis1d_json, read_processing_recipe_1d_json,
        read_spectrum_bundle_json, read_spectrum1d_csv, read_spectrum1d_json, write_analysis1d_csv,
        write_analysis1d_json, write_processing_recipe_1d_json, write_spectrum_bundle_json,
        write_spectrum1d_csv, write_spectrum1d_json,
    };
    use rspin_processing::{BaselineMethod, FftDirection, ProcessingRecipe1D, fit_baseline};
    use ruviz::prelude::{IntoPlot, LegendPosition, Plot};

    pub fn run() -> Result<()> {
        let root = repo_root()?;
        let output_dir = root.join("docs/assets/examples");
        fs::create_dir_all(&output_dir).with_context(|| {
            format!(
                "failed to create example output directory {}",
                output_dir.display()
            )
        })?;

        write_recipe_chain_plot(&output_dir.join("processed_recipe_chain.png"))?;
        write_baseline_plot(&output_dir.join("processed_baseline.png"))?;
        write_analysis_plot(&output_dir.join("analysis_peaks_ranges.png"))?;
        let visual_output_dir = root.join("target/rspin-visual-tests");
        write_oracle_visual_artifacts(&root, &visual_output_dir)?;

        println!(
            "Generated {}, {}, {}, and local visual artifacts under {}",
            output_dir.join("processed_recipe_chain.png").display(),
            output_dir.join("processed_baseline.png").display(),
            output_dir.join("analysis_peaks_ranges.png").display(),
            visual_output_dir.display()
        );
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

    fn write_recipe_chain_plot(path: &Path) -> Result<()> {
        let raw = Spectrum1D::new(
            Axis::linear("point", Unit::Points, 0.0, 2.0, 3)?,
            vec![1.0, -2.0, 4.0],
            Metadata::new().with_name("recipe-chain-demo"),
        )?;
        let scaled = ProcessingRecipe1D::new().scale(2.0).apply(&raw)?;
        let offset = ProcessingRecipe1D::new()
            .scale(2.0)
            .offset(-2.0)
            .apply(&raw)?;
        let absolute = ProcessingRecipe1D::new()
            .scale(2.0)
            .offset(-2.0)
            .absolute_value()
            .apply(&raw)?;
        let normalized = ProcessingRecipe1D::new()
            .scale(2.0)
            .offset(-2.0)
            .absolute_value()
            .normalize_max_abs()
            .apply(&raw)?;

        Plot::new()
            .title("RSpin 1D Processing Recipe")
            .xlabel("point")
            .ylabel("intensity")
            .max_resolution(1600, 1000)
            .legend_position(LegendPosition::LowerLeft)
            .line(&raw.x.values, &raw.intensities)
            .label("raw")
            .line(&raw.x.values, &scaled.intensities)
            .label("scale x2")
            .line(&raw.x.values, &offset.intensities)
            .label("offset -2")
            .line(&raw.x.values, &absolute.intensities)
            .label("absolute")
            .line(&raw.x.values, &normalized.intensities)
            .label("normalized")
            .save(path_to_str(path)?)?;

        Ok(())
    }

    fn write_baseline_plot(path: &Path) -> Result<()> {
        let spectrum = synthetic_baseline_spectrum()?;
        let method = BaselineMethod::WhittakerAsls {
            lambda: 1.0e5,
            p: 0.01,
            max_iter: 50,
            tolerance: 1.0e-4,
        };
        let fit = fit_baseline(&spectrum, method)?;
        let processed = ProcessingRecipe1D::new()
            .subtract_baseline_with(method)
            .normalize_max_abs()
            .apply(&spectrum)?;

        Plot::new()
            .title("RSpin Baseline Correction")
            .xlabel("chemical shift / ppm")
            .ylabel("intensity")
            .max_resolution(1600, 1000)
            .legend_position(LegendPosition::Best)
            .line(&spectrum.x.values, &spectrum.intensities)
            .label("raw")
            .line(&spectrum.x.values, &fit.baseline)
            .label("fitted baseline")
            .line(&processed.x.values, &processed.intensities)
            .label("corrected normalized")
            .save(path_to_str(path)?)?;

        Ok(())
    }

    fn write_analysis_plot(path: &Path) -> Result<()> {
        let spectrum = synthetic_analysis_spectrum()?;
        let analysis = analyze_for_visual(&spectrum)?;
        write_analysis_overlay_plot(path, "RSpin 1D Analysis", &spectrum, &analysis)
    }

    fn write_oracle_visual_artifacts(root: &Path, output_dir: &Path) -> Result<()> {
        fs::create_dir_all(output_dir).with_context(|| {
            format!(
                "failed to create local visual output directory {}",
                output_dir.display()
            )
        })?;
        write_text(
            &output_dir.join("README.txt"),
            "\
RSpin local visual artifacts.

These files are generated from local raw oracle fixtures and are intentionally untracked.
Use them for visual checks after IO or processing changes.
JSON and CSV outputs are consistency artifacts; PNG outputs are generated with ruviz.
",
        )?;

        let fixture_root = root.join("crates/rspin-io/testdata/zenodo_7100132");
        write_varian_oracle_artifacts(&fixture_root, &output_dir.join("varian_1h"))?;
        write_bruker_oracle_artifacts(&fixture_root, &output_dir.join("bruker_without_expno"))?;
        Ok(())
    }

    fn write_varian_oracle_artifacts(fixture_root: &Path, output_dir: &Path) -> Result<()> {
        fs::create_dir_all(output_dir).with_context(|| {
            format!(
                "failed to create Varian/Agilent visual output directory {}",
                output_dir.display()
            )
        })?;
        let bundle = load_spectra(fixture_root.join("varian_1h"))?;
        write_bundle_outputs(output_dir, &bundle)?;

        let raw = bundle.only_1d()?;
        let recipe = raw_fid_processing_recipe(raw)?;
        let processed = recipe.apply(raw)?;

        write_spectrum_outputs(output_dir, "raw_fid", raw)?;
        write_spectrum_outputs(output_dir, "processed_fft_magnitude", &processed)?;
        write_recipe_output(output_dir, &recipe)?;
        write_spectrum_plot(
            &output_dir.join("raw_fid.png"),
            "Oracle Varian/Agilent Raw FID",
            axis_label(raw.x.unit),
            "intensity",
            &raw.x.values,
            &raw.intensities,
            "real",
        )?;
        write_spectrum_plot(
            &output_dir.join("processed_fft_magnitude.png"),
            "Oracle Varian/Agilent FFT Magnitude",
            axis_label(processed.x.unit),
            "normalized magnitude",
            &processed.x.values,
            &processed.intensities,
            "spectrum",
        )?;
        Ok(())
    }

    fn write_bruker_oracle_artifacts(fixture_root: &Path, output_dir: &Path) -> Result<()> {
        fs::create_dir_all(output_dir).with_context(|| {
            format!(
                "failed to create Bruker visual output directory {}",
                output_dir.display()
            )
        })?;
        let bundle = load_spectra(fixture_root.join("bruker_without_expno"))?;
        write_bundle_outputs(output_dir, &bundle)?;

        let raw = spectrum_with_unit(&bundle, Unit::Seconds, "raw Bruker FID")?;
        let processed = spectrum_with_unit(&bundle, Unit::Ppm, "processed Bruker spectrum")?;
        let normalized = ProcessingRecipe1D::new()
            .normalize_max_abs()
            .apply(processed)?;
        let analysis = analyze_for_visual(&normalized)?;

        write_spectrum_outputs(output_dir, "raw_fid", raw)?;
        write_spectrum_outputs(output_dir, "processed_vendor", processed)?;
        write_spectrum_outputs(output_dir, "processed_vendor_normalized", &normalized)?;
        write_analysis_outputs(output_dir, &analysis)?;
        write_spectrum_plot(
            &output_dir.join("raw_fid.png"),
            "Oracle Bruker Raw FID",
            axis_label(raw.x.unit),
            "intensity",
            &raw.x.values,
            &raw.intensities,
            "real",
        )?;
        write_spectrum_plot(
            &output_dir.join("processed_vendor_normalized.png"),
            "Oracle Bruker Processed Spectrum",
            axis_label(normalized.x.unit),
            "normalized intensity",
            &normalized.x.values,
            &normalized.intensities,
            "spectrum",
        )?;
        write_analysis_overlay_plot(
            &output_dir.join("analysis_peaks_ranges.png"),
            "Oracle Bruker Analysis",
            &normalized,
            &analysis,
        )?;
        Ok(())
    }

    fn write_bundle_outputs(output_dir: &Path, bundle: &SpectrumBundle) -> Result<()> {
        let json = write_spectrum_bundle_json(bundle)?;
        read_spectrum_bundle_json(&json)?;
        write_text(&output_dir.join("bundle.json"), &json)
    }

    fn write_spectrum_outputs(output_dir: &Path, stem: &str, spectrum: &Spectrum1D) -> Result<()> {
        let json = write_spectrum1d_json(spectrum)?;
        read_spectrum1d_json(&json)?;
        let csv = write_spectrum1d_csv(spectrum)?;
        read_spectrum1d_csv(&csv)?;

        write_text(&output_dir.join(format!("{stem}.json")), &json)?;
        write_text(&output_dir.join(format!("{stem}.csv")), &csv)
    }

    fn write_recipe_output(output_dir: &Path, recipe: &ProcessingRecipe1D) -> Result<()> {
        let json = write_processing_recipe_1d_json(recipe)?;
        read_processing_recipe_1d_json(&json)?;
        write_text(&output_dir.join("processing_recipe.json"), &json)
    }

    fn write_analysis_outputs(output_dir: &Path, analysis: &SpectrumAnalysis1D) -> Result<()> {
        let json = write_analysis1d_json(analysis)?;
        read_analysis1d_json(&json)?;
        let csv = write_analysis1d_csv(analysis)?;

        write_text(&output_dir.join("analysis.json"), &json)?;
        write_text(&output_dir.join("analysis.csv"), &csv)
    }

    fn write_text(path: &Path, contents: &str) -> Result<()> {
        fs::write(path, contents).with_context(|| format!("failed to write {}", path.display()))?;
        Ok(())
    }

    fn raw_fid_processing_recipe(spectrum: &Spectrum1D) -> Result<ProcessingRecipe1D> {
        let target_len = spectrum
            .len()
            .checked_mul(2)
            .context("raw FID processing target length overflow")?;
        Ok(ProcessingRecipe1D::new()
            .exponential_apodization(1.0, dwell_time_seconds(spectrum)?)
            .zero_fill(target_len)
            .fft(FftDirection::Forward)
            .magnitude()
            .normalize_max_abs())
    }

    fn dwell_time_seconds(spectrum: &Spectrum1D) -> Result<f64> {
        if spectrum.x.unit != Unit::Seconds {
            anyhow::bail!("raw FID x-axis must be in seconds");
        }
        let first = spectrum
            .x
            .values
            .first()
            .context("raw FID needs at least two time points")?;
        let second = spectrum
            .x
            .values
            .get(1)
            .context("raw FID needs at least two time points")?;
        let dwell_time = (second - first).abs();
        if dwell_time.is_finite() && dwell_time > 0.0 {
            Ok(dwell_time)
        } else {
            anyhow::bail!("raw FID dwell time must be positive and finite")
        }
    }

    fn spectrum_with_unit<'a>(
        bundle: &'a SpectrumBundle,
        unit: Unit,
        description: &str,
    ) -> Result<&'a Spectrum1D> {
        bundle
            .spectra_1d()
            .find(|spectrum| spectrum.x.unit == unit)
            .with_context(|| format!("missing {description}"))
    }

    fn analyze_for_visual(spectrum: &Spectrum1D) -> Result<SpectrumAnalysis1D> {
        Ok(analyze_spectrum_1d(
            spectrum,
            SpectrumAnalysis1DOptions::new()
                .with_peak_options(
                    PeakPickOptions::new()
                        .with_min_abs_intensity(0.20)
                        .with_min_prominence(0.003)
                        .with_polarity(PeakPolarity::Both),
                )
                .with_range_options(
                    RangeDetectionOptions::new()
                        .with_threshold_abs(0.16)
                        .with_min_active_points(3)
                        .with_merge_gap_points(2),
                ),
        )?)
    }

    fn write_analysis_overlay_plot(
        path: &Path,
        title: &str,
        spectrum: &Spectrum1D,
        analysis: &SpectrumAnalysis1D,
    ) -> Result<()> {
        let peak_x = analysis.peaks.iter().map(|peak| peak.x).collect::<Vec<_>>();
        let peak_y = analysis
            .peaks
            .iter()
            .map(|peak| peak.intensity)
            .collect::<Vec<_>>();
        let (range_x, range_y) = range_points(spectrum, &analysis.ranges);

        let mut plot = Plot::new()
            .title(title)
            .xlabel(axis_label(spectrum.x.unit))
            .ylabel("intensity")
            .max_resolution(1600, 1000)
            .legend_position(LegendPosition::Best)
            .line(&spectrum.x.values, &spectrum.intensities)
            .label("spectrum")
            .into_plot();
        if !range_x.is_empty() {
            plot = plot
                .scatter(&range_x, &range_y)
                .marker_size(4.0)
                .label("detected ranges")
                .into_plot();
        }
        if !peak_x.is_empty() {
            plot = plot
                .scatter(&peak_x, &peak_y)
                .marker_size(10.0)
                .label("picked peaks")
                .into_plot();
        }
        plot.save(path_to_str(path)?)?;

        Ok(())
    }

    fn write_spectrum_plot(
        path: &Path,
        title: &str,
        x_label: &str,
        y_label: &str,
        x: &[f64],
        y: &[f64],
        series_label: &str,
    ) -> Result<()> {
        Plot::new()
            .title(title)
            .xlabel(x_label)
            .ylabel(y_label)
            .max_resolution(1600, 1000)
            .legend_position(LegendPosition::Best)
            .line(&x, &y)
            .label(series_label)
            .save(path_to_str(path)?)?;
        Ok(())
    }

    fn axis_label(unit: Unit) -> &'static str {
        match unit {
            Unit::Ppm => "chemical shift / ppm",
            Unit::Hertz => "frequency / Hz",
            Unit::Seconds => "time / s",
            Unit::Points => "point",
            Unit::Arbitrary => "x",
            _ => "x",
        }
    }

    fn synthetic_baseline_spectrum() -> Result<Spectrum1D> {
        let axis = Axis::linear_ppm(0.0, 10.0, 512)?;
        let intensities = axis
            .values
            .iter()
            .copied()
            .map(|x| {
                let baseline = 0.18 + 0.035 * x + 0.06 * (0.8 * x).sin();
                baseline
                    + gaussian(x, 2.2, 0.07, 1.4)
                    + gaussian(x, 4.8, 0.12, 0.9)
                    + gaussian(x, 7.3, 0.18, 1.2)
            })
            .collect();

        Ok(Spectrum1D::new(
            axis,
            intensities,
            Metadata::new()
                .with_name("baseline-demo")
                .with_frequency_mhz(400.0),
        )?)
    }

    fn synthetic_analysis_spectrum() -> Result<Spectrum1D> {
        let axis = Axis::linear_ppm(0.0, 10.0, 700)?;
        let intensities = axis
            .values
            .iter()
            .copied()
            .map(|x| {
                0.015 * (4.0 * x).sin()
                    + gaussian(x, 1.5, 0.06, 0.55)
                    + gaussian(x, 2.2, 0.08, 0.78)
                    - gaussian(x, 4.1, 0.10, 0.45)
                    + gaussian(x, 5.35, 0.13, 0.72)
                    + gaussian(x, 5.80, 0.08, 0.38)
                    - gaussian(x, 8.4, 0.16, 0.30)
            })
            .collect();

        Ok(Spectrum1D::new(
            axis,
            intensities,
            Metadata::new()
                .with_name("analysis-demo")
                .with_frequency_mhz(400.0),
        )?)
    }

    fn range_points(
        spectrum: &Spectrum1D,
        ranges: &[rspin_analysis::DetectedRange],
    ) -> (Vec<f64>, Vec<f64>) {
        let mut x = Vec::new();
        let mut y = Vec::new();
        for range in ranges {
            for index in range.start_index..=range.end_index {
                if let (Some(x_value), Some(y_value)) = (
                    spectrum.x.values.get(index),
                    spectrum.intensities.get(index),
                ) {
                    x.push(*x_value);
                    y.push(*y_value);
                }
            }
        }
        (x, y)
    }

    fn gaussian(x: f64, center: f64, width: f64, height: f64) -> f64 {
        let scaled = (x - center) / width;
        height * (-0.5 * scaled * scaled).exp()
    }

    fn path_to_str(path: &Path) -> Result<&str> {
        path.to_str()
            .with_context(|| format!("path is not valid UTF-8: {}", path.display()))
    }
}
