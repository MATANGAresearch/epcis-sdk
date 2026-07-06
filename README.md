# GS1 EPCIS 2.0 Rust SDK

A high-performance, native Rust SDK for working with GS1 EPCIS 2.0 events, canonical hashing, and identifier translations. This library is designed to be type-safe, strict, and highly efficient.

## Workspace Crates

This workspace consists of three modular crates:

1. **[`epcis-models`](./epcis-models)**: Type-safe event models (`ObjectEvent`, `AggregationEvent`, `TransformationEvent`, etc.) and CBV (Core Business Vocabulary) enum systems.
2. **[`epcis-hash`](./epcis-hash)**: Deterministic, canonical SHA-256 event hashing implementation conforming to OpenEPCIS and GS1 standards.
3. **[`epcis-translate`](./epcis-translate)**: Zero-allocation, bidirectional translators for converting GS1 keys (SGTIN, SSCC, SGLN, GRAI, GIAI) between EPC URN and GS1 Digital Link path formats.

---

## Getting Started

### Installation

Add the crates to your `Cargo.toml` dependencies:

```toml
[dependencies]
epcis-models = "0.1.1"
epcis-hash = "0.1.1"
epcis-translate = "0.1.1"
```

### Complete Example

Here is an end-to-end example demonstrating how to construct an event, calculate its canonical hash, and translate an SGTIN identifier:

```rust
use epcis_models::{
    Action, BizStep, Disposition, EPCISEvent, Epc, ObjectEvent,
    StandardBizStep, StandardDisposition,
};
use epcis_hash::compute_canonical_hash;
use epcis_translate::Sgtin;
use chrono::Utc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Translate SGTIN from Digital Link to EPC URN format
    let digital_link = "https://id.gs1.org/01/04012345987652/21/12345";
    let company_prefix_len = 7; // Length of company prefix "4012345"
    let sgtin = Sgtin::from_digital_link(digital_link, company_prefix_len)?;
    let epc_urn = sgtin.to_urn();
    
    println!("Translated URN: {}", epc_urn);
    // Outputs: urn:epc:id:sgtin:4012345.098765.12345

    // 2. Build a type-safe EPCIS Object Event
    let event_time = Utc::now();
    let mut event = ObjectEvent::new(event_time, "+00:00".to_string(), Action::Observe);
    event.biz_step = Some(BizStep::Standard(StandardBizStep::Receiving));
    event.disposition = Some(Disposition::Standard(StandardDisposition::InTransit));
    event.epc_list = Some(vec![Epc::try_from(epc_urn.as_str())?]);

    // 3. Compute deterministic canonical hash ID
    let event_enum = EPCISEvent::ObjectEvent(event);
    let hash_urn = compute_canonical_hash(&event_enum)?;
    
    println!("Canonical Hash URN: {}", hash_urn);
    // Outputs: ni:///sha-256;...
    
    Ok(())
}
```

## Running Tests

To run the test suite and confirm everything compiles and runs correctly:

```bash
cargo test
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
