//! Exact cipher profiles forward byte ports and private settings to Core.
use super::{bytes, field};
use crate::{
    ciphers::{
        aes_blocks, aes_modes, age, block_profiles, fpe, key_wrap, modern_aead, openssl, stream,
        xxtea,
    },
    errors::{ErrorCode, OperationError},
};
use serde_json::{Value, json};
use stream::reference::authentication;
pub(super) fn supports(op: &str) -> bool {
    stream::Profile::from_id(op).is_some()
        || matches!(
            op,
            "cipher.vernam"
                | "cipher.aes.encrypt_blocks"
                | "cipher.aes.decrypt_blocks"
                | age::DECRYPT
                | openssl::DECRYPT
                | fpe::ENCRYPT
                | fpe::DECRYPT
                | key_wrap::WRAP
                | key_wrap::UNWRAP
                | modern_aead::ENCRYPT
                | modern_aead::DECRYPT
                | modern_aead::SIV_ENCRYPT
                | modern_aead::SIV_DECRYPT
                | aes_modes::ENCRYPT
                | aes_modes::DECRYPT
                | block_profiles::ENCRYPT
                | block_profiles::DECRYPT
                | xxtea::ENCRYPT
                | xxtea::DECRYPT
                | stream::reference::ENCRYPT
                | stream::reference::DECRYPT
                | authentication::ENCRYPT
                | authentication::DECRYPT
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
    if matches!(op, fpe::ENCRYPT | fpe::DECRYPT) {
        let document = zeroize::Zeroizing::new(p.to_string());
        let settings = fpe::Settings::parse(&document)?;
        let input = zeroize::Zeroizing::new(field::<Vec<u16>>(&i, "input")?);
        let output = if op == fpe::ENCRYPT {
            fpe::encrypt(&input, &settings)?
        } else {
            fpe::decrypt(&input, &settings)?
        };
        return Ok(json!({"output":output}));
    }
    let input = bytes(&i["input"])?;
    if matches!(
        op,
        modern_aead::ENCRYPT
            | modern_aead::DECRYPT
            | modern_aead::SIV_ENCRYPT
            | modern_aead::SIV_DECRYPT
    ) {
        return authenticated(op, &input, &i, &p);
    }
    if matches!(op, authentication::ENCRYPT | authentication::DECRYPT) {
        let settings = authentication::Settings::parse(&p.to_string())?;
        return if op == authentication::ENCRYPT {
            let result = authentication::encrypt(&input, &settings)?;
            Ok(
                json!({"output":{"base64":crate::encoding::base64_encode(&result.ciphertext)},"tag":{"base64":crate::encoding::base64_encode(&result.tag)}}),
            )
        } else {
            let result = authentication::decrypt(&input, &bytes(&i["tag"])?, &settings)?;
            Ok(json!({"output":{"base64":crate::encoding::base64_encode(&result)}}))
        };
    }
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
    } else if matches!(op, stream::reference::ENCRYPT | stream::reference::DECRYPT) {
        let settings = stream::reference::Settings::parse(&p.to_string())?;
        if op == stream::reference::ENCRYPT {
            stream::reference::encrypt(&input, &settings)?
        } else {
            stream::reference::decrypt(&input, &settings)?
        }
    } else if matches!(op, aes_modes::ENCRYPT | aes_modes::DECRYPT) {
        let settings = aes_modes::Settings::parse(&p.to_string())?;
        if op == aes_modes::ENCRYPT {
            aes_modes::encrypt(&input, &settings)?
        } else {
            aes_modes::decrypt(&input, &settings)?
        }
    } else if matches!(op, key_wrap::WRAP | key_wrap::UNWRAP) {
        let document = zeroize::Zeroizing::new(p.to_string());
        let s = key_wrap::Settings::parse(&document)?;
        if op == key_wrap::WRAP {
            key_wrap::wrap(&input, &s)?
        } else {
            key_wrap::unwrap(&input, &s)?
        }
    } else if op == age::DECRYPT {
        let document = zeroize::Zeroizing::new(p.to_string());
        let settings = age::Settings::parse(&document)?;
        age::decrypt(&input, &settings)?
    } else if op == openssl::DECRYPT {
        let document = zeroize::Zeroizing::new(p.to_string());
        let settings = openssl::Settings::parse(&document)?;
        openssl::decrypt(&input, &settings)?
    } else if op == "cipher.vernam" {
        stream::vernam(&input, &bytes(&i["key"])?)?
    } else if matches!(op, xxtea::ENCRYPT | xxtea::DECRYPT) {
        let key = xxtea::parse_key(&p.to_string())?;
        if op == xxtea::ENCRYPT {
            xxtea::encrypt(&input, &key)?
        } else {
            xxtea::decrypt(&input, &key)?
        }
    } else if matches!(op, block_profiles::ENCRYPT | block_profiles::DECRYPT) {
        let settings = block_profiles::Settings::parse(&p.to_string())?;
        if op == block_profiles::ENCRYPT {
            block_profiles::encrypt(&input, &settings)?
        } else {
            block_profiles::decrypt(&input, &settings)?
        }
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

fn authenticated(op: &str, input: &[u8], i: &Value, p: &Value) -> Result<Value, OperationError> {
    let document = zeroize::Zeroizing::new(p.to_string());
    if matches!(op, modern_aead::SIV_ENCRYPT | modern_aead::SIV_DECRYPT) {
        let s = modern_aead::SivSettings::parse(&document)?;
        let components = field::<Vec<Vec<u8>>>(i, "aad_components")?;
        if op == modern_aead::SIV_ENCRYPT {
            let result = modern_aead::siv_encrypt(input, &components, &s)?;
            Ok(detached(result))
        } else {
            let plain = modern_aead::siv_decrypt(input, &components, &bytes(&i["tag"])?, &s)?;
            Ok(json!({"output":{"base64":crate::encoding::base64_encode(&plain)}}))
        }
    } else {
        let s = modern_aead::Settings::parse(&document)?;
        let aad = bytes(&i["aad"])?;
        if op == modern_aead::ENCRYPT {
            let result = modern_aead::encrypt(input, &aad, &s)?;
            Ok(detached(result))
        } else {
            let plain = modern_aead::decrypt(input, &aad, &bytes(&i["tag"])?, &s)?;
            Ok(json!({"output":{"base64":crate::encoding::base64_encode(&plain)}}))
        }
    }
}
fn detached(result: modern_aead::Detached) -> Value {
    json!({"output":{"base64":crate::encoding::base64_encode(&result.ciphertext)},"tag":{"base64":crate::encoding::base64_encode(&result.tag)}})
}
