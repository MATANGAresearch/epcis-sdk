//! `EPCISDocument` schema definition.

#![deny(missing_docs)]
#![deny(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use crate::events::EPCISEvent;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
                if key == "$text" || key == "$value" || key.is_empty() {
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
    ///
    /// # Errors
    ///
    /// Returns error if the underlying serializer fails.
    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&date.to_rfc3339())
    }

    /// Deserializes string or map under quick-xml.
    ///
    /// # Errors
    ///
    /// Returns error if the value is not a parseable RFC3339 datetime.
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
    ///
    /// # Errors
    ///
    /// Returns error if the underlying serializer fails.
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
    ///
    /// # Errors
    ///
    /// Returns error if a present value is not a parseable RFC3339 datetime.
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
                    s.parse::<DateTime<Utc>>()
                        .map(Some)
                        .map_err(serde::de::Error::custom)
                }
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if v.is_empty() {
                    Ok(None)
                } else {
                    v.parse::<DateTime<Utc>>()
                        .map(Some)
                        .map_err(serde::de::Error::custom)
                }
            }

            fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let s = super::deserialize_string_or_map_text(
                    serde::de::value::MapAccessDeserializer::new(map),
                )?;
                if s.is_empty() {
                    Ok(None)
                } else {
                    s.parse::<DateTime<Utc>>()
                        .map(Some)
                        .map_err(serde::de::Error::custom)
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

    /// Parses a standard EPCIS 2.0 XML document.
    ///
    /// Accepts the `<epcis:EPCISDocument>` element structure defined by the
    /// EPCIS 2.0 XSD: event types as element names inside
    /// `<EPCISBody><EventList>`, sensor data as XML attributes, `type`
    /// attributes on business transactions / sources / destinations, and
    /// foreign-namespace user extension elements (whose prefix declarations
    /// are carried over into the document's JSON-LD `@context`).
    ///
    /// `EPCISHeader` master data (`EPCISMasterData` vocabularies) is mapped
    /// into the typed header. A Standard Business Document Header (SBDH)
    /// inside `EPCISHeader`, if present, is not modelled and is skipped.
    ///
    /// # Errors
    ///
    /// Returns [`crate::EpcisModelError::InvalidXml`] if the document is not
    /// well-formed EPCIS 2.0 XML.
    pub fn from_xml(xml_str: &str) -> Result<Self, crate::EpcisModelError> {
        let json_shape = crate::xml::epcis_xml_to_json(xml_str)?;
        serde_json::from_value(json_shape)
            .map_err(|e| crate::EpcisModelError::InvalidXml(e.to_string()))
    }

    /// Serializes the document as standard EPCIS 2.0 XML.
    ///
    /// Event children are emitted in the order required by the EPCIS 2.0
    /// XSD, and prefix mappings found in the JSON-LD `@context` are declared
    /// as `xmlns:` attributes on the root element so extension elements stay
    /// resolvable.
    ///
    /// # Errors
    ///
    /// Returns [`crate::EpcisModelError::InvalidXml`] if the document cannot
    /// be represented (e.g. serialization of a field fails).
    pub fn to_xml(&self) -> Result<String, crate::EpcisModelError> {
        let json_shape = serde_json::to_value(self)
            .map_err(|e| crate::EpcisModelError::InvalidXml(e.to_string()))?;
        crate::xml::json_to_epcis_xml(&json_shape)
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
    pub vocabulary_list: Vec<Vocabulary>,
}

/// A master data vocabulary: a type plus its element list.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vocabulary {
    /// Type of the vocabulary (e.g. `urn:epcglobal:epcis:vtype:BusinessLocation`)
    #[serde(rename = "type")]
    pub r#type: String,
    /// The vocabulary elements of this type
    pub vocabulary_element_list: Vec<VocabularyElement>,
}

/// A single master data element: an identifier with attributes and children.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VocabularyElement {
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
    /// Attribute value (string or structured content), serialized as
    /// `attribute` per the EPCIS 2.0 JSON schema
    #[serde(rename = "attribute")]
    pub attribute: serde_json::Value,
}

/// Body container holding the array of EPCIS events.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EPCISBody {
    /// List of events in the body
    pub event_list: Vec<EPCISEvent>,
}

/// Represents a top-level EPCIS 2.0 query document (`EPCISQueryDocument`),
/// the envelope returned by EPCIS query interfaces.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EPCISQueryDocument {
    /// Context property containing schema mappings for JSON-LD compliance.
    #[serde(rename = "@context")]
    pub context: serde_json::Value,

    /// Document identifier (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Type identifier, always "`EPCISQueryDocument`".
    #[serde(rename = "type")]
    pub r#type: String,

    /// Schema version, typically "2.0".
    pub schema_version: String,

    /// Creation timestamp of this document.
    #[serde(with = "crate::document::datetime_serde")]
    pub creation_date: DateTime<Utc>,

    /// Body containing the query results.
    pub epcis_body: EPCISQueryBody,

    /// Extension elements.
    #[serde(flatten)]
    pub extensions: serde_json::Map<String, serde_json::Value>,
}

impl EPCISQueryDocument {
    /// Parses a standard EPCIS 2.0 query document from XML
    /// (`<epcisq:EPCISQueryDocument>` with
    /// `<EPCISBody><QueryResults><resultsBody><EventList>` structure).
    ///
    /// # Errors
    ///
    /// Returns [`crate::EpcisModelError::InvalidXml`] if the document is not
    /// well-formed EPCIS 2.0 query XML.
    pub fn from_xml(xml_str: &str) -> Result<Self, crate::EpcisModelError> {
        let json_shape = crate::xml::epcis_query_xml_to_json(xml_str)?;
        serde_json::from_value(json_shape)
            .map_err(|e| crate::EpcisModelError::InvalidXml(e.to_string()))
    }

    /// Serializes the query document as standard EPCIS 2.0 query XML.
    ///
    /// # Errors
    ///
    /// Returns [`crate::EpcisModelError::InvalidXml`] if the document cannot
    /// be represented.
    pub fn to_xml(&self) -> Result<String, crate::EpcisModelError> {
        let json_shape = serde_json::to_value(self)
            .map_err(|e| crate::EpcisModelError::InvalidXml(e.to_string()))?;
        crate::xml::json_to_epcis_query_xml(&json_shape)
    }
}

/// Body container of a query document.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EPCISQueryBody {
    /// The results of the query
    pub query_results: QueryResults,
}

/// Results envelope of an EPCIS query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryResults {
    /// Identifier of the standing-query subscription, if any
    #[serde(rename = "subscriptionID", skip_serializing_if = "Option::is_none")]
    pub subscription_id: Option<String>,
    /// Name of the query that produced these results
    pub query_name: String,
    /// The result payload
    pub results_body: ResultsBody,
    /// Extra custom fields (e.g. repository ignore-field instructions)
    #[serde(flatten)]
    pub extensions: serde_json::Map<String, serde_json::Value>,
}

/// Result payload holding the matched events.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultsBody {
    /// List of events returned by the query
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

        // Serialize to standard EPCIS 2.0 XML
        let xml_str = doc.to_xml().unwrap();
        assert!(xml_str.contains("<epcis:EPCISDocument"));
        assert!(xml_str.contains("xmlns:epcis=\"urn:epcglobal:epcis:xsd:2\""));
        assert!(xml_str.contains("<EPCISBody>"));
        assert!(xml_str.contains("<ObjectEvent>"));
        assert!(xml_str.contains("<action>OBSERVE</action>"));

        // Deserialize back from XML
        let parsed_doc = EPCISDocument::from_xml(&xml_str).unwrap();
        assert_eq!(parsed_doc.schema_version, "2.0");
        assert_eq!(parsed_doc.r#type, "EPCISDocument");
        assert_eq!(parsed_doc.epcis_body.event_list.len(), 1);
        assert!(matches!(
            parsed_doc.epcis_body.event_list[0],
            EPCISEvent::ObjectEvent(_)
        ));
    }

    #[test]
    fn test_xml_master_data_roundtrip() {
        // Modeled on the EPCIS 2.0 standard's master data example.
        let xml = r#"<?xml version="1.0"?>
<epcis:EPCISDocument xmlns:epcis="urn:epcglobal:epcis:xsd:2" xmlns:cbvmda="urn:epcglobal:cbv:mda" schemaVersion="2.0" creationDate="2020-01-15T10:00:00.000+01:00">
  <EPCISHeader>
    <EPCISMasterData>
      <VocabularyList>
        <Vocabulary type="urn:epcglobal:epcis:vtype:BusinessLocation">
          <VocabularyElementList>
            <VocabularyElement id="urn:epc:id:sgln:0037000.00729.0">
              <attribute id="cbvmda:site">0037000007296</attribute>
              <attribute id="cbvmda:address">
                <cbvmda:Street>100 Nowhere Street</cbvmda:Street>
                <cbvmda:City>Fancy</cbvmda:City>
              </attribute>
              <children>
                <id>urn:epc:id:sgln:0037000.00729.8201</id>
                <id>urn:epc:id:sgln:0037000.00729.8202</id>
              </children>
            </VocabularyElement>
          </VocabularyElementList>
        </Vocabulary>
      </VocabularyList>
    </EPCISMasterData>
  </EPCISHeader>
  <EPCISBody>
    <EventList>
      <ObjectEvent>
        <eventTime>2020-01-15T10:00:00.000+01:00</eventTime>
        <eventTimeZoneOffset>+01:00</eventTimeZoneOffset>
        <epcList><epc>urn:epc:id:sgtin:0037000.030241.1041970</epc></epcList>
        <action>OBSERVE</action>
      </ObjectEvent>
    </EventList>
  </EPCISBody>
</epcis:EPCISDocument>"#;

        let doc = EPCISDocument::from_xml(xml).unwrap();
        let master = doc
            .epcis_header
            .as_ref()
            .and_then(|h| h.epcis_master_data.as_ref())
            .expect("master data parsed");
        assert_eq!(master.vocabulary_list.len(), 1);
        let vocab = &master.vocabulary_list[0];
        assert_eq!(vocab.r#type, "urn:epcglobal:epcis:vtype:BusinessLocation");
        let element = &vocab.vocabulary_element_list[0];
        assert_eq!(element.id, "urn:epc:id:sgln:0037000.00729.0");
        let attrs = element.attributes.as_ref().unwrap();
        assert_eq!(attrs[0].id, "cbvmda:site");
        assert_eq!(attrs[0].attribute, serde_json::json!("0037000007296"));
        assert_eq!(
            attrs[1].attribute["cbvmda:City"],
            serde_json::json!("Fancy")
        );
        assert_eq!(element.children.as_ref().unwrap().len(), 2);

        // Round-trip: re-serialized XML must parse to the same typed header.
        let rewritten = doc.to_xml().unwrap();
        let reparsed = EPCISDocument::from_xml(&rewritten).unwrap();
        assert_eq!(
            serde_json::to_value(reparsed.epcis_header).unwrap(),
            serde_json::to_value(doc.epcis_header).unwrap()
        );
        assert_eq!(reparsed.epcis_body.event_list.len(), 1);
    }

    #[test]
    fn test_from_xml_standard_document() {
        let xml = r#"<?xml version="1.0"?>
<epcis:EPCISDocument xmlns:epcis="urn:epcglobal:epcis:xsd:2" xmlns:example="https://ns.example.com/epcis/" schemaVersion="2.0" creationDate="2019-10-21T14:59:02.099+02:00">
  <EPCISBody>
    <EventList>
      <ObjectEvent>
        <eventTime>2019-10-21T11:00:30.000+01:00</eventTime>
        <eventTimeZoneOffset>+01:00</eventTimeZoneOffset>
        <epcList><epc>urn:epc:id:sscc:5200001.0111111146</epc></epcList>
        <action>OBSERVE</action>
        <bizStep>urn:epcglobal:cbv:bizstep:departing</bizStep>
        <readPoint><id>urn:epc:id:sgln:5200001.99901.0</id></readPoint>
        <bizTransactionList>
          <bizTransaction type="urn:epcglobal:cbv:btt:desadv">urn:epcglobal:cbv:bt:5200001000008:4711</bizTransaction>
        </bizTransactionList>
        <example:myField>custom</example:myField>
      </ObjectEvent>
    </EventList>
  </EPCISBody>
</epcis:EPCISDocument>"#;
        let doc = EPCISDocument::from_xml(xml).unwrap();
        assert_eq!(doc.epcis_body.event_list.len(), 1);
        let EPCISEvent::ObjectEvent(event) = &doc.epcis_body.event_list[0] else {
            panic!("expected ObjectEvent");
        };
        assert_eq!(event.action, Action::Observe);
        assert_eq!(event.epc_list.as_ref().unwrap().len(), 1);
        assert_eq!(
            event.biz_transaction_list.as_ref().unwrap()[0].r#type,
            "urn:epcglobal:cbv:btt:desadv"
        );
        assert_eq!(
            event.extensions.get("example:myField").unwrap(),
            &serde_json::json!("custom")
        );
        // Prefix declaration carried into the JSON-LD context
        assert!(
            serde_json::to_string(&doc.context)
                .unwrap()
                .contains("https://ns.example.com/epcis/")
        );
    }
}
