# epcis-models

Type-safe, native Rust representations of GS1 EPCIS 2.0 Core Event Models and Core Business Vocabulary (CBV) standard vocabularies.

## Features

- **Standardized Event Types**: Implements structs for all core EPCIS event types:
  - `ObjectEvent`
  - `AggregationEvent`
  - `TransformationEvent`
  - `AssociationEvent`
  - `TransactionEvent`
- **Flexibility for Custom Vocabularies**: Leverages open-ended enum structures via `strum`-backed CBV bindings, allowing you to easily use either GS1 Standard elements (e.g. `StandardBizStep::Receiving`) or any Custom URNs/URLs.
- **Fail-Safe Enforcements**: Implements `#![deny(unsafe_code)]` and strict linting.
- **Full Serialization**: Built on top of `serde` and `serde_json` for high-fidelity conversion.

## Quick Start

### Basic Event Structure

```rust
use epcis_models::{
    Action, BizStep, Disposition, EPCISEvent, ObjectEvent,
    StandardBizStep, StandardDisposition,
};
use chrono::Utc;

// Create a standard Object Event
let mut event = ObjectEvent::new(
    Utc::now(),
    "+02:00".to_string(), // Timezone offset
    Action::Observe,
);

// Assign CBV standard business step and disposition
event.biz_step = Some(BizStep::Standard(StandardBizStep::Receiving));
event.disposition = Some(Disposition::Standard(StandardDisposition::InTransit));

// Serialize to JSON
let serialized = serde_json::to_string_pretty(&event).unwrap();
println!("{}", serialized);
```

### Custom vocabularies

```rust
use epcis_models::{BizStep, StandardBizStep};

// Can use standard
let standard_step = BizStep::Standard(StandardBizStep::Shipping);

// Or easily fall back to custom URNs/URLs
let custom_step = BizStep::Custom("https://example.com/bizstep/my-special-step".to_string());
```

## License

Licensed under either of Apache-2.0 or MIT.
