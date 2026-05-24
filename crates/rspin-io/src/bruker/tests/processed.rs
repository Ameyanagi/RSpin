use super::*;

#[test]
fn reads_processed_1d_dataset_root() -> anyhow::Result<()> {
    let root = synthetic_dataset("root")?;
    write_text(
        &root.join("acqus"),
        "\
##$NUC1= <1H>
##$SFO1= 400.13
##$SOLVENT= <CDCl3>
##$TE= 298.15
##$OWNER= <local fixture>
",
    )?;
    write_processed_dir(
        &root,
        "\
##$SI= 4
##$BYTORDP= 0
##$DTYPP= 0
##$NC_proc= 0
##$OFFSET= 10
##$SW_p= 4000
##$SF= 400
##$AXNUC= <1H>
",
        &[100, -50, 25, 0],
        ByteOrder::Little,
    )?;
    write_text(&root.join("pdata/1/title"), "ethyl acetate\n")?;

    let spectrum = read_bruker_processed_1d_dir(&root)?;

    assert_eq!(spectrum.metadata.name.as_deref(), Some("ethyl acetate"));
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(spectrum.metadata.frequency_mhz, Some(400.0));
    assert_eq!(spectrum.metadata.solvent.as_deref(), Some("CDCl3"));
    assert_eq!(spectrum.metadata.temperature_k, Some(298.15));
    assert_eq!(spectrum.metadata.origin.as_deref(), Some("local fixture"));
    assert_eq!(spectrum.x.unit, Unit::Ppm);
    assert_close(spectrum.x.values[0], 10.0);
    assert_close(spectrum.x.values[1], 20.0 / 3.0);
    assert_close(spectrum.x.values[2], 10.0 / 3.0);
    assert_close(spectrum.x.values[3], 0.0);
    assert_eq!(spectrum.intensities, vec![100.0, -50.0, 25.0, 0.0]);

    remove_dir(root)?;
    Ok(())
}

#[test]
fn reads_processed_directory_with_scaling_and_big_endian_data() -> anyhow::Result<()> {
    let root = synthetic_dataset("processed")?;
    write_processed_dir(
        &root,
        "\
##$SI= 3
##$BYTORDP= 1
##$DTYPP= 0
##$NC_proc= -1
",
        &[2, -4, 6],
        ByteOrder::Big,
    )?;

    let spectrum = BrukerProcessed1D.read_dir(root.join("pdata/1"))?;

    assert_eq!(spectrum.x.unit, Unit::Points);
    assert_eq!(spectrum.x.values, vec![0.0, 1.0, 2.0]);
    assert_eq!(spectrum.intensities, vec![4.0, -8.0, 12.0]);

    remove_dir(root)?;
    Ok(())
}

#[test]
fn reads_processed_1d_directory_with_imaginary_plane() -> anyhow::Result<()> {
    let root = synthetic_dataset("processed-1d-complex")?;
    write_processed_dir(
        &root,
        "\
##$SI= 3
##$BYTORDP= 0
##$DTYPP= 0
##$NC_proc= -1
",
        &[1, -2, 3],
        ByteOrder::Little,
    )?;
    write_processed_1d_imaginary(&root, &[-1, 2, -3], ByteOrder::Little)?;

    let spectrum = read_bruker_processed_1d_dir(root.join("pdata/1"))?;

    assert_eq!(spectrum.intensities, vec![2.0, -4.0, 6.0]);
    assert_eq!(spectrum.imaginary, Some(vec![-2.0, 4.0, -6.0]));

    remove_dir(root)?;
    Ok(())
}

#[test]
fn rejects_unsupported_processed_data_type() -> anyhow::Result<()> {
    let root = synthetic_dataset("unsupported")?;
    write_processed_dir(
        &root,
        "\
##$SI= 1
##$DTYPP= 2
",
        &[1],
        ByteOrder::Little,
    )?;

    let error = read_bruker_processed_1d_dir(&root)
        .expect_err("unsupported processed data type should fail");
    assert!(matches!(error, RSpinError::Unsupported { .. }));

    remove_dir(root)?;
    Ok(())
}

#[test]
fn rejects_truncated_processed_1d_imaginary_plane() -> anyhow::Result<()> {
    let root = synthetic_dataset("truncated-1i")?;
    write_processed_dir(
        &root,
        "\
##$SI= 3
##$DTYPP= 0
",
        &[1, 2, 3],
        ByteOrder::Little,
    )?;
    write_processed_1d_imaginary(&root, &[1, 2], ByteOrder::Little)?;

    let error = read_bruker_processed_1d_dir(&root).expect_err("truncated 1i should fail");
    assert!(matches!(error, RSpinError::Parse { .. }));
    assert!(error.to_string().contains("1i"));

    remove_dir(root)?;
    Ok(())
}

#[test]
fn reads_processed_2d_dataset_root_with_imaginary_plane() -> anyhow::Result<()> {
    let root = synthetic_dataset("processed-2d")?;
    write_text(
        &root.join("acqus"),
        "\
##$NUC1= <1H>
##$SFO1= 500.0
##$SOLVENT= <DMSO>
##$TE= 301
##$OWNER= <processed 2d fixture>
",
    )?;
    write_processed_2d_dir(
        &root,
        "\
##$SI= 3
##$BYTORDP= 0
##$DTYPP= 0
##$NC_proc= -1
##$OFFSET= 10
##$SW_p= 3000
##$SF= 500
##$AXNUC= <1H>
",
        "\
##$SI= 2
##$OFFSET= 120
##$SW_p= 2000
##$SF= 100
##$AXNUC= <13C>
",
        &[1, 2, 3, 4, 5, 6],
        Some(&[-1, -2, -3, -4, -5, -6]),
        ByteOrder::Little,
    )?;
    write_text(&root.join("pdata/1/title"), "processed hsqc\n")?;

    let spectrum = BrukerProcessed2D.read_dir(&root)?;

    assert_eq!(spectrum.shape(), (3, 2));
    assert_eq!(spectrum.metadata.name.as_deref(), Some("processed hsqc"));
    assert_eq!(spectrum.metadata.nucleus, Some(Nucleus::Hydrogen1));
    assert_eq!(spectrum.metadata.frequency_mhz, Some(500.0));
    assert_eq!(spectrum.metadata.solvent.as_deref(), Some("DMSO"));
    assert_eq!(spectrum.metadata.temperature_k, Some(301.0));
    assert_eq!(
        spectrum.metadata.origin.as_deref(),
        Some("processed 2d fixture")
    );
    assert_eq!(spectrum.x.unit, Unit::Ppm);
    assert_eq!(spectrum.y.unit, Unit::Ppm);
    assert_close(spectrum.x.values[0], 10.0);
    assert_close(spectrum.x.values[2], 4.0);
    assert_close(spectrum.y.values[0], 120.0);
    assert_close(spectrum.y.values[1], 100.0);
    assert_eq!(spectrum.z, vec![2.0, 4.0, 6.0, 8.0, 10.0, 12.0]);
    assert_eq!(
        spectrum.imaginary,
        Some(vec![-2.0, -4.0, -6.0, -8.0, -10.0, -12.0])
    );

    remove_dir(root)?;
    Ok(())
}

#[test]
fn rejects_processed_2d_truncated_matrix() -> anyhow::Result<()> {
    let root = synthetic_dataset("processed-2d-truncated")?;
    write_processed_2d_dir(
        &root,
        "\
##$SI= 3
##$DTYPP= 0
",
        "\
##$SI= 2
",
        &[1, 2, 3, 4, 5],
        None,
        ByteOrder::Little,
    )?;

    let error = read_bruker_processed_2d_dir(&root).expect_err("truncated 2rr should be rejected");
    assert!(matches!(error, RSpinError::Parse { .. }));

    remove_dir(root)?;
    Ok(())
}
