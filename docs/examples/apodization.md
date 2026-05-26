# Apodization windows

RSpin provides a complete set of NMR time-domain weighting (apodization)
functions. Apply them to the complex FID **after** any linear-prediction
extension and **before** zero-filling and FFT, following the standard
nmrPipe / nmrglue convention.

## Available windows

| Function | Public name | Best for | Parameters |
|----------|-------------|----------|------------|
| Exponential (EM) | `ExponentialApodization` | SNR enhancement | `lb_hz`, `dwell` |
| Gaussian (GM) | `GaussianApodization` | Smooth tail damping | `gb_hz`, `dwell` |
| Lorentz-to-Gauss | `LorentzToGaussApodization` | Resolution enhancement (1H) | `lb_undo_hz`, `gb_fwhm_hz`, `shift`, `dwell` |
| TRAF (Traficante) | `TrafApodization` | 13C SNR-preserving sharpening | `lb_hz`, `dwell` |
| Sine-bell / SP | `SineBellApodization` | HSQC indirect dim | `start_deg`, `end_deg`, `exp` |
| Trapezoidal (TM) | `TrapezoidalApodization` | LP-extrapolation tail | `rise`, `fall` |
| Bruker GMB | `GaussMultiplyBrukerApodization` | Bruker `procs` interop | `lb_hz`, `gb_fraction`, `dwell` |
| Convolution difference | `ConvolutionDifferenceApodization` | Paramagnetic / solid cleanup | `lb_narrow`, `lb_broad`, `mixing`, `dwell` |

The shifted-sine family has convenience constructors that match nmrPipe's
`-fn SP` defaults:

```rust
SineBellApodization::sine_squared();   // 0 → 180°, exp = 2
SineBellApodization::cosine_bell();    // 90 → 180°, exp = 1 (Hann)
SineBellApodization::cosine_squared(); // 90 → 180°, exp = 2 (HSQC F1 default)
SineBellApodization::shifted_sine(off, exp);
```

## Auto-pick the SNR-optimal exponential broadening

`matched_filter_em(spectrum)` runs a forward FFT internally, measures
the FWHM of the strongest magnitude peak, and returns the
`ExponentialApodization` whose `lb_hz` matches the natural linewidth
(divided by √3 to convert magnitude FWHM → absorption FWHM, the
Ernst-optimal value). The input must be a uniformly-sampled
time-domain FID.

```rust
let em = matched_filter_em(&fid)?;
let processed = em.apply(&fid)?;
let frequency = fft_1d(&processed, FftDirection::Forward)?;
```

## Recommended defaults

| Experiment | Nucleus | Window |
|------------|---------|--------|
| 1D | 1H | EM 0.3 Hz, or Lorentz-to-Gauss for resolution |
| 1D | 13C | EM 1.0–3.0 Hz, or TRAF |
| 2D HSQC | F2 (1H) | cosine-squared |
| 2D HSQC | F1 (13C, indirect) | cosine-squared with shift 0.5 |
| FID + LP | any | EM/GM then trapezoidal fall ~0.85 |

## Order in the processing pipeline

1. Group-delay removal (`remove_group_delay`).
2. Backward linear prediction (replaces broken first points), if any.
3. Forward linear prediction (extends tail), if any.
4. **Apodization** (one window from this page).
5. First-point scaling (multiply intensity[0] by 0.5).
6. Zero-fill.
7. FFT.
8. Phase correction, baseline correction, etc.

## References

- Ferrige & Lindon, *J. Magn. Reson.* 31 (1978) 337 — Lorentz-to-Gauss.
- Traficante, *Concepts Magn. Reson.* 12 (2000) 83–101 — TRAF.
- Campbell, Dobson, Williams & Xavier, *J. Magn. Reson.* 11 (1973) 172 — convolution difference.
- Ernst, Bodenhausen & Wokaun, *Principles of NMR in One and Two Dimensions* (1987) — matched filter.
- nmrPipe `apod` documentation, NIH (Delaglio et al.) — sine-bell, GMB, trapezoidal conventions.
