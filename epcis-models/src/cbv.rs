//! Core Business Vocabulary (CBV) standard enums and helper representations.

use std::str::FromStr;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use strum_macros::{Display, EnumString, AsRefStr};

/// Standard business step (bizStep) values as defined by GS1 CBV.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, EnumString, AsRefStr)]
#[non_exhaustive]
pub enum StandardBizStep {
    /// accepting
    #[strum(serialize = "urn:epcglobal:cbv:bizstep:accepting")]
    Accepting,
    /// arriving
    #[strum(serialize = "urn:epcglobal:cbv:bizstep:arriving")]
    Arriving,
    /// assembling
    #[strum(serialize = "urn:epcglobal:cbv:bizstep:assembling")]
    Assembling,
    /// collecting
    #[strum(serialize = "urn:epcglobal:cbv:bizstep:collecting")]
    Collecting,
    /// commissioning
    #[strum(serialize = "urn:epcglobal:cbv:bizstep:commissioning")]
    Commissioning,
    /// decommissioning
    #[strum(serialize = "urn:epcglobal:cbv:bizstep:decommissioning")]
    Decommissioning,
    /// departing
    #[strum(serialize = "urn:epcglobal:cbv:bizstep:departing")]
    Departing,
    /// destroying
    #[strum(serialize = "urn:epcglobal:cbv:bizstep:destroying")]
    Destroying,
    /// disassembling
    #[strum(serialize = "urn:epcglobal:cbv:bizstep:disassembling")]
    Disassembling,
    /// `entering_exiting`
    #[strum(serialize = "urn:epcglobal:cbv:bizstep:entering_exiting")]
    EnteringExiting,
    /// holding
    #[strum(serialize = "urn:epcglobal:cbv:bizstep:holding")]
    Holding,
    /// inspecting
    #[strum(serialize = "urn:epcglobal:cbv:bizstep:inspecting")]
    Inspecting,
    /// installing
    #[strum(serialize = "urn:epcglobal:cbv:bizstep:installing")]
    Installing,
    /// killing
    #[strum(serialize = "urn:epcglobal:cbv:bizstep:killing")]
    Killing,
    /// loading
    #[strum(serialize = "urn:epcglobal:cbv:bizstep:loading")]
    Loading,
    /// other
    #[strum(serialize = "urn:epcglobal:cbv:bizstep:other")]
    Other,
    /// packing
    #[strum(serialize = "urn:epcglobal:cbv:bizstep:packing")]
    Packing,
    /// picking
    #[strum(serialize = "urn:epcglobal:cbv:bizstep:picking")]
    Picking,
    /// receiving
    #[strum(serialize = "urn:epcglobal:cbv:bizstep:receiving")]
    Receiving,
    /// removing
    #[strum(serialize = "urn:epcglobal:cbv:bizstep:removing")]
    Removing,
    /// repackaging
    #[strum(serialize = "urn:epcglobal:cbv:bizstep:repackaging")]
    Repackaging,
    /// repairing
    #[strum(serialize = "urn:epcglobal:cbv:bizstep:repairing")]
    Repairing,
    /// reselling
    #[strum(serialize = "urn:epcglobal:cbv:bizstep:reselling")]
    Reselling,
    /// shipping
    #[strum(serialize = "urn:epcglobal:cbv:bizstep:shipping")]
    Shipping,
    /// stocking
    #[strum(serialize = "urn:epcglobal:cbv:bizstep:stocking")]
    Stocking,
    /// storing
    #[strum(serialize = "urn:epcglobal:cbv:bizstep:storing")]
    Storing,
    /// transporting
    #[strum(serialize = "urn:epcglobal:cbv:bizstep:transporting")]
    Transporting,
    /// unloading
    #[strum(serialize = "urn:epcglobal:cbv:bizstep:unloading")]
    Unloading,
    /// unpacking
    #[strum(serialize = "urn:epcglobal:cbv:bizstep:unpacking")]
    Unpacking,
    /// voiding
    #[strum(serialize = "urn:epcglobal:cbv:bizstep:voiding")]
    Voiding,
}

/// A business step that can either be a standard GS1 CBV value or a custom URI.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BizStep {
    /// A standard GS1 CBV business step.
    Standard(StandardBizStep),
    /// A custom business step URI.
    Custom(String),
}

impl BizStep {
    /// Returns the string representation of the business step.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            BizStep::Standard(s) => s.as_ref(),
            BizStep::Custom(c) => c.as_str(),
        }
    }
}

impl FromStr for BizStep {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match StandardBizStep::from_str(s) {
            Ok(standard) => Ok(BizStep::Standard(standard)),
            Err(_) => Ok(BizStep::Custom(s.to_string())),
        }
    }
}

impl Serialize for BizStep {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for BizStep {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(BizStep::from_str(&s).unwrap())
    }
}

/// Standard disposition values as defined by GS1 CBV.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, EnumString, AsRefStr)]
#[non_exhaustive]
pub enum StandardDisposition {
    /// active
    #[strum(serialize = "urn:epcglobal:cbv:disp:active")]
    Active,
    /// `container_closed`
    #[strum(serialize = "urn:epcglobal:cbv:disp:container_closed")]
    ContainerClosed,
    /// `container_open`
    #[strum(serialize = "urn:epcglobal:cbv:disp:container_open")]
    ContainerOpen,
    /// damaged
    #[strum(serialize = "urn:epcglobal:cbv:disp:damaged")]
    Damaged,
    /// destroyed
    #[strum(serialize = "urn:epcglobal:cbv:disp:destroyed")]
    Destroyed,
    /// dispensed
    #[strum(serialize = "urn:epcglobal:cbv:disp:dispensed")]
    Dispensed,
    /// disposed
    #[strum(serialize = "urn:epcglobal:cbv:disp:disposed")]
    Disposed,
    /// encoded
    #[strum(serialize = "urn:epcglobal:cbv:disp:encoded")]
    Encoded,
    /// expired
    #[strum(serialize = "urn:epcglobal:cbv:disp:expired")]
    Expired,
    /// `in_progress`
    #[strum(serialize = "urn:epcglobal:cbv:disp:in_progress")]
    InProgress,
    /// `in_transit`
    #[strum(serialize = "urn:epcglobal:cbv:disp:in_transit")]
    InTransit,
    /// inactive
    #[strum(serialize = "urn:epcglobal:cbv:disp:inactive")]
    Inactive,
    /// `no_pedigree_match`
    #[strum(serialize = "urn:epcglobal:cbv:disp:no_pedigree_match")]
    NoPedigreeMatch,
    /// `non_sellable`
    #[strum(serialize = "urn:epcglobal:cbv:disp:non_sellable")]
    NonSellable,
    /// `non_disposed`
    #[strum(serialize = "urn:epcglobal:cbv:disp:non_disposed")]
    NonDisposed,
    /// `parts_damaged`
    #[strum(serialize = "urn:epcglobal:cbv:disp:parts_damaged")]
    PartsDamaged,
    /// `pedigree_match`
    #[strum(serialize = "urn:epcglobal:cbv:disp:pedigree_match")]
    PedigreeMatch,
    /// recalled
    #[strum(serialize = "urn:epcglobal:cbv:disp:recalled")]
    Recalled,
    /// `receiving_unconfirmed`
    #[strum(serialize = "urn:epcglobal:cbv:disp:receiving_unconfirmed")]
    ReceivingUnconfirmed,
    /// reshelved
    #[strum(serialize = "urn:epcglobal:cbv:disp:reshelved")]
    Reshelved,
    /// returned
    #[strum(serialize = "urn:epcglobal:cbv:disp:returned")]
    Returned,
    /// `sellable_accessible`
    #[strum(serialize = "urn:epcglobal:cbv:disp:sellable_accessible")]
    SellableAccessible,
    /// `sellable_not_accessible`
    #[strum(serialize = "urn:epcglobal:cbv:disp:sellable_not_accessible")]
    SellableNotAccessible,
    /// stolen
    #[strum(serialize = "urn:epcglobal:cbv:disp:stolen")]
    Stolen,
    /// unknown
    #[strum(serialize = "urn:epcglobal:cbv:disp:unknown")]
    Unknown,
}

/// A disposition that can either be a standard GS1 CBV value or a custom URI.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Disposition {
    /// A standard GS1 CBV disposition.
    Standard(StandardDisposition),
    /// A custom disposition URI.
    Custom(String),
}

impl Disposition {
    /// Returns the string representation of the disposition.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Disposition::Standard(s) => s.as_ref(),
            Disposition::Custom(c) => c.as_str(),
        }
    }
}

impl FromStr for Disposition {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match StandardDisposition::from_str(s) {
            Ok(standard) => Ok(Disposition::Standard(standard)),
            Err(_) => Ok(Disposition::Custom(s.to_string())),
        }
    }
}

impl Serialize for Disposition {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Disposition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Disposition::from_str(&s).unwrap())
    }
}
