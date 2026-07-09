#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![deny(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

/// Parsing errors that can occur during identifier translation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    /// The URN scheme or prefix is invalid (e.g. missing "urn:epc:id:").
    InvalidPrefix,
    /// A required field is missing from the input string.
    MissingField,
    /// The input string contains extra fields or elements.
    ExtraFields,
    /// General format error.
    InvalidFormat,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::InvalidPrefix => write!(f, "Invalid identifier prefix"),
            ParseError::MissingField => write!(f, "Missing field in identifier"),
            ParseError::ExtraFields => write!(f, "Unexpected extra fields"),
            ParseError::InvalidFormat => write!(f, "Invalid identifier format"),
        }
    }
}

impl std::error::Error for ParseError {}

/// Serialized Global Trade Item Number (SGTIN).
///
/// # Examples
///
/// ```
/// use epcis_translate::Sgtin;
///
/// // Parse from URN
/// let urn = "urn:epc:id:sgtin:4012345.098765.12345";
/// let sgtin = Sgtin::from_urn(urn).unwrap();
/// assert_eq!(sgtin.company_prefix, "4012345");
/// assert_eq!(sgtin.serial_number, "12345");
///
/// // Translate to GS1 Digital Link URL path
/// let dl = sgtin.to_digital_link("https://id.gs1.org");
/// assert_eq!(dl, "https://id.gs1.org/01/04012345987652/21/12345");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sgtin<'a> {
    /// GS1 Company Prefix
    pub company_prefix: &'a str,
    /// Indicator digit (placed before the item reference)
    pub indicator: &'a str,
    /// Item reference
    pub item_ref: &'a str,
    /// Serial number
    pub serial_number: &'a str,
}

impl<'a> Sgtin<'a> {
    /// Parses an SGTIN from a URN format.
    /// E.g. `urn:epc:id:sgtin:CompanyPrefix.IndicatorAndItemRef.SerialNumber`
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if parsing fails.
    pub fn from_urn(urn: &'a str) -> Result<Self, ParseError> {
        let body = urn
            .strip_prefix("urn:epc:id:sgtin:")
            .ok_or(ParseError::InvalidPrefix)?;
        let mut parts = body.split('.');
        let company_prefix = parts.next().ok_or(ParseError::MissingField)?;
        let indicator_and_item_ref = parts.next().ok_or(ParseError::MissingField)?;
        let serial_number = parts.next().ok_or(ParseError::MissingField)?;
        if parts.next().is_some() {
            return Err(ParseError::ExtraFields);
        }

        if !is_ascii_digits(company_prefix) || !is_ascii_digits(indicator_and_item_ref) {
            return Err(ParseError::InvalidFormat);
        }
        let indicator = &indicator_and_item_ref[0..1];
        let item_ref = &indicator_and_item_ref[1..];

        Ok(Self {
            company_prefix,
            indicator,
            item_ref,
            serial_number,
        })
    }

    /// Converts the SGTIN to its URN representation.
    #[must_use]
    pub fn to_urn(&self) -> String {
        format!(
            "urn:epc:id:sgtin:{}.{}{}.{}",
            self.company_prefix, self.indicator, self.item_ref, self.serial_number
        )
    }

    /// Parses an SGTIN from a GS1 Digital Link path structure.
    /// E.g. `/01/GTIN/21/SERIAL`
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if digital link pattern is not matched.
    pub fn from_digital_link(url: &'a str, prefix_len: usize) -> Result<Self, ParseError> {
        let idx = url.find("/01/").ok_or(ParseError::InvalidFormat)?;
        let path = &url[idx + 4..];
        let mut parts = path.split('/');
        let gtin = parts.next().ok_or(ParseError::MissingField)?;
        let ai = parts.next().ok_or(ParseError::MissingField)?;
        if ai != "21" {
            return Err(ParseError::InvalidFormat);
        }
        let serial_number = parts.next().ok_or(ParseError::MissingField)?;

        if gtin.len() != 14 || !is_ascii_digits(gtin) {
            return Err(ParseError::InvalidFormat);
        }
        if prefix_len >= 13 {
            return Err(ParseError::InvalidFormat);
        }

        let indicator = &gtin[0..1];
        let company_prefix = &gtin[1..=prefix_len];
        let item_ref = &gtin[1 + prefix_len..13];

        Ok(Self {
            company_prefix,
            indicator,
            item_ref,
            serial_number,
        })
    }

    /// Converts the SGTIN to a Digital Link URL using a default base domain.
    #[must_use]
    pub fn to_digital_link(&self, base_url: &str) -> String {
        // Compute dummy check digit or placeholder for check digit
        // Typically it is a single check digit (we use 0 or calculate standard Luhn checksum)
        // Let's use standard GS1 Luhn checksum for correctness
        let gtin_without_check =
            format!("{}{}{}", self.indicator, self.company_prefix, self.item_ref);
        let check_digit = calculate_check_digit(&gtin_without_check);
        format!(
            "{}/01/{}{}/21/{}",
            base_url.trim_end_matches('/'),
            gtin_without_check,
            check_digit,
            self.serial_number
        )
    }
}

/// Serial Shipping Container Code (SSCC).
///
/// # Examples
///
/// ```
/// use epcis_translate::Sscc;
///
/// // Parse from URN
/// let urn = "urn:epc:id:sscc:4012345.0123456789";
/// let sscc = Sscc::from_urn(urn).unwrap();
/// assert_eq!(sscc.company_prefix, "4012345");
/// assert_eq!(sscc.serial_ref, "123456789");
///
/// // Translate to GS1 Digital Link format
/// let dl = sscc.to_digital_link("https://id.gs1.org");
/// assert_eq!(dl, "https://id.gs1.org/00/040123451234567894");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sscc<'a> {
    /// GS1 Company Prefix
    pub company_prefix: &'a str,
    /// Extension digit
    pub extension_digit: &'a str,
    /// Serial reference
    pub serial_ref: &'a str,
}

impl<'a> Sscc<'a> {
    /// Parses an SSCC from a URN format.
    /// E.g. `urn:epc:id:sscc:CompanyPrefix.ExtensionAndSerialRef`
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if parsing fails.
    pub fn from_urn(urn: &'a str) -> Result<Self, ParseError> {
        let body = urn
            .strip_prefix("urn:epc:id:sscc:")
            .ok_or(ParseError::InvalidPrefix)?;
        let mut parts = body.split('.');
        let company_prefix = parts.next().ok_or(ParseError::MissingField)?;
        let ext_and_serial = parts.next().ok_or(ParseError::MissingField)?;
        if parts.next().is_some() {
            return Err(ParseError::ExtraFields);
        }

        if !is_ascii_digits(company_prefix) || !is_ascii_digits(ext_and_serial) {
            return Err(ParseError::InvalidFormat);
        }
        let extension_digit = &ext_and_serial[0..1];
        let serial_ref = &ext_and_serial[1..];

        Ok(Self {
            company_prefix,
            extension_digit,
            serial_ref,
        })
    }

    /// Converts the SSCC to its URN representation.
    #[must_use]
    pub fn to_urn(&self) -> String {
        format!(
            "urn:epc:id:sscc:{}.{}{}",
            self.company_prefix, self.extension_digit, self.serial_ref
        )
    }

    /// Parses an SSCC from a GS1 Digital Link path structure.
    /// E.g. `/00/SSCC`
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if format is invalid.
    pub fn from_digital_link(url: &'a str, prefix_len: usize) -> Result<Self, ParseError> {
        let idx = url.find("/00/").ok_or(ParseError::InvalidFormat)?;
        let path = &url[idx + 4..];
        let mut parts = path.split('/');
        let sscc = parts.next().ok_or(ParseError::MissingField)?;

        if sscc.len() != 18 || !is_ascii_digits(sscc) {
            return Err(ParseError::InvalidFormat);
        }
        if prefix_len >= 17 {
            return Err(ParseError::InvalidFormat);
        }

        let extension_digit = &sscc[0..1];
        let company_prefix = &sscc[1..=prefix_len];
        let serial_ref = &sscc[1 + prefix_len..17];

        Ok(Self {
            company_prefix,
            extension_digit,
            serial_ref,
        })
    }

    /// Converts the SSCC to a Digital Link URL.
    #[must_use]
    pub fn to_digital_link(&self, base_url: &str) -> String {
        let sscc_without_check = format!(
            "{}{}{}",
            self.extension_digit, self.company_prefix, self.serial_ref
        );
        let check_digit = calculate_check_digit(&sscc_without_check);
        format!(
            "{}/00/{}{}",
            base_url.trim_end_matches('/'),
            sscc_without_check,
            check_digit
        )
    }
}

/// Global Location Number (SGLN) with optional extension.
///
/// # Examples
///
/// ```
/// use epcis_translate::Sgln;
///
/// // Parse from URN
/// let urn = "urn:epc:id:sgln:4012345.01234.567";
/// let sgln = Sgln::from_urn(urn).unwrap();
/// assert_eq!(sgln.company_prefix, "4012345");
/// assert_eq!(sgln.extension, "567");
///
/// // Translate to GS1 Digital Link path structure
/// let dl = sgln.to_digital_link("https://id.gs1.org");
/// assert_eq!(dl, "https://id.gs1.org/414/4012345012347/254/567");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sgln<'a> {
    /// GS1 Company Prefix
    pub company_prefix: &'a str,
    /// Location Reference
    pub location_reference: &'a str,
    /// GLN extension string
    pub extension: &'a str,
}

impl<'a> Sgln<'a> {
    /// Parses an SGLN from a URN format.
    /// E.g. `urn:epc:id:sgln:CompanyPrefix.LocationRef.Extension`
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if parsing fails.
    pub fn from_urn(urn: &'a str) -> Result<Self, ParseError> {
        let body = urn
            .strip_prefix("urn:epc:id:sgln:")
            .ok_or(ParseError::InvalidPrefix)?;
        let mut parts = body.split('.');
        let company_prefix = parts.next().ok_or(ParseError::MissingField)?;
        let location_reference = parts.next().ok_or(ParseError::MissingField)?;
        let extension = parts.next().ok_or(ParseError::MissingField)?;
        if parts.next().is_some() {
            return Err(ParseError::ExtraFields);
        }

        // Location reference may be empty (12-digit company prefixes) but must
        // be numeric when present; the extension is free-form per GS1.
        if !is_ascii_digits(company_prefix)
            || !location_reference.bytes().all(|b| b.is_ascii_digit())
        {
            return Err(ParseError::InvalidFormat);
        }

        Ok(Self {
            company_prefix,
            location_reference,
            extension,
        })
    }

    /// Converts the SGLN to its URN representation.
    #[must_use]
    pub fn to_urn(&self) -> String {
        format!(
            "urn:epc:id:sgln:{}.{}.{}",
            self.company_prefix, self.location_reference, self.extension
        )
    }

    /// Parses an SGLN from a GS1 Digital Link path structure.
    /// E.g. `/414/GLN/254/EXTENSION`
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if format is invalid.
    pub fn from_digital_link(url: &'a str, prefix_len: usize) -> Result<Self, ParseError> {
        let idx = url.find("/414/").ok_or(ParseError::InvalidFormat)?;
        let path = &url[idx + 5..];
        let mut parts = path.split('/');
        let gln = parts.next().ok_or(ParseError::MissingField)?;

        // A plain GLN without a /254/ qualifier means "no extension" (GS1
        // encodes that as extension "0"); any other qualifier is invalid.
        let extension = match parts.next() {
            None => "0",
            Some("254") => parts.next().ok_or(ParseError::MissingField)?,
            Some(_) => return Err(ParseError::InvalidFormat),
        };

        if gln.len() != 13 || !is_ascii_digits(gln) {
            return Err(ParseError::InvalidFormat);
        }
        if prefix_len >= 12 {
            return Err(ParseError::InvalidFormat);
        }

        let company_prefix = &gln[0..prefix_len];
        let location_reference = &gln[prefix_len..12];

        Ok(Self {
            company_prefix,
            location_reference,
            extension,
        })
    }

    /// Converts the SGLN to a Digital Link URL.
    ///
    /// Extension `"0"` means "no extension" per GS1, so the `/254/` qualifier
    /// is omitted in that case — matching the canonical form used by
    /// `epcis-hash` when normalizing SGLN URNs.
    #[must_use]
    pub fn to_digital_link(&self, base_url: &str) -> String {
        let gln_without_check = format!("{}{}", self.company_prefix, self.location_reference);
        let check_digit = calculate_check_digit(&gln_without_check);
        if self.extension == "0" {
            format!(
                "{}/414/{}{}",
                base_url.trim_end_matches('/'),
                gln_without_check,
                check_digit
            )
        } else {
            format!(
                "{}/414/{}{}/254/{}",
                base_url.trim_end_matches('/'),
                gln_without_check,
                check_digit,
                self.extension
            )
        }
    }
}

/// Global Returnable Asset Identifier (GRAI).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grai<'a> {
    /// GS1 Company Prefix
    pub company_prefix: &'a str,
    /// Asset type identifier
    pub asset_type: &'a str,
    /// Serial number
    pub serial_number: &'a str,
}

impl<'a> Grai<'a> {
    /// Parses a GRAI from a URN format.
    /// E.g. `urn:epc:id:grai:CompanyPrefix.AssetType.SerialNumber`
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if parsing fails.
    pub fn from_urn(urn: &'a str) -> Result<Self, ParseError> {
        let body = urn
            .strip_prefix("urn:epc:id:grai:")
            .ok_or(ParseError::InvalidPrefix)?;
        let mut parts = body.split('.');
        let company_prefix = parts.next().ok_or(ParseError::MissingField)?;
        let asset_type = parts.next().ok_or(ParseError::MissingField)?;
        let serial_number = parts.next().ok_or(ParseError::MissingField)?;
        if parts.next().is_some() {
            return Err(ParseError::ExtraFields);
        }

        // Asset type may be empty (12-digit company prefixes) but must be
        // numeric when present; the serial number is free-form per GS1.
        if !is_ascii_digits(company_prefix) || !asset_type.bytes().all(|b| b.is_ascii_digit()) {
            return Err(ParseError::InvalidFormat);
        }

        Ok(Self {
            company_prefix,
            asset_type,
            serial_number,
        })
    }

    /// Converts the GRAI to its URN representation.
    #[must_use]
    pub fn to_urn(&self) -> String {
        format!(
            "urn:epc:id:grai:{}.{}.{}",
            self.company_prefix, self.asset_type, self.serial_number
        )
    }

    /// Parses a GRAI from a GS1 Digital Link path structure.
    /// E.g. `/8003/GRAI` (GRAI-14 + Serial)
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if format is invalid.
    pub fn from_digital_link(url: &'a str, prefix_len: usize) -> Result<Self, ParseError> {
        let idx = url.find("/8003/").ok_or(ParseError::InvalidFormat)?;
        let path = &url[idx + 6..];

        // Find serial division if there is one (GRAI serials are appended to the 14-digit asset type)
        // GRAI-14 is always the first 14 digits, serial number follows it directly or after optional parameters
        // The 14-digit GRAI id must be numeric before slicing; the serial that
        // follows is free-form per GS1.
        let Some((grai_id, serial_number)) = path.split_at_checked(14) else {
            return Err(ParseError::InvalidFormat);
        };
        if !is_ascii_digits(grai_id) {
            return Err(ParseError::InvalidFormat);
        }

        if prefix_len >= 12 {
            return Err(ParseError::InvalidFormat);
        }

        // GRAI has first digit 0
        let company_prefix = &grai_id[1..=prefix_len];
        let asset_type = &grai_id[1 + prefix_len..13];

        Ok(Self {
            company_prefix,
            asset_type,
            serial_number,
        })
    }

    /// Converts the GRAI to a Digital Link URL.
    #[must_use]
    pub fn to_digital_link(&self, base_url: &str) -> String {
        let grai_without_check = format!("0{}{}", self.company_prefix, self.asset_type);
        let check_digit = calculate_check_digit(&grai_without_check);
        format!(
            "{}/8003/{}{}{}",
            base_url.trim_end_matches('/'),
            grai_without_check,
            check_digit,
            self.serial_number
        )
    }
}

/// Global Individual Asset Identifier (GIAI).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Giai<'a> {
    /// GS1 Company Prefix
    pub company_prefix: &'a str,
    /// Individual Asset Reference
    pub individual_asset_reference: &'a str,
}

impl<'a> Giai<'a> {
    /// Parses a GIAI from a URN format.
    /// E.g. `urn:epc:id:giai:CompanyPrefix.IndividualAssetRef`
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if parsing fails.
    pub fn from_urn(urn: &'a str) -> Result<Self, ParseError> {
        let body = urn
            .strip_prefix("urn:epc:id:giai:")
            .ok_or(ParseError::InvalidPrefix)?;
        let mut parts = body.split('.');
        let company_prefix = parts.next().ok_or(ParseError::MissingField)?;
        let individual_asset_reference = parts.next().ok_or(ParseError::MissingField)?;
        if parts.next().is_some() {
            return Err(ParseError::ExtraFields);
        }

        // The asset reference is free-form per GS1; only the company prefix
        // must be numeric.
        if !is_ascii_digits(company_prefix) {
            return Err(ParseError::InvalidFormat);
        }

        Ok(Self {
            company_prefix,
            individual_asset_reference,
        })
    }

    /// Converts the GIAI to its URN representation.
    #[must_use]
    pub fn to_urn(&self) -> String {
        format!(
            "urn:epc:id:giai:{}.{}",
            self.company_prefix, self.individual_asset_reference
        )
    }

    /// Parses a GIAI from a GS1 Digital Link path structure.
    /// E.g. `/8004/GIAI`
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if format is invalid.
    pub fn from_digital_link(url: &'a str, prefix_len: usize) -> Result<Self, ParseError> {
        let idx = url.find("/8004/").ok_or(ParseError::InvalidFormat)?;
        let path = &url[idx + 6..];
        let mut parts = path.split('/');
        let raw_giai = parts.next().ok_or(ParseError::MissingField)?;

        // The company prefix portion must be numeric before slicing; the
        // remaining asset reference is free-form per GS1.
        let Some((company_prefix, individual_asset_reference)) =
            raw_giai.split_at_checked(prefix_len)
        else {
            return Err(ParseError::InvalidFormat);
        };
        if !is_ascii_digits(company_prefix) || individual_asset_reference.is_empty() {
            return Err(ParseError::InvalidFormat);
        }

        Ok(Self {
            company_prefix,
            individual_asset_reference,
        })
    }

    /// Converts the GIAI to a Digital Link URL.
    #[must_use]
    pub fn to_digital_link(&self, base_url: &str) -> String {
        format!(
            "{}/8004/{}{}",
            base_url.trim_end_matches('/'),
            self.company_prefix,
            self.individual_asset_reference
        )
    }
}

/// Returns true if the string is non-empty and consists solely of ASCII digits.
///
/// Numeric identifier segments must be validated with this before any byte
/// slicing: it guarantees every byte is a char boundary and rejects garbage
/// that would otherwise produce nonsense identifiers.
fn is_ascii_digits(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

/// Helper to calculate the GS1 Luhn-like check digit.
fn calculate_check_digit(digits: &str) -> u32 {
    let mut sum = 0;
    // Iterate from right to left
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sgtin_parse_errors() {
        // URN Errors
        assert_eq!(
            Sgtin::from_urn("urn:epc:id:sscc:4012345.098765.12345"),
            Err(ParseError::InvalidPrefix)
        );
        assert_eq!(
            Sgtin::from_urn("urn:epc:id:sgtin:4012345"),
            Err(ParseError::MissingField)
        );
        assert_eq!(
            Sgtin::from_urn("urn:epc:id:sgtin:4012345.098765.12345.extra"),
            Err(ParseError::ExtraFields)
        );
        assert_eq!(
            Sgtin::from_urn("urn:epc:id:sgtin:4012345..12345"),
            Err(ParseError::InvalidFormat)
        );

        // Digital Link Errors
        assert_eq!(
            Sgtin::from_digital_link("https://id.gs1.org/00/340123450123456784", 7),
            Err(ParseError::InvalidFormat)
        );
        assert_eq!(
            Sgtin::from_digital_link("https://id.gs1.org/01/04012345987652/22/12345", 7),
            Err(ParseError::InvalidFormat)
        );
        assert_eq!(
            Sgtin::from_digital_link("https://id.gs1.org/01/0401234598765/21/12345", 7),
            Err(ParseError::InvalidFormat)
        ); // length != 14
        assert_eq!(
            Sgtin::from_digital_link("https://id.gs1.org/01/04012345987652/21/12345", 14),
            Err(ParseError::InvalidFormat)
        ); // prefix_len >= 13
    }

    #[test]
    fn test_sscc_parse_errors() {
        // URN Errors
        assert_eq!(
            Sscc::from_urn("urn:epc:id:sgtin:4012345.0123456789"),
            Err(ParseError::InvalidPrefix)
        );
        assert_eq!(
            Sscc::from_urn("urn:epc:id:sscc:4012345"),
            Err(ParseError::MissingField)
        );
        assert_eq!(
            Sscc::from_urn("urn:epc:id:sscc:4012345.0123456789.extra"),
            Err(ParseError::ExtraFields)
        );
        assert_eq!(
            Sscc::from_urn("urn:epc:id:sscc:"),
            Err(ParseError::MissingField)
        );

        // Digital Link Errors
        assert_eq!(
            Sscc::from_digital_link("https://id.gs1.org/01/04012345987652/21/12345", 7),
            Err(ParseError::InvalidFormat)
        );
        assert_eq!(
            Sscc::from_digital_link("https://id.gs1.org/00/34012345012345678", 7),
            Err(ParseError::InvalidFormat)
        ); // length != 18
        assert_eq!(
            Sscc::from_digital_link("https://id.gs1.org/00/340123450123456784", 17),
            Err(ParseError::InvalidFormat)
        ); // prefix_len >= 17
    }

    #[test]
    fn test_sgln_parse_errors() {
        // URN Errors
        assert_eq!(
            Sgln::from_urn("urn:epc:id:sscc:4012345.00001.0"),
            Err(ParseError::InvalidPrefix)
        );
        assert_eq!(
            Sgln::from_urn("urn:epc:id:sgln:4012345.00001"),
            Err(ParseError::MissingField)
        );
        assert_eq!(
            Sgln::from_urn("urn:epc:id:sgln:4012345.00001.0.extra"),
            Err(ParseError::ExtraFields)
        );

        // Digital Link Errors
        assert_eq!(
            Sgln::from_digital_link("https://id.gs1.org/415/4012345000016/254/0", 7),
            Err(ParseError::InvalidFormat)
        );
        assert_eq!(
            Sgln::from_digital_link("https://id.gs1.org/414/4012345000016/255/0", 7),
            Err(ParseError::InvalidFormat)
        );
        assert_eq!(
            Sgln::from_digital_link("https://id.gs1.org/414/401234500001/254/0", 7),
            Err(ParseError::InvalidFormat)
        ); // length != 13
        assert_eq!(
            Sgln::from_digital_link("https://id.gs1.org/414/4012345000016/254/0", 12),
            Err(ParseError::InvalidFormat)
        ); // prefix_len >= 12
    }

    #[test]
    fn test_grai_parse_errors() {
        // URN Errors
        assert_eq!(
            Grai::from_urn("urn:epc:id:sscc:4012345.00001.12345"),
            Err(ParseError::InvalidPrefix)
        );
        assert_eq!(
            Grai::from_urn("urn:epc:id:grai:4012345.00001"),
            Err(ParseError::MissingField)
        );
        assert_eq!(
            Grai::from_urn("urn:epc:id:grai:4012345.00001.12345.extra"),
            Err(ParseError::ExtraFields)
        );

        // Digital Link Errors
        assert_eq!(
            Grai::from_digital_link("https://id.gs1.org/8004/0401234500001612345", 7),
            Err(ParseError::InvalidFormat)
        );
        assert_eq!(
            Grai::from_digital_link("https://id.gs1.org/8003/0401234500001", 7),
            Err(ParseError::InvalidFormat)
        ); // length < 14
        assert_eq!(
            Grai::from_digital_link("https://id.gs1.org/8003/0401234500001612345", 12),
            Err(ParseError::InvalidFormat)
        ); // prefix_len >= 12
    }

    #[test]
    fn test_giai_parse_errors() {
        // URN Errors
        assert_eq!(
            Giai::from_urn("urn:epc:id:sscc:4012345.12345"),
            Err(ParseError::InvalidPrefix)
        );
        assert_eq!(
            Giai::from_urn("urn:epc:id:giai:4012345"),
            Err(ParseError::MissingField)
        );
        assert_eq!(
            Giai::from_urn("urn:epc:id:giai:4012345.12345.extra"),
            Err(ParseError::ExtraFields)
        );

        // Digital Link Errors
        assert_eq!(
            Giai::from_digital_link("https://id.gs1.org/8003/401234512345", 7),
            Err(ParseError::InvalidFormat)
        );
        assert_eq!(
            Giai::from_digital_link("https://id.gs1.org/8004/401234512345", 15),
            Err(ParseError::InvalidFormat)
        ); // prefix_len >= raw_giai.len()
    }

    #[test]
    fn test_multibyte_input_returns_error_not_panic() {
        // Multi-byte UTF-8 at slicing positions previously panicked
        assert_eq!(
            Sgtin::from_urn("urn:epc:id:sgtin:4012345.é8765.1"),
            Err(ParseError::InvalidFormat)
        );
        assert_eq!(
            Sscc::from_urn("urn:epc:id:sscc:4012345.é123456789"),
            Err(ParseError::InvalidFormat)
        );
        assert_eq!(
            Sgln::from_urn("urn:epc:id:sgln:4012345.é0001.0"),
            Err(ParseError::InvalidFormat)
        );
        assert_eq!(
            Grai::from_urn("urn:epc:id:grai:4012345.é0001.1"),
            Err(ParseError::InvalidFormat)
        );
        assert_eq!(
            Giai::from_urn("urn:epc:id:giai:é012345.12345"),
            Err(ParseError::InvalidFormat)
        );
        assert_eq!(
            Sgtin::from_digital_link("https://id.gs1.org/01/é4012345987652/21/1", 7),
            Err(ParseError::InvalidFormat)
        );
        assert_eq!(
            Grai::from_digital_link("https://id.gs1.org/8003/é401234500001612345", 7),
            Err(ParseError::InvalidFormat)
        );
        assert_eq!(
            Giai::from_digital_link("https://id.gs1.org/8004/é01234512345", 7),
            Err(ParseError::InvalidFormat)
        );
    }

    #[test]
    fn test_non_numeric_segments_rejected() {
        assert_eq!(
            Sgtin::from_urn("urn:epc:id:sgtin:abcdefg.hijkl.serial"),
            Err(ParseError::InvalidFormat)
        );
        assert_eq!(
            Sscc::from_urn("urn:epc:id:sscc:4012345.0123x5678"),
            Err(ParseError::InvalidFormat)
        );
        assert_eq!(
            Sgln::from_urn("urn:epc:id:sgln:40123a5.00001.0"),
            Err(ParseError::InvalidFormat)
        );
        assert_eq!(
            Grai::from_urn("urn:epc:id:grai:4012345.000x1.12345"),
            Err(ParseError::InvalidFormat)
        );
        assert_eq!(
            Sgtin::from_digital_link("https://id.gs1.org/01/0401234598765X/21/12345", 7),
            Err(ParseError::InvalidFormat)
        );
        // Free-form components stay permissive: serials, extensions, asset refs
        assert!(Sgtin::from_urn("urn:epc:id:sgtin:4012345.098765.ABC-123").is_ok());
        assert!(Giai::from_urn("urn:epc:id:giai:4012345.ASSET-9").is_ok());
    }

    #[test]
    fn test_sgln_extension_zero_omits_qualifier() {
        let sgln = Sgln::from_urn("urn:epc:id:sgln:4012345.00001.0").unwrap();
        assert_eq!(
            sgln.to_digital_link("https://id.gs1.org"),
            "https://id.gs1.org/414/4012345000016"
        );

        let parsed = Sgln::from_digital_link("https://id.gs1.org/414/4012345000016", 7).unwrap();
        assert_eq!(parsed.extension, "0");
        assert_eq!(parsed.to_urn(), "urn:epc:id:sgln:4012345.00001.0");
    }

    #[test]
    fn test_translation_performance_guard() {
        use std::time::Instant;

        let urn = "urn:epc:id:sgtin:4012345.098765.12345";
        let dl = "https://id.gs1.org/01/04012345987652/21/12345";

        let start = Instant::now();
        for _ in 0..100_000 {
            let sgtin = Sgtin::from_urn(urn).unwrap();
            let _dl_out = sgtin.to_digital_link("https://id.gs1.org");

            let sgtin_dl = Sgtin::from_digital_link(dl, 7).unwrap();
            let _urn_out = sgtin_dl.to_urn();
        }
        let elapsed = start.elapsed();
        println!("Translated 100k inputs in: {elapsed:?}");

        // Exceedingly high threshold to account for slow CPU environments, 100k roundtrips in 500ms
        assert!(
            elapsed.as_millis() < 500,
            "Performance regression detected! 100k roundtrips took {elapsed:?}"
        );
    }
}

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm")]
/// Translates an EPC URN to a GS1 Digital Link URL in WebAssembly environments.
///
/// # Errors
/// Returns an error string if parsing or format validation fails.
#[wasm_bindgen]
pub fn translate_urn_to_dl_wasm(urn: &str, base_url: &str) -> Result<String, String> {
    let parts: Vec<&str> = urn.split(':').collect();
    if parts.len() < 5 {
        return Err("Invalid URN format. Expected e.g. urn:epc:id:sgtin:...".to_string());
    }
    let scheme = parts[3];
    match scheme {
        "sgtin" => {
            let sgtin =
                Sgtin::from_urn(urn).map_err(|e| format!("Failed to parse SGTIN: {:?}", e))?;
            Ok(sgtin.to_digital_link(base_url))
        }
        "sscc" => {
            let sscc = Sscc::from_urn(urn).map_err(|e| format!("Failed to parse SSCC: {:?}", e))?;
            Ok(sscc.to_digital_link(base_url))
        }
        "sgln" => {
            let sgln = Sgln::from_urn(urn).map_err(|e| format!("Failed to parse SGLN: {:?}", e))?;
            Ok(sgln.to_digital_link(base_url))
        }
        "grai" => {
            let grai = Grai::from_urn(urn).map_err(|e| format!("Failed to parse GRAI: {:?}", e))?;
            Ok(grai.to_digital_link(base_url))
        }
        "giai" => {
            let giai = Giai::from_urn(urn).map_err(|e| format!("Failed to parse GIAI: {:?}", e))?;
            Ok(giai.to_digital_link(base_url))
        }
        other => Err(format!("Unsupported URN scheme: {}", other)),
    }
}

/// Translates a GS1 Digital Link URL to an EPC URN in WebAssembly environments.
///
/// # Errors
/// Returns an error string if parsing or check digit verification fails.
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn translate_dl_to_urn_wasm(dl: &str, prefix_len: usize) -> Result<String, String> {
    if dl.contains("/01/") {
        let sgtin = Sgtin::from_digital_link(dl, prefix_len)
            .map_err(|e| format!("Failed to parse SGTIN DL: {:?}", e))?;
        Ok(sgtin.to_urn())
    } else if dl.contains("/00/") {
        let sscc = Sscc::from_digital_link(dl, prefix_len)
            .map_err(|e| format!("Failed to parse SSCC DL: {:?}", e))?;
        Ok(sscc.to_urn())
    } else if dl.contains("/414/") {
        let sgln = Sgln::from_digital_link(dl, prefix_len)
            .map_err(|e| format!("Failed to parse SGLN DL: {:?}", e))?;
        Ok(sgln.to_urn())
    } else if dl.contains("/8003/") {
        let grai = Grai::from_digital_link(dl, prefix_len)
            .map_err(|e| format!("Failed to parse GRAI DL: {:?}", e))?;
        Ok(grai.to_urn())
    } else if dl.contains("/8004/") {
        let giai = Giai::from_digital_link(dl, prefix_len)
            .map_err(|e| format!("Failed to parse GIAI DL: {:?}", e))?;
        Ok(giai.to_urn())
    } else {
        Err("Could not detect GS1 Application Identifier (AI) in Digital Link path (expected e.g. /01/, /00/, /414/, /8003/, /8004/)".to_string())
    }
}
