//! Typed transport for public additive ciphertext evaluation.
use super::bytes;
use crate::{
    asymmetric::additive as a,
    errors::{ErrorCode, OperationError},
    operations::execution::{Budget, Elapsed},
};
use serde_json::{Value, json};

pub(super) fn supports(operation: &str) -> bool {
    a::Profile::from_operation(operation).is_some()
}

pub(super) fn execute(
    operation: &str,
    inputs: Value,
    parameters: Value,
    budget: Budget,
    _clock: Option<Elapsed<'_>>,
) -> Result<Value, OperationError> {
    let profile = a::Profile::from_operation(operation).ok_or_else(|| {
        OperationError::new(ErrorCode::InvalidParameter, "unknown additive operation")
    })?;
    if budget.memory_bytes < a::SCRATCH_BYTES || budget.instructions < 1 {
        return Err(OperationError::new(
            ErrorCode::ResourceLimit,
            "additive ciphertext evaluation exceeds the invocation budget",
        ));
    }
    profile.validate_parameters(&parameters.to_string())?;
    let output = a::add(
        &bytes(&inputs["left"])?,
        &bytes(&inputs["right"])?,
        &bytes(&inputs["public_modulus"])?,
        profile,
    )?;
    Ok(json!({"ciphertext":{"base64":crate::encoding::base64_encode(&output)}}))
}
