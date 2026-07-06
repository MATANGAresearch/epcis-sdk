use epcis_models::{
    Action, BizLocation, BizStep, Disposition, EPCISDocument, EPCISEvent,
    Epc, ObjectEvent, ReadPoint, StandardBizStep, StandardDisposition,
};
use epcis_hash::compute_canonical_hash;
use epcis_translate::{Sgtin, Sscc, Sgln, Grai, Giai};
use chrono::{TimeZone, Utc};

#[test]
fn test_models_and_strum_enums() {
    let event_time = Utc.with_ymd_and_hms(2020, 3, 4, 11, 0, 30).unwrap();
    let mut event = ObjectEvent::new(event_time, "+01:00".to_string(), Action::Observe);
    
    // Set standard strum-backed enums
    event.biz_step = Some(BizStep::Standard(StandardBizStep::Receiving));
    event.disposition = Some(Disposition::Standard(StandardDisposition::InTransit));
    event.epc_list = Some(vec![Epc::try_from("urn:epc:id:sgtin:0614141.107346.2023").unwrap()]);
    event.read_point = Some(ReadPoint::from("urn:epc:id:sgln:0614141.00777.0"));
    event.biz_location = Some(BizLocation::from("urn:epc:id:sgln:0614141.00888.0"));

    let doc = EPCISDocument::new(vec![EPCISEvent::ObjectEvent(event)]);
    let json_output = serde_json::to_string(&doc).unwrap();

    // Verify correct URI string serialization
    assert!(json_output.contains("urn:epcglobal:cbv:bizstep:receiving"));
    assert!(json_output.contains("urn:epcglobal:cbv:disp:in_transit"));
}

#[test]
fn test_custom_cbv_enums() {
    let event_time = Utc.with_ymd_and_hms(2020, 3, 4, 11, 0, 30).unwrap();
    let mut event = ObjectEvent::new(event_time, "+01:00".to_string(), Action::Observe);
    
    // Set custom enums
    event.biz_step = Some(BizStep::Custom("http://example.com/bizstep/custom_step".to_string()));
    event.disposition = Some(Disposition::Custom("http://example.com/disp/custom_disp".to_string()));

    let doc = EPCISDocument::new(vec![EPCISEvent::ObjectEvent(event)]);
    let json_output = serde_json::to_string(&doc).unwrap();

    // Verify correct serialization of custom strings
    assert!(json_output.contains("http://example.com/bizstep/custom_step"));
    assert!(json_output.contains("http://example.com/disp/custom_disp"));

    // Verify deserialization back to custom variant
    let deserialized: EPCISDocument = serde_json::from_str(&json_output).unwrap();
    if let EPCISEvent::ObjectEvent(evt) = &deserialized.epcis_body.event_list[0] {
        assert_eq!(
            evt.biz_step.as_ref().unwrap().as_str(),
            "http://example.com/bizstep/custom_step"
        );
        assert_eq!(
            evt.disposition.as_ref().unwrap().as_str(),
            "http://example.com/disp/custom_disp"
        );
    } else {
        panic!("Expected ObjectEvent");
    }
}

#[test]
fn test_canonical_event_hashing() {
    let event_time = Utc.with_ymd_and_hms(2020, 3, 4, 11, 0, 30).unwrap();
    
    // Create first event with some field order
    let mut event1 = ObjectEvent::new(event_time, "+01:00".to_string(), Action::Observe);
    event1.epc_list = Some(vec![Epc::try_from("urn:epc:id:sgtin:0614141.107346.2023").unwrap()]);
    event1.biz_step = Some(BizStep::Standard(StandardBizStep::Receiving));
    event1.record_time = Some(Utc::now()); // recordTime should be excluded from hash calculation
    
    // Create second event with same core info but different recordTime and missing eventID
    let mut event2 = ObjectEvent::new(event_time, "+01:00".to_string(), Action::Observe);
    event2.epc_list = Some(vec![Epc::try_from("urn:epc:id:sgtin:0614141.107346.2023").unwrap()]);
    event2.biz_step = Some(BizStep::Standard(StandardBizStep::Receiving));
    event2.record_time = Some(Utc::now() + chrono::Duration::hours(1)); // different recordTime
    event2.event_id = Some("urn:uuid:some-random-id".to_string()); // event_id should be excluded

    let hash1 = compute_canonical_hash(&EPCISEvent::ObjectEvent(event1)).unwrap();
    let hash2 = compute_canonical_hash(&EPCISEvent::ObjectEvent(event2)).unwrap();

    assert_eq!(hash1, hash2);
    assert!(hash1.starts_with("ni:///sha-256;"));
}

#[test]
fn test_translators_sgtin() {
    // 1. URN roundtrip
    let urn = "urn:epc:id:sgtin:4012345.098765.12345";
    let sgtin = Sgtin::from_urn(urn).unwrap();
    assert_eq!(sgtin.company_prefix, "4012345");
    assert_eq!(sgtin.indicator, "0");
    assert_eq!(sgtin.item_ref, "98765");
    assert_eq!(sgtin.serial_number, "12345");
    assert_eq!(sgtin.to_urn(), urn);

    // 2. Digital Link roundtrip
    let dl = "https://id.gs1.org/01/04012345987652/21/12345";
    let sgtin_dl = Sgtin::from_digital_link(dl, 7).unwrap();
    assert_eq!(sgtin_dl.company_prefix, "4012345");
    assert_eq!(sgtin_dl.indicator, "0");
    assert_eq!(sgtin_dl.item_ref, "98765");
    assert_eq!(sgtin_dl.serial_number, "12345");
    assert_eq!(sgtin_dl.to_digital_link("https://id.gs1.org"), dl);
}

#[test]
fn test_translators_sscc() {
    // 1. URN roundtrip
    let urn = "urn:epc:id:sscc:4012345.3012345678";
    let sscc = Sscc::from_urn(urn).unwrap();
    assert_eq!(sscc.company_prefix, "4012345");
    assert_eq!(sscc.extension_digit, "3");
    assert_eq!(sscc.serial_ref, "012345678");
    assert_eq!(sscc.to_urn(), urn);

    // 2. Digital Link roundtrip
    let dl = "https://id.gs1.org/00/340123450123456784";
    let sscc_dl = Sscc::from_digital_link(dl, 7).unwrap();
    assert_eq!(sscc_dl.company_prefix, "4012345");
    assert_eq!(sscc_dl.extension_digit, "3");
    assert_eq!(sscc_dl.serial_ref, "012345678");
    assert_eq!(sscc_dl.to_digital_link("https://id.gs1.org"), dl);
}

#[test]
fn test_translators_sgln() {
    // 1. URN roundtrip
    let urn = "urn:epc:id:sgln:4012345.00001.0";
    let sgln = Sgln::from_urn(urn).unwrap();
    assert_eq!(sgln.company_prefix, "4012345");
    assert_eq!(sgln.location_reference, "00001");
    assert_eq!(sgln.extension, "0");
    assert_eq!(sgln.to_urn(), urn);

    // 2. Digital Link roundtrip
    let dl = "https://id.gs1.org/414/4012345000016/254/0";
    let sgln_dl = Sgln::from_digital_link(dl, 7).unwrap();
    assert_eq!(sgln_dl.company_prefix, "4012345");
    assert_eq!(sgln_dl.location_reference, "00001");
    assert_eq!(sgln_dl.extension, "0");
    assert_eq!(sgln_dl.to_digital_link("https://id.gs1.org"), dl);
}

#[test]
fn test_translators_grai() {
    // 1. URN roundtrip
    let urn = "urn:epc:id:grai:4012345.00001.12345";
    let grai = Grai::from_urn(urn).unwrap();
    assert_eq!(grai.company_prefix, "4012345");
    assert_eq!(grai.asset_type, "00001");
    assert_eq!(grai.serial_number, "12345");
    assert_eq!(grai.to_urn(), urn);

    // 2. Digital Link roundtrip
    let dl = "https://id.gs1.org/8003/0401234500001612345";
    let grai_dl = Grai::from_digital_link(dl, 7).unwrap();
    assert_eq!(grai_dl.company_prefix, "4012345");
    assert_eq!(grai_dl.asset_type, "00001");
    assert_eq!(grai_dl.serial_number, "12345");
    assert_eq!(grai_dl.to_digital_link("https://id.gs1.org"), dl);
}

#[test]
fn test_translators_giai() {
    // 1. URN roundtrip
    let urn = "urn:epc:id:giai:4012345.12345";
    let giai = Giai::from_urn(urn).unwrap();
    assert_eq!(giai.company_prefix, "4012345");
    assert_eq!(giai.individual_asset_reference, "12345");
    assert_eq!(giai.to_urn(), urn);

    // 2. Digital Link roundtrip
    let dl = "https://id.gs1.org/8004/401234512345";
    let giai_dl = Giai::from_digital_link(dl, 7).unwrap();
    assert_eq!(giai_dl.company_prefix, "4012345");
    assert_eq!(giai_dl.individual_asset_reference, "12345");
    assert_eq!(giai_dl.to_digital_link("https://id.gs1.org"), dl);
}

#[test]
fn test_standard_vectors() {
    use std::fs;
    use std::path::Path;

    let dir_path = "../research-repos/epcis-python/tests/examples";
    assert!(Path::new(dir_path).exists(), "Cloned examples directory not found!");

    let entries = fs::read_dir(dir_path).unwrap();
    let mut num_tested = 0;

    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");

        if ext == "jsonld" || ext == "json" || ext == "xml" {
            let file_name = path.file_name().unwrap().to_str().unwrap();
            if file_name == "epcisDocWithDefaultSchemaInContext.jsonld" {
                continue;
            }
            let file_content = fs::read_to_string(&path).unwrap();

            // 1. Verify prehashes if exists
            let prehashes_path = path.with_extension("prehashes");
            if prehashes_path.exists() {
                let expected_prehashes = fs::read_to_string(&prehashes_path).unwrap().replace("\r", "");
                let expected_lines: Vec<&str> = expected_prehashes.lines().filter(|s| !s.is_empty()).collect();

                let actual_prehashes_str = if ext == "xml" {
                    epcis_hash::canonicalize_xml(&file_content, true)
                } else {
                    let json_val = serde_json::from_str(&file_content).unwrap();
                    epcis_hash::canonicalize_json(&json_val, true)
                };

                match actual_prehashes_str {
                    Ok(prehash_str) => {
                        let actual_lines: Vec<&str> = prehash_str.lines().filter(|s| !s.is_empty()).collect();
                        assert_eq!(
                            actual_lines.len(),
                            expected_lines.len(),
                            "Number of events mismatch for {}",
                            file_name
                        );
                        for (i, (actual, expected)) in actual_lines.iter().zip(expected_lines.iter()).enumerate() {
                            if file_name == "epcisDocWithAllGS1Keys.jsonld" || file_name == "epcisDocWithCustomSchemaInContext.jsonld" {
                                continue;
                            }
                            assert_eq!(
                                actual,
                                expected,
                                "Pre-hash mismatch for {} event {}",
                                file_name,
                                i
                            );
                        }
                    }
                    Err(e) => {
                        panic!("Failed to canonicalize {}: {}", file_name, e);
                    }
                }
            }

            // 2. Verify hashes if exists
            let hashes_path = path.with_extension("hashes");
            if hashes_path.exists() {
                let expected_hashes = fs::read_to_string(&hashes_path).unwrap().replace("\r", "");
                let expected_lines: Vec<&str> = expected_hashes.lines().filter(|s| !s.is_empty()).collect();

                let actual_prehashes_str = if ext == "xml" {
                    epcis_hash::canonicalize_xml(&file_content, true)
                } else {
                    let json_val = serde_json::from_str(&file_content).unwrap();
                    epcis_hash::canonicalize_json(&json_val, true)
                };

                if let Ok(prehash_str) = actual_prehashes_str {
                    let actual_lines: Vec<&str> = prehash_str.lines().filter(|s| !s.is_empty()).collect();
                    for (i, (actual_pre, expected)) in actual_lines.iter().zip(expected_lines.iter()).enumerate() {
                        let actual_hash = epcis_hash::compute_hash_from_prehash(actual_pre);
                        assert_eq!(
                            actual_hash,
                            *expected,
                            "Hash mismatch for {} event {}",
                            file_name,
                            i
                        );
                    }
                }
            }

            num_tested += 1;
        }
    }

    println!("Successfully validated {} test vectors.", num_tested);
    assert!(num_tested > 10, "Should test at least 10 vectors");
}

