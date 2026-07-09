# GS1 EPCIS 2.0 Rust SDK

A high-performance, developer-friendly, and native Rust SDK for working with the GS1 EPCIS 2.0 visibility events standard. 

This SDK implements type-safe schema models, deterministic canonical event hashing, and zero-allocation GS1 identifier translations. It is designed to be strict, efficient, and fully interoperable with modern supply chain ledgers and APIs.

---

## What is EPCIS 2.0?

**EPCIS (Electronic Product Code Information Services)** is a global standard (ISO/IEC 19987) for capturing and sharing visibility events across supply chains. It acts as a common language for tracking the physical movement and status of goods.

Every EPCIS event answers the **5 Ws and How**:
1. **What**: The products or assets involved (expressed via `epcList` or `quantityList` in `epcis-models`).
2. **When**: The date, time, and timezone offset of the event (`eventTime`, `eventTimeZoneOffset`).
3. **Where**: The physical location where it occurred (`readPoint`, `bizLocation`).
4. **Why**: The business context (`bizStep` and `disposition`).
5. **How (IoT Sensors)**: Standardized sensor readings (e.g. temperature, humidity, coordinate systems) to track environmental conditions during transit.

---

## Workspace Crates

This workspace consists of four modular crates:

1. **[`epcis-models`](./epcis-models)**: Type-safe event models (`ObjectEvent`, `AggregationEvent`, `TransformationEvent`, etc.) and strum-backed CBV (Core Business Vocabulary) enum systems. Fully serializable to and from JSON/JSON-LD. (An internal XML round-trip format is provided; parsing standard EPCIS 2.0 XML documents into typed models is planned — `epcis-hash` already consumes standard EPCIS XML for hashing.)
2. **[`epcis-hash`](./epcis-hash)**: Deterministic, representation-agnostic canonical SHA-256 event hashing conforming to GS1 and `OpenEPCIS` specifications. Supports both XML and JSON/JSON-LD input payloads.
3. **[`epcis-translate`](./epcis-translate)**: Zero-allocation, bidirectional translators for converting GS1 keys (SGTIN, SSCC, SGLN, GRAI, GIAI) between EPC URN and GS1 Digital Link path formats without heap allocation.
4. **[`epcis-cli`](./epcis-cli)**: A command line binary wrapper for running event hashing and GS1 identifier translation directly from the terminal.

---

## Why Hashing & Canonicalization?

In modern supply chain architectures (especially distributed shared ledgers, blockchains, or cloud-native event-sourcing databases), unique and immutable references to events are required:
* **Tamper Proofing / Notarization**: Saving only the event hash on-chain preserves privacy while proving that the underlying data has not been modified.
* **Deduplication / Idempotency**: Automatically generating unique event IDs based on their semantic content to prevent duplicate records from ingestion glitches.
* **Error Declarations**: Referencing the original event’s hash to declare correction events.

Because events can be serialized in different ways (differing whitespaces, key ordering, XML vs. JSON-LD, compact URIs vs. bare words), they must first be transformed into a **pre-hash string** using strict canonical rules before hashing.

### Supported Formats & Versions
* **Input Formats**: Generates canonical pre-hash strings from both **JSON-LD/JSON** (via `canonicalize_json`) and **XML** documents (via `canonicalize_xml`).
* **Standard Versions**: Supports both the latest **CBV 2.0 / 2.1** standards (including user extensions, GS1 Digital Link conversions, and Web URI dictionary expansions) and legacy **CBV 1.2 / EPCIS 1.2** standards (where user extensions are excluded from canonical hashing). You can toggle between these modes using the `is_cbv_2_0` boolean parameter.

### Canonical Property Order (CBV 2.0 / 2.1)
The algorithm concatenates elements in this exact sequence:
1. `eventType`
2. `eventTime`
3. `eventTimeZoneOffset`
4. `epcList` - `epc` (sorted)
5. `parentID`
6. `inputEPCList` - `epc` (sorted)
7. `childEPCs` - `epc` (sorted)
8. `quantityList`
9. `childQuantityList`
10. `inputQuantityList`
11. `outputEPCList` - `epc` (sorted)
12. `outputQuantityList`
13. `action`
14. `transformationID`
15. `bizStep`
16. `disposition`
17. `persistentDisposition`
18. `readPoint` - `id`
19. `bizLocation` - `id`
20. `bizTransactionList`
21. `sourceList`
22. `destinationList`
23. `sensorElementList`
24. `ilmd` (user extensions)
25. User extensions at event level (sorted namespaces)

---

## Walkthrough: JSON-LD Event to Canonical Hash

Here is an example showing how a raw JSON-LD event payload is canonicalized and hashed by the SDK.

### 1. Raw JSON-LD Input
```json
{
  "@context": [
    "https://ref.gs1.org/standards/epcis/2.0.0/epcis-context.jsonld",
    { "gs1": "https://gs1.org/voc/", "example": "https://ns.example.com/epcis/" }
  ],
  "type": "ObjectEvent",
  "eventTime": "2020-03-04T11:00:30.000+01:00",
  "eventTimeZoneOffset": "+01:00",
  "recordTime": "2020-03-04T11:00:30.999+01:00",
  "epcList": [
    "urn:epc:id:sscc:4012345.0000000333",
    "urn:epc:id:sscc:4012345.0000000111",
    "urn:epc:id:sscc:4012345.0000000222"
  ],
  "action": "OBSERVE",
  "bizStep": "departing",
  "readPoint": { "id": "urn:epc:id:sgln:4012345.00011.987" },
  "example:myField1": {
    "example:mySubField1": "2",
    "example:mySubField2": "5"
  }
}
```

### 2. Semantic Canonicalization Behavior
During processing:
* **Time Conversion**: `eventTime` is converted from localized offset `11:00:30+01:00` to UTC `10:00:30.000Z`.
* **Metadata Exclusion**: `recordTime` and metadata properties are stripped out.
* **Lexicographical Sorting**: `epcList` items are sorted alphabetically by their numeric segments (e.g. `111` first, then `222`, then `333`).
* **Digital Link Conversion**: EPC URNs (like `urn:epc:id:sscc:...`) are parsed and converted to canonical GS1 Digital Link URIs (like `https://id.gs1.org/00/...` with check-digits calculated).
* **Vocabulary Normalization**: Bare step value `"departing"` is expanded to the official Web URI `"https://ref.gs1.org/cbv/BizStep-departing"`.
* **Namespace Expansion**: Custom extensions are prefixed with their URI namespaces.

### 3. Intermediate Pre-Hash String
The canonical string passed to the SHA-256 hasher:
```text
eventType=ObjectEventeventTime=2020-03-04T10:00:30.000ZeventTimeZoneOffset=+01:00epcListepc=https://id.gs1.org/00/040123450000001112epc=https://id.gs1.org/00/040123450000002225epc=https://id.gs1.org/00/040123450000003338action=OBSERVEbizStep=https://ref.gs1.org/cbv/BizStep-departingreadPointid=https://id.gs1.org/414/4012345000115/254/987{https://ns.example.com/epcis/}myField1{https://ns.example.com/epcis/}mySubField1=2{https://ns.example.com/epcis/}mySubField2=5
```

### 4. Final URN Hash
```text
ni:///sha-256;6ae96341e0acc6d7a261364751f60e68278a81cdf51da0abb6b4e617014e39d7?ver=CBV2.0
```

---

## Installation & Getting Started

Add the crates to your `Cargo.toml`:

```toml
[dependencies]
epcis-models = "0.2.0"
epcis-hash = "0.2.0"
epcis-translate = "0.2.0"
```

### End-to-End Rust Example

Here is how you parse a Digital Link identifier, construct a type-safe ObjectEvent, and calculate its canonical hash:

```rust
use epcis_models::{
    Action, BizStep, Disposition, EPCISEvent, Epc, ObjectEvent,
    StandardBizStep, StandardDisposition,
};
use epcis_hash::compute_canonical_hash;
use epcis_translate::Sgtin;
use chrono::Utc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Translate SGTIN from Digital Link to EPC URN format (zero-allocation parsing)
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

---

## Command Line Interface (`epcis-cli`)

We provide a built-in command line utility for hashing documents and translating keys directly from the terminal.

### Installation
To compile and install the CLI tool locally from the workspace:
```bash
cargo install --path epcis-cli
```

### Usage Examples
* **Translate an SGTIN EPC URN to GS1 Digital Link**:
  ```bash
  epcis-cli --translate "urn:epc:id:sgtin:4012345.098765.12345"
  ```
* **Translate a GS1 Digital Link URL back to EPC URN**:
  ```bash
  epcis-cli --translate "https://id.gs1.org/01/04012345987652/21/12345" -l 7
  ```
* **Generate Canonical Pre-hashes from an XML or JSON-LD document**:
  ```bash
  epcis-cli -p document.xml
  # Or pipe via stdin
  cat document.jsonld | epcis-cli -p
  ```
* **Generate final SHA-256 Hash URNs**:
  ```bash
  epcis-cli document.xml
  ```

---

## Running Tests

To verify correctness and run compliance checks against the official test vectors:

```bash
cargo test
```

## License

Licensed under either of Apache-2.0 or MIT.
