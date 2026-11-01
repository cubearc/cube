//! Restore only schema-annotated integer fields after canonical validation.
use crate::{
    errors::{ErrorCode, OperationError},
    operations::contracts::{self, PortContract},
};
use serde_json::{Value, json};
fn invalid() -> OperationError {
    OperationError::new(
        ErrorCode::InvalidInput,
        "canonical value cannot be represented by the typed binding",
    )
}
fn rebase(value: &mut Value) {
    match value {
        Value::Object(fields) => {
            for (key, value) in fields {
                if key == "$ref" {
                    if let Some(suffix) = value.as_str().and_then(|r| r.strip_prefix('#')) {
                        *value = json!(format!("#/$defs/root{suffix}"));
                    }
                } else {
                    rebase(value)
                }
            }
        }
        Value::Array(values) => {
            for value in values {
                rebase(value)
            }
        }
        _ => (),
    }
}
fn matches(branch: &Value, root: &Value, value: &Value) -> Result<bool, OperationError> {
    let mut branch = branch.clone();
    let mut root = root.clone();
    rebase(&mut branch);
    rebase(&mut root);
    let schema = serde_json::from_value(
        json!({"$ref":"#/$defs/branch","$defs":{"root":root,"branch":branch}}),
    )
    .map_err(|_| invalid())?;
    Ok(contracts::validate_value(&schema, value).is_ok())
}
fn restore(
    value: Value,
    schema: &Value,
    root: &Value,
    depth: usize,
) -> Result<Value, OperationError> {
    if depth > 128 {
        return Err(OperationError::new(
            ErrorCode::ResourceLimit,
            "schema traversal exceeds the binding budget",
        ));
    }
    // Already validated scalars cannot contain nested decimal integer fields.
    if matches!(value, Value::Null | Value::Bool(_) | Value::Number(_)) {
        return Ok(value);
    }
    if schema.get("x-cube-integer").is_some() {
        return match value {
            Value::String(text) => {
                if text.starts_with('-') {
                    text.parse::<i64>().map(|n| json!(n)).map_err(|_| invalid())
                } else {
                    text.parse::<u64>().map(|n| json!(n)).map_err(|_| invalid())
                }
            }
            value => Ok(value),
        };
    }
    if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
        let target = reference
            .strip_prefix('#')
            .and_then(|p| root.pointer(p))
            .ok_or_else(invalid)?;
        return restore(value, target, root, depth + 1);
    }
    if let Some(branches) = schema
        .get("anyOf")
        .or_else(|| schema.get("oneOf"))
        .and_then(Value::as_array)
    {
        for branch in branches {
            if matches(branch, root, &value)? {
                return restore(value, branch, root, depth + 1);
            }
        }
        return Err(invalid());
    }
    if let Some(branches) = schema.get("allOf").and_then(Value::as_array) {
        return branches
            .iter()
            .try_fold(value, |v, s| restore(v, s, root, depth + 1));
    }
    match value {
        Value::Object(fields) => fields
            .into_iter()
            .map(|(key, value)| {
                let child = schema["properties"]
                    .get(&key)
                    .or_else(|| schema.get("additionalProperties"))
                    .unwrap_or(&Value::Null);
                restore(value, child, root, depth + 1).map(|v| (key, v))
            })
            .collect::<Result<serde_json::Map<_, _>, _>>()
            .map(Value::Object),
        Value::Array(values) => values
            .into_iter()
            .enumerate()
            .map(|(index, value)| {
                let child = schema["prefixItems"]
                    .get(index)
                    .or_else(|| schema.get("items"))
                    .unwrap_or(&Value::Null);
                restore(value, child, root, depth + 1)
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Value::Array),
        value => Ok(value),
    }
}
pub(super) fn parameters(value: Value, schema: &Value) -> Result<Value, OperationError> {
    restore(value, schema, schema, 0)
}
pub(super) fn inputs(value: Value, ports: &[PortContract]) -> Result<Value, OperationError> {
    let mut fields = value.as_object().ok_or_else(invalid)?.clone();
    for port in ports {
        if let Some(value) = fields.remove(&port.name) {
            fields.insert(
                port.name.clone(),
                parameters(value, port.schema.as_value())?,
            );
        }
    }
    Ok(Value::Object(fields))
}
