# Public Analysis Fixture Sources

RSpin examples should prefer small committed fixtures with clear attribution and
download-on-demand scripts for larger public datasets. Do not vendor external
archives until the specific file license and attribution requirements are
recorded next to the fixture.

## Recommended Sources

- nmrML examples: `https://nmrml.org/examples/`
  - Best first source for example workflows because it includes raw FID,
    processed 1D, annotated reference spectra, complex biosample, and 2D HSQC
    examples.
  - Repository: `https://github.com/nmrML/nmrML`
  - The repository is MIT licensed, but each example directory should still be
    checked for source notes before vendoring data.
- nmrXiv: `https://docs.nmrxiv.org/`
  - Good source for realistic open datasets and larger manual examples.
  - Public data has explicit project or study licensing. Record the concrete
    license, creator, URL, and access date before use.
- BMRB / PDBj-BMRB: `https://bmrb.io/`
  - Useful for biomolecular and small-molecule NMR examples, including peak
    lists, time-domain data, and 1D/2D spectra.
  - Prefer download-on-demand examples unless a fixture has a clearly recorded
    redistribution status.
- IUPAC JCAMP-DX archive: `https://github.com/IUPAC/JCAMP-DX`
  - Useful for parser conformance and edge cases.
  - Treat as reference/download-on-demand until redistribution terms for the
    specific files are reviewed.

## Avoid For Default Fixtures

- Non-commercial or unclear datasets should not be committed to the default test
  tree. They may be useful for local behavior checks, but keep them out of the
  default crate fixtures unless the license fits the project policy.
- Do not add GPL, CC-BY-NC, CC-BY-NC-SA, or no-license data to default crates.

## Analysis Workflow Example

The high-level API is intended to make public data easy to inspect once parsed
into RSpin spectra:

```rust
use rspin::prelude::*;

fn analyze_public_1d_fixture(jcamp: &str) -> Result<SpectrumAnalysis1D> {
    let spectrum = read_jcamp_dx_1d(jcamp)?;
    analyze_spectrum_1d(
        &spectrum,
        SpectrumAnalysis1DOptions::new()
            .with_peak_options(
                PeakPickOptions::new()
                    .with_min_abs_intensity(0.05)
                    .with_min_prominence(0.01),
            )
            .with_range_options(
                RangeDetectionOptions::new()
                    .with_threshold_abs(0.03)
                    .with_merge_gap_points(2),
            )
            .with_multiplet_options(
                MultipletDetectionOptions::new()
                    .with_max_peak_gap_ppm(0.04)
                    .with_singlets(true),
            ),
    )
}
```

For 2D spectra, use the same workflow shape after import:

```rust
use rspin::prelude::*;

fn analyze_public_2d_fixture(spectrum: &Spectrum2D) -> Result<SpectrumAnalysis2D> {
    analyze_spectrum_2d(
        spectrum,
        SpectrumAnalysis2DOptions::new().with_zone_options(
            ZoneDetectionOptions::new()
                .with_threshold_abs(0.05)
                .with_connectivity(ZoneConnectivity::Eight),
        ),
    )
}
```

Rust callers can also use the chainable workflow builders from the prelude:

```rust
use rspin::prelude::*;

fn analyze_with_chain_methods(spectrum: &Spectrum1D) -> Result<SpectrumAnalysis1D> {
    spectrum
        .analyze()
        .with_peak_options(PeakPickOptions::new().with_min_abs_intensity(0.05))
        .with_range_options(RangeDetectionOptions::new().with_threshold_abs(0.03))
        .with_multiplet_options(MultipletDetectionOptions::new().with_max_peak_gap_ppm(0.04))
        .run()
}
```

Analysis workflow results can be exported as sectioned CSV for quick inspection:

```rust
use rspin::prelude::*;

fn analyze_and_export_csv(spectrum: &Spectrum1D) -> Result<String> {
    let analysis = spectrum
        .analyze()
        .with_peak_options(PeakPickOptions::new().with_min_abs_intensity(0.05))
        .run()?;
    write_analysis1d_csv(&analysis)
}
```
