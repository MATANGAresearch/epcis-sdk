//! Standard EPCIS 2.0 XML document parsing and serialization.
//!
//! EPCIS XML and EPCIS JSON/JSON-LD express the same data model with
//! different shapes (element names as event types, XML attributes on sensor
//! elements, `type` attributes on business transactions, and so on). This
//! module converts between standard EPCIS 2.0 XML and the JSON document
//! shape used by the typed models, so [`crate::EPCISDocument`] can read and
//! write both representations.

use crate::error::EpcisModelError;
use quick_xml::Reader;
use quick_xml::events::Event;
use serde_json::{Map, Value, json};
use std::fmt::Write as _;

// ── XML → tree ──────────────────────────────────────────────────────────────

/// A raw XML element with prefixes preserved as written.
#[derive(Debug, Default)]
struct XmlNode {
    name: String,
    attrs: Vec<(String, String)>,
    text: String,
    children: Vec<XmlNode>,
}

impl XmlNode {
    fn local_name(&self) -> &str {
        self.name.rsplit(':').next().unwrap_or(&self.name)
    }

    fn child(&self, local: &str) -> Option<&XmlNode> {
        self.children.iter().find(|c| c.local_name() == local)
    }
}

/// Namespace declarations (`prefix -> URI`) gathered from the whole document.
type Namespaces = Vec<(String, String)>;

fn parse_tree(xml: &str) -> Result<(XmlNode, Namespaces), EpcisModelError> {
    // EPCIS 1.2 compatibility wrappers carry no semantics of their own.
    let clean = xml
        .replace("<extension>", "")
        .replace("</extension>", "")
        .replace("<baseExtension>", "")
        .replace("</baseExtension>", "");

    let mut reader = Reader::from_str(&clean);
    reader.config_mut().trim_text(true);

    let mut stack: Vec<XmlNode> = vec![];
    let mut root: Option<XmlNode> = None;
    let mut namespaces: Namespaces = vec![];
    let mut buf = Vec::new();

    loop {
        let event = reader
            .read_event_into(&mut buf)
            .map_err(|err| EpcisModelError::InvalidXml(format!("{err:?}")))?;
        match event {
            Event::Start(ref e) | Event::Empty(ref e) => {
                let is_empty = matches!(event, Event::Empty(_));
                let mut node = XmlNode {
                    name: String::from_utf8_lossy(e.name().as_ref()).into_owned(),
                    ..XmlNode::default()
                };
                for attr in e.attributes().flatten() {
                    let key = String::from_utf8_lossy(attr.key.as_ref()).into_owned();
                    let val = attr
                        .decoded_and_normalized_value(
                            quick_xml::XmlVersion::Implicit1_0,
                            reader.decoder(),
                        )
                        .map_err(|err| EpcisModelError::InvalidXml(err.to_string()))?
                        .into_owned();
                    if let Some(prefix) = key.strip_prefix("xmlns:") {
                        if !namespaces.iter().any(|(p, _)| p == prefix) {
                            namespaces.push((prefix.to_string(), val));
                        }
                    } else if key == "xmlns" || key.starts_with("xsi:") {
                        // Default namespace / schema location: not needed for
                        // the JSON shape.
                    } else {
                        node.attrs.push((key, val));
                    }
                }
                if is_empty {
                    // Self-closing elements produce no End event.
                    if let Some(parent) = stack.last_mut() {
                        parent.children.push(node);
                    } else {
                        root = Some(node);
                        break;
                    }
                } else {
                    stack.push(node);
                }
            }
            Event::Text(ref e) => {
                if let Some(node) = stack.last_mut() {
                    let decoded = e
                        .decode()
                        .map_err(|err| EpcisModelError::InvalidXml(err.to_string()))?;
                    let unescaped = quick_xml::escape::unescape(decoded.as_ref())
                        .unwrap_or(std::borrow::Cow::Borrowed(decoded.as_ref()));
                    let trimmed = unescaped.trim();
                    if !trimmed.is_empty() {
                        node.text.push_str(trimmed);
                    }
                }
            }
            Event::End(_) => {
                if let Some(done) = stack.pop() {
                    if let Some(parent) = stack.last_mut() {
                        parent.children.push(done);
                    } else {
                        root = Some(done);
                        break;
                    }
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    root.map(|r| (r, namespaces))
        .ok_or_else(|| EpcisModelError::InvalidXml("empty document".to_string()))
}

// ── tree → JSON document shape ──────────────────────────────────────────────

const EPC_LISTS: [&str; 4] = ["epcList", "childEPCs", "inputEPCList", "outputEPCList"];
const QUANTITY_LISTS: [&str; 4] = [
    "quantityList",
    "childQuantityList",
    "inputQuantityList",
    "outputQuantityList",
];
/// Standard document-level keys that are not extension elements.
const DOC_KEYS: [&str; 6] = [
    "@context",
    "type",
    "schemaVersion",
    "creationDate",
    "epcisBody",
    "epcisHeader",
];
const NUMERIC_SENSOR_ATTRS: [&str; 7] = [
    "value",
    "minValue",
    "maxValue",
    "meanValue",
    "sDev",
    "percRank",
    "percValue",
];

fn attr_value(key: &str, raw: &str) -> Value {
    if NUMERIC_SENSOR_ATTRS.contains(&key)
        && let Ok(num) = raw.parse::<f64>()
        && let Some(n) = serde_json::Number::from_f64(num)
    {
        return Value::Number(n);
    }
    if key == "booleanValue" {
        match raw {
            "true" | "1" => return Value::Bool(true),
            "false" | "0" => return Value::Bool(false),
            _ => {}
        }
    }
    Value::String(raw.to_string())
}

fn attrs_to_object(node: &XmlNode) -> Value {
    let mut map = Map::new();
    for (k, v) in &node.attrs {
        map.insert(k.clone(), attr_value(k, v));
    }
    Value::Object(map)
}

/// Generic conversion for extension / ilmd content.
///
/// XML attributes on extension elements are represented as plain fields,
/// matching the EPCIS JSON-LD convention (and the canonical hash, which
/// treats attributes and leaf children identically).
fn generic_value(node: &XmlNode) -> Value {
    if node.children.is_empty() && node.attrs.is_empty() {
        return Value::String(node.text.clone());
    }
    let mut map = Map::new();
    for (k, v) in &node.attrs {
        map.insert(k.clone(), Value::String(v.clone()));
    }
    for child in &node.children {
        let value = generic_value(child);
        match map.get_mut(&child.name) {
            Some(Value::Array(arr)) => arr.push(value),
            Some(existing) => {
                let prev = existing.take();
                *existing = Value::Array(vec![prev, value]);
            }
            None => {
                map.insert(child.name.clone(), value);
            }
        }
    }
    if !node.text.is_empty() {
        map.insert("#text".to_string(), Value::String(node.text.clone()));
    }
    Value::Object(map)
}

fn quantity_element_to_json(node: &XmlNode) -> Value {
    let mut map = Map::new();
    for child in &node.children {
        match child.local_name() {
            "quantity" => {
                if let Ok(num) = child.text.parse::<f64>()
                    && let Some(n) = serde_json::Number::from_f64(num)
                {
                    map.insert("quantity".to_string(), Value::Number(n));
                }
            }
            other => {
                map.insert(other.to_string(), Value::String(child.text.clone()));
            }
        }
    }
    Value::Object(map)
}

fn typed_pair_list_to_json(node: &XmlNode, value_key: &str) -> Value {
    // <bizTransaction type="...">text</bizTransaction> and the source /
    // destination equivalents become {"type": ..., value_key: ...} objects.
    let items = node
        .children
        .iter()
        .map(|child| {
            let mut map = Map::new();
            if let Some((_, t)) = child.attrs.iter().find(|(k, _)| k == "type") {
                map.insert("type".to_string(), Value::String(t.clone()));
            }
            map.insert(value_key.to_string(), Value::String(child.text.clone()));
            Value::Object(map)
        })
        .collect();
    Value::Array(items)
}

fn sensor_element_to_json(node: &XmlNode) -> Value {
    let mut map = Map::new();
    let mut reports = vec![];
    for child in &node.children {
        match child.local_name() {
            "sensorMetadata" => {
                map.insert("sensorMetadata".to_string(), attrs_to_object(child));
            }
            "sensorReport" => reports.push(attrs_to_object(child)),
            _ => {
                // Extension elements keep their prefixed name.
                map.insert(child.name.clone(), generic_value(child));
            }
        }
    }
    if !reports.is_empty() {
        map.insert("sensorReport".to_string(), Value::Array(reports));
    }
    Value::Object(map)
}

fn error_declaration_to_json(node: &XmlNode) -> Value {
    let mut map = Map::new();
    for child in &node.children {
        match child.local_name() {
            "declarationTime" | "reason" => {
                map.insert(
                    child.local_name().to_string(),
                    Value::String(child.text.clone()),
                );
            }
            "correctiveEventIDs" => {
                let ids = child
                    .children
                    .iter()
                    .map(|id| Value::String(id.text.clone()))
                    .collect();
                map.insert("correctiveEventIDs".to_string(), Value::Array(ids));
            }
            _ => {
                map.insert(child.name.clone(), generic_value(child));
            }
        }
    }
    Value::Object(map)
}

// One branch per EPCIS field shape; the dispatch reads clearest as a unit.
#[allow(clippy::too_many_lines)]
fn event_to_json(node: &XmlNode) -> Value {
    let mut map = Map::new();
    map.insert(
        "type".to_string(),
        Value::String(node.local_name().to_string()),
    );

    for child in &node.children {
        let local = child.local_name();
        let value = if EPC_LISTS.contains(&local) {
            Value::Array(
                child
                    .children
                    .iter()
                    .map(|epc| Value::String(epc.text.clone()))
                    .collect(),
            )
        } else if QUANTITY_LISTS.contains(&local) {
            Value::Array(
                child
                    .children
                    .iter()
                    .map(quantity_element_to_json)
                    .collect(),
            )
        } else if local == "bizTransactionList" {
            typed_pair_list_to_json(child, "bizTransaction")
        } else if local == "sourceList" {
            typed_pair_list_to_json(child, "source")
        } else if local == "destinationList" {
            typed_pair_list_to_json(child, "destination")
        } else if local == "sensorElementList" {
            Value::Array(child.children.iter().map(sensor_element_to_json).collect())
        } else if local == "persistentDisposition" {
            let mut pd = Map::new();
            for entry in &child.children {
                let key = entry.local_name().to_string();
                match pd.get_mut(&key) {
                    Some(Value::Array(arr)) => arr.push(Value::String(entry.text.clone())),
                    _ => {
                        pd.insert(key, Value::Array(vec![Value::String(entry.text.clone())]));
                    }
                }
            }
            Value::Object(pd)
        } else if local == "readPoint" || local == "bizLocation" {
            let mut loc = Map::new();
            for sub in &child.children {
                if sub.local_name() == "id" {
                    loc.insert("id".to_string(), Value::String(sub.text.clone()));
                } else {
                    // Spec permits extension elements inside readPoint /
                    // bizLocation alongside the id.
                    loc.insert(sub.name.clone(), generic_value(sub));
                }
            }
            Value::Object(loc)
        } else if local == "errorDeclaration" {
            error_declaration_to_json(child)
        } else if local == "ilmd" {
            generic_value(child)
        } else if child.name.contains(':') {
            // Foreign-namespace user extension: keep the prefixed name.
            map.insert(child.name.clone(), generic_value(child));
            continue;
        } else if child.children.is_empty() {
            // Standard scalar (eventTime, action, bizStep, transformationID…).
            Value::String(child.text.clone())
        } else {
            generic_value(child)
        };
        map.insert(local.to_string(), value);
    }

    Value::Object(map)
}

fn event_list_from_body(root: &XmlNode) -> Result<Vec<Value>, EpcisModelError> {
    let body = root
        .child("EPCISBody")
        .ok_or_else(|| EpcisModelError::InvalidXml("missing EPCISBody".to_string()))?;
    let event_list = body
        .child("EventList")
        .ok_or_else(|| EpcisModelError::InvalidXml("missing EventList".to_string()))?;
    Ok(event_list.children.iter().map(event_to_json).collect())
}

/// Converts a standard EPCIS 2.0 XML document into the JSON document shape
/// accepted by [`crate::EPCISDocument`]'s serde implementation.
pub(crate) fn epcis_xml_to_json(xml: &str) -> Result<Value, EpcisModelError> {
    let (root, namespaces) = parse_tree(xml)?;
    if root.local_name() != "EPCISDocument" {
        return Err(EpcisModelError::InvalidXml(format!(
            "expected EPCISDocument root, found {}",
            root.name
        )));
    }

    let mut doc = Map::new();
    let mut context_map = Map::new();
    for (prefix, uri) in &namespaces {
        if prefix != "epcis" && prefix != "xsi" {
            context_map.insert(prefix.clone(), Value::String(uri.clone()));
        }
    }
    let mut context = vec![Value::String(
        "https://ref.gs1.org/standards/epcis/2.0.0/epcis-context.jsonld".to_string(),
    )];
    if !context_map.is_empty() {
        context.push(Value::Object(context_map));
    }
    doc.insert("@context".to_string(), Value::Array(context));
    doc.insert(
        "type".to_string(),
        Value::String("EPCISDocument".to_string()),
    );

    let mut schema_version = "2.0".to_string();
    let mut creation_date = None;
    for (k, v) in &root.attrs {
        match k.as_str() {
            "schemaVersion" => schema_version.clone_from(v),
            "creationDate" => creation_date = Some(v.clone()),
            _ => {}
        }
    }
    doc.insert("schemaVersion".to_string(), Value::String(schema_version));
    doc.insert(
        "creationDate".to_string(),
        Value::String(creation_date.ok_or_else(|| {
            EpcisModelError::InvalidXml("missing creationDate attribute".to_string())
        })?),
    );

    // Hash-algorithm ignoreFields instructions live at the document root as a
    // foreign-namespace element listing field names as empty child elements.
    for child in &root.children {
        if child.local_name() == "ignoreFields" {
            let fields = child
                .children
                .iter()
                .map(|f| Value::String(f.name.clone()))
                .collect();
            doc.insert(child.name.clone(), Value::Array(fields));
        }
    }

    doc.insert(
        "epcisBody".to_string(),
        json!({"eventList": Value::Array(event_list_from_body(&root)?)}),
    );

    Ok(Value::Object(doc))
}

// ── JSON document shape → XML ───────────────────────────────────────────────

/// XSD child-element order per event type (fields absent from an event are
/// simply skipped; anything not listed is emitted afterwards in map order).
// A pure lookup table mirroring the XSD sequences; splitting would obscure it.
#[allow(clippy::too_many_lines)]
fn xsd_order(event_type: &str) -> &'static [&'static str] {
    const COMMON: [&str; 6] = [
        "eventTime",
        "recordTime",
        "eventTimeZoneOffset",
        "eventID",
        "errorDeclaration",
        "certificationInfo",
    ];
    let _ = COMMON; // documented above; concatenated per type below
    match event_type {
        "ObjectEvent" => &[
            "eventTime",
            "recordTime",
            "eventTimeZoneOffset",
            "eventID",
            "errorDeclaration",
            "certificationInfo",
            "epcList",
            "action",
            "bizStep",
            "disposition",
            "persistentDisposition",
            "readPoint",
            "bizLocation",
            "bizTransactionList",
            "quantityList",
            "sourceList",
            "destinationList",
            "sensorElementList",
            "ilmd",
        ],
        "AggregationEvent" => &[
            "eventTime",
            "recordTime",
            "eventTimeZoneOffset",
            "eventID",
            "errorDeclaration",
            "certificationInfo",
            "parentID",
            "childEPCs",
            "action",
            "bizStep",
            "disposition",
            "readPoint",
            "bizLocation",
            "bizTransactionList",
            "childQuantityList",
            "sourceList",
            "destinationList",
            "sensorElementList",
        ],
        "TransactionEvent" => &[
            "eventTime",
            "recordTime",
            "eventTimeZoneOffset",
            "eventID",
            "errorDeclaration",
            "certificationInfo",
            "bizTransactionList",
            "parentID",
            "epcList",
            "action",
            "bizStep",
            "disposition",
            "readPoint",
            "bizLocation",
            "quantityList",
            "sourceList",
            "destinationList",
            "sensorElementList",
        ],
        "TransformationEvent" => &[
            "eventTime",
            "recordTime",
            "eventTimeZoneOffset",
            "eventID",
            "errorDeclaration",
            "certificationInfo",
            "inputEPCList",
            "inputQuantityList",
            "outputEPCList",
            "outputQuantityList",
            "transformationID",
            "bizStep",
            "disposition",
            "persistentDisposition",
            "readPoint",
            "bizLocation",
            "bizTransactionList",
            "sourceList",
            "destinationList",
            "ilmd",
            "sensorElementList",
        ],
        "AssociationEvent" => &[
            "eventTime",
            "recordTime",
            "eventTimeZoneOffset",
            "eventID",
            "errorDeclaration",
            "certificationInfo",
            "parentID",
            "childEPCs",
            "childQuantityList",
            "action",
            "bizStep",
            "disposition",
            "readPoint",
            "bizLocation",
            "bizTransactionList",
            "sourceList",
            "destinationList",
            "sensorElementList",
        ],
        _ => &[
            "eventTime",
            "recordTime",
            "eventTimeZoneOffset",
            "eventID",
            "errorDeclaration",
            "certificationInfo",
        ],
    }
}

struct XmlWriter {
    out: String,
    depth: usize,
}

impl XmlWriter {
    fn indent(&mut self) {
        for _ in 0..self.depth {
            self.out.push_str("  ");
        }
    }

    fn escape(text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
    }

    fn leaf(&mut self, name: &str, text: &str) {
        self.indent();
        let _ = writeln!(self.out, "<{name}>{}</{name}>", Self::escape(text));
    }

    fn open(&mut self, name: &str, attrs: &[(String, String)]) {
        self.indent();
        self.out.push('<');
        self.out.push_str(name);
        for (k, v) in attrs {
            let _ = write!(self.out, " {k}=\"{}\"", Self::escape(v));
        }
        self.out.push_str(">\n");
        self.depth += 1;
    }

    fn close(&mut self, name: &str) {
        self.depth -= 1;
        self.indent();
        let _ = writeln!(self.out, "</{name}>");
    }

    fn empty_with_attrs(&mut self, name: &str, attrs: &[(String, String)]) {
        self.indent();
        self.out.push('<');
        self.out.push_str(name);
        for (k, v) in attrs {
            let _ = write!(self.out, " {k}=\"{}\"", Self::escape(v));
        }
        self.out.push_str("/>\n");
    }
}

fn scalar_to_text(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

fn object_attrs(map: &Map<String, Value>) -> Vec<(String, String)> {
    map.iter()
        .map(|(k, v)| (k.clone(), scalar_to_text(v)))
        .collect()
}

/// Writes generic (extension / ilmd) content.
fn write_generic(w: &mut XmlWriter, name: &str, value: &Value) {
    match value {
        Value::Object(map) => {
            let elements: Vec<(&String, &Value)> =
                map.iter().filter(|(k, _)| k.as_str() != "#text").collect();
            if elements.is_empty() {
                let text = map.get("#text").map(scalar_to_text).unwrap_or_default();
                w.leaf(name, &text);
            } else {
                w.open(name, &[]);
                if let Some(text) = map.get("#text") {
                    let escaped = XmlWriter::escape(&scalar_to_text(text));
                    w.indent();
                    w.out.push_str(&escaped);
                    w.out.push('\n');
                }
                for (k, v) in elements {
                    write_generic(w, k, v);
                }
                w.close(name);
            }
        }
        Value::Array(items) => {
            for item in items {
                write_generic(w, name, item);
            }
        }
        other => w.leaf(name, &scalar_to_text(other)),
    }
}

fn write_typed_pair_list(
    w: &mut XmlWriter,
    list_name: &str,
    item_name: &str,
    value_key: &str,
    items: &Value,
) {
    let Some(items) = items.as_array() else {
        return;
    };
    w.open(list_name, &[]);
    for item in items {
        if let Some(obj) = item.as_object() {
            let text = obj.get(value_key).map(scalar_to_text).unwrap_or_default();
            let attrs: Vec<(String, String)> = obj
                .get("type")
                .map(|t| vec![("type".to_string(), scalar_to_text(t))])
                .unwrap_or_default();
            w.indent();
            w.out.push('<');
            w.out.push_str(item_name);
            for (k, v) in &attrs {
                let _ = write!(w.out, " {k}=\"{}\"", XmlWriter::escape(v));
            }
            let _ = writeln!(w.out, ">{}</{item_name}>", XmlWriter::escape(&text));
        }
    }
    w.close(list_name);
}

// Mirror of event_to_json: one branch per EPCIS field shape.
#[allow(clippy::too_many_lines)]
fn write_event_field(w: &mut XmlWriter, key: &str, value: &Value) {
    if EPC_LISTS.contains(&key) {
        w.open(key, &[]);
        if let Some(arr) = value.as_array() {
            for epc in arr {
                w.leaf("epc", &scalar_to_text(epc));
            }
        }
        w.close(key);
    } else if QUANTITY_LISTS.contains(&key) {
        w.open(key, &[]);
        if let Some(arr) = value.as_array() {
            for qe in arr {
                if let Some(obj) = qe.as_object() {
                    w.open("quantityElement", &[]);
                    for field in ["epcClass", "quantity", "uom"] {
                        if let Some(v) = obj.get(field) {
                            w.leaf(field, &scalar_to_text(v));
                        }
                    }
                    w.close("quantityElement");
                }
            }
        }
        w.close(key);
    } else if key == "bizTransactionList" {
        write_typed_pair_list(w, key, "bizTransaction", "bizTransaction", value);
    } else if key == "sourceList" {
        write_typed_pair_list(w, key, "source", "source", value);
    } else if key == "destinationList" {
        write_typed_pair_list(w, key, "destination", "destination", value);
    } else if key == "sensorElementList" {
        w.open(key, &[]);
        if let Some(arr) = value.as_array() {
            for element in arr {
                if let Some(obj) = element.as_object() {
                    w.open("sensorElement", &[]);
                    if let Some(Value::Object(meta)) = obj.get("sensorMetadata") {
                        w.empty_with_attrs("sensorMetadata", &object_attrs(meta));
                    }
                    if let Some(Value::Array(reports)) = obj.get("sensorReport") {
                        for report in reports {
                            if let Some(rep) = report.as_object() {
                                w.empty_with_attrs("sensorReport", &object_attrs(rep));
                            }
                        }
                    }
                    for (k, v) in obj {
                        if k != "sensorMetadata" && k != "sensorReport" {
                            write_generic(w, k, v);
                        }
                    }
                    w.close("sensorElement");
                }
            }
        }
        w.close(key);
    } else if key == "persistentDisposition" {
        if let Some(obj) = value.as_object() {
            w.open(key, &[]);
            for entry in ["set", "unset"] {
                if let Some(Value::Array(values)) = obj.get(entry) {
                    for v in values {
                        w.leaf(entry, &scalar_to_text(v));
                    }
                }
            }
            w.close(key);
        }
    } else if key == "readPoint" || key == "bizLocation" {
        if let Some(obj) = value.as_object() {
            w.open(key, &[]);
            if let Some(id) = obj.get("id") {
                w.leaf("id", &scalar_to_text(id));
            }
            for (k, v) in obj {
                if k != "id" {
                    write_generic(w, k, v);
                }
            }
            w.close(key);
        }
    } else if key == "errorDeclaration" {
        if let Some(obj) = value.as_object() {
            w.open(key, &[]);
            for (k, v) in obj {
                if k == "correctiveEventIDs" {
                    if let Some(ids) = v.as_array() {
                        w.open(k, &[]);
                        for id in ids {
                            w.leaf("correctiveEventID", &scalar_to_text(id));
                        }
                        w.close(k);
                    }
                } else if k == "declarationTime" || k == "reason" {
                    w.leaf(k, &scalar_to_text(v));
                } else {
                    write_generic(w, k, v);
                }
            }
            w.close(key);
        }
    } else if key == "ilmd" {
        write_generic(w, key, value);
    } else {
        match value {
            Value::String(s) => w.leaf(key, s),
            Value::Number(_) | Value::Bool(_) => w.leaf(key, &scalar_to_text(value)),
            other => write_generic(w, key, other),
        }
    }
}

/// Serializes the JSON document shape into standard EPCIS 2.0 XML.
pub(crate) fn json_to_epcis_xml(doc: &Value) -> Result<String, EpcisModelError> {
    let obj = doc
        .as_object()
        .ok_or_else(|| EpcisModelError::InvalidXml("document is not an object".to_string()))?;

    let mut root_attrs = vec![
        (
            "xmlns:epcis".to_string(),
            "urn:epcglobal:epcis:xsd:2".to_string(),
        ),
        (
            "schemaVersion".to_string(),
            obj.get("schemaVersion")
                .map_or_else(|| "2.0".to_string(), scalar_to_text),
        ),
        (
            "creationDate".to_string(),
            obj.get("creationDate")
                .map(scalar_to_text)
                .ok_or_else(|| EpcisModelError::InvalidXml("missing creationDate".to_string()))?,
        ),
    ];

    // Re-emit prefix declarations recorded in the JSON-LD context so
    // prefixed extension elements stay resolvable.
    if let Some(Value::Array(context)) = obj.get("@context") {
        for entry in context {
            if let Some(map) = entry.as_object() {
                for (prefix, uri) in map {
                    if let Value::String(uri) = uri {
                        root_attrs.push((format!("xmlns:{prefix}"), uri.clone()));
                    }
                }
            }
        }
    }

    let events = obj
        .get("epcisBody")
        .and_then(|b| b.get("eventList"))
        .and_then(|l| l.as_array())
        .ok_or_else(|| EpcisModelError::InvalidXml("missing epcisBody.eventList".to_string()))?;

    let mut w = XmlWriter {
        out: String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n"),
        depth: 0,
    };
    w.open("epcis:EPCISDocument", &root_attrs);

    // Document-level extension elements (e.g. hash-algorithm ignoreFields
    // instructions) live directly under the root.
    for (key, value) in obj {
        if DOC_KEYS.contains(&key.as_str()) || key == "id" {
            continue;
        }
        if key.ends_with("ignoreFields") {
            // Field names are represented as empty child elements in XML.
            if let Some(names) = value.as_array() {
                w.open(key, &[]);
                for name in names {
                    w.empty_with_attrs(&scalar_to_text(name), &[]);
                }
                w.close(key);
            }
        } else {
            write_generic(&mut w, key, value);
        }
    }

    w.open("EPCISBody", &[]);
    w.open("EventList", &[]);

    for event in events {
        let Some(event_obj) = event.as_object() else {
            continue;
        };
        let event_type = event_obj
            .get("type")
            .map(scalar_to_text)
            .ok_or_else(|| EpcisModelError::InvalidXml("event missing type".to_string()))?;
        w.open(&event_type, &[]);

        let order = xsd_order(&event_type);
        for &field in order {
            if let Some(value) = event_obj.get(field) {
                write_event_field(&mut w, field, value);
            }
        }
        for (key, value) in event_obj {
            if key == "type" || order.contains(&key.as_str()) {
                continue;
            }
            write_event_field(&mut w, key, value);
        }

        w.close(&event_type);
    }

    w.close("EventList");
    w.close("EPCISBody");
    w.close("epcis:EPCISDocument");
    Ok(w.out)
}
