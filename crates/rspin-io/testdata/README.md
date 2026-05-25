# RSpin IO Test Data

This directory may contain small parser fixtures only when redistribution is
permitted and provenance is documented next to the files. Do not add fixture
archives or extracted data here when the license is unclear, non-redistributable,
or only acceptable for private comparison.

## Committed Sources

- `zenodo_7100132/`: selected MIT-licensed software-record fixtures from
  Zenodo records `10.5281/zenodo.7100132` and `10.5281/zenodo.8338410`. See the
  nested README for authors, copyright, changes, and checksums.
- `nmrxiv/cc0/`: selected CC0 NMRXiv CENAPTNMR/Myrcene fixtures. See the nested
  README for study DOI, source URLs, authors, license URL, and file-level
  provenance.
- `bundle_nmredata/`: small project-authored synthetic NMReDATA fixture for
  bundle metadata tests.

## Fixture Rules

- Prefer MIT, Apache-2.0, BSD, CC0/public-domain, or otherwise explicitly
  redistributable data.
- CC-BY or CC-BY-SA data may be committed only after the exact attribution and
  redistribution requirements are recorded in the fixture README.
- Keep CC-BY-NC, CC-BY-NC-SA, no-license, or unclear-license files in the local
  external cache, not in this repository.
- Do not commit derived golden outputs, screenshots, or parsed snapshots from
  restricted external fixtures.
- Keep committed fixtures small and targeted to parser behavior.
