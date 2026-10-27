//! Lossless JSON transport: unsafe JavaScript integers are canonical decimal
//! strings. Standard validation handles shape; one scalar keyword preserves the
//! original Rust integer bounds. HTTP and filesystem reference resolvers are off.
use super::*;
use jsonschema::{Keyword, ValidationError};
const SAFE: i64 = 9_007_199_254_740_991;

pub(super) fn schema(schema: Schema) -> Schema {
    fn visit(value: &mut Value) {
        let Value::Object(fields) = value else {
            if let Value::Array(items) = value {
                for item in items {
                    visit(item);
                }
            }
            return;
        };
        if fields.contains_key("x-cube-integer") {
            return;
        }
        for child in fields.values_mut() {
            visit(child);
        }
        if fields.get("type").and_then(Value::as_str) != Some("integer") {
            return;
        }
        let format = fields
            .get("format")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();
        let range = match format.as_str() {
            "int8" => Some((i8::MIN as i64, i8::MAX as i64)),
            "int16" => Some((i16::MIN as i64, i16::MAX as i64)),
            "int32" => Some((i32::MIN as i64, i32::MAX as i64)),
            "uint8" => Some((0, u8::MAX as i64)),
            "uint16" => Some((0, u16::MAX as i64)),
            "uint32" => Some((0, u32::MAX as i64)),
            _ => None,
        };
        if let Some((min, max)) = range {
            fields.entry("minimum").or_insert(json!(min));
            fields.entry("maximum").or_insert(json!(max));
        }
        let wide = matches!(
            format.as_str(),
            "int64" | "uint64" | "int" | "uint" | "int128" | "uint128"
        ) || fields
            .get("maximum")
            .and_then(Value::as_u64)
            .is_some_and(|v| v > SAFE as u64)
            || fields
                .get("minimum")
                .and_then(Value::as_i64)
                .is_some_and(|v| v < -SAFE);
        if !wide {
            return;
        }
        let mut native = Value::Object(fields.clone());
        let (minimum, maximum) = match format.as_str() {
            "uint64" | "uint" => ("0".to_string(), u64::MAX.to_string()),
            "int64" | "int" => (i64::MIN.to_string(), i64::MAX.to_string()),
            "uint128" => ("0".to_string(), u128::MAX.to_string()),
            "int128" => (i128::MIN.to_string(), i128::MAX.to_string()),
            _ => (
                fields
                    .get("minimum")
                    .map(ToString::to_string)
                    .unwrap_or_else(|| i64::MIN.to_string()),
                fields
                    .get("maximum")
                    .map(ToString::to_string)
                    .unwrap_or_else(|| u64::MAX.to_string()),
            ),
        };
        native["x-cube-minimum-decimal"] = json!(minimum);
        native["x-cube-maximum-decimal"] = json!(maximum);
        let mut numeric = native.clone();
        let min = numeric
            .get("minimum")
            .and_then(Value::as_i64)
            .unwrap_or(-SAFE)
            .max(-SAFE);
        let max = numeric
            .get("maximum")
            .and_then(Value::as_i64)
            .unwrap_or(SAFE)
            .min(SAFE);
        numeric["minimum"] = json!(min);
        numeric["maximum"] = json!(max);
        let mut text = json!({"type":"string","pattern":"^(0|-?[1-9][0-9]*)$","maxLength":40,"x-cube-native-integer":native});
        if let Some(default) = fields.get("default") {
            numeric["default"] = canonical(default.clone());
        }
        if let Some(choices) = fields.get("enum").and_then(Value::as_array) {
            text["enum"] = json!(choices.iter().map(|v| v.to_string()).collect::<Vec<_>>());
        }
        let mut wire = json!({"anyOf":[numeric,text],"x-cube-integer":native});
        for name in [
            "title",
            "description",
            "writeOnly",
            "readOnly",
            "deprecated",
        ] {
            if let Some(annotation) = fields.get(name) {
                wire[name] = annotation.clone();
            }
        }
        if let Some(default) = fields.get("default") {
            wire["default"] = canonical(default.clone());
        }
        *value = wire;
    }
    let mut value = schema.to_value();
    visit(&mut value);
    Schema::try_from(value).expect("schema remains an object")
}
pub fn canonical(value: Value) -> Value {
    match value {
        Value::Number(n)
            if n.as_u64().is_some_and(|n| n > SAFE as u64)
                || n.as_i64().is_some_and(|n| n < -SAFE) =>
        {
            json!(n.to_string())
        }
        Value::Array(items) => Value::Array(items.into_iter().map(canonical).collect()),
        Value::Object(fields) => {
            Value::Object(fields.into_iter().map(|(k, v)| (k, canonical(v))).collect())
        }
        value => value,
    }
}
struct NativeInteger {
    validator: jsonschema::Validator,
    format: String,
}
impl<'a> Keyword<'a> for NativeInteger {
    fn is_valid(&self, value: &'a Value) -> bool {
        let Some(text) = value.as_str() else {
            return false;
        };
        if self.format == "uint128" {
            return text.parse::<u128>().is_ok();
        }
        if self.format == "int128" {
            return text.parse::<i128>().is_ok();
        }
        let parsed = if text.starts_with('-') {
            text.parse::<i64>().map(|v| json!(v)).ok()
        } else {
            text.parse::<u64>().map(|v| json!(v)).ok()
        };
        parsed.is_some_and(|v| self.validator.is_valid(&v))
    }
    fn validate(&self, value: &'a Value) -> Result<(), ValidationError<'a>> {
        if self.is_valid(value) {
            Ok(())
        } else {
            Err(ValidationError::custom(
                "canonical integer violates its Rust contract",
            ))
        }
    }
}
struct Utf8Bytes(usize);
impl<'a> Keyword<'a> for Utf8Bytes {
    fn is_valid(&self, value: &'a Value) -> bool {
        value.as_str().is_none_or(|s| s.len() <= self.0)
    }
    fn validate(&self, value: &'a Value) -> Result<(), ValidationError<'a>> {
        if self.is_valid(value) {
            Ok(())
        } else {
            Err(ValidationError::custom("UTF-8 byte ceiling exceeded"))
        }
    }
}
pub(super) fn validator(schema: &Schema) -> Result<jsonschema::Validator, OperationError> {
    jsonschema::options()
        .with_keyword("x-cube-native-integer", |_, value, _| {
            let validator = jsonschema::validator_for(value)
                .map_err(|_| ValidationError::schema("invalid native integer contract"))?;
            Ok(Box::new(NativeInteger {
                validator,
                format: value
                    .get("format")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .into(),
            }) as Box<dyn for<'i> Keyword<'i>>)
        })
        .with_keyword("x-cube-max-utf8-bytes", |_, value, _| {
            let limit = value
                .as_u64()
                .and_then(|n| usize::try_from(n).ok())
                .filter(|n| *n <= MAX_INSTANCE_BYTES)
                .ok_or_else(|| ValidationError::schema("invalid UTF-8 byte ceiling"))?;
            Ok(Box::new(Utf8Bytes(limit)) as Box<dyn for<'i> Keyword<'i>>)
        })
        .build(schema.as_value())
        .map_err(|_| invalid("invalid or unresolved contract schema"))
}
