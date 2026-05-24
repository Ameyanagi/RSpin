//! Spectrum input and output.

mod agilent;
mod analysis_csv;
mod analysis_json;
mod bruker;
mod csv;
mod csv_2d;
mod csv_common;
mod jcamp;
mod json;
mod processing_json;
mod traits;

pub use agilent::{AgilentFid1D, read_agilent_fid_1d_dir};
pub use analysis_csv::{CsvAnalysis1D, CsvAnalysis2D, write_analysis1d_csv, write_analysis2d_csv};
pub use analysis_json::{
    JsonAnalysis1D, JsonAnalysis2D, read_analysis1d_json, read_analysis2d_json,
    write_analysis1d_json, write_analysis2d_json,
};
pub use bruker::{BrukerProcessed1D, read_bruker_processed_1d_dir};
pub use csv::{CsvSpectrum1D, read_spectrum1d_csv, write_spectrum1d_csv};
pub use csv_2d::{CsvSpectrum2D, read_spectrum2d_csv, write_spectrum2d_csv};
pub use jcamp::{JcampDx, read_jcamp_dx_1d, write_jcamp_dx_1d};
pub use json::{
    JsonSpectrum1D, JsonSpectrum2D, read_spectrum1d_json, read_spectrum2d_json,
    write_spectrum1d_json, write_spectrum2d_json,
};
pub use processing_json::{
    JsonProcessingRecipe1D, JsonProcessingRecipe2D, read_processing_recipe_1d_json,
    read_processing_recipe_2d_json, write_processing_recipe_1d_json,
    write_processing_recipe_2d_json,
};
pub use traits::{SpectrumReader, SpectrumWriter};
