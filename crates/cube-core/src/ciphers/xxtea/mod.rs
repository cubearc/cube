//! Corrected Block TEA over one complete little-endian word sequence.
//!
//! Educational unauthenticated interoperability primitive. No length suffix,
//! string normalization, padding, nonce, mode or key derivation is performed.
mod catalog;
mod kernel;
use crate::errors::{ErrorCode, OperationError};
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;
use schemars::JsonSchema;
use serde::Deserialize;
use zeroize::{Zeroize, Zeroizing};

pub const MAX_INPUT: usize = 64 * 1024;
pub const MAX_PARAMETERS: usize = 4096;
pub const ENCRYPT: &str = "cipher.xxtea.encrypt_words";
pub const DECRYPT: &str = "cipher.xxtea.decrypt_words";

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Parameters {
    #[schemars(regex(pattern = "^[0-9A-Fa-f]{32}$"))]
    key_hex: String,
}
impl Drop for Parameters {
    fn drop(&mut self) {
        self.key_hex.zeroize();
    }
}
/// Validated exact 128-bit key, with owned bytes wiped on drop.
/// Borrowed, parser, SDK and OS copies are outside that guarantee.
pub struct Key(Zeroizing<[u8; 16]>);
impl std::fmt::Debug for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Key([REDACTED])")
    }
}
impl Key {
    /// Validate and own exactly sixteen raw key bytes without normalization.
    pub fn new(bytes: &[u8]) -> Result<Self, OperationError> {
        let exact = bytes.try_into().map_err(|_| invalid_key())?;
        Ok(Self(Zeroizing::new(exact)))
    }
}
fn invalid_key() -> OperationError {
    OperationError::new(
        ErrorCode::InvalidParameter,
        "XXTEA requires exactly sixteen key bytes",
    )
    .parameter("key_hex")
}
/// Parse bounded private JSON, rejecting unknown/duplicate fields and malformed
/// keys without copying the private document into error messages.
pub fn parse_key(document: &str) -> Result<Key, OperationError> {
    if document.len() > MAX_PARAMETERS {
        return Err(OperationError::new(
            ErrorCode::ResourceLimit,
            "XXTEA parameters exceed 4096 bytes",
        ));
    }
    let p: Parameters = serde_json::from_str(document).map_err(|_| invalid_key())?;
    if p.key_hex.len() != 32 {
        return Err(invalid_key());
    }
    let mut key = Zeroizing::new([0u8; 16]);
    hex::decode_to_slice(&p.key_hex, &mut key[..]).map_err(|_| invalid_key())?;
    Ok(Key(key))
}
/// Encrypt one whole little-endian sequence of 2..=16384 32-bit words.
/// Empty input is identity with a valid key; a single word is rejected.
pub fn encrypt(data: &[u8], key: &Key) -> Result<Vec<u8>, OperationError> {
    transform(data, key, false)
}
/// Decrypt a complete word sequence. Success does not verify key or integrity.
pub fn decrypt(data: &[u8], key: &Key) -> Result<Vec<u8>, OperationError> {
    transform(data, key, true)
}
fn transform(data: &[u8], key: &Key, inverse: bool) -> Result<Vec<u8>, OperationError> {
    if data.len() > MAX_INPUT {
        return Err(OperationError::new(
            ErrorCode::ResourceLimit,
            "XXTEA input exceeds 64 KiB",
        ));
    }
    if !data.is_empty() && (data.len() < 8 || !data.len().is_multiple_of(4)) {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            "XXTEA requires at least two whole 32-bit words; no padding",
        ));
    }
    Ok(kernel::transform(data, &key.0[..], inverse))
}
