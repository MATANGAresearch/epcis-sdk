#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![deny(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

pub mod cbv;
pub mod document;
pub mod error;
pub mod events;
pub mod types;
pub mod validation;
mod xml;

// Re-export common types at crate level for convenient usage
pub use cbv::{BizStep, Disposition, StandardBizStep, StandardDisposition};
pub use document::{
    EPCISBody, EPCISDocument, EPCISHeader, EPCISMasterData, VocabularyAttribute, VocabularyElement,
    VocabularyElementList,
};
pub use error::EpcisModelError;
pub use events::{
    AggregationEvent, AssociationEvent, EPCISEvent, ObjectEvent, PersistentDisposition,
    TransactionEvent, TransformationEvent,
};
pub use types::{
    Action, BizLocation, BizTransaction, Destination, Epc, ErrorDeclaration, QuantityElement,
    ReadPoint, SensorElement, SensorMetadata, SensorReport, Source,
};
pub use validation::{ValidationError, validate_extension_keys};
