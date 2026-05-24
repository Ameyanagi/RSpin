//! Spectrum input and output.

mod csv;
mod csv_2d;
mod csv_common;
mod jcamp;
mod json;
mod traits;

pub use csv::{CsvSpectrum1D, read_spectrum1d_csv, write_spectrum1d_csv};
pub use csv_2d::{CsvSpectrum2D, read_spectrum2d_csv, write_spectrum2d_csv};
pub use jcamp::{JcampDx, read_jcamp_dx_1d, write_jcamp_dx_1d};
pub use json::{
    JsonSpectrum1D, JsonSpectrum2D, read_spectrum1d_json, read_spectrum2d_json,
    write_spectrum1d_json, write_spectrum2d_json,
};
pub use traits::{SpectrumReader, SpectrumWriter};
