#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![deny(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

use epcis_models::EPCISEvent;
use quick_xml::Reader;
use quick_xml::events::Event;
use regex::Regex;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::borrow::Cow;
use std::collections::BTreeMap;

pub mod error;
pub use error::EpcisHashError;

use std::sync::LazyLock;

static SGTIN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^urn:epc:id:sgtin:(\d+)\.(\d+)\.([^\n\r]+)$").expect("valid regex")
});
static SSCC_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^urn:epc:id:sscc:(\d+)\.(\d+)$").expect("valid regex"));
static SGLN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^urn:epc:id:sgln:(\d+)\.(\d*)\.([^\n\r]+)$").expect("valid regex")
});
static GRAI_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^urn:epc:id:grai:(\d+)\.(\d*)\.([^\n\r]+)$").expect("valid regex")
});
static GIAI_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^urn:epc:id:giai:(\d+)\.([^\n\r]+)$").expect("valid regex"));
static PGLN_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^urn:epc:id:pgln:(\d+)\.(\d*)$").expect("valid regex"));
static LGTIN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^urn:epc:class:lgtin:(\d+)\.(\d+)\.([^\n\r]+)$").expect("valid regex")
});
static IDPAT_SGTIN_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^urn:epc:idpat:sgtin:(\d+)\.(\d+)\.\*$").expect("valid regex"));
static IDPAT_GRAI_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^urn:epc:idpat:grai:(\d+)\.(\d*)\.\*$").expect("valid regex"));
static IDPAT_GDTI_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^urn:epc:idpat:gdti:(\d+)\.(\d*)\.\*$").expect("valid regex"));
static IDPAT_SGCN_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^urn:epc:idpat:sgcn:(\d+)\.(\d*)\.\*$").expect("valid regex"));
static IDPAT_CPI_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^urn:epc:idpat:cpi:(\d+)\.([0-9a-zA-Z%-]+)\.\*$").expect("valid regex")
});
static IDPAT_ITIP_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^urn:epc:idpat:itip:(\d+)\.(\d+)\.(\d+)\.(\d+)\.\*$").expect("valid regex")
});
static IDPAT_UPUI_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^urn:epc:idpat:upui:(\d+)\.(\d*)\.\*$").expect("valid regex"));
static GSRN_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^urn:epc:id:gsrn:(\d+)\.(\d*)$").expect("valid regex"));
static GSRNP_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^urn:epc:id:gsrnp:(\d+)\.(\d*)$").expect("valid regex"));
static GDTI_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^urn:epc:id:gdti:(\d+)\.(\d*)\.([^\n\r]+)$").expect("valid regex")
});
static CPI_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^urn:epc:id:cpi:(\d+)\.([^\n\r]+)\.(\d+)$").expect("valid regex")
});
static SGCN_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^urn:epc:id:sgcn:(\d+)\.(\d*)\.(\d+)$").expect("valid regex"));
static GINC_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^urn:epc:id:ginc:(\d+)\.([^\n\r]+)$").expect("valid regex"));
static GSIN_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^urn:epc:id:gsin:(\d+)\.(\d*)$").expect("valid regex"));
static ITIP_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^urn:epc:id:itip:(\d+)\.(\d+)\.(\d+)\.(\d+)\.([^\n\r]+)$").expect("valid regex")
});
static UPUI_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^urn:epc:id:upui:(\d+)\.(\d+)\.([^\n\r]+)$").expect("valid regex")
});

/// Node representing an EPCIS event element.
#[derive(Debug, Clone)]
pub struct ContextNode {
    /// Element name.
    pub name: Option<String>,
    /// Primitive value string.
    pub value: Option<String>,
    /// Child nodes.
    pub children: Vec<ContextNode>,
}

fn calculate_check_digit(digits: &str) -> u32 {
    let mut sum = 0;
    for (i, char) in digits.chars().rev().enumerate() {
        if let Some(digit) = char.to_digit(10) {
            if i % 2 == 0 {
                sum += digit * 3;
            } else {
                sum += digit;
            }
        }
    }
    let remainder = sum % 10;
    if remainder == 0 { 0 } else { 10 - remainder }
}

fn percent_encode(input: &str) -> String {
    input
        .replace('!', "%21")
        .replace('(', "%28")
        .replace(')', "%29")
        .replace('*', "%2A")
        .replace('+', "%2B")
        .replace(',', "%2C")
        .replace(':', "%3A")
}

fn normalize_dl_url(uri: &str) -> Option<String> {
    if !uri.starts_with("http://") && !uri.starts_with("https://") {
        return None;
    }

    let mut clean_uri = match uri.find('?') {
        Some(idx) => &uri[..idx],
        None => uri,
    }
    .to_string();

    clean_uri = clean_uri
        .replace("/gtin/", "/01/")
        .replace("/itip/", "/8006/")
        .replace("/cpid/", "/8010/")
        .replace("/gln/", "/414/")
        .replace("/party/", "/417/")
        .replace("/gsrnp/", "/8017/")
        .replace("/gsrn/", "/8018/")
        .replace("/gcn/", "/255/")
        .replace("/sscc/", "/00/")
        .replace("/gdti/", "/253/")
        .replace("/ginc/", "/401/")
        .replace("/gsin/", "/402/")
        .replace("/grai/", "/8003/")
        .replace("/giai/", "/8004/")
        .replace("/cpv/", "/22/")
        .replace("/lot/", "/10/")
        .replace("/ser/", "/21/");

    let ais = [
        "/00/", "/01/", "/253/", "/255/", "/401/", "/402/", "/414/", "/417/", "/8003/", "/8004/",
        "/8006/", "/8010/", "/8017/", "/8018/",
    ];
    let mut mapped = false;
    for ai in &ais {
        if let Some(idx) = clean_uri.find(ai) {
            clean_uri = format!("https://id.gs1.org{}", &clean_uri[idx..]);
            mapped = true;
            break;
        }
    }

    if !mapped {
        return None;
    }

    if clean_uri.starts_with("https://id.gs1.org/01/") {
        let rest = &clean_uri["https://id.gs1.org/01/".len()..];
        let gtin_part = match rest.find('/') {
            Some(idx) => &rest[..idx],
            None => rest,
        };
        if gtin_part.len() == 13 {
            clean_uri = clean_uri.replacen("/01/", "/01/0", 1);
        } else if gtin_part.len() == 12 {
            clean_uri = clean_uri.replacen("/01/", "/01/00", 1);
        } else if gtin_part.len() == 8 {
            clean_uri = clean_uri.replacen("/01/", "/01/000000", 1);
        }
    }

    if let Some(idx_10) = clean_uri.find("/10/")
        && let Some(idx_21) = clean_uri.find("/21/")
        && idx_21 > idx_10
    {
        let after_10 = &clean_uri[idx_10 + 4..];
        if let Some(slash_idx) = after_10.find('/') {
            clean_uri = format!("{}{}", &clean_uri[..idx_10], &after_10[slash_idx..]);
        }
    }

    if let Some(idx_22) = clean_uri.find("/22/") {
        let after_22 = &clean_uri[idx_22 + 4..];
        if let Some(slash_idx) = after_22.find('/') {
            clean_uri = format!("{}{}", &clean_uri[..idx_22], &after_22[slash_idx..]);
        } else {
            clean_uri = clean_uri[..idx_22].to_string();
        }
    }

    Some(clean_uri)
}

/// Normalizes an EPC URN or Digital Link URL.
// One arm per GS1 key scheme; splitting would obscure the dispatch table.
// The regex-capture unwraps cannot fail: every group is non-optional in its
// pattern, so a successful match guarantees their presence.
#[allow(clippy::too_many_lines, clippy::missing_panics_doc)]
#[must_use]
pub fn normalise_uri(uri: &str) -> String {
    if let Some(caps) = SGTIN_RE.captures(uri) {
        let cp = caps.get(1).unwrap().as_str();
        let r = caps.get(2).unwrap().as_str();
        let serial = caps.get(3).unwrap().as_str();
        let raw_gtin = format!("{}{}{}", &r[0..1], cp, &r[1..]);
        let cd = calculate_check_digit(&raw_gtin);
        format!(
            "https://id.gs1.org/01/{}{}/21/{}",
            raw_gtin,
            cd,
            percent_encode(serial)
        )
    } else if let Some(caps) = SSCC_RE.captures(uri) {
        let cp = caps.get(1).unwrap().as_str();
        let ref_part = caps.get(2).unwrap().as_str();
        let ext = &ref_part[0..1];
        let serial_ref = &ref_part[1..];
        let raw_sscc = format!("{ext}{cp}{serial_ref}");
        let cd = calculate_check_digit(&raw_sscc);
        format!("https://id.gs1.org/00/{raw_sscc}{cd}")
    } else if let Some(caps) = SGLN_RE.captures(uri) {
        let cp = caps.get(1).unwrap().as_str();
        let loc = caps.get(2).unwrap().as_str();
        let ext = caps.get(3).unwrap().as_str();
        let raw_gln = format!("{cp}{loc}");
        let cd = calculate_check_digit(&raw_gln);
        if ext == "0" {
            format!("https://id.gs1.org/414/{raw_gln}{cd}")
        } else {
            format!(
                "https://id.gs1.org/414/{}{}/254/{}",
                raw_gln,
                cd,
                percent_encode(ext)
            )
        }
    } else if let Some(caps) = GRAI_RE.captures(uri) {
        let cp = caps.get(1).unwrap().as_str();
        let asset = caps.get(2).unwrap().as_str();
        let serial = caps.get(3).unwrap().as_str();
        let raw_grai = format!("0{cp}{asset}");
        let cd = calculate_check_digit(&raw_grai);
        format!(
            "https://id.gs1.org/8003/{}{}{}",
            raw_grai,
            cd,
            percent_encode(serial)
        )
    } else if let Some(caps) = GIAI_RE.captures(uri) {
        let cp = caps.get(1).unwrap().as_str();
        let asset = caps.get(2).unwrap().as_str();
        format!("https://id.gs1.org/8004/{}{}", cp, percent_encode(asset))
    } else if let Some(caps) = PGLN_RE.captures(uri) {
        let cp = caps.get(1).unwrap().as_str();
        let party = caps.get(2).unwrap().as_str();
        let raw_gln = format!("{cp}{party}");
        let cd = calculate_check_digit(&raw_gln);
        format!("https://id.gs1.org/417/{raw_gln}{cd}")
    } else if let Some(caps) = LGTIN_RE.captures(uri) {
        let cp = caps.get(1).unwrap().as_str();
        let r = caps.get(2).unwrap().as_str();
        let lot = caps.get(3).unwrap().as_str();
        let raw_gtin = format!("{}{}{}", &r[0..1], cp, &r[1..]);
        let cd = calculate_check_digit(&raw_gtin);
        format!(
            "https://id.gs1.org/01/{}{}/10/{}",
            raw_gtin,
            cd,
            percent_encode(lot)
        )
    } else if let Some(caps) = IDPAT_SGTIN_RE.captures(uri) {
        let cp = caps.get(1).unwrap().as_str();
        let r = caps.get(2).unwrap().as_str();
        let raw_gtin = format!("{}{}{}", &r[0..1], cp, &r[1..]);
        let cd = calculate_check_digit(&raw_gtin);
        format!("https://id.gs1.org/01/{raw_gtin}{cd}")
    } else if let Some(caps) = GSRN_RE.captures(uri) {
        let cp = caps.get(1).unwrap().as_str();
        let service = caps.get(2).unwrap().as_str();
        let raw_gln = format!("{cp}{service}");
        let cd = calculate_check_digit(&raw_gln);
        format!("https://id.gs1.org/8018/{raw_gln}{cd}")
    } else if let Some(caps) = GSRNP_RE.captures(uri) {
        let cp = caps.get(1).unwrap().as_str();
        let service = caps.get(2).unwrap().as_str();
        let raw_gln = format!("{cp}{service}");
        let cd = calculate_check_digit(&raw_gln);
        format!("https://id.gs1.org/8017/{raw_gln}{cd}")
    } else if let Some(caps) = GDTI_RE.captures(uri) {
        let cp = caps.get(1).unwrap().as_str();
        let doc = caps.get(2).unwrap().as_str();
        let serial = caps.get(3).unwrap().as_str();
        let raw_gln = format!("{cp}{doc}");
        let cd = calculate_check_digit(&raw_gln);
        format!(
            "https://id.gs1.org/253/{}{}{}",
            raw_gln,
            cd,
            percent_encode(serial)
        )
    } else if let Some(caps) = CPI_RE.captures(uri) {
        let cp = caps.get(1).unwrap().as_str();
        let cpid = caps.get(2).unwrap().as_str();
        let serial = caps.get(3).unwrap().as_str();
        format!(
            "https://id.gs1.org/8010/{}/8011/{}",
            percent_encode(&format!("{cp}{cpid}")),
            serial
        )
    } else if let Some(caps) = SGCN_RE.captures(uri) {
        let cp = caps.get(1).unwrap().as_str();
        let coupon = caps.get(2).unwrap().as_str();
        let serial = caps.get(3).unwrap().as_str();
        let raw_gln = format!("{cp}{coupon}");
        let cd = calculate_check_digit(&raw_gln);
        format!("https://id.gs1.org/255/{raw_gln}{cd}{serial}")
    } else if let Some(caps) = IDPAT_GRAI_RE.captures(uri) {
        let cp = caps.get(1).unwrap().as_str();
        let asset = caps.get(2).unwrap().as_str();
        let raw_grai = format!("0{cp}{asset}");
        let cd = calculate_check_digit(&raw_grai);
        format!("https://id.gs1.org/8003/{raw_grai}{cd}")
    } else if let Some(caps) = IDPAT_GDTI_RE.captures(uri) {
        let cp = caps.get(1).unwrap().as_str();
        let doc = caps.get(2).unwrap().as_str();
        let raw_gdti = format!("{cp}{doc}");
        let cd = calculate_check_digit(&raw_gdti);
        format!("https://id.gs1.org/253/{raw_gdti}{cd}")
    } else if let Some(caps) = IDPAT_SGCN_RE.captures(uri) {
        let cp = caps.get(1).unwrap().as_str();
        let coupon = caps.get(2).unwrap().as_str();
        let raw_sgcn = format!("{cp}{coupon}");
        let cd = calculate_check_digit(&raw_sgcn);
        format!("https://id.gs1.org/255/{raw_sgcn}{cd}")
    } else if let Some(caps) = IDPAT_CPI_RE.captures(uri) {
        let cp = caps.get(1).unwrap().as_str();
        let cpid = caps.get(2).unwrap().as_str();
        format!(
            "https://id.gs1.org/8010/{}",
            percent_encode(&format!("{cp}{cpid}"))
        )
    } else if let Some(caps) = IDPAT_ITIP_RE.captures(uri) {
        let cp = caps.get(1).unwrap().as_str();
        let item = caps.get(2).unwrap().as_str();
        let piece = caps.get(3).unwrap().as_str();
        let total = caps.get(4).unwrap().as_str();
        let raw_gtin = format!("{}{}{}", &item[0..1], cp, &item[1..]);
        let cd = calculate_check_digit(&raw_gtin);
        format!("https://id.gs1.org/8006/{raw_gtin}{cd}{piece}{total}")
    } else if let Some(caps) = IDPAT_UPUI_RE.captures(uri) {
        let cp = caps.get(1).unwrap().as_str();
        let item = caps.get(2).unwrap().as_str();
        let raw_gtin = format!("{}{}{}", &item[0..1], cp, &item[1..]);
        let cd = calculate_check_digit(&raw_gtin);
        format!("https://id.gs1.org/01/{raw_gtin}{cd}")
    } else if let Some(caps) = GINC_RE.captures(uri) {
        let cp = caps.get(1).unwrap().as_str();
        let consignment = caps.get(2).unwrap().as_str();
        format!(
            "https://id.gs1.org/401/{}{}",
            cp,
            percent_encode(consignment)
        )
    } else if let Some(caps) = GSIN_RE.captures(uri) {
        let cp = caps.get(1).unwrap().as_str();
        let shipper = caps.get(2).unwrap().as_str();
        let raw_gln = format!("{cp}{shipper}");
        let cd = calculate_check_digit(&raw_gln);
        format!("https://id.gs1.org/402/{raw_gln}{cd}")
    } else if let Some(caps) = ITIP_RE.captures(uri) {
        let cp = caps.get(1).unwrap().as_str();
        let item = caps.get(2).unwrap().as_str();
        let piece = caps.get(3).unwrap().as_str();
        let total = caps.get(4).unwrap().as_str();
        let serial = caps.get(5).unwrap().as_str();
        let raw_gtin = format!("{}{}{}", &item[0..1], cp, &item[1..]);
        let cd = calculate_check_digit(&raw_gtin);
        format!(
            "https://id.gs1.org/8006/{}{}{}{}/21/{}",
            raw_gtin,
            cd,
            piece,
            total,
            percent_encode(serial)
        )
    } else if let Some(caps) = UPUI_RE.captures(uri) {
        let cp = caps.get(1).unwrap().as_str();
        let item = caps.get(2).unwrap().as_str();
        let serial = caps.get(3).unwrap().as_str();
        let raw_gtin = format!("{}{}{}", &item[0..1], cp, &item[1..]);
        let cd = calculate_check_digit(&raw_gtin);
        format!(
            "https://id.gs1.org/01/{}{}/235/{}",
            raw_gtin,
            cd,
            percent_encode(serial)
        )
    } else if let Some(dl) = normalize_dl_url(uri) {
        dl
    } else {
        uri.to_string()
    }
}

fn strip_epcis_namespace(name: &str) -> &str {
    name.strip_prefix("{urn:epcglobal:epcis:xsd:1}")
        .or_else(|| name.strip_prefix("{urn:epcglobal:epcis:xsd:2}"))
        .or_else(|| name.strip_prefix("{https://ref.gs1.org/epcis/}"))
        .unwrap_or(name)
}

fn try_format_web_vocabulary(text: &str) -> String {
    let mut s = text.to_string();
    if s.starts_with("gs1:") {
        s = s.replace("gs1:", "https://gs1.org/voc/");
    }
    if s.starts_with("cbv:") {
        s = s.replace("cbv:", "https://ref.gs1.org/cbv/");
    }
    s.replace(
        "urn:epcglobal:cbv:bizstep:",
        "https://ref.gs1.org/cbv/BizStep-",
    )
    .replace("urn:epcglobal:cbv:disp:", "https://ref.gs1.org/cbv/Disp-")
    .replace("urn:epcglobal:cbv:btt:", "https://ref.gs1.org/cbv/BTT-")
    .replace("urn:epcglobal:cbv:sdt:", "https://ref.gs1.org/cbv/SDT-")
    .replace("urn:epcglobal:cbv:er:", "https://ref.gs1.org/cbv/ER-")
}

fn normalize_cbv_value(parent_name: Option<&str>, field_name: Option<&str>, value: &str) -> String {
    let parent = strip_epcis_namespace(parent_name.unwrap_or(""));
    let name = strip_epcis_namespace(field_name.unwrap_or(""));
    let mut normalized = try_format_web_vocabulary(value);

    if !normalized.contains(':') && !normalized.contains('/') && !normalized.is_empty() {
        if name == "bizStep" {
            normalized = format!("https://ref.gs1.org/cbv/BizStep-{normalized}");
        } else if name == "disposition" || name == "set" || name == "unset" {
            normalized = format!("https://ref.gs1.org/cbv/Disp-{normalized}");
        } else if name == "type" && parent == "bizTransactionList" {
            normalized = format!("https://ref.gs1.org/cbv/BTT-{normalized}");
        } else if name == "type" && (parent == "sourceList" || parent == "destinationList") {
            normalized = format!("https://ref.gs1.org/cbv/SDT-{normalized}");
        } else if (name == "type" || name == "exception") && parent == "sensorReport" {
            normalized = format!("https://gs1.org/voc/{normalized}");
        } else if name == "component" && parent == "sensorReport" {
            normalized = format!("https://ref.gs1.org/cbv/Comp-{normalized}");
        }
    }
    normalized
}

fn normalize_datetime(val: &str) -> String {
    // Per spec rule 9: if >3 decimal places, round the 3rd decimal digit up if 4th is 5-9.
    // Chrono's SecondsFormat::Millis truncates, so we pre-round the fractional seconds manually.
    let rounded = round_timestamp_to_millis(val);
    let parsed = chrono::DateTime::parse_from_rfc3339(&rounded)
        .or_else(|_| chrono::DateTime::parse_from_str(&rounded, "%Y-%m-%dT%H:%M:%SZ"))
        .or_else(|_| chrono::DateTime::parse_from_str(&rounded, "%Y-%m-%dT%H:%M:%S%.fZ"))
        .or_else(|_| chrono::DateTime::parse_from_str(&rounded, "%Y-%m-%dT%H:%M:%S%.f%z"))
        .or_else(|_| chrono::DateTime::parse_from_str(&rounded, "%Y-%m-%d %H:%M:%S%.f%z"));

    if let Ok(dt) = parsed {
        let utc_dt = dt.with_timezone(&chrono::Utc);
        utc_dt.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
    } else {
        val.to_string()
    }
}

/// Round a timestamp string's fractional seconds to 3 decimal places (milliseconds),
/// rounding up the 3rd digit if the 4th decimal digit is >= 5.
fn round_timestamp_to_millis(val: &str) -> String {
    // Find the decimal point after the seconds field
    // Timestamps look like: 2023-01-18T11:04:03.1415Z or ...+01:00
    // Find the 'T' first, then look for '.' after the time part
    if let Some(t_pos) = val.find('T') {
        let time_part = &val[t_pos + 1..];
        if let Some(dot_pos) = time_part.find('.') {
            let abs_dot_pos = t_pos + 1 + dot_pos;
            // Find where the fractional part ends (Z or + or -)
            let frac_start = abs_dot_pos + 1;
            let rest = &val[frac_start..];
            let frac_end = rest.find(['Z', '+', '-']).unwrap_or(rest.len());
            let frac_str = &rest[..frac_end];

            if frac_str.len() > 3 {
                // Parse the fractional digits
                let digits: Vec<u8> = frac_str
                    .chars()
                    .filter(char::is_ascii_digit)
                    .take(4)
                    .map(|c| c as u8 - b'0')
                    .collect();

                if digits.len() >= 4 {
                    // Round: if 4th digit >= 5, round up 3rd digit
                    let d0 = u32::from(digits[0]);
                    let d1 = u32::from(digits[1]);
                    let mut d2 = u32::from(digits[2]);
                    let d3 = u32::from(digits[3]);

                    if d3 >= 5 {
                        d2 += 1;
                    }

                    // Handle carry
                    let ms = d0 * 100 + d1 * 10 + d2;
                    // ms might be 1000 if rounding caused overflow — let chrono handle via nanoseconds
                    let new_frac = format!("{:03}", ms % 1000);
                    let carry = ms / 1000;

                    let prefix = &val[..frac_start];
                    let suffix = &val[frac_start + frac_end..];

                    if carry > 0 {
                        // Need to carry into seconds — rebuild without fractional part and let chrono add
                        // Get base up to the dot
                        let base = &val[..abs_dot_pos];
                        // Actually re-inject carry by re-parsing as a number:
                        // Simplest: remove the fractional part, parse, add 1 second, re-serialize
                        let no_frac = format!("{base}{suffix}");
                        if let Ok(dt) =
                            chrono::DateTime::parse_from_rfc3339(&no_frac).or_else(|_| {
                                chrono::DateTime::parse_from_str(&no_frac, "%Y-%m-%dT%H:%M:%S%z")
                            })
                        {
                            let dt_bumped = dt + chrono::Duration::seconds(1);
                            return format!(
                                "{}.000{}",
                                &dt_bumped.to_rfc3339()[..19],
                                if suffix.starts_with('Z') { "Z" } else { suffix }
                            );
                        }
                        return format!("{prefix}{new_frac}{suffix}");
                    }
                    return format!("{prefix}{new_frac}{suffix}");
                }
            }
        }
    }
    val.to_string()
}

fn normalize_numeric(val: &str) -> String {
    // The reference implementation canonicalizes every numeric-looking value
    // through a 64-bit float (`str(int(float(text)))` in Python), so integers
    // above 2^53 intentionally lose precision here; matching that exactly is
    // required for hash interoperability (see SensorDataExamples.xml vectors).
    // Textual values that f64::from_str also accepts ("inf"/"nan") must pass
    // through untouched, as they do in the reference.
    if !val.bytes().all(|b| {
        b.is_ascii_digit() || b == b'.' || b == b'-' || b == b'+' || b == b'e' || b == b'E'
    }) || val.is_empty()
    {
        return val.to_string();
    }
    if let Ok(num) = val.parse::<f64>() {
        if (num - num.trunc()).abs() < f64::EPSILON {
            format!("{num:.0}")
        } else {
            format!("{num}")
        }
    } else {
        val.to_string()
    }
}

fn is_ignored_field(name: &str, namespaces: &BTreeMap<String, String>) -> bool {
    let stripped = strip_epcis_namespace(name);
    if stripped == "recordTime"
        || stripped == "eventID"
        || stripped == "eventId"
        || stripped == "errorDeclaration"
        || stripped == "declarationTime"
        || stripped == "reason"
        || stripped == "correctiveEventIDs"
        || stripped == "correctiveEventId"
        || stripped == "@context"
        || stripped == "context"
        || stripped == "rdfs:comment"
        || stripped == "comment"
        || stripped == "#text"
    {
        return true;
    }

    if namespaces.contains_key(&format!("ignore:{name}")) {
        return true;
    }

    if name.contains(':') && !name.starts_with("http://") && !name.starts_with("https://") {
        let parts: Vec<&str> = name.splitn(2, ':').collect();
        let prefix = parts[0];
        let local = parts[1];
        if let Some(ns_uri) = namespaces.get(prefix) {
            let clark = format!("{{{ns_uri}}}{local}");
            if namespaces.contains_key(&format!("ignore:{clark}")) {
                return true;
            }
        }
    }

    false
}

// A single lookup table mirroring the reference implementation's property
// order; splitting it would hurt comparability with the spec.
#[allow(clippy::too_many_lines)]
fn get_sort_order(parent_name: Option<&str>, name: Option<&str>) -> usize {
    let parent = strip_epcis_namespace(parent_name.unwrap_or(""));
    let name = strip_epcis_namespace(name.unwrap_or(""));

    match parent {
        "" => match name {
            "eventTime" => 0,
            "eventTimeZoneOffset" => 1,
            "certificationInfo" => 2,
            "parentID" => 3,
            "epcList" => 4,
            "inputEPCList" => 5,
            "childEPCs" => 6,
            "quantityList" => 7,
            "childQuantityList" => 8,
            "inputQuantityList" => 9,
            "outputEPCList" => 10,
            "outputQuantityList" => 11,
            "action" => 12,
            "transformationID" => 13,
            "bizStep" => 14,
            "disposition" => 15,
            "persistentDisposition" => 16,
            "readPoint" => 17,
            "bizLocation" => 18,
            "sensorElementList" => 19,
            // bizTransactionList/sourceList/destinationList are treated as residuals
            // at root level (order > 1000) to match reference impl behavior
            _ => 1000,
        },
        "epcList" | "childEPCs" | "inputEPCList" | "outputEPCList" => match name {
            "epc" => 0,
            _ => 1000,
        },
        "quantityList" | "childQuantityList" | "inputQuantityList" | "outputQuantityList" => {
            match name {
                "quantityElement" => 0,
                _ => 1000,
            }
        }
        "quantityElement" => match name {
            "epcClass" => 0,
            "quantity" => 1,
            "uom" => 2,
            _ => 1000,
        },
        "persistentDisposition" => match name {
            "set" => 0,
            "unset" => 1,
            _ => 1000,
        },
        "readPoint" | "bizLocation" => match name {
            "id" => 0,
            _ => 1000,
        },
        "sensorElementList" => match name {
            "sensorElement" => 0,
            _ => 1000,
        },
        "sensorElement" => match name {
            "sensorMetadata" => 0,
            "sensorReport" => 1,
            _ => 1000,
        },
        "sensorMetadata" => match name {
            "time" => 0,
            "startTime" => 1,
            "endTime" => 2,
            "deviceID" => 3,
            "deviceMetadata" => 4,
            "rawData" => 5,
            "dataProcessingMethod" => 6,
            "bizRules" => 7,
            _ => 1000,
        },
        "sensorReport" => match name {
            "type" => 0,
            "exception" => 1,
            "deviceID" => 2,
            "deviceMetadata" => 3,
            "rawData" => 4,
            "dataProcessingMethod" => 5,
            "time" => 6,
            "microorganism" => 7,
            "chemicalSubstance" => 8,
            "value" => 9,
            "component" => 10,
            "stringValue" => 11,
            "booleanValue" => 12,
            "hexBinaryValue" => 13,
            "uriValue" => 14,
            "minValue" => 15,
            "maxValue" => 16,
            "meanValue" => 17,
            "sDev" => 18,
            "percRank" => 19,
            "percValue" => 20,
            "uom" => 21,
            "coordinateReferenceSystem" => 22,
            _ => 1000,
        },
        "bizTransactionList" => match name {
            "type" => 0,
            "bizTransaction" => 1,
            _ => 1000,
        },
        "sourceList" => match name {
            "type" => 0,
            "source" => 1,
            _ => 1000,
        },
        "destinationList" => match name {
            "type" => 0,
            "destination" => 1,
            _ => 1000,
        },
        _ => 1000,
    }
}

fn is_epcis_field(parent: Option<&str>, name: &str) -> bool {
    let parent_stripped = parent.map(strip_epcis_namespace);
    let name_stripped = strip_epcis_namespace(name);
    get_sort_order(parent_stripped, Some(name_stripped)) < 1000
}

fn should_emit_parent_name(name: &str, children: &[ContextNode], _is_cbv_2_0: bool) -> bool {
    let name_stripped = strip_epcis_namespace(name);
    if children.is_empty() {
        return false;
    }
    if name_stripped == "set" || name_stripped == "unset" {
        return false;
    }
    true
}

fn format_leaf_standard(parent_name: Option<&str>, name: &str, value: &str) -> String {
    if value.is_empty() {
        return String::new();
    }
    let parent_stripped = parent_name.map(strip_epcis_namespace);
    let name_stripped = strip_epcis_namespace(name);
    let mut normalized_val = try_format_web_vocabulary(value);

    if name_stripped == "eventTime"
        || name_stripped == "time"
        || name_stripped == "startTime"
        || name_stripped == "endTime"
        || name_stripped == "declarationTime"
    {
        normalized_val = normalize_datetime(&normalized_val);
    } else if name_stripped == "bizStep"
        || name_stripped == "disposition"
        || name_stripped == "set"
        || name_stripped == "unset"
        || name_stripped == "type"
        || name_stripped == "exception"
        || name_stripped == "component"
    {
        normalized_val = normalize_cbv_value(parent_stripped, Some(name_stripped), &normalized_val);
    } else {
        let norm_uri = normalise_uri(&normalized_val);
        if norm_uri == normalized_val {
            normalized_val = normalize_numeric(&normalized_val);
        } else {
            normalized_val = norm_uri;
        }
    }

    if name_stripped == "type" && parent_stripped.is_none() {
        format!("eventType={normalized_val}\n")
    } else {
        format!("{name_stripped}={normalized_val}\n")
    }
}

impl ContextNode {
    /// Generates a pre-hash string for standard EPCIS fields in this node.
    // Standard fields, biz-list residuals, and user extensions must be emitted
    // in one pass to preserve spec ordering; splitting would obscure that.
    #[allow(clippy::too_many_lines)]
    #[must_use]
    pub fn to_prehash_string(
        &self,
        parent_name: Option<&str>,
        is_cbv_2_0: bool,
        namespaces: &BTreeMap<String, String>,
    ) -> String {
        let mut sb = String::new();

        if self.children.is_empty()
            && let (Some(name), Some(value)) = (self.name.as_deref(), self.value.as_deref())
        {
            if is_epcis_field(parent_name, name) {
                sb.push_str(&format_leaf_standard(parent_name, name, value));
            }
            return sb;
        }

        let name_str = self.name.as_deref().unwrap_or("");
        let name_stripped = strip_epcis_namespace(name_str);
        let is_biz_list_container = name_stripped == "bizTransactionList"
            || name_stripped == "sourceList"
            || name_stripped == "destinationList";
        let should_emit =
            self.name.is_some() && should_emit_parent_name(name_str, &self.children, is_cbv_2_0);

        if should_emit {
            // Emit without the EPCIS XML namespace so default-xmlns documents
            // (`{urn:epcglobal:epcis:xsd:2}epcList`) hash identically to
            // unprefixed ones; non-EPCIS namespaces pass through untouched.
            sb.push_str(name_stripped);
        }

        let current_parent = if self.name.is_none() {
            parent_name
        } else {
            self.name.as_deref()
        };

        // For biz list containers, sort children by their prehash string value and emit inline
        if is_biz_list_container {
            let mut list_children: Vec<String> = self
                .children
                .iter()
                .map(|child| child.to_prehash_string(self.name.as_deref(), is_cbv_2_0, namespaces))
                .collect();
            list_children.sort();
            for s in list_children {
                sb.push_str(&s);
            }
            return sb;
        }

        for child in &self.children {
            let child_name = child.name.as_deref().unwrap_or("");
            if is_ignored_field(child_name, namespaces) {
                continue;
            }
            if child_name == "type" && self.name.is_none() && parent_name.is_none() {
                continue;
            }
            let is_ext = child.name.is_some() && !is_epcis_field(current_parent, child_name);

            // Treat biz lists (bizTransactionList/sourceList/destinationList) as
            // residuals at the root event level — emit them in a separate sorted block below
            let is_biz_list = {
                let cn_stripped = strip_epcis_namespace(child_name);
                parent_name.is_none()
                    && self.name.is_none()
                    && (cn_stripped == "bizTransactionList"
                        || cn_stripped == "sourceList"
                        || cn_stripped == "destinationList")
            };

            if !is_ext && !is_biz_list {
                let next_parent = if self.name.is_none() {
                    parent_name
                } else {
                    self.name.as_deref()
                };
                sb.push_str(&child.to_prehash_string(next_parent, is_cbv_2_0, namespaces));
            }
        }

        // Collect biz lists as residuals (appended after all standard fields incl sensorElementList)
        if parent_name.is_none() && self.name.is_none() {
            let mut biz_list_values: Vec<String> = self
                .children
                .iter()
                .filter(|child| {
                    if let Some(ref cn) = child.name {
                        let cn_stripped = strip_epcis_namespace(cn);
                        cn_stripped == "bizTransactionList"
                            || cn_stripped == "sourceList"
                            || cn_stripped == "destinationList"
                    } else {
                        false
                    }
                })
                .map(|child| child.to_prehash_string(None, is_cbv_2_0, namespaces))
                .collect();
            biz_list_values.sort();
            for s in biz_list_values {
                sb.push_str(&s);
            }
        }

        if is_cbv_2_0 {
            let mut ext_values = vec![];
            for child in &self.children {
                let child_name = child.name.as_deref().unwrap_or("");
                if is_ignored_field(child_name, namespaces) {
                    continue;
                }
                if child_name == "type" && self.name.is_none() && parent_name.is_none() {
                    continue;
                }
                let is_ext = child.name.is_some() && !is_epcis_field(current_parent, child_name);
                // Don't double-emit biz lists as user extensions
                let is_biz_list = {
                    let cn_stripped = strip_epcis_namespace(child_name);
                    cn_stripped == "bizTransactionList"
                        || cn_stripped == "sourceList"
                        || cn_stripped == "destinationList"
                };
                if is_ext && !is_biz_list {
                    let next_parent = if self.name.is_none() {
                        parent_name
                    } else {
                        self.name.as_deref()
                    };
                    let formatted = child.user_extensions_prehash_builder(next_parent, namespaces);
                    if !formatted.is_empty() {
                        ext_values.push(formatted);
                    }
                }
            }
            ext_values.sort();
            for ext in ext_values {
                sb.push_str(&ext);
            }
        }

        sb
    }

    /// Recursively sorts the children of this node according to EPCIS hash rules.
    pub fn sort_children(
        &mut self,
        parent_name: Option<&str>,
        is_cbv_2_0: bool,
        namespaces: &BTreeMap<String, String>,
    ) {
        let current_parent = if self.name.is_none() {
            parent_name
        } else {
            self.name.as_deref()
        };
        self.children.sort_by(|a, b| {
            let a_name = a.name.as_deref();
            let b_name = b.name.as_deref();

            let a_order = get_sort_order(current_parent, a_name);
            let b_order = get_sort_order(current_parent, b_name);

            match a_order.cmp(&b_order) {
                std::cmp::Ordering::Equal => match (a.name.as_deref(), b.name.as_deref()) {
                    (None, None) => {
                        let a_str = a.find_children_string(current_parent, is_cbv_2_0, namespaces);
                        let b_str = b.find_children_string(current_parent, is_cbv_2_0, namespaces);
                        a_str.cmp(&b_str)
                    }
                    (Some(a_n), Some(b_n)) => {
                        let a_is_ext = !is_epcis_field(current_parent, a_n);
                        let b_is_ext = !is_epcis_field(current_parent, b_n);

                        if a_is_ext && b_is_ext {
                            let a_ext = a.format_user_extension(namespaces);
                            let b_ext = b.format_user_extension(namespaces);
                            a_ext.cmp(&b_ext)
                        } else if let (Some(a_val), Some(b_val)) =
                            (a.value.as_deref(), b.value.as_deref())
                        {
                            let a_n_stripped = strip_epcis_namespace(a_n);
                            if a_n_stripped == "epc"
                                || a_n_stripped == "epcClass"
                                || a_n_stripped == "id"
                            {
                                let a_norm = normalise_uri(a_val);
                                let b_norm = normalise_uri(b_val);
                                a_norm.cmp(&b_norm)
                            } else {
                                a_val.cmp(b_val)
                            }
                        } else {
                            let a_str =
                                a.find_children_string(current_parent, is_cbv_2_0, namespaces);
                            let b_str =
                                b.find_children_string(current_parent, is_cbv_2_0, namespaces);
                            a_str.cmp(&b_str)
                        }
                    }
                    (None, Some(_)) => std::cmp::Ordering::Less,
                    (Some(_), None) => std::cmp::Ordering::Greater,
                },
                ord => ord,
            }
        });

        for child in &mut self.children {
            let next_parent = if self.name.is_none() {
                parent_name
            } else {
                self.name.as_deref()
            };
            child.sort_children(next_parent, is_cbv_2_0, namespaces);
        }
    }

    fn find_children_string(
        &self,
        parent_name: Option<&str>,
        is_cbv_2_0: bool,
        namespaces: &BTreeMap<String, String>,
    ) -> String {
        let mut cloned = self.clone();
        let next_parent = if self.name.is_none() {
            parent_name
        } else {
            self.name.as_deref()
        };
        cloned.sort_children(next_parent, is_cbv_2_0, namespaces);
        let mut sb = String::new();
        for child in &cloned.children {
            if child.value.is_some() {
                sb.push_str(&child.format_leaf(next_parent, namespaces));
            } else {
                sb.push_str(&child.find_children_string(next_parent, is_cbv_2_0, namespaces));
            }
        }
        sb
    }

    fn format_leaf(
        &self,
        parent_name: Option<&str>,
        namespaces: &BTreeMap<String, String>,
    ) -> String {
        if let Some(ref name) = self.name {
            if let Some(ref val) = self.value {
                if is_epcis_field(parent_name, name) {
                    format_leaf_standard(parent_name, name, val)
                } else {
                    self.format_user_extension(namespaces)
                }
            } else {
                String::new()
            }
        } else {
            self.value.clone().unwrap_or_default()
        }
    }

    fn format_user_extension(&self, namespaces: &BTreeMap<String, String>) -> String {
        let name = self.name.as_deref().unwrap_or("");
        let value = self.value.as_deref().unwrap_or("");
        let normalized_val = normalize_numeric(value);
        let normalized_val = normalise_uri(&normalized_val);

        let formatted_val = if normalized_val.is_empty() {
            String::new()
        } else {
            format!("={normalized_val}")
        };

        if name.starts_with('{') {
            format!("{name}{formatted_val}\n")
        } else if name.contains(':')
            && !name.starts_with("http://")
            && !name.starts_with("https://")
        {
            let parts: Vec<&str> = name.splitn(2, ':').collect();
            let prefix = parts[0];
            let local = parts[1];
            if let Some(ns_uri) = namespaces.get(prefix) {
                format!("{{{ns_uri}}}{local}{formatted_val}\n")
            } else {
                format!("{name}{formatted_val}\n")
            }
        } else if name.starts_with("http://") || name.starts_with("https://") {
            let split_idx = name.rfind('#').or_else(|| name.rfind('/'));
            if let Some(idx) = split_idx {
                let ns = &name[..=idx];
                let local = &name[idx + 1..];
                format!("{{{ns}}}{local}{formatted_val}\n")
            } else {
                format!("{name}{formatted_val}\n")
            }
        } else {
            format!("{name}{formatted_val}\n")
        }
    }

    /// Determines if this is a leaf node carrying user extension data.
    #[must_use]
    pub fn is_leaf_user_extension(&self, parent_name: Option<&str>) -> bool {
        self.children.is_empty()
            && self.value.is_some()
            && self
                .name
                .as_deref()
                .is_some_and(|n| !is_epcis_field(parent_name, n))
    }

    /// Traverses this subtree to construct the user extension pre-hash string (recursive formatting).
    #[must_use]
    pub fn user_extensions_prehash_builder(
        &self,
        parent_name: Option<&str>,
        namespaces: &BTreeMap<String, String>,
    ) -> String {
        let mut sb = String::new();

        if self.is_leaf_user_extension(parent_name) {
            let name_str = self.name.as_deref().unwrap_or("");
            if name_str == "type" && parent_name.is_none() {
                return sb;
            }
            sb.push_str(&self.format_user_extension(namespaces));
            return sb;
        }

        let name_str = self.name.as_deref().unwrap_or("");
        let is_ext_parent = self.name.is_some()
            && !is_epcis_field(parent_name, name_str)
            && !is_ignored_field(name_str, namespaces)
            && name_str != "type";

        if is_ext_parent {
            sb.push_str(&self.format_user_extension_wrapper(namespaces));
        }

        for child in &self.children {
            let child_name = child.name.as_deref().unwrap_or("");
            if is_ignored_field(child_name, namespaces) {
                continue;
            }
            if child_name == "type" && self.name.is_none() && parent_name.is_none() {
                continue;
            }
            let next_parent = if self.name.is_none() {
                parent_name
            } else {
                self.name.as_deref()
            };
            sb.push_str(&child.user_extensions_prehash_builder(next_parent, namespaces));
        }

        sb
    }

    fn format_user_extension_wrapper(&self, namespaces: &BTreeMap<String, String>) -> String {
        let name = self.name.as_deref().unwrap_or("");
        if name.starts_with('{') {
            format!("{name}\n")
        } else if name.contains(':')
            && !name.starts_with("http://")
            && !name.starts_with("https://")
        {
            let parts: Vec<&str> = name.splitn(2, ':').collect();
            let prefix = parts[0];
            let local = parts[1];
            if let Some(ns_uri) = namespaces.get(prefix) {
                format!("{{{ns_uri}}}{local}\n")
            } else {
                format!("{name}\n")
            }
        } else if name.starts_with("http://") || name.starts_with("https://") {
            let split_idx = name.rfind('#').or_else(|| name.rfind('/'));
            if let Some(idx) = split_idx {
                let ns = &name[..=idx];
                let local = &name[idx + 1..];
                format!("{{{ns}}}{local}\n")
            } else {
                format!("{name}\n")
            }
        } else {
            format!("{name}\n")
        }
    }

    /// Bubbles up bare user extensions (non-standard, non-namespace prefixed fields) to the event root level.
    pub fn bubble_up_bare_extensions(&mut self, parent_name: Option<&str>) -> Vec<ContextNode> {
        let mut bubbled = vec![];
        let current_parent = if self.name.is_none() {
            parent_name
        } else {
            self.name.as_deref()
        };

        let is_self_standard = if let Some(ref self_name) = self.name {
            is_epcis_field(parent_name, self_name)
        } else {
            true
        };

        let mut i = 0;
        while i < self.children.len() {
            let child = &mut self.children[i];
            let child_parent = if self.name.is_none() {
                parent_name
            } else {
                self.name.as_deref()
            };
            let child_bubbled = child.bubble_up_bare_extensions(child_parent);
            if !child_bubbled.is_empty() {
                for mut b in child_bubbled {
                    if let Some(ref child_name) = child.name {
                        let child_name_stripped = strip_epcis_namespace(child_name);
                        if let Some(ref mut b_name) = b.name {
                            *b_name = format!("{child_name_stripped}{b_name}");
                        }
                    }
                    bubbled.push(b);
                }
            }
            i += 1;
        }

        if is_self_standard {
            let own_children = std::mem::take(&mut self.children);
            let mut retained = vec![];
            for child in own_children {
                let is_bare_ext = if let Some(ref child_name) = child.name {
                    !is_epcis_field(current_parent, child_name)
                        && !child_name.starts_with('{')
                        && !child_name.contains(':')
                } else {
                    false
                };

                if is_bare_ext {
                    bubbled.push(child);
                } else {
                    retained.push(child);
                }
            }
            self.children = retained;
        }

        bubbled
    }
}

/// Recursively builds a `ContextNode` from a `serde_json::Value`.
// One match arm per JSON value kind with list-specific child naming; the
// recursion reads clearest as a single function.
#[allow(clippy::too_many_lines)]
#[must_use]
pub fn json_to_context_node(
    name: Option<String>,
    value: &Value,
    namespaces: &BTreeMap<String, String>,
) -> Option<ContextNode> {
    if let Some(ref n) = name
        && is_ignored_field(n, namespaces)
    {
        return None;
    }

    match value {
        Value::Null => None,
        Value::Bool(b) => Some(ContextNode {
            name,
            value: Some(b.to_string()),
            children: vec![],
        }),
        Value::Number(num) => Some(ContextNode {
            name,
            value: Some(num.to_string()),
            children: vec![],
        }),
        Value::String(s) => Some(ContextNode {
            name,
            value: Some(s.clone()),
            children: vec![],
        }),
        Value::Array(arr) => {
            let mut children = vec![];
            let name_str = name.as_deref().unwrap_or("");
            for item in arr {
                if item.is_object() {
                    if name_str == "quantityList"
                        || name_str == "childQuantityList"
                        || name_str == "inputQuantityList"
                        || name_str == "outputQuantityList"
                    {
                        if let Some(child_node) = json_to_context_node(
                            Some("quantityElement".to_string()),
                            item,
                            namespaces,
                        ) {
                            children.push(child_node);
                        }
                    } else if name_str == "sensorElementList" {
                        if let Some(child_node) = json_to_context_node(
                            Some("sensorElement".to_string()),
                            item,
                            namespaces,
                        ) {
                            children.push(child_node);
                        }
                    } else if name_str == "sensorReport" {
                        if let Some(child_node) =
                            json_to_context_node(Some("sensorReport".to_string()), item, namespaces)
                        {
                            children.push(child_node);
                        }
                    } else if name_str == "bizTransactionList"
                        || name_str == "sourceList"
                        || name_str == "destinationList"
                        || name_str == "persistentDisposition"
                    {
                        if let Some(child_node) = json_to_context_node(None, item, namespaces) {
                            children.push(child_node);
                        }
                    } else if let Some(child_node) =
                        json_to_context_node(name.clone(), item, namespaces)
                    {
                        children.push(child_node);
                    }
                } else if item.is_array() {
                    if let Some(child_node) = json_to_context_node(name.clone(), item, namespaces) {
                        children.push(child_node);
                    }
                } else {
                    let is_epc_list = name_str == "epcList"
                        || name_str == "childEPCs"
                        || name_str == "inputEPCList"
                        || name_str == "outputEPCList";
                    let child_name = if is_epc_list {
                        Some("epc".to_string())
                    } else {
                        name.clone()
                    };
                    if let Some(child_node) = json_to_context_node(child_name, item, namespaces) {
                        children.push(child_node);
                    }
                }
            }
            Some(ContextNode {
                name,
                value: None,
                children,
            })
        }
        Value::Object(map) => {
            let mut children = vec![];
            for (k, v) in map {
                if is_ignored_field(k, namespaces) {
                    continue;
                }
                let is_standard_list = k == "epcList"
                    || k == "childEPCs"
                    || k == "inputEPCList"
                    || k == "outputEPCList"
                    || k == "quantityList"
                    || k == "childQuantityList"
                    || k == "inputQuantityList"
                    || k == "outputQuantityList"
                    || k == "sensorElementList"
                    || k == "bizTransactionList"
                    || k == "sourceList"
                    || k == "destinationList"
                    || k == "persistentDisposition";

                if v.is_array() && !is_standard_list {
                    if let Value::Array(arr) = v {
                        for item in arr {
                            if let Some(child_node) =
                                json_to_context_node(Some(k.clone()), item, namespaces)
                            {
                                children.push(child_node);
                            }
                        }
                    }
                } else if let Some(child_node) =
                    json_to_context_node(Some(k.clone()), v, namespaces)
                {
                    children.push(child_node);
                }
            }
            Some(ContextNode {
                name,
                value: None,
                children,
            })
        }
    }
}

fn extract_namespaces_from_json(context_val: &Value, namespaces: &mut BTreeMap<String, String>) {
    match context_val {
        Value::Object(obj) => {
            for (k, v) in obj {
                if let Value::String(s) = v {
                    namespaces.insert(k.clone(), s.clone());
                }
            }
        }
        Value::Array(arr) => {
            for item in arr {
                extract_namespaces_from_json(item, namespaces);
            }
        }
        _ => {}
    }
}

/// Parses an XML document into a `ContextNode` tree.
///
/// # Errors
///
/// Returns `EpcisHashError` if XML parsing fails.
// Start/Empty/Text/End event handling shares state through the local stack;
// extracting helpers would only add plumbing.
#[allow(clippy::too_many_lines)]
pub fn xml_to_context_node(
    xml_str: &str,
    namespaces: &mut BTreeMap<String, String>,
) -> Result<ContextNode, EpcisHashError> {
    let clean_xml = xml_str
        .replace("<extension>", "")
        .replace("</extension>", "")
        .replace("<baseExtension>", "")
        .replace("</baseExtension>", "");

    let mut reader = Reader::from_str(&clean_xml);
    reader.config_mut().trim_text(true);

    let mut stack: Vec<ContextNode> = vec![];
    let mut root_node: Option<ContextNode> = None;
    let mut buf = Vec::new();
    let mut ns_stack: Vec<BTreeMap<String, String>> = vec![];

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let mut current_ns = ns_stack.last().cloned().unwrap_or_default();
                for attr in e.attributes().flatten() {
                    let key = String::from_utf8_lossy(attr.key.as_ref()).into_owned();
                    let val = String::from_utf8_lossy(&attr.value).into_owned();
                    if let Some(prefix) = key.strip_prefix("xmlns:") {
                        current_ns.insert(prefix.to_string(), val.clone());
                        namespaces.insert(prefix.to_string(), val);
                    } else if key == "xmlns" {
                        current_ns.insert(String::new(), val);
                    }
                }
                ns_stack.push(current_ns.clone());

                let raw_name = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                let resolved_name = if raw_name.contains(':') {
                    let parts: Vec<&str> = raw_name.splitn(2, ':').collect();
                    let prefix = parts[0];
                    let local = parts[1];
                    if let Some(ns_uri) = current_ns.get(prefix) {
                        format!("{{{ns_uri}}}{local}")
                    } else {
                        raw_name.clone()
                    }
                } else if let Some(ns_uri) = current_ns.get("") {
                    format!("{{{ns_uri}}}{raw_name}")
                } else {
                    raw_name.clone()
                };

                let mut attrs = vec![];
                for attr in e.attributes().flatten() {
                    let key = String::from_utf8_lossy(attr.key.as_ref()).into_owned();
                    let val = String::from_utf8_lossy(&attr.value).into_owned();
                    if !key.starts_with("xmlns")
                        && !key.starts_with("xsi:")
                        && !key.starts_with("xsd:")
                    {
                        let resolved_key = if key.contains(':') {
                            let parts: Vec<&str> = key.splitn(2, ':').collect();
                            let prefix = parts[0];
                            let local = parts[1];
                            if let Some(ns_uri) = current_ns.get(prefix) {
                                format!("{{{ns_uri}}}{local}")
                            } else {
                                key.clone()
                            }
                        } else {
                            key.clone()
                        };
                        attrs.push((resolved_key, val));
                    }
                }

                let mut node = ContextNode {
                    name: Some(resolved_name),
                    value: None,
                    children: vec![],
                };

                for (attr_k, attr_v) in attrs {
                    node.children.push(ContextNode {
                        name: Some(attr_k),
                        value: Some(attr_v),
                        children: vec![],
                    });
                }

                stack.push(node);
            }
            Ok(Event::Empty(ref e)) => {
                let mut current_ns = ns_stack.last().cloned().unwrap_or_default();
                for attr in e.attributes().flatten() {
                    let key = String::from_utf8_lossy(attr.key.as_ref()).into_owned();
                    let val = String::from_utf8_lossy(&attr.value).into_owned();
                    if let Some(prefix) = key.strip_prefix("xmlns:") {
                        current_ns.insert(prefix.to_string(), val.clone());
                        namespaces.insert(prefix.to_string(), val);
                    } else if key == "xmlns" {
                        current_ns.insert(String::new(), val);
                    }
                }

                let raw_name = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                let resolved_name = if raw_name.contains(':') {
                    let parts: Vec<&str> = raw_name.splitn(2, ':').collect();
                    let prefix = parts[0];
                    let local = parts[1];
                    if let Some(ns_uri) = current_ns.get(prefix) {
                        format!("{{{ns_uri}}}{local}")
                    } else {
                        raw_name.clone()
                    }
                } else if let Some(ns_uri) = current_ns.get("") {
                    format!("{{{ns_uri}}}{raw_name}")
                } else {
                    raw_name.clone()
                };

                let mut node = ContextNode {
                    name: Some(resolved_name),
                    value: None,
                    children: vec![],
                };

                for attr in e.attributes().flatten() {
                    let key = String::from_utf8_lossy(attr.key.as_ref()).into_owned();
                    let val = String::from_utf8_lossy(&attr.value).into_owned();
                    if !key.starts_with("xmlns")
                        && !key.starts_with("xsi:")
                        && !key.starts_with("xsd:")
                    {
                        let resolved_key = if key.contains(':') {
                            let parts: Vec<&str> = key.splitn(2, ':').collect();
                            let prefix = parts[0];
                            let local = parts[1];
                            if let Some(ns_uri) = current_ns.get(prefix) {
                                format!("{{{ns_uri}}}{local}")
                            } else {
                                key.clone()
                            }
                        } else {
                            key.clone()
                        };
                        node.children.push(ContextNode {
                            name: Some(resolved_key),
                            value: Some(val),
                            children: vec![],
                        });
                    }
                }

                if let Some(parent) = stack.last_mut() {
                    parent.children.push(node);
                } else if root_node.is_none() {
                    root_node = Some(node);
                }
            }
            Ok(Event::Text(ref e)) => {
                if let Some(node) = stack.last_mut()
                    && let Ok(decoded) = e.decode()
                {
                    let decoded_str = decoded.as_ref();
                    let unescaped = quick_xml::escape::unescape(decoded_str)
                        .unwrap_or(Cow::Borrowed(decoded_str));
                    let text = unescaped.trim().to_string();
                    if !text.is_empty() {
                        node.value = Some(text);
                    }
                }
            }
            Ok(Event::End(ref _e)) => {
                ns_stack.pop();
                if let Some(mut finished_node) = stack.pop() {
                    if let Some(parent) = stack.last_mut() {
                        let tag_name =
                            strip_epcis_namespace(finished_node.name.as_deref().unwrap_or(""));
                        if tag_name == "bizTransaction"
                            || tag_name == "source"
                            || tag_name == "destination"
                        {
                            let type_idx = finished_node.children.iter().position(|c| {
                                strip_epcis_namespace(c.name.as_deref().unwrap_or("")) == "type"
                            });
                            if let Some(idx) = type_idx {
                                let type_node = finished_node.children.remove(idx);
                                let value_node = ContextNode {
                                    name: finished_node.name.clone(),
                                    value: finished_node.value.clone(),
                                    children: vec![],
                                };
                                let unnamed_node = ContextNode {
                                    name: None,
                                    value: None,
                                    children: vec![type_node, value_node],
                                };
                                parent.children.push(unnamed_node);
                            } else {
                                parent.children.push(finished_node);
                            }
                        } else {
                            parent.children.push(finished_node);
                        }
                    } else {
                        root_node = Some(finished_node);
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(EpcisHashError::XmlParse(format!("{e:?}"))),
            _ => {}
        }
        buf.clear();
    }

    root_node.ok_or(EpcisHashError::EmptyDocument)
}

fn find_events_in_tree(node: ContextNode, events: &mut Vec<ContextNode>) {
    let name_str = strip_epcis_namespace(node.name.as_deref().unwrap_or(""));
    if name_str == "ObjectEvent"
        || name_str == "AggregationEvent"
        || name_str == "TransformationEvent"
        || name_str == "AssociationEvent"
        || name_str == "TransactionEvent"
    {
        events.push(node);
    } else {
        for child in node.children {
            find_events_in_tree(child, events);
        }
    }
}

/// Computes the deterministic canonical hash for a given EPCIS event.
///
/// Under the hood, this standardizes fields (such as timestamps), strips transient fields,
/// orders child elements alphabetically, and generates a deterministic SHA-256 hash.
///
/// # Examples
///
/// ```
/// use epcis_models::{ObjectEvent, Action, EPCISEvent};
/// use epcis_hash::compute_canonical_hash;
/// use chrono::Utc;
///
/// let event = ObjectEvent::new(
///     Utc::now(),
///     "+00:00".to_string(),
///     Action::Observe
/// );
/// let event_enum = EPCISEvent::ObjectEvent(event);
///
/// let hash_urn = compute_canonical_hash(&event_enum);
/// assert!(hash_urn.is_ok());
/// assert!(hash_urn.unwrap().starts_with("ni:///sha-256;"));
/// ```
///
/// # Errors
///
/// Returns `EpcisHashError` if serialization to JSON or hashing fails.
pub fn compute_canonical_hash(event: &EPCISEvent) -> Result<String, EpcisHashError> {
    // Delegate to the same pipeline used for JSON documents so typed events
    // and raw JSON always canonicalize identically.
    let val = serde_json::to_value(event)?;
    let prehash = canonicalize_json(&val, true)?;
    Ok(compute_hash_from_prehash(&prehash))
}

/// Collects `ignoreFields` declarations from the document (top level and
/// query results) and registers them as `ignore:` entries in `namespaces`.
fn register_ignore_fields(json_val: &Value, namespaces: &mut BTreeMap<String, String>) {
    let mut ignore_fields = vec![];
    let mut ignore_key = "repository-x:ignoreFields".to_string();
    for (k, v) in namespaces.iter() {
        if v == "https://repository-x.example.com/" {
            ignore_key = format!("{k}:ignoreFields");
            break;
        }
    }
    if let Some(fields) = json_val
        .get(&ignore_key)
        .or_else(|| json_val.get("ignoreFields"))
        && let Some(arr) = fields.as_array()
    {
        for item in arr {
            if let Some(s) = item.as_str() {
                ignore_fields.push(s.to_string());
            }
        }
    }
    if let Some(body) = json_val.get("epcisBody")
        && let Some(query_results) = body.get("queryResults")
        && let Some(fields) = query_results
            .get(&ignore_key)
            .or_else(|| query_results.get("ignoreFields"))
        && let Some(arr) = fields.as_array()
    {
        for item in arr {
            if let Some(s) = item.as_str() {
                ignore_fields.push(s.to_string());
            }
        }
    }
    for field in ignore_fields {
        let resolved = if field.contains(':')
            && !field.starts_with("http://")
            && !field.starts_with("https://")
        {
            let parts: Vec<&str> = field.splitn(2, ':').collect();
            let prefix = parts[0];
            let local = parts[1];
            if let Some(ns_uri) = namespaces.get(prefix) {
                format!("{{{ns_uri}}}{local}")
            } else {
                field.clone()
            }
        } else {
            field.clone()
        };
        namespaces.insert(format!("ignore:{resolved}"), "true".to_string());
        namespaces.insert(format!("ignore:{field}"), "true".to_string());
    }
}

/// Helper to generate pre-hash string from JSON Value.
///
/// # Errors
///
/// Returns `EpcisHashError` if parsing fails.
pub fn canonicalize_json(json_val: &Value, is_cbv_2_0: bool) -> Result<String, EpcisHashError> {
    let mut namespaces = BTreeMap::new();
    namespaces.insert("gs1".to_string(), "https://gs1.org/voc/".to_string());
    namespaces.insert("cbv".to_string(), "https://ref.gs1.org/cbv/".to_string());
    namespaces.insert("cbvmda".to_string(), "urn:epcglobal:cbv:mda".to_string());

    if let Some(context_val) = json_val.get("@context") {
        extract_namespaces_from_json(context_val, &mut namespaces);
    }

    register_ignore_fields(json_val, &mut namespaces);

    // Traverse down to eventList if this is a document wrapper
    let mut event_list_vals = vec![];
    if let Some(body) = json_val.get("epcisBody") {
        if let Some(list) = body.get("eventList") {
            if let Some(arr) = list.as_array() {
                for item in arr {
                    event_list_vals.push(item);
                }
            }
        } else if let Some(query_results) = body.get("queryResults") {
            if let Some(results_body) = query_results.get("resultsBody")
                && let Some(list) = results_body.get("eventList")
                && let Some(arr) = list.as_array()
            {
                for item in arr {
                    event_list_vals.push(item);
                }
            }
        } else if let Some(evt) = body.get("event") {
            event_list_vals.push(evt);
        }
    } else {
        event_list_vals.push(json_val);
    }

    let mut prehashes = vec![];
    for item in event_list_vals {
        let mut event_node =
            json_to_context_node(None, item, &namespaces).ok_or(EpcisHashError::EmptyDocument)?;

        let type_val = event_node
            .children
            .iter()
            .find(|c| c.name.as_deref() == Some("type"))
            .and_then(|c| c.value.clone())
            .ok_or(EpcisHashError::MissingField { field: "type" })?;

        event_node.name = None;
        let bubbled = event_node.bubble_up_bare_extensions(None);
        event_node.children.extend(bubbled);
        event_node.sort_children(None, is_cbv_2_0, &namespaces);

        let mut prehash = format!("eventType={type_val}\n");
        prehash.push_str(&event_node.to_prehash_string(None, is_cbv_2_0, &namespaces));

        let stripped = prehash.replace(['\n', '\r'], "");
        prehashes.push(stripped);
    }

    Ok(prehashes.join("\n"))
}

/// Helper to generate pre-hash string from XML string.
///
/// # Errors
///
/// Returns `EpcisHashError` if parsing fails.
pub fn canonicalize_xml(xml_str: &str, is_cbv_2_0: bool) -> Result<String, EpcisHashError> {
    let mut namespaces = BTreeMap::new();
    namespaces.insert("gs1".to_string(), "https://gs1.org/voc/".to_string());
    namespaces.insert("cbv".to_string(), "https://ref.gs1.org/cbv/".to_string());
    namespaces.insert("cbvmda".to_string(), "urn:epcglobal:cbv:mda".to_string());

    let root_node = xml_to_context_node(xml_str, &mut namespaces)?;
    let mut xml_ignore_fields = vec![];
    for child in &root_node.children {
        let child_name = strip_epcis_namespace(child.name.as_deref().unwrap_or(""));
        if child_name.ends_with("ignoreFields") {
            for sub_child in &child.children {
                if let Some(ref sub_name) = sub_child.name {
                    xml_ignore_fields.push(sub_name.clone());
                }
            }
        }
    }
    for field in xml_ignore_fields {
        namespaces.insert(format!("ignore:{field}"), "true".to_string());
    }

    let mut event_nodes = vec![];
    find_events_in_tree(root_node, &mut event_nodes);

    let mut prehashes = vec![];
    for mut event_node in event_nodes {
        let type_val = event_node
            .name
            .as_deref()
            .map(|n| strip_epcis_namespace(n).to_string())
            .ok_or(EpcisHashError::MissingField { field: "eventType" })?;
        event_node.name = None;
        let bubbled = event_node.bubble_up_bare_extensions(None);
        event_node.children.extend(bubbled);
        event_node.sort_children(None, is_cbv_2_0, &namespaces);

        let mut prehash = format!("eventType={type_val}\n");
        prehash.push_str(&event_node.to_prehash_string(None, is_cbv_2_0, &namespaces));

        let stripped = prehash.replace(['\n', '\r'], "");
        prehashes.push(stripped);
    }

    Ok(prehashes.join("\n"))
}

/// Hashes a pre-hash string to standard EPCIS URN format.
#[must_use]
pub fn compute_hash_from_prehash(prehash: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(prehash.as_bytes());
    let hash_result = hasher.finalize();
    let hash_hex = hash_result.iter().fold(
        String::with_capacity(hash_result.len() * 2),
        |mut acc, b| {
            use std::fmt::Write;
            let _ = write!(acc, "{b:02x}");
            acc
        },
    );
    format!("ni:///sha-256;{hash_hex}?ver=CBV2.0")
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    // ── timestamp rounding ─────────────────────────────────────────────────────

    #[test]
    fn test_round_timestamp_no_rounding_needed() {
        // Exactly 3 decimal places – must be preserved as-is
        assert_eq!(
            round_timestamp_to_millis("2023-01-18T11:04:03.141Z"),
            "2023-01-18T11:04:03.141Z"
        );
    }

    #[test]
    fn test_round_timestamp_4th_digit_less_than_5() {
        // 4th digit is 4 → truncate (no round-up)
        assert_eq!(
            round_timestamp_to_millis("2023-02-02T11:04:03.1414Z"),
            "2023-02-02T11:04:03.141Z"
        );
    }

    #[test]
    fn test_round_timestamp_4th_digit_5_rounds_up() {
        // 4th digit is 5 → round 3rd digit up: 1 → 2
        assert_eq!(
            round_timestamp_to_millis("2023-02-02T11:04:03.1415Z"),
            "2023-02-02T11:04:03.142Z"
        );
    }

    #[test]
    fn test_round_timestamp_4th_digit_9_rounds_up() {
        assert_eq!(
            round_timestamp_to_millis("2023-01-18T11:04:03.1419Z"),
            "2023-01-18T11:04:03.142Z"
        );
    }

    #[test]
    fn test_round_timestamp_with_timezone_offset() {
        assert_eq!(
            round_timestamp_to_millis("2023-01-18T11:04:03.1415+01:00"),
            "2023-01-18T11:04:03.142+01:00"
        );
    }

    #[test]
    fn test_normalize_datetime_rounds_and_converts_to_utc() {
        // .1415Z → should produce .142Z
        let result = normalize_datetime("2023-02-02T11:04:03.1415Z");
        assert_eq!(result, "2023-02-02T11:04:03.142Z");
    }

    #[test]
    fn test_normalize_datetime_no_fractional_zero_fills() {
        let result = normalize_datetime("2023-02-02T11:04:03Z");
        assert_eq!(result, "2023-02-02T11:04:03.000Z");
    }

    #[test]
    fn test_normalize_datetime_offset_to_utc() {
        // +01:00 offset → UTC conversion
        let result = normalize_datetime("2023-02-02T12:04:03.000+01:00");
        assert_eq!(result, "2023-02-02T11:04:03.000Z");
    }

    #[test]
    fn test_normalize_datetime_truncate_4_places_lt5() {
        // .1414 → 4th digit 4, no round-up → .141
        let result = normalize_datetime("2023-02-02T11:04:03.1414Z");
        assert_eq!(result, "2023-02-02T11:04:03.141Z");
    }

    // ── CBV value normalisation ────────────────────────────────────────────────

    #[test]
    fn test_normalize_cbv_biz_step_bare() {
        let result = normalize_cbv_value(None, Some("bizStep"), "shipping");
        assert_eq!(result, "https://ref.gs1.org/cbv/BizStep-shipping");
    }

    #[test]
    fn test_normalize_cbv_disposition_bare() {
        let result = normalize_cbv_value(None, Some("disposition"), "in_transit");
        assert_eq!(result, "https://ref.gs1.org/cbv/Disp-in_transit");
    }

    #[test]
    fn test_normalize_cbv_biz_transaction_type() {
        let result = normalize_cbv_value(Some("bizTransactionList"), Some("type"), "po");
        assert_eq!(result, "https://ref.gs1.org/cbv/BTT-po");
    }

    #[test]
    fn test_normalize_cbv_source_type() {
        let result = normalize_cbv_value(Some("sourceList"), Some("type"), "owning_party");
        assert_eq!(result, "https://ref.gs1.org/cbv/SDT-owning_party");
    }

    #[test]
    fn test_normalize_cbv_destination_type() {
        let result = normalize_cbv_value(Some("destinationList"), Some("type"), "possessing_party");
        assert_eq!(result, "https://ref.gs1.org/cbv/SDT-possessing_party");
    }

    #[test]
    fn test_normalize_cbv_sensor_report_type() {
        let result = normalize_cbv_value(Some("sensorReport"), Some("type"), "Dimensionless");
        assert_eq!(result, "https://gs1.org/voc/Dimensionless");
    }

    #[test]
    fn test_normalize_cbv_sensor_report_exception() {
        let result =
            normalize_cbv_value(Some("sensorReport"), Some("exception"), "ALARM_CONDITION");
        assert_eq!(result, "https://gs1.org/voc/ALARM_CONDITION");
    }

    #[test]
    fn test_normalize_cbv_sensor_report_component() {
        let result = normalize_cbv_value(Some("sensorReport"), Some("component"), "Exterior");
        assert_eq!(result, "https://ref.gs1.org/cbv/Comp-Exterior");
    }

    #[test]
    fn test_normalize_cbv_already_full_uri_unchanged() {
        let uri = "https://ref.gs1.org/cbv/BizStep-shipping";
        let result = normalize_cbv_value(None, Some("bizStep"), uri);
        assert_eq!(result, uri);
    }

    // ── numeric normalisation ─────────────────────────────────────────────────

    #[test]
    fn test_normalize_numeric_integer_no_trailing_zero() {
        assert_eq!(normalize_numeric("1.0"), "1");
        assert_eq!(normalize_numeric("10.00"), "10");
        assert_eq!(normalize_numeric("0.0"), "0");
    }

    #[test]
    fn test_normalize_numeric_float_preserved() {
        assert_eq!(normalize_numeric("0.3434"), "0.3434");
        assert_eq!(normalize_numeric("3.14"), "3.14");
    }

    #[test]
    fn test_normalize_numeric_non_number_passthrough() {
        assert_eq!(normalize_numeric("abc"), "abc");
        assert_eq!(normalize_numeric(""), "");
    }

    #[test]
    fn test_normalize_numeric_matches_reference_float_semantics() {
        // The reference implementation routes numerics through a 64-bit float,
        // so values above 2^53 round — reproducing that exactly is required
        // for hash interoperability (SensorDataExamples.xml relies on it).
        assert_eq!(normalize_numeric("9007199254740993"), "9007199254740992");
        assert_eq!(
            normalize_numeric("111100001111000011110000"),
            "111100001111000003641344"
        );
    }

    #[test]
    fn test_normalize_numeric_inf_nan_passthrough() {
        // f64::from_str accepts these, but they are text in EPCIS terms
        assert_eq!(normalize_numeric("inf"), "inf");
        assert_eq!(normalize_numeric("NaN"), "NaN");
        assert_eq!(normalize_numeric("-infinity"), "-infinity");
    }

    // ── URI normalisation ─────────────────────────────────────────────────────

    #[test]
    fn test_normalise_sgtin() {
        let urn = "urn:epc:id:sgtin:4012345.011111.987";
        let result = normalise_uri(urn);
        // Should convert to GS1 Web Vocabulary form
        assert!(result.starts_with("https://id.gs1.org/"), "got: {result}");
    }

    #[test]
    fn test_normalise_sscc() {
        let urn = "urn:epc:id:sscc:4012345.0111111111";
        let result = normalise_uri(urn);
        assert!(result.starts_with("https://id.gs1.org/"), "got: {result}");
    }

    #[test]
    fn test_normalise_unknown_uri_passthrough() {
        let uri = "https://example.com/foo/bar";
        assert_eq!(normalise_uri(uri), uri);
    }

    // ── web vocabulary formatting ─────────────────────────────────────────────

    #[test]
    fn test_try_format_web_vocabulary_cbv_prefix() {
        // gs1:shipping → https://ref.gs1.org/cbv/BizStep-shipping
        let result = try_format_web_vocabulary("gs1:shipping");
        // Should expand gs1: prefix
        assert!(
            !result.starts_with("gs1:"),
            "expected expansion, got: {result}"
        );
    }

    #[test]
    fn test_try_format_web_vocabulary_passthrough() {
        let uri = "https://example.com/something";
        assert_eq!(try_format_web_vocabulary(uri), uri);
    }

    // ── bubble_up_bare_extensions ─────────────────────────────────────────────

    #[test]
    fn test_bubble_up_bare_extensions_empty() {
        let mut root = ContextNode {
            name: None,
            value: None,
            children: vec![],
        };
        root.bubble_up_bare_extensions(None);
        assert!(root.children.is_empty());
    }

    #[test]
    fn test_bubble_up_bare_extensions_does_not_modify_named_children() {
        let child = ContextNode {
            name: Some("ex:foo".to_string()),
            value: Some("bar".to_string()),
            children: vec![],
        };
        let mut root = ContextNode {
            name: Some("ex:parent".to_string()),
            value: None,
            children: vec![child],
        };
        root.bubble_up_bare_extensions(None);
        // Named children should stay
        assert_eq!(root.children.len(), 1);
    }

    // ── compute_hash_from_prehash ─────────────────────────────────────────────

    #[test]
    fn test_compute_hash_format() {
        let hash = compute_hash_from_prehash("test input");
        assert!(hash.starts_with("ni:///sha-256;"), "wrong prefix: {hash}");
        assert!(hash.ends_with("?ver=CBV2.0"), "wrong suffix: {hash}");
    }

    #[test]
    fn test_compute_hash_deterministic() {
        let a = compute_hash_from_prehash("hello world");
        let b = compute_hash_from_prehash("hello world");
        assert_eq!(a, b);
    }

    #[test]
    fn test_compute_hash_different_inputs_differ() {
        let a = compute_hash_from_prehash("event A");
        let b = compute_hash_from_prehash("event B");
        assert_ne!(a, b);
    }
}

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm")]
/// Generates canonical hashes for all events in an XML document in WebAssembly environments.
///
/// # Errors
/// Returns an error string if XML parsing or canonicalization fails.
#[wasm_bindgen]
pub fn hash_xml_document_wasm(xml_str: &str, is_cbv_2_0: bool) -> Result<String, String> {
    let prehashes = canonicalize_xml(xml_str, is_cbv_2_0).map_err(|e| e.to_string())?;
    let mut hashes = Vec::new();
    for line in prehashes.lines() {
        if !line.trim().is_empty() {
            hashes.push(compute_hash_from_prehash(line));
        }
    }
    Ok(hashes.join("\n"))
}

#[cfg(feature = "wasm")]
/// Generates canonical hashes for all events in a JSON/JSON-LD document in WebAssembly environments.
///
/// # Errors
/// Returns an error string if JSON parsing or canonicalization fails.
#[wasm_bindgen]
pub fn hash_json_document_wasm(json_str: &str, is_cbv_2_0: bool) -> Result<String, String> {
    let json_val = serde_json::from_str(json_str).map_err(|e| e.to_string())?;
    let prehashes = canonicalize_json(&json_val, is_cbv_2_0).map_err(|e| e.to_string())?;
    let mut hashes = Vec::new();
    for line in prehashes.lines() {
        if !line.trim().is_empty() {
            hashes.push(compute_hash_from_prehash(line));
        }
    }
    Ok(hashes.join("\n"))
}
