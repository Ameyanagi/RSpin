# NMRXiv Test Fixtures

These fixtures are redistributed for parser tests. They are a small extracted
subset of public NMRXiv data, not the full study archives.

## License

All files currently committed under `cc0/` come from the NMRXiv CENAPTNMR
project:

- Project: CENAPTNMR
- Project identifier: NMRXIV:P33
- Project DOI: 10.57992/nmrxiv.p33
- License: Creative Commons Zero v1.0 Universal (CC0 1.0)
- License URL: https://creativecommons.org/publicdomain/zero/1.0/legalcode
- Project URL: https://nmrxiv.org/project/P33

CC0 allows copying, modification, redistribution, and commercial use without
permission. Author/source metadata is retained below for provenance and
citation hygiene.

## Source Study

- Study: Myrcene 60/400/900 MHz in CDCl3 NMR data
- Study identifier: NMRXIV:S217
- Study DOI: 10.57992/nmrxiv.p33.s217
- Study URL: https://nmrxiv.org/sample/S217
- Authors: Bisson J, McAlpine JB, Friesen JB, Chen SN, Graham J, Pauli GF.
- Original archive URL: https://s3.uni-jena.de/nmrxiv/production/archive/49c84d81-f37e-46a8-ae1c-da18f979751c/Myrcene%2060:400:900%20MHz%20in%20CDCl3%20NMR%20data.zip
- Original archive SHA-256: 7ae9672de021f93e6a61fd63fd8575b23200a5e8289c6cd6de65d6180c022fb3

## Included Files

| Local path | Original file | Source DOI | Purpose | Changes |
| --- | --- | --- | --- | --- |
| `cc0/myrcene/bruker_1h_raw/` | `Myrcene_100000ug700uL_CDCl3_1HNMR_900MHz_Bruker/{acqus,fid}` | 10.57992/nmrxiv.p33.s217.d1188 | Bruker raw 1D 1H FID reader | Extracted subset only |
| `cc0/myrcene/bruker_cosy_raw/` | `Myrcene_100000ug700uL_CDCl3_COSY_900MHz_Bruker/{acqus,acqu2s,ser}` | 10.57992/nmrxiv.p33.s217.d1190 | Bruker raw 2D COSY SER reader | Extracted subset only |
| `cc0/myrcene/jeol/myrcene_1h_400mhz.jdf` | `Myrcene_100000ug700uL_CDCl3_1H_400MHz_Jeol.jdf` | 10.57992/nmrxiv.p33.s217.d1195 | JEOL 1D 1H JDF reader | Filename normalized |
| `cc0/myrcene/jeol/myrcene_13c_400mhz.jdf` | `Mycene_3120ug200uL_CDCl3_13C_400MHz_Jeol.jdf` | 10.57992/nmrxiv.p33.s217.d1187 | JEOL 1D 13C JDF reader | Filename normalized |
| `cc0/myrcene/jeol/myrcene_hsqc_400mhz.jdf` | `Mycene_3120ug200uL_CDCl3_HSQC_400MHz_Jeol.jdf` | 10.57992/nmrxiv.p33.s217.d1197 | JEOL 2D HSQC JDF reader | Filename normalized |
| `cc0/myrcene/jcamp/myrcene_1h_400mhz_jcamp_dx_6_link.jdx` | `Myrcene_100000ug700uL_CDCl3_1H_400MHz_JDX.jdx` | 10.57992/nmrxiv.p33.s217.d1183 | JCAMP-DX 6.0 LINK 1D reader | Filename normalized |
| `cc0/myrcene/jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx` | `Mycene_3120ug200uL_CDCl3_13C_400MHz_JDX.jdx` | 10.57992/nmrxiv.p33.s217.d1187 | JCAMP-DX 6.0 LINK 13C reader | Filename normalized |
