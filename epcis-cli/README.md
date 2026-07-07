# `epcis-cli`

A command line utility for working with GS1 EPCIS 2.0 events. It supports generating canonical pre-hashes, final SHA-256 hashes, and translating GS1 visibility keys between EPC URN and Digital Link formats.

---

## Installation

To compile and install the CLI tool locally from the workspace:

```bash
cargo install --path .
```

---

## Usage

```text
Usage: epcis-cli [OPTIONS] [FILE]

Arguments:
  [FILE]  File path to parse (use "-" or omit to read from standard input)

Options:
  -p, --prehash          Print the canonical pre-hash string instead of the hash URN
  -e, --enforce-format   Force input format: "json" or "xml"
  -t, --translate <KEY>  Translate a GS1 key between EPC URN and Digital Link formats
  -l, --prefix-len <LEN> The company prefix length to use for Digital Link translation [default: 7]
      --cbv-1-2          Use legacy CBV 1.2 rules (omits user extensions from hashing)
      --base-url <URL>   The base URL to use when translating URN to Digital Link [default: https://id.gs1.org]
  -h, --help             Print help
  -V, --version          Print version
```

---

## Examples

### 1. Translate GS1 Identifiers

* **URN to Digital Link**:
  ```bash
  epcis-cli --translate "urn:epc:id:sgtin:4012345.098765.12345"
  # Output: https://id.gs1.org/01/04012345987652/21/12345
  ```

* **Digital Link to URN** (using default company prefix length of 7):
  ```bash
  epcis-cli --translate "https://id.gs1.org/01/04012345987652/21/12345"
  # Output: urn:epc:id:sgtin:4012345.098765.12345
  ```

* **Digital Link to URN** (specifying custom company prefix length of 10):
  ```bash
  epcis-cli -l 10 --translate "https://id.gs1.org/01/04012345678904/21/12345"
  # Output: urn:epc:id:sgtin:4012345678.90.12345
  ```

---

### 2. Generate Canonical Pre-hashes

* **From JSON-LD Document**:
  ```bash
  epcis-cli -p document.jsonld
  ```

* **From XML Document piped through Stdin**:
  ```bash
  cat document.xml | epcis-cli -p
  ```

---

### 3. Generate SHA-256 Hash URNs

* **From any EPCIS document**:
  ```bash
  epcis-cli document.xml
  # Output URN: ni:///sha-256;...
  ```
