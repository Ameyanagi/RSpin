# NMRXiv Test Fixtures

These fixtures are redistributed for parser tests. They are a small extracted
subset of public NMRXiv data, not the full study archives.

## CC0 Myrcene Fixtures

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

## Source Study: Myrcene

- Study: Myrcene 60/400/900 MHz in CDCl3 NMR data
- Study identifier: NMRXIV:S217
- Study DOI: 10.57992/nmrxiv.p33.s217
- Study URL: https://nmrxiv.org/sample/S217
- Authors: Bisson J, McAlpine JB, Friesen JB, Chen SN, Graham J, Pauli GF.
- Source APIs checked: https://nmrxiv.org/api/v1/S217 and https://nmrxiv.org/api/v1/P33
- Access date: 2026-05-25
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

## Source Study: Eucalyptol

- Study: Eucalyptol 400 MHz in CDCl3 NMR data
- Study identifier: NMRXIV:S218
- Study DOI: 10.57992/nmrxiv.p33.s218
- Study URL: https://nmrxiv.org/sample/S218
- Authors: Bisson J, McAlpine JB, Friesen JB, Chen SN, Graham J, Pauli GF.
- Source APIs checked: https://nmrxiv.org/api/v1/S218 and https://nmrxiv.org/api/v1/P33
- Access date: 2026-05-25
- Original archive URL: https://s3.uni-jena.de/nmrxiv/production/archive/18c47f7f-69c0-4858-9d6d-f67e04099f92/%20Eucalyptol%20400%20MHz%20in%20CDCl3%20NMR%20data%20.zip
- Original archive SHA-256: 30bc79d21902c3b94d9603f476a493c88bffc9fe3184eabef450a3c8ae3afba5

| Local path | Original file | Source DOI | Purpose | Changes |
| --- | --- | --- | --- | --- |
| `cc0/eucalyptol/jeol/eucalyptol_qhnmr_400mhz.jdf` | `Eucalyptol_9070ug200uL_CDCl3_qHNMR_400MHz_Jeol.jdf` | 10.57992/nmrxiv.p33.s218.d1203 | JEOL 1D 1H JDF reader | Filename normalized |
| `cc0/eucalyptol/jeol/eucalyptol_13cnmr_400mhz.jdf` | `Eucalyptol_9070ug200uL_CDCl3_13CNMR_400MHz_Jeol.jdf` | 10.57992/nmrxiv.p33.s218.d1201 | JEOL 1D 13C JDF reader | Filename normalized |
| `cc0/eucalyptol/jeol/eucalyptol_hsqc_400mhz.jdf` | `Eucalyptol_9070ug200uL_CDCl3_HSQC_400MHz_Jeol.jdf` | 10.57992/nmrxiv.p33.s218.d1202 | JEOL 2D HSQC JDF reader | Filename normalized |
| `cc0/eucalyptol/jcamp/eucalyptol_qh_400mhz_jcamp_dx_6_link.jdx` | `Eucalyptol_9070ug200uL_CDCl3_qHNMR_400MHz_JDX.jdx` | 10.57992/nmrxiv.p33.s218.d1206 | JCAMP-DX 6.0 LINK 1D 1H reader | Filename normalized |
| `cc0/eucalyptol/jcamp/eucalyptol_13c_400mhz_jcamp_dx_6_link.jdx` | `Eucalyptol_9070ug200uL_CDCl3_13CNMR_400MHz_JDX.jdx` | 10.57992/nmrxiv.p33.s218.d1207 | JCAMP-DX 6.0 LINK 1D 13C reader | Filename normalized |
| `cc0/eucalyptol/jcamp/eucalyptol_hsqc_400mhz_jcamp_dx_6_link.jdx` | `Eucalyptol_9070ug200uL_CDCl3_HSQC_400MHz_JDX.jdx` | 10.57992/nmrxiv.p33.s218.d1199 | JCAMP-DX 6.0 LINK 2D HSQC reader | Filename normalized |

## CC0 Fixture Checksums

```text
3ca0f67327a8a10ac2d674944f245f0265f5b0bd80f0f707c0aed36f92af8176  cc0/eucalyptol/jcamp/eucalyptol_qh_400mhz_jcamp_dx_6_link.jdx
c51b192b6cb1a4e2983cc6fcf1db8eff235c0cca05c0508423944ebea2e1558f  cc0/eucalyptol/jcamp/eucalyptol_13c_400mhz_jcamp_dx_6_link.jdx
588c1294dd1bb6bd963870622b2f3e71cf4024ce6fcc114221be8112e636ff9b  cc0/eucalyptol/jcamp/eucalyptol_hsqc_400mhz_jcamp_dx_6_link.jdx
672ea4f438254ddcc190c7d82e5fe44b03256b3fdb9653dd3723f29caa769bca  cc0/eucalyptol/jeol/eucalyptol_qhnmr_400mhz.jdf
99db1fbba394d88063ac45a3afd490e71eb991c51fbcab97c060ee8876c43186  cc0/eucalyptol/jeol/eucalyptol_13cnmr_400mhz.jdf
1f0f0f973bb83d62381f834e5aa02cc5b0647349e01f5745f8c85b01721b0813  cc0/eucalyptol/jeol/eucalyptol_hsqc_400mhz.jdf
2ac9051cdd709d6a83a04fdee2b4e888c7b1d9fbcfd299fc53a2b034eb8bc78f  cc0/myrcene/bruker_1h_raw/acqus
11805b5f53539b497f1c2b8e0ae79f86d4c51bfe442890d6314f959474daba27  cc0/myrcene/bruker_1h_raw/fid
e15f6d9f09307549153be80152a3f38bc08fe8dc8a9e6f8acd236c468d82104d  cc0/myrcene/bruker_cosy_raw/acqu2s
c5fdc2c9e3c804ad7ea8ec45aa71d265a1871e93f629c13e9e42e44c4560599d  cc0/myrcene/bruker_cosy_raw/acqus
5a6a19df4af686a1914c7fa5eadb9556dc852e02ca504d4deb3afe926216537c  cc0/myrcene/bruker_cosy_raw/ser
682dbf06e2f2c152ea4fc267774bb75b37d235890f8a83ef5cf66d3a74931d82  cc0/myrcene/jcamp/myrcene_13c_400mhz_jcamp_dx_6_link.jdx
d3e11e49efbb6a4a5c004530c91029f707d236bbef1bbfbb8275e2de1c510d36  cc0/myrcene/jcamp/myrcene_1h_400mhz_jcamp_dx_6_link.jdx
5726137b3bbaa9eab5f31ed43b29955d2cf188ad9a2ae51f778cd745914131b1  cc0/myrcene/jeol/myrcene_13c_400mhz.jdf
81bf08236f08268baf82e3379ece13a18d9b0120f52d9805731713f8433f2af2  cc0/myrcene/jeol/myrcene_1h_400mhz.jdf
9393780f1eaaa080714053d17ef242e6057d94357ed091bd27cdbaf355773a68  cc0/myrcene/jeol/myrcene_hsqc_400mhz.jdf
```

## CC-BY Varian Fixtures

Files committed under `cc-by-4.0/varian_11a/` come from an NMRXiv study whose
project is licensed under Creative Commons Attribution 4.0 International
(`CC-BY-4.0`). CC-BY permits redistribution when attribution is retained and
changes are indicated.

- Project: Synthesis and biological evaluation of highly potent fungicidal deoxy-Hygrophorones
- Project identifier: NMRXIV:P57
- Project DOI: 10.57992/nmrxiv.p57
- Project URL: https://nmrxiv.org/project/P57
- Study: 11a_Varian
- Study identifier: NMRXIV:S332
- Study DOI: 10.57992/nmrxiv.p57.s332
- Study URL: https://nmrxiv.org/sample/S332
- License: Creative Commons Attribution 4.0 International (CC BY 4.0)
- License URL: https://creativecommons.org/licenses/by/4.0/legalcode
- Authors: Ludger A Wessjohann, Toni Ditfe, Norbert Arnold, Bernhard Westermann, Haider Sultani, Eileen Bette.
- Source APIs checked: https://nmrxiv.org/api/v1/S332 and https://nmrxiv.org/api/v1/P57
- Access date: 2026-05-25
- Original archive URL: https://s3.uni-jena.de/nmrxiv/production/archive/bd0e206e-6103-496c-9975-4f72d0d0f0b1/11a_Varian.zip
- Original archive SHA-256: 51c9a04c997b64241b61021baa6366faf33c9fdc40998b996556efbd5da464dd
- Changes: selected only the `fid` and `procpar` files needed for parser tests;
  directory names were normalized for stable test paths.

| Local path | Original file | Source DOI | Purpose | Changes |
| --- | --- | --- | --- | --- |
| `cc-by-4.0/varian_11a/proton_1h/` | `11a_HEE294_20140210_01.1H.fid/{fid,procpar}` | 10.57992/nmrxiv.p57.s332 | Agilent/Varian raw 1D 1H FID reader | Extracted subset; directory renamed |
| `cc-by-4.0/varian_11a/carbon_13c/` | `11a_HEE294_20140210_01.13C.fid/{fid,procpar}` | 10.57992/nmrxiv.p57.s332 | Agilent/Varian raw 1D 13C FID reader | Extracted subset; directory renamed |
| `cc-by-4.0/varian_11a/dept_13c/` | `11a_HEE294_20140210_01.dept.fid/{fid,procpar}` | 10.57992/nmrxiv.p57.s332 | Agilent/Varian raw 1D 13C DEPT FID reader | Extracted subset; directory renamed |

## CC-BY Fixture Checksums

```text
a99121fee1e4feabe36a58edf1bcc633aa27853dd83a71f7d46ad8bfd75c91ea  cc-by-4.0/varian_11a/carbon_13c/fid
ffbd347fba876bfe10fec4ca2e27f0198485d9372b54635bc8c64a97c03901e3  cc-by-4.0/varian_11a/carbon_13c/procpar
ca0bdb7e1b953b2690cda48740b6524fabc7021ba8633b6d203c48fbec6661ba  cc-by-4.0/varian_11a/dept_13c/fid
c4f7d18b70cad4b7cec5f6186861f8e79ef2047e21e8273d60ea27dda81e0eaf  cc-by-4.0/varian_11a/dept_13c/procpar
0227e7b9133639a0d3241c840fd830587c6da7aa9d42abd73de05fff0bacd132  cc-by-4.0/varian_11a/proton_1h/fid
9eb18bda75b089f24cce66ea6703e9addb86039b5657ec3f797025b499f1ee86  cc-by-4.0/varian_11a/proton_1h/procpar
```
