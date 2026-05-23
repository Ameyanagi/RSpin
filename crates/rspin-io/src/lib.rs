//! Spectrum input and output.

mod jcamp;
mod json;
mod traits;

pub use jcamp::{JcampDx, read_jcamp_dx_1d, write_jcamp_dx_1d};
pub use json::{
    JsonSpectrum1D, JsonSpectrum2D, read_spectrum1d_json, read_spectrum2d_json,
    write_spectrum1d_json, write_spectrum2d_json,
};
pub use traits::{SpectrumReader, SpectrumWriter};
