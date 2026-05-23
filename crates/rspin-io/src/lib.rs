//! Spectrum input and output.

mod jcamp;
mod traits;

pub use jcamp::{JcampDx, read_jcamp_dx_1d, write_jcamp_dx_1d};
pub use traits::{SpectrumReader, SpectrumWriter};
