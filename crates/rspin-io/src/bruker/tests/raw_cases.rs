use super::*;

#[test]
fn reads_raw_1d_fid_dataset_root() -> anyhow::Result<()> {
    let root = synthetic_dataset("raw")?;
    write_text(
        &root.join("acqus"),
        "\
##$TD= 4
##$BYTORDA= 1
##$DTYPA= 0
##$NC= -1
##$SW_h= 1000
##$NUC1= <13C>
##$SFO1= 125.5
##$SOLVENT= <CDCl3>
##$TE= 300
##$OWNER= <raw fixture>
##$PULPROG= <zg>
",
    )?;
    write_raw_fid(&root, &[1, -2, 3, -4], ByteOrder::Big)?;

    let spectrum = read_bruker_fid_1d_dir(&root)?;

    assert_eq!(spectrum.len(), 2);
    assert_eq!(spectrum.x.unit, Unit::Seconds);
    assert_eq!(spectrum.x.values, vec![0.0, 0.001]);
    assert_eq!(spectrum.intensities, vec![2.0, 6.0]);
    assert_eq!(spectrum.imaginary, Some(vec![-4.0, -8.0]));
    assert_eq!(spectrum.metadata.name.as_deref(), Some("zg"));
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Carbon13));
    assert_eq!(spectrum.metadata.frequency_mhz, Some(125.5));
    assert_eq!(spectrum.metadata.solvent.as_deref(), Some("CDCl3"));
    assert_eq!(spectrum.metadata.temperature_k, Some(300.0));
    assert_eq!(spectrum.metadata.origin.as_deref(), Some("raw fixture"));
    assert_eq!(
        spectrum.metadata.property("bruker.acqus.PULPROG"),
        Some("zg")
    );

    remove_dir(root)?;
    Ok(())
}

#[test]
fn reads_raw_1d_fid_file_path() -> anyhow::Result<()> {
    let root = synthetic_dataset("raw-file")?;
    write_text(
        &root.join("acqus"),
        "\
##$TD= 2
##$BYTORDA= 0
##$DTYPA= 0
",
    )?;
    write_raw_fid(&root, &[5, -7], ByteOrder::Little)?;

    let spectrum = BrukerFid1D.read_path(&root.join("fid"))?;

    assert_eq!(spectrum.x.unit, Unit::Points);
    assert_eq!(spectrum.intensities, vec![5.0]);
    assert_eq!(spectrum.imaginary, Some(vec![-7.0]));

    remove_dir(root)?;
    Ok(())
}

#[test]
fn reads_raw_1d_fid_bytes_without_dataset_path() -> anyhow::Result<()> {
    let spectrum = read_bruker_fid_1d_bytes(
        "\
##$TD= 4
##$BYTORDA= 1
##$DTYPA= 0
##$NC= -1
##$SW_h= 1000
##$NUC1= <13C>
##$SFO1= 125.5
##$SOLVENT= <CDCl3>
##$TE= 300
##$OWNER= <raw bytes fixture>
##$PULPROG= <zg>
",
        &i32_bytes(&[1, -2, 3, -4], ByteOrder::Big),
    )?;

    assert_eq!(spectrum.len(), 2);
    assert_eq!(spectrum.x.unit, Unit::Seconds);
    assert_eq!(spectrum.x.values, vec![0.0, 0.001]);
    assert_eq!(spectrum.intensities, vec![2.0, 6.0]);
    assert_eq!(spectrum.imaginary, Some(vec![-4.0, -8.0]));
    assert_eq!(spectrum.metadata.name.as_deref(), Some("zg"));
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Carbon13));
    assert_eq!(spectrum.metadata.frequency_mhz, Some(125.5));
    assert_eq!(spectrum.metadata.solvent.as_deref(), Some("CDCl3"));
    assert_eq!(spectrum.metadata.temperature_k, Some(300.0));
    assert_eq!(
        spectrum.metadata.origin.as_deref(),
        Some("raw bytes fixture")
    );

    let builder_spectrum = BrukerFid1DBytes::new(
        "\
##$TD= 2
##$BYTORDA= 0
##$DTYPA= 0
",
        &i32_bytes(&[5, -7], ByteOrder::Little),
    )
    .read()?;
    assert_eq!(builder_spectrum.x.unit, Unit::Points);
    assert_eq!(builder_spectrum.intensities, vec![5.0]);
    assert_eq!(builder_spectrum.imaginary, Some(vec![-7.0]));
    Ok(())
}

#[test]
fn rejects_unsupported_raw_data_type() -> anyhow::Result<()> {
    let root = synthetic_dataset("raw-unsupported")?;
    write_text(
        &root.join("acqus"),
        "\
##$TD= 2
##$DTYPA= 2
",
    )?;
    write_raw_fid(&root, &[1, 2], ByteOrder::Little)?;

    let error = read_bruker_fid_1d_dir(&root).expect_err("unsupported raw data type should fail");
    assert!(matches!(error, RSpinError::Unsupported { .. }));

    remove_dir(root)?;
    Ok(())
}

#[test]
fn reads_raw_2d_ser_dataset_root() -> anyhow::Result<()> {
    let root = synthetic_dataset("raw-2d")?;
    write_text(
        &root.join("acqus"),
        "\
##$TD= 4
##$BYTORDA= 1
##$DTYPA= 0
##$NC= -1
##$SW_h= 1000
##$NUC1= <1H>
##$SFO1= 400.25
##$SOLVENT= <D2O>
##$TE= 299
##$OWNER= <ser fixture>
##$PULPROG= <hsqc>
",
    )?;
    write_text(
        &root.join("acqu2s"),
        "\
##$TD= 2
##$SW_h= 200
##$FnMODE= 0
",
    )?;
    write_raw_ser(&root, &[vec![1, 2, 3, 4], vec![5, 6, 7, 8]], ByteOrder::Big)?;

    let spectrum = BrukerSer2D.read_path(&root)?;

    assert_eq!(spectrum.shape(), (2, 2));
    assert_eq!(spectrum.x.unit, Unit::Seconds);
    assert_eq!(spectrum.y.unit, Unit::Seconds);
    assert_eq!(spectrum.x.values, vec![0.0, 0.001]);
    assert_eq!(spectrum.y.values, vec![0.0, 0.005]);
    assert_eq!(spectrum.z, vec![2.0, 6.0, 10.0, 14.0]);
    assert_eq!(spectrum.imaginary, Some(vec![4.0, 8.0, 12.0, 16.0]));
    assert_eq!(spectrum.metadata.name.as_deref(), Some("hsqc"));
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(spectrum.metadata.frequency_mhz, Some(400.25));
    assert_eq!(spectrum.metadata.solvent.as_deref(), Some("D2O"));
    assert_eq!(spectrum.metadata.temperature_k, Some(299.0));
    assert_eq!(spectrum.metadata.origin.as_deref(), Some("ser fixture"));

    remove_dir(root)?;
    Ok(())
}

#[test]
fn reads_raw_2d_ser_bytes_without_dataset_path() -> anyhow::Result<()> {
    let direct_parameters = "\
##$TD= 4
##$BYTORDA= 1
##$DTYPA= 0
##$NC= -1
##$SW_h= 1000
##$NUC1= <1H>
##$SFO1= 400.25
##$SOLVENT= <D2O>
##$TE= 299
##$OWNER= <ser bytes fixture>
##$PULPROG= <hsqc>
";
    let indirect_parameters = "\
##$TD= 2
##$SW_h= 200
##$FnMODE= 0
";
    let spectrum = read_bruker_ser_2d_bytes(
        direct_parameters,
        indirect_parameters,
        &raw_ser_bytes(&[vec![1, 2, 3, 4], vec![5, 6, 7, 8]], ByteOrder::Big),
    )?;

    assert_eq!(spectrum.shape(), (2, 2));
    assert_eq!(spectrum.x.unit, Unit::Seconds);
    assert_eq!(spectrum.y.unit, Unit::Seconds);
    assert_eq!(spectrum.x.values, vec![0.0, 0.001]);
    assert_eq!(spectrum.y.values, vec![0.0, 0.005]);
    assert_eq!(spectrum.z, vec![2.0, 6.0, 10.0, 14.0]);
    assert_eq!(spectrum.imaginary, Some(vec![4.0, 8.0, 12.0, 16.0]));
    assert_eq!(spectrum.metadata.name.as_deref(), Some("hsqc"));
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(spectrum.metadata.frequency_mhz, Some(400.25));
    assert_eq!(spectrum.metadata.solvent.as_deref(), Some("D2O"));
    assert_eq!(spectrum.metadata.temperature_k, Some(299.0));
    assert_eq!(
        spectrum.metadata.origin.as_deref(),
        Some("ser bytes fixture")
    );

    let builder_spectrum = BrukerSer2DBytes::new(
        "\
##$TD= 4
##$BYTORDA= 0
##$DTYPA= 0
",
        "\
##$TD= 1
##$FnMODE= 1
",
        &raw_ser_bytes(&[vec![1, -1, 2, -2]], ByteOrder::Little),
    )
    .read()?;
    assert_eq!(builder_spectrum.shape(), (2, 1));
    assert_eq!(builder_spectrum.y.unit, Unit::Points);
    assert_eq!(builder_spectrum.z, vec![1.0, 2.0]);
    assert_eq!(builder_spectrum.imaginary, Some(vec![-1.0, -2.0]));
    Ok(())
}

#[test]
fn reads_raw_2d_ser_phase_sensitive_metadata() -> anyhow::Result<()> {
    let direct_parameters = "\
##$TD= 4
##$BYTORDA= 1
##$DTYPA= 0
##$NC= -1
##$SW_h= 1000
##$NUC1= <1H>
##$SFO1= 400.25
##$SFO2= 100.62
##$PULPROG= <hsqcetgpsi>
";
    let indirect_parameters = "\
##$TD= 2
##$SW_h= 200
##$FnMODE= 4
";
    let spectrum = read_bruker_ser_2d_bytes(
        direct_parameters,
        indirect_parameters,
        &raw_ser_bytes(&[vec![1, 2, 3, 4], vec![5, 6, 7, 8]], ByteOrder::Big),
    )?;

    assert_eq!(spectrum.metadata.indirect_frequency_mhz, Some(100.62));
    assert_eq!(spectrum.metadata.quad_mode, Some(QuadMode::States));
    assert_eq!(spectrum.metadata.experiment, Some(ExperimentKind::Hsqc));
    // The raw indirect parameters are preserved as namespaced properties.
    assert_eq!(
        spectrum.metadata.property("bruker.acqu2s.FNMODE"),
        Some("4")
    );
    Ok(())
}

#[test]
fn raw_2d_ser_leaves_indirect_metadata_absent_without_parameters() -> anyhow::Result<()> {
    let direct_parameters = "\
##$TD= 4
##$BYTORDA= 1
##$DTYPA= 0
##$NC= -1
##$SW_h= 1000
##$SFO1= 400.25
";
    let indirect_parameters = "\
##$TD= 2
##$SW_h= 200
";
    let spectrum = read_bruker_ser_2d_bytes(
        direct_parameters,
        indirect_parameters,
        &raw_ser_bytes(&[vec![1, 2, 3, 4], vec![5, 6, 7, 8]], ByteOrder::Big),
    )?;

    assert_eq!(spectrum.metadata.indirect_frequency_mhz, None);
    assert_eq!(spectrum.metadata.quad_mode, None);
    assert_eq!(spectrum.metadata.experiment, None);
    Ok(())
}

#[test]
fn rejects_raw_2d_ser_with_incomplete_padded_row() -> anyhow::Result<()> {
    let root = synthetic_dataset("raw-2d-short")?;
    write_text(
        &root.join("acqus"),
        "\
##$TD= 4
##$DTYPA= 0
",
    )?;
    write_text(
        &root.join("acqu2s"),
        "\
##$TD= 1
",
    )?;
    write_raw_fid(&root, &[1, 2, 3, 4], ByteOrder::Little)?;
    fs::rename(root.join("fid"), root.join("ser"))?;

    let error = read_bruker_ser_2d_dir(&root).expect_err("short Bruker ser row should be rejected");
    assert!(matches!(error, RSpinError::Parse { .. }));

    remove_dir(root)?;
    Ok(())
}
