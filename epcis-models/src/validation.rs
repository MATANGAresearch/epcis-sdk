//! Validation helpers for EPCIS 2.0 user extensions and attributes.

use thiserror::Error;

/// Errors that can occur during custom extension validation.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ValidationError {
    /// An extension key is not qualified (must contain a namespace prefix with a colon, or be a full URI)
    #[error(
        "Extension key '{key}' is not qualified (must be a valid URI or contain a namespace prefix like prefix:name)"
    )]
    UnqualifiedKey {
        /// The unqualified key
        key: String,
    },
    /// An extension key is in a reserved/standard namespace but is not recognized
    #[error(
        "Extension key '{key}' uses a standard namespace prefix but is not recognized as a valid extension"
    )]
    InvalidStandardPrefix {
        /// The key using standard namespace prefix
        key: String,
    },
}

/// Validates a map of user-defined custom extensions.
///
/// Under EPCIS rules, custom extension elements must be fully qualified.
/// They must either be absolute URIs (e.g., `http://namespace.org/vocab#property`)
/// or contain a colon `:` demarcating a namespace prefix (e.g., `prefix:property`).
/// Plain unqualified keys (e.g., `unqualifiedProperty`) are invalid.
///
/// # Errors
/// Returns `ValidationError` if any key is unqualified or invalid.
pub fn validate_extension_keys(
    extensions: &serde_json::Map<String, serde_json::Value>,
) -> Result<(), ValidationError> {
    for key in extensions.keys() {
        // Skip context declarations
        if key.starts_with('@') {
            continue;
        }

        // Standard event attributes are not valid custom extension keys
        let is_standard = matches!(
            key.as_str(),
            "type"
                | "eventTime"
                | "eventTimeZoneOffset"
                | "epcList"
                | "bizStep"
                | "disposition"
                | "readPoint"
                | "bizLocation"
                | "bizTransactionList"
                | "sourceList"
                | "destinationList"
                | "quantityList"
                | "sensorElementList"
                | "action"
                | "errorDeclaration"
                | "persistentDisposition"
        );
        if is_standard {
            return Err(ValidationError::InvalidStandardPrefix { key: key.clone() });
        }

        // Check if it's a URI or has a namespace colon
        let has_colon = key.contains(':');
        let is_uri =
            key.starts_with("http://") || key.starts_with("https://") || key.starts_with("urn:");

        if !has_colon && !is_uri {
            return Err(ValidationError::UnqualifiedKey { key: key.clone() });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_valid_extensions() {
        let mut ext = serde_json::Map::new();
        ext.insert("custom:myField".to_string(), json!("value"));
        ext.insert(
            "http://example.com/vocab#anotherField".to_string(),
            json!(123),
        );
        ext.insert("urn:uuid:12345".to_string(), json!(true));

        assert_eq!(validate_extension_keys(&ext), Ok(()));
    }

    #[test]
    fn test_unqualified_extensions() {
        let mut ext = serde_json::Map::new();
        ext.insert("unqualifiedProperty".to_string(), json!("value"));

        assert_eq!(
            validate_extension_keys(&ext),
            Err(ValidationError::UnqualifiedKey {
                key: "unqualifiedProperty".to_string()
            })
        );
    }

    #[test]
    fn test_invalid_standard_namespace() {
        let mut ext = serde_json::Map::new();
        ext.insert("eventTime".to_string(), json!("2020-03-04T11:00:30.000Z"));

        assert_eq!(
            validate_extension_keys(&ext),
            Err(ValidationError::InvalidStandardPrefix {
                key: "eventTime".to_string()
            })
        );
    }
}
