//! Precisely configured historical mechanical ciphers. No modern security.
pub mod alberti;
mod alphabet;
mod catalog;
pub mod disks;
pub mod hagelin;
mod hagelin_catalog;
pub mod kl7;
mod kl7_catalog;
mod kl7_data;
pub mod kryha;
pub mod lorenz;
mod lorenz_catalog;
pub mod nema;
mod nema_catalog;
mod nema_data;
mod rotor_contacts;
pub mod sigaba;
mod sigaba_catalog;
mod sigaba_data;
mod stateful_catalog;
pub mod t52ab;
mod t52ab_catalog;
pub mod typex;
mod typex_catalog;
pub mod wheatstone;
use crate::errors::{ErrorCode, OperationError};
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;
pub(crate) use hagelin_catalog::MODULE as HAGELIN_MODULE;
pub use hagelin_catalog::OPERATIONS as HAGELIN_OPERATIONS;
pub(crate) use kl7_catalog::MODULE as KL7_MODULE;
pub use kl7_catalog::OPERATIONS as KL7_OPERATIONS;
pub(crate) use lorenz_catalog::MODULE as LORENZ_MODULE;
pub use lorenz_catalog::OPERATIONS as LORENZ_OPERATIONS;
pub(crate) use nema_catalog::MODULE as NEMA_MODULE;
pub use nema_catalog::OPERATIONS as NEMA_OPERATIONS;
use serde::{Serialize, de::DeserializeOwned};
pub(crate) use sigaba_catalog::MODULE as SIGABA_MODULE;
pub use sigaba_catalog::OPERATIONS as SIGABA_OPERATIONS;
pub(crate) use stateful_catalog::MODULE as STATEFUL_MODULE;
pub use stateful_catalog::OPERATIONS as STATEFUL_OPERATIONS;
pub(crate) use t52ab_catalog::MODULE as T52AB_MODULE;
pub use t52ab_catalog::OPERATIONS as T52AB_OPERATIONS;
pub(crate) use typex_catalog::MODULE as TYPEX_MODULE;
pub use typex_catalog::OPERATIONS as TYPEX_OPERATIONS;
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
        .chain(T52AB_OPERATIONS)
        .chain(NEMA_OPERATIONS)
        .chain(TYPEX_OPERATIONS)
        .chain(SIGABA_OPERATIONS)
        .chain(KL7_OPERATIONS)
        .find(|d| d.id == operation)
}
pub(crate) fn execute(
    operation: &str,
    inputs: &serde_json::Value,
    parameters: &serde_json::Value,
) -> Result<serde_json::Value, OperationError> {
    if SIGABA_OPERATIONS.iter().any(|d| d.id == operation) {
        return sigaba_catalog::execute(operation, inputs, parameters);
    }
    if KL7_OPERATIONS.iter().any(|d| d.id == operation) {
        return kl7_catalog::execute(operation, inputs, parameters);
    }
    if TYPEX_OPERATIONS.iter().any(|d| d.id == operation) {
        return typex_catalog::execute(operation, inputs, parameters);
    }
    if NEMA_OPERATIONS.iter().any(|d| d.id == operation) {
        return nema_catalog::execute(operation, inputs, parameters);
    }
    if T52AB_OPERATIONS.iter().any(|d| d.id == operation) {
        return t52ab_catalog::execute(operation, inputs, parameters);
    }
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
    if let Some(d) = LORENZ_OPERATIONS
        .iter()
        .chain(T52AB_OPERATIONS)
        .find(|d| d.id == operation)
    {
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
