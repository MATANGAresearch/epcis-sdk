use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use epcis_hash::{canonicalize_json, canonicalize_xml};

fn bench_hashing(c: &mut Criterion) {
    let json_input = r#"{
      "@context": ["https://ref.gs1.org/standards/epcis/2.0.0/epcis-context.jsonld"],
      "type": "EPCISDocument",
      "schemaVersion": "2.0",
      "creationDate": "2020-03-04T11:00:30.000Z",
      "epcisBody": {
        "eventList": [
          {
            "type": "ObjectEvent",
            "eventTime": "2020-03-04T11:00:30.000Z",
            "eventTimeZoneOffset": "+00:00",
            "action": "OBSERVE",
            "epcList": ["urn:epc:id:sgtin:4012345.098765.12345"]
          }
        ]
      }
    }"#;

    let xml_input = r#"<?xml version="1.0" encoding="UTF-8"?>
    <epcis:EPCISDocument xmlns:epcis="urn:epcglobal:epcis:xsd:2" schemaVersion="2.0" creationDate="2005-07-11T11:30:47.0Z">
      <epcisBody>
        <EventList>
          <ObjectEvent>
            <eventTime>2005-07-11T11:30:47.0Z</eventTime>
            <eventTimeZoneOffset>+02:00</eventTimeZoneOffset>
            <action>OBSERVE</action>
            <epcList>
              <epc>urn:epc:id:sgtin:0614141.107346.2017</epc>
            </epcList>
          </ObjectEvent>
        </EventList>
      </epcisBody>
    </epcis:EPCISDocument>"#;

    let json_val: serde_json::Value = serde_json::from_str(json_input).unwrap();

    c.bench_function("JSON Canonicalize", |b| {
        b.iter(|| {
            let _ = black_box(canonicalize_json(black_box(&json_val), true));
        })
    });

    c.bench_function("XML Canonicalize", |b| {
        b.iter(|| {
            let _ = black_box(canonicalize_xml(black_box(xml_input), true));
        })
    });
}

criterion_group!(benches, bench_hashing);
criterion_main!(benches);
