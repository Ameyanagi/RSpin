//! Nucleus identifiers.

use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::{RSpinError, Result};

/// Nucleus labels commonly used in NMR spectra.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Nucleus {
    /// Proton.
    Hydrogen1,
    /// Carbon-13.
    Carbon13,
    /// Nitrogen-15.
    Nitrogen15,
    /// Fluorine-19.
    Fluorine19,
    /// Phosphorus-31.
    Phosphorus31,
    /// Any nucleus not covered by a dedicated variant.
    Other(String),
}

impl Nucleus {
    /// Returns the canonical label.
    #[must_use]
    pub fn as_label(&self) -> &str {
        match self {
            Self::Hydrogen1 => "1H",
            Self::Carbon13 => "13C",
            Self::Nitrogen15 => "15N",
            Self::Fluorine19 => "19F",
            Self::Phosphorus31 => "31P",
            Self::Other(label) => label.as_str(),
        }
    }
}

impl fmt::Display for Nucleus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_label())
    }
}

impl FromStr for Nucleus {
    type Err = RSpinError;

    fn from_str(value: &str) -> Result<Self> {
        let normalized = value.trim();
        match normalized {
            "1H" | "H1" | "H" => Ok(Self::Hydrogen1),
            "13C" | "C13" | "C" => Ok(Self::Carbon13),
            "15N" | "N15" | "N" => Ok(Self::Nitrogen15),
            "19F" | "F19" | "F" => Ok(Self::Fluorine19),
            "31P" | "P31" | "P" => Ok(Self::Phosphorus31),
            "" => Err(RSpinError::Parse {
                format: "nucleus",
                message: "empty nucleus label".to_owned(),
            }),
            other => Ok(Self::Other(other.to_owned())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_common_nuclei() {
        assert_eq!("1H".parse::<Nucleus>(), Ok(Nucleus::Hydrogen1));
        assert_eq!("C13".parse::<Nucleus>(), Ok(Nucleus::Carbon13));
        assert_eq!("19F".parse::<Nucleus>(), Ok(Nucleus::Fluorine19));
    }

    #[test]
    fn displays_canonical_labels() {
        assert_eq!(Nucleus::Phosphorus31.to_string(), "31P");
        assert_eq!(Nucleus::Other("7Li".to_owned()).to_string(), "7Li");
    }
}
