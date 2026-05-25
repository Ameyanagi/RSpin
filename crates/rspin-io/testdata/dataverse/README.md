# Dataverse Test Fixtures

These fixtures are redistributed for parser tests. They are a small selected
subset of public Harvard Dataverse data and MIT-licensed test-data package
copies, not the full upstream archives.

## CC0 Rutin Fixtures

All files currently committed under `cc0/rutin/` trace back to this dataset:

- Dataset: Rutin 400 MHz in DMSOd6 NMR data
- Dataset DOI: 10.7910/DVN/ZAZDNM
- License: Creative Commons Zero v1.0 Universal (CC0 1.0)
- License URL: http://creativecommons.org/publicdomain/zero/1.0
- Dataverse API checked:
  https://dataverse.harvard.edu/api/datasets/:persistentId/?persistentId=doi:10.7910/DVN/ZAZDNM
- Access date: 2026-05-25
- Authors: Kim, Seon Beom; Chen, Shao Nong; Simmler, Charlotte; Bisson,
  Jonathan; Pauli, Guido.
- Producer: CENAPT, University of Illinois at Chicago.

CC0 allows copying, modification, redistribution, and commercial use without
permission. Author/source metadata is retained for provenance and citation
hygiene.

## Package Provenance

The committed file bytes were selected from local unpacked copies of these npm
packages:

- `jcamp-data-test` `2.5.0`
  - Package license: MIT
  - Package author: Julien Wist
  - Package metadata checked with `npm view jcamp-data-test@2.5.0` on
    2026-05-25.
  - Tarball: https://registry.npmjs.org/jcamp-data-test/-/jcamp-data-test-2.5.0.tgz
  - Source package files:
    `data/nmr/rutin/qH.jdx`,
    `data/nmr/rutin/13c.jdx`
- `jeol-data-test` `1.0.0`
  - Package license: MIT
  - Package author: Julien Wist
  - Package metadata checked with `npm view jeol-data-test@1.0.0` on
    2026-05-25.
  - Repository: https://github.com/cheminfo/jeol-data-test
  - Tarball: https://registry.npmjs.org/jeol-data-test/-/jeol-data-test-1.0.0.tgz
  - Source package files:
    `data/Rutin_3080ug200uL_DMSOd6_qHNMR_400MHz_Jeol.jdf`,
    `data/Rutin_3080ug200uL_DMSOd6_13CNMR_400MHz_Jeol.jdf`

The package READMEs identify Harvard Dataverse DOI `10.7910/DVN/ZAZDNM` as the
original data source.

## Included Files

| Local path | Original/package file | Purpose | Changes |
| --- | --- | --- | --- |
| `cc0/rutin/jcamp/rutin_qh_400mhz.jdx` | `jcamp-data-test/data/nmr/rutin/qH.jdx` | JCAMP-DX 6.0 LINK 1D 1H reader | Filename normalized |
| `cc0/rutin/jcamp/rutin_13c_400mhz.jdx` | `jcamp-data-test/data/nmr/rutin/13c.jdx` | JCAMP-DX 6.0 LINK 1D 13C reader | Filename normalized |
| `cc0/rutin/jeol/rutin_qhnmr_400mhz.jdf` | `jeol-data-test/data/Rutin_3080ug200uL_DMSOd6_qHNMR_400MHz_Jeol.jdf` | JEOL 1D 1H JDF reader | Filename normalized |
| `cc0/rutin/jeol/rutin_13cnmr_400mhz.jdf` | `jeol-data-test/data/Rutin_3080ug200uL_DMSOd6_13CNMR_400MHz_Jeol.jdf` | JEOL 1D 13C JDF reader | Filename normalized |

## CC0 Fixture Checksums

```text
266154637771622f5b0ad53b55bcc695aaaf347816afb98f17b91c8939a87057  cc0/rutin/jcamp/rutin_13c_400mhz.jdx
3e37069c351ba6ab145360b1fc8ea7fd3af176db858203e65e3bd3733cc1c5a7  cc0/rutin/jcamp/rutin_qh_400mhz.jdx
c870bd413d4be6b31c17d7e8995a63bfef031777c26bad61aa2ff48e1eb43e46  cc0/rutin/jeol/rutin_13cnmr_400mhz.jdf
bb76e9d4a8bb9dd66b8ddbaeffcee10ce3635f615861caa75630a46453e0cf71  cc0/rutin/jeol/rutin_qhnmr_400mhz.jdf
```

## Package MIT Notice

MIT License

Copyright (c) 2020 cheminfo

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
