use rspin_core::RSpinError;

use crate::{SpectrumReader, SpectrumWriter, read_nmredata_records_str, read_nmredata_str};

use super::*;

#[test]
fn round_trips_one_record_json() -> anyhow::Result<()> {
    let record = record_fixture()?;
    let text = write_nmredata_record_json(&record)?;
    let parsed = read_nmredata_record_json(&text)?;

    assert!(text.contains(&format!("\"format\":\"{NMREDATA_RECORD_JSON_FORMAT}\"")));
    assert!(text.contains(&format!("\"version\":{NMREDATA_JSON_VERSION}")));
    assert!(text.contains("\"record\""));
    assert_eq!(parsed, record);
    Ok(())
}

#[test]
fn reads_legacy_raw_record_json_with_version_field() -> anyhow::Result<()> {
    let record = record_fixture()?;
    let text = serde_json::to_string(&record)?;
    let parsed = read_nmredata_record_json(&text)?;

    assert_eq!(
        parsed.version.as_ref().map(|version| version.raw.as_str()),
        Some("1.1")
    );
    assert_eq!(parsed, record);
    Ok(())
}

#[test]
fn round_trips_record_list_json() -> anyhow::Result<()> {
    let records = read_nmredata_records_str(
        r"
>  <NMREDATA_VERSION>
1.0
$$$$
>  <NMREDATA_VERSION>
1.1

>  <NMREDATA_FORMULA>
C2H6O
$$$$
",
    )?;
    let text = write_nmredata_records_json(&records)?;
    let parsed = read_nmredata_records_json(&text)?;

    assert!(text.contains(&format!("\"format\":\"{NMREDATA_RECORDS_JSON_FORMAT}\"")));
    assert!(text.contains("\"records\""));
    assert_eq!(parsed, records);
    Ok(())
}

#[test]
fn reads_legacy_raw_record_list_json() -> anyhow::Result<()> {
    let records = vec![record_fixture()?];
    let text = serde_json::to_string(&records)?;
    let parsed = read_nmredata_records_json(&text)?;

    assert_eq!(parsed, records);
    Ok(())
}

#[test]
fn rejects_wrong_nmredata_json_headers() {
    let wrong_format = read_nmredata_record_json(
        r#"{"format":"rspin.nmredata_records","version":1,"record":{"tags":[]}}"#,
    )
    .expect_err("wrong record JSON format should fail");
    assert!(matches!(wrong_format, RSpinError::Parse { .. }));

    let unsupported_version = read_nmredata_records_json(
        r#"{"format":"rspin.nmredata_records","version":2,"records":[]}"#,
    )
    .expect_err("unsupported NMReDATA JSON version should fail");
    assert!(matches!(
        unsupported_version,
        RSpinError::Unsupported {
            feature: "NMReDATA JSON version"
        }
    ));
}

#[test]
fn nmredata_json_codecs_implement_traits() -> anyhow::Result<()> {
    let record = record_fixture()?;
    let text = <JsonNmreDataRecord as SpectrumWriter<NmreDataRecord>>::write_string(
        &JsonNmreDataRecord,
        &record,
    )?;
    let parsed: NmreDataRecord = SpectrumReader::read_str(&JsonNmreDataRecord, &text)?;

    assert_eq!(format!("{JsonNmreDataRecord:?}"), "JsonNmreDataRecord");
    assert_eq!(parsed, record);

    let records = vec![record];
    let records_text = <JsonNmreDataRecords as SpectrumWriter<[NmreDataRecord]>>::write_string(
        &JsonNmreDataRecords,
        &records,
    )?;
    let parsed_records: Vec<NmreDataRecord> =
        SpectrumReader::read_str(&JsonNmreDataRecords, &records_text)?;

    assert_eq!(format!("{JsonNmreDataRecords:?}"), "JsonNmreDataRecords");
    assert_eq!(parsed_records, records);
    Ok(())
}

fn record_fixture() -> anyhow::Result<NmreDataRecord> {
    Ok(read_nmredata_str(
        r"
>  <NMREDATA_VERSION>
1.1

>  <NMREDATA_FORMULA>
C2H6O

>  <NMREDATA_ASSIGNMENT>
H1, 4.200, H1
",
    )?)
}
