//! Canonicalization of arbitrary JSON must return Ok or Err, never panic.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data)
        && let Ok(value) = serde_json::from_str::<serde_json::Value>(s)
    {
        let _ = epcis_hash::canonicalize_json(&value, true);
        let _ = epcis_hash::canonicalize_json(&value, false);
    }
});
