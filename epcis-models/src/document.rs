//! `EPCISDocument` schema definition.

#![deny(missing_docs)]
#![deny(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::events::EPCISEvent;

/// Helper to deserialize a string that might be wrapped as a map under quick-xml (e.g. text tag values).
///
/// # Errors
///
/// Returns error if deserialization fails.
pub fn deserialize_string_or_map_text<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct StringVisitor;
    impl<'de> serde::de::Visitor<'de> for StringVisitor {
        type Value = String;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string or a map with a text value")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(v.to_string())
        }

        fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(v)
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>,
        {
            let mut value: Option<String> = None;
            while let Some(key) = map.next_key::<String>()? {
                if key == "$text" || key == "$value" || key == "" {
                    value = Some(map.next_value()?);
                } else {
                    let _: serde::de::IgnoredAny = map.next_value()?;
                }
            }
            value.ok_or_else(|| serde::de::Error::custom("missing string value"))
        }
    }
    deserializer.deserialize_any(StringVisitor)
}

/// Custom serialization/deserialization helper for `DateTime<Utc>` under XML/JSON.
pub mod datetime_serde {
    use chrono::{DateTime, Utc};
    use serde::{self, Serializer};

    /// Serializes to RFC3339 string.
    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&date.to_rfc3339())
    }

    /// Deserializes string or map under quick-xml.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = super::deserialize_string_or_map_text(deserializer)?;
        s.parse::<DateTime<Utc>>().map_err(serde::de::Error::custom)
    }
}

/// Custom serialization/deserialization helper for `Option<DateTime<Utc>>` under XML/JSON.
pub mod opt_datetime_serde {
    use chrono::{DateTime, Utc};
    use serde::{self, Serializer};

    /// Serializes to RFC3339 string.
    pub fn serialize<S>(date: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match date {
            Some(d) => serializer.serialize_some(&d.to_rfc3339()),
            None => serializer.serialize_none(),
        }
    }

    /// Deserializes option string or map.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct OptDateTimeVisitor;
        impl<'de> serde::de::Visitor<'de> for OptDateTimeVisitor {
            type Value = Option<DateTime<Utc>>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("an optional datetime string")
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(None)
            }

            fn visit_some<D2>(self, deserializer: D2) -> Result<Self::Value, D2::Error>
            where
                D2: serde::Deserializer<'de>,
            {
                let s = super::deserialize_string_or_map_text(deserializer)?;
                if s.is_empty() {
                    Ok(None)
                } else {
                    s.parse::<DateTime<Utc>>().map(Some).map_err(serde::de::Error::custom)
                }
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if v.is_empty() {
                    Ok(None)
                } else {
                    v.parse::<DateTime<Utc>>().map(Some).map_err(serde::de::Error::custom)
                }
            }

            fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let s = super::deserialize_string_or_map_text(serde::de::value::MapAccessDeserializer::new(map))?;
                if s.is_empty() {
                    Ok(None)
                } else {
                    s.parse::<DateTime<Utc>>().map(Some).map_err(serde::de::Error::custom)
                }
            }
        }

        deserializer.deserialize_any(OptDateTimeVisitor)
    }
}

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
    #[serde(with = "crate::document::datetime_serde")]
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

    /// Parses an EPCISDocument from an XML string.
    ///
    /// # Errors
    ///
    /// Returns error if XML parsing fails.
    pub fn from_xml(xml_str: &str) -> Result<Self, quick_xml::DeError> {
        let raw_val: serde_json::Value = quick_xml::de::from_str(xml_str)?;
        let clean_val = clean_json_value(raw_val);
        serde_json::from_value(clean_val).map_err(|e| {
            quick_xml::DeError::Custom(e.to_string())
        })
    }

    /// Serializes the EPCISDocument to an XML string.
    ///
    /// # Errors
    ///
    /// Returns error if XML serialization fails.
    pub fn to_xml(&self) -> Result<String, quick_xml::se::SeError> {
        quick_xml::se::to_string_with_root("EPCISDocument", self)
    }
}

fn is_array_field(key: &str) -> bool {
    matches!(
        key,
        "eventList"
            | "epcList"
            | "quantityList"
            | "sensorElementList"
            | "sensorReport"
            | "sourceList"
            | "destinationList"
            | "bizTransactionList"
    )
}

fn clean_json_value(v: serde_json::Value) -> serde_json::Value {
    match v {
        serde_json::Value::Object(mut map) => {
            if map.len() == 1 {
                if let Some(val) = map.remove("$text").or_else(|| map.remove("$value")) {
                    return clean_json_value(val);
                }
            }
            let cleaned = map
                .into_iter()
                .map(|(k, val)| {
                    let cleaned_val = clean_json_value(val);
                    if is_array_field(&k) {
                        match cleaned_val {
                            serde_json::Value::Array(_) => (k, cleaned_val),
                            other => (k, serde_json::Value::Array(vec![other])),
                        }
                    } else {
                        (k, cleaned_val)
                    }
                })
                .collect();
            serde_json::Value::Object(cleaned)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(clean_json_value).collect())
        }
        other => other,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::ObjectEvent;
    use crate::types::Action;
    use chrono::Utc;

    #[test]
    fn test_xml_roundtrip() {
        let event = ObjectEvent::new(Utc::now(), "+00:00".to_string(), Action::Observe);
        let doc = EPCISDocument::new(vec![EPCISEvent::ObjectEvent(event)]);

        // Serialize to XML
        let xml_output = doc.to_xml();
        assert!(xml_output.is_ok());
        let xml_str = xml_output.unwrap();
        assert!(xml_str.contains("<EPCISDocument"));
        assert!(xml_str.contains("<epcisBody>"));
        assert!(xml_str.contains("<type>ObjectEvent</type>"));

        // Deserialize back from XML
        let deserialized = EPCISDocument::from_xml(&xml_str);
        assert!(deserialized.is_ok());
        let parsed_doc = deserialized.unwrap();
        assert_eq!(parsed_doc.schema_version, "2.0");
        assert_eq!(parsed_doc.r#type, "EPCISDocument");
    }
}
