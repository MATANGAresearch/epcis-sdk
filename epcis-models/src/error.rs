//! Error types for the EPCIS models crate.

use thiserror::Error;

/// Enumeration of all error types in the EPCIS model crate.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum EpcisModelError {
    /// Invalid Electronic Product Code (EPC) URN or Digital Link.
    #[error("invalid EPC: {0}")]
    InvalidEpc(String),

    /// Invalid GS1 URI.
    #[error("invalid URI: {0}")]
    InvalidUri(String),
}
