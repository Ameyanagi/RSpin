//! Error types shared across crates.

use thiserror::Error;

/// Convenient result type for `RSpin` operations.
pub type Result<T> = std::result::Result<T, RSpinError>;

/// Errors returned by `RSpin` crates.
#[derive(Debug, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum RSpinError {
    /// A numeric value was not finite.
    #[error("non-finite value in {field}")]
    NonFinite {
        /// Field or value group that failed validation.
        field: &'static str,
    },

    /// An axis is empty or inconsistent.
    #[error("invalid axis: {message}")]
    InvalidAxis {
        /// Human-readable validation message.
        message: String,
    },

    /// Spectrum data does not match its axes.
    #[error("invalid spectrum data: {message}")]
    InvalidSpectrum {
        /// Human-readable validation message.
        message: String,
    },

    /// The requested operation is not supported.
    #[error("unsupported feature: {feature}")]
    Unsupported {
        /// Name of the unsupported feature.
        feature: &'static str,
    },

    /// Parsing failed.
    #[error("failed to parse {format}: {message}")]
    Parse {
        /// Format being parsed.
        format: &'static str,
        /// Parser message.
        message: String,
    },
}
