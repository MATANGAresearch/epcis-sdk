# Fuzzing

libFuzzer targets for the SDK's untrusted-input surfaces. The crate is
excluded from the workspace; it needs a nightly toolchain and
[`cargo-fuzz`](https://github.com/rust-fuzz/cargo-fuzz).

```bash
cargo install cargo-fuzz
rustup toolchain install nightly
cargo +nightly fuzz run <target> -- -max_total_time=60
```

| Target          | Surface                                                     |
| --------------- | ----------------------------------------------------------- |
| `translate_urn` | every `*::from_urn` parser in `epcis-translate`             |
| `translate_dl`  | every `*::from_digital_link` parser (first byte = prefix len) |
| `hash_json`     | `epcis_hash::canonicalize_json` on arbitrary JSON            |
| `hash_xml`      | `epcis_hash::canonicalize_xml` on arbitrary XML              |
| `models_xml`    | `EPCISDocument`/`EPCISQueryDocument::from_xml` + re-serialization |

The invariant in every target: malformed input returns `Err`, never panics.
Seed corpora live in `corpus/<target>/` (gitignored); reference test vectors
make good seeds.
