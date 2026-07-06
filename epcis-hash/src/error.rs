//! Error type for the canonical event hash generator.

use thiserror::Error;

/// Error type for all canonical event hash operations.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum EpcisHashError {
    /// XML parsing failed.
    #[error("XML parse error: {0}")]
    XmlParse(String),

    /// The document contains no recognizable EPCIS events.
    #[error("empty document: no EPCIS events found")]
    EmptyDocument,

    /// A required field was missing from the event.
    #[error("missing required field: {field}")]
    MissingField {
        /// Name of the missing field.
        field: &'static str,
    },

    /// JSON serialization or deserialization failed.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
