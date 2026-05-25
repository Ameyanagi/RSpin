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
        SpectrumAnalysis1DOptions, analyze_spectrum_1d, pick_peaks,
    };
    use rspin_core::{Axis, Metadata, Spectrum1D, Spectrum2D, Unit};
    use rspin_io::{
        SpectrumBundle, load_spectra, read_analysis1d_json, read_processing_recipe_1d_json,
        read_spectrum_bundle_json, read_spectrum1d_csv, read_spectrum1d_json, write_analysis1d_csv,
        write_analysis1d_json, write_processing_recipe_1d_json, write_spectrum_bundle_json,
        write_spectrum1d_csv, write_spectrum1d_json,
    };
    use rspin_processing::{
        AutoPhaseCost, AutoPhaseOptions, AutoPhaseStrategy, BaselineMethod, FftDirection,
        ProcessSpectrum2D, ProcessingRecipe1D, apply_subsample_shift, auto_phase_correct,
        auto_phase_correct_with_peaks, fit_baseline, remove_group_delay,
    };
    use ruviz::prelude::{IntoPlot, LegendPosition, Plot};
    use ruviz::core::subplot::subplots;

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
        write_curated_auto_phase_plot(&root, &output_dir)?;
        let vendor_dir = output_dir.join("vendors");
        write_vendor_showcase(&root, &vendor_dir)?;
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

    fn write_curated_auto_phase_plot(root: &Path, output_dir: &Path) -> Result<()> {
        let fixture_root = root.join("crates/rspin-io/testdata/zenodo_7100132");
        let bundle = load_spectra(fixture_root.join("varian_1h"))?;
        let raw = bundle.only_1d()?;
        write_auto_phase_plot(output_dir, raw)?;
        write_auto_phase_comparison_plot(output_dir, raw)?;
        write_auto_phase_peak_zoom_plot(output_dir, raw)?;
        write_jeol_auto_phase_plots(root, output_dir)?;
        write_jeol_method_panels(root, output_dir)?;
        Ok(())
    }

    fn write_jeol_method_panels(root: &Path, output_dir: &Path) -> Result<()> {
        let entries: &[(&str, &str, &str)] = &[
            (
                "crates/rspin-io/testdata/nmrxiv/cc0/myrcene/jeol/myrcene_13c_400mhz.jdf",
                "auto_phase_methods_jeol_13c",
                "JEOL 13C Myrcene (NMRXiv CC0)",
            ),
            (
                "crates/rspin-io/testdata/nmrxiv/cc0/myrcene/jeol/myrcene_1h_400mhz.jdf",
                "auto_phase_methods_jeol_myrcene_1h",
                "JEOL 1H Myrcene (NMRXiv CC0)",
            ),
            (
                "crates/rspin-io/testdata/dataverse/cc0/rutin/jeol/rutin_qhnmr_400mhz.jdf",
                "auto_phase_methods_jeol_rutin",
                "JEOL 1H Rutin (Dataverse CC0)",
            ),
        ];
        for (path, stem, title) in entries {
            write_jeol_method_panel(root, output_dir, path, stem, title)?;
        }
        write_varian_method_panel(root, output_dir)?;
        Ok(())
    }

    fn write_jeol_method_panel(
        root: &Path,
        output_dir: &Path,
        fixture: &str,
        stem: &str,
        title: &str,
    ) -> Result<()> {
        let bundle = load_spectra(root.join(fixture))?;
        let Some(spectrum) = bundle.spectra_1d().next() else {
            return Ok(());
        };
        if spectrum.imaginary.is_none() {
            return Ok(());
        }
        let auto = jeol_group_delay(spectrum);
        let (magnitude, _) = jeol_magnitude(spectrum, auto)?;

        let legacy = jeol_phase_with_opts(
            spectrum,
            auto,
            AutoPhaseOptions::default()
                .with_strategy(AutoPhaseStrategy::GlobalCost)
                .with_cost(AutoPhaseCost::LegacyImagNegArea)
                .with_refine(false),
        )?;
        let acme = jeol_phase_with_opts(
            spectrum,
            auto,
            AutoPhaseOptions::default()
                .with_strategy(AutoPhaseStrategy::GlobalCost)
                .with_cost(AutoPhaseCost::AcmeEntropy),
        )?;
        let regions = jeol_phase_with_opts(spectrum, auto, AutoPhaseOptions::default())?;

        save_method_panel(
            output_dir,
            stem,
            title,
            &magnitude,
            &legacy,
            &acme,
            &regions,
        )
    }

    fn write_varian_method_panel(root: &Path, output_dir: &Path) -> Result<()> {
        let bundle =
            load_spectra(root.join("crates/rspin-io/testdata/zenodo_7100132/varian_1h"))?;
        let raw = bundle.only_1d()?;
        let target_len = raw
            .len()
            .checked_mul(2)
            .context("varian method-panel target length overflow")?;
        let magnitude = ProcessingRecipe1D::new()
            .exponential_apodization(1.0, dwell_time_seconds(raw)?)
            .zero_fill(target_len)
            .fft(FftDirection::Forward)
            .magnitude()
            .normalize_max_abs()
            .apply(raw)?;
        let magnitude = relabel_hz_to_ppm(magnitude);

        let prepare = |opts: AutoPhaseOptions| -> Result<rspin_processing::AutoPhaseResult> {
            let complex_recipe = ProcessingRecipe1D::new()
                .exponential_apodization(1.0, dwell_time_seconds(raw)?)
                .zero_fill(target_len)
                .fft(FftDirection::Forward)
                .normalize_max_abs();
            let unphased = complex_recipe.apply(raw)?;
            Ok(auto_phase_correct(&unphased, opts)?)
        };

        let legacy = prepare(
            AutoPhaseOptions::default()
                .with_strategy(AutoPhaseStrategy::GlobalCost)
                .with_cost(AutoPhaseCost::LegacyImagNegArea)
                .with_refine(false),
        )?;
        let acme = prepare(
            AutoPhaseOptions::default()
                .with_strategy(AutoPhaseStrategy::GlobalCost)
                .with_cost(AutoPhaseCost::AcmeEntropy),
        )?;
        let regions = prepare(AutoPhaseOptions::default())?;

        save_method_panel(
            output_dir,
            "auto_phase_methods_varian_1h",
            "Varian 1H (Zenodo MIT)",
            &magnitude,
            &legacy,
            &acme,
            &regions,
        )
    }

    fn save_method_panel(
        output_dir: &Path,
        stem: &str,
        title: &str,
        magnitude: &Spectrum1D,
        legacy: &rspin_processing::AutoPhaseResult,
        acme: &rspin_processing::AutoPhaseResult,
        regions: &rspin_processing::AutoPhaseResult,
    ) -> Result<()> {
        let mk = |panel_title: String, xs: &Vec<f64>, ys: &Vec<f64>, label: &str| -> Plot {
            Plot::new()
                .title(&panel_title)
                .xlabel("chemical shift / ppm")
                .ylabel("intensity")
                .max_resolution(900, 600)
                .legend_position(LegendPosition::Best)
                .line(xs, ys)
                .label(label)
                .into()
        };
        let magnitude_panel = mk(
            format!("{title} — magnitude (reference)"),
            &magnitude.x.values,
            &magnitude.intensities,
            "|spectrum|",
        );
        let legacy_panel = mk(
            format!(
                "legacy ({:.0}\u{00B0}/{:.0}\u{00B0})",
                legacy.zero_order_deg, legacy.first_order_deg
            ),
            &legacy.spectrum.x.values,
            &legacy.spectrum.intensities,
            "real",
        );
        let acme_panel = mk(
            format!(
                "ACME entropy ({:.0}\u{00B0}/{:.0}\u{00B0})",
                acme.zero_order_deg, acme.first_order_deg
            ),
            &acme.spectrum.x.values,
            &acme.spectrum.intensities,
            "real",
        );
        let regions_panel = mk(
            format!(
                "Regions Zorin 2017 ({:.0}\u{00B0}/{:.0}\u{00B0})",
                regions.zero_order_deg, regions.first_order_deg
            ),
            &regions.spectrum.x.values,
            &regions.spectrum.intensities,
            "real",
        );
        let figure = subplots(2, 2, 1800, 1200)?
            .subplot_at(0, magnitude_panel)?
            .subplot_at(1, legacy_panel)?
            .subplot_at(2, acme_panel)?
            .subplot_at(3, regions_panel)?;
        figure.save(path_to_str(&output_dir.join(format!("{stem}.png")))?)?;
        Ok(())
    }

    fn write_jeol_auto_phase_plots(root: &Path, output_dir: &Path) -> Result<()> {
        let entries: &[(&str, &str, &str)] = &[
            (
                "crates/rspin-io/testdata/nmrxiv/cc0/myrcene/jeol/myrcene_1h_400mhz.jdf",
                "JEOL 1H Auto-Phase (NMRXiv CC0 Myrcene)",
                "auto_phase_jeol_myrcene_1h",
            ),
            (
                "crates/rspin-io/testdata/nmrxiv/cc0/myrcene/jeol/myrcene_13c_400mhz.jdf",
                "JEOL 13C Auto-Phase (NMRXiv CC0 Myrcene)",
                "auto_phase_jeol_myrcene_13c",
            ),
            (
                "crates/rspin-io/testdata/dataverse/cc0/rutin/jeol/rutin_qhnmr_400mhz.jdf",
                "JEOL 1H Auto-Phase (Dataverse CC0 Rutin)",
                "auto_phase_jeol_rutin_qh",
            ),
        ];
        for (fixture, title, stem) in entries {
            let bundle = load_spectra(root.join(fixture))?;
            let Some(spectrum) = bundle.spectra_1d().next() else {
                continue;
            };
            if spectrum.imaginary.is_none() {
                eprintln!(
                    "skipping JEOL auto-phase plot {fixture}: spectrum has no imaginary channel"
                );
                continue;
            }
            // Header-derived group-delay shift.
            let auto = jeol_group_delay(spectrum);
            // Magnitude reference (phase-independent absorption envelope).
            let (magnitude, _) = jeol_magnitude(spectrum, auto)?;

            // Three phased traces using the same group-delay shift.
            let legacy_opts = AutoPhaseOptions::default()
                .with_strategy(AutoPhaseStrategy::GlobalCost)
                .with_cost(AutoPhaseCost::LegacyImagNegArea)
                .with_refine(false);
            let acme_opts = AutoPhaseOptions::default()
                .with_strategy(AutoPhaseStrategy::GlobalCost)
                .with_cost(AutoPhaseCost::AcmeEntropy);
            let regions_opts = AutoPhaseOptions::default();
            let legacy = jeol_phase_with_opts(spectrum, auto, legacy_opts)?;
            let acme = jeol_phase_with_opts(spectrum, auto, acme_opts)?;
            let regions = jeol_phase_with_opts(spectrum, auto, regions_opts)?;

            let fmt = |label: &str, r: &rspin_processing::AutoPhaseResult| {
                let neg = negative_fraction(&r.spectrum.intensities);
                format!(
                    "{label} ({:.0}\u{00B0}/{:.0}\u{00B0} neg={:.2})",
                    r.zero_order_deg, r.first_order_deg, neg
                )
            };

            Plot::new()
                .title(*title)
                .xlabel("chemical shift / ppm")
                .ylabel("normalized intensity")
                .max_resolution(1600, 1000)
                .legend_position(LegendPosition::Best)
                .line(&magnitude.x.values, &magnitude.intensities)
                .label(&format!("magnitude (shift={auto:.1})"))
                .line(&legacy.spectrum.x.values, &legacy.spectrum.intensities)
                .label(&fmt("legacy", &legacy))
                .line(&acme.spectrum.x.values, &acme.spectrum.intensities)
                .label(&fmt("ACME", &acme))
                .line(&regions.spectrum.x.values, &regions.spectrum.intensities)
                .label(&fmt("Regions (Zorin 2017)", &regions))
                .save(path_to_str(&output_dir.join(format!("{stem}.png")))?)?;
        }
        Ok(())
    }

    /// Line broadening (Hz) used when building the spectrum that the
    /// auto-phase cost function inspects. Heavier apodization than the
    /// final display spectrum suppresses truncation sidelobes that
    /// would otherwise be interpreted as residual dispersion.
    const PHASING_LINE_BROADENING_HZ: f64 = 5.0;
    /// Line broadening (Hz) used for the visualisation spectrum.
    const DISPLAY_LINE_BROADENING_HZ: f64 = 1.0;

    fn jeol_prepare_complex_spectrum(
        spectrum: &Spectrum1D,
        shift: f64,
        line_broadening_hz: f64,
    ) -> Result<Spectrum1D> {
        let integer = shift.trunc().max(0.0);
        let frac = shift - integer;
        let shifted = if integer > 0.0 {
            remove_group_delay(spectrum, integer)?
        } else {
            spectrum.clone()
        };
        let post_fft = if shifted.x.unit == Unit::Seconds {
            let target_len = shifted
                .len()
                .checked_mul(2)
                .context("JEOL prep target length overflow")?;
            ProcessingRecipe1D::new()
                .exponential_apodization(line_broadening_hz, dwell_time_seconds(&shifted)?)
                .zero_fill(target_len)
                .fft(FftDirection::Forward)
                .apply(&shifted)?
        } else {
            shifted
        };
        let polished = if frac.abs() > 1.0e-6 {
            apply_subsample_shift(&post_fft, frac)?
        } else {
            post_fft
        };
        let normalized = ProcessingRecipe1D::new()
            .normalize_max_abs()
            .apply(&polished)?;
        Ok(relabel_hz_to_ppm(normalized))
    }

    fn jeol_phase_with_opts(
        spectrum: &Spectrum1D,
        shift: f64,
        options: AutoPhaseOptions,
    ) -> Result<rspin_processing::AutoPhaseResult> {
        // Run auto-phase on a *heavily* apodized copy so truncation
        // sidelobes do not pollute the cost function, then re-apply the
        // measured (ph0, ph1) to a lightly-apodized display spectrum.
        let phasing_spectrum =
            jeol_prepare_complex_spectrum(spectrum, shift, PHASING_LINE_BROADENING_HZ)?;
        let phasing_result = auto_phase_correct(&phasing_spectrum, options)?;
        let display_spectrum =
            jeol_prepare_complex_spectrum(spectrum, shift, DISPLAY_LINE_BROADENING_HZ)?;
        let pivot_fraction = options.pivot_fraction;
        let display_phased = rspin_processing::phase_correct(
            &display_spectrum,
            phasing_result.zero_order_deg,
            phasing_result.first_order_deg,
            pivot_fraction,
        )?;
        Ok(rspin_processing::AutoPhaseResult {
            spectrum: display_phased,
            zero_order_deg: phasing_result.zero_order_deg,
            first_order_deg: phasing_result.first_order_deg,
            score: phasing_result.score,
        })
    }

    fn jeol_magnitude(
        spectrum: &Spectrum1D,
        shift: f64,
    ) -> Result<(Spectrum1D, rspin_processing::AutoPhaseResult)> {
        let prepared =
            jeol_prepare_complex_spectrum(spectrum, shift, DISPLAY_LINE_BROADENING_HZ)?;
        let magnitude = ProcessingRecipe1D::new()
            .magnitude()
            .normalize_max_abs()
            .apply(&prepared)?;
        let dummy = rspin_processing::AutoPhaseResult {
            spectrum: magnitude.clone(),
            zero_order_deg: 0.0,
            first_order_deg: 0.0,
            score: 0.0,
        };
        Ok((magnitude, dummy))
    }

    fn jeol_best_shift(
        spectrum: &Spectrum1D,
        magnitude: &Spectrum1D,
    ) -> Result<(f64, rspin_processing::AutoPhaseResult)> {
        // Coarse sweep, then refine around the best.
        let coarse: Vec<f64> = (0..=30).map(|i| f64::from(i as u32) * 4.0).collect();
        let mut best_shift = 0.0_f64;
        let mut best_loss = f64::INFINITY;
        let mut best_result: Option<rspin_processing::AutoPhaseResult> = None;
        for &candidate in &coarse {
            let (_unphased, phased) = jeol_phase_with_shift(spectrum, candidate)?;
            let loss = magnitude_target_loss(magnitude, &phased.spectrum);
            if loss < best_loss {
                best_loss = loss;
                best_shift = candidate;
                best_result = Some(phased);
            }
        }
        // Refine around the best coarse winner.
        let lo = (best_shift - 4.0).max(0.0);
        let hi = best_shift + 4.0;
        let steps: u32 = 16;
        for i in 0..=steps {
            let candidate = lo + (hi - lo) * f64::from(i) / f64::from(steps);
            let (_unphased, phased) = jeol_phase_with_shift(spectrum, candidate)?;
            let loss = magnitude_target_loss(magnitude, &phased.spectrum);
            if loss < best_loss {
                best_loss = loss;
                best_shift = candidate;
                best_result = Some(phased);
            }
        }
        Ok((best_shift, best_result.context("best shift not found")?))
    }

    fn magnitude_target_loss(magnitude: &Spectrum1D, phased_real: &Spectrum1D) -> f64 {
        // Weighted least squares over the high-magnitude region.
        let max_mag = magnitude
            .intensities
            .iter()
            .copied()
            .fold(0.0_f64, f64::max);
        if max_mag <= 0.0 {
            return f64::INFINITY;
        }
        let threshold = 0.02 * max_mag;
        let mut loss = 0.0_f64;
        let n = magnitude.intensities.len().min(phased_real.intensities.len());
        for index in 0..n {
            let m = magnitude.intensities[index];
            if m < threshold {
                continue;
            }
            let r = phased_real.intensities[index];
            let diff = m - r;
            loss += m * diff * diff;
        }
        loss
    }

    fn negative_fraction(values: &[f64]) -> f64 {
        let neg: f64 = values
            .iter()
            .map(|v| if *v < 0.0 { v.abs() } else { 0.0 })
            .sum();
        let total: f64 = values.iter().map(|v| v.abs()).sum();
        if total <= 0.0 { 0.0 } else { neg / total }
    }

    fn jeol_phase_with_shift(
        spectrum: &Spectrum1D,
        shift_samples: f64,
    ) -> Result<(Spectrum1D, rspin_processing::AutoPhaseResult)> {
        let shifted = if shift_samples > 0.0 {
            remove_group_delay(spectrum, shift_samples)?
        } else {
            spectrum.clone()
        };
        let unphased = if shifted.x.unit == Unit::Seconds {
            let target_len = shifted
                .len()
                .checked_mul(2)
                .context("JEOL auto-phase target length overflow")?;
            ProcessingRecipe1D::new()
                .exponential_apodization(1.0, dwell_time_seconds(&shifted)?)
                .zero_fill(target_len)
                .fft(FftDirection::Forward)
                .normalize_max_abs()
                .apply(&shifted)?
        } else {
            ProcessingRecipe1D::new()
                .normalize_max_abs()
                .apply(&shifted)?
        };
        let unphased = relabel_hz_to_ppm(unphased);
        let phased = auto_phase_correct(&unphased, AutoPhaseOptions::default())?;
        Ok((unphased, phased))
    }

    fn jeol_group_delay(spectrum: &Spectrum1D) -> f64 {
        let props = &spectrum.metadata.properties;
        let factor = props
            .get("jeol.parameter.filter_factor")
            .and_then(|v| v.parse::<f64>().ok());
        let decim_raw = props
            .get("jeol.parameter.decimation_reg")
            .and_then(|v| parse_decimation_reg(v));
        match (decim_raw, factor) {
            (Some(raw), Some(f)) if f > 0.0 => raw / f,
            _ => 0.0,
        }
    }

    fn parse_decimation_reg(raw: &str) -> Option<f64> {
        let trimmed = raw.trim();
        let after = trimmed.strip_prefix("r:")?.trim_start();
        // Prefer the parenthesized "adjusted" value (e.g. "r: 834( 833)") —
        // JEOL appears to track both the raw and effective FIR-tap counts
        // here, and the adjusted value lines up with the true group delay.
        if let Some(open) = after.find('(') {
            let after_paren = &after[open + 1..];
            let token = after_paren
                .split(|c: char| !c.is_ascii_digit())
                .find(|t| !t.is_empty());
            if let Some(t) = token {
                if let Ok(value) = t.parse::<f64>() {
                    return Some(value);
                }
            }
        }
        let first_token = after
            .split(|c: char| !c.is_ascii_digit())
            .find(|t| !t.is_empty())?;
        first_token.parse::<f64>().ok()
    }

    fn write_auto_phase_peak_zoom_plot(output_dir: &Path, raw: &Spectrum1D) -> Result<()> {
        let target_len = raw
            .len()
            .checked_mul(2)
            .context("auto-phase zoom target length overflow")?;
        let complex_recipe = ProcessingRecipe1D::new()
            .exponential_apodization(1.0, dwell_time_seconds(raw)?)
            .zero_fill(target_len)
            .fft(FftDirection::Forward)
            .normalize_max_abs();
        let unphased = complex_recipe.apply(raw)?;
        let legacy = auto_phase_correct(
            &unphased,
            AutoPhaseOptions::default()
                .with_cost(AutoPhaseCost::LegacyImagNegArea)
                .with_refine(false),
        )?
        .spectrum;
        let acme_grid = auto_phase_correct(
            &unphased,
            AutoPhaseOptions::default()
                .with_cost(AutoPhaseCost::AcmeEntropy)
                .with_refine(false),
        )?
        .spectrum;
        let acme_refined =
            auto_phase_correct(&unphased, AutoPhaseOptions::default())?.spectrum;
        let pivot_ppm = 2.7_f64;
        let active_region = (1.0_f64, 3.5_f64);
        let acme_pivot = auto_phase_correct(
            &unphased,
            AutoPhaseOptions::default().with_pivot_value(pivot_ppm),
        )?
        .spectrum;
        let acme_active = auto_phase_correct(
            &unphased,
            AutoPhaseOptions::default()
                .with_pivot_value(pivot_ppm)
                .with_active_region(active_region.0, active_region.1),
        )?
        .spectrum;
        let peak_centers = detect_peak_centers(&unphased)?;
        let acme_peak = auto_phase_correct_with_peaks(
            &unphased,
            AutoPhaseOptions::default().with_pivot_value(pivot_ppm),
            &peak_centers,
        )?
        .spectrum;

        let peaks = pick_peaks(
            &acme_refined,
            PeakPickOptions::new()
                .with_min_abs_intensity(0.05)
                .with_min_prominence(0.0)
                .with_polarity(PeakPolarity::Positive),
        )?;
        let mut sorted_peaks = peaks.clone();
        sorted_peaks.sort_by(|a, b| b.intensity.abs().total_cmp(&a.intensity.abs()));
        let mut centers: Vec<f64> = Vec::new();
        for peak in sorted_peaks {
            if centers
                .iter()
                .all(|existing| (existing - peak.x).abs() > 0.25)
            {
                centers.push(peak.x);
                if centers.len() == 9 {
                    break;
                }
            }
        }
        centers.sort_by(|a, b| a.total_cmp(b));
        if centers.is_empty() {
            eprintln!("auto-phase peak zoom: no peaks found");
            return Ok(());
        }

        let columns: usize = 3;
        let rows = centers.len().div_ceil(columns);
        let width = u32::try_from(columns * 480).unwrap_or(1440);
        let height = u32::try_from(rows * 360).unwrap_or(1080);
        let mut figure = subplots(rows, columns, width, height)?;
        let half_window = 0.15_f64;

        for (index, center) in centers.iter().enumerate() {
            let lo = center - half_window;
            let hi = center + half_window;
            let title = format!("{center:.2} ppm");
            let panel = Plot::new()
                .title(&title)
                .xlabel(axis_label(unphased.x.unit))
                .ylabel("intensity")
                .legend_position(LegendPosition::Best)
                .line(
                    &slice_window(&unphased.x.values, lo, hi),
                    &slice_window_y(&unphased.x.values, &unphased.intensities, lo, hi),
                )
                .label("unphased")
                .line(
                    &slice_window(&legacy.x.values, lo, hi),
                    &slice_window_y(&legacy.x.values, &legacy.intensities, lo, hi),
                )
                .label("legacy")
                .line(
                    &slice_window(&acme_grid.x.values, lo, hi),
                    &slice_window_y(&acme_grid.x.values, &acme_grid.intensities, lo, hi),
                )
                .label("ACME")
                .line(
                    &slice_window(&acme_refined.x.values, lo, hi),
                    &slice_window_y(
                        &acme_refined.x.values,
                        &acme_refined.intensities,
                        lo,
                        hi,
                    ),
                )
                .label("ACME+refine")
                .line(
                    &slice_window(&acme_pivot.x.values, lo, hi),
                    &slice_window_y(&acme_pivot.x.values, &acme_pivot.intensities, lo, hi),
                )
                .label("+pivot")
                .line(
                    &slice_window(&acme_active.x.values, lo, hi),
                    &slice_window_y(&acme_active.x.values, &acme_active.intensities, lo, hi),
                )
                .label("+active")
                .line(
                    &slice_window(&acme_peak.x.values, lo, hi),
                    &slice_window_y(&acme_peak.x.values, &acme_peak.intensities, lo, hi),
                )
                .label("peak-warmed");
            figure = figure.subplot_at(index, panel.into())?;
        }

        figure.save(path_to_str(
            &output_dir.join("auto_phase_peak_zoom.png"),
        )?)?;
        Ok(())
    }

    fn slice_window(values: &[f64], lo: f64, hi: f64) -> Vec<f64> {
        values
            .iter()
            .filter(|value| **value >= lo && **value <= hi)
            .copied()
            .collect()
    }

    fn slice_window_y(x: &[f64], y: &[f64], lo: f64, hi: f64) -> Vec<f64> {
        x.iter()
            .zip(y)
            .filter_map(|(xv, yv)| {
                if *xv >= lo && *xv <= hi {
                    Some(*yv)
                } else {
                    None
                }
            })
            .collect()
    }

    fn write_auto_phase_comparison_plot(output_dir: &Path, raw: &Spectrum1D) -> Result<()> {
        let target_len = raw
            .len()
            .checked_mul(2)
            .context("auto-phase comparison target length overflow")?;
        let complex_recipe = ProcessingRecipe1D::new()
            .exponential_apodization(1.0, dwell_time_seconds(raw)?)
            .zero_fill(target_len)
            .fft(FftDirection::Forward)
            .normalize_max_abs();
        let unphased = complex_recipe.apply(raw)?;

        let pivot_ppm = 2.7_f64;
        let active_region = (1.0_f64, 3.5_f64);

        let legacy = AutoPhaseOptions::default()
            .with_strategy(AutoPhaseStrategy::GlobalCost)
            .with_cost(AutoPhaseCost::LegacyImagNegArea)
            .with_refine(false);
        let acme_grid = AutoPhaseOptions::default()
            .with_strategy(AutoPhaseStrategy::GlobalCost)
            .with_cost(AutoPhaseCost::AcmeEntropy)
            .with_refine(false);
        let acme_refined = AutoPhaseOptions::default()
            .with_strategy(AutoPhaseStrategy::GlobalCost);
        let acme_pivot = AutoPhaseOptions::default()
            .with_strategy(AutoPhaseStrategy::GlobalCost)
            .with_pivot_value(pivot_ppm);
        let acme_active = AutoPhaseOptions::default()
            .with_strategy(AutoPhaseStrategy::GlobalCost)
            .with_pivot_value(pivot_ppm)
            .with_active_region(active_region.0, active_region.1);
        let regions = AutoPhaseOptions::default();

        let legacy_result = auto_phase_correct(&unphased, legacy)?;
        let acme_grid_result = auto_phase_correct(&unphased, acme_grid)?;
        let acme_refined_result = auto_phase_correct(&unphased, acme_refined)?;
        let acme_pivot_result = auto_phase_correct(&unphased, acme_pivot)?;
        let acme_active_result = auto_phase_correct(&unphased, acme_active)?;
        let regions_result = auto_phase_correct(&unphased, regions)?;
        let peak_centers = detect_peak_centers(&unphased)?;
        let acme_peak_result = auto_phase_correct_with_peaks(
            &unphased,
            AutoPhaseOptions::default().with_pivot_value(pivot_ppm),
            &peak_centers,
        )?;

        let fmt_label = |stem: &str, r: &rspin_processing::AutoPhaseResult| {
            format!(
                "{stem} ({:.1}\u{00B0}/{:.1}\u{00B0})",
                r.zero_order_deg, r.first_order_deg
            )
        };

        Plot::new()
            .title("Auto-Phase Comparison (Varian/Agilent 1H)")
            .xlabel(axis_label(unphased.x.unit))
            .ylabel("normalized intensity")
            .max_resolution(1600, 1000)
            .legend_position(LegendPosition::Best)
            .line(&unphased.x.values, &unphased.intensities)
            .label("unphased real")
            .line(
                &legacy_result.spectrum.x.values,
                &legacy_result.spectrum.intensities,
            )
            .label(&fmt_label("legacy", &legacy_result))
            .line(
                &acme_grid_result.spectrum.x.values,
                &acme_grid_result.spectrum.intensities,
            )
            .label(&fmt_label("ACME grid", &acme_grid_result))
            .line(
                &acme_refined_result.spectrum.x.values,
                &acme_refined_result.spectrum.intensities,
            )
            .label(&fmt_label("ACME + refine", &acme_refined_result))
            .line(
                &acme_pivot_result.spectrum.x.values,
                &acme_pivot_result.spectrum.intensities,
            )
            .label(&fmt_label("+ pivot 2.7 ppm", &acme_pivot_result))
            .line(
                &acme_active_result.spectrum.x.values,
                &acme_active_result.spectrum.intensities,
            )
            .label(&fmt_label("+ active 1-3.5 ppm", &acme_active_result))
            .line(
                &acme_peak_result.spectrum.x.values,
                &acme_peak_result.spectrum.intensities,
            )
            .label(&fmt_label("peak-warmed", &acme_peak_result))
            .line(
                &regions_result.spectrum.x.values,
                &regions_result.spectrum.intensities,
            )
            .label(&fmt_label("Regions (Zorin 2017)", &regions_result))
            .save(path_to_str(&output_dir.join("auto_phase_comparison.png"))?)?;

        Ok(())
    }

    fn detect_peak_centers(spectrum: &Spectrum1D) -> Result<Vec<f64>> {
        let magnitude = ProcessingRecipe1D::new()
            .magnitude()
            .normalize_max_abs()
            .apply(spectrum)?;
        let peaks = pick_peaks(
            &magnitude,
            PeakPickOptions::new()
                .with_min_abs_intensity(0.05)
                .with_min_prominence(0.0)
                .with_polarity(PeakPolarity::Positive),
        )?;
        let mut sorted = peaks;
        sorted.sort_by(|a, b| b.intensity.abs().total_cmp(&a.intensity.abs()));
        let mut centers: Vec<f64> = Vec::new();
        for peak in sorted {
            if centers
                .iter()
                .all(|existing| (existing - peak.x).abs() > 0.2)
            {
                centers.push(peak.x);
                if centers.len() == 8 {
                    break;
                }
            }
        }
        if centers.is_empty() {
            return Err(anyhow::anyhow!(
                "detect_peak_centers found no peaks in magnitude spectrum"
            ));
        }
        Ok(centers)
    }

    struct VendorContourEntry {
        vendor: &'static str,
        stem: &'static str,
        title: &'static str,
        fixture: &'static str,
    }

    fn write_vendor_contour_entry(
        entry: &VendorContourEntry,
        fixture_root: &Path,
        out_dir: &Path,
    ) -> Result<()> {
        let bundle = load_spectra(fixture_root.join(entry.fixture))
            .with_context(|| format!("failed to load 2D fixture {}", entry.fixture))?;
        let spectra: Vec<&Spectrum2D> = bundle.spectra_2d().collect();
        let Some(spectrum) = spectra.first() else {
            return Ok(());
        };
        let processed = process_2d_for_contour(spectrum)?;
        let png_path = out_dir.join(format!("{}.png", entry.stem));
        write_contour_plot(
            &png_path,
            entry.title,
            axis_label(processed.x.unit),
            axis_label(processed.y.unit),
            &processed,
        )?;
        Ok(())
    }

    fn process_2d_for_contour(spectrum: &Spectrum2D) -> Result<Spectrum2D> {
        if spectrum.x.unit == Unit::Seconds || spectrum.y.unit == Unit::Seconds {
            let dwell_x = axis_step(&spectrum.x.values).unwrap_or(1.0e-6);
            let dwell_y = axis_step(&spectrum.y.values).unwrap_or(1.0e-6);
            spectrum
                .process()
                .exponential_apodization(5.0, 5.0, dwell_x, dwell_y)
                .fft(FftDirection::Forward)
                .absolute_value()
                .normalize_max_abs()
                .finish()
                .context("2D contour FFT pipeline failed")
        } else {
            spectrum
                .process()
                .absolute_value()
                .normalize_max_abs()
                .finish()
                .context("2D contour normalization failed")
        }
    }

    fn axis_step(values: &[f64]) -> Option<f64> {
        if values.len() < 2 {
            return None;
        }
        let step = (values[1] - values[0]).abs();
        if step.is_finite() && step > 0.0 {
            Some(step)
        } else {
            None
        }
    }

    fn write_contour_plot(
        path: &Path,
        title: &str,
        x_label: &str,
        y_label: &str,
        spectrum: &Spectrum2D,
    ) -> Result<()> {
        let levels = autoscale_contour_levels(&spectrum.z);
        Plot::new()
            .title(title)
            .xlabel(x_label)
            .ylabel(y_label)
            .max_resolution(1400, 1200)
            .contour(&spectrum.x.values, &spectrum.y.values, &spectrum.z)
            .level_values(levels)
            .filled(false)
            .save(path_to_str(path)?)?;
        Ok(())
    }

    fn autoscale_contour_levels(z: &[f64]) -> Vec<f64> {
        let max_abs = z
            .iter()
            .copied()
            .map(f64::abs)
            .fold(0.0_f64, f64::max);
        if !(max_abs > 0.0) {
            return vec![0.0];
        }
        let base = max_abs * 0.005;
        let ratio = 1.3_f64;
        let count: usize = 20;
        (0..u32::try_from(count).unwrap_or(0))
            .map(|i| base * ratio.powi(i as i32))
            .filter(|level| *level <= max_abs)
            .collect()
    }

    fn write_vendor_showcase(root: &Path, output_dir: &Path) -> Result<()> {
        fs::create_dir_all(output_dir).with_context(|| {
            format!(
                "failed to create vendor showcase directory {}",
                output_dir.display()
            )
        })?;
        let fixture_root = root.join("crates/rspin-io/testdata");

        let entries: &[VendorShowcaseEntry] = &[
            VendorShowcaseEntry {
                vendor: "bruker",
                stem: "processed_1h_zenodo",
                title: "Bruker processed 1H (Zenodo MIT)",
                fixture: "zenodo_7100132/bruker_without_expno",
            },
            VendorShowcaseEntry {
                vendor: "bruker",
                stem: "raw_1h_myrcene_nmrxiv",
                title: "Bruker raw 1H FID (NMRXiv CC0 Myrcene)",
                fixture: "nmrxiv/cc0/myrcene/bruker_1h_raw",
            },
            VendorShowcaseEntry {
                vendor: "varian",
                stem: "raw_1h_zenodo",
                title: "Varian/Agilent raw 1H FID (Zenodo MIT)",
                fixture: "zenodo_7100132/varian_1h",
            },
            VendorShowcaseEntry {
                vendor: "jeol",
                stem: "myrcene_1h_nmrxiv",
                title: "JEOL 1H (NMRXiv CC0 Myrcene)",
                fixture: "nmrxiv/cc0/myrcene/jeol/myrcene_1h_400mhz.jdf",
            },
            VendorShowcaseEntry {
                vendor: "jeol",
                stem: "myrcene_13c_nmrxiv",
                title: "JEOL 13C (NMRXiv CC0 Myrcene)",
                fixture: "nmrxiv/cc0/myrcene/jeol/myrcene_13c_400mhz.jdf",
            },
            VendorShowcaseEntry {
                vendor: "jeol",
                stem: "rutin_qh_dataverse",
                title: "JEOL 1H (Dataverse CC0 Rutin)",
                fixture: "dataverse/cc0/rutin/jeol/rutin_qhnmr_400mhz.jdf",
            },
            VendorShowcaseEntry {
                vendor: "jcamp",
                stem: "myrcene_1h_nmrxiv",
                title: "JCAMP-DX 1H (NMRXiv CC0 Myrcene)",
                fixture: "nmrxiv/cc0/myrcene/jcamp/myrcene_1h_400mhz_jcamp_dx_6_link.jdx",
            },
            VendorShowcaseEntry {
                vendor: "jcamp",
                stem: "rutin_qh_dataverse",
                title: "JCAMP-DX 1H (Dataverse CC0 Rutin)",
                fixture: "dataverse/cc0/rutin/jcamp/rutin_qh_400mhz.jdx",
            },
            VendorShowcaseEntry {
                vendor: "nmrml",
                stem: "mmbbi_10m12_mit",
                title: "nmrML example (MIT)",
                fixture: "nmrml/mit/MMBBI_10M12-CE01-1a.nmrML",
            },
        ];

        for entry in entries {
            let dir = output_dir.join(entry.vendor);
            fs::create_dir_all(&dir).with_context(|| {
                format!("failed to create vendor dir {}", dir.display())
            })?;
            write_vendor_showcase_entry(entry, &fixture_root, &dir)?;
        }

        let contour_entries: &[VendorContourEntry] = &[
            VendorContourEntry {
                vendor: "bruker",
                stem: "cosy_2d_myrcene_nmrxiv",
                title: "Bruker raw COSY 2D (NMRXiv CC0 Myrcene)",
                fixture: "nmrxiv/cc0/myrcene/bruker_cosy_raw",
            },
            VendorContourEntry {
                vendor: "jeol",
                stem: "hsqc_2d_myrcene_nmrxiv",
                title: "JEOL HSQC 2D (NMRXiv CC0 Myrcene)",
                fixture: "nmrxiv/cc0/myrcene/jeol/myrcene_hsqc_400mhz.jdf",
            },
        ];
        for entry in contour_entries {
            let dir = output_dir.join(entry.vendor);
            fs::create_dir_all(&dir).with_context(|| {
                format!("failed to create vendor dir {}", dir.display())
            })?;
            write_vendor_contour_entry(entry, &fixture_root, &dir)?;
        }
        Ok(())
    }

    struct VendorShowcaseEntry {
        vendor: &'static str,
        stem: &'static str,
        title: &'static str,
        fixture: &'static str,
    }

    fn write_vendor_showcase_entry(
        entry: &VendorShowcaseEntry,
        fixture_root: &Path,
        out_dir: &Path,
    ) -> Result<()> {
        let bundle = load_spectra(fixture_root.join(entry.fixture))
            .with_context(|| format!("failed to load fixture {}", entry.fixture))?;
        let spectra: Vec<&Spectrum1D> = bundle.spectra_1d().collect();
        let Some(spectrum) = spectra.first() else {
            return Ok(());
        };
        let processed = if spectrum.x.unit == Unit::Seconds {
            let target_len = spectrum
                .len()
                .checked_mul(2)
                .context("vendor showcase target length overflow")?;
            ProcessingRecipe1D::new()
                .exponential_apodization(1.0, dwell_time_seconds(spectrum)?)
                .zero_fill(target_len)
                .fft(FftDirection::Forward)
                .magnitude()
                .normalize_max_abs()
                .apply(spectrum)?
        } else {
            ProcessingRecipe1D::new()
                .normalize_max_abs()
                .apply(spectrum)?
        };
        let processed = relabel_hz_to_ppm(processed);

        let png_path = out_dir.join(format!("{}.png", entry.stem));
        write_spectrum_plot(
            &png_path,
            entry.title,
            axis_label(processed.x.unit),
            "normalized intensity",
            &processed.x.values,
            &processed.intensities,
            "spectrum",
        )?;
        Ok(())
    }

    fn relabel_hz_to_ppm(mut spectrum: Spectrum1D) -> Spectrum1D {
        if spectrum.x.unit != Unit::Hertz {
            return spectrum;
        }
        let Some(freq_mhz) = spectrum.metadata.frequency_mhz else {
            return spectrum;
        };
        if !freq_mhz.is_finite() || freq_mhz.abs() <= 0.0 {
            return spectrum;
        }
        spectrum.x.unit = Unit::Ppm;
        spectrum.x.label = "chemical shift".to_owned();
        for value in &mut spectrum.x.values {
            *value /= freq_mhz;
        }
        spectrum
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
        write_auto_phase_plot(output_dir, raw)?;
        Ok(())
    }

    fn write_auto_phase_plot(output_dir: &Path, raw: &Spectrum1D) -> Result<()> {
        let target_len = raw
            .len()
            .checked_mul(2)
            .context("auto-phase target length overflow")?;
        let complex_recipe = ProcessingRecipe1D::new()
            .exponential_apodization(1.0, dwell_time_seconds(raw)?)
            .zero_fill(target_len)
            .fft(FftDirection::Forward)
            .normalize_max_abs();
        let unphased = complex_recipe.apply(raw)?;
        let phased = auto_phase_correct(&unphased, AutoPhaseOptions::default())?.spectrum;

        Plot::new()
            .title("Oracle Varian/Agilent Auto-Phase")
            .xlabel(axis_label(unphased.x.unit))
            .ylabel("normalized intensity")
            .max_resolution(1600, 1000)
            .legend_position(LegendPosition::Best)
            .line(&unphased.x.values, &unphased.intensities)
            .label("unphased real")
            .line(&phased.x.values, &phased.intensities)
            .label("auto-phased real")
            .save(path_to_str(&output_dir.join("processed_auto_phase.png"))?)?;

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
