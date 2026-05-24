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

## Development

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

## Examples

- Processing and analysis PNG examples: `docs/examples/processed-data.md`
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
