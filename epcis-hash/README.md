# epcis-hash

Deterministic SHA-256 Canonical Hashing implementation for GS1 EPCIS 2.0 events. 

This crate implements the deterministic serialization and hashing procedure outlined by the GS1 and `OpenEPCIS` standards, allowing systems to produce matching, duplicate-preventing hash identifiers for identical events.

## Features

- **Field Exclusion**: Automatically strips event metadata fields such as `recordTime`, `eventID`, and `eventId` that are set downstream and do not belong in the canonical hash.
- **Normalization**: Standardizes timestamps (e.g. `eventTime`, `declarationTime`) to UTC with millisecond-precision formats prior to hashing.
- **Deterministic Ordering**: Recursively sorts object keys and primitive arrays alphabetically to enforce ordering independence.

## Quick Start

```rust
use epcis_models::{Action, ObjectEvent, EPCISEvent};
use epcis_hash::compute_canonical_hash;
use chrono::Utc;

fn main() {
    let event = ObjectEvent::new(
        Utc::now(),
        "+00:00".to_string(),
        Action::Observe
    );
    
    let event_enum = EPCISEvent::ObjectEvent(event);
    
    // Generate the SHA-256 URN
    let hash_urn = compute_canonical_hash(&event_enum).unwrap();
    println!("Hash: {}", hash_urn);
    // e.g. "ni:///sha-256;9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
}
```

## License

Licensed under either of Apache-2.0 or MIT.
