//! Zero-allocation bidirectional translators for GS1 identifiers.
//!
//! Supports parsing and generating both EPC URNs and GS1 Digital Link formats
//! for SGTIN, SSCC, SGLN, GRAI, and GIAI keys without allocating memory where possible.

#![deny(missing_docs)]
#![deny(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

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
        let body = urn.strip_prefix("urn:epc:id:sgtin:")
            .ok_or(ParseError::InvalidPrefix)?;
        let mut parts = body.split('.');
        let company_prefix = parts.next().ok_or(ParseError::MissingField)?;
        let indicator_and_item_ref = parts.next().ok_or(ParseError::MissingField)?;
        let serial_number = parts.next().ok_or(ParseError::MissingField)?;
        if parts.next().is_some() {
            return Err(ParseError::ExtraFields);
        }

        if indicator_and_item_ref.is_empty() {
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

        if gtin.len() != 14 {
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
        let gtin_without_check = format!("{}{}{}", self.indicator, self.company_prefix, self.item_ref);
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
        let body = urn.strip_prefix("urn:epc:id:sscc:")
            .ok_or(ParseError::InvalidPrefix)?;
        let mut parts = body.split('.');
        let company_prefix = parts.next().ok_or(ParseError::MissingField)?;
        let ext_and_serial = parts.next().ok_or(ParseError::MissingField)?;
        if parts.next().is_some() {
            return Err(ParseError::ExtraFields);
        }

        if ext_and_serial.is_empty() {
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

        if sscc.len() != 18 {
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
        let sscc_without_check = format!("{}{}{}", self.extension_digit, self.company_prefix, self.serial_ref);
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
        let body = urn.strip_prefix("urn:epc:id:sgln:")
            .ok_or(ParseError::InvalidPrefix)?;
        let mut parts = body.split('.');
        let company_prefix = parts.next().ok_or(ParseError::MissingField)?;
        let location_reference = parts.next().ok_or(ParseError::MissingField)?;
        let extension = parts.next().ok_or(ParseError::MissingField)?;
        if parts.next().is_some() {
            return Err(ParseError::ExtraFields);
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

        let next_ai = parts.next().ok_or(ParseError::MissingField)?;
        if next_ai != "254" {
            return Err(ParseError::InvalidFormat);
        }
        let extension = parts.next().ok_or(ParseError::MissingField)?;

        if gln.len() != 13 {
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
    #[must_use]
    pub fn to_digital_link(&self, base_url: &str) -> String {
        let gln_without_check = format!("{}{}", self.company_prefix, self.location_reference);
        let check_digit = calculate_check_digit(&gln_without_check);
        format!(
            "{}/414/{}{}/254/{}",
            base_url.trim_end_matches('/'),
            gln_without_check,
            check_digit,
            self.extension
        )
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
        let body = urn.strip_prefix("urn:epc:id:grai:")
            .ok_or(ParseError::InvalidPrefix)?;
        let mut parts = body.split('.');
        let company_prefix = parts.next().ok_or(ParseError::MissingField)?;
        let asset_type = parts.next().ok_or(ParseError::MissingField)?;
        let serial_number = parts.next().ok_or(ParseError::MissingField)?;
        if parts.next().is_some() {
            return Err(ParseError::ExtraFields);
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
        if path.len() < 14 {
            return Err(ParseError::InvalidFormat);
        }
        let grai_id = &path[0..14];
        let serial_number = &path[14..];

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
        let body = urn.strip_prefix("urn:epc:id:giai:")
            .ok_or(ParseError::InvalidPrefix)?;
        let mut parts = body.split('.');
        let company_prefix = parts.next().ok_or(ParseError::MissingField)?;
        let individual_asset_reference = parts.next().ok_or(ParseError::MissingField)?;
        if parts.next().is_some() {
            return Err(ParseError::ExtraFields);
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

        if raw_giai.len() <= prefix_len {
            return Err(ParseError::InvalidFormat);
        }

        let company_prefix = &raw_giai[0..prefix_len];
        let individual_asset_reference = &raw_giai[prefix_len..];

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
    if remainder == 0 {
        0
    } else {
        10 - remainder
    }
}
