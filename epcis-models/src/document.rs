//! `EPCISDocument` schema definition.

#![deny(missing_docs)]
#![deny(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::events::EPCISEvent;

/// Represents the top-level EPCIS 2.0 JSON-LD Document.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EPCISDocument {
    /// Context property containing schema mappings for JSON-LD compliance.
    #[serde(rename = "@context")]
    pub context: serde_json::Value,
    
    /// Document identifier (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    
    /// Type identifier, always "`EPCISDocument`".
    #[serde(rename = "type")]
    pub r#type: String,
    
    /// Schema version, typically "2.0".
    pub schema_version: String,
    
    /// Creation timestamp of this document.
    pub creation_date: DateTime<Utc>,
    
    /// Header containing master data vocabulary context (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub epcis_header: Option<EPCISHeader>,
    
    /// Body containing the list of actual tracking events.
    pub epcis_body: EPCISBody,
    
    /// Extension elements.
    #[serde(flatten)]
    pub extensions: serde_json::Map<String, serde_json::Value>,
}

impl EPCISDocument {
    /// Creates a standard EPCIS 2.0 document from a list of events.
    #[must_use]
    pub fn new(event_list: Vec<EPCISEvent>) -> Self {
        Self {
            context: serde_json::json!([
                "https://ref.gs1.org/standards/epcis/2.0.0/epcis-context.jsonld"
            ]),
            id: None,
            r#type: "EPCISDocument".to_string(),
            schema_version: "2.0".to_string(),
            creation_date: Utc::now(),
            epcis_header: None,
            epcis_body: EPCISBody { event_list },
            extensions: serde_json::Map::new(),
        }
    }
}

/// Header containing metadata and master data dictionary elements.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EPCISHeader {
    /// Optional master data vocabulary list
    #[serde(skip_serializing_if = "Option::is_none")]
    pub epcis_master_data: Option<EPCISMasterData>,
}

/// List of master data elements mapped to vocabularies.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EPCISMasterData {
    /// The master data vocabulary list
    pub vocabulary_list: Vec<VocabularyElement>,
}

/// Vocabularies mapping IDs to types and attributes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VocabularyElement {
    /// Type of the vocabulary element
    #[serde(rename = "type")]
    pub r#type: String,
    /// List of attributes and mappings
    pub element_list: Vec<VocabularyElementList>,
}

/// A specific master data attribute / children list mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VocabularyElementList {
    /// Identifier of the element
    pub id: String,
    /// Attributes linked to the identifier (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Vec<VocabularyAttribute>>,
    /// List of child IDs under this element
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<String>>,
}

/// Attribute property value within vocabulary configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VocabularyAttribute {
    /// Attribute identifier
    pub id: String,
    /// Attribute value
    pub value: serde_json::Value,
}

/// Body container holding the array of EPCIS events.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EPCISBody {
    /// List of events in the body
    pub event_list: Vec<EPCISEvent>,
}
