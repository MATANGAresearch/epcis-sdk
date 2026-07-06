//! Canonical Event Hashing library for GS1 EPCIS 2.0.
//!
//! Provides deterministic SHA-256 generation conforming to the GS1 and `OpenEPCIS` specifications.

#![deny(missing_docs)]
#![deny(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use sha2::{Sha256, Digest};
use serde_json::Value;
use epcis_models::EPCISEvent;

/// Recursively canonicalizes a `serde_json::Value` according to EPCIS hash rules.
///
/// Under these rules:
/// - Object keys are sorted alphabetically.
/// - The `recordTime`, `eventID`, and `eventId` keys are stripped from the calculations.
/// - Date/time string values are normalized to UTC with millisecond precision format.
/// - Arrays of primitives (strings, numbers) are sorted to ensure format-independence.
#[must_use]
pub fn canonicalize_value(val: &Value) -> Value {
    match val {
        Value::Object(map) => {
            let mut sorted = std::collections::BTreeMap::new();
            for (k, v) in map {
                // Exclude recordTime and eventID / eventId from hash calculation
                if k == "recordTime" || k == "eventID" || k == "eventId" {
                    continue;
                }
                
                let canonical_v = canonicalize_value(v);
                
                // If it is a timestamp string, normalize it to UTC with millisecond precision
                let canonical_v = if (k == "eventTime" || k == "declarationTime" || k == "time") && canonical_v.is_string() {
                    if let Some(s) = canonical_v.as_str() {
                        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
                            let utc_dt = dt.with_timezone(&chrono::Utc);
                            Value::String(utc_dt.to_rfc3339_opts(chrono::SecondsFormat::Millis, true))
                        } else {
                            canonical_v
                        }
                    } else {
                        canonical_v
                    }
                } else {
                    canonical_v
                };
                
                sorted.insert(k.clone(), canonical_v);
            }
            Value::Object(sorted.into_iter().collect())
        }
        Value::Array(arr) => {
            let mut canon_arr: Vec<Value> = arr.iter().map(canonicalize_value).collect();
            // Sort array elements if they are primitives (strings/numbers) to be deterministic
            canon_arr.sort_by(|a, b| {
                let a_str = serde_json::to_string(a).unwrap_or_default();
                let b_str = serde_json::to_string(b).unwrap_or_default();
                a_str.cmp(&b_str)
            });
            Value::Array(canon_arr)
        }
        _ => val.clone(),
    }
}

/// Generates a canonical SHA-256 Event Hash ID URN for any EPCIS event.
///
/// # Errors
///
/// Returns an error if serialization to JSON fails.
pub fn compute_canonical_hash(event: &EPCISEvent) -> Result<String, serde_json::Error> {
    let val = serde_json::to_value(event)?;
    let canonical = canonicalize_value(&val);
    let canonical_str = serde_json::to_string(&canonical)?;
    
    let mut hasher = Sha256::new();
    hasher.update(canonical_str.as_bytes());
    let hash_result = hasher.finalize();
    
    Ok(format!("ni:///sha-256;{hash_result:x}"))
}
