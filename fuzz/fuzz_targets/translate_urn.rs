//! All URN parsers must return Err on malformed input, never panic.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        use epcis_translate::*;
        let _ = Sgtin::from_urn(s).map(|k| (k.to_urn(), k.to_digital_link("https://id.gs1.org")));
        let _ = Sscc::from_urn(s).map(|k| (k.to_urn(), k.to_digital_link("https://id.gs1.org")));
        let _ = Sgln::from_urn(s).map(|k| (k.to_urn(), k.to_digital_link("https://id.gs1.org")));
        let _ = Grai::from_urn(s).map(|k| (k.to_urn(), k.to_digital_link("https://id.gs1.org")));
        let _ = Giai::from_urn(s).map(|k| (k.to_urn(), k.to_digital_link("https://id.gs1.org")));
        let _ = Pgln::from_urn(s).map(|k| (k.to_urn(), k.to_digital_link("https://id.gs1.org")));
        let _ = Gdti::from_urn(s).map(|k| (k.to_urn(), k.to_digital_link("https://id.gs1.org")));
        let _ = Gsrn::from_urn(s).map(|k| (k.to_urn(), k.to_digital_link("https://id.gs1.org")));
        let _ = Gsrnp::from_urn(s).map(|k| (k.to_urn(), k.to_digital_link("https://id.gs1.org")));
        let _ = Sgcn::from_urn(s).map(|k| (k.to_urn(), k.to_digital_link("https://id.gs1.org")));
        let _ = Ginc::from_urn(s).map(|k| (k.to_urn(), k.to_digital_link("https://id.gs1.org")));
        let _ = Gsin::from_urn(s).map(|k| (k.to_urn(), k.to_digital_link("https://id.gs1.org")));
        let _ = Itip::from_urn(s).map(|k| (k.to_urn(), k.to_digital_link("https://id.gs1.org")));
        let _ = Upui::from_urn(s).map(|k| (k.to_urn(), k.to_digital_link("https://id.gs1.org")));
        let _ = Cpi::from_urn(s).map(|k| (k.to_urn(), k.to_digital_link("https://id.gs1.org")));
        let _ = Lgtin::from_urn(s).map(|k| (k.to_urn(), k.to_digital_link("https://id.gs1.org")));
    }
});
