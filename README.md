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
directories into one `SpectrumBundle`.

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
```

`RSpinReader` exposes the same reader with chainable options:

```rust,no_run
use rspin::prelude::*;

fn load_processed_only() -> Result<SpectrumBundle> {
    RSpinReader::new()
        .with_raw(false)
        .with_processed(true)
        .with_strict(true)
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
