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
    if crate::ciphers::kdf::supports(op) || crate::asymmetric::historical_public_key::supports(op) {
        return true;
    }
    matches!(
        op,
        crate::asymmetric::group_encryption::ELGAMAL_ENCRYPT
            | crate::asymmetric::group_encryption::ELGAMAL_DECRYPT
            | crate::asymmetric::group_encryption::CRAMER_SHOUP_ENCRYPT
            | crate::asymmetric::group_encryption::CRAMER_SHOUP_DECRYPT
            | crate::asymmetric::ies::ENCRYPT
            | crate::asymmetric::ies::DECRYPT
            | crate::asymmetric::sm2::ENCRYPT
            | crate::asymmetric::sm2::DECRYPT
            | crate::asymmetric::rsa::ENCRYPT
            | crate::asymmetric::rsa::DECRYPT
            | crate::asymmetric::rsa::LEGACY_DECRYPT
    ) || op == crate::asymmetric::dh::AGREE
        || op == crate::asymmetric::ecdh::AGREE
        || stream::Profile::from_id(op).is_some()
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
    if crate::asymmetric::historical_public_key::supports(op) {
        use crate::asymmetric::historical_public_key as h;
        let key = zeroize::Zeroizing::new(bytes(&i["key"])?);
        let tokens = zeroize::Zeroizing::new(field::<Vec<String>>(&i, "tokens")?);
        let p = h::bounded_parameters(p, _budget)?;
        return h::execute(op, &tokens, &key, &p.to_string());
    }
    {
        use crate::asymmetric::group_encryption as g;
        if matches!(
            op,
            g::ELGAMAL_ENCRYPT
                | g::ELGAMAL_DECRYPT
                | g::CRAMER_SHOUP_ENCRYPT
                | g::CRAMER_SHOUP_DECRYPT
        ) {
            let document = p.to_string();
            let input = zeroize::Zeroizing::new(bytes(&i["input"])?);
            let key = zeroize::Zeroizing::new(bytes(&i["key"])?);
            let output = zeroize::Zeroizing::new(match op {
                g::ELGAMAL_ENCRYPT => g::elgamal_encrypt(
                    &input,
                    &g::ElGamalEncryptionSettings::parse(&document, &key)?,
                )?,
                g::ELGAMAL_DECRYPT => g::elgamal_decrypt(
                    &input,
                    &g::ElGamalDecryptionSettings::parse(&document, &key)?,
                )?,
                g::CRAMER_SHOUP_ENCRYPT => g::cramer_shoup_encrypt(
                    &input,
                    &g::CramerShoupEncryptionSettings::parse(&document, &key)?,
                )?,
                _ => g::cramer_shoup_decrypt(
                    &input,
                    &g::CramerShoupDecryptionSettings::parse(&document, &key)?,
                )?,
            });
            return Ok(json!({"output":{"base64":crate::encoding::base64_encode(&output)}}));
        }
    }
    if matches!(
        op,
        crate::asymmetric::ies::ENCRYPT | crate::asymmetric::ies::DECRYPT
    ) {
        use crate::asymmetric::ies;
        let key = zeroize::Zeroizing::new(bytes(&i["key"])?);
        let input = zeroize::Zeroizing::new(bytes(&i["input"])?);
        let output = zeroize::Zeroizing::new(if op == ies::ENCRYPT {
            ies::encrypt(
                &input,
                &ies::EncryptionSettings::parse(&p.to_string(), &key)?,
            )?
        } else {
            ies::decrypt(
                &input,
                &ies::DecryptionSettings::parse(&p.to_string(), &key)?,
            )?
        });
        return Ok(json!({"output":{"base64":crate::encoding::base64_encode(&output)}}));
    }
    if matches!(
        op,
        crate::asymmetric::sm2::ENCRYPT | crate::asymmetric::sm2::DECRYPT
    ) {
        use crate::asymmetric::sm2;
        let key = zeroize::Zeroizing::new(bytes(&i["key"])?);
        let input = zeroize::Zeroizing::new(bytes(&i["input"])?);
        let output = zeroize::Zeroizing::new(if op == sm2::ENCRYPT {
            sm2::encrypt(
                &input,
                &sm2::EncryptionSettings::parse(&p.to_string(), &key)?,
            )?
        } else {
            sm2::decrypt(
                &input,
                &sm2::DecryptionSettings::parse(&p.to_string(), &key)?,
            )?
        });
        return Ok(json!({"output":{"base64":crate::encoding::base64_encode(&output)}}));
    }
    if matches!(
        op,
        crate::asymmetric::rsa::ENCRYPT
            | crate::asymmetric::rsa::DECRYPT
            | crate::asymmetric::rsa::LEGACY_DECRYPT
    ) {
        use crate::asymmetric::rsa;
        let document = zeroize::Zeroizing::new(p.to_string());
        let input = zeroize::Zeroizing::new(bytes(&i["input"])?);
        let key = zeroize::Zeroizing::new(bytes(&i["key"])?);
        let output = zeroize::Zeroizing::new(match op {
            rsa::ENCRYPT => {
                rsa::encrypt(&input, &rsa::EncryptionSettings::parse(&document, &key)?)?
            }
            rsa::DECRYPT => {
                rsa::decrypt(&input, &rsa::DecryptionSettings::parse(&document, &key)?)?
            }
            _ => rsa::legacy_decrypt(&input, &rsa::LegacySettings::parse(&document, &key)?)?,
        });
        return Ok(json!({"output":{"base64":crate::encoding::base64_encode(&output)}}));
    }
    if op == crate::asymmetric::dh::AGREE {
        use crate::asymmetric::dh;
        let key = zeroize::Zeroizing::new(bytes(&i["key"])?);
        let settings = dh::Settings::parse(&p.to_string(), &key)?;
        let output = zeroize::Zeroizing::new(dh::agree(&bytes(&i["input"])?, &settings)?);
        return Ok(json!({"output":{"base64":crate::encoding::base64_encode(&output)}}));
    }
    if op == crate::asymmetric::ecdh::AGREE {
        let document = zeroize::Zeroizing::new(p.to_string());
        let settings = crate::asymmetric::ecdh::Settings::parse(&document)?;
        let output = crate::asymmetric::ecdh::agree(&bytes(&i["input"])?, &settings)?;
        return Ok(json!({"output":{"base64":crate::encoding::base64_encode(&output)}}));
    }
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
    if crate::ciphers::kdf::supports(op) {
        return derive_key(op, &i, p);
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

fn derive_key(operation: &str, inputs: &Value, parameters: Value) -> Result<Value, OperationError> {
    use crate::ciphers::kdf;
    let invalid = || OperationError::new(ErrorCode::InvalidParameter, "invalid KDF settings");
    let salt = zeroize::Zeroizing::new(bytes(&inputs["salt"])?);
    let secret = zeroize::Zeroizing::new(bytes(
        &inputs[if operation == kdf::HKDF {
            "ikm"
        } else {
            "password"
        }],
    )?);
    let byte_value = |value: &[u8]| json!({"base64": crate::encoding::base64_encode(value)});
    let output = match operation {
        kdf::PBKDF2 => kdf::pbkdf2(
            &secret,
            &salt,
            serde_json::from_value(parameters).map_err(|_| invalid())?,
        )?,
        kdf::HKDF => {
            let info = zeroize::Zeroizing::new(bytes(&inputs["info"])?);
            kdf::hkdf(
                &secret,
                &salt,
                &info,
                serde_json::from_value(parameters).map_err(|_| invalid())?,
            )?
        }
        kdf::SCRYPT => kdf::scrypt(
            &secret,
            &salt,
            serde_json::from_value(parameters).map_err(|_| invalid())?,
        )?,
        kdf::ARGON2ID => kdf::argon2id(
            &secret,
            &salt,
            serde_json::from_value(parameters).map_err(|_| invalid())?,
        )?,
        kdf::EVP_BYTES_TO_KEY => {
            let result = kdf::evp_bytes_to_key(
                &secret,
                &salt,
                serde_json::from_value(parameters).map_err(|_| invalid())?,
            )?;
            return Ok(json!({"key": byte_value(result.key()), "iv": byte_value(result.iv())}));
        }
        _ => return Err(invalid()),
    };
    Ok(json!({"output": byte_value(output.as_slice())}))
}
