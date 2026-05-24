use rspin_core::Unit;

use super::*;

#[test]
fn reads_compressed_complex64_2d_fid() -> Result<()> {
    let input = r#"
        <nmrML version="v1.0.rc1" id="two-d">
          <acquisition>
            <acquisitionMultiD>
              <acquisitionParameterSet numberOfScans="1" numberOfSteadyStateScans="0">
                <sampleAcquisitionTemperature value="25.0" unitName="degree celsius"/>
                <directDimensionParameterSet decoupled="false" numberOfDataPoints="2">
                  <acquisitionNucleus cvRef="NMR" accession="NMR:1400151" name="1H"/>
                  <irradiationFrequency value="600.0" unitName="megaHertz"/>
                  <sweepWidth value="2.0" unitName="hertz"/>
                </directDimensionParameterSet>
                <indirectDimensionParameterSet decoupled="false" numberOfDataPoints="2">
                  <acquisitionNucleus cvRef="NMR" accession="NMR:1400154" name="13C"/>
                  <irradiationFrequency value="150.0" unitName="megaHertz"/>
                  <sweepWidth value="4.0" unitName="hertz"/>
                </indirectDimensionParameterSet>
              </acquisitionParameterSet>
              <fidData compressed="true" encodedLength="44" byteFormat="complex64">
                eJxjYGiwZ2Bo2M/AwOAAxAeAFJB2ANINQLrhAABd6gZ/
              </fidData>
            </acquisitionMultiD>
          </acquisition>
        </nmrML>
    "#;

    let spectrum = read_nmrml_2d_str(input)?;

    assert_eq!(spectrum.shape(), (2, 2));
    assert_eq!(spectrum.x.unit, Unit::Seconds);
    assert_eq!(spectrum.y.unit, Unit::Seconds);
    assert_eq!(spectrum.x.values, vec![0.0, 0.5]);
    assert_eq!(spectrum.y.values, vec![0.0, 0.25]);
    assert_eq!(spectrum.z, vec![1.0, 2.0, 3.0, 4.0]);
    assert_eq!(spectrum.imaginary, Some(vec![-1.0, -2.0, -3.0, -4.0]));
    assert_eq!(spectrum.metadata.name.as_deref(), Some("two-d"));
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(spectrum.metadata.frequency_mhz, Some(600.0));
    assert_eq!(spectrum.metadata.temperature_k, Some(298.15));
    Ok(())
}

#[test]
fn reads_processed_float64_2d_spectrum() -> Result<()> {
    let input = r#"
        <nmrML version="v1.0.rc1">
          <acquisition>
            <acquisitionMultiD>
              <acquisitionParameterSet numberOfScans="1" numberOfSteadyStateScans="0">
                <sampleAcquisitionTemperature value="298.15" unitName="kelvin"/>
                <directDimensionParameterSet decoupled="false" numberOfDataPoints="3">
                  <acquisitionNucleus cvRef="NMR" accession="NMR:1400151" name="1H"/>
                  <irradiationFrequency value="600.0" unitName="megaHertz"/>
                </directDimensionParameterSet>
                <indirectDimensionParameterSet decoupled="false" numberOfDataPoints="2">
                  <acquisitionNucleus cvRef="NMR" accession="NMR:1400154" name="13C"/>
                  <irradiationFrequency value="150.0" unitName="megaHertz"/>
                </indirectDimensionParameterSet>
              </acquisitionParameterSet>
            </acquisitionMultiD>
          </acquisition>
          <spectrumList>
            <spectrumMultiD id="processed" name="Processed 2D" numberOfDataPoints="6">
              <spectrumDataArray compressed="false" encodedLength="64" byteFormat="float64">
                AAAAAAAA8D8AAAAAAAAAQAAAAAAAAAhAAAAAAAAAEEAAAAAAAAAUQAAAAAAAABhA
              </spectrumDataArray>
              <xAxis unitName="parts per million" startValue="10.0" endValue="8.0"/>
              <firstDimensionProcessingParameterSet/>
              <higherDimensionProcessingParameterSet/>
            </spectrumMultiD>
          </spectrumList>
        </nmrML>
    "#;

    let spectrum = read_nmrml_2d_str(input)?;

    assert_eq!(spectrum.shape(), (3, 2));
    assert_eq!(spectrum.x.unit, Unit::Ppm);
    assert_eq!(spectrum.x.values, vec![10.0, 9.0, 8.0]);
    assert_eq!(spectrum.y.unit, Unit::Points);
    assert_eq!(spectrum.y.values, vec![0.0, 1.0]);
    assert_eq!(spectrum.z, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    assert_eq!(spectrum.value_at(2, 1), Some(6.0));
    assert_eq!(spectrum.imaginary, None);
    assert_eq!(spectrum.metadata.name.as_deref(), Some("Processed 2D"));
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(spectrum.metadata.frequency_mhz, Some(600.0));
    assert_eq!(spectrum.metadata.temperature_k, Some(298.15));
    Ok(())
}

#[test]
fn rejects_processed_length_mismatch() {
    let input = r#"
        <nmrML version="v1.0.rc1">
          <acquisition>
            <acquisitionMultiD>
              <acquisitionParameterSet>
                <directDimensionParameterSet decoupled="false" numberOfDataPoints="3"/>
                <indirectDimensionParameterSet decoupled="false" numberOfDataPoints="2"/>
              </acquisitionParameterSet>
            </acquisitionMultiD>
          </acquisition>
          <spectrumList>
            <spectrumMultiD id="processed" numberOfDataPoints="5">
              <spectrumDataArray compressed="false" encodedLength="64" byteFormat="float64">
                AAAAAAAA8D8AAAAAAAAAQAAAAAAAAAhAAAAAAAAAEEAAAAAAAAAUQAAAAAAAABhA
              </spectrumDataArray>
              <xAxis unitName="parts per million" startValue="10.0" endValue="8.0"/>
              <firstDimensionProcessingParameterSet/>
              <higherDimensionProcessingParameterSet/>
            </spectrumMultiD>
          </spectrumList>
        </nmrML>
    "#;

    let error = read_nmrml_2d_str(input).expect_err("declared point mismatch should fail");

    assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));
}

#[test]
fn rejects_length_mismatch() {
    let input = r#"
        <nmrML version="v1.0.rc1">
          <acquisition>
            <acquisitionMultiD>
              <acquisitionParameterSet>
                <directDimensionParameterSet decoupled="false" numberOfDataPoints="3"/>
                <indirectDimensionParameterSet decoupled="false" numberOfDataPoints="2"/>
              </acquisitionParameterSet>
              <fidData compressed="true" encodedLength="44" byteFormat="complex64">
                eJxjYGiwZ2Bo2M/AwOAAxAeAFJB2ANINQLrhAABd6gZ/
              </fidData>
            </acquisitionMultiD>
          </acquisition>
        </nmrML>
    "#;

    let error = read_nmrml_2d_str(input).expect_err("dimension mismatch should fail");

    assert!(matches!(error, RSpinError::InvalidSpectrum { .. }));
}
