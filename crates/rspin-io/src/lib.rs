//! Spectrum input and output.

mod agilent;
mod analysis_csv;
mod analysis_json;
mod assignment_json;
mod auto;
mod bruker;
mod bundle;
mod bundle_json;
mod csv;
mod csv_2d;
mod csv_common;
mod jcamp;
mod jeol;
mod json;
mod nmredata;
mod nmredata_json;
mod nmrml;
mod nmrml_2d;
mod nmrml_2d_writer;
mod nmrml_info;
mod nmrml_writer;
mod prediction_csv;
mod prediction_json;
mod processing_json;
mod simulation_csv;
mod simulation_json;
mod traits;

pub use agilent::{
    AgilentBinaryFileInfo, AgilentFid1D, AgilentFid2D, AgilentProcessed1D, AgilentProcessed2D,
    AgilentProcparInfo, inspect_agilent_binary_bytes, inspect_agilent_binary_file,
    inspect_agilent_procpar, read_agilent_arrayed_fid_1d_bytes, read_agilent_arrayed_fid_1d_dir,
    read_agilent_arrayed_fid_2d_bytes, read_agilent_arrayed_fid_2d_dir, read_agilent_fid_1d_bytes,
    read_agilent_fid_1d_dir, read_agilent_fid_2d_bytes, read_agilent_fid_2d_dir,
    read_agilent_processed_1d_bytes, read_agilent_processed_1d_dir,
    read_agilent_processed_2d_bytes, read_agilent_processed_2d_dir,
};
pub use analysis_csv::{CsvAnalysis1D, CsvAnalysis2D, write_analysis1d_csv, write_analysis2d_csv};
pub use analysis_json::{
    ANALYSIS_1D_JSON_FORMAT, ANALYSIS_2D_JSON_FORMAT, ANALYSIS_JSON_VERSION, JsonAnalysis1D,
    JsonAnalysis2D, read_analysis1d_json, read_analysis2d_json, write_analysis1d_json,
    write_analysis2d_json,
};
pub use assignment_json::{
    ASSIGNMENT_JSON_VERSION, ASSIGNMENT_SET_JSON_FORMAT, J_COUPLING_GRAPH_JSON_FORMAT,
    JsonAssignmentSet, JsonJCouplingGraph, read_assignment_set_json, read_j_coupling_graph_json,
    write_assignment_set_json, write_j_coupling_graph_json,
};
pub use auto::{
    AutoSpectrum1DPath, AutoSpectrum1DPathWriter, AutoSpectrum1DText, AutoSpectrum2DPath,
    AutoSpectrum2DPathWriter, AutoSpectrum2DText, Spectrum1DBytes, Spectrum1DBytesFormat,
    Spectrum1DPathFormat, Spectrum1DTextWriter, Spectrum1DWriteFormat, Spectrum1DWritePathFormat,
    Spectrum2DBytes, Spectrum2DBytesFormat, Spectrum2DPathFormat, Spectrum2DTextWriter,
    Spectrum2DWriteFormat, Spectrum2DWritePathFormat, SpectrumTextFormat,
    detect_spectrum_text_format, detect_spectrum1d_path_format,
    detect_spectrum1d_write_path_format, detect_spectrum2d_path_format,
    detect_spectrum2d_write_path_format, parse_spectrum_text_format, parse_spectrum1d_bytes_format,
    parse_spectrum1d_path_format, parse_spectrum1d_write_format,
    parse_spectrum1d_write_path_format, parse_spectrum2d_bytes_format,
    parse_spectrum2d_path_format, parse_spectrum2d_write_format,
    parse_spectrum2d_write_path_format, read_spectrum1d_bytes_as, read_spectrum1d_path,
    read_spectrum1d_path_as, read_spectrum1d_text, read_spectrum1d_text_as,
    read_spectrum2d_bytes_as, read_spectrum2d_path, read_spectrum2d_path_as, read_spectrum2d_text,
    read_spectrum2d_text_as, write_spectrum1d_path, write_spectrum1d_path_as,
    write_spectrum1d_text, write_spectrum2d_path, write_spectrum2d_path_as, write_spectrum2d_text,
};
pub use bruker::{
    BrukerFid1D, BrukerFid1DBytes, BrukerParameterFileInfo, BrukerProcessed1D,
    BrukerProcessed1DBytes, BrukerProcessed2D, BrukerProcessed2DBytes, BrukerSer2D,
    BrukerSer2DBytes, inspect_bruker_parameter_file, read_bruker_fid_1d_bytes,
    read_bruker_fid_1d_dir, read_bruker_processed_1d_bytes, read_bruker_processed_1d_dir,
    read_bruker_processed_2d_bytes, read_bruker_processed_2d_dir, read_bruker_ser_2d_bytes,
    read_bruker_ser_2d_dir,
};
pub use bundle::{
    LoadWarning, LoadedSource, LoadedSpectrum, RSpinReader, SpectrumBundle, SpectrumBundleLoader,
    load_spectra, load_spectra_many, load_spectra_many_relative_to, load_spectrum_1d,
    load_spectrum_1d_many, load_spectrum_1d_many_relative_to, load_spectrum_2d,
    load_spectrum_2d_many, load_spectrum_2d_many_relative_to,
};
pub use bundle_json::{
    JsonSpectrumBundle, SPECTRUM_BUNDLE_JSON_FORMAT, SPECTRUM_BUNDLE_JSON_VERSION,
    read_spectrum_bundle_json, read_spectrum_bundle_json_file, write_spectrum_bundle_json,
};
pub use csv::{CsvSpectrum1D, read_spectrum1d_csv, write_spectrum1d_csv};
pub use csv_2d::{CsvSpectrum2D, read_spectrum2d_csv, write_spectrum2d_csv};
pub use jcamp::{
    JcampDx, JcampDx2D, JcampDxVersion, parse_jcamp_dx_version, read_jcamp_dx_1d, read_jcamp_dx_2d,
    write_jcamp_dx_1d, write_jcamp_dx_2d,
};
pub use jeol::{
    JeolJdf1D, JeolJdf2D, JeolJdfInfo, JeolJdfVersion, inspect_jeol_jdf_bytes,
    inspect_jeol_jdf_file, read_jeol_jdf_1d_bytes, read_jeol_jdf_1d_file, read_jeol_jdf_2d_bytes,
    read_jeol_jdf_2d_file,
};
pub use json::{
    JsonSpectrum1D, JsonSpectrum2D, SPECTRUM_1D_JSON_FORMAT, SPECTRUM_2D_JSON_FORMAT,
    SPECTRUM_JSON_VERSION, read_spectrum1d_json, read_spectrum2d_json, write_spectrum1d_json,
    write_spectrum2d_json,
};
pub use nmredata::{
    NmreData, NmreDataAnalysis, NmreDataAssignment, NmreDataCoupling, NmreDataRecord,
    NmreDataRecords, NmreDataSignal1D, NmreDataSignal2D, NmreDataSpectrum, NmreDataSpectrumKind,
    NmreDataTag, NmreDataVersion, nmredata_1d_signals_to_assignment_set,
    nmredata_2d_signal_zone_id, nmredata_2d_signals_to_assignment_set,
    nmredata_assignments_to_assignment_set, nmredata_couplings_to_j_coupling_graph,
    nmredata_to_analysis, parse_nmredata_version, read_nmredata_bytes, read_nmredata_file,
    read_nmredata_records_bytes, read_nmredata_records_file, read_nmredata_records_str,
    read_nmredata_str, write_nmredata_file, write_nmredata_record, write_nmredata_records,
    write_nmredata_records_file,
};
pub use nmredata_json::{
    JsonNmreDataRecord, JsonNmreDataRecords, NMREDATA_JSON_VERSION, NMREDATA_RECORD_JSON_FORMAT,
    NMREDATA_RECORDS_JSON_FORMAT, read_nmredata_record_json, read_nmredata_records_json,
    write_nmredata_record_json, write_nmredata_records_json,
};
pub use nmrml::{NmrMl1D, read_nmrml_1d_bytes, read_nmrml_1d_file, read_nmrml_1d_str};
pub use nmrml_2d::{NmrMl2D, read_nmrml_2d_bytes, read_nmrml_2d_file, read_nmrml_2d_str};
pub use nmrml_2d_writer::{write_nmrml_2d, write_nmrml_2d_file};
pub use nmrml_info::{
    NMRML_SCHEMA_DIRECTORY, NMRML_SCHEMA_REPOSITORY, NmrMlDocumentInfo, NmrMlSchemaLocation,
    NmrMlVersion, parse_nmrml_version, read_nmrml_document_info_bytes,
    read_nmrml_document_info_file, read_nmrml_document_info_str,
};
pub use nmrml_writer::{write_nmrml_1d, write_nmrml_1d_file};
pub use prediction_csv::{CsvPrediction, read_prediction_csv, write_prediction_csv};
pub use prediction_json::{
    JsonPrediction, PREDICTION_JSON_FORMAT, PREDICTION_JSON_VERSION, read_prediction_json,
    write_prediction_json,
};
pub use processing_json::{
    JsonProcessingRecipe1D, JsonProcessingRecipe2D, PROCESSING_RECIPE_1D_FORMAT,
    PROCESSING_RECIPE_2D_FORMAT, PROCESSING_RECIPE_JSON_VERSION, read_processing_recipe_1d_json,
    read_processing_recipe_2d_json, write_processing_recipe_1d_json,
    write_processing_recipe_2d_json,
};
pub use simulation_csv::{
    CsvExactTransitions, read_exact_transitions_csv, write_exact_transitions_csv,
};
pub use simulation_json::{
    EXACT_DECOMPOSITION_1D_JSON_FORMAT, EXACT_DECOMPOSITION_2D_JSON_FORMAT,
    EXACT_SPECTRUM_1D_OPTIONS_JSON_FORMAT, EXACT_SPECTRUM_2D_OPTIONS_JSON_FORMAT,
    EXACT_SPIN_OPTIONS_JSON_FORMAT, EXACT_TRANSITIONS_JSON_FORMAT, JsonExactDecomposition1D,
    JsonExactDecomposition2D, JsonExactSpectrum2DOptions, JsonExactSpectrumOptions,
    JsonExactSpinOptions, JsonExactTransitions, JsonSpinHalfSystem, SIMULATION_JSON_VERSION,
    SPIN_HALF_SYSTEM_JSON_FORMAT, read_exact_decomposition_1d_json,
    read_exact_decomposition_2d_json, read_exact_spectrum_2d_options_json,
    read_exact_spectrum_options_json, read_exact_spin_options_json, read_exact_transitions_json,
    read_spin_half_system_json, write_exact_decomposition_1d_json,
    write_exact_decomposition_2d_json, write_exact_spectrum_2d_options_json,
    write_exact_spectrum_options_json, write_exact_spin_options_json, write_exact_transitions_json,
    write_spin_half_system_json,
};
pub use traits::{SpectrumPathReader, SpectrumPathWriter, SpectrumReader, SpectrumWriter};
