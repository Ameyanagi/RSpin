# Zenodo 10.5281/zenodo.7100132 Loader Fixtures

These small fixtures are copied from the MIT-licensed Zenodo software record
for "Javascript NMR loader and saver".

- Concept DOI: `https://doi.org/10.5281/zenodo.7100132`
- Version DOI used for files: `https://doi.org/10.5281/zenodo.8338410`
- License: MIT
- Copyright: Copyright (c) 2021 cheminfo
- Authors: Alejandro Bolanos, Luc Patiny, Michael Zasso
- Changes: selected a small Varian/Agilent raw 1D fixture, a Bruker raw and
  processed fixture without experiment-number nesting, and a minimal empty
  JCAMP-DX warning fixture. Directory names, text fixture line endings, and
  trailing text whitespace were normalized for RSpin tests.

## Fixture Checksums

```text
ee01f040f1c3bfc5aecbfb0c710f5d4ed1aeed7e83b6ade8f0188e4db06497d6  bruker_without_expno/acqus
46d1cc0963bb5b3c918380f0234169595db19714fccb496f52c018390668de5a  bruker_without_expno/fid
973a9e88d295f4e79ce837383af1a6bd2f5b1f452e959d8b921d6913b52dbaa8  bruker_without_expno/pdata/1/1i
eb3f97cc2f9607f93a7c552c5f531ce74f28df9740ecb34154ca3080785bab11  bruker_without_expno/pdata/1/1r
a7401ab53563c9aca0087d221140e7aad39fc89c67ccad27b8e89f8cac07c31a  bruker_without_expno/pdata/1/procs
2944bf4dacbb76b557e32a89cc695af687a40de659782c9c9fea154b472a83a1  empty_jcamp/empty.jdx
fba1f8312bde9b03ee9c786b8c9cff26f10f692bfc1383429840c440fbddca05  varian_1h/fid
dce3ffcd97233364b945caafdc27357482e4e91b1a65461b78df435f5796a331  varian_1h/procpar
```

## MIT Notice

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
