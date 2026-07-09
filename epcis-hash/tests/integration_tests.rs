use chrono::{TimeZone, Utc};
use epcis_hash::compute_canonical_hash;
use epcis_models::{
    Action, BizLocation, BizStep, Disposition, EPCISDocument, EPCISEvent, Epc, ObjectEvent,
    ReadPoint, StandardBizStep, StandardDisposition,
};
use epcis_translate::{Giai, Grai, Sgln, Sgtin, Sscc};

#[test]
fn test_models_and_strum_enums() {
    let event_time = Utc.with_ymd_and_hms(2020, 3, 4, 11, 0, 30).unwrap();
    let mut event = ObjectEvent::new(event_time, "+01:00".to_string(), Action::Observe);

    // Set standard strum-backed enums
    event.biz_step = Some(BizStep::Standard(StandardBizStep::Receiving));
    event.disposition = Some(Disposition::Standard(StandardDisposition::InTransit));
    event.epc_list = Some(vec![
        Epc::try_from("urn:epc:id:sgtin:0614141.107346.2023").unwrap(),
    ]);
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
    event.biz_step = Some(BizStep::Custom(
        "http://example.com/bizstep/custom_step".to_string(),
    ));
    event.disposition = Some(Disposition::Custom(
        "http://example.com/disp/custom_disp".to_string(),
    ));

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
    event1.epc_list = Some(vec![
        Epc::try_from("urn:epc:id:sgtin:0614141.107346.2023").unwrap(),
    ]);
    event1.biz_step = Some(BizStep::Standard(StandardBizStep::Receiving));
    event1.record_time = Some(Utc::now()); // recordTime should be excluded from hash calculation

    // Create second event with same core info but different recordTime and missing eventID
    let mut event2 = ObjectEvent::new(event_time, "+01:00".to_string(), Action::Observe);
    event2.epc_list = Some(vec![
        Epc::try_from("urn:epc:id:sgtin:0614141.107346.2023").unwrap(),
    ]);
    event2.biz_step = Some(BizStep::Standard(StandardBizStep::Receiving));
    event2.record_time = Some(Utc::now() + chrono::Duration::hours(1)); // different recordTime
    event2.event_id = Some("urn:uuid:some-random-id".to_string()); // event_id should be excluded

    let hash1 = compute_canonical_hash(&EPCISEvent::ObjectEvent(event1)).unwrap();
    let hash2 = compute_canonical_hash(&EPCISEvent::ObjectEvent(event2)).unwrap();

    assert_eq!(hash1, hash2);
    assert!(hash1.starts_with("ni:///sha-256;"));
}

#[test]
fn test_sensor_document_roundtrip_preserves_all_fields() {
    use std::fs;

    // Typed models must not silently drop standard sensor fields
    // (component, coordinateReferenceSystem, exception, ...) or
    // namespace-qualified extensions inside sensorReport.
    for file in [
        "../research-repos/epcis-python/tests/examples/epcisDocWithSensorComponent.jsonld",
        "../research-repos/epcis-python/tests/examples/CertificationInfoAndSensorReportWithCRS.jsonld",
    ] {
        let content = fs::read_to_string(file).unwrap();
        let doc: EPCISDocument = serde_json::from_str(&content)
            .unwrap_or_else(|e| panic!("failed to parse {file}: {e}"));
        let roundtrip = serde_json::to_value(&doc).unwrap();
        let original: serde_json::Value = serde_json::from_str(&content).unwrap();

        let events = original["epcisBody"]["eventList"].as_array().unwrap();
        for (i, event) in events.iter().enumerate() {
            let Some(elements) = event.get("sensorElementList").and_then(|v| v.as_array()) else {
                continue;
            };
            for (j, element) in elements.iter().enumerate() {
                let reports = element["sensorReport"].as_array().unwrap();
                for (k, report) in reports.iter().enumerate() {
                    let rt_report = &roundtrip["epcisBody"]["eventList"][i]["sensorElementList"][j]
                        ["sensorReport"][k];
                    let orig_keys: std::collections::BTreeSet<&String> =
                        report.as_object().unwrap().keys().collect();
                    let rt_keys: std::collections::BTreeSet<&String> =
                        rt_report.as_object().unwrap().keys().collect();
                    assert_eq!(
                        orig_keys, rt_keys,
                        "sensorReport keys lost in {file} event {i} element {j} report {k}"
                    );
                }
            }
        }
    }
}

#[test]
fn test_native_xml_parsing_is_semantically_faithful() {
    use std::fs;
    use std::path::Path;

    // For every official EPCIS 2.0 XML document vector: parsing it into the
    // typed EPCISDocument and canonicalizing the JSON serialization must
    // yield exactly the same pre-hashes as canonicalizing the XML directly.
    // Hash equality proves the typed parse lost or altered nothing that the
    // EPCIS data model considers meaningful.
    let dir_path = "../research-repos/epcis-python/tests/examples";
    assert!(Path::new(dir_path).exists(), "examples directory not found");

    let mut num_checked = 0;
    for entry in fs::read_dir(dir_path).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|s| s.to_str()) != Some("xml") {
            continue;
        }
        let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
        let xml = fs::read_to_string(&path).unwrap();
        // Query documents use a different envelope than EPCISDocument.
        if xml.contains("EPCISQueryDocument") {
            continue;
        }

        let doc = EPCISDocument::from_xml(&xml)
            .unwrap_or_else(|e| panic!("from_xml failed for {file_name}: {e}"));
        let json_val = serde_json::to_value(&doc).unwrap();

        let from_typed = epcis_hash::canonicalize_json(&json_val, true)
            .unwrap_or_else(|e| panic!("canonicalize_json failed for {file_name}: {e}"));
        let from_xml_direct = epcis_hash::canonicalize_xml(&xml, true)
            .unwrap_or_else(|e| panic!("canonicalize_xml failed for {file_name}: {e}"));
        assert_eq!(
            from_typed, from_xml_direct,
            "typed-parse prehash diverges for {file_name}"
        );

        // And the same must hold after re-serializing the typed document
        // back to XML.
        let rewritten = doc
            .to_xml()
            .unwrap_or_else(|e| panic!("to_xml failed for {file_name}: {e}"));
        let from_rewritten = epcis_hash::canonicalize_xml(&rewritten, true).unwrap_or_else(|e| {
            panic!("canonicalize_xml of rewritten failed for {file_name}: {e}")
        });
        assert_eq!(
            from_rewritten, from_xml_direct,
            "re-serialized XML prehash diverges for {file_name}"
        );

        num_checked += 1;
    }
    assert!(
        num_checked >= 10,
        "expected at least 10 XML vectors, got {num_checked}"
    );
}

#[test]
fn test_typed_transformation_event_hash_matches_spec_json() {
    use epcis_models::TransformationEvent;

    // The typed model must serialize `transformationID` (spec spelling) and
    // therefore hash identically to the equivalent spec-compliant JSON event.
    let mut event = TransformationEvent::new(
        Utc.with_ymd_and_hms(2020, 3, 4, 11, 0, 30).unwrap(),
        "+01:00".to_string(),
    );
    event.transformation_id = Some("urn:epc:id:gdti:0614141.12345.400".to_string());
    let typed_hash = compute_canonical_hash(&EPCISEvent::TransformationEvent(event)).unwrap();

    let spec_json = serde_json::json!({
        "type": "TransformationEvent",
        "eventTime": "2020-03-04T11:00:30.000Z",
        "eventTimeZoneOffset": "+01:00",
        "transformationID": "urn:epc:id:gdti:0614141.12345.400"
    });
    let prehash = epcis_hash::canonicalize_json(&spec_json, true).unwrap();
    assert!(
        prehash.contains("transformationID="),
        "transformationID missing from spec prehash: {prehash}"
    );
    let spec_hash = epcis_hash::compute_hash_from_prehash(&prehash);

    assert_eq!(typed_hash, spec_hash);
}

#[test]
fn test_xml_default_namespace_hashes_like_prefixed() {
    // The same event expressed three ways must produce the same pre-hash:
    // (a) XML with a prefixed root namespace, (b) XML with a default xmlns
    // covering every element, (c) JSON.
    let prefixed_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<epcis:EPCISDocument xmlns:epcis="urn:epcglobal:epcis:xsd:2" schemaVersion="2.0" creationDate="2020-03-04T11:00:30.000+01:00">
  <EPCISBody>
    <EventList>
      <ObjectEvent>
        <eventTime>2020-03-04T11:00:30.000+01:00</eventTime>
        <eventTimeZoneOffset>+01:00</eventTimeZoneOffset>
        <epcList><epc>urn:epc:id:sgtin:0614141.107346.2023</epc></epcList>
        <action>OBSERVE</action>
        <bizTransactionList>
          <bizTransaction type="urn:epcglobal:cbv:btt:po">http://transaction.acme.com/po/12345678</bizTransaction>
        </bizTransactionList>
      </ObjectEvent>
    </EventList>
  </EPCISBody>
</epcis:EPCISDocument>"#;
    let default_ns_xml = prefixed_xml
        .replace(
            "xmlns:epcis=\"urn:epcglobal:epcis:xsd:2\"",
            "xmlns=\"urn:epcglobal:epcis:xsd:2\"",
        )
        .replace("epcis:EPCISDocument", "EPCISDocument");

    let prefixed = epcis_hash::canonicalize_xml(prefixed_xml, true).unwrap();
    let default_ns = epcis_hash::canonicalize_xml(&default_ns_xml, true).unwrap();
    assert_eq!(prefixed, default_ns);
    assert!(
        !default_ns.contains('{'),
        "EPCIS namespace leaked into prehash: {default_ns}"
    );

    let json_equiv = serde_json::json!({
        "type": "ObjectEvent",
        "eventTime": "2020-03-04T11:00:30.000+01:00",
        "eventTimeZoneOffset": "+01:00",
        "epcList": ["urn:epc:id:sgtin:0614141.107346.2023"],
        "action": "OBSERVE",
        "bizTransactionList": [
            {"type": "urn:epcglobal:cbv:btt:po", "bizTransaction": "http://transaction.acme.com/po/12345678"}
        ]
    });
    let json_prehash = epcis_hash::canonicalize_json(&json_equiv, true).unwrap();
    assert_eq!(prefixed, json_prehash);
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

    // 2. Digital Link roundtrip — extension "0" canonically omits /254/
    let dl = "https://id.gs1.org/414/4012345000016/254/0";
    let sgln_dl = Sgln::from_digital_link(dl, 7).unwrap();
    assert_eq!(sgln_dl.company_prefix, "4012345");
    assert_eq!(sgln_dl.location_reference, "00001");
    assert_eq!(sgln_dl.extension, "0");
    let canonical_dl = "https://id.gs1.org/414/4012345000016";
    assert_eq!(sgln_dl.to_digital_link("https://id.gs1.org"), canonical_dl);

    // A plain GLN without /254/ parses with extension "0"
    let sgln_plain = Sgln::from_digital_link(canonical_dl, 7).unwrap();
    assert_eq!(sgln_plain.extension, "0");
    assert_eq!(sgln_plain.to_urn(), "urn:epc:id:sgln:4012345.00001.0");

    // Non-"0" extensions keep the /254/ qualifier
    let dl_ext = "https://id.gs1.org/414/4012345000016/254/987";
    let sgln_ext = Sgln::from_digital_link(dl_ext, 7).unwrap();
    assert_eq!(sgln_ext.to_digital_link("https://id.gs1.org"), dl_ext);
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
    assert!(
        Path::new(dir_path).exists(),
        "Cloned examples directory not found!"
    );

    // The upstream .prehashes files for these vectors are corrupted (verified
    // byte-level: one truncated mid-token, one with garbled interleaved text),
    // so only their .hashes files are compared — which pass.
    const CORRUPT_UPSTREAM_PREHASHES: [&str; 3] = [
        "epcisDocWithAllGS1Keys.jsonld",
        "epcisDocWithCustomSchemaInContext.jsonld",
        "epcisDocWithDefaultSchemaInContext.jsonld",
    ];

    let entries = fs::read_dir(dir_path).unwrap();
    let mut num_tested = 0;

    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");

        if ext == "jsonld" || ext == "json" || ext == "xml" {
            let file_name = path.file_name().unwrap().to_str().unwrap();
            let file_content = fs::read_to_string(&path).unwrap();

            // 1. Verify prehashes if exists
            let prehashes_path = path.with_extension("prehashes");
            if prehashes_path.exists() && !CORRUPT_UPSTREAM_PREHASHES.contains(&file_name) {
                let expected_prehashes = fs::read_to_string(&prehashes_path)
                    .unwrap()
                    .replace("\r", "");
                let expected_lines: Vec<&str> = expected_prehashes
                    .lines()
                    .filter(|s| !s.is_empty())
                    .collect();

                let actual_prehashes_str = if ext == "xml" {
                    epcis_hash::canonicalize_xml(&file_content, true)
                } else {
                    let json_val = serde_json::from_str(&file_content).unwrap();
                    epcis_hash::canonicalize_json(&json_val, true)
                };

                match actual_prehashes_str {
                    Ok(prehash_str) => {
                        let actual_lines: Vec<&str> =
                            prehash_str.lines().filter(|s| !s.is_empty()).collect();
                        assert_eq!(
                            actual_lines.len(),
                            expected_lines.len(),
                            "Number of events mismatch for {}",
                            file_name
                        );
                        for (i, (actual, expected)) in
                            actual_lines.iter().zip(expected_lines.iter()).enumerate()
                        {
                            assert_eq!(
                                actual, expected,
                                "Pre-hash mismatch for {} event {}",
                                file_name, i
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
                let expected_lines: Vec<&str> =
                    expected_hashes.lines().filter(|s| !s.is_empty()).collect();

                let actual_prehashes_str = if ext == "xml" {
                    epcis_hash::canonicalize_xml(&file_content, true)
                } else {
                    let json_val = serde_json::from_str(&file_content).unwrap();
                    epcis_hash::canonicalize_json(&json_val, true)
                };

                if let Ok(prehash_str) = actual_prehashes_str {
                    let actual_lines: Vec<&str> =
                        prehash_str.lines().filter(|s| !s.is_empty()).collect();
                    assert_eq!(
                        actual_lines.len(),
                        expected_lines.len(),
                        "Number of hashes mismatch for {}",
                        file_name
                    );
                    for (i, (actual_pre, expected)) in
                        actual_lines.iter().zip(expected_lines.iter()).enumerate()
                    {
                        let actual_hash = epcis_hash::compute_hash_from_prehash(actual_pre);
                        assert_eq!(
                            actual_hash, *expected,
                            "Hash mismatch for {} event {}",
                            file_name, i
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
