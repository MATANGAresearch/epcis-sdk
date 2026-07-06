//! EPCIS 2.0 event model definitions.

#![deny(missing_docs)]
#![deny(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::types::{
    Action, BizLocation, BizTransaction, Destination, Epc,
    ErrorDeclaration, QuantityElement, ReadPoint, SensorElement, Source,
};
use crate::cbv::{BizStep, Disposition};

/// Persistent disposition settings for tracking updates in an `ObjectEvent`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersistentDisposition {
    /// List of dispositions to unset
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unset: Option<Vec<Disposition>>,
    /// List of dispositions to set
    #[serde(skip_serializing_if = "Option::is_none")]
    pub set: Option<Vec<Disposition>>,
}

/// An `ObjectEvent` captures an observation, addition, or deletion of specific items.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectEvent {
    /// Date and time when the event occurred
    pub event_time: DateTime<Utc>,
    /// Timezone offset (e.g. "+01:00")
    pub event_time_zone_offset: String,
    /// Record timestamp (optional, set by repository)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub record_time: Option<DateTime<Utc>>,
    /// Event identifier (often URN like urn:uuid)
    #[serde(rename = "eventID", skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
    /// Error declaration for corrective events
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_declaration: Option<ErrorDeclaration>,
    
    /// Action specifying lifecycle state change
    pub action: Action,
    /// List of item EPCs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub epc_list: Option<Vec<Epc>>,
    /// List of class/quantity items
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity_list: Option<Vec<QuantityElement>>,
    
    /// Business step (e.g. receiving)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub biz_step: Option<BizStep>,
    /// Item disposition (e.g. `in_transit`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disposition: Option<Disposition>,
    /// Read point identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_point: Option<ReadPoint>,
    /// Business location identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub biz_location: Option<BizLocation>,
    /// List of business transactions linked to the event
    #[serde(skip_serializing_if = "Option::is_none")]
    pub biz_transaction_list: Option<Vec<BizTransaction>>,
    /// List of source identifiers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_list: Option<Vec<Source>>,
    /// List of destination identifiers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_list: Option<Vec<Destination>>,
    /// List of sensor records
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sensor_element_list: Option<Vec<SensorElement>>,
    /// Persistent state changes of disposition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persistent_disposition: Option<PersistentDisposition>,

    /// Extra custom JSON elements
    #[serde(flatten)]
    pub extensions: serde_json::Map<String, serde_json::Value>,
}

impl ObjectEvent {
    /// Creates a new `ObjectEvent` with required properties.
    #[must_use]
    pub fn new(event_time: DateTime<Utc>, tz_offset: String, action: Action) -> Self {
        Self {
            event_time,
            event_time_zone_offset: tz_offset,
            record_time: None,
            event_id: None,
            error_declaration: None,
            action,
            epc_list: None,
            quantity_list: None,
            biz_step: None,
            disposition: None,
            read_point: None,
            biz_location: None,
            biz_transaction_list: None,
            source_list: None,
            destination_list: None,
            sensor_element_list: None,
            persistent_disposition: None,
            extensions: serde_json::Map::new(),
        }
    }
}

/// An `AggregationEvent` captures an event where objects are aggregated into a parent container.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AggregationEvent {
    /// Date and time when the event occurred
    pub event_time: DateTime<Utc>,
    /// Timezone offset
    pub event_time_zone_offset: String,
    /// Record timestamp (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub record_time: Option<DateTime<Utc>>,
    /// Event identifier
    #[serde(rename = "eventID", skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
    /// Error declaration details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_declaration: Option<ErrorDeclaration>,
    
    /// Action specifying parent-child state change (ADD, DELETE, OBSERVE)
    pub action: Action,
    /// Parent container EPC
    #[serde(rename = "parentID", skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<Epc>,
    /// List of child EPCs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub child_e_p_cs: Option<Vec<Epc>>,
    /// List of child quantities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub child_quantity_list: Option<Vec<QuantityElement>>,
    
    /// Business step (e.g. packing)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub biz_step: Option<BizStep>,
    /// Disposition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disposition: Option<Disposition>,
    /// Read point
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_point: Option<ReadPoint>,
    /// Business location
    #[serde(skip_serializing_if = "Option::is_none")]
    pub biz_location: Option<BizLocation>,
    /// Business transactions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub biz_transaction_list: Option<Vec<BizTransaction>>,
    /// Source list
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_list: Option<Vec<Source>>,
    /// Destination list
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_list: Option<Vec<Destination>>,
    /// Sensor element list
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sensor_element_list: Option<Vec<SensorElement>>,

    /// Extra custom fields
    #[serde(flatten)]
    pub extensions: serde_json::Map<String, serde_json::Value>,
}

impl AggregationEvent {
    /// Creates a new `AggregationEvent` with required properties.
    #[must_use]
    pub fn new(event_time: DateTime<Utc>, tz_offset: String, action: Action) -> Self {
        Self {
            event_time,
            event_time_zone_offset: tz_offset,
            record_time: None,
            event_id: None,
            error_declaration: None,
            action,
            parent_id: None,
            child_e_p_cs: None,
            child_quantity_list: None,
            biz_step: None,
            disposition: None,
            read_point: None,
            biz_location: None,
            biz_transaction_list: None,
            source_list: None,
            destination_list: None,
            sensor_element_list: None,
            extensions: serde_json::Map::new(),
        }
    }
}

/// A `TransformationEvent` captures the consumption of input items and the creation of output items.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformationEvent {
    /// Date and time when the event occurred
    pub event_time: DateTime<Utc>,
    /// Timezone offset
    pub event_time_zone_offset: String,
    /// Record timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub record_time: Option<DateTime<Utc>>,
    /// Event identifier
    #[serde(rename = "eventID", skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
    /// Error declaration details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_declaration: Option<ErrorDeclaration>,
    
    /// Input item EPCs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_e_p_c_list: Option<Vec<Epc>>,
    /// Input item quantities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_quantity_list: Option<Vec<QuantityElement>>,
    /// Output item EPCs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_e_p_c_list: Option<Vec<Epc>>,
    /// Output item quantities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_quantity_list: Option<Vec<QuantityElement>>,
    /// Unique transformation logic identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transformation_id: Option<String>,
    
    /// Business step
    #[serde(skip_serializing_if = "Option::is_none")]
    pub biz_step: Option<BizStep>,
    /// Disposition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disposition: Option<Disposition>,
    /// Read point
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_point: Option<ReadPoint>,
    /// Business location
    #[serde(skip_serializing_if = "Option::is_none")]
    pub biz_location: Option<BizLocation>,
    /// Business transactions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub biz_transaction_list: Option<Vec<BizTransaction>>,
    /// Source list
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_list: Option<Vec<Source>>,
    /// Destination list
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_list: Option<Vec<Destination>>,
    /// Sensor element list
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sensor_element_list: Option<Vec<SensorElement>>,

    /// Extra custom fields
    #[serde(flatten)]
    pub extensions: serde_json::Map<String, serde_json::Value>,
}

impl TransformationEvent {
    /// Creates a new `TransformationEvent` with required properties.
    #[must_use]
    pub fn new(event_time: DateTime<Utc>, tz_offset: String) -> Self {
        Self {
            event_time,
            event_time_zone_offset: tz_offset,
            record_time: None,
            event_id: None,
            error_declaration: None,
            input_e_p_c_list: None,
            input_quantity_list: None,
            output_e_p_c_list: None,
            output_quantity_list: None,
            transformation_id: None,
            biz_step: None,
            disposition: None,
            read_point: None,
            biz_location: None,
            biz_transaction_list: None,
            source_list: None,
            destination_list: None,
            sensor_element_list: None,
            extensions: serde_json::Map::new(),
        }
    }
}

/// An `AssociationEvent` captures the assembly or disassembly of reusable assets.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssociationEvent {
    /// Date and time when the event occurred
    pub event_time: DateTime<Utc>,
    /// Timezone offset
    pub event_time_zone_offset: String,
    /// Record timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub record_time: Option<DateTime<Utc>>,
    /// Event identifier
    #[serde(rename = "eventID", skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
    /// Error declaration details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_declaration: Option<ErrorDeclaration>,
    
    /// Action specifying parent-child state change (ADD, DELETE, OBSERVE)
    pub action: Action,
    /// Parent container EPC
    #[serde(rename = "parentID", skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<Epc>,
    /// List of child EPCs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub child_e_p_cs: Option<Vec<Epc>>,
    /// List of child quantities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub child_quantity_list: Option<Vec<QuantityElement>>,
    
    /// Business step
    #[serde(skip_serializing_if = "Option::is_none")]
    pub biz_step: Option<BizStep>,
    /// Disposition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disposition: Option<Disposition>,
    /// Read point
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_point: Option<ReadPoint>,
    /// Business location
    #[serde(skip_serializing_if = "Option::is_none")]
    pub biz_location: Option<BizLocation>,
    /// Business transactions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub biz_transaction_list: Option<Vec<BizTransaction>>,
    /// Source list
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_list: Option<Vec<Source>>,
    /// Destination list
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_list: Option<Vec<Destination>>,
    /// Sensor element list
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sensor_element_list: Option<Vec<SensorElement>>,

    /// Extra custom fields
    #[serde(flatten)]
    pub extensions: serde_json::Map<String, serde_json::Value>,
}

impl AssociationEvent {
    /// Creates a new `AssociationEvent` with required properties.
    #[must_use]
    pub fn new(event_time: DateTime<Utc>, tz_offset: String, action: Action) -> Self {
        Self {
            event_time,
            event_time_zone_offset: tz_offset,
            record_time: None,
            event_id: None,
            error_declaration: None,
            action,
            parent_id: None,
            child_e_p_cs: None,
            child_quantity_list: None,
            biz_step: None,
            disposition: None,
            read_point: None,
            biz_location: None,
            biz_transaction_list: None,
            source_list: None,
            destination_list: None,
            sensor_element_list: None,
            extensions: serde_json::Map::new(),
        }
    }
}

/// A `TransactionEvent` captures items associated with a business transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionEvent {
    /// Date and time when the event occurred
    pub event_time: DateTime<Utc>,
    /// Timezone offset
    pub event_time_zone_offset: String,
    /// Record timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub record_time: Option<DateTime<Utc>>,
    /// Event identifier
    #[serde(rename = "eventID", skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
    /// Error declaration details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_declaration: Option<ErrorDeclaration>,
    
    /// Action specifying parent-child state change
    pub action: Action,
    /// Parent identifier
    #[serde(rename = "parentID", skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<Epc>,
    /// List of item EPCs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub epc_list: Option<Vec<Epc>>,
    /// List of quantities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity_list: Option<Vec<QuantityElement>>,
    
    /// Business step
    #[serde(skip_serializing_if = "Option::is_none")]
    pub biz_step: Option<BizStep>,
    /// Disposition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disposition: Option<Disposition>,
    /// Read point
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_point: Option<ReadPoint>,
    /// Business location
    #[serde(skip_serializing_if = "Option::is_none")]
    pub biz_location: Option<BizLocation>,
    /// Business transactions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub biz_transaction_list: Option<Vec<BizTransaction>>,
    /// Source list
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_list: Option<Vec<Source>>,
    /// Destination list
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_list: Option<Vec<Destination>>,
    /// Sensor element list
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sensor_element_list: Option<Vec<SensorElement>>,

    /// Extra custom fields
    #[serde(flatten)]
    pub extensions: serde_json::Map<String, serde_json::Value>,
}

impl TransactionEvent {
    /// Creates a new `TransactionEvent` with required properties.
    #[must_use]
    pub fn new(event_time: DateTime<Utc>, tz_offset: String, action: Action) -> Self {
        Self {
            event_time,
            event_time_zone_offset: tz_offset,
            record_time: None,
            event_id: None,
            error_declaration: None,
            action,
            parent_id: None,
            epc_list: None,
            quantity_list: None,
            biz_step: None,
            disposition: None,
            read_point: None,
            biz_location: None,
            biz_transaction_list: None,
            source_list: None,
            destination_list: None,
            sensor_element_list: None,
            extensions: serde_json::Map::new(),
        }
    }
}

/// Sum-type representing any EPCIS event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EPCISEvent {
    /// An `ObjectEvent` variant.
    ObjectEvent(ObjectEvent),
    /// An `AggregationEvent` variant.
    AggregationEvent(AggregationEvent),
    /// A `TransformationEvent` variant.
    TransformationEvent(TransformationEvent),
    /// An `AssociationEvent` variant.
    AssociationEvent(AssociationEvent),
    /// A `TransactionEvent` variant.
    TransactionEvent(TransactionEvent),
}
