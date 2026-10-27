//! Exact bounded monoalphabetic mappings. No normalization or hidden encoder state.
pub mod alphabet;
pub mod checkerboard;
pub mod grandpre;
pub mod homophonic;
pub mod params;
pub mod rotations;
use crate::errors::{ErrorCode, OperationError};
pub const MAX_INPUT: usize = 1024 * 1024;
pub const MAX_DEFINITION: usize = 65536;
pub const MAX_SYMBOLS: usize = 4096;
pub const MAX_TOKENS: usize = 65536;
pub const MAX_OUTPUT: usize = 4 * MAX_INPUT;
pub(crate) fn error(
    op: &'static str,
    code: ErrorCode,
    parameter: &'static str,
    message: &'static str,
) -> OperationError {
    OperationError::new(code, message)
        .operation(op)
        .parameter(parameter)
}
pub(crate) fn bound(
    op: &'static str,
    name: &'static str,
    size: usize,
    maximum: usize,
) -> Result<(), OperationError> {
    if size > maximum {
        Err(error(
            op,
            ErrorCode::ResourceLimit,
            name,
            "value exceeds the documented byte/item budget",
        ))
    } else {
        Ok(())
    }
}
pub(crate) fn policy(op: &'static str, name: &str) -> Result<bool, OperationError> {
    match name {
        "reject" => Ok(false),
        "preserve" => Ok(true),
        _ => Err(error(
            op,
            ErrorCode::InvalidParameter,
            "unmapped",
            "unmapped must be reject or preserve",
        )),
    }
}

mod catalog;
mod fixtures;
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;

fn parse_data<T: serde::de::DeserializeOwned>(
    document: &str,
    maximum: usize,
    nodes: usize,
    op: &'static str,
    parameter: &'static str,
) -> Result<T, OperationError> {
    let value =
        crate::pipeline::json::parse_with_limits(document, maximum, 6, nodes).map_err(|e| {
            error(
                op,
                e.error.code,
                parameter,
                "invalid bounded JSON transport",
            )
        })?;
    serde_json::from_value(value).map_err(|_| {
        error(
            op,
            ErrorCode::InvalidInput,
            parameter,
            "invalid typed JSON fields",
        )
    })
}
/// Strict token-array JSON, <=1 MiB/65536 tokens; no concatenated-token guessing.
pub fn symbol_tokens(document: &str) -> Result<Vec<String>, OperationError> {
    let op = "cipher.homophonic.decode";
    let tokens: Vec<String> = parse_data(document, MAX_INPUT, MAX_TOKENS + 2, op, "tokens")?;
    bound(op, "tokens", tokens.len(), MAX_TOKENS)?;
    Ok(tokens)
}
/// Strict coordinate/null-array JSON, <=1 MiB/65536 items; no numeric coercion.
pub fn coordinate_tokens(document: &str) -> Result<Vec<grandpre::Token>, OperationError> {
    let op = "cipher.grandpre.decode";
    let tokens: Vec<grandpre::Token> =
        parse_data(document, MAX_INPUT, MAX_TOKENS * 3 + 2, op, "tokens")?;
    bound(op, "tokens", tokens.len(), MAX_TOKENS)?;
    Ok(tokens)
}
/// Strict canonical starting-index records, <=64 KiB/4096 records.
pub fn starting_counters(document: &str) -> Result<Vec<homophonic::Counter>, OperationError> {
    let op = "cipher.homophonic.encode";
    let counters: Vec<homophonic::Counter> =
        parse_data(document, MAX_DEFINITION, 32768, op, "counters")?;
    bound(op, "counters", counters.len(), MAX_SYMBOLS)?;
    Ok(counters)
}
