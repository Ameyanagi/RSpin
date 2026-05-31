# 2D contour plots with external 1D spectra

RSpin's optional `visualization` feature exposes PNG helpers for NMR-aware
plots. The 2D helper can render a contour plot with caller-provided 1D spectra
around it:

```rust
use rspin::plot::{Spectrum2DPlotOptions, plot_spectrum_2d_with};
use rspin::prelude::*;

let options = Spectrum2DPlotOptions::new()
    .with_x_overlay("1H", &proton_spectrum)
    .with_y_overlay("13C", &carbon_spectrum);

plot_spectrum_2d_with(
    "hsqc_with_1d.png".as_ref(),
    "HSQC with external spectra",
    &hsqc_spectrum,
    &options,
)?;
# Ok::<(), RSpinError>(())
```

The X overlay is rendered above the contour and the Y overlay is rendered on
the left, matching the usual placement of external 1D spectra around a 2D NMR
plot. Overlay axes must use the same unit as the corresponding 2D axis. Point
counts and ranges can differ; the plot clips traces to the 2D axis window.
Overlay intensities are normalized per marginal panel so multiple traces remain
readable.

The generated example command writes:

- `docs/assets/examples/vendors/jeol/eucalyptol_hsqc_2d_nmrxiv_with_1d.png`

Generate it with:

```sh
cargo run --release --manifest-path examples/ruviz-processing/Cargo.toml --features visualization-ruviz
```
