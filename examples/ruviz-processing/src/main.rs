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
    use rspin_core::{Axis, Metadata, Nucleus, Spectrum1D, Spectrum2D, Unit};
    use rspin_io::{
        SpectrumBundle, load_spectra, read_analysis1d_json, read_jeol_jdf_2d_hypercomplex_file,
        read_processing_recipe_1d_json, read_spectrum_bundle_json, read_spectrum1d_csv,
        read_spectrum1d_json, write_analysis1d_csv, write_analysis1d_json,
        write_processing_recipe_1d_json, write_spectrum_bundle_json, write_spectrum1d_csv,
        write_spectrum1d_json,
    };
    use rspin_processing::{
        AutoPhaseCost, AutoPhaseOptions, AutoPhaseStrategy, AutoProcessingOptions, BaselineMethod,
        FftDirection, HyperComplex2DOptions, ProcessSpectrum2D, ProcessingRecipe1D,
        process_hypercomplex_planes_magnitude, apply_subsample_shift,
        auto_phase_correct, auto_phase_correct_with_peaks, convolution_difference_apodization,
        exponential_apodization, fit_baseline, gauss_multiply_bruker_apodization,
        gaussian_apodization, lorentz_to_gauss_apodization, magnitude_spectrum, matched_filter_em,
        process_spectrum_auto, remove_group_delay, traf_apodization, trapezoidal_apodization,
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
        write_apodization_comparison_plot(&root, &output_dir)?;
        write_auto_processing_plot(&root, &output_dir)?;
        let vendor_dir = output_dir.join("vendors");
        write_vendor_showcase(&root, &vendor_dir)?;
        write_jeol_group_delay_comparison(&root, &output_dir)?;
        write_hsqc_phase_sensitive_contours(&root, &output_dir)?;
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

        nmr_plot_base(
            "RSpin 1D Processing Recipe",
            "point",
            "intensity",
            &raw.x.values,
            &[
                &raw.intensities,
                &scaled.intensities,
                &offset.intensities,
                &absolute.intensities,
                &normalized.intensities,
            ],
            raw.x.unit,
        )
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

        nmr_plot_base(
            "RSpin Baseline Correction",
            "chemical shift / ppm",
            "intensity",
            &spectrum.x.values,
            &[&spectrum.intensities, &fit.baseline, &processed.intensities],
            spectrum.x.unit,
        )
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

    struct ApodizationPanels {
        raw: Spectrum1D,
        em: Spectrum1D,
        gm: Spectrum1D,
        l2g: Spectrum1D,
        traf: Spectrum1D,
        gmb: Spectrum1D,
        trap: Spectrum1D,
        conv: Spectrum1D,
        matched: Spectrum1D,
        matched_lb_hz: f64,
        ph0_deg: f64,
        ph1_deg: f64,
        subsample_frac: f64,
    }

    fn build_apodization_panels(
        fid: &Spectrum1D,
        lb_hz: f64,
        gauss_fwhm_hz: f64,
        conv_broad_hz: f64,
    ) -> Result<ApodizationPanels> {
        let group_delay = jeol_group_delay(fid);
        let group_delay_integer = group_delay.trunc();
        let group_delay_frac = group_delay - group_delay_integer;
        let shifted = if group_delay_integer > 0.0 {
            remove_group_delay(fid, group_delay_integer)?
        } else {
            fid.clone()
        };
        let dwell = dwell_time_seconds(&shifted)?;
        let zero_fill_len = shifted
            .len()
            .checked_mul(2)
            .context("apodization comparison target length overflow")?;
        let pivot_fraction = 0.5_f64;

        // Run auto-phase ONCE on a moderately broadened reference so it
        // sees clean Lorentzian peaks, then apply that same (ph0, ph1)
        // to every windowed version. Apodization is a real-valued
        // multiplication and cannot change the FID's phase; phasing
        // each panel independently introduces noise from the auto-phase
        // search converging slightly differently per spectrum.
        let fft_then_subsample = |windowed: &Spectrum1D| -> Result<Spectrum1D> {
            let mut out = ProcessingRecipe1D::new()
                .zero_fill(zero_fill_len)
                .fft(FftDirection::Forward)
                .apply(windowed)?;
            if group_delay_frac.abs() > f64::EPSILON {
                out = apply_subsample_shift(&out, group_delay_frac)?;
            }
            Ok(relabel_hz_to_ppm(out))
        };

        let reference_windowed = exponential_apodization(&shifted, lb_hz, dwell)?;
        let reference_freq = fft_then_subsample(&reference_windowed)?;
        let reference_spectrum = ProcessingRecipe1D::new()
            .normalize_max_abs()
            .apply(&reference_freq)?;
        let reference = auto_phase_correct(
            &reference_spectrum,
            AutoPhaseOptions::default().pivot_fraction(pivot_fraction),
        )?;
        let ph0_deg = reference.zero_order_deg;
        let ph1_deg = reference.first_order_deg;

        let apodise_with_shared_phase = |windowed: Spectrum1D| -> Result<Spectrum1D> {
            let processed = fft_then_subsample(&windowed)?;
            let processed = ProcessingRecipe1D::new()
                .normalize_max_abs()
                .apply(&processed)?;
            // Manual phase with the same (ph0, ph1) recovered above.
            let phased = ProcessingRecipe1D::new()
                .phase(ph0_deg, ph1_deg, pivot_fraction)
                .apply(&processed)?;
            Ok(phased)
        };
        let magnitude_only = |windowed: Spectrum1D| -> Result<Spectrum1D> {
            let processed = fft_then_subsample(&windowed)?;
            let mag = magnitude_spectrum(&processed)?;
            let mag = ProcessingRecipe1D::new()
                .normalize_max_abs()
                .apply(&mag)?;
            Ok(mag)
        };

        let raw = magnitude_only(shifted.clone())?;
        // The reference itself becomes the EM panel — no duplicate work.
        let em = reference.spectrum.clone();
        let gm = apodise_with_shared_phase(gaussian_apodization(&shifted, lb_hz, dwell)?)?;
        let l2g = apodise_with_shared_phase(lorentz_to_gauss_apodization(
            &shifted,
            lb_hz,
            gauss_fwhm_hz,
            0.0,
            dwell,
        )?)?;
        let traf = apodise_with_shared_phase(traf_apodization(&shifted, lb_hz, dwell)?)?;
        let gmb = apodise_with_shared_phase(gauss_multiply_bruker_apodization(
            &shifted, -lb_hz, 0.3, dwell,
        )?)?;
        let trap = apodise_with_shared_phase(trapezoidal_apodization(&shifted, 0.0, 0.7)?)?;
        let conv = apodise_with_shared_phase(convolution_difference_apodization(
            &shifted,
            lb_hz / 3.0,
            conv_broad_hz,
            0.5,
            dwell,
        )?)?;
        let matched_step = matched_filter_em(&shifted)?;
        let matched_lb_hz = matched_step.line_broadening_hz;
        let matched = apodise_with_shared_phase(exponential_apodization(
            &shifted,
            matched_lb_hz,
            matched_step.dwell_time_s,
        )?)?;

        Ok(ApodizationPanels {
            raw,
            em,
            gm,
            l2g,
            traf,
            gmb,
            trap,
            conv,
            matched,
            matched_lb_hz,
            ph0_deg,
            ph1_deg,
            subsample_frac: group_delay_frac,
        })
    }

    fn save_apodization_panel(
        panels: &ApodizationPanels,
        title_prefix: &str,
        lb_hz: f64,
        gauss_fwhm_hz: f64,
        conv_broad_hz: f64,
        zoom: Option<(f64, f64)>,
        path: &Path,
    ) -> Result<()> {
        let zoom_suffix = zoom
            .map(|(lo, hi)| format!(" — zoom {lo:.1}…{hi:.1} ppm"))
            .unwrap_or_default();
        let restrict = |spectrum: &Spectrum1D| -> (Vec<f64>, Vec<f64>) {
            let Some((lo_raw, hi_raw)) = zoom else {
                return (spectrum.x.values.clone(), spectrum.intensities.clone());
            };
            let lo = lo_raw.min(hi_raw);
            let hi = lo_raw.max(hi_raw);
            let mut xs = Vec::new();
            let mut ys = Vec::new();
            for (x, y) in spectrum.x.values.iter().zip(&spectrum.intensities) {
                if *x >= lo && *x <= hi {
                    xs.push(*x);
                    ys.push(*y);
                }
            }
            (xs, ys)
        };
        let mk = |title: String, spectrum: &Spectrum1D, label: &str| -> Plot {
            let (xs, ys) = restrict(spectrum);
            let mut plot = Plot::new()
                .title(&title)
                .xlabel("chemical shift / ppm")
                .ylabel("intensity")
                .max_resolution(900, 600)
                .legend_position(LegendPosition::Best);
            if let Some((x_max, x_min)) = nmr_x_limits(&xs, spectrum.x.unit) {
                plot = plot.xlim(x_max, x_min);
            }
            if let Some((y_min, y_max)) = padded_y_limits(&[&ys]) {
                plot = plot.ylim(y_min, y_max);
            }
            plot.line(&xs, &ys).label(label).into()
        };
        let phase_tag = format!(
            "phased: ph0={:.0}°, ph1={:.0}°, frac={:+.3}",
            panels.ph0_deg, panels.ph1_deg, panels.subsample_frac
        );
        let figure = subplots(3, 3, 2700, 1800)?
            .subplot_at(
                0,
                mk(
                    format!("{title_prefix} — |raw FFT|{zoom_suffix} ({phase_tag})"),
                    &panels.raw,
                    "|spectrum|",
                ),
            )?
            .subplot_at(
                1,
                mk(
                    format!("Exponential (EM, {lb_hz:.1} Hz)"),
                    &panels.em,
                    "real",
                ),
            )?
            .subplot_at(
                2,
                mk(
                    format!("Gaussian (GM, {lb_hz:.1} Hz)"),
                    &panels.gm,
                    "real",
                ),
            )?
            .subplot_at(
                3,
                mk(
                    format!("Lorentz→Gauss (lb={lb_hz:.1}, gb={gauss_fwhm_hz:.1})"),
                    &panels.l2g,
                    "real",
                ),
            )?
            .subplot_at(
                4,
                mk(format!("TRAF (lb={lb_hz:.1} Hz)"), &panels.traf, "real"),
            )?
            .subplot_at(
                5,
                mk(
                    format!("Bruker GMB (lb=-{lb_hz:.1}, gb=0.3)"),
                    &panels.gmb,
                    "real",
                ),
            )?
            .subplot_at(
                6,
                mk("Trapezoidal (fall=0.7)".into(), &panels.trap, "real"),
            )?
            .subplot_at(
                7,
                mk(
                    format!(
                        "Conv-diff (lb={:.1}/{conv_broad_hz:.0}, k=0.5)",
                        lb_hz / 3.0
                    ),
                    &panels.conv,
                    "real",
                ),
            )?
            .subplot_at(
                8,
                mk(
                    format!(
                        "Matched-filter EM (lb={:.2} Hz)",
                        panels.matched_lb_hz
                    ),
                    &panels.matched,
                    "real",
                ),
            )?;
        figure.save(path_to_str(path)?)?;
        Ok(())
    }

    fn write_apodization_comparison_plot(root: &Path, output_dir: &Path) -> Result<()> {
        // JEOL 13C Myrcene — full range + tight zoom on the aromatic cluster.
        let fixture_13c = root.join(
            "crates/rspin-io/testdata/nmrxiv/cc0/myrcene/jeol/myrcene_13c_400mhz.jdf",
        );
        if let Some(fid) = load_first_complex_fid(&fixture_13c)? {
            let lb_13c = 3.0_f64;
            let gauss_13c = 6.0_f64;
            let conv_broad_13c = 20.0_f64;
            let panels = build_apodization_panels(&fid, lb_13c, gauss_13c, conv_broad_13c)?;
            save_apodization_panel(
                &panels,
                "JEOL 13C Myrcene (NMRXiv CC0)",
                lb_13c,
                gauss_13c,
                conv_broad_13c,
                None,
                &output_dir.join("apodization_methods_jeol_13c.png"),
            )?;
            save_apodization_panel(
                &panels,
                "JEOL 13C Myrcene (NMRXiv CC0)",
                lb_13c,
                gauss_13c,
                conv_broad_13c,
                // Tight zoom on a small mid-range cluster so lineshape
                // differences between EM/GM/L2G/TRAF are clearly visible.
                Some((20.0, 70.0)),
                &output_dir.join("apodization_methods_jeol_13c_zoom.png"),
            )?;
        }

        // JEOL 1H Myrcene — same comparison on a 1H spectrum.
        let fixture_1h = root.join(
            "crates/rspin-io/testdata/nmrxiv/cc0/myrcene/jeol/myrcene_1h_400mhz.jdf",
        );
        if let Some(fid) = load_first_complex_fid(&fixture_1h)? {
            let lb_1h = 0.5_f64;
            let gauss_1h = 1.0_f64;
            let conv_broad_1h = 4.0_f64;
            let panels = build_apodization_panels(&fid, lb_1h, gauss_1h, conv_broad_1h)?;
            save_apodization_panel(
                &panels,
                "JEOL 1H Myrcene (NMRXiv CC0)",
                lb_1h,
                gauss_1h,
                conv_broad_1h,
                None,
                &output_dir.join("apodization_methods_jeol_myrcene_1h_full.png"),
            )?;
            save_apodization_panel(
                &panels,
                "JEOL 1H Myrcene (NMRXiv CC0)",
                lb_1h,
                gauss_1h,
                conv_broad_1h,
                // JEOL writes carrier-centered ppm axes; the visible
                // peak cluster sits at ±1 ppm around the carrier.
                Some((-1.5, 1.5)),
                &output_dir.join("apodization_methods_jeol_myrcene_1h_zoom.png"),
            )?;
        }

        Ok(())
    }

    fn write_auto_processing_plot(root: &Path, output_dir: &Path) -> Result<()> {
        let entries: &[(&str, &str, &str)] = &[
            (
                "crates/rspin-io/testdata/nmrxiv/cc0/myrcene/jeol/myrcene_13c_400mhz.jdf",
                "JEOL 13C Myrcene (NMRXiv CC0)",
                "auto_processing_jeol_13c",
            ),
            (
                "crates/rspin-io/testdata/nmrxiv/cc0/myrcene/jeol/myrcene_1h_400mhz.jdf",
                "JEOL 1H Myrcene (NMRXiv CC0)",
                "auto_processing_jeol_1h",
            ),
        ];
        for (fixture_path, title, stem) in entries {
            let fixture = root.join(fixture_path);
            let Some(fid) = load_first_complex_fid(&fixture)? else {
                continue;
            };
            let opts = AutoProcessingOptions {
                // Let the orchestrator pick the JEOL group delay from
                // metadata; the FIR cascade formula it now uses is
                // more accurate than the example's local heuristic.
                subtract_baseline: false,
                ..AutoProcessingOptions::default()
            };
            let processed = process_spectrum_auto(&fid, &opts)?;
            let processed = relabel_hz_to_ppm(processed);
            let normalized = ProcessingRecipe1D::new()
                .normalize_max_abs()
                .apply(&processed)?;

            // Reference: raw |FFT| of the integer-shifted FID (no
            // apodization, no LP, no phase). Use the example-local
            // group-delay helper for parity with the apodization
            // comparison PNG; process_spectrum_auto uses the cascade
            // formula internally and may pick a different integer.
            let group_delay = jeol_group_delay(&fid);
            let integer_shift = group_delay.trunc().max(0.0);
            let shifted = if integer_shift > 0.0 {
                remove_group_delay(&fid, integer_shift)?
            } else {
                fid.clone()
            };
            let raw_magnitude = ProcessingRecipe1D::new()
                .zero_fill(shifted.len() * 2)
                .fft(FftDirection::Forward)
                .apply(&shifted)?;
            let raw_magnitude = magnitude_spectrum(&raw_magnitude)?;
            let raw_magnitude = ProcessingRecipe1D::new()
                .normalize_max_abs()
                .apply(&raw_magnitude)?;
            let raw_magnitude = relabel_hz_to_ppm(raw_magnitude);

            nmr_plot_base(
                &format!("{title} — process_spectrum_auto"),
                "chemical shift / ppm",
                "normalized intensity",
                &normalized.x.values,
                &[&raw_magnitude.intensities, &normalized.intensities],
                normalized.x.unit,
            )
            .line(&raw_magnitude.x.values, &raw_magnitude.intensities)
            .label("|raw FFT| (no apodization)")
            .line(&normalized.x.values, &normalized.intensities)
            .label("process_spectrum_auto")
            .save(path_to_str(&output_dir.join(format!("{stem}.png")))?)?;
        }
        Ok(())
    }

    fn load_first_complex_fid(fixture: &Path) -> Result<Option<Spectrum1D>> {
        let bundle = load_spectra(fixture)?;
        let Some(fid) = bundle.spectra_1d().next() else {
            return Ok(None);
        };
        if fid.x.unit != Unit::Seconds || fid.imaginary.is_none() {
            return Ok(None);
        }
        Ok(Some(fid.clone()))
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
            let mut plot = Plot::new()
                .title(&panel_title)
                .xlabel("chemical shift / ppm")
                .ylabel("intensity")
                .max_resolution(900, 600)
                .legend_position(LegendPosition::Best);
            if let Some((x_max, x_min)) = nmr_x_limits(xs, magnitude.x.unit) {
                plot = plot.xlim(x_max, x_min);
            }
            if let Some((y_min, y_max)) = padded_y_limits(&[ys]) {
                plot = plot.ylim(y_min, y_max);
            }
            plot.line(xs, ys).label(label).into()
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

            nmr_plot_base(
                title,
                "chemical shift / ppm",
                "normalized intensity",
                &magnitude.x.values,
                &[
                    &magnitude.intensities,
                    &legacy.spectrum.intensities,
                    &acme.spectrum.intensities,
                    &regions.spectrum.intensities,
                ],
                magnitude.x.unit,
            )
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

    fn jeol_phase_with_opts(
        spectrum: &Spectrum1D,
        shift: f64,
        options: AutoPhaseOptions,
    ) -> Result<rspin_processing::AutoPhaseResult> {
        let shifted = if shift > 0.0 {
            remove_group_delay(spectrum, shift)?
        } else {
            spectrum.clone()
        };
        let prepared = if shifted.x.unit == Unit::Seconds {
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
        let prepared = relabel_hz_to_ppm(prepared);
        Ok(auto_phase_correct(&prepared, options)?)
    }

    fn jeol_magnitude(
        spectrum: &Spectrum1D,
        shift: f64,
    ) -> Result<(Spectrum1D, rspin_processing::AutoPhaseResult)> {
        let shifted = if shift > 0.0 {
            remove_group_delay(spectrum, shift)?
        } else {
            spectrum.clone()
        };
        let magnitude = if shifted.x.unit == Unit::Seconds {
            let target_len = shifted
                .len()
                .checked_mul(2)
                .context("JEOL magnitude target length overflow")?;
            ProcessingRecipe1D::new()
                .exponential_apodization(1.0, dwell_time_seconds(&shifted)?)
                .zero_fill(target_len)
                .fft(FftDirection::Forward)
                .magnitude()
                .normalize_max_abs()
                .apply(&shifted)?
        } else {
            ProcessingRecipe1D::new()
                .magnitude()
                .normalize_max_abs()
                .apply(&shifted)?
        };
        let magnitude = relabel_hz_to_ppm(magnitude);
        // Return a dummy auto-phase result so the API matches jeol_phase_with_shift.
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

    /// JEOL Delta digital-filter group delay, sourced from the
    /// library's `group_delay_from_metadata` which prefers the FIR
    /// cascade formula but accepts other vendors too. Kept as a
    /// one-line shim so callsites read naturally.
    fn jeol_group_delay(spectrum: &Spectrum1D) -> f64 {
        rspin_processing::group_delay_from_metadata(&spectrum.metadata)
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
            let mut panel = Plot::new()
                .title(&title)
                .xlabel(axis_label(unphased.x.unit))
                .ylabel("intensity")
                .legend_position(LegendPosition::Best);
            if matches!(unphased.x.unit, Unit::Ppm | Unit::Hertz) {
                panel = panel.xlim(hi, lo);
            }
            let panel = panel
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

        nmr_plot_base(
            "Auto-Phase Comparison (Varian/Agilent 1H)",
            axis_label(unphased.x.unit),
            "normalized intensity",
            &unphased.x.values,
            &[
                &unphased.intensities,
                &legacy_result.spectrum.intensities,
                &acme_grid_result.spectrum.intensities,
                &acme_refined_result.spectrum.intensities,
                &acme_pivot_result.spectrum.intensities,
                &acme_active_result.spectrum.intensities,
                &acme_peak_result.spectrum.intensities,
                &regions_result.spectrum.intensities,
            ],
            unphased.x.unit,
        )
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
        let mut plot = Plot::new()
            .title(title)
            .xlabel(x_label)
            .ylabel(y_label)
            .max_resolution(1400, 1200);
        if let Some((x_max, x_min)) = nmr_x_limits(&spectrum.x.values, spectrum.x.unit) {
            plot = plot.xlim(x_max, x_min);
        }
        if let Some((y_max, y_min)) = nmr_x_limits(&spectrum.y.values, spectrum.y.unit) {
            plot = plot.ylim(y_max, y_min);
        }
        plot.contour(&spectrum.x.values, &spectrum.y.values, &spectrum.z)
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
        // Noise-aware base level: in a sparse 2D spectrum most cells are noise,
        // so the median magnitude is a robust noise-floor proxy. Start contours
        // well above it to clip the t1-noise floor rather than drawing it.
        let mut sorted: Vec<f64> = z.iter().map(|value| value.abs()).collect();
        sorted.sort_by(f64::total_cmp);
        let median = sorted.get(sorted.len() / 2).copied().unwrap_or(0.0);
        let base = (median * 8.0).max(max_abs * 0.03);
        let ratio = 1.3_f64;
        let count: usize = 20;
        (0..u32::try_from(count).unwrap_or(0))
            .map(|i| base * ratio.powi(i as i32))
            .filter(|level| *level <= max_abs)
            .collect()
    }

    /// Generates a single comparison PNG for the Eucalyptol 13C JEOL
    /// fixture showing process_spectrum_auto output at the FIR-cascade
    /// group-delay value (~19.66 samples, library default) overlaid
    /// against the empirically-swept optimum (~16.46 samples).
    fn write_jeol_group_delay_comparison(root: &Path, output_dir: &Path) -> Result<()> {
        let fixture = root
            .join("crates/rspin-io/testdata/nmrxiv/cc0/eucalyptol/jeol/eucalyptol_13cnmr_400mhz.jdf");
        let bundle = load_spectra(&fixture)
            .with_context(|| format!("failed to load fixture {}", fixture.display()))?;
        let fid = bundle
            .spectra_1d()
            .next()
            .context("eucalyptol 13C fixture has no 1D spectrum")?;

        let run = |gd: Option<f64>| -> Result<Spectrum1D> {
            let opts = AutoProcessingOptions {
                group_delay_samples: gd,
                subtract_baseline: false,
                ..AutoProcessingOptions::default()
            };
            let processed = process_spectrum_auto(fid, &opts)?;
            let normalized = ProcessingRecipe1D::new()
                .normalize_max_abs()
                .apply(&processed)?;
            Ok(normalized)
        };

        let cascade = run(None)?;
        let empirical = run(Some(16.46))?;
        // Demonstrate the opt-in auto_group_delay_sweep: lets the
        // orchestrator pick the best value automatically without the
        // caller knowing the empirical optimum.
        let sweep_opts = AutoProcessingOptions {
            subtract_baseline: false,
            auto_group_delay_sweep: Some(rspin_processing::GroupDelaySweepOptions {
                delta_samples: 5.0,
                step_samples: 0.2,
            }),
            ..AutoProcessingOptions::default()
        };
        let auto_swept = process_spectrum_auto(fid, &sweep_opts)?;
        let auto_swept = ProcessingRecipe1D::new()
            .normalize_max_abs()
            .apply(&auto_swept)?;

        let title = "JEOL 13C — Eucalyptol — group-delay cascade vs empirical vs auto-sweep";
        let png_path = output_dir.join("auto_processing_jeol_eucalyptol_13c_group_delay.png");
        nmr_plot_base(
            title,
            "chemical shift / ppm",
            "normalized intensity",
            &cascade.x.values,
            &[
                &cascade.intensities,
                &empirical.intensities,
                &auto_swept.intensities,
            ],
            cascade.x.unit,
        )
        .line(&cascade.x.values, &cascade.intensities)
        .label("cascade (19.66, library default)")
        .line(&empirical.x.values, &empirical.intensities)
        .label("empirical override (16.46)")
        .line(&auto_swept.x.values, &auto_swept.intensities)
        .label("auto_group_delay_sweep (Δ±5, step 0.2)")
        .save(path_to_str(&png_path)?)?;
        Ok(())
    }

    /// Hypercomplex-modulus HSQC contours via the four-plane JEOL path
    /// (`read_jeol_jdf_2d_hypercomplex_file` → `process_hypercomplex_planes_magnitude`).
    ///
    /// Uses higher-resolution (256 t1 increment) HSQC fixtures from the cheminfo
    /// jeol-data-test submodule, which give well-resolved cross-peaks; rendered
    /// only when the submodule is initialized. (The committed nmrxiv eucalyptol
    /// /myrcene HSQC have only 32 t1 increments, so they are t1-noise dominated
    /// and not used for the showcase.)
    fn write_hsqc_phase_sensitive_contours(root: &Path, output_dir: &Path) -> Result<()> {
        let entries = [
            (
                "external-testdata/cheminfo/jeol-data-test/data/Rutin_3080ug200uL_DMSOd6_HSQC_400MHz_Jeol.jdf",
                "hsqc_rutin_hypercomplex_modulus_contour.png",
                "JEOL HSQC — Rutin — hypercomplex modulus",
            ),
            (
                "external-testdata/cheminfo/jeol-data-test/data/EC=8C_5m200u_MeOD_bzhou21_20190228__HSQC-1-1.jdf",
                "hsqc_ec_hypercomplex_modulus_contour.png",
                "JEOL HSQC — EC — hypercomplex modulus",
            ),
        ];
        let options = HyperComplex2DOptions::default().with_indirect_zero_fill(512);
        for (fixture, stem, title) in entries {
            let path = root.join(fixture);
            if !path.exists() {
                // Submodule fixtures are optional; skip when not initialized.
                continue;
            }
            let hc = read_jeol_jdf_2d_hypercomplex_file(&path)
                .with_context(|| format!("failed to load HSQC hypercomplex {fixture}"))?;
            // Hypercomplex-modulus display (sqrt of all four quadrants): a
            // phase-insensitive 2D magnitude, so cross-peaks read cleanly
            // without a perfect direct/indirect phase.
            let display = process_hypercomplex_planes_magnitude(&hc, &options)
                .context("phase-sensitive HSQC processing failed")?;
            write_contour_plot(
                &output_dir.join(stem),
                title,
                axis_label(display.x.unit),
                axis_label(display.y.unit),
                &display,
            )?;
        }
        Ok(())
    }

    fn write_vendor_showcase(root: &Path, output_dir: &Path) -> Result<()> {
        fs::create_dir_all(output_dir).with_context(|| {
            format!(
                "failed to create vendor showcase directory {}",
                output_dir.display()
            )
        })?;
        let fixture_root = root.join("crates/rspin-io/testdata");

        // Vendor showcase matrix: every vendor gets ¹H + ¹³C entries
        // (and 2D below) where a permissively-licensed fixture exists.
        let entries: &[VendorShowcaseEntry] = &[
            // ── Bruker ─────────────────────────────────────────
            VendorShowcaseEntry {
                vendor: "bruker",
                stem: "myrcene_1h_raw_nmrxiv",
                title: "Bruker 1H raw FID — Myrcene (NMRXiv CC0)",
                fixture: "nmrxiv/cc0/myrcene/bruker_1h_raw",
            },
            VendorShowcaseEntry {
                vendor: "bruker",
                stem: "zenodo_processed_1h",
                title: "Bruker 1H processed — Zenodo MIT",
                fixture: "zenodo_7100132/bruker_without_expno",
            },
            // ── Varian / Agilent ───────────────────────────────
            VendorShowcaseEntry {
                vendor: "varian",
                stem: "zenodo_1h_raw",
                title: "Varian/Agilent 1H raw FID — Zenodo MIT",
                fixture: "zenodo_7100132/varian_1h",
            },
            // ── JEOL Delta ─────────────────────────────────────
            VendorShowcaseEntry {
                vendor: "jeol",
                stem: "myrcene_1h_nmrxiv",
                title: "JEOL 1H — Myrcene (NMRXiv CC0)",
                fixture: "nmrxiv/cc0/myrcene/jeol/myrcene_1h_400mhz.jdf",
            },
            VendorShowcaseEntry {
                vendor: "jeol",
                stem: "myrcene_13c_nmrxiv",
                title: "JEOL 13C — Myrcene (NMRXiv CC0)",
                fixture: "nmrxiv/cc0/myrcene/jeol/myrcene_13c_400mhz.jdf",
            },
            VendorShowcaseEntry {
                vendor: "jeol",
                stem: "rutin_qh_dataverse",
                title: "JEOL 1H — Rutin (Dataverse CC0)",
                fixture: "dataverse/cc0/rutin/jeol/rutin_qhnmr_400mhz.jdf",
            },
            VendorShowcaseEntry {
                vendor: "jeol",
                stem: "rutin_13c_dataverse",
                title: "JEOL 13C — Rutin (Dataverse CC0)",
                fixture: "dataverse/cc0/rutin/jeol/rutin_13cnmr_400mhz.jdf",
            },
            VendorShowcaseEntry {
                vendor: "jeol",
                stem: "eucalyptol_qh_nmrxiv",
                title: "JEOL 1H — Eucalyptol (NMRXiv CC0)",
                fixture: "nmrxiv/cc0/eucalyptol/jeol/eucalyptol_qhnmr_400mhz.jdf",
            },
            VendorShowcaseEntry {
                vendor: "jeol",
                stem: "eucalyptol_13c_nmrxiv",
                title: "JEOL 13C — Eucalyptol (NMRXiv CC0)",
                fixture: "nmrxiv/cc0/eucalyptol/jeol/eucalyptol_13cnmr_400mhz.jdf",
            },
            // ── JCAMP-DX ───────────────────────────────────────
            VendorShowcaseEntry {
                vendor: "jcamp",
                stem: "myrcene_1h_nmrxiv",
                title: "JCAMP-DX 1H — Myrcene (NMRXiv CC0)",
                fixture: "nmrxiv/cc0/myrcene/jcamp/myrcene_1h_400mhz_jcamp_dx_6_link.jdx",
            },
            VendorShowcaseEntry {
                vendor: "jcamp",
                stem: "myrcene_13c_nmrxiv",
                title: "JCAMP-DX 13C — Myrcene (NMRXiv CC0)",
                fixture: "nmrxiv/cc0/myrcene/jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx",
            },
            VendorShowcaseEntry {
                vendor: "jcamp",
                stem: "rutin_qh_dataverse",
                title: "JCAMP-DX 1H — Rutin (Dataverse CC0)",
                fixture: "dataverse/cc0/rutin/jcamp/rutin_qh_400mhz.jdx",
            },
            VendorShowcaseEntry {
                vendor: "jcamp",
                stem: "rutin_13c_dataverse",
                title: "JCAMP-DX 13C — Rutin (Dataverse CC0)",
                fixture: "dataverse/cc0/rutin/jcamp/rutin_13c_400mhz.jdx",
            },
            VendorShowcaseEntry {
                vendor: "jcamp",
                stem: "eucalyptol_qh_nmrxiv",
                title: "JCAMP-DX 1H — Eucalyptol (NMRXiv CC0)",
                fixture: "nmrxiv/cc0/eucalyptol/jcamp/eucalyptol_qh_400mhz_jcamp_dx_6_link.jdx",
            },
            VendorShowcaseEntry {
                vendor: "jcamp",
                stem: "eucalyptol_13c_nmrxiv",
                title: "JCAMP-DX 13C — Eucalyptol (NMRXiv CC0)",
                fixture: "nmrxiv/cc0/eucalyptol/jcamp/eucalyptol_13c_400mhz_jcamp_dx_6_link.jdx",
            },
            // ── nmrML ──────────────────────────────────────────
            VendorShowcaseEntry {
                vendor: "nmrml",
                stem: "mmbbi_10m12_mit",
                title: "nmrML 1H — MMBBI 10M12 (MIT)",
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
                stem: "myrcene_cosy_2d_nmrxiv",
                title: "Bruker COSY 2D — Myrcene (NMRXiv CC0)",
                fixture: "nmrxiv/cc0/myrcene/bruker_cosy_raw",
            },
            VendorContourEntry {
                vendor: "jeol",
                stem: "myrcene_hsqc_2d_nmrxiv",
                title: "JEOL HSQC 2D — Myrcene (NMRXiv CC0)",
                fixture: "nmrxiv/cc0/myrcene/jeol/myrcene_hsqc_400mhz.jdf",
            },
            VendorContourEntry {
                vendor: "jeol",
                stem: "eucalyptol_hsqc_2d_nmrxiv",
                title: "JEOL HSQC 2D — Eucalyptol (NMRXiv CC0)",
                fixture: "nmrxiv/cc0/eucalyptol/jeol/eucalyptol_hsqc_400mhz.jdf",
            },
            VendorContourEntry {
                vendor: "jcamp",
                stem: "eucalyptol_hsqc_2d_nmrxiv",
                title: "JCAMP-DX HSQC 2D — Eucalyptol (NMRXiv CC0)",
                fixture: "nmrxiv/cc0/eucalyptol/jcamp/eucalyptol_hsqc_400mhz_jcamp_dx_6_link.jdx",
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
            // Polynomial refine (ph2/ph3) is intentionally OFF here:
            // it overfits the JEOL group-delay residual on Myrcene 13C,
            // producing a 180°-flipped CDCl3 solvent peak even when
            // the sample resonances are well-phased.
            let opts = AutoProcessingOptions {
                subtract_baseline: false,
                ..AutoProcessingOptions::default()
            };
            let auto = process_spectrum_auto(spectrum, &opts)?;
            ProcessingRecipe1D::new()
                .normalize_max_abs()
                .apply(&auto)?
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

        // Companion zoom plot, clipped to the signal-bearing window.
        // For 1H/13C spectra in ppm units the window is always extended
        // to include 0 ppm so the absence of TMS / reference signal is
        // visible too.
        let include_zero = processed.x.unit == Unit::Ppm
            && matches!(
                processed.metadata.nucleus,
                Some(Nucleus::Hydrogen1) | Some(Nucleus::Carbon13)
            );
        let window = signal_window(&processed.x.values, &processed.intensities, 0.08)
            .map(|(lo, hi)| {
                if include_zero {
                    (lo.min(-0.2), hi.max(0.2))
                } else {
                    (lo, hi)
                }
            });
        if let Some((lo, hi)) = window {
            let mut xs = Vec::new();
            let mut ys = Vec::new();
            for (x, y) in processed.x.values.iter().zip(&processed.intensities) {
                if *x >= lo && *x <= hi {
                    xs.push(*x);
                    ys.push(*y);
                }
            }
            if xs.len() >= 4 {
                let zoom_path = out_dir.join(format!("{}_zoom.png", entry.stem));
                write_spectrum_plot(
                    &zoom_path,
                    &format!("{} — zoom", entry.title),
                    axis_label(processed.x.unit),
                    "normalized intensity",
                    &xs,
                    &ys,
                    "spectrum (signal window)",
                )?;
            }
        }
        Ok(())
    }

    /// Auto-detects the signal-bearing window on a normalized spectrum
    /// by finding the leftmost/rightmost x where `|y|` first exceeds
    /// `threshold_fraction × peak`, then padding 10 % on each side.
    /// Returns `None` for spectra with no clear signal.
    fn signal_window(x: &[f64], y: &[f64], threshold_fraction: f64) -> Option<(f64, f64)> {
        let peak = y
            .iter()
            .copied()
            .fold(0.0_f64, |acc, value| acc.max(value.abs()));
        if !peak.is_finite() || peak <= 0.0 {
            return None;
        }
        let threshold = peak * threshold_fraction;
        let mut x_lo = f64::INFINITY;
        let mut x_hi = f64::NEG_INFINITY;
        for (xi, yi) in x.iter().zip(y) {
            if yi.abs() >= threshold && xi.is_finite() {
                if *xi < x_lo {
                    x_lo = *xi;
                }
                if *xi > x_hi {
                    x_hi = *xi;
                }
            }
        }
        if !x_lo.is_finite() || !x_hi.is_finite() || x_lo >= x_hi {
            return None;
        }
        let pad = 0.10 * (x_hi - x_lo);
        Some((x_lo - pad, x_hi + pad))
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

        nmr_plot_base(
            "Oracle Varian/Agilent Auto-Phase",
            axis_label(unphased.x.unit),
            "normalized intensity",
            &unphased.x.values,
            &[&unphased.intensities, &phased.intensities],
            unphased.x.unit,
        )
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

        let mut base = Plot::new()
            .title(title)
            .xlabel(axis_label(spectrum.x.unit))
            .ylabel("intensity")
            .max_resolution(1600, 1000)
            .legend_position(LegendPosition::Best);
        if let Some((x_max, x_min)) = nmr_x_limits(&spectrum.x.values, spectrum.x.unit) {
            base = base.xlim(x_max, x_min);
        }
        if let Some((y_min, y_max)) = padded_y_limits(&[&spectrum.intensities]) {
            base = base.ylim(y_min, y_max);
        }
        let mut plot = base
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
        let mut plot = Plot::new()
            .title(title)
            .xlabel(x_label)
            .ylabel(y_label)
            .max_resolution(1600, 1000)
            .legend_position(LegendPosition::Best);
        let unit = unit_from_label(x_label);
        if let Some((x_max, x_min)) = nmr_x_limits(x, unit) {
            plot = plot.xlim(x_max, x_min);
        }
        if let Some((y_min, y_max)) = padded_y_limits(&[y]) {
            plot = plot.ylim(y_min, y_max);
        }
        plot.line(&x, &y)
            .label(series_label)
            .save(path_to_str(path)?)?;
        Ok(())
    }

    /// X-axis limits in NMR convention (high → low) for Ppm/Hz units;
    /// returns `(x_max, x_min)` so callers can pass them straight to
    /// `Plot::xlim` to invert the axis. Returns `None` for time-domain
    /// or point-domain plots where NMR convention does not apply.
    fn nmr_x_limits(x: &[f64], unit: Unit) -> Option<(f64, f64)> {
        if !matches!(unit, Unit::Ppm | Unit::Hertz) {
            return None;
        }
        let mut lo = f64::INFINITY;
        let mut hi = f64::NEG_INFINITY;
        for value in x.iter().copied().filter(|value| value.is_finite()) {
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
        Some((hi, lo))
    }

    /// Y-axis limits padded below the minimum and above the maximum so
    /// small negative baseline excursions remain visible. Accepts any
    /// number of overlaid traces.
    fn padded_y_limits(series: &[&[f64]]) -> Option<(f64, f64)> {
        let mut lo = f64::INFINITY;
        let mut hi = f64::NEG_INFINITY;
        for trace in series {
            for value in trace.iter().copied().filter(|value| value.is_finite()) {
                if value < lo {
                    lo = value;
                }
                if value > hi {
                    hi = value;
                }
            }
        }
        if !lo.is_finite() || !hi.is_finite() || lo >= hi {
            return None;
        }
        let span = hi - lo;
        Some((lo - 0.05 * span, hi + 0.10 * span))
    }

    /// Builds the standard NMR-styled `Plot` used by every panel in
    /// this example: 1600×1000 resolution, "Best" legend position,
    /// x-axis flipped high→low for Ppm/Hz units, and y-axis padded so
    /// negative baseline excursions stay visible.
    fn nmr_plot_base(
        title: &str,
        x_label: &str,
        y_label: &str,
        x: &[f64],
        y_traces: &[&[f64]],
        unit: Unit,
    ) -> Plot {
        let mut plot = Plot::new()
            .title(title)
            .xlabel(x_label)
            .ylabel(y_label)
            .max_resolution(1600, 1000)
            .legend_position(LegendPosition::Best);
        if let Some((x_max, x_min)) = nmr_x_limits(x, unit) {
            plot = plot.xlim(x_max, x_min);
        }
        if let Some((y_min, y_max)) = padded_y_limits(y_traces) {
            plot = plot.ylim(y_min, y_max);
        }
        plot
    }

    /// Best-effort mapping from a free-form axis label back to its
    /// [`Unit`]; used by `write_spectrum_plot` to decide whether to
    /// flip the x-axis. Falls back to [`Unit::Arbitrary`] when the
    /// label is unrecognised.
    fn unit_from_label(label: &str) -> Unit {
        let lower = label.to_ascii_lowercase();
        if lower.contains("ppm") {
            Unit::Ppm
        } else if lower.contains("hz") || lower.contains("hertz") || lower.contains("frequency") {
            Unit::Hertz
        } else if lower.contains("time") || lower.contains(" s") {
            Unit::Seconds
        } else {
            Unit::Arbitrary
        }
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
