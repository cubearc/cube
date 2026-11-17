//! Typed named byte ports for the educational lattice examples.
use super::bytes;
use crate::{
    asymmetric::lattice_reference as l,
    errors::{ErrorCode, OperationError},
    operations::execution::{Budget, Elapsed},
};
use serde_json::{Value, json};
use zeroize::Zeroizing;

pub(super) fn supports(operation: &str) -> bool {
    l::Profile::from_operation(operation).is_some()
}

pub(super) fn execute(
    operation: &str,
    inputs: Value,
    parameters: Value,
    budget: Budget,
    _clock: Option<Elapsed<'_>>,
) -> Result<Value, OperationError> {
    let (profile, encrypt) = l::Profile::from_operation(operation).ok_or_else(|| {
        OperationError::new(
            ErrorCode::InvalidParameter,
            "unknown educational lattice operation",
        )
    })?;
    if budget.memory_bytes < l::SCRATCH_BYTES || budget.instructions < 1 {
        return Err(OperationError::new(
            ErrorCode::ResourceLimit,
            "educational lattice operation exceeds the invocation budget",
        ));
    }
    profile.validate_parameters(&parameters.to_string())?;
    let input = Zeroizing::new(bytes(&inputs["input"])?);
    let result = Zeroizing::new(if encrypt {
        l::encrypt(&input, &bytes(&inputs["public_key"])?, profile)?
    } else {
        let key = Zeroizing::new(bytes(&inputs["private_key"])?);
        l::decrypt(&input, &key, profile)?
    });
    Ok(json!({"output":{"base64":crate::encoding::base64_encode(&result)}}))
}
