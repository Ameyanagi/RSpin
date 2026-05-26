# External test data (git submodules)

This directory references third-party NMR test-data repositories as **git
submodules**. The data is **not** vendored into RSpin's history — the main repo
stores only submodule pointers (gitlinks). The bytes live in the upstream
forks and are downloaded only when you initialize the submodules.

## Initialize

```sh
git submodule update --init --depth 1 external-testdata/cheminfo/<name>
# or all of them:
git submodule update --init --depth 1 --recursive
```

Tests that use this data **skip cleanly** when the submodule directory is
absent or empty, so the standard `cargo test --workspace` does not require it.

## Sources and licenses

All four repositories are MIT-licensed (verified via the GitHub license API).
They are forked under `Ameyanagi/*` to insulate against upstream changes; each
fork preserves the upstream `LICENSE`.

| Submodule | Fork | Upstream | License |
|-----------|------|----------|---------|
| `cheminfo/bruker-data-test`   | `Ameyanagi/bruker-data-test`   | `cheminfo/bruker-data-test`   | MIT |
| `cheminfo/jeol-data-test`     | `Ameyanagi/jeol-data-test`     | `cheminfo/jeol-data-test`     | MIT |
| `cheminfo/jcamp-data-test`    | `Ameyanagi/jcamp-data-test`    | `cheminfo/jcamp-data-test`    | MIT |
| `cheminfo/nmredata-data-test` | `Ameyanagi/nmredata-data-test` | `cheminfo/nmredata-data-test` | MIT |

`cheminfo/quadsystems-data-test` was intentionally **excluded** — it has no
detected license file, so it is treated as license-unclear and kept out per the
repository's data-licensing policy. `cheminfo/data-test-api` is tooling (an API
server), not fixture data, so it is not included.

Do not re-license or relicense this data; it remains under its upstream MIT
terms.
