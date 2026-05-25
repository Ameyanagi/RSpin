# RSpin

RSpin is a Rust library workspace for nuclear magnetic resonance data workflows.

The first implementation target is a pure-Rust library stack for IO, processing,
analysis, simulation, and prediction abstractions. A GUI is intentionally out of
scope for now.

## Workspace

- `rspin`: facade crate.
- `rspin-core`: shared model, units, metadata, and errors.
- `rspin-io`: spectrum readers and writers.
- `rspin-processing`: signal-processing operations.
- `rspin-analysis`: peak, range, integral, zone, and assignment logic.
- `rspin-simulation`: synthetic spectrum generation.
- `rspin-prediction`: prediction traits and adapter types.
- `rspin-wasm`: WebAssembly bindings.

## Unified IO

Use the facade crate for normal loading. `load_spectra` accepts one supported
file or directory, while `load_spectra_many` merges selected files and
directories into one `SpectrumBundle`. Use `load_spectra_many_relative_to` when
selected paths should be resolved below one base directory and reported with
stable relative source paths.

```rust,no_run
use rspin::prelude::*;

fn load_one_dataset() -> Result<SpectrumBundle> {
    load_spectra("data/experiment")
}

fn load_selected_inputs() -> Result<SpectrumBundle> {
    load_spectra_many([
        "data/proton.fid",
        "data/carbon.jdf",
        "data/bruker/pdata/1",
    ])
}

fn load_selected_inputs_with_stable_sources() -> Result<SpectrumBundle> {
    load_spectra_many_relative_to("data", ["proton.fid", "carbon.jdf", "bruker/pdata/1"])
}
```

Bundles expose direct counts and owned extraction helpers for simple workflows:

```rust,no_run
use rspin::prelude::*;

fn inspect_bundle() -> Result<Vec<Spectrum1D>> {
    let bundle = load_spectra("data/experiment")?;
    println!("loaded {} 1D and {} 2D spectra", bundle.len_1d(), bundle.len_2d());
    Ok(bundle.into_spectra_1d())
}
```

`RSpinReader` exposes the same reader with chainable options:

```rust,no_run
use rspin::prelude::*;

fn load_processed_only() -> Result<SpectrumBundle> {
    RSpinReader::new()
        .processed_only()
        .strict()
        .read_path("data/bruker")
}
```

The unified loader currently routes supported Bruker, Agilent/Varian, JEOL,
JCAMP-DX, nmrML, NMReDATA, JSON, and CSV inputs without replacing the
format-specific readers. Browser callers should parse uploaded bytes with the
format-specific WASM helpers, then use `createSpectrumBundle` to assemble the
same versioned bundle JSON used by native code.

The small committed loader fixtures under
`crates/rspin-io/testdata/zenodo_7100132` come from the MIT-licensed Zenodo
software record `https://doi.org/10.5281/zenodo.7100132`; see the fixture
README for file-level provenance and checksums.

The committed public parser fixtures under `crates/rspin-io/testdata/nmrxiv/cc0`
come from a CC0 NMRXiv study; see the fixture README for source DOI, authors,
license URL, and included-file provenance. Other public datasets should only be
committed when redistribution is permitted and documented next to the files. The
top-level `crates/rspin-io/testdata/README.md` records the fixture policy.

## Development

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

## Examples

- Processing and analysis PNG examples: `docs/examples/processed-data.md`
- Local visual artifacts from raw oracle fixtures:
  `target/rspin-visual-tests` after running the feature-gated `ruviz` example.
- Public analysis fixture sources and workflow snippets:
  `docs/examples/public-analysis-fixtures.md`

## Optional Features

- `external-baselines`: enables the optional crates.io `baselines` crate for
  additional baseline correction methods. It is off by default so the core
  library dependency graph stays small and reviewable.
- `visualization-ruviz`: enables the standalone processed-data PNG example in
  `examples/ruviz-processing`. It is not part of the main workspace defaults.

## License

Licensed under either of:

- Apache License, Version 2.0
- MIT license

at your option.
