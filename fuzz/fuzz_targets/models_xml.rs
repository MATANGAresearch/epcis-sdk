//! Typed XML document parsing must return Ok or Err, never panic; successful
//! parses must survive re-serialization.
#![no_main]

use epcis_models::{EPCISDocument, EPCISQueryDocument};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        if let Ok(doc) = EPCISDocument::from_xml(s) {
            let _ = doc.to_xml();
        }
        if let Ok(doc) = EPCISQueryDocument::from_xml(s) {
            let _ = doc.to_xml();
        }
    }
});
