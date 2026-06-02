//! Chainable source shortcut filters for the bundle loader.

use super::SpectrumBundleLoader;
use crate::bundle::{LoadedSourceFormat, LoadedSourceVendor};

impl SpectrumBundleLoader {
    /// Restricts loading to Bruker source formats.
    #[must_use]
    pub fn bruker(self) -> Self {
        self.only_source_vendor(LoadedSourceVendor::Bruker)
    }

    /// Restricts loading to JEOL Delta source formats.
    #[must_use]
    pub fn jeol(self) -> Self {
        self.only_source_vendor(LoadedSourceVendor::Jeol)
    }

    /// Restricts loading to Agilent/Varian source formats.
    #[must_use]
    pub fn agilent_varian(self) -> Self {
        self.only_source_vendor(LoadedSourceVendor::AgilentVarian)
    }

    /// Restricts loading to Agilent/Varian source formats.
    #[must_use]
    pub fn agilent(self) -> Self {
        self.agilent_varian()
    }

    /// Restricts loading to Agilent/Varian source formats.
    #[must_use]
    pub fn varian(self) -> Self {
        self.agilent_varian()
    }

    /// Restricts loading to JCAMP-DX source files.
    #[must_use]
    pub fn jcamp_dx(self) -> Self {
        self.only_source_format(LoadedSourceFormat::JcampDx)
    }

    /// Restricts loading to JCAMP-DX source files.
    #[must_use]
    pub fn jcamp(self) -> Self {
        self.jcamp_dx()
    }

    /// Restricts loading to nmrML source files.
    #[must_use]
    pub fn nmrml(self) -> Self {
        self.only_source_format(LoadedSourceFormat::NmrMl)
    }

    /// Restricts loading to `RSpin` JSON source files.
    #[must_use]
    pub fn json(self) -> Self {
        self.only_source_format(LoadedSourceFormat::Json)
    }

    /// Restricts loading to `RSpin` CSV source files.
    #[must_use]
    pub fn csv(self) -> Self {
        self.only_source_format(LoadedSourceFormat::Csv)
    }
}
