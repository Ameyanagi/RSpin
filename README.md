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
file or directory, while `load_spectra_relative_to` resolves one selected path
below a base directory and reports stable relative source paths.
`load_spectra_many` merges selected files and directories into one
`SpectrumBundle`. Use `load_spectra_many_relative_to` when multiple selected
paths should share one base directory.

```rust,no_run
use rspin::prelude::*;

fn load_one_dataset() -> Result<SpectrumBundle> {
    load_spectra("data/experiment")
}

fn load_one_dataset_with_stable_sources() -> Result<SpectrumBundle> {
    load_spectra_relative_to("data", "experiment")
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

fn load_selected_bruker_inputs() -> Result<SpectrumBundle> {
    load_spectra_many_by_source_vendor(
        ["data/proton.fid", "data/bruker/pdata/1"],
        LoadedSourceVendor::Bruker,
    )
}

fn load_selected_with_runtime_filter() -> Result<SpectrumBundle> {
    let filter = LoadedSourceFilter::vendor("bruker");
    load_spectra_many_by_source(["data/proton.fid", "data/bruker/pdata/1"], filter)
}

fn load_selected_with_runtime_filters() -> Result<SpectrumBundle> {
    load_spectra_by_sources(
        "data/mixed-vendor",
        [
            LoadedSourceFilter::vendor("bruker"),
            LoadedSourceFilter::path("jcamp/carbon_13c.jdx"),
        ],
    )
}

fn load_one_with_runtime_filter() -> Result<Spectrum1D> {
    load_spectrum_1d_by_source("data/mixed-vendor", LoadedSourceFilter::vendor("bruker"))
}

fn load_one_bruker_spectrum_from_selected_inputs() -> Result<Spectrum1D> {
    load_spectrum_1d_many_by_source_vendor(
        ["data/proton.fid", "data/bruker/pdata/1"],
        LoadedSourceVendor::Bruker,
    )
}

fn load_exactly_one_selected_spectrum() -> Result<Spectrum1D> {
    load_spectrum_1d_relative_to("data", "proton.fid")
}

fn load_exactly_one_bruker_spectrum() -> Result<Spectrum1D> {
    load_spectrum_1d_by_source_vendor_relative_to(
        "data",
        "mixed-vendor",
        LoadedSourceVendor::Bruker,
    )
}

fn load_exactly_one_tracked_source() -> Result<Spectrum2D> {
    load_spectrum_2d_by_source_path_relative_to(
        "data",
        "mixed-vendor",
        "mixed-vendor/jeol/hsqc.jdf",
    )
}

fn load_jcamp_bundle() -> Result<SpectrumBundle> {
    load_spectra_by_source_format("data/mixed-vendor", LoadedSourceFormat::JcampDx)
}

fn load_bruker_bundle() -> Result<SpectrumBundle> {
    load_spectra_by_source_vendor("data/mixed-vendor", LoadedSourceVendor::Bruker)
}

fn load_one_tracked_source_as_bundle() -> Result<SpectrumBundle> {
    load_spectra_by_source_path("data/mixed-vendor", "jcamp/carbon_13c.jdx")
}
```

Bundles expose direct counts and owned extraction helpers for simple workflows:

```rust,no_run
use rspin::prelude::*;

fn inspect_bundle() -> Result<Vec<Spectrum1D>> {
    let bundle = load_spectra("data/experiment")?;
    let summary = bundle.summary();
    println!("loaded {} 1D and {} 2D spectra", summary.spectra_1d(), summary.spectra_2d());
    println!(
        "loaded {} JCAMP-DX spectra",
        summary.source_format_count(LoadedSourceFormat::JcampDx)
    );
    for count in &summary.source_formats {
        println!("{}: {}", count.format(), count.count());
    }
    for count in &summary.source_vendors {
        println!("{} vendor spectra: {}", count.vendor(), count.count());
    }
    for (spectrum, source) in bundle.loaded_1d_by_source_format(LoadedSourceFormat::JcampDx) {
        let label = source
            .path()
            .map_or_else(|| "<memory>".to_owned(), |path| path.display().to_string());
        println!("{label} has {} points", spectrum.len());
    }
    for path in bundle.source_paths_for_source(LoadedSourceFilter::vendor("bruker")) {
        println!("Bruker source: {}", path.display());
    }
    Ok(bundle.into_spectra_1d())
}

fn load_owned_vendor_subset() -> Result<Vec<Spectrum1D>> {
    let bundle = load_spectra("data/mixed-vendor")?;
    Ok(bundle.into_spectra_1d_by_source(LoadedSourceFilter::vendor("bruker")))
}

fn load_owned_runtime_subset() -> Result<Vec<Spectrum1D>> {
    let bundle = load_spectra("data/mixed-vendor")?;
    Ok(bundle.into_spectra_1d_by_sources([
        LoadedSourceFilter::vendor("bruker"),
        LoadedSourceFilter::path("jcamp/carbon_13c.jdx"),
    ]))
}

fn keep_runtime_subset_as_bundle() -> Result<SpectrumBundle> {
    let bundle = load_spectra("data/mixed-vendor")?;
    Ok(bundle.source_subset_by_sources([
        LoadedSourceFilter::vendor("bruker"),
        LoadedSourceFilter::path("jcamp/carbon_13c.jdx"),
    ]))
}

fn load_named_carbon_spectrum() -> Result<Spectrum1D> {
    let bundle = load_spectra("data/sample")?;
    let (spectrum, _) = bundle
        .loaded_1d_by_source_path("carbon_13c")
        .ok_or_else(|| RSpinError::Parse {
            format: "spectrum bundle",
            message: "missing carbon_13c".to_owned(),
        })?;
    Ok(spectrum.clone())
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

fn load_jcamp_only() -> Result<SpectrumBundle> {
    RSpinReader::new()
        .only_source_format(LoadedSourceFormat::JcampDx)
        .read_path("data/mixed-vendor")
}

fn load_bruker_only() -> Result<SpectrumBundle> {
    RSpinReader::new()
        .only_source_vendor(LoadedSourceVendor::Bruker)
        .read_path("data/mixed-vendor")
}

fn load_with_runtime_filter(filter: LoadedSourceFilter) -> Result<SpectrumBundle> {
    RSpinReader::new()
        .only_source(filter)
        .read_path("data/mixed-vendor")
}

fn load_one_tracked_path() -> Result<SpectrumBundle> {
    RSpinReader::new()
        .only_source_path("jcamp/carbon_13c.jdx")
        .read_path("data/mixed-vendor")
}
```

Use the type-safe discovery helpers when building format or vendor selectors:

```rust,no_run
use rspin::prelude::*;

fn supported_vendor_filters() -> Vec<&'static str> {
    LoadedSourceVendor::all()
        .iter()
        .map(|vendor| vendor.as_str())
        .collect()
}

fn supported_format_filters() -> Vec<&'static str> {
    LoadedSourceFormat::all()
        .iter()
        .map(|format| format.as_str())
        .collect()
}

fn supported_standalone_extensions() -> Vec<&'static str> {
    LoadedSourceFormat::all()
        .iter()
        .flat_map(|format| format.file_extensions())
        .copied()
        .collect()
}
```

The unified loader currently routes supported Bruker, Agilent/Varian, JEOL,
JCAMP-DX, nmrML, NMReDATA, JSON, and CSV inputs without replacing the
format-specific readers. Browser callers should parse uploaded bytes with the
format-specific WASM helpers, then use `createSpectrumBundle` to assemble the
same versioned bundle JSON used by native code. Use
`spectrumBundleSourceFormats` and `spectrumBundleSourceVendors` to populate
browser format, extension, vendor, and directory-marker selectors from the same
source list as native Rust.

The small committed loader fixtures under
`crates/rspin-io/testdata/zenodo_7100132` come from the MIT-licensed Zenodo
software record `https://doi.org/10.5281/zenodo.7100132`; see the fixture
README for file-level provenance and checksums.

The committed public parser fixtures under `crates/rspin-io/testdata/nmrxiv`
come from CC0 and CC-BY-4.0 NMRXiv studies; see the fixture README for source
DOIs, authors, license URLs, included-file provenance, changes, and checksums.
Other public datasets should only be committed when redistribution is permitted
and documented next to the files. The top-level
`crates/rspin-io/testdata/README.md` records the fixture policy.

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
