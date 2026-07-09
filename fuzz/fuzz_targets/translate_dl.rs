//! All Digital Link parsers must return Err on malformed input, never panic,
//! for any prefix length.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Some((&len_byte, rest)) = data.split_first() else {
        return;
    };
    let prefix_len = usize::from(len_byte) % 20;
    if let Ok(s) = std::str::from_utf8(rest) {
        use epcis_translate::*;
        let _ = Sgtin::from_digital_link(s, prefix_len).map(|k| k.to_urn());
        let _ = Sscc::from_digital_link(s, prefix_len).map(|k| k.to_urn());
        let _ = Sgln::from_digital_link(s, prefix_len).map(|k| k.to_urn());
        let _ = Grai::from_digital_link(s, prefix_len).map(|k| k.to_urn());
        let _ = Giai::from_digital_link(s, prefix_len).map(|k| k.to_urn());
        let _ = Pgln::from_digital_link(s, prefix_len).map(|k| k.to_urn());
        let _ = Gdti::from_digital_link(s, prefix_len).map(|k| k.to_urn());
        let _ = Gsrn::from_digital_link(s, prefix_len).map(|k| k.to_urn());
        let _ = Gsrnp::from_digital_link(s, prefix_len).map(|k| k.to_urn());
        let _ = Sgcn::from_digital_link(s, prefix_len).map(|k| k.to_urn());
        let _ = Ginc::from_digital_link(s, prefix_len).map(|k| k.to_urn());
        let _ = Gsin::from_digital_link(s, prefix_len).map(|k| k.to_urn());
        let _ = Itip::from_digital_link(s, prefix_len).map(|k| k.to_urn());
        let _ = Upui::from_digital_link(s, prefix_len).map(|k| k.to_urn());
        let _ = Cpi::from_digital_link(s, prefix_len).map(|k| k.to_urn());
        let _ = Lgtin::from_digital_link(s, prefix_len).map(|k| k.to_urn());
    }
});
