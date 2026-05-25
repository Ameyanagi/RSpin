# Auto-Phase Variants and Recommendations

Notes from benchmarking the auto-phase variants on the committed Varian
Zenodo MIT `varian_1h` fixture (1H FID, ~32k points after zero-fill).

## Generating the plots

```sh
cargo run --manifest-path examples/ruviz-processing/Cargo.toml --features visualization-ruviz
```

Curated outputs:

- `docs/assets/examples/processed_auto_phase.png` — unphased vs default auto-phase.
- `docs/assets/examples/auto_phase_comparison.png` — full-spectrum overlay of every variant, labeled with the (ph0, ph1) each one converged on.
- `docs/assets/examples/auto_phase_peak_zoom.png` — 3x3 panel zoomed on each detected peak.

## Variant ranking on the Varian fixture

| Rank | Variant | (ph0, ph1) | Cost evals* | Notes |
|---|---|---|---|---|
| 1 | `AutoPhaseOptions::default()` (ACME + refine) | -58.4 deg, -5.1 deg | ~501 | Coarse grid guarantees global basin; Nelder-Mead polishes to sub-degree. |
| 1 | `auto_phase_correct_with_peaks` (peak-warmed) | -59.8 deg, -5.1 deg | ~20 | Same minimum to 0.0 deg, ~25x cheaper. Requires a peak detector that returns 2 or more clean peaks. |
| 2 | default + `.with_pivot_value(2.7)` | -59.8 deg, -5.1 deg | ~501 | Identical answer to peak-warmed but pays the full grid cost. Useful when peak centers aren't available. |
| 3 | ACME grid only (`.with_refine(false)`) | -60.0 deg, 0.0 deg | 481 | Snaps to the nearest grid corner (10 deg / 30 deg). Fine if sub-degree precision doesn't matter. |
| - | + active region 1-3.5 ppm | -61.0 deg, -77.1 deg | ~501 | Off-axis - the windowed cost prefers a steeper ph1 to flatten the multiplet edges. Treat as a region-specific tool, not a default. |
| - | legacy (`AutoPhaseCost::LegacyImagNegArea`) | -70.0 deg, 30.0 deg | 481 | Snaps further from the optimum because the imag+neg cost is fooled by intermediate phases that keep `re >= 0`. |

*Approximate evaluations of the cost function (37 x 13 = 481 grid plus the
Nelder-Mead iterations). Peak-warmed skips the grid entirely.

## Choosing a variant

- **Default for users**: `AutoPhaseOptions::default()`. Zero configuration, safest. Uses ACME entropy + Nelder-Mead refinement.
- **High-throughput** (phasing many rows of a 2D, long serial runs): build peak centers from `rspin_analysis::pick_peaks` on the magnitude spectrum and call `auto_phase_correct_with_peaks`. Equivalent quality with ~25x fewer cost evaluations.
- **Off-center spectra** (heteronuclear with carrier far from any peak): default + `.with_pivot_value(center_ppm)`. The pivot doesn't change the global optimum on a clean centered spectrum, but it stabilizes Nelder-Mead and gives more interpretable (ph0, ph1).
- **Region-specific phasing** (only one multiplet is of interest): default + `.with_active_region(start_ppm, end_ppm)`. Expect the resulting ph1 to differ from the spectrum-wide answer.
- **Avoid for production**: `AutoPhaseCost::LegacyImagNegArea` (kept for back-compat and synthetic 3-point tests).

## Stage history

The current auto-phase implementation evolved through five stages:

1. **Coarse grid + legacy `imag^2 + neg^2` cost** (initial). Resolution-limited; first-order silently disabled by default.
2. **Wider grid** with first-order search enabled.
3. **Nelder-Mead refinement** layered on top of the grid (`refine: true`). Closes the precision gap.
4. **ACME entropy cost** (Chen, Marion, Le Comte, JMR 158 (2002) 164) as the new default scorer.
5. **Pivot in ppm**, **active-region windowing**, and **peak-based warm start**.

Each stage shipped with regression tests on synthetic Lorentzians; the current
suite has 18 auto-phase tests covering grid recovery, refinement scoring,
pivot conversion, active-region masking, peak-based estimation, and the
warm-started hybrid.
