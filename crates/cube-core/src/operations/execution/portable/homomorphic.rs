//! Typed transport for public homomorphic ciphertext evaluation.
use super::bytes;
use crate::{
    asymmetric::homomorphic as h,
    errors::{ErrorCode, OperationError},
    operations::execution::{Budget, Elapsed},
};
use serde_json::{Value, json};

pub(super) fn supports(operation: &str) -> bool {
    h::Profile::from_operation(operation).is_some()
}

pub(super) fn execute(
    operation: &str,
    inputs: Value,
    parameters: Value,
    budget: Budget,
    _clock: Option<Elapsed<'_>>,
) -> Result<Value, OperationError> {
    let profile = h::Profile::from_operation(operation).ok_or_else(|| {
        OperationError::new(ErrorCode::InvalidParameter, "unknown homomorphic operation")
    })?;
    if budget.memory_bytes < h::SCRATCH_BYTES || budget.instructions < profile.bytes() as u64 {
        return Err(OperationError::new(
            ErrorCode::ResourceLimit,
            "homomorphic operation exceeds the invocation budget",
        ));
    }
    profile.validate_parameters(&parameters.to_string())?;
    let result = if profile == h::Profile::Tfhe {
        h::tfhe_not(&bytes(&inputs["ciphertext"])?)?
    } else {
        h::add(&bytes(&inputs["left"])?, &bytes(&inputs["right"])?, profile)?
    };
    Ok(json!({"ciphertext":{"base64":crate::encoding::base64_encode(&result)}}))
}
