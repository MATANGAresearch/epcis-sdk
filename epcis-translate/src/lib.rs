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

/// Party Global Location Number (PGLN).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pgln<'a> {
    /// GS1 Company Prefix
    pub company_prefix: &'a str,
    /// Party reference
    pub party_ref: &'a str,
}

impl<'a> Pgln<'a> {
    /// Parses a PGLN from a URN format.
    /// E.g. `urn:epc:id:pgln:CompanyPrefix.PartyRef`
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if parsing fails.
    pub fn from_urn(urn: &'a str) -> Result<Self, ParseError> {
        let body = urn
            .strip_prefix("urn:epc:id:pgln:")
            .ok_or(ParseError::InvalidPrefix)?;
        let mut parts = body.split('.');
        let company_prefix = parts.next().ok_or(ParseError::MissingField)?;
        let party_ref = parts.next().ok_or(ParseError::MissingField)?;
        if parts.next().is_some() {
            return Err(ParseError::ExtraFields);
        }

        // Party reference may be empty (12-digit company prefixes).
        if !is_ascii_digits(company_prefix) || !party_ref.bytes().all(|b| b.is_ascii_digit()) {
            return Err(ParseError::InvalidFormat);
        }

        Ok(Self {
            company_prefix,
            party_ref,
        })
    }

    /// Converts the PGLN to its URN representation.
    #[must_use]
    pub fn to_urn(&self) -> String {
        format!("urn:epc:id:pgln:{}.{}", self.company_prefix, self.party_ref)
    }

    /// Parses a PGLN from a GS1 Digital Link path structure.
    /// E.g. `/417/GLN`
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if format is invalid.
    pub fn from_digital_link(url: &'a str, prefix_len: usize) -> Result<Self, ParseError> {
        let idx = url.find("/417/").ok_or(ParseError::InvalidFormat)?;
        let path = &url[idx + 5..];
        let gln = path.split('/').next().ok_or(ParseError::MissingField)?;

        if gln.len() != 13 || !is_ascii_digits(gln) {
            return Err(ParseError::InvalidFormat);
        }
        if prefix_len >= 12 {
            return Err(ParseError::InvalidFormat);
        }

        Ok(Self {
            company_prefix: &gln[0..prefix_len],
            party_ref: &gln[prefix_len..12],
        })
    }

    /// Converts the PGLN to a Digital Link URL.
    #[must_use]
    pub fn to_digital_link(&self, base_url: &str) -> String {
        let gln_without_check = format!("{}{}", self.company_prefix, self.party_ref);
        let check_digit = calculate_check_digit(&gln_without_check);
        format!(
            "{}/417/{}{}",
            base_url.trim_end_matches('/'),
            gln_without_check,
            check_digit
        )
    }
}

/// Global Document Type Identifier (GDTI).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gdti<'a> {
    /// GS1 Company Prefix
    pub company_prefix: &'a str,
    /// Document type reference
    pub doc_type: &'a str,
    /// Serial number
    pub serial_number: &'a str,
}

impl<'a> Gdti<'a> {
    /// Parses a GDTI from a URN format.
    /// E.g. `urn:epc:id:gdti:CompanyPrefix.DocType.Serial`
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if parsing fails.
    pub fn from_urn(urn: &'a str) -> Result<Self, ParseError> {
        let body = urn
            .strip_prefix("urn:epc:id:gdti:")
            .ok_or(ParseError::InvalidPrefix)?;
        let mut parts = body.split('.');
        let company_prefix = parts.next().ok_or(ParseError::MissingField)?;
        let doc_type = parts.next().ok_or(ParseError::MissingField)?;
        let serial_number = parts.next().ok_or(ParseError::MissingField)?;
        if parts.next().is_some() {
            return Err(ParseError::ExtraFields);
        }

        // Document type may be empty (12-digit company prefixes); the serial
        // is free-form per GS1.
        if !is_ascii_digits(company_prefix) || !doc_type.bytes().all(|b| b.is_ascii_digit()) {
            return Err(ParseError::InvalidFormat);
        }

        Ok(Self {
            company_prefix,
            doc_type,
            serial_number,
        })
    }

    /// Converts the GDTI to its URN representation.
    #[must_use]
    pub fn to_urn(&self) -> String {
        format!(
            "urn:epc:id:gdti:{}.{}.{}",
            self.company_prefix, self.doc_type, self.serial_number
        )
    }

    /// Parses a GDTI from a GS1 Digital Link path structure.
    /// E.g. `/253/GDTI` (13-digit GDTI followed directly by the serial)
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if format is invalid.
    pub fn from_digital_link(url: &'a str, prefix_len: usize) -> Result<Self, ParseError> {
        let idx = url.find("/253/").ok_or(ParseError::InvalidFormat)?;
        let path = &url[idx + 5..];
        let segment = path.split('/').next().ok_or(ParseError::MissingField)?;

        let Some((gdti_id, serial_number)) = segment.split_at_checked(13) else {
            return Err(ParseError::InvalidFormat);
        };
        if !is_ascii_digits(gdti_id) || prefix_len >= 12 {
            return Err(ParseError::InvalidFormat);
        }

        Ok(Self {
            company_prefix: &gdti_id[0..prefix_len],
            doc_type: &gdti_id[prefix_len..12],
            serial_number,
        })
    }

    /// Converts the GDTI to a Digital Link URL.
    #[must_use]
    pub fn to_digital_link(&self, base_url: &str) -> String {
        let gdti_without_check = format!("{}{}", self.company_prefix, self.doc_type);
        let check_digit = calculate_check_digit(&gdti_without_check);
        format!(
            "{}/253/{}{}{}",
            base_url.trim_end_matches('/'),
            gdti_without_check,
            check_digit,
            self.serial_number
        )
    }
}

/// Global Service Relation Number — Recipient (GSRN).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gsrn<'a> {
    /// GS1 Company Prefix
    pub company_prefix: &'a str,
    /// Service reference
    pub service_ref: &'a str,
}

impl<'a> Gsrn<'a> {
    /// Parses a GSRN from a URN format.
    /// E.g. `urn:epc:id:gsrn:CompanyPrefix.ServiceRef`
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if parsing fails.
    pub fn from_urn(urn: &'a str) -> Result<Self, ParseError> {
        gsrn_like_from_urn(urn, "urn:epc:id:gsrn:").map(|(company_prefix, service_ref)| Self {
            company_prefix,
            service_ref,
        })
    }

    /// Converts the GSRN to its URN representation.
    #[must_use]
    pub fn to_urn(&self) -> String {
        format!(
            "urn:epc:id:gsrn:{}.{}",
            self.company_prefix, self.service_ref
        )
    }

    /// Parses a GSRN from a GS1 Digital Link path structure.
    /// E.g. `/8018/GSRN`
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if format is invalid.
    pub fn from_digital_link(url: &'a str, prefix_len: usize) -> Result<Self, ParseError> {
        gsrn_like_from_digital_link(url, "/8018/", prefix_len).map(
            |(company_prefix, service_ref)| Self {
                company_prefix,
                service_ref,
            },
        )
    }

    /// Converts the GSRN to a Digital Link URL.
    #[must_use]
    pub fn to_digital_link(&self, base_url: &str) -> String {
        gsrn_like_to_digital_link(base_url, "8018", self.company_prefix, self.service_ref)
    }
}

/// Global Service Relation Number — Provider (GSRNP).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gsrnp<'a> {
    /// GS1 Company Prefix
    pub company_prefix: &'a str,
    /// Service reference
    pub service_ref: &'a str,
}

impl<'a> Gsrnp<'a> {
    /// Parses a GSRNP from a URN format.
    /// E.g. `urn:epc:id:gsrnp:CompanyPrefix.ServiceRef`
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if parsing fails.
    pub fn from_urn(urn: &'a str) -> Result<Self, ParseError> {
        gsrn_like_from_urn(urn, "urn:epc:id:gsrnp:").map(|(company_prefix, service_ref)| Self {
            company_prefix,
            service_ref,
        })
    }

    /// Converts the GSRNP to its URN representation.
    #[must_use]
    pub fn to_urn(&self) -> String {
        format!(
            "urn:epc:id:gsrnp:{}.{}",
            self.company_prefix, self.service_ref
        )
    }

    /// Parses a GSRNP from a GS1 Digital Link path structure.
    /// E.g. `/8017/GSRNP`
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if format is invalid.
    pub fn from_digital_link(url: &'a str, prefix_len: usize) -> Result<Self, ParseError> {
        gsrn_like_from_digital_link(url, "/8017/", prefix_len).map(
            |(company_prefix, service_ref)| Self {
                company_prefix,
                service_ref,
            },
        )
    }

    /// Converts the GSRNP to a Digital Link URL.
    #[must_use]
    pub fn to_digital_link(&self, base_url: &str) -> String {
        gsrn_like_to_digital_link(base_url, "8017", self.company_prefix, self.service_ref)
    }
}

fn gsrn_like_from_urn<'a>(urn: &'a str, prefix: &str) -> Result<(&'a str, &'a str), ParseError> {
    let body = urn.strip_prefix(prefix).ok_or(ParseError::InvalidPrefix)?;
    let mut parts = body.split('.');
    let company_prefix = parts.next().ok_or(ParseError::MissingField)?;
    let service_ref = parts.next().ok_or(ParseError::MissingField)?;
    if parts.next().is_some() {
        return Err(ParseError::ExtraFields);
    }
    // Service reference may be empty (12-digit company prefixes).
    if !is_ascii_digits(company_prefix) || !service_ref.bytes().all(|b| b.is_ascii_digit()) {
        return Err(ParseError::InvalidFormat);
    }
    Ok((company_prefix, service_ref))
}

fn gsrn_like_from_digital_link<'a>(
    url: &'a str,
    ai: &str,
    prefix_len: usize,
) -> Result<(&'a str, &'a str), ParseError> {
    let idx = url.find(ai).ok_or(ParseError::InvalidFormat)?;
    let path = &url[idx + ai.len()..];
    let gsrn = path.split('/').next().ok_or(ParseError::MissingField)?;

    if gsrn.len() != 18 || !is_ascii_digits(gsrn) {
        return Err(ParseError::InvalidFormat);
    }
    if prefix_len >= 17 {
        return Err(ParseError::InvalidFormat);
    }
    Ok((&gsrn[0..prefix_len], &gsrn[prefix_len..17]))
}

fn gsrn_like_to_digital_link(
    base_url: &str,
    ai: &str,
    company_prefix: &str,
    service_ref: &str,
) -> String {
    let without_check = format!("{company_prefix}{service_ref}");
    let check_digit = calculate_check_digit(&without_check);
    format!(
        "{}/{ai}/{without_check}{check_digit}",
        base_url.trim_end_matches('/')
    )
}

/// Serialized Global Coupon Number (SGCN).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sgcn<'a> {
    /// GS1 Company Prefix
    pub company_prefix: &'a str,
    /// Coupon reference
    pub coupon_ref: &'a str,
    /// Serial component (numeric per GS1)
    pub serial_number: &'a str,
}

impl<'a> Sgcn<'a> {
    /// Parses an SGCN from a URN format.
    /// E.g. `urn:epc:id:sgcn:CompanyPrefix.CouponRef.Serial`
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if parsing fails.
    pub fn from_urn(urn: &'a str) -> Result<Self, ParseError> {
        let body = urn
            .strip_prefix("urn:epc:id:sgcn:")
            .ok_or(ParseError::InvalidPrefix)?;
        let mut parts = body.split('.');
        let company_prefix = parts.next().ok_or(ParseError::MissingField)?;
        let coupon_ref = parts.next().ok_or(ParseError::MissingField)?;
        let serial_number = parts.next().ok_or(ParseError::MissingField)?;
        if parts.next().is_some() {
            return Err(ParseError::ExtraFields);
        }

        // Coupon reference may be empty (12-digit company prefixes); the SGCN
        // serial component is numeric per GS1.
        if !is_ascii_digits(company_prefix)
            || !coupon_ref.bytes().all(|b| b.is_ascii_digit())
            || !is_ascii_digits(serial_number)
        {
            return Err(ParseError::InvalidFormat);
        }

        Ok(Self {
            company_prefix,
            coupon_ref,
            serial_number,
        })
    }

    /// Converts the SGCN to its URN representation.
    #[must_use]
    pub fn to_urn(&self) -> String {
        format!(
            "urn:epc:id:sgcn:{}.{}.{}",
            self.company_prefix, self.coupon_ref, self.serial_number
        )
    }

    /// Parses an SGCN from a GS1 Digital Link path structure.
    /// E.g. `/255/GCN` (13-digit GCN followed directly by the serial)
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if format is invalid.
    pub fn from_digital_link(url: &'a str, prefix_len: usize) -> Result<Self, ParseError> {
        let idx = url.find("/255/").ok_or(ParseError::InvalidFormat)?;
        let path = &url[idx + 5..];
        let segment = path.split('/').next().ok_or(ParseError::MissingField)?;

        let Some((gcn_id, serial_number)) = segment.split_at_checked(13) else {
            return Err(ParseError::InvalidFormat);
        };
        if !is_ascii_digits(gcn_id) || !is_ascii_digits(serial_number) || prefix_len >= 12 {
            return Err(ParseError::InvalidFormat);
        }

        Ok(Self {
            company_prefix: &gcn_id[0..prefix_len],
            coupon_ref: &gcn_id[prefix_len..12],
            serial_number,
        })
    }

    /// Converts the SGCN to a Digital Link URL.
    #[must_use]
    pub fn to_digital_link(&self, base_url: &str) -> String {
        let gcn_without_check = format!("{}{}", self.company_prefix, self.coupon_ref);
        let check_digit = calculate_check_digit(&gcn_without_check);
        format!(
            "{}/255/{}{}{}",
            base_url.trim_end_matches('/'),
            gcn_without_check,
            check_digit,
            self.serial_number
        )
    }
}

/// Global Identification Number for Consignment (GINC).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ginc<'a> {
    /// GS1 Company Prefix
    pub company_prefix: &'a str,
    /// Consignment reference (free-form per GS1)
    pub consignment_ref: &'a str,
}

impl<'a> Ginc<'a> {
    /// Parses a GINC from a URN format.
    /// E.g. `urn:epc:id:ginc:CompanyPrefix.ConsignmentRef`
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if parsing fails.
    pub fn from_urn(urn: &'a str) -> Result<Self, ParseError> {
        let body = urn
            .strip_prefix("urn:epc:id:ginc:")
            .ok_or(ParseError::InvalidPrefix)?;
        let mut parts = body.split('.');
        let company_prefix = parts.next().ok_or(ParseError::MissingField)?;
        let consignment_ref = parts.next().ok_or(ParseError::MissingField)?;
        if parts.next().is_some() {
            return Err(ParseError::ExtraFields);
        }

        if !is_ascii_digits(company_prefix) || consignment_ref.is_empty() {
            return Err(ParseError::InvalidFormat);
        }

        Ok(Self {
            company_prefix,
            consignment_ref,
        })
    }

    /// Converts the GINC to its URN representation.
    #[must_use]
    pub fn to_urn(&self) -> String {
        format!(
            "urn:epc:id:ginc:{}.{}",
            self.company_prefix, self.consignment_ref
        )
    }

    /// Parses a GINC from a GS1 Digital Link path structure.
    /// E.g. `/401/GINC` (no check digit; the company prefix length splits
    /// the identifier)
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if format is invalid.
    pub fn from_digital_link(url: &'a str, prefix_len: usize) -> Result<Self, ParseError> {
        let idx = url.find("/401/").ok_or(ParseError::InvalidFormat)?;
        let path = &url[idx + 5..];
        let ginc = path.split('/').next().ok_or(ParseError::MissingField)?;

        let Some((company_prefix, consignment_ref)) = ginc.split_at_checked(prefix_len) else {
            return Err(ParseError::InvalidFormat);
        };
        if !is_ascii_digits(company_prefix) || consignment_ref.is_empty() {
            return Err(ParseError::InvalidFormat);
        }

        Ok(Self {
            company_prefix,
            consignment_ref,
        })
    }

    /// Converts the GINC to a Digital Link URL.
    #[must_use]
    pub fn to_digital_link(&self, base_url: &str) -> String {
        format!(
            "{}/401/{}{}",
            base_url.trim_end_matches('/'),
            self.company_prefix,
            self.consignment_ref
        )
    }
}

/// Global Shipment Identification Number (GSIN).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gsin<'a> {
    /// GS1 Company Prefix
    pub company_prefix: &'a str,
    /// Shipper reference
    pub shipper_ref: &'a str,
}

impl<'a> Gsin<'a> {
    /// Parses a GSIN from a URN format.
    /// E.g. `urn:epc:id:gsin:CompanyPrefix.ShipperRef`
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if parsing fails.
    pub fn from_urn(urn: &'a str) -> Result<Self, ParseError> {
        let body = urn
            .strip_prefix("urn:epc:id:gsin:")
            .ok_or(ParseError::InvalidPrefix)?;
        let mut parts = body.split('.');
        let company_prefix = parts.next().ok_or(ParseError::MissingField)?;
        let shipper_ref = parts.next().ok_or(ParseError::MissingField)?;
        if parts.next().is_some() {
            return Err(ParseError::ExtraFields);
        }

        // Shipper reference may be empty (12-digit company prefixes).
        if !is_ascii_digits(company_prefix) || !shipper_ref.bytes().all(|b| b.is_ascii_digit()) {
            return Err(ParseError::InvalidFormat);
        }

        Ok(Self {
            company_prefix,
            shipper_ref,
        })
    }

    /// Converts the GSIN to its URN representation.
    #[must_use]
    pub fn to_urn(&self) -> String {
        format!(
            "urn:epc:id:gsin:{}.{}",
            self.company_prefix, self.shipper_ref
        )
    }

    /// Parses a GSIN from a GS1 Digital Link path structure.
    /// E.g. `/402/GSIN`
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if format is invalid.
    pub fn from_digital_link(url: &'a str, prefix_len: usize) -> Result<Self, ParseError> {
        let idx = url.find("/402/").ok_or(ParseError::InvalidFormat)?;
        let path = &url[idx + 5..];
        let gsin = path.split('/').next().ok_or(ParseError::MissingField)?;

        if gsin.len() != 17 || !is_ascii_digits(gsin) {
            return Err(ParseError::InvalidFormat);
        }
        if prefix_len >= 16 {
            return Err(ParseError::InvalidFormat);
        }

        Ok(Self {
            company_prefix: &gsin[0..prefix_len],
            shipper_ref: &gsin[prefix_len..16],
        })
    }

    /// Converts the GSIN to a Digital Link URL.
    #[must_use]
    pub fn to_digital_link(&self, base_url: &str) -> String {
        let gsin_without_check = format!("{}{}", self.company_prefix, self.shipper_ref);
        let check_digit = calculate_check_digit(&gsin_without_check);
        format!(
            "{}/402/{}{}",
            base_url.trim_end_matches('/'),
            gsin_without_check,
            check_digit
        )
    }
}

/// Individual Trade Item Piece (ITIP).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Itip<'a> {
    /// GS1 Company Prefix
    pub company_prefix: &'a str,
    /// Indicator digit
    pub indicator: &'a str,
    /// Item reference
    pub item_ref: &'a str,
    /// Piece number (2 digits)
    pub piece: &'a str,
    /// Total number of pieces (2 digits)
    pub total: &'a str,
    /// Serial number
    pub serial_number: &'a str,
}

impl<'a> Itip<'a> {
    /// Parses an ITIP from a URN format.
    /// E.g. `urn:epc:id:itip:CompanyPrefix.IndicatorAndItemRef.Piece.Total.Serial`
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if parsing fails.
    pub fn from_urn(urn: &'a str) -> Result<Self, ParseError> {
        let body = urn
            .strip_prefix("urn:epc:id:itip:")
            .ok_or(ParseError::InvalidPrefix)?;
        let mut parts = body.split('.');
        let company_prefix = parts.next().ok_or(ParseError::MissingField)?;
        let indicator_and_item_ref = parts.next().ok_or(ParseError::MissingField)?;
        let piece = parts.next().ok_or(ParseError::MissingField)?;
        let total = parts.next().ok_or(ParseError::MissingField)?;
        let serial_number = parts.next().ok_or(ParseError::MissingField)?;
        if parts.next().is_some() {
            return Err(ParseError::ExtraFields);
        }

        if !is_ascii_digits(company_prefix)
            || !is_ascii_digits(indicator_and_item_ref)
            || piece.len() != 2
            || !is_ascii_digits(piece)
            || total.len() != 2
            || !is_ascii_digits(total)
        {
            return Err(ParseError::InvalidFormat);
        }

        Ok(Self {
            company_prefix,
            indicator: &indicator_and_item_ref[0..1],
            item_ref: &indicator_and_item_ref[1..],
            piece,
            total,
            serial_number,
        })
    }

    /// Converts the ITIP to its URN representation.
    #[must_use]
    pub fn to_urn(&self) -> String {
        format!(
            "urn:epc:id:itip:{}.{}{}.{}.{}.{}",
            self.company_prefix,
            self.indicator,
            self.item_ref,
            self.piece,
            self.total,
            self.serial_number
        )
    }

    /// Parses an ITIP from a GS1 Digital Link path structure.
    /// E.g. `/8006/ITIP/21/SERIAL` (14-digit GTIN + 2-digit piece + 2-digit total)
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if format is invalid.
    pub fn from_digital_link(url: &'a str, prefix_len: usize) -> Result<Self, ParseError> {
        let idx = url.find("/8006/").ok_or(ParseError::InvalidFormat)?;
        let path = &url[idx + 6..];
        let mut parts = path.split('/');
        let itip = parts.next().ok_or(ParseError::MissingField)?;
        let ai = parts.next().ok_or(ParseError::MissingField)?;
        if ai != "21" {
            return Err(ParseError::InvalidFormat);
        }
        let serial_number = parts.next().ok_or(ParseError::MissingField)?;

        if itip.len() != 18 || !is_ascii_digits(itip) {
            return Err(ParseError::InvalidFormat);
        }
        if prefix_len >= 13 {
            return Err(ParseError::InvalidFormat);
        }

        Ok(Self {
            indicator: &itip[0..1],
            company_prefix: &itip[1..=prefix_len],
            item_ref: &itip[1 + prefix_len..13],
            piece: &itip[14..16],
            total: &itip[16..18],
            serial_number,
        })
    }

    /// Converts the ITIP to a Digital Link URL.
    #[must_use]
    pub fn to_digital_link(&self, base_url: &str) -> String {
        let gtin_without_check =
            format!("{}{}{}", self.indicator, self.company_prefix, self.item_ref);
        let check_digit = calculate_check_digit(&gtin_without_check);
        format!(
            "{}/8006/{}{}{}{}/21/{}",
            base_url.trim_end_matches('/'),
            gtin_without_check,
            check_digit,
            self.piece,
            self.total,
            self.serial_number
        )
    }
}

/// Unit Pack Identifier (UPUI).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Upui<'a> {
    /// GS1 Company Prefix
    pub company_prefix: &'a str,
    /// Indicator digit
    pub indicator: &'a str,
    /// Item reference
    pub item_ref: &'a str,
    /// Third-party unit pack serial
    pub serial_number: &'a str,
}

impl<'a> Upui<'a> {
    /// Parses a UPUI from a URN format.
    /// E.g. `urn:epc:id:upui:CompanyPrefix.IndicatorAndItemRef.Serial`
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if parsing fails.
    pub fn from_urn(urn: &'a str) -> Result<Self, ParseError> {
        let body = urn
            .strip_prefix("urn:epc:id:upui:")
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

        Ok(Self {
            company_prefix,
            indicator: &indicator_and_item_ref[0..1],
            item_ref: &indicator_and_item_ref[1..],
            serial_number,
        })
    }

    /// Converts the UPUI to its URN representation.
    #[must_use]
    pub fn to_urn(&self) -> String {
        format!(
            "urn:epc:id:upui:{}.{}{}.{}",
            self.company_prefix, self.indicator, self.item_ref, self.serial_number
        )
    }

    /// Parses a UPUI from a GS1 Digital Link path structure.
    /// E.g. `/01/GTIN/235/SERIAL`
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if format is invalid.
    pub fn from_digital_link(url: &'a str, prefix_len: usize) -> Result<Self, ParseError> {
        let (gtin, serial_number) = gtin_with_qualifier(url, "235")?;
        if prefix_len >= 13 {
            return Err(ParseError::InvalidFormat);
        }
        Ok(Self {
            indicator: &gtin[0..1],
            company_prefix: &gtin[1..=prefix_len],
            item_ref: &gtin[1 + prefix_len..13],
            serial_number,
        })
    }

    /// Converts the UPUI to a Digital Link URL.
    #[must_use]
    pub fn to_digital_link(&self, base_url: &str) -> String {
        let gtin_without_check =
            format!("{}{}{}", self.indicator, self.company_prefix, self.item_ref);
        let check_digit = calculate_check_digit(&gtin_without_check);
        format!(
            "{}/01/{}{}/235/{}",
            base_url.trim_end_matches('/'),
            gtin_without_check,
            check_digit,
            self.serial_number
        )
    }
}

/// Component / Part Identifier (CPI).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cpi<'a> {
    /// GS1 Company Prefix
    pub company_prefix: &'a str,
    /// Component/part reference
    pub part_ref: &'a str,
    /// Serial component (numeric per GS1)
    pub serial_number: &'a str,
}

impl<'a> Cpi<'a> {
    /// Parses a CPI from a URN format.
    /// E.g. `urn:epc:id:cpi:CompanyPrefix.PartRef.Serial`
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if parsing fails.
    pub fn from_urn(urn: &'a str) -> Result<Self, ParseError> {
        let body = urn
            .strip_prefix("urn:epc:id:cpi:")
            .ok_or(ParseError::InvalidPrefix)?;
        let mut parts = body.split('.');
        let company_prefix = parts.next().ok_or(ParseError::MissingField)?;
        let part_ref = parts.next().ok_or(ParseError::MissingField)?;
        let serial_number = parts.next().ok_or(ParseError::MissingField)?;
        if parts.next().is_some() {
            return Err(ParseError::ExtraFields);
        }

        // The part reference is upper-alphanumeric per GS1; the CPI serial
        // component is numeric.
        if !is_ascii_digits(company_prefix)
            || part_ref.is_empty()
            || !is_ascii_digits(serial_number)
        {
            return Err(ParseError::InvalidFormat);
        }

        Ok(Self {
            company_prefix,
            part_ref,
            serial_number,
        })
    }

    /// Converts the CPI to its URN representation.
    #[must_use]
    pub fn to_urn(&self) -> String {
        format!(
            "urn:epc:id:cpi:{}.{}.{}",
            self.company_prefix, self.part_ref, self.serial_number
        )
    }

    /// Parses a CPI from a GS1 Digital Link path structure.
    /// E.g. `/8010/CPID/8011/SERIAL`
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if format is invalid.
    pub fn from_digital_link(url: &'a str, prefix_len: usize) -> Result<Self, ParseError> {
        let idx = url.find("/8010/").ok_or(ParseError::InvalidFormat)?;
        let path = &url[idx + 6..];
        let mut parts = path.split('/');
        let cpid = parts.next().ok_or(ParseError::MissingField)?;
        let ai = parts.next().ok_or(ParseError::MissingField)?;
        if ai != "8011" {
            return Err(ParseError::InvalidFormat);
        }
        let serial_number = parts.next().ok_or(ParseError::MissingField)?;

        let Some((company_prefix, part_ref)) = cpid.split_at_checked(prefix_len) else {
            return Err(ParseError::InvalidFormat);
        };
        if !is_ascii_digits(company_prefix)
            || part_ref.is_empty()
            || !is_ascii_digits(serial_number)
        {
            return Err(ParseError::InvalidFormat);
        }

        Ok(Self {
            company_prefix,
            part_ref,
            serial_number,
        })
    }

    /// Converts the CPI to a Digital Link URL.
    #[must_use]
    pub fn to_digital_link(&self, base_url: &str) -> String {
        format!(
            "{}/8010/{}{}/8011/{}",
            base_url.trim_end_matches('/'),
            self.company_prefix,
            self.part_ref,
            self.serial_number
        )
    }
}

/// Lot-level GTIN (LGTIN) — class-level identification of a batch/lot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lgtin<'a> {
    /// GS1 Company Prefix
    pub company_prefix: &'a str,
    /// Indicator digit
    pub indicator: &'a str,
    /// Item reference
    pub item_ref: &'a str,
    /// Lot / batch number
    pub lot: &'a str,
}

impl<'a> Lgtin<'a> {
    /// Parses an LGTIN from a URN format.
    /// E.g. `urn:epc:class:lgtin:CompanyPrefix.IndicatorAndItemRef.Lot`
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if parsing fails.
    pub fn from_urn(urn: &'a str) -> Result<Self, ParseError> {
        let body = urn
            .strip_prefix("urn:epc:class:lgtin:")
            .ok_or(ParseError::InvalidPrefix)?;
        let mut parts = body.split('.');
        let company_prefix = parts.next().ok_or(ParseError::MissingField)?;
        let indicator_and_item_ref = parts.next().ok_or(ParseError::MissingField)?;
        let lot = parts.next().ok_or(ParseError::MissingField)?;
        if parts.next().is_some() {
            return Err(ParseError::ExtraFields);
        }

        if !is_ascii_digits(company_prefix) || !is_ascii_digits(indicator_and_item_ref) {
            return Err(ParseError::InvalidFormat);
        }

        Ok(Self {
            company_prefix,
            indicator: &indicator_and_item_ref[0..1],
            item_ref: &indicator_and_item_ref[1..],
            lot,
        })
    }

    /// Converts the LGTIN to its URN representation.
    #[must_use]
    pub fn to_urn(&self) -> String {
        format!(
            "urn:epc:class:lgtin:{}.{}{}.{}",
            self.company_prefix, self.indicator, self.item_ref, self.lot
        )
    }

    /// Parses an LGTIN from a GS1 Digital Link path structure.
    /// E.g. `/01/GTIN/10/LOT`
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if format is invalid.
    pub fn from_digital_link(url: &'a str, prefix_len: usize) -> Result<Self, ParseError> {
        let (gtin, lot) = gtin_with_qualifier(url, "10")?;
        if prefix_len >= 13 {
            return Err(ParseError::InvalidFormat);
        }
        Ok(Self {
            indicator: &gtin[0..1],
            company_prefix: &gtin[1..=prefix_len],
            item_ref: &gtin[1 + prefix_len..13],
            lot,
        })
    }

    /// Converts the LGTIN to a Digital Link URL.
    #[must_use]
    pub fn to_digital_link(&self, base_url: &str) -> String {
        let gtin_without_check =
            format!("{}{}{}", self.indicator, self.company_prefix, self.item_ref);
        let check_digit = calculate_check_digit(&gtin_without_check);
        format!(
            "{}/01/{}{}/10/{}",
            base_url.trim_end_matches('/'),
            gtin_without_check,
            check_digit,
            self.lot
        )
    }
}

/// Extracts a validated 14-digit GTIN and its qualifier value (e.g. `/10/`
/// lot or `/235/` unit-pack serial) from a Digital Link path.
fn gtin_with_qualifier<'a>(
    url: &'a str,
    qualifier: &str,
) -> Result<(&'a str, &'a str), ParseError> {
    let idx = url.find("/01/").ok_or(ParseError::InvalidFormat)?;
    let path = &url[idx + 4..];
    let mut parts = path.split('/');
    let gtin = parts.next().ok_or(ParseError::MissingField)?;
    let ai = parts.next().ok_or(ParseError::MissingField)?;
    if ai != qualifier {
        return Err(ParseError::InvalidFormat);
    }
    let value = parts.next().ok_or(ParseError::MissingField)?;

    if gtin.len() != 14 || !is_ascii_digits(gtin) {
        return Err(ParseError::InvalidFormat);
    }
    Ok((gtin, value))
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
    if urn.starts_with("urn:epc:class:lgtin:") {
        return Lgtin::from_urn(urn)
            .map(|k| k.to_digital_link(base_url))
            .map_err(|e| format!("Failed to parse LGTIN: {e:?}"));
    }
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
        "pgln" => Pgln::from_urn(urn)
            .map(|k| k.to_digital_link(base_url))
            .map_err(|e| format!("Failed to parse PGLN: {e:?}")),
        "gdti" => Gdti::from_urn(urn)
            .map(|k| k.to_digital_link(base_url))
            .map_err(|e| format!("Failed to parse GDTI: {e:?}")),
        "gsrn" => Gsrn::from_urn(urn)
            .map(|k| k.to_digital_link(base_url))
            .map_err(|e| format!("Failed to parse GSRN: {e:?}")),
        "gsrnp" => Gsrnp::from_urn(urn)
            .map(|k| k.to_digital_link(base_url))
            .map_err(|e| format!("Failed to parse GSRNP: {e:?}")),
        "sgcn" => Sgcn::from_urn(urn)
            .map(|k| k.to_digital_link(base_url))
            .map_err(|e| format!("Failed to parse SGCN: {e:?}")),
        "ginc" => Ginc::from_urn(urn)
            .map(|k| k.to_digital_link(base_url))
            .map_err(|e| format!("Failed to parse GINC: {e:?}")),
        "gsin" => Gsin::from_urn(urn)
            .map(|k| k.to_digital_link(base_url))
            .map_err(|e| format!("Failed to parse GSIN: {e:?}")),
        "itip" => Itip::from_urn(urn)
            .map(|k| k.to_digital_link(base_url))
            .map_err(|e| format!("Failed to parse ITIP: {e:?}")),
        "upui" => Upui::from_urn(urn)
            .map(|k| k.to_digital_link(base_url))
            .map_err(|e| format!("Failed to parse UPUI: {e:?}")),
        "cpi" => Cpi::from_urn(urn)
            .map(|k| k.to_digital_link(base_url))
            .map_err(|e| format!("Failed to parse CPI: {e:?}")),
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
    if dl.contains("/01/") && dl.contains("/235/") {
        return Upui::from_digital_link(dl, prefix_len)
            .map(|k| k.to_urn())
            .map_err(|e| format!("Failed to parse UPUI DL: {e:?}"));
    }
    if dl.contains("/01/") && dl.contains("/10/") {
        return Lgtin::from_digital_link(dl, prefix_len)
            .map(|k| k.to_urn())
            .map_err(|e| format!("Failed to parse LGTIN DL: {e:?}"));
    }
    for (ai, translate) in [
        (
            "/8006/",
            (|d, l| Itip::from_digital_link(d, l).map(|k| k.to_urn()))
                as fn(&str, usize) -> Result<String, ParseError>,
        ),
        ("/8010/", |d, l| {
            Cpi::from_digital_link(d, l).map(|k| k.to_urn())
        }),
        ("/8017/", |d, l| {
            Gsrnp::from_digital_link(d, l).map(|k| k.to_urn())
        }),
        ("/8018/", |d, l| {
            Gsrn::from_digital_link(d, l).map(|k| k.to_urn())
        }),
        ("/253/", |d, l| {
            Gdti::from_digital_link(d, l).map(|k| k.to_urn())
        }),
        ("/255/", |d, l| {
            Sgcn::from_digital_link(d, l).map(|k| k.to_urn())
        }),
        ("/401/", |d, l| {
            Ginc::from_digital_link(d, l).map(|k| k.to_urn())
        }),
        ("/402/", |d, l| {
            Gsin::from_digital_link(d, l).map(|k| k.to_urn())
        }),
        ("/417/", |d, l| {
            Pgln::from_digital_link(d, l).map(|k| k.to_urn())
        }),
    ] {
        if dl.contains(ai) {
            return translate(dl, prefix_len)
                .map_err(|e| format!("Failed to parse {ai} DL: {e:?}"));
        }
    }
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
