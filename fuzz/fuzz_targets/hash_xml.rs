//! Canonicalization of arbitrary XML must return Ok or Err, never panic.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = epcis_hash::canonicalize_xml(s, true);
        let _ = epcis_hash::canonicalize_xml(s, false);
    }
});
