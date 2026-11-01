//! Bounded, explicit historical polyalphabetic/polygraphic profiles.
//! No profile supplies modern confidentiality or authentication.
pub mod bellaso;
mod catalog;
mod fixtures;
pub mod geometry;
pub mod hill;
pub mod key_profiles;
pub mod schedules;
pub mod stateful;
pub mod tableaux;
#[cfg(test)]
mod tests;
use crate::errors::{ErrorCode, OperationError};
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;
use serde::{Serialize, de::DeserializeOwned};

pub const MAX_INPUT: usize = 1 << 20;
pub const MAX_KEY: usize = 1 << 16;
pub const MAX_SYMBOLS: usize = 1 << 16;
pub const MAX_PREPARED: usize = 2 << 20;
pub const MAX_PARAMETERS: usize = 2 << 20;
pub const MAX_REPORT: usize = 16 << 20;
pub const AZ: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
pub const IJ: &str = "ABCDEFGHIKLMNOPQRSTUVWXYZ";

pub(crate) fn invalid(
    op: &'static str,
    field: &'static str,
    message: &'static str,
) -> OperationError {
    OperationError::new(ErrorCode::InvalidParameter, message)
        .operation(op)
        .parameter(field)
}
pub(crate) fn input_error(op: &'static str, offset: usize) -> OperationError {
    OperationError::new(
        ErrorCode::InvalidInput,
        "input is not in the declared canonical alphabet",
    )
    .operation(op)
    .parameter("text")
    .offset(offset as u64)
}
pub(crate) fn bound(
    op: &'static str,
    field: &'static str,
    n: usize,
    max: usize,
) -> Result<(), OperationError> {
    if n > max {
        Err(OperationError::new(
            ErrorCode::ResourceLimit,
            "value exceeds the declared byte/item/work ceiling",
        )
        .operation(op)
        .parameter(field))
    } else {
        Ok(())
    }
}
pub(crate) fn json_bound<T: Serialize>(
    value: &T,
    max: usize,
    op: &'static str,
) -> Result<(), OperationError> {
    let mut writer = crate::text::mapping::ByteBudget::new(max);
    serde_json::to_writer(&mut writer, value).map_err(|_| {
        OperationError::new(
            ErrorCode::ResourceLimit,
            "serialized definition/result exceeds its byte budget",
        )
        .operation(op)
    })
}
/// Strict bounded transport; duplicate keys, excessive nesting and unknown DTO
/// fields reject. It never retrieves references or executes user expressions.
pub fn parse_parameters<T: DeserializeOwned>(document: &str) -> Result<T, OperationError> {
    let value = crate::pipeline::json::parse_with_limits(document, MAX_PARAMETERS, 8, 200000)
        .map_err(|e| {
            OperationError::new(e.error.code, "invalid bounded parameter JSON")
                .operation("cipher.poly.parameters")
                .parameter("parameters")
        })?;
    serde_json::from_value(value).map_err(|_| {
        invalid(
            "cipher.poly.parameters",
            "parameters",
            "use the exact typed parameter fields",
        )
    })
}
pub(crate) fn letters(
    text: &str,
    max: usize,
    op: &'static str,
    merge_j: bool,
) -> Result<Vec<u8>, OperationError> {
    bound(op, "text", text.len(), max)?;
    text.bytes()
        .enumerate()
        .map(|(i, c)| {
            if c.is_ascii_uppercase() {
                Ok(if merge_j && c == b'J' { b'I' } else { c })
            } else {
                Err(input_error(op, i))
            }
        })
        .collect()
}
pub(crate) fn letter_key(
    key: &str,
    max: usize,
    empty: bool,
    op: &'static str,
) -> Result<Vec<u8>, OperationError> {
    bound(op, "key", key.len(), max)?;
    if (!empty && key.is_empty()) || !key.bytes().all(|b| b.is_ascii_alphabetic()) {
        return Err(invalid(
            op,
            "key",
            "key must contain only ASCII letters, with the declared nonempty policy",
        ));
    }
    Ok(key.bytes().map(|b| b.to_ascii_uppercase() - b'A').collect())
}
pub(crate) fn permutation(
    text: &str,
    base: &str,
    op: &'static str,
    field: &'static str,
) -> Result<Vec<u8>, OperationError> {
    crate::ciphers::mono::alphabet::Mapping::new(base, text).map_err(|_| {
        invalid(
            op,
            field,
            "alphabet must be an exact unique permutation of the required symbols",
        )
    })?;
    if text.len() != base.len() || !text.bytes().all(|b| base.as_bytes().contains(&b)) {
        return Err(invalid(
            op,
            field,
            "alphabet must be an exact permutation of the required ASCII alphabet",
        ));
    }
    Ok(text.bytes().collect())
}
pub(crate) fn preserve<F>(text: &str, op: &'static str, mut f: F) -> Result<String, OperationError>
where
    F: FnMut(usize, u8) -> u8,
{
    bound(op, "text", text.len(), MAX_INPUT)?;
    let mut i = 0;
    Ok(text
        .chars()
        .map(|c| {
            if c.is_ascii_alphabetic() {
                let base = if c.is_ascii_lowercase() { b'a' } else { b'A' };
                let out = char::from(base + f(i, c as u8 - base));
                i += 1;
                out
            } else {
                c
            }
        })
        .collect())
}
