//! Precisely configured historical mechanical ciphers. No modern security.
mod catalog;
pub mod disks;
use crate::errors::{ErrorCode, OperationError};
pub use catalog::OPERATIONS;
pub(crate) use catalog::{MODULE, execute};
use serde::{Serialize, de::DeserializeOwned};
pub const MAX_INPUT: usize = 1024 * 1024;
pub const MAX_PARAMETERS: usize = 512 * 1024;
fn error(
    op: &'static str,
    code: ErrorCode,
    field: &'static str,
    message: &'static str,
) -> OperationError {
    OperationError::new(code, message)
        .operation(op)
        .parameter(field)
}
/// Reject duplicate/unknown fields, excessive nesting and unsupported profiles
/// before interpreting bounded settings in the CLI or native SDK.
pub fn parse_parameters<P: DeserializeOwned>(
    document: &str,
    op: &'static str,
) -> Result<P, OperationError> {
    let value = crate::pipeline::json::parse_with_limits(document, MAX_PARAMETERS, 12, 131072)
        .map_err(|e| {
            error(
                op,
                e.error.code,
                "parameters",
                "invalid bounded machine settings",
            )
        })?;
    serde_json::from_value(value).map_err(|_| {
        error(
            op,
            ErrorCode::InvalidParameter,
            "parameters",
            "use exact typed machine settings",
        )
    })
}
fn parameter_size(parameters: &impl Serialize, op: &'static str) -> Result<(), OperationError> {
    crate::ciphers::mono::homophonic::serialized_size(parameters, MAX_PARAMETERS, op)
}
fn text_budget(text: &str, op: &'static str) -> Result<(), OperationError> {
    if text.len() > MAX_INPUT {
        Err(error(
            op,
            ErrorCode::ResourceLimit,
            "text",
            "text exceeds 1 MiB",
        ))
    } else {
        Ok(())
    }
}
/// Preserve the owning operation's resource error before generic schema
/// validation can classify an oversized, otherwise typed string as bad input.
pub(crate) fn preflight(operation: &str, inputs: &serde_json::Value) -> Result<(), OperationError> {
    if let Some(descriptor) = OPERATIONS.iter().find(|d| d.id == operation)
        && let Some(text) = inputs.get("text").and_then(serde_json::Value::as_str)
    {
        text_budget(text, descriptor.id)?;
        if let Some(settings) = inputs.get("settings") {
            parameter_size(settings, descriptor.id)?;
        }
    }
    Ok(())
}
