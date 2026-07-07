//! Core shared types for the EPCIS 2.0 SDK.

#![deny(missing_docs)]
#![deny(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::borrow::Cow;
use crate::error::EpcisModelError;

/// Newtype representing an Electronic Product Code (EPC) URN.
///
/// # Examples
///
/// ```
/// use epcis_models::Epc;
///
/// let epc = Epc::try_from("urn:epc:id:sgtin:4012345.098765.12345").unwrap();
/// assert_eq!(epc.to_string(), "urn:epc:id:sgtin:4012345.098765.12345");
///
/// // Invalid schemes will fail validation
/// let invalid = Epc::try_from("invalid-scheme");
/// assert!(invalid.is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Epc(pub Cow<'static, str>);

impl From<String> for Epc {
    fn from(s: String) -> Self {
        Epc(Cow::Owned(s))
    }
}

impl From<Cow<'static, str>> for Epc {
    fn from(s: Cow<'static, str>) -> Self {
        Epc(s)
    }
}

impl TryFrom<&str> for Epc {
    type Error = EpcisModelError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        if s.is_empty() {
            return Err(EpcisModelError::InvalidEpc("EPC URN cannot be empty".to_string()));
        }
        if !s.starts_with("urn:epc:") && !s.starts_with("http://") && !s.starts_with("https://") {
            return Err(EpcisModelError::InvalidEpc(format!("EPC URN must start with URN or HTTP scheme: {s}")));
        }
        Ok(Epc(Cow::Owned(s.to_string())))
    }
}

impl std::fmt::Display for Epc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The Action component of an EPCIS event, specifying the status of the objects
/// identified in the event.
///
/// # Examples
///
/// ```
/// use epcis_models::Action;
///
/// let action = Action::Observe;
/// assert_eq!(action, Action::Observe);
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Action {
    /// Add action
    Add,
    /// Observe action
    Observe,
    /// Delete action
    Delete,
}

impl<'de> Deserialize<'de> for Action {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = crate::document::deserialize_string_or_map_text(deserializer)?;
        match s.as_str() {
            "ADD" => Ok(Action::Add),
            "OBSERVE" => Ok(Action::Observe),
            "DELETE" => Ok(Action::Delete),
            other => Err(serde::de::Error::custom(format!(
                "invalid action: {}, expected ADD, OBSERVE, or DELETE",
                other
            ))),
        }
    }
}

/// Type-safe identifier wrapper for ReadPoint.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ReadPointId(pub Cow<'static, str>);

impl From<&'static str> for ReadPointId {
    fn from(s: &'static str) -> Self {
        ReadPointId(Cow::Borrowed(s))
    }
}

impl From<String> for ReadPointId {
    fn from(s: String) -> Self {
        ReadPointId(Cow::Owned(s))
    }
}

impl std::fmt::Display for ReadPointId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The physical location where the event took place.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ReadPoint {
    /// Unique identifier of the read point
    pub id: ReadPointId,
}

impl From<&'static str> for ReadPoint {
    fn from(id: &'static str) -> Self {
        ReadPoint { id: ReadPointId::from(id) }
    }
}

impl From<String> for ReadPoint {
    fn from(id: String) -> Self {
        ReadPoint { id: ReadPointId::from(id) }
    }
}

/// Type-safe identifier wrapper for BizLocation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BizLocationId(pub Cow<'static, str>);

impl From<&'static str> for BizLocationId {
    fn from(s: &'static str) -> Self {
        BizLocationId(Cow::Borrowed(s))
    }
}

impl From<String> for BizLocationId {
    fn from(s: String) -> Self {
        BizLocationId(Cow::Owned(s))
    }
}

impl std::fmt::Display for BizLocationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The business location where the objects are expected to be after the event.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BizLocation {
    /// Unique identifier of the business location
    pub id: BizLocationId,
}

impl From<&'static str> for BizLocation {
    fn from(id: &'static str) -> Self {
        BizLocation { id: BizLocationId::from(id) }
    }
}

impl From<String> for BizLocation {
    fn from(id: String) -> Self {
        BizLocation { id: BizLocationId::from(id) }
    }
}

/// A business transaction associated with the event.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BizTransaction {
    /// Type of the business transaction (e.g. PO)
    #[serde(rename = "type")]
    pub r#type: String,
    /// Transaction ID value (e.g. URI)
    pub biz_transaction: String,
}

/// Source of the objects in the event (e.g. shipping location, possessing party).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    /// Type of the source (e.g. possessing party)
    #[serde(rename = "type")]
    pub r#type: String,
    /// Source value (e.g. SGLN URI)
    pub source: String,
}

/// Destination of the objects in the event (e.g. receiving location, owning party).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Destination {
    /// Type of the destination (e.g. owning party)
    #[serde(rename = "type")]
    pub r#type: String,
    /// Destination value (e.g. SGLN URI)
    pub destination: String,
}

/// An element representing a quantity of EPCs of a single class.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuantityElement {
    /// The EPC class (e.g. GTIN class)
    pub epc_class: String,
    /// Quantity of items in the class
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<f64>,
    /// Unit of measure (e.g. KGM)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uom: Option<String>,
}

/// Metadata about a sensor device or sensor data source.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SensorMetadata {
    /// Time when the sensor metadata was generated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<DateTime<Utc>>,
    /// Device identifier
    #[serde(rename = "deviceID", skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
    /// URI pointing to device metadata details
    #[serde(rename = "deviceMetadataURI", skip_serializing_if = "Option::is_none")]
    pub device_metadata_uri: Option<String>,
    /// URI pointing to raw sensor data
    #[serde(rename = "rawDataURI", skip_serializing_if = "Option::is_none")]
    pub raw_data_uri: Option<String>,
    /// URI pointing to parsed data content
    #[serde(rename = "dataContentURI", skip_serializing_if = "Option::is_none")]
    pub data_content_uri: Option<String>,
    /// URI pointing to business logic rules applied to the sensor
    #[serde(rename = "bizRulesURI", skip_serializing_if = "Option::is_none")]
    pub biz_rules_uri: Option<String>,
}

/// A specific sensor report (e.g., temperature reading).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SensorReport {
    /// Type of measurement (e.g., temperature, relative humidity)
    #[serde(rename = "type")]
    pub r#type: String,
    /// Numerical value of the sensor reading
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
    /// Unit of measure (e.g. CEL)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uom: Option<String>,
    /// Processor component responsible for the sensor reading
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sensor_processor: Option<String>,
    /// Time when the reading occurred
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<DateTime<Utc>>,
    /// Microsecond offset from event time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub microsecond_offset: Option<i32>,
    /// Chemical substance measured (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chemical_substance: Option<String>,
    /// Data value in string/URI format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_value: Option<String>,
    /// String representation value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub string_value: Option<String>,
    /// Boolean representation value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boolean_value: Option<bool>,
    /// Hex-binary value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hex_binary_value: Option<String>,
    /// URI value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri_value: Option<String>,
}

/// A sensor element grouping sensor metadata and reports.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SensorElement {
    /// Sensor device metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sensor_metadata: Option<SensorMetadata>,
    /// List of sensor reports
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sensor_report: Option<Vec<SensorReport>>,
}

/// Declaration of error correction context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorDeclaration {
    /// The timestamp when the error was declared
    pub declaration_time: DateTime<Utc>,
    /// The reason for error declaration (e.g. incorrect data)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// List of event IDs that correct this event
    #[serde(rename = "correctiveEventIDs", skip_serializing_if = "Option::is_none")]
    pub corrective_event_ids: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cbv::BizStep;
    use std::str::FromStr;

    #[test]
    fn test_epc_validation() {
        // Valid URN
        let epc = Epc::try_from("urn:epc:id:sgtin:0614141.107346.2023");
        assert!(epc.is_ok());
        assert_eq!(epc.unwrap().to_string(), "urn:epc:id:sgtin:0614141.107346.2023");

        // Valid URI
        let epc_uri = Epc::try_from("https://id.gs1.org/01/04012345987652/21/12345");
        assert!(epc_uri.is_ok());

        // Error: Empty
        let empty = Epc::try_from("");
        assert!(empty.is_err());
        assert!(matches!(empty.unwrap_err(), EpcisModelError::InvalidEpc(_)));

        // Error: Invalid scheme
        let bad_epc = Epc::try_from("bad:scheme:123");
        assert!(bad_epc.is_err());
    }

    #[test]
    fn test_action_deserialization_errors() {
        // Valid deserializations
        let observe: Action = serde_json::from_str("\"OBSERVE\"").unwrap();
        assert_eq!(observe, Action::Observe);

        // Invalid deserialization
        let err: Result<Action, _> = serde_json::from_str("\"INVALID\"");
        assert!(err.is_err());
    }

    #[test]
    fn test_biz_step_custom_and_standard() {
        // Standard CBV step parsing
        let step = BizStep::from_str("urn:epcglobal:cbv:bizstep:receiving").unwrap();
        assert_eq!(step.as_str(), "urn:epcglobal:cbv:bizstep:receiving");

        // Custom step parsing
        let custom = BizStep::from_str("https://example.com/custom-bizstep").unwrap();
        assert_eq!(custom.as_str(), "https://example.com/custom-bizstep");
    }
}
