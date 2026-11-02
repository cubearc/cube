//! Precisely configured historical mechanical ciphers. No modern security.
pub mod alberti;
mod alphabet;
mod catalog;
pub mod disks;
pub mod hagelin;
mod hagelin_catalog;
pub mod kryha;
pub mod lorenz;
mod lorenz_catalog;
mod stateful_catalog;
pub mod wheatstone;
use crate::errors::{ErrorCode, OperationError};
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;
pub(crate) use hagelin_catalog::MODULE as HAGELIN_MODULE;
pub use hagelin_catalog::OPERATIONS as HAGELIN_OPERATIONS;
pub(crate) use lorenz_catalog::MODULE as LORENZ_MODULE;
pub use lorenz_catalog::OPERATIONS as LORENZ_OPERATIONS;
use serde::{Serialize, de::DeserializeOwned};
pub(crate) use stateful_catalog::MODULE as STATEFUL_MODULE;
pub use stateful_catalog::OPERATIONS as STATEFUL_OPERATIONS;
pub const MAX_INPUT: usize = 1024 * 1024;
pub const MAX_PARAMETERS: usize = 512 * 1024;
pub const MAX_TRACE: usize = 65536;
pub const MAX_OUTPUT: usize = 4 * 1024 * 1024;

pub(crate) fn supports(operation: &str) -> bool {
    descriptor(operation).is_some()
}
fn descriptor(operation: &str) -> Option<&'static crate::operations::Descriptor> {
    OPERATIONS
        .iter()
        .chain(STATEFUL_OPERATIONS)
        .chain(LORENZ_OPERATIONS)
        .chain(HAGELIN_OPERATIONS)
        .find(|d| d.id == operation)
}
pub(crate) fn execute(
    operation: &str,
    inputs: &serde_json::Value,
    parameters: &serde_json::Value,
) -> Result<serde_json::Value, OperationError> {
    if HAGELIN_OPERATIONS.iter().any(|d| d.id == operation) {
        return hagelin_catalog::execute(operation, inputs, parameters);
    }
    if LORENZ_OPERATIONS.iter().any(|d| d.id == operation) {
        return lorenz_catalog::execute(operation, inputs, parameters);
    }
    if STATEFUL_OPERATIONS.iter().any(|d| d.id == operation) {
        stateful_catalog::execute(operation, inputs, parameters)
    } else {
        catalog::execute(operation, inputs, parameters)
    }
}
fn bounded_text(text: &str, trace: bool, op: &'static str) -> Result<(), OperationError> {
    text_budget(text, op)?;
    if trace && text.len() > MAX_TRACE {
        return Err(error(
            op,
            ErrorCode::ResourceLimit,
            "trace",
            "trace exceeds65536 input symbols",
        ));
    }
    Ok(())
}
fn finish_report<T: Serialize>(value: T, op: &'static str) -> Result<T, OperationError> {
    crate::ciphers::mono::homophonic::serialized_size(&value, MAX_OUTPUT, op)
        .map_err(|e| e.parameter("report"))?;
    Ok(value)
}
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
    if let Some(d) = LORENZ_OPERATIONS.iter().find(|d| d.id == operation) {
        if let Some(codes) = inputs.get("codes").and_then(serde_json::Value::as_array) {
            lorenz::input_budget(codes.len(), d.id)?;
        }
        if let Some(settings) = inputs.get("settings") {
            parameter_size(settings, d.id)?;
        }
    }
    if let Some(descriptor) = descriptor(operation)
        && let Some(text) = inputs.get("text").and_then(serde_json::Value::as_str)
    {
        text_budget(text, descriptor.id)?;
        if let Some(settings) = inputs.get("settings") {
            parameter_size(settings, descriptor.id)?;
        }
    }
    Ok(())
}
