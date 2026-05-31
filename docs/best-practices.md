# Best practices for NMR processing in RSpin

A short, opinionated guide to the recommended order, defaults, and
trade-offs when building a processing pipeline with `rspin-processing`.

## Pipeline order

For a typical complex FID:

1. **Vendor-specific corrections** — `remove_group_delay` for Bruker /
   JEOL digital filters; SI-unit axis scaling is handled by the
   readers.
2. **Linear prediction** (optional) — backward LP to repair broken
   first points, then forward LP to extend the tail. *RSpin does not
   ship LP yet — coming in a follow-up.* When present, LP goes before
   apodization.
3. **Apodization** — pick one window (see below).
4. **First-point scaling** — multiply `intensities[0]` by 0.5 to remove
   the DC offset that survives a one-sided FT. Not yet a separate step
   in RSpin; users typically zero-fill straight after apodization.
5. **Zero-fill** — `recipe.zero_fill(2 * raw.len())` is a common
   default (one round of doubling).
6. **FFT** — `recipe.fft(FftDirection::Forward)`. The output axis is
   relabeled to Hz or ppm automatically.
7. **Phase correction** — `recipe.auto_phase()` (default = Regions
   algorithm) or `recipe.phase(ph0, ph1, pivot)` if you know the
   values.
8. **Baseline correction** — `recipe.subtract_baseline()` if needed.

## Choosing an apodization window

`rspin-processing` ships eight windows. The right one depends on what
you want to optimise:

### SNR ("see weak peaks")

| Rank | Window | When |
|------|--------|------|
| 1 | `matched_filter_em(&fid)?` | First choice — auto-picks the SNR-optimal LB |
| 2 | `TrafApodization` (lb ≈ natural LW) | Better when the spectrum has both broad and narrow peaks |
| 3 | `ExponentialApodization` (lb = natural LW) | Same theory as matched filter, manual control |

### Resolution ("separate overlapping peaks")

| Rank | Window | When |
|------|--------|------|
| 1 | `LorentzToGaussApodization` (lb=natural, gb=2×lb) | The standard 1H resolution-enhancement window |
| 2 | `GaussMultiplyBrukerApodization` | Same math; choose this if you are cross-checking against Bruker `procs` |

### HSQC / biomolecular 2D F1

`SineBellApodization::cosine_squared()` — the nmrPipe biomolecular
template default. Pair with `SineBellApodization2D::new(...)` for
explicit X/Y angles.

### When the FID has had forward linear prediction applied

`TrapezoidalApodization::new(0.05, 0.85)` — ramp up smoothly, hold for
the real samples, ramp down across the LP-extrapolated tail to
suppress any extrapolation artefacts.

### Paramagnetic / solid-state spectra with broad components

`ConvolutionDifferenceApodization` (narrow LB1 ≈ 1 Hz, broad LB2 ≈
20 Hz, mixing k ≈ 0.5) — actively *subtracts* the broad background
rather than just weighting it down.

## The matched filter, in detail

The matched filter is the SNR-optimal window when the signal is of
known shape. For an FID

```
s(t) = exp(-π·LB·t) · cos(2π·ν·t) + n(t)
```

where `LB` is the natural linewidth and `n(t)` is white noise,
multiplying by `w(t) = s(t)` before FFT maximises the peak SNR in the
resulting spectrum. For NMR that reduces to

```
w(t) = exp(-π·LB·t)
```

which is exponential apodization with `LB` set to the **natural
linewidth**. Two consequences:

- **Peak SNR roughly doubles** vs. no apodization.
- **Linewidths broaden by ×2** in the frequency domain (you decayed
  the signal once physically and once with the window).

You always pay that resolution penalty for the SNR gain. If you cannot
afford it — e.g. 1H spectra with many overlapping multiplets — use a
gentler LB (≪ natural linewidth) and accept lower SNR, or switch to
Lorentz-to-Gauss for active resolution enhancement.

### `matched_filter_em` in RSpin

`matched_filter_em(&fid)` automates the LB selection:

1. Forward-FFT the FID with no window.
2. Take the magnitude spectrum.
3. Find the strongest peak.
4. Measure its FWHM in Hz.
5. Divide by √3 (the magnitude-mode lineshape has FWHM = √3·LB while
   the absorption-mode lineshape has FWHM = LB).
6. Return `ExponentialApodization { line_broadening_hz: LB, dwell_time_s }`.

The returned step is the matched filter for that spectrum. Apply it
**before** zero-fill and FFT:

```rust
use rspin_processing::{matched_filter_em, ProcessingRecipe1D, FftDirection};

let em = matched_filter_em(&fid)?;
let processed = ProcessingRecipe1D::new()
    .exponential_apodization(em.line_broadening_hz, em.dwell_time_s)
    .zero_fill(fid.len() * 2)
    .fft(FftDirection::Forward)
    .auto_phase()
    .apply(&fid)?;
```

The helper works on complex (quadrature) FIDs whose x-axis is
`Unit::Seconds` with uniform dwell. It will return an error on
frequency-domain inputs.

### When *not* to use the matched filter

- **Spectra with very different natural linewidths** (e.g. 1H with
  fast-relaxing methyls alongside slow-relaxing aromatics). The
  matched filter optimises for the *strongest* peak's width; weaker
  peaks of different width are misweighted. Pick a manual `LB` or
  apply per-region phasing instead.
- **Resolution-critical work** (couplings, dispersion analysis). The
  ×2 linewidth penalty is exactly the wrong direction. Use
  `LorentzToGaussApodization` instead.
- **Spectra where Bruker `procs` already includes an `LB` value** —
  apodising again would double-broaden. Either skip apodization
  entirely or use `GaussMultiplyBrukerApodization` with `procs`-style
  parameters so you can see what is happening.

## One-shot processing: `process_spectrum_auto`

For the common case where you just want a phased, baseline-corrected
spectrum from a vendor FID, skip the recipe and call
`process_spectrum_auto(&fid, &options)`. It chains, in order:

1. Group-delay correction (integer drop + zero-pad, fractional residual
   recorded for the post-FFT pass).
2. Optional backward linear prediction.
3. Nucleus-aware exponential apodization (LB from `NucleusLbDefaults`).
4. First-point scaling `s[0] *= 0.5`.
5. Zero-fill to the next power of two ≥ `2 × len`.
6. Forward FFT.
7. Fractional sub-sample shift (Fourier phase ramp for the residual
   from step 1).
8. Auto-phase (Regions or GlobalCost, configurable).
9. Optional polynomial (ph2/ph3) refinement.
10. Baseline subtraction.

Every step records itself in `processed.processing` so the call is
fully reproducible.

### Opt-in `auto_group_delay_sweep`

The vendor cascade formula for the digital-filter group delay is
exact in theory but occasionally off in practice — JEOL Eucalyptol
13C is the worked example: cascade predicts 19.66 samples while the
true optimum (the value that leaves the post-pipeline `|ph1|`
residual smallest) is 16.46. The resulting 3.2-sample miscalibration
shows up as a 180°-flipped CDCl3 solvent peak between the two
sample-peak clusters.

When the cascade prediction is suspect, set
`auto_group_delay_sweep` to opt into a brute-force search:

```rust
use rspin_processing::{
    AutoProcessingOptions, GroupDelaySweepOptions, process_spectrum_auto,
};

let options = AutoProcessingOptions {
    auto_group_delay_sweep: Some(GroupDelaySweepOptions {
        delta_samples: 5.0,    // ±5 samples around the cascade value
        step_samples: 0.1,     // 0.1-sample resolution → ~100 candidates
    }),
    ..AutoProcessingOptions::default()
};
let phased = process_spectrum_auto(&fid, &options)?;
```

The orchestrator runs the full pipeline once per candidate and keeps
the result whose residual `|ph1|` from a second auto-phase pass is
smallest. Cost scales linearly with the candidate count (≈ 100× the
single-shot path for the defaults; tighter `±2.0 / step 0.5` is ~9×
and is usually enough to catch a several-sample miscalibration).

Recommended use:

- **Off by default** — the cascade is correct on most fixtures and
  the sweep is expensive.
- **Turn it on when** an experienced eye sees a clearly inverted
  solvent or reference peak with sample peaks otherwise phased
  correctly, or when batch-processing fixtures from a previously
  uncharacterised JEOL configuration.
- **Tighten before disabling** — narrow `delta_samples` to ±2 once
  you've measured the typical miscalibration for the instrument.

See `docs/assets/examples/auto_processing_jeol_eucalyptol_13c_group_delay.png`
for the three-trace comparison: cascade default, manual empirical
override, and the auto-sweep result (which matches the empirical
override without the caller knowing the right value).

## Reproducibility

RSpin records every processing step in `Spectrum1D.processing` with
its operation name and parameters. There is **no hidden default
apodization** — if you don't add a window to your recipe, the FID
goes to FFT unwindowed. Always check `processed.processing` (or its
serialised form) before publishing numbers.

## See also

- [Apodization windows](examples/apodization.md) — formulas, defaults,
  comparison PNG.
- [Auto-phase](examples/auto-phase.md) — Regions vs global-cost, ACME
  entropy, peak-warmed hybrid.
- [Processed-data example](examples/processed-data.md) — end-to-end
  recipe walkthrough.
