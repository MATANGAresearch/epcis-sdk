# epcis-translate

High-performance, zero-allocation bidirectional translators for GS1 identifiers (SGTIN, SSCC, SGLN, GRAI, GIAI) between EPC URN and GS1 Digital Link path formats.

## Features

- **Zero-Allocation Parsing**: Leverages lifetime-bound string slices (`&'a str`) to prevent heap allocation during parser operations.
- **Check-Digit Verification**: Includes built-in Luhn checksum calculations for GS1 identifiers.
- **Complete Bidirectional Translations**: Converts both *to* and *from* EPC URN and GS1 Digital Link formats.

## Quick Start

```rust
use epcis_translate::Sgtin;

fn main() {
    // Parse an SGTIN EPC URN
    let urn = "urn:epc:id:sgtin:4012345.098765.12345";
    let sgtin = Sgtin::from_urn(urn).unwrap();
    
    assert_eq!(sgtin.company_prefix, "4012345");
    assert_eq!(sgtin.indicator, "0");
    assert_eq!(sgtin.item_ref, "98765");
    assert_eq!(sgtin.serial_number, "12345");
    
    // Translate to GS1 Digital Link format
    let digital_link = sgtin.to_digital_link("https://id.gs1.org");
    println!("Digital Link: {}", digital_link);
    // Outputs: "https://id.gs1.org/01/04012345987652/21/12345"
}
```

## Supported Translators

- **SGTIN** (Serialized Global Trade Item Number)
- **SSCC** (Serial Shipping Container Code)
- **SGLN** (Global Location Number)
- **GRAI** (Global Returnable Asset Identifier)
- **GIAI** (Global Individual Asset Identifier)

## License

Licensed under either of Apache-2.0 or MIT.
