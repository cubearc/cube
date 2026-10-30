//! Bounded JSON selection and exact integer composition; no expression evaluator.
mod contracts;
pub(crate) mod execution;
mod metadata;
use crate::errors::{ErrorCode, OperationError};
pub(crate) use metadata::MODULE;
use serde_json::Value;
use std::io::{self, Write};

pub use crate::pipeline::json::{MAX_DEPTH, MAX_DOCUMENT_BYTES, MAX_NODES};
/// Largest integer exactly representable by every supported JSON interface.
pub const MAX_INTEGER: i64 = 9_007_199_254_740_991;
/// Maximum UTF-8 byte length of a JSON Pointer.
pub const MAX_POINTER_BYTES: usize = 1024;

fn error(id: &'static str, code: ErrorCode, message: &'static str) -> OperationError {
    OperationError::new(code, message).operation(id)
}
struct Size(usize);
impl Write for Size {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0 = self
            .0
            .checked_add(bytes.len())
            .filter(|n| *n <= MAX_DOCUMENT_BYTES)
            .ok_or_else(|| io::Error::other("JSON byte budget"))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
/// Check constructed values before cloning/serializing a selected subtree.
pub fn validate_json(value: &Value) -> Result<(), OperationError> {
    let mut pending = vec![(value, 0u8)];
    let mut nodes = 0;
    let mut raw_bytes = 0usize;
    while let Some((value, depth)) = pending.pop() {
        nodes += 1;
        if value
            .as_f64()
            .is_some_and(|number| number.abs() > MAX_INTEGER as f64)
        {
            return Err(error(
                "json.select",
                ErrorCode::ResourceLimit,
                "JSON number exceeds the shared exact integer magnitude",
            ));
        }
        if depth > MAX_DEPTH || nodes > MAX_NODES {
            return Err(error(
                "json.select",
                ErrorCode::ResourceLimit,
                "JSON depth or node budget exceeded",
            ));
        }
        let count = match value {
            Value::Array(a) => a.len(),
            Value::Object(o) => o.len(),
            _ => 0,
        };
        if count > MAX_NODES - nodes - pending.len() {
            return Err(error(
                "json.select",
                ErrorCode::ResourceLimit,
                "JSON node budget exceeded",
            ));
        }
        match value {
            Value::Array(a) => pending.extend(a.iter().map(|v| (v, depth + 1))),
            Value::Object(o) => {
                for (key, child) in o {
                    raw_bytes = raw_bytes
                        .checked_add(key.len())
                        .filter(|n| *n <= MAX_DOCUMENT_BYTES)
                        .ok_or_else(|| {
                            error(
                                "json.select",
                                ErrorCode::ResourceLimit,
                                "JSON keys exceed the byte budget",
                            )
                        })?;
                    pending.push((child, depth + 1));
                }
            }
            Value::String(s) => {
                raw_bytes = raw_bytes
                    .checked_add(s.len())
                    .filter(|n| *n <= MAX_DOCUMENT_BYTES)
                    .ok_or_else(|| {
                        error(
                            "json.select",
                            ErrorCode::ResourceLimit,
                            "JSON text exceeds the byte budget",
                        )
                    })?;
            }
            _ => {}
        }
    }
    serde_json::to_writer(&mut Size(0), value).map_err(|_| {
        error(
            "json.select",
            ErrorCode::ResourceLimit,
            "JSON document exceeds 256 KiB",
        )
    })
}
/// Parse inert JSON through Core's duplicate-key/depth/node/byte-bounded parser.
pub fn parse_json(document: &str) -> Result<Value, OperationError> {
    let value =
        crate::pipeline::json::parse(document).map_err(|e| (*e.error).operation("json.parse"))?;
    validate_json(&value).map_err(|e| e.operation("json.parse"))?;
    Ok(value)
}
/// Select an RFC 6901 pointer without coercion. Empty pointer selects the root;
/// null exists, while an absent key/index fails. Invalid escapes/indices fail.
pub fn select_field(value: &Value, pointer: &str) -> Result<Value, OperationError> {
    validate_json(value)?;
    if pointer.len() > MAX_POINTER_BYTES {
        return Err(error(
            "json.select",
            ErrorCode::ResourceLimit,
            "field pointer exceeds 1024 UTF-8 bytes",
        ));
    }
    if !pointer.is_empty() && !pointer.starts_with('/') {
        return Err(error(
            "json.select",
            ErrorCode::InvalidParameter,
            "field pointer must be empty or begin with /",
        ));
    }
    for token in pointer.split('/').skip(1) {
        let mut bytes = token.bytes();
        while let Some(b) = bytes.next() {
            if b == b'~' && !matches!(bytes.next(), Some(b'0' | b'1')) {
                return Err(error(
                    "json.select",
                    ErrorCode::InvalidParameter,
                    "field pointer contains an invalid escape",
                ));
            }
        }
    }
    value.pointer(pointer).cloned().ok_or_else(|| {
        error(
            "json.select",
            ErrorCode::InvalidInput,
            "field or array index does not exist",
        )
    })
}
/// Return only an actual JSON string. Numbers, null, arrays and objects fail.
pub fn json_text(value: &Value) -> Result<String, OperationError> {
    validate_json(value).map_err(|e| e.operation("json.text"))?;
    value.as_str().map(str::to_owned).ok_or_else(|| {
        error(
            "json.text",
            ErrorCode::InvalidInput,
            "selected value must be text",
        )
    })
}
fn integer(value: i64, id: &'static str) -> Result<i64, OperationError> {
    if !(-MAX_INTEGER..=MAX_INTEGER).contains(&value) {
        return Err(error(
            id,
            ErrorCode::ResourceLimit,
            "integer exceeds the exact JSON integer range",
        ));
    }
    Ok(value)
}
/// Parse optional '-' and canonical ASCII decimal digits, with no spaces,
/// plus sign, leading zeros, decimal point or exponent. '-0' yields zero.
pub fn parse_integer(text: &str) -> Result<i64, OperationError> {
    const ID: &str = "text.integer";
    if text.len() > 17 {
        return Err(error(
            ID,
            ErrorCode::ResourceLimit,
            "integer text exceeds 17 ASCII bytes",
        ));
    }
    let digits = text.strip_prefix('-').unwrap_or(text);
    if digits.is_empty()
        || !digits.bytes().all(|b| b.is_ascii_digit())
        || (digits.len() > 1 && digits.starts_with('0'))
    {
        return Err(error(
            ID,
            ErrorCode::InvalidInput,
            "use canonical decimal integer text without spaces or an exponent",
        ));
    }
    integer(
        text.parse().map_err(|_| {
            error(
                ID,
                ErrorCode::ResourceLimit,
                "integer exceeds the exact JSON integer range",
            )
        })?,
        ID,
    )
}
/// Maximum of a nonempty bounded list of exact integers. Empty lists fail.
pub fn maximum(values: &[i64]) -> Result<i64, OperationError> {
    const ID: &str = "number.maximum";
    if values.len() > MAX_NODES {
        return Err(error(
            ID,
            ErrorCode::ResourceLimit,
            "integer list exceeds 4096 items",
        ));
    }
    let mut result = None;
    for &value in values {
        let value = integer(value, ID)?;
        result = Some(result.map_or(value, |previous: i64| previous.max(value)));
    }
    result.ok_or_else(|| {
        error(
            ID,
            ErrorCode::InvalidInput,
            "maximum requires at least one integer",
        )
    })
}
/// Add two exact signed JSON integers. Overflow fails before returning a value.
pub fn add(left: i64, right: i64) -> Result<i64, OperationError> {
    const ID: &str = "number.add";
    let left = integer(left, ID)?;
    let right = integer(right, ID)?;
    // Two validated 53-bit magnitudes fit in i64; the shared exact range still applies.
    integer(left + right, ID)
}
