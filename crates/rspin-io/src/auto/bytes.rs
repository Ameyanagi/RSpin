//! Explicit byte-format routing for uploaded or in-memory spectra.

use std::{fmt, str::FromStr};

use rspin_core::{RSpinError, Result, Spectrum1D, Spectrum2D};

use crate::{
    read_agilent_fid_1d_bytes, read_agilent_fid_2d_bytes, read_agilent_processed_1d_bytes,
    read_agilent_processed_2d_bytes, read_bruker_fid_1d_bytes, read_bruker_processed_1d_bytes,
    read_bruker_processed_2d_bytes, read_bruker_ser_2d_bytes, read_jeol_jdf_1d_bytes,
    read_jeol_jdf_2d_bytes,
};

use super::normalized_format_name;

/// Byte-oriented one-dimensional spectrum formats.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spectrum1DBytesFormat {
    /// JEOL Delta `.jdf` payload.
    JeolJdf,
    /// Bruker processed `1r` payload with `procs` text.
    BrukerProcessed,
    /// Bruker raw `fid` payload with `acqus` text.
    BrukerFid,
    /// Agilent/Varian processed `phasefile` payload with `procpar` text.
    AgilentProcessed,
    /// Agilent/Varian raw `fid` payload with `procpar` text.
    AgilentFid,
}

impl Spectrum1DBytesFormat {
    /// Returns the canonical snake-case format name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::JeolJdf => "jeol_jdf",
            Self::BrukerProcessed => "bruker_processed",
            Self::BrukerFid => "bruker_fid",
            Self::AgilentProcessed => "agilent_processed",
            Self::AgilentFid => "agilent_fid",
        }
    }
}

impl fmt::Display for Spectrum1DBytesFormat {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for Spectrum1DBytesFormat {
    type Err = RSpinError;

    fn from_str(input: &str) -> Result<Self> {
        parse_spectrum1d_bytes_format(input)
    }
}

/// Byte-oriented two-dimensional spectrum formats.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spectrum2DBytesFormat {
    /// JEOL Delta `.jdf` payload.
    JeolJdf,
    /// Bruker processed `2rr` payload with `procs` and `proc2s` text.
    BrukerProcessed,
    /// Bruker raw `ser` payload with `acqus` and `acqu2s` text.
    BrukerSer,
    /// Agilent/Varian processed `phasefile` payload with `procpar` text.
    AgilentProcessed,
    /// Agilent/Varian raw `fid` payload with `procpar` text.
    AgilentFid,
}

impl Spectrum2DBytesFormat {
    /// Returns the canonical snake-case format name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::JeolJdf => "jeol_jdf",
            Self::BrukerProcessed => "bruker_processed",
            Self::BrukerSer => "bruker_ser",
            Self::AgilentProcessed => "agilent_processed",
            Self::AgilentFid => "agilent_fid",
        }
    }
}

impl fmt::Display for Spectrum2DBytesFormat {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for Spectrum2DBytesFormat {
    type Err = RSpinError;

    fn from_str(input: &str) -> Result<Self> {
        parse_spectrum2d_bytes_format(input)
    }
}

/// Chainable reader for one-dimensional in-memory spectrum payloads.
#[derive(Clone, Copy, Debug)]
pub struct Spectrum1DBytes<'a> {
    format: Spectrum1DBytesFormat,
    data_bytes: &'a [u8],
    parameters: Option<&'a str>,
}

impl<'a> Spectrum1DBytes<'a> {
    /// Creates a byte reader for an explicit one-dimensional format.
    #[must_use]
    pub fn new(format: Spectrum1DBytesFormat, data_bytes: &'a [u8]) -> Self {
        Self {
            format,
            data_bytes,
            parameters: None,
        }
    }

    /// Attaches the format-specific parameter text.
    #[must_use]
    pub fn with_parameters(mut self, parameters: &'a str) -> Self {
        self.parameters = Some(parameters);
        self
    }

    /// Attaches optional format-specific parameter text.
    #[must_use]
    pub fn with_optional_parameters(mut self, parameters: Option<&'a str>) -> Self {
        self.parameters = parameters;
        self
    }

    /// Attaches Agilent/Varian `procpar` text.
    #[must_use]
    pub fn with_procpar(self, procpar: &'a str) -> Self {
        self.with_parameters(procpar)
    }

    /// Attaches Bruker `procs` text.
    #[must_use]
    pub fn with_procs(self, procs: &'a str) -> Self {
        self.with_parameters(procs)
    }

    /// Attaches Bruker `acqus` text.
    #[must_use]
    pub fn with_acqus(self, acqus: &'a str) -> Self {
        self.with_parameters(acqus)
    }

    /// Reads the byte payload into a one-dimensional spectrum.
    ///
    /// # Errors
    ///
    /// Returns an error when required parameter text is missing or the selected
    /// format reader rejects the payload.
    pub fn read(self) -> Result<Spectrum1D> {
        match self.format {
            Spectrum1DBytesFormat::JeolJdf => read_jeol_jdf_1d_bytes(self.data_bytes),
            Spectrum1DBytesFormat::BrukerProcessed => read_bruker_processed_1d_bytes(
                required_parameters(self.parameters, "Bruker processed 1D procs")?,
                self.data_bytes,
            ),
            Spectrum1DBytesFormat::BrukerFid => read_bruker_fid_1d_bytes(
                required_parameters(self.parameters, "Bruker raw 1D acqus")?,
                self.data_bytes,
            ),
            Spectrum1DBytesFormat::AgilentProcessed => read_agilent_processed_1d_bytes(
                required_parameters(self.parameters, "Agilent processed 1D procpar")?,
                self.data_bytes,
            ),
            Spectrum1DBytesFormat::AgilentFid => read_agilent_fid_1d_bytes(
                required_parameters(self.parameters, "Agilent raw 1D procpar")?,
                self.data_bytes,
            ),
        }
    }
}

/// Chainable reader for two-dimensional in-memory spectrum payloads.
#[derive(Clone, Copy, Debug)]
pub struct Spectrum2DBytes<'a> {
    format: Spectrum2DBytesFormat,
    data_bytes: &'a [u8],
    parameters: Option<&'a str>,
    indirect_parameters: Option<&'a str>,
}

impl<'a> Spectrum2DBytes<'a> {
    /// Creates a byte reader for an explicit two-dimensional format.
    #[must_use]
    pub fn new(format: Spectrum2DBytesFormat, data_bytes: &'a [u8]) -> Self {
        Self {
            format,
            data_bytes,
            parameters: None,
            indirect_parameters: None,
        }
    }

    /// Attaches the primary format-specific parameter text.
    #[must_use]
    pub fn with_parameters(mut self, parameters: &'a str) -> Self {
        self.parameters = Some(parameters);
        self
    }

    /// Attaches optional primary format-specific parameter text.
    #[must_use]
    pub fn with_optional_parameters(mut self, parameters: Option<&'a str>) -> Self {
        self.parameters = parameters;
        self
    }

    /// Attaches secondary Bruker dimension parameter text.
    #[must_use]
    pub fn with_indirect_parameters(mut self, parameters: &'a str) -> Self {
        self.indirect_parameters = Some(parameters);
        self
    }

    /// Attaches optional secondary Bruker dimension parameter text.
    #[must_use]
    pub fn with_optional_indirect_parameters(mut self, parameters: Option<&'a str>) -> Self {
        self.indirect_parameters = parameters;
        self
    }

    /// Attaches Agilent/Varian `procpar` text.
    #[must_use]
    pub fn with_procpar(self, procpar: &'a str) -> Self {
        self.with_parameters(procpar)
    }

    /// Attaches Bruker direct-dimension `procs` or `acqus` text.
    #[must_use]
    pub fn with_direct_parameters(self, parameters: &'a str) -> Self {
        self.with_parameters(parameters)
    }

    /// Attaches Bruker `procs` text.
    #[must_use]
    pub fn with_procs(self, procs: &'a str) -> Self {
        self.with_parameters(procs)
    }

    /// Attaches Bruker `proc2s` text.
    #[must_use]
    pub fn with_proc2s(self, proc2s: &'a str) -> Self {
        self.with_indirect_parameters(proc2s)
    }

    /// Attaches Bruker `acqus` text.
    #[must_use]
    pub fn with_acqus(self, acqus: &'a str) -> Self {
        self.with_parameters(acqus)
    }

    /// Attaches Bruker `acqu2s` text.
    #[must_use]
    pub fn with_acqu2s(self, acqu2s: &'a str) -> Self {
        self.with_indirect_parameters(acqu2s)
    }

    /// Reads the byte payload into a two-dimensional spectrum.
    ///
    /// # Errors
    ///
    /// Returns an error when required parameter text is missing or the selected
    /// format reader rejects the payload.
    pub fn read(self) -> Result<Spectrum2D> {
        match self.format {
            Spectrum2DBytesFormat::JeolJdf => read_jeol_jdf_2d_bytes(self.data_bytes),
            Spectrum2DBytesFormat::BrukerProcessed => read_bruker_processed_2d_bytes(
                required_parameters(self.parameters, "Bruker processed 2D procs")?,
                required_parameters(self.indirect_parameters, "Bruker processed 2D proc2s")?,
                self.data_bytes,
            ),
            Spectrum2DBytesFormat::BrukerSer => read_bruker_ser_2d_bytes(
                required_parameters(self.parameters, "Bruker raw 2D acqus")?,
                required_parameters(self.indirect_parameters, "Bruker raw 2D acqu2s")?,
                self.data_bytes,
            ),
            Spectrum2DBytesFormat::AgilentProcessed => read_agilent_processed_2d_bytes(
                required_parameters(self.parameters, "Agilent processed 2D procpar")?,
                self.data_bytes,
            ),
            Spectrum2DBytesFormat::AgilentFid => read_agilent_fid_2d_bytes(
                required_parameters(self.parameters, "Agilent raw 2D procpar")?,
                self.data_bytes,
            ),
        }
    }
}

/// Parses a one-dimensional byte format name.
///
/// Accepted names include `jeol_jdf`, `jdf`, `bruker_processed`,
/// `bruker_fid`, `agilent_processed`, `agilent_fid`, `varian_processed`, and
/// `varian_fid`.
///
/// # Errors
///
/// Returns an unsupported-feature error when `input` is not a supported
/// one-dimensional byte format name.
pub fn parse_spectrum1d_bytes_format(input: &str) -> Result<Spectrum1DBytesFormat> {
    match normalized_format_name(input).as_str() {
        "jeoljdf" | "jeol" | "jdf" => Ok(Spectrum1DBytesFormat::JeolJdf),
        "brukerprocessed" | "brukerpdata" | "bruker1r" => {
            Ok(Spectrum1DBytesFormat::BrukerProcessed)
        }
        "brukerfid" | "brukerraw" => Ok(Spectrum1DBytesFormat::BrukerFid),
        "agilentprocessed" | "varianprocessed" | "agilentphasefile" | "varianphasefile" => {
            Ok(Spectrum1DBytesFormat::AgilentProcessed)
        }
        "agilentfid" | "varianfid" => Ok(Spectrum1DBytesFormat::AgilentFid),
        _ => Err(RSpinError::Unsupported {
            feature: "one-dimensional spectrum byte format name",
        }),
    }
}

/// Parses a two-dimensional byte format name.
///
/// Accepted names include `jeol_jdf`, `jdf`, `bruker_processed`,
/// `bruker_ser`, `agilent_processed`, `agilent_fid`, `varian_processed`, and
/// `varian_fid`.
///
/// # Errors
///
/// Returns an unsupported-feature error when `input` is not a supported
/// two-dimensional byte format name.
pub fn parse_spectrum2d_bytes_format(input: &str) -> Result<Spectrum2DBytesFormat> {
    match normalized_format_name(input).as_str() {
        "jeoljdf" | "jeol" | "jdf" => Ok(Spectrum2DBytesFormat::JeolJdf),
        "brukerprocessed" | "brukerpdata" | "bruker2rr" => {
            Ok(Spectrum2DBytesFormat::BrukerProcessed)
        }
        "brukerser" | "ser" | "brukerraw" => Ok(Spectrum2DBytesFormat::BrukerSer),
        "agilentprocessed" | "varianprocessed" | "agilentphasefile" | "varianphasefile" => {
            Ok(Spectrum2DBytesFormat::AgilentProcessed)
        }
        "agilentfid" | "varianfid" => Ok(Spectrum2DBytesFormat::AgilentFid),
        _ => Err(RSpinError::Unsupported {
            feature: "two-dimensional spectrum byte format name",
        }),
    }
}

/// Reads a one-dimensional spectrum from bytes using an explicit format.
///
/// Parameter text is required for Bruker and Agilent/Varian formats. Use
/// [`Spectrum1DBytes`] when chainable setup reads more clearly.
///
/// # Errors
///
/// Returns an error when required parameter text is missing or the selected
/// format reader rejects the payload.
pub fn read_spectrum1d_bytes_as(
    data_bytes: &[u8],
    format: Spectrum1DBytesFormat,
    parameters: Option<&str>,
) -> Result<Spectrum1D> {
    Spectrum1DBytes::new(format, data_bytes)
        .with_optional_parameters(parameters)
        .read()
}

/// Reads a two-dimensional spectrum from bytes using an explicit format.
///
/// Primary parameter text is required for Bruker and Agilent/Varian formats.
/// Bruker two-dimensional formats also require indirect-dimension parameter
/// text. Use [`Spectrum2DBytes`] when chainable setup reads more clearly.
///
/// # Errors
///
/// Returns an error when required parameter text is missing or the selected
/// format reader rejects the payload.
pub fn read_spectrum2d_bytes_as(
    data_bytes: &[u8],
    format: Spectrum2DBytesFormat,
    parameters: Option<&str>,
    indirect_parameters: Option<&str>,
) -> Result<Spectrum2D> {
    Spectrum2DBytes::new(format, data_bytes)
        .with_optional_parameters(parameters)
        .with_optional_indirect_parameters(indirect_parameters)
        .read()
}

fn required_parameters<'a>(
    parameters: Option<&'a str>,
    description: &'static str,
) -> Result<&'a str> {
    parameters.ok_or_else(|| RSpinError::Parse {
        format: "spectrum bytes",
        message: format!("{description} parameter text is required"),
    })
}

#[cfg(test)]
mod tests {
    use rspin_core::{Nucleus, Unit};

    use super::*;

    #[test]
    fn parses_and_displays_byte_format_names() -> Result<()> {
        assert_eq!(
            "jdf".parse::<Spectrum1DBytesFormat>()?,
            Spectrum1DBytesFormat::JeolJdf
        );
        assert_eq!(
            parse_spectrum1d_bytes_format("bruker 1r")?,
            Spectrum1DBytesFormat::BrukerProcessed
        );
        assert_eq!(
            parse_spectrum1d_bytes_format("varian-fid")?,
            Spectrum1DBytesFormat::AgilentFid
        );
        assert_eq!(
            parse_spectrum2d_bytes_format("bruker 2rr")?,
            Spectrum2DBytesFormat::BrukerProcessed
        );
        assert_eq!(
            parse_spectrum2d_bytes_format("ser")?,
            Spectrum2DBytesFormat::BrukerSer
        );
        assert_eq!(
            Spectrum1DBytesFormat::AgilentProcessed.to_string(),
            "agilent_processed"
        );
        assert_eq!(Spectrum2DBytesFormat::JeolJdf.as_str(), "jeol_jdf");

        let error = parse_spectrum1d_bytes_format("unknown")
            .expect_err("unsupported byte format should fail");
        assert!(matches!(error, RSpinError::Unsupported { .. }));
        Ok(())
    }

    #[test]
    fn routes_one_dimensional_bruker_bytes() -> Result<()> {
        let processed = Spectrum1DBytes::new(
            Spectrum1DBytesFormat::BrukerProcessed,
            &i32_bytes(&[2, -4, 6], ByteOrder::Big),
        )
        .with_procs(
            "\
##$SI= 3
##$BYTORDP= 1
##$DTYPP= 0
##$NC_proc= -1
##$OFFSET= 10
##$SW_p= 3000
##$SF= 500
##$AXNUC= <1H>
",
        )
        .read()?;

        assert_eq!(processed.x.unit, Unit::Ppm);
        assert_eq!(processed.x.values, vec![10.0, 7.0, 4.0]);
        assert_eq!(processed.intensities, vec![4.0, -8.0, 12.0]);
        assert_eq!(processed.metadata.nucleus, Some(Nucleus::Hydrogen1));

        let fid = read_spectrum1d_bytes_as(
            &i32_bytes(&[1, -2, 3, -4], ByteOrder::Little),
            Spectrum1DBytesFormat::BrukerFid,
            Some(
                "\
##$TD= 4
##$BYTORDA= 0
##$DTYPA= 0
##$NC= -1
##$SW_h= 1000
##$NUC1= <13C>
##$SFO1= 125.5
",
            ),
        )?;

        assert_eq!(fid.x.unit, Unit::Seconds);
        assert_eq!(fid.x.values, vec![0.0, 0.001]);
        assert_eq!(fid.intensities, vec![2.0, 6.0]);
        assert_eq!(fid.imaginary, Some(vec![-4.0, -8.0]));
        assert_eq!(fid.metadata.nucleus, Some(Nucleus::Carbon13));
        Ok(())
    }

    #[test]
    fn routes_two_dimensional_bruker_bytes() -> Result<()> {
        let processed = read_spectrum2d_bytes_as(
            &i32_bytes(&[1, 2, 3, 4], ByteOrder::Little),
            Spectrum2DBytesFormat::BrukerProcessed,
            Some(
                "\
##$SI= 2
##$BYTORDP= 0
##$DTYPP= 0
##$NC_proc= -1
##$OFFSET= 10
##$SW_p= 2000
##$SF= 500
##$AXNUC= <1H>
",
            ),
            Some(
                "\
##$SI= 2
##$OFFSET= 120
##$SW_p= 2000
##$SF= 100
##$AXNUC= <13C>
",
            ),
        )?;

        assert_eq!(processed.shape(), (2, 2));
        assert_eq!(processed.x.values, vec![10.0, 6.0]);
        assert_eq!(processed.y.values, vec![120.0, 100.0]);
        assert_eq!(processed.z, vec![2.0, 4.0, 6.0, 8.0]);

        let ser = Spectrum2DBytes::new(
            Spectrum2DBytesFormat::BrukerSer,
            &raw_ser_bytes(&[vec![1, 2, 3, 4], vec![5, 6, 7, 8]], ByteOrder::Big),
        )
        .with_acqus(
            "\
##$TD= 4
##$BYTORDA= 1
##$DTYPA= 0
##$NC= -1
##$SW_h= 1000
##$NUC1= <1H>
##$SFO1= 400.25
",
        )
        .with_acqu2s(
            "\
##$TD= 2
##$SW_h= 200
",
        )
        .read()?;

        assert_eq!(ser.shape(), (2, 2));
        assert_eq!(ser.x.values, vec![0.0, 0.001]);
        assert_eq!(ser.y.values, vec![0.0, 0.005]);
        assert_eq!(ser.z, vec![2.0, 6.0, 10.0, 14.0]);
        assert_eq!(ser.imaginary, Some(vec![4.0, 8.0, 12.0, 16.0]));
        Ok(())
    }

    #[test]
    fn rejects_missing_required_byte_parameters() {
        let error = Spectrum2DBytes::new(Spectrum2DBytesFormat::BrukerSer, b"not ser")
            .with_acqus("##$TD= 4\n")
            .read()
            .expect_err("missing Bruker indirect parameters should fail");

        assert!(matches!(error, RSpinError::Parse { .. }));
        assert!(error.to_string().contains("acqu2s"));
    }

    #[derive(Clone, Copy)]
    enum ByteOrder {
        Little,
        Big,
    }

    fn raw_ser_bytes(rows: &[Vec<i32>], byte_order: ByteOrder) -> Vec<u8> {
        let mut bytes = Vec::new();
        for row in rows {
            for value in row {
                bytes.extend_from_slice(&match byte_order {
                    ByteOrder::Little => value.to_le_bytes(),
                    ByteOrder::Big => value.to_be_bytes(),
                });
            }
            let padded_words = 256usize.saturating_sub(row.len());
            bytes.extend(std::iter::repeat_n(0, padded_words * 4));
        }
        bytes
    }

    fn i32_bytes(values: &[i32], byte_order: ByteOrder) -> Vec<u8> {
        values
            .iter()
            .flat_map(|value| match byte_order {
                ByteOrder::Little => value.to_le_bytes(),
                ByteOrder::Big => value.to_be_bytes(),
            })
            .collect::<Vec<_>>()
    }
}
