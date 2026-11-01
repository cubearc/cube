//! Raw cipher profiles forward exact bytes and explicit key material to Core.
use super::{bytes, field};
use crate::{
    ciphers::{aes_blocks, stream},
    errors::{ErrorCode, OperationError},
};
use serde_json::{Value, json};
pub(super) fn supports(op: &str) -> bool {
    stream::Profile::from_id(op).is_some()
        || matches!(
            op,
            "cipher.vernam"
                | "cipher.aes.encrypt_blocks"
                | "cipher.aes.decrypt_blocks"
                | "encoding.base58check.decode"
        )
}
pub(super) fn execute(
    op: &str,
    i: Value,
    p: Value,
    _budget: crate::operations::execution::Budget,
    _clock: Option<crate::operations::execution::Elapsed<'_>>,
) -> Result<Value, OperationError> {
    if op == "encoding.base58check.decode" {
        return Ok(
            json!({"output":crate::encoding::base58check::decode(&field::<String>(&i,"input")?)?}),
        );
    }
    let input = bytes(&i["input"])?;
    let output = if let Some(profile) = stream::Profile::from_id(op) {
        let counter = p["counter"]
            .as_u64()
            .or_else(|| p["counter"].as_str()?.parse().ok())
            .ok_or_else(invalid)?;
        stream::transform(
            profile,
            &input,
            &bytes(&i["key"])?,
            &field::<Vec<u8>>(&p, "nonce")?,
            counter,
        )?
    } else if op == "cipher.vernam" {
        stream::vernam(&input, &bytes(&i["key"])?)?
    } else {
        let key = aes_blocks::parse_key(&p.to_string())?;
        if op == "cipher.aes.encrypt_blocks" {
            aes_blocks::encrypt(&input, key.as_ref())?
        } else {
            aes_blocks::decrypt(&input, key.as_ref())?
        }
    };
    Ok(json!({"output":{"base64":crate::encoding::base64_encode(&output)}}))
}
fn invalid() -> OperationError {
    OperationError::new(
        ErrorCode::InvalidParameter,
        "use the exact key, nonce and counter fields",
    )
}
