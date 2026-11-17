//! Typed direct KEM adapter. Cryptographic operations live in the backend module.
use super::bytes;
use crate::{
    asymmetric::{alternate_kem, kyber, ml_kem},
    errors::{ErrorCode, OperationError},
    operations::execution::{Budget, Elapsed},
};
use serde_json::{Value, json};
use zeroize::Zeroizing;

pub(super) fn supports(operation: &str) -> bool {
    matches!(
        operation,
        ml_kem::KEYGEN | ml_kem::ENCAPSULATE | ml_kem::DECAPSULATE | kyber::DECAPSULATE
    ) || alternate_kem::Profile::for_operation(operation).is_some()
}

pub(super) fn execute(
    operation: &str,
    inputs: Value,
    parameters: Value,
    budget: Budget,
    _clock: Option<Elapsed<'_>>,
) -> Result<Value, OperationError> {
    let alternate = alternate_kem::Profile::for_operation(operation);
    let memory = if alternate.is_some() {
        alternate_kem::MEMORY_RESERVATION
    } else {
        ml_kem::MEMORY_RESERVATION
    };
    if budget.memory_bytes < memory || budget.instructions < 1 {
        return Err(OperationError::new(
            ErrorCode::ResourceLimit,
            "KEM invocation requires one work unit and its fixed memory reservation",
        ));
    }
    let settings = parameters.to_string();
    if let Some(profile) = alternate {
        profile.validate_parameters(&settings)?;
        let key = Zeroizing::new(bytes(&inputs["private_key"])?);
        let shared = alternate_kem::decapsulate(&bytes(&inputs["ciphertext"])?, &key, profile)?;
        return Ok(json!({"shared_secret":wire(&shared)}));
    }
    match operation {
        ml_kem::KEYGEN => {
            let keys = ml_kem::keygen(ml_kem::Profile::parse(&settings)?)?;
            Ok(json!({"public_key":wire(&keys.public_key),"private_key":wire(&keys.private_key)}))
        }
        ml_kem::ENCAPSULATE => {
            let result = ml_kem::encapsulate(
                &bytes(&inputs["public_key"])?,
                ml_kem::Profile::parse(&settings)?,
            )?;
            Ok(
                json!({"ciphertext":wire(&result.ciphertext),"shared_secret":wire(&result.shared_secret[..])}),
            )
        }
        ml_kem::DECAPSULATE | kyber::DECAPSULATE => {
            let key = Zeroizing::new(bytes(&inputs["private_key"])?);
            let ciphertext = bytes(&inputs["ciphertext"])?;
            let shared = if operation == kyber::DECAPSULATE {
                kyber::decapsulate(&ciphertext, &key, kyber::Profile::parse(&settings)?)?
            } else {
                ml_kem::decapsulate(&ciphertext, &key, ml_kem::Profile::parse(&settings)?)?
            };
            Ok(json!({"shared_secret":wire(&shared[..])}))
        }
        _ => Err(OperationError::new(
            ErrorCode::InvalidParameter,
            "unknown KEM operation",
        )),
    }
}

fn wire(data: &[u8]) -> Value {
    json!({"base64":crate::encoding::base64_encode(data)})
}
