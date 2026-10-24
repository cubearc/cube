//! Bounded non-executable JSON transport, rejecting duplicate keys before schema parsing.
use super::{Error, invalid};
use crate::errors::{ErrorCode, OperationError};
use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde_json::Value;
/// Shared document, nesting and node limits apply before schema/operation parsing.
pub const MAX_DOCUMENT_BYTES: usize = 256 * 1024;
pub const MAX_DEPTH: u8 = 16;
pub const MAX_NODES: usize = 4096;
struct Seed<'a> {
    depth: u8,
    nodes: &'a mut usize,
}
impl<'de> DeserializeSeed<'de> for Seed<'_> {
    type Value = Value;
    fn deserialize<D: de::Deserializer<'de>>(self, d: D) -> Result<Value, D::Error> {
        *self.nodes += 1;
        if self.depth > MAX_DEPTH || *self.nodes > MAX_NODES {
            return Err(de::Error::custom("JSON budget exceeded"));
        }
        d.deserialize_any(self)
    }
}
impl<'de> Visitor<'de> for Seed<'_> {
    type Value = Value;
    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("bounded JSON without duplicate keys")
    }
    fn visit_bool<E: de::Error>(self, v: bool) -> Result<Value, E> {
        Ok(Value::Bool(v))
    }
    fn visit_i64<E: de::Error>(self, v: i64) -> Result<Value, E> {
        Ok(Value::from(v))
    }
    fn visit_u64<E: de::Error>(self, v: u64) -> Result<Value, E> {
        Ok(Value::from(v))
    }
    fn visit_f64<E: de::Error>(self, v: f64) -> Result<Value, E> {
        serde_json::Number::from_f64(v)
            .map(Value::Number)
            .ok_or_else(|| de::Error::custom("finite JSON number required"))
    }
    fn visit_str<E: de::Error>(self, v: &str) -> Result<Value, E> {
        Ok(Value::String(v.to_owned()))
    }
    fn visit_string<E: de::Error>(self, v: String) -> Result<Value, E> {
        Ok(Value::String(v))
    }
    fn visit_none<E: de::Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }
    fn visit_unit<E: de::Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Value, A::Error> {
        let mut values = Vec::new();
        while let Some(value) = seq.next_element_seed(Seed {
            depth: self.depth + 1,
            nodes: self.nodes,
        })? {
            values.push(value);
        }
        Ok(Value::Array(values))
    }
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Value, A::Error> {
        let mut values = serde_json::Map::new();
        while let Some(key) = map.next_key::<String>()? {
            if values.contains_key(&key) {
                return Err(de::Error::custom("duplicate JSON object key"));
            }
            let value = map.next_value_seed(Seed {
                depth: self.depth + 1,
                nodes: self.nodes,
            })?;
            values.insert(key, value);
        }
        Ok(Value::Object(values))
    }
}
pub fn parse(document: &str) -> Result<Value, Error> {
    if document.len() > MAX_DOCUMENT_BYTES {
        return Err(Error::new(
            ErrorCode::ResourceLimit,
            "JSON document exceeds 256 KiB",
        ));
    }
    let mut deserializer = serde_json::Deserializer::from_str(document);
    let mut nodes = 0;
    let value=Seed{depth:0,nodes:&mut nodes}.deserialize(&mut deserializer).map_err(|e|{
        let code=if e.to_string().starts_with("JSON budget exceeded"){ErrorCode::ResourceLimit}else{ErrorCode::InvalidInput};
        Error::from_operation(OperationError::new(code,format!("invalid JSON at line {}, column {} (duplicate keys, invalid syntax and excessive nesting/nodes are rejected)",e.line(),e.column())).caused_by(e)).parameter("document")
    })?;
    deserializer.end().map_err(|e| {
        Error::from_operation(
            OperationError::new(
                ErrorCode::InvalidInput,
                "unexpected data after JSON document",
            )
            .caused_by(e),
        )
    })?;
    Ok(value)
}
pub(super) fn schema<T: serde::de::DeserializeOwned>(
    value: Value,
    field: &str,
) -> Result<T, Error> {
    serde_json::from_value(value).map_err(|e|Error::from_operation(OperationError::new(ErrorCode::InvalidInput,"invalid schema: check required/unknown fields, scalar types and explicit base64 tags").caused_by(e)).parameter(field))
}
pub fn value(document: &str) -> Result<super::Value, Error> {
    let value: super::Value = schema(parse(document)?, "value")?;
    value.validate(super::MAX_VALUE_BYTES)?;
    Ok(value)
}
pub fn steps(document: &str) -> Result<Vec<super::Step>, Error> {
    let raw = parse(document)?;
    let raw = raw
        .as_array()
        .ok_or_else(|| invalid("steps must be a JSON array"))?;
    if raw.len() > super::MAX_STEPS {
        return Err(Error::new(
            ErrorCode::ResourceLimit,
            "pipeline exceeds 64 steps",
        ));
    }
    raw.iter()
        .enumerate()
        .map(|(i, value)| {
            schema(value.clone(), "steps")
                .map_err(|e| e.at(i, value.get("operation").and_then(Value::as_str)))
        })
        .collect()
}
pub fn limits(document: &str) -> Result<super::Limits, Error> {
    let limits: super::Limits = schema(parse(document)?, "limits")?;
    limits.validate()?;
    Ok(limits)
}
