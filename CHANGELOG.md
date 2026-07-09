# Changelog

All notable changes to this workspace are documented here. Versions follow
[Semantic Versioning](https://semver.org); while below 1.0.0, minor bumps may
contain breaking changes.

## 0.2.0 — 2026-07-09

This release fixes correctness bugs found in a full audit against the official
GS1/OpenEPCIS reference test vectors. Several fixes change serialized output
or canonical hashes, hence the minor version bump.

### Hash-affecting fixes (epcis-models, epcis-hash)

- **`transformationID` serialization** — `TransformationEvent::transformation_id`
  now serializes as `transformationID` per the EPCIS 2.0 JSON schema (it was
  `transformationId`). Typed `TransformationEvent`s previously produced
  canonical hashes that differed from equivalent spec-compliant JSON
  documents; they now match. Deserializing spec JSON also populates the field
  instead of routing it into `extensions`.
- **XML default-namespace handling** — pre-hash emission strips the EPCIS XML
  namespace from container names and the event type, so documents using a
  default `xmlns` (e.g. `xmlns="urn:epcglobal:epcis:xsd:2"`) hash identically
  to prefix-namespaced and JSON representations. Previously such documents
  produced incorrect hashes containing Clark-notation names.
- **`compute_canonical_hash` unified with `canonicalize_json`** — typed events
  and raw JSON now share one canonicalization pipeline, including bare-
  extension bubbling and ignore-field handling.
- **inf/nan text normalization** — values such as `"nan"` are no longer
  mutated (previously `"nan"` became `"NaN"` via the float path); the
  reference implementation leaves them untouched. Note: numeric values are
  intentionally still canonicalized through a 64-bit float, including
  precision loss above 2^53 — this matches the reference implementation and
  its published test vectors.

### Breaking model changes (epcis-models)

- **`SensorReport`** now covers the full EPCIS 2.0 field set: added
  `exception`, `device_id`, `device_metadata`, `raw_data`,
  `data_processing_method`, `microorganism`, `component`, `min_value`,
  `max_value`, `mean_value`, `s_dev`, `perc_rank`, `perc_value`,
  `coordinate_reference_system`, plus a flattened `extensions` map. `type` is
  now `Option<String>` (optional when `exception` is present). Removed the
  non-spec fields `sensor_processor`, `data_value`, and `microsecond_offset`.
  Previously, standard fields absent from the struct were silently dropped on
  round-trip.
- **`SensorMetadata`** uses spec field names `deviceMetadata`, `rawData`,
  `bizRules` (previously `deviceMetadataURI`, `rawDataURI`, `bizRulesURI`;
  `dataContentURI` removed), and gains `start_time`, `end_time`,
  `data_processing_method`, and a flattened `extensions` map.
- **`ilmd`** is now a typed field on `ObjectEvent` and `TransformationEvent`.

### Breaking behavior changes (epcis-translate)

- All `from_urn`/`from_digital_link` parsers validate that numeric identifier
  segments are ASCII digits and return `ParseError::InvalidFormat` instead of
  panicking on multi-byte UTF-8 input or silently accepting non-numeric
  identifiers. Serial numbers, extensions, and asset references remain
  free-form per GS1.
- `Sgln::to_digital_link` omits the `/254/` qualifier when the extension is
  `"0"` (GS1 canonical form, matching `epcis-hash`), and
  `Sgln::from_digital_link` accepts a plain `/414/GLN` without a qualifier,
  defaulting the extension to `"0"`.

### Native EPCIS 2.0 XML support (epcis-models)

- `EPCISDocument::from_xml` now parses **standard EPCIS 2.0 XML documents**
  (`<epcis:EPCISDocument>` with `<EPCISBody><EventList>` structure, sensor
  data as XML attributes, `type` attributes on business transactions, ilmd,
  error declarations, and foreign-namespace extension elements whose prefix
  declarations are carried into the JSON-LD `@context`). `to_xml` emits
  standard EPCIS 2.0 XML with XSD-ordered event children. The previous
  internal quick-xml round-trip format is gone. Verified against every
  official XML test vector: parsing to typed models and re-serializing both
  preserve the exact canonical pre-hash. Known limitation: `EPCISHeader`
  master data in XML form is not yet mapped.
- `ReadPoint`, `BizLocation`, `SensorElement`, and `ErrorDeclaration` gained
  flattened `extensions` maps (the spec permits extension elements inside
  all four), so such fields survive typed round-trips.

### Other

- `epcis-cli --version` now reports the workspace version (was hardcoded
  `0.1.2`).
- CI now clones the pinned reference-vector repository before testing (the
  compliance suite previously could not run in CI), and gains wasm-build,
  bench-compile, and MSRV (1.88) check steps.
- Removed unused `url` and `uuid` dependencies from `epcis-models`.
- Added `LICENSE-MIT` and `LICENSE-APACHE` texts; `rust-version` relaxed from
  1.96.1 to 1.88 (verified by building with a 1.88 toolchain).
- The compliance test now exercises every reference vector. The three vectors
  whose upstream `.prehashes` files are corrupted are verified via their
  `.hashes` files, which all match.
- Workspace is clippy-pedantic clean and rustfmt-formatted; CI enforces both.
