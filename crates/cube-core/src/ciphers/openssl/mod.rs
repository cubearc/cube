//! Explicit decrypt-only OpenSSL Salted__ interoperability, without authentication.
mod catalog;
use crate::{
    ciphers::aes_modes,
    errors::{ErrorCode, OperationError},
};
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;
use schemars::JsonSchema;
use serde::Deserialize;
use zeroize::{Zeroize, Zeroizing};
pub const DECRYPT: &str = "cipher.openssl.decrypt";
pub const MAX_INPUT: usize = 16 * 1024 * 1024;
pub const MAX_PARAMETERS: usize = 4096;
pub const MAX_PASSWORD: usize = 1024;
pub const MAX_ITERATIONS: u32 = 100_000;
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema)]
pub enum Profile {
    #[serde(rename = "aes128_cbc_pkcs7_evp_md5")]
    Aes128Md5,
    #[serde(rename = "aes256_cbc_pkcs7_evp_md5")]
    Aes256Md5,
    #[serde(rename = "aes128_cbc_pkcs7_evp_sha256")]
    Aes128Sha256,
    #[serde(rename = "aes256_cbc_pkcs7_evp_sha256")]
    Aes256Sha256,
    #[serde(rename = "aes128_cbc_pkcs7_pbkdf2_sha256_salt8")]
    Aes128Pbkdf8,
    #[serde(rename = "aes256_cbc_pkcs7_pbkdf2_sha256_salt8")]
    Aes256Pbkdf8,
    #[serde(rename = "aes128_cbc_pkcs7_pbkdf2_sha256_salt16")]
    Aes128Pbkdf16,
    #[serde(rename = "aes256_cbc_pkcs7_pbkdf2_sha256_salt16")]
    Aes256Pbkdf16,
}
impl Profile {
    pub const NAMES: &[&str] = &[
        "aes128_cbc_pkcs7_evp_md5",
        "aes256_cbc_pkcs7_evp_md5",
        "aes128_cbc_pkcs7_evp_sha256",
        "aes256_cbc_pkcs7_evp_sha256",
        "aes128_cbc_pkcs7_pbkdf2_sha256_salt8",
        "aes256_cbc_pkcs7_pbkdf2_sha256_salt8",
        "aes128_cbc_pkcs7_pbkdf2_sha256_salt16",
        "aes256_cbc_pkcs7_pbkdf2_sha256_salt16",
    ];
    fn key_bytes(self) -> usize {
        match self {
            Self::Aes128Md5 | Self::Aes128Sha256 | Self::Aes128Pbkdf8 | Self::Aes128Pbkdf16 => 16,
            _ => 32,
        }
    }
    fn salt_bytes(self) -> usize {
        match self {
            Self::Aes128Pbkdf16 | Self::Aes256Pbkdf16 => 16,
            _ => 8,
        }
    }
    fn pbkdf(self) -> bool {
        matches!(
            self,
            Self::Aes128Pbkdf8 | Self::Aes256Pbkdf8 | Self::Aes128Pbkdf16 | Self::Aes256Pbkdf16
        )
    }
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Parameters {
    profile: Profile,
    #[schemars(length(max = 2048), regex(pattern = "^(?:[0-9A-Fa-f]{2})*$"))]
    password_hex: String,
    #[serde(default, deserialize_with = "present_iterations")]
    #[schemars(with = "u32", range(min = 1, max = 100000))]
    iterations: Option<u32>,
}
impl Drop for Parameters {
    fn drop(&mut self) {
        self.password_hex.zeroize();
    }
}
/// Validated private binary password and explicit KDF profile. Debug is redacted.
pub struct Settings {
    profile: Profile,
    password: Zeroizing<Vec<u8>>,
    iterations: Option<u32>,
}
impl std::fmt::Debug for Settings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("OpenSslSettings { [REDACTED] }")
    }
}
impl Settings {
    /// Require a named profile and exact password bytes, up to1024 bytes. PBKDF2
    /// requires an explicit1..100000 iteration count; EVP rejects that field.
    pub fn parse(document: &str) -> Result<Self, OperationError> {
        if document.len() > MAX_PARAMETERS {
            return Err(limit());
        }
        let p: Parameters = serde_json::from_str(document).map_err(|_| invalid())?;
        if p.password_hex.len() > MAX_PASSWORD * 2
            || !p.password_hex.len().is_multiple_of(2)
            || p.profile.pbkdf() != p.iterations.is_some()
            || p.iterations.is_some_and(|n| n == 0 || n > MAX_ITERATIONS)
        {
            return Err(invalid());
        }
        let password = Zeroizing::new(hex::decode(&p.password_hex).map_err(|_| invalid())?);
        Ok(Self {
            profile: p.profile,
            password,
            iterations: p.iterations,
        })
    }
}
/// Decrypt a complete binary Salted__ file using exactly the caller's profile.
/// Padding success is not authentication: wrong credentials can yield plausible
/// plaintext. Malformed headers/lengths/padding fail uniformly with no output.
pub fn decrypt(input: &[u8], settings: &Settings) -> Result<Vec<u8>, OperationError> {
    if input.len() > MAX_INPUT {
        return Err(limit());
    }
    let start = 8 + settings.profile.salt_bytes();
    if input.len() < start + 16
        || !input.starts_with(b"Salted__")
        || !(input.len() - start).is_multiple_of(16)
    {
        return Err(failed());
    }
    let salt = &input[8..start];
    let key_bytes = settings.profile.key_bytes();
    let mut derived = Zeroizing::new(vec![0u8; key_bytes + 16]);
    if let Some(iterations) = settings.iterations {
        pbkdf2::pbkdf2_hmac::<sha2::Sha256>(&settings.password, salt, iterations, &mut derived);
    } else {
        use crate::ciphers::kdf::{LegacyDigest, evp_chain};
        let digest = if matches!(settings.profile, Profile::Aes128Md5 | Profile::Aes256Md5) {
            LegacyDigest::Md5
        } else {
            LegacyDigest::Sha256
        };
        evp_chain(digest, &settings.password, salt, 1, &mut derived);
    }
    let doc=Zeroizing::new(serde_json::json!({"mode":"cbc","padding":"pkcs7","key_hex":hex::encode(&derived[..key_bytes]),"iv_hex":hex::encode(&derived[key_bytes..])}).to_string());
    let s = aes_modes::Settings::parse(&doc).map_err(|_| failed())?;
    aes_modes::decrypt(&input[start..], &s).map_err(|_| failed())
}
fn invalid() -> OperationError {
    OperationError::new(
        ErrorCode::InvalidParameter,
        "invalid explicit OpenSSL profile, password or iteration count",
    )
}
fn limit() -> OperationError {
    OperationError::new(
        ErrorCode::ResourceLimit,
        "OpenSSL input or settings exceed the documented bound",
    )
}
fn failed() -> OperationError {
    OperationError::new(
        ErrorCode::InvalidInput,
        "OpenSSL legacy decryption failed (unauthenticated format)",
    )
}

fn present_iterations<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<u32>, D::Error> {
    u32::deserialize(deserializer).map(Some)
}
