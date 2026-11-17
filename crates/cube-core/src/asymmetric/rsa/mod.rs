//! RFC8017 OAEP and explicit legacy PKCS1v1.5 implicit rejection.
mod catalog;
#[cfg(test)]
mod tests;
use super::{entropy::SystemRng, keys};
use crate::errors::{ErrorCode, OperationError};
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;
use rand_core::TryCryptoRng;
use schemars::JsonSchema;
use serde::Deserialize;
use sha2::Sha256;
use zeroize::Zeroizing;
pub const ENCRYPT: &str = "cipher.rsa.oaep.encrypt";
pub const DECRYPT: &str = "cipher.rsa.oaep.decrypt";
pub const LEGACY_DECRYPT: &str = "cipher.rsa.pkcs1_v1_5.decrypt";
pub const MAX_PARAMETERS: usize = 10 * 1024;
pub const MAX_LABEL: usize = 4096;
pub const MAX_CIPHERTEXT: usize = 1024;
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema)]
pub enum OaepProfile {
    #[serde(rename = "rsa_oaep_sha256_mgf1_sha256_cfrg10")]
    Sha256,
}
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema)]
pub enum LegacyProfile {
    #[serde(rename = "rsaes_pkcs1_v1_5_implicit_rejection_cfrg10")]
    ImplicitRejection,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct PublicParameters {
    profile: OaepProfile,
    key_format: keys::PublicFormat,
    #[serde(default)]
    #[schemars(length(max = 8192), regex(pattern = "^(?:[0-9A-Fa-f]{2})*$"))]
    label_hex: String,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct PrivateParameters {
    profile: OaepProfile,
    key_format: keys::PrivateFormat,
    #[serde(default)]
    #[schemars(length(max = 8192), regex(pattern = "^(?:[0-9A-Fa-f]{2})*$"))]
    label_hex: String,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct LegacyParameters {
    profile: LegacyProfile,
    key_format: keys::PrivateFormat,
}
/// Parsed SPKI public key and explicit OAEP label.
pub struct EncryptionSettings {
    key: rsa::PublicKey,
    label: Vec<u8>,
}
/// Parsed PKCS8 private key and explicit OAEP label.
pub struct DecryptionSettings {
    key: rsa::PrivateKey,
    label: Vec<u8>,
}
/// Explicit legacy key; successful output includes synthetic implicit rejection.
pub struct LegacySettings {
    key: rsa::PrivateKey,
}
macro_rules! redact {($($t:ty),*)=>{$(impl std::fmt::Debug for $t{fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{f.write_str("RsaSettings { [REDACTED] }")}})*};}
redact!(EncryptionSettings, DecryptionSettings, LegacySettings);
fn parameters<T: serde::de::DeserializeOwned>(document: &str) -> Result<T, OperationError> {
    if document.len() > MAX_PARAMETERS {
        return Err(limit());
    }
    serde_json::from_str(document).map_err(|_| keys::invalid())
}
fn decode(text: &str, max: usize) -> Result<Zeroizing<Vec<u8>>, OperationError> {
    if text.len() > max * 2 {
        return Err(limit());
    }
    let mut result = Zeroizing::new(vec![0; text.len() / 2]);
    hex::decode_to_slice(text, &mut result[..]).map_err(|_| keys::invalid())?;
    Ok(result)
}
impl EncryptionSettings {
    /// Require RSA-OAEP SHA256/MGF1-SHA256 and SPKI DER/PEM; label defaults empty.
    pub fn parse(document: &str, encoded_key: &[u8]) -> Result<Self, OperationError> {
        let p: PublicParameters = parameters(document)?;
        let _ = p.profile;
        let key = keys::rsa_public(encoded_key, p.key_format)?;
        let label = decode(&p.label_hex, MAX_LABEL)?.to_vec();
        Ok(Self { key, label })
    }
}
impl DecryptionSettings {
    /// Require the same OAEP hash/label and a canonical unencrypted PKCS8 key.
    pub fn parse(document: &str, encoded_key: &[u8]) -> Result<Self, OperationError> {
        let p: PrivateParameters = parameters(document)?;
        let _ = p.profile;
        let key = keys::rsa_private(encoded_key, p.key_format)?;
        let label = decode(&p.label_hex, MAX_LABEL)?.to_vec();
        Ok(Self { key, label })
    }
}
impl LegacySettings {
    /// Require the named legacy implicit-rejection profile; no OAEP fallback.
    pub fn parse(document: &str, encoded_key: &[u8]) -> Result<Self, OperationError> {
        let p: LegacyParameters = parameters(document)?;
        let _ = p.profile;
        let key = keys::rsa_private(encoded_key, p.key_format)?;
        Ok(Self { key })
    }
}
/// Encrypt at most k−66 bytes with fresh OS/browser CSPRNG seed material.
/// No production API accepts deterministic coins or generates an RSA key.
pub fn encrypt(input: &[u8], s: &EncryptionSettings) -> Result<Vec<u8>, OperationError> {
    encrypt_with_rng(input, s, &mut SystemRng::default())
}
fn encrypt_with_rng(
    input: &[u8],
    s: &EncryptionSettings,
    rng: &mut impl TryCryptoRng,
) -> Result<Vec<u8>, OperationError> {
    if input.len() > keys::width(&s.key) - 66 {
        return Err(limit());
    }
    s.key
        .encrypt(
            rng,
            rsa::Oaep::<Sha256>::new_with_label(s.label.clone()),
            input,
        )
        .map_err(|_| failed())
}
/// Decrypt exactly k ciphertext bytes with fresh private-operation blinding.
/// Wrong label, padding or representative returns the same error and no output.
pub fn decrypt(input: &[u8], s: &DecryptionSettings) -> Result<Vec<u8>, OperationError> {
    decrypt_with_rng(input, s, &mut SystemRng::default())
}
fn decrypt_with_rng(
    input: &[u8],
    s: &DecryptionSettings,
    rng: &mut impl TryCryptoRng,
) -> Result<Vec<u8>, OperationError> {
    if input.len() > MAX_CIPHERTEXT {
        return Err(limit());
    }
    if input.len() != keys::width(&s.key) {
        return Err(failed());
    }
    s.key
        .decrypt_blinded(
            rng,
            rsa::Oaep::<Sha256>::new_with_label(s.label.clone()),
            input,
        )
        .map_err(|_| failed())
}
/// Legacy decrypt-only implicit rejection: invalid padding yields deterministic
/// synthetic bytes. Success never establishes valid padding or authenticity.
/// Authenticate the enclosing protocol before using any returned bytes.
pub fn legacy_decrypt(input: &[u8], s: &LegacySettings) -> Result<Vec<u8>, OperationError> {
    legacy_decrypt_with_rng(input, s, &mut SystemRng::default())
}
fn legacy_decrypt_with_rng(
    input: &[u8],
    s: &LegacySettings,
    rng: &mut impl TryCryptoRng,
) -> Result<Vec<u8>, OperationError> {
    if input.len() > MAX_CIPHERTEXT {
        return Err(limit());
    }
    if input.len() != keys::width(&s.key) {
        return Err(failed());
    }
    s.key
        .pkcs1v15_decrypt_with_rng(rng, input)
        .map_err(|_| failed())
}
fn failed() -> OperationError {
    OperationError::new(ErrorCode::InvalidInput, "RSA operation failed")
}
fn limit() -> OperationError {
    OperationError::new(
        ErrorCode::ResourceLimit,
        "RSA input or settings exceed the selected profile bounds",
    )
}
