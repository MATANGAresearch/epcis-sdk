#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![deny(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

pub mod cbv;
pub mod types;
pub mod events;
pub mod document;
pub mod error;
pub mod validation;

// Re-export common types at crate level for convenient usage
pub use error::EpcisModelError;
pub use cbv::{BizStep, Disposition, StandardBizStep, StandardDisposition};
pub use types::{
    Action, BizLocation, BizTransaction, Destination, Epc, ErrorDeclaration, QuantityElement,
    ReadPoint, SensorElement, SensorMetadata, SensorReport, Source,
};
pub use events::{
    EPCISEvent, ObjectEvent, AggregationEvent, TransformationEvent, AssociationEvent,
    TransactionEvent, PersistentDisposition,
};
pub use document::{
    EPCISDocument, EPCISBody, EPCISHeader, EPCISMasterData, VocabularyElement,
    VocabularyElementList, VocabularyAttribute,
};
pub use validation::{validate_extension_keys, ValidationError};
